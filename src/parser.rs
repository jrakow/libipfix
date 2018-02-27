use information_element::*;
use nom::*;
use structs::*;

// TODO insert padding

#[cfg_attr(rustfmt, rustfmt_skip)]
named!(pub message_parser<Message>,
	complete!(do_parse!(
		message_header : message_header_parser >>
		sets : length_value!(
			value!(message_header.length - MESSAGE_HEADER_LENGTH),
			many0!(
				complete!(do_parse!(
					set_header : verify!(set_header_parser, |header : Set_Header| header.length > SET_HEADER_LENGTH) >>
					data : length_data!(value!(set_header.length - SET_HEADER_LENGTH)) >>
					(set_header, data)
				))
			)
		) >>
		(Message{ header : message_header, sets : sets })
	))
);

#[cfg_attr(rustfmt, rustfmt_skip)]
named!(
	message_header_parser<Message_Header>,
	do_parse!(
		/* version_number */ tag!([0x00, 0x0a]) >>
		length : verify!(be_u16, |length| length >= MESSAGE_HEADER_LENGTH) >>
		export_time : be_u32 >>
		sequence_number : be_u32 >>
		observation_domain_id : be_u32 >>
		(Message_Header {
				version_number : 0x000au16,
				length,
				export_time,
				sequence_number,
				observation_domain_id,
		})
	)
);

#[cfg_attr(rustfmt, rustfmt_skip)]
named!(
	set_header_parser<Set_Header>,
	do_parse!(
		set_id : be_u16 >>
		length : be_u16 >>
		(Set_Header{ set_id, length })
	)
);

#[cfg_attr(rustfmt, rustfmt_skip)]
named!(
	field_specifier_parser<Field_Specifier>,
	do_parse!(
		information_element_id : be_u16 >>
		field_length : be_u16 >>
		enterprise_number : cond!(
			information_element_id & 0x8000 != 0x0000,
			be_u32
		) >>
		(Field_Specifier{
			information_element_id : information_element_id & 0x7fff,
			field_length,
			enterprise_number,
		})
	)
);

#[cfg_attr(rustfmt, rustfmt_skip)]
named_args!(
	pub template_records_parser(set_header : Set_Header)<Vec<Template_Record>>,
	length_value!(
		value!(set_header.length - SET_HEADER_LENGTH),
		many1!(complete!(
			call!(template_record_parser, set_header.set_id == 3))
		)
	)
);

pub fn data_records_parser<'input>(
	input : &'input [u8],
	records_length : u16,
	template : &Template_Record,
) -> IResult<&'input [u8], Vec<Data_Record>> {
	let (rest_after_records, mut input) = take!(input, records_length as usize)?;

	// do-while loop
	// at least 1 data record
	let mut records = Vec::<Data_Record>::default();
	loop {
		let (rest, record) = data_record_parser(input, template)?;
		input = rest;
		records.push(record);

		if input.is_empty() {
			break;
		}
	}
	Ok((rest_after_records, records))
}

pub mod error_kind {
	use nom::ErrorKind;

	#[repr(u32)]
	enum semantic_error {
		INFORMATION_ELEMENT_UNKNOWN,
		BOOL_INVALID,
		STRING_NOT_UTF8,
	}

	pub const INFORMATION_ELEMENT_UNKNOWN : ErrorKind<u32> =
		ErrorKind::Custom(semantic_error::INFORMATION_ELEMENT_UNKNOWN as u32);
	pub const BOOL_INVALID : ErrorKind<u32> =
		ErrorKind::Custom(semantic_error::BOOL_INVALID as u32);
	pub const STRING_NOT_UTF8 : ErrorKind<u32> =
		ErrorKind::Custom(semantic_error::STRING_NOT_UTF8 as u32);
}

fn data_record_parser<'input>(
	input : &'input [u8],
	template : &Template_Record,
) -> IResult<&'input [u8], Data_Record> {
	let mut input = input;
	let mut fields = Vec::<Data_Value>::default();

	for field in &template.fields {
		let information_element = lookup(field.information_element_id).ok_or_else(|| Err::Error(
			error_position!(input, error_kind::INFORMATION_ELEMENT_UNKNOWN),
		))?; // return if Err

		match information_element_parser(
			input,
			information_element.abstract_data_type,
			field.field_length,
		) {
			Err(err) => return Err(err),
			Ok((rest, field)) => {
				input = rest;
				fields.push(field);
			}
		}
	}
	Ok((input, Data_Record { fields }))
}

fn information_element_parser(
	input : &[u8],
	abstract_data_type : Abstract_Data_Type,
	length : u16,
) -> IResult<&[u8], Data_Value> {
	use structs::Abstract_Data_Type::*;

	match abstract_data_type {
		unsigned8 | unsigned16 | unsigned32 | unsigned64 => match length {
			1 => map!(input, be_u8, Data_Value::unsigned8),
			2 => map!(input, be_u16, Data_Value::unsigned16),
			4 => map!(input, be_u32, Data_Value::unsigned32),
			8 => map!(input, be_u64, Data_Value::unsigned64),
			_ => panic!(),
		},
		signed8 | signed16 | signed32 | signed64 => match length {
			1 => map!(input, be_i8, Data_Value::signed8),
			2 => map!(input, be_i16, Data_Value::signed16),
			4 => map!(input, be_i32, Data_Value::signed32),
			8 => map!(input, be_i64, Data_Value::signed64),
			_ => panic!(),
		},
		float32 | float64 => match length {
			4 => map!(input, be_f32, Data_Value::float32),
			8 => map!(input, be_f64, Data_Value::float64),
			_ => panic!(),
		},
		boolean => match length {
			1 => match be_u8(input) {
				Ok((rest, 1u8)) => Ok((rest, Data_Value::boolean(true))),
				Ok((rest, 2u8)) => Ok((rest, Data_Value::boolean(false))),
				Ok(_) => Err(Err::Error(error_position!(input, error_kind::BOOL_INVALID))),
				Err(e) => Err(e),
			},
			_ => panic!(),
		},
		macAddress => match length {
			6 => map!(input, take!(6), |slice| Data_Value::macAddress(
				slice.to_vec()
			)),
			_ => panic!(),
		},
		octetArray => match length {
			0xffffu16 => information_element_variable_length_parser(input),
			_ => take!(input, length),
		}.map(|(input, slice)| (input, Data_Value::octetArray(slice.to_vec()))),
		string => {
			// try cast to utf8
			let string_result = match length {
				0xffffu16 => information_element_variable_length_parser(input),
				_ => take!(input, length),
			}.map(|(input, slice)| (input, String::from_utf8(slice.to_vec())));
			match string_result {
				Ok((input, Ok(s))) => Ok((input, Data_Value::string(s))),
				// cast fail is semantic error
				Ok((input, Err(_))) => Err(Err::Error(error_position!(
					input,
					error_kind::STRING_NOT_UTF8
				))),
				Err(e) => Err(e),
			}
		}
		dateTimeSeconds => match length {
			4 => map!(input, be_u32, Data_Value::dateTimeSeconds),
			_ => panic!(),
		},
		dateTimeMilliseconds => match length {
			8 => map!(input, be_u64, Data_Value::dateTimeMilliseconds),
			_ => panic!(),
		},
		dateTimeMicroseconds => match length {
			8 => map!(input, tuple!(be_u32, be_u32), |(seconds, fraction)| {
				Data_Value::dateTimeMicroseconds(seconds, fraction & 0xFFFF_F800) // ignore lower 11 Bit of fraction
			}),
			_ => panic!(),
		},
		dateTimeNanoseconds => match length {
			8 => map!(input, tuple!(be_u32, be_u32), |(seconds, fraction)| {
				Data_Value::dateTimeNanoseconds(seconds, fraction)
			}),
			_ => panic!(),
		},
		ipv4Address => match length {
			4 => map!(input, take!(4), |slice| Data_Value::ipv4Address(
				slice.to_vec()
			)),
			_ => panic!(),
		},
		ipv6Address => match length {
			16 => map!(input, take!(16), |slice| Data_Value::ipv6Address(
				slice.to_vec()
			)),
			_ => panic!(),
		},
		basicList => panic!("type not implemented"),
		subTemplateList => panic!("type not implemented"),
		subTemplateMultiList => panic!("type not implemented"),
	}
}

#[cfg_attr(rustfmt, rustfmt_skip)]
named!(
	information_element_variable_length_parser<&[u8]>,
	alt_complete!(
		length_data!(verify!(be_u8, |length| length < 255)) |
		preceded!(tag!([255u8]), length_data!(be_u16))
	)
);

#[cfg_attr(rustfmt, rustfmt_skip)]
named_args!(
	template_record_parser(is_options_template : bool)<Template_Record>,
	do_parse!(
		header : call!(template_record_header_parser, is_options_template) >>
		scope_fields : count!(
			field_specifier_parser,
			header.scope_field_count as usize) >>
		fields : count!(
			field_specifier_parser,
			header.field_count as usize - header.scope_field_count as usize) >>
		(Template_Record{ header, fields, scope_fields })
	)
);

#[cfg_attr(rustfmt, rustfmt_skip)]
named_args!(
	template_record_header_parser(is_options_template : bool)<Template_Record_Header>,
	do_parse!(
		template_id : be_u16 >>
		field_count : be_u16 >>
		scope_field_count : map!(
				cond!(
					is_options_template && field_count != 0,// withdrawal has no scope_field_count
					be_u16
				),
				|option| option.unwrap_or(0)
		) >>
		(Template_Record_Header{ template_id, field_count, scope_field_count })
	)
);

#[cfg(test)]
mod tests {
	use super::*;
	use nom;

	#[test]
	fn message_parser_test() {
		let data : [u8; 56] = [
			0x00, 0x0a, 0x00, 56, // version, length
			0x5A, 0x88, 0x08, 0x2E, // time
			0xfe, 0xdc, 0xba, 0x98, // seq num
			0xde, 0xad, 0xbe, 0xef, // domain id
			0x00, 0x02, 0x00, 20, // set id, set length
			// set data
			0x01, 0x23, 0x45, 0x67,
			0x89, 0xab, 0xcd, 0xef,
			0x01, 0x23, 0x45, 0x67,
			0x89, 0xab, 0xcd, 0xef,
			0x00, 0x03, 0x00, 20, // set id, set length
			// set data
			0x89, 0xab, 0xcd, 0xef,
			0x01, 0x23, 0x45, 0x67,
			0x89, 0xab, 0xcd, 0xef,
			0x01, 0x23, 0x45, 0x67,
		];
		let res = Message {
			header : Message_Header {
				version_number : 0x000au16,
				length : 56,
				export_time : 0x5a88082eu32,
				sequence_number : 0xfedcba98u32,
				observation_domain_id : 0xdeadbeefu32,
			},
			sets : vec![
				(
					Set_Header {
						set_id : 2u16,
						length : 20u16,
					},
					&[
						0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef, 0x01, 0x23, 0x45, 0x67,
						0x89, 0xab, 0xcd, 0xef,
					],
				),
				(
					Set_Header {
						set_id : 3u16,
						length : 20u16,
					},
					&[
						0x89, 0xab, 0xcd, 0xef, 0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef,
						0x01, 0x23, 0x45, 0x67,
					],
				),
			],
		};
		assert_eq!(message_parser(&data), Ok((&[][..], res)));
	}

	#[test]
	fn message_header_parser_test() {
		let data : [u8; 16] = [
			0x00, 0x0a, 0x00, 56, // version, length
			0x5A, 0x88, 0x08, 0x2E, // time
			0xfe, 0xdc, 0xba, 0x98, // seq num
			0xde, 0xad, 0xbe, 0xef, // domain id
		];
		let res = Message_Header {
			version_number : 0x000au16,
			length : 56,
			export_time : 0x5a88082eu32,
			sequence_number : 0xfedcba98u32,
			observation_domain_id : 0xdeadbeefu32,
		};
		assert_eq!(message_header_parser(&data), Ok((&b""[..], res)));
	}

	#[test]
	fn set_header_parser_test() {
		let data : [u8; 4] = [
			0x00, 0x02, 0x00, 4 // set id, set length
		];
		let res = Set_Header {
			set_id : 2u16,
			length : 4u16,
		};
		assert_eq!(set_header_parser(&data), Ok((&[][..], res)));
	}

	#[test]
	fn template_records_parser_test() {
		let set_header = Set_Header {
			set_id : 3,
			length : 50,
		};
		let data : &[u8] = &[
			1, 0, // template_id
			0, 10, // field_count
			0, 2, // scope_field_count

			1, 90, 0, 4, // scope field 0
			1, 47, 0, 2, // scope field 1
			1, 83, 0, 1, // field 0
			1, 88, 0, 1, // field 1
			1, 89, 0, 2, // field 2
			0, 210, 0, 6, // field 3
			1, 86, 0, 8, // field 4
			1, 87, 0, 8, // field 5
			1, 85, 255, 255, // field 6
			1, 84, 255, 255, // field 7
		];
		let res = vec![
			Template_Record {
				header : Template_Record_Header {
					template_id : 0x0100u16,
					field_count : 0x000au16,
					scope_field_count : 0x0002,
				},
				scope_fields : vec![
					Field_Specifier {
						information_element_id : 0x015au16,
						field_length : 4u16,
						enterprise_number : None,
					},
					Field_Specifier {
						information_element_id : 0x012fu16,
						field_length : 2u16,
						enterprise_number : None,
					},
				],
				fields : vec![
					Field_Specifier {
						information_element_id : 0x0153u16,
						field_length : 1u16,
						enterprise_number : None,
					},
					Field_Specifier {
						information_element_id : 0x0158u16,
						field_length : 1u16,
						enterprise_number : None,
					},
					Field_Specifier {
						information_element_id : 0x0159u16,
						field_length : 2u16,
						enterprise_number : None,
					},
					Field_Specifier {
						information_element_id : 0x00d2u16,
						field_length : 6u16,
						enterprise_number : None,
					},
					Field_Specifier {
						information_element_id : 0x0156u16,
						field_length : 8u16,
						enterprise_number : None,
					},
					Field_Specifier {
						information_element_id : 0x0157u16,
						field_length : 8u16,
						enterprise_number : None,
					},
					Field_Specifier {
						information_element_id : 0x0155u16,
						field_length : 0xffffu16,
						enterprise_number : None,
					},
					Field_Specifier {
						information_element_id : 0x0154u16,
						field_length : 0xffffu16,
						enterprise_number : None,
					},
				],
			},
		];
		assert_eq!(
			template_records_parser(&data, set_header),
			Ok((&[][..], res))
		);

		let set_header = Set_Header {
			set_id : 2,
			length : 20,
		};
		let data : &[u8] = &[
			0x01, 0x02, 0x00, 0x01, // template record header
			0x00, 0x07, 0x00, 0x02, // field 0

			0x01, 0x02, 0x00, 0x01, // template record header
			0x00, 0x07, 0x00, 0x02, // field 0
		];
		let res = vec![
			Template_Record {
				header : Template_Record_Header {
					template_id : 0x0102u16,
					field_count : 0x0001u16,
					scope_field_count : 0u16,
				},
				scope_fields : vec![],
				fields : vec![
					Field_Specifier {
						information_element_id : 7u16,
						field_length : 2u16,
						enterprise_number : None,
					},
				],
			},
			Template_Record {
				header : Template_Record_Header {
					template_id : 0x0102u16,
					field_count : 0x0001u16,
					scope_field_count : 0u16,
				},
				scope_fields : vec![],
				fields : vec![
					Field_Specifier {
						information_element_id : 7u16,
						field_length : 2u16,
						enterprise_number : None,
					},
				],
			},
		];
		assert_eq!(
			template_records_parser(&data, set_header),
			Ok((&[][..], res))
		);
	}

	#[test]
	fn data_records_parser_test() {
		let template = Template_Record {
			header : Template_Record_Header {
				template_id : 256,
				scope_field_count : 0,
				field_count : 1,
			},
			scope_fields : vec![],
			fields : vec![
				Field_Specifier {
					information_element_id : 210,
					field_length : 4,
					enterprise_number : None,
				},
			],
		};
		let data : &[u8] = &[0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77];
		assert_eq!(
			data_records_parser(data, 8, &template),
			Ok((
				&[][..],
				vec![
					Data_Record {
						fields : vec![Data_Value::octetArray(vec![0x00, 0x11, 0x22, 0x33])],
					},
					Data_Record {
						fields : vec![Data_Value::octetArray(vec![0x44, 0x55, 0x66, 0x77])],
					},
				]
			)),
		);
	}

	#[test]
	fn unsigned_integer_parser_test() {
		use Data_Value::*;

		let data : &[u8] = &[0x88, 0x99, 0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff];

		let res = unsigned8(0x88);
		let field_length = 1;
		for type_ in [
			Abstract_Data_Type::unsigned8,
			Abstract_Data_Type::unsigned16,
			Abstract_Data_Type::unsigned32,
			Abstract_Data_Type::unsigned64,
		].iter()
		{
			assert_eq!(
				information_element_parser(&data, *type_, field_length),
				Ok((&data[1..8], res.clone()))
			);
		}

		let res = unsigned16(0x8899);
		let field_length = 2;
		for type_ in [
			Abstract_Data_Type::unsigned16,
			Abstract_Data_Type::unsigned32,
			Abstract_Data_Type::unsigned64,
		].iter()
		{
			assert_eq!(
				information_element_parser(&data, *type_, field_length),
				Ok((&data[2..8], res.clone()))
			);
		}

		let res = unsigned32(0x8899aabb);
		let field_length = 4;
		for type_ in [
			Abstract_Data_Type::unsigned32,
			Abstract_Data_Type::unsigned64,
		].iter()
		{
			assert_eq!(
				information_element_parser(&data, *type_, field_length),
				Ok((&data[4..8], res.clone()))
			);
		}

		let res = unsigned64(0x8899aabbccddeeff);
		let field_length = 8;
		assert_eq!(
			information_element_parser(&data, Abstract_Data_Type::unsigned64, field_length),
			Ok((&[][..], res))
		);
	}

	#[test]
	#[should_panic(expected = "explicit panic")]
	fn unsigned_integer_parser_fail() {
		let data : &[u8] = &[0x01, 0x02, 0x3, 0x04];
		let _res = information_element_parser(&data, Abstract_Data_Type::unsigned32, 3);
	}

	#[test]
	fn signed_integer_parser_test() {
		use Data_Value::*;

		let data : &[u8] = &[0x77, 0x88, 0x99, 0xaa, 0xbb, 0xcc, 0xdd, 0xee];

		let res = signed8(0x77);
		let field_length = 1;
		for type_ in [
			Abstract_Data_Type::signed8,
			Abstract_Data_Type::signed16,
			Abstract_Data_Type::signed32,
			Abstract_Data_Type::signed64,
		].iter()
		{
			assert_eq!(
				information_element_parser(&data, *type_, field_length),
				Ok((&data[1..8], res.clone()))
			);
		}

		let res = signed16(0x7788);
		let field_length = 2;
		for type_ in [
			Abstract_Data_Type::signed16,
			Abstract_Data_Type::signed32,
			Abstract_Data_Type::signed64,
		].iter()
		{
			assert_eq!(
				information_element_parser(&data, *type_, field_length),
				Ok((&data[2..8], res.clone()))
			);
		}

		let res = signed32(0x778899aa);
		let field_length = 4;
		for type_ in [Abstract_Data_Type::signed32, Abstract_Data_Type::signed64].iter() {
			assert_eq!(
				information_element_parser(&data, *type_, field_length),
				Ok((&data[4..8], res.clone()))
			);
		}

		let res = signed64(0x778899aabbccddee);
		let field_length = 8;
		assert_eq!(
			information_element_parser(&data, Abstract_Data_Type::signed64, field_length),
			Ok((&[][..], res))
		);
	}

	#[test]
	#[should_panic(expected = "explicit panic")]
	fn signed_integer_parser_fail() {
		let data : &[u8] = &[0x01, 0x02, 0x3, 0x04];
		let _res = information_element_parser(&data, Abstract_Data_Type::signed32, 3);
	}

	#[test]
	fn float_parser_test() {
		// TODO
		// Testing would be easier with hexadecimal floating point literals.
	}

	#[test]
	#[should_panic(expected = "explicit panic")]
	fn float_parser_fail() {
		let data : &[u8] = &[0x01, 0x02, 0x3, 0x04];
		let _res = information_element_parser(&data, Abstract_Data_Type::float32, 3);
	}

	#[test]
	fn bool_parser_test() {
		use Data_Value::*;

		let data : &[u8] = &[0x00];
		assert_eq!(
			information_element_parser(&data, Abstract_Data_Type::boolean, 1),
			Err(Err::Error(nom::Context::Code(
				data,
				error_kind::BOOL_INVALID
			)))
		);

		let data : &[u8] = &[0x01];
		let res = boolean(true);
		assert_eq!(
			information_element_parser(&data, Abstract_Data_Type::boolean, 1),
			Ok((&[][..], res))
		);

		let data : &[u8] = &[0x02];
		let res = boolean(false);
		assert_eq!(
			information_element_parser(&data, Abstract_Data_Type::boolean, 1),
			Ok((&[][..], res))
		);

		let data : &[u8] = &[0x03];
		assert_eq!(
			information_element_parser(&data, Abstract_Data_Type::boolean, 1),
			Err(Err::Error(nom::Context::Code(
				data,
				error_kind::BOOL_INVALID
			)))
		);

		let data : &[u8] = &[];
		assert_eq!(
			information_element_parser(&data, Abstract_Data_Type::boolean, 1),
			Err(Err::Incomplete(Needed::Size(1)))
		);
	}

	#[test]
	#[should_panic(expected = "explicit panic")]
	fn bool_parser_fail() {
		let data : &[u8] = &[0x01];
		let _res = information_element_parser(&data, Abstract_Data_Type::boolean, 0);
	}

	#[test]
	fn mac_address_parser_test() {
		use Data_Value::*;

		let data : &[u8] = &[0x01, 0x02, 0x3, 0x04, 0x05, 0x06];
		assert_eq!(
			information_element_parser(&data, Abstract_Data_Type::macAddress, 6),
			Ok((&[][..], macAddress(data.to_vec())))
		);
	}

	#[test]
	#[should_panic(expected = "explicit panic")]
	fn mac_address_parser_fail() {
		let data : &[u8] = &[0x01, 0x02, 0x3, 0x04, 0x05, 0x06];
		let _res = information_element_parser(&data, Abstract_Data_Type::macAddress, 4);
	}

	#[test]
	fn octet_array_parser_test() {
		use Data_Value::*;

		let data : &[u8] = &[0x00, 0x00, 0x00, 0x00];
		let res = octetArray(vec![0x00, 0x00, 0x00, 0x00]);
		let field_length = 4;
		assert_eq!(
			information_element_parser(&data, Abstract_Data_Type::octetArray, field_length),
			Ok((&[][..], res))
		);

		let variable_length = 0xffffu16;

		let data : &[u8] = &[0x00];
		let res = octetArray(vec![]);
		assert_eq!(
			information_element_parser(&data, Abstract_Data_Type::octetArray, variable_length),
			Ok((&[][..], res))
		);

		let data : &[u8] = &[0x04, 0x00, 0x00, 0x00, 0x00];
		let res = octetArray(vec![0x00, 0x00, 0x00, 0x00]);
		assert_eq!(
			information_element_parser(&data, Abstract_Data_Type::octetArray, variable_length),
			Ok((&[][..], res))
		);

		let data : &[u8] = &[0xff, 0x00, 0x00];
		let res = octetArray(vec![]);
		assert_eq!(
			information_element_parser(&data, Abstract_Data_Type::octetArray, variable_length),
			Ok((&[][..], res))
		);

		let data : &[u8] = &[0xff, 0x00, 0x04, 0x00, 0x00, 0x00, 0x00];
		let res = octetArray(vec![0x00, 0x00, 0x00, 0x00]);
		assert_eq!(
			information_element_parser(&data, Abstract_Data_Type::octetArray, variable_length),
			Ok((&[][..], res))
		);

		let mut vector = vec![0xff, 0x04, 0x01];
		vector.extend(vec![0x00; 1025]);
		let data : &[u8] = &vector[..];
		let res = octetArray(vec![0x00; 1025]);
		assert_eq!(
			information_element_parser(&data, Abstract_Data_Type::octetArray, variable_length),
			Ok((&[][..], res))
		);

		let mut vector = vec![0xff, 0xff, 0xff];
		vector.extend(vec![0x00; 0xffff]);
		let data : &[u8] = &vector[..];
		let res = octetArray(vec![0x00; 0xffff]);
		assert_eq!(
			information_element_parser(&data, Abstract_Data_Type::octetArray, variable_length),
			Ok((&[][..], res))
		);
	}

	#[test]
	fn string_parser_test() {
		let data : &[u8] = &[240, 159, 146, 150];
		let res = Data_Value::string("💖".to_string());
		assert_eq!(
			information_element_parser(&data, Abstract_Data_Type::string, 4),
			Ok((&[][..], res))
		);

		let data : &[u8] = &[4, 240, 159, 146, 150];
		let res = Data_Value::string("💖".to_string());
		assert_eq!(
			information_element_parser(&data, Abstract_Data_Type::string, 0xffffu16),
			Ok((&[][..], res))
		);

		let data : &[u8] = &[240, 0, 146, 151]; // modified
		let _res = Data_Value::string("💖".to_string());
		assert_eq!(
			information_element_parser(&data, Abstract_Data_Type::string, 4),
			Err(Err::Error(nom::Context::Code(
				&[][..],
				error_kind::STRING_NOT_UTF8
			)))
		);

		let data : &[u8] = &[4, 240, 0, 146, 151]; // modified
		let _res = Data_Value::string("💖".to_string());
		assert_eq!(
			information_element_parser(&data, Abstract_Data_Type::string, 0xffffu16),
			Err(Err::Error(nom::Context::Code(
				&[][..],
				error_kind::STRING_NOT_UTF8
			)))
		);
	}

	#[test]
	fn template_record_parser_test() {
		let data : &[u8] = &[
			0x01, 0x02, 0x00, 0x04, // template record header
			0x00, 0x07, 0x00, 0x02, // field 0
			0x01, 0x37, 0x00, 0x08, // field 1
			0x00, 0xd2, 0x00, 0x04, // field 2
			0x81, 0x02, 0x00, 0x04, 0x00, 0x00, 0xC3, 0x3C, // field 3
		];
		let res = Template_Record {
			header : Template_Record_Header {
				template_id : 0x0102u16,
				field_count : 0x0004u16,
				scope_field_count : 0u16,
			},
			scope_fields : vec![],
			fields : vec![
				Field_Specifier {
					information_element_id : 7u16,
					field_length : 2u16,
					enterprise_number : None,
				},
				Field_Specifier {
					information_element_id : 311u16,
					field_length : 8u16,
					enterprise_number : None,
				},
				Field_Specifier {
					information_element_id : 210u16,
					field_length : 4,
					enterprise_number : None,
				},
				Field_Specifier {
					information_element_id : 258u16,
					field_length : 4,
					enterprise_number : Some(0xC33C),
				},
			],
		};
		assert_eq!(template_record_parser(&data, false), Ok((&[][..], res)));
	}

	#[test]
	fn field_specifier_parser_test() {
		let data : &[u8] = &[
			0x00, 0x07, // id
			0x00, 0x02, // length
		];
		let res = Field_Specifier {
			information_element_id : 7u16,
			field_length : 2u16,
			enterprise_number : None,
		};
		assert_eq!(field_specifier_parser(&data), Ok((&[][..], res)));

		let data : &[u8] = &[
			0x01, 0x37, // id
			0x00, 0x08, // length
		];
		let res = Field_Specifier {
			information_element_id : 311u16,
			field_length : 8u16,
			enterprise_number : None,
		};
		assert_eq!(field_specifier_parser(&data), Ok((&[][..], res)));

		let data : &[u8] = &[
			0x00, 0xd2, // id
			0x00, 0x04, // length
		];
		let res = Field_Specifier {
			information_element_id : 210u16,
			field_length : 4,
			enterprise_number : None,
		};
		assert_eq!(field_specifier_parser(&data), Ok((&[][..], res)));

		let data : &[u8] = &[
			0x81, 0x02, // id
			0x00, 0x04, // length
			0x00, 0x00, 0xC3, 0x3C, // enterprise number
		];
		let res = Field_Specifier {
			information_element_id : 258u16,
			field_length : 4,
			enterprise_number : Some(0xC33C),
		};
		assert_eq!(field_specifier_parser(&data), Ok((&[][..], res)));
	}
}
