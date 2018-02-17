use nom::*;
use structs::*;

// TODO insert padding

named!(pub message_parser<Message>,
	do_parse!(
		// TODO check length here too
		message_header : message_header_parser >>
		sets : many0!(
			complete!(do_parse!(
				set_header : verify!(set_header_parser, |header : Set_Header| header.length > SET_HEADER_LENGTH) >>
				data : length_data!(value!(set_header.length - SET_HEADER_LENGTH)) >>
				(set_header, data)
			))
		) >>
		(Message{ header : message_header, sets : sets })
	)
);

named!(message_header_parser<Message_Header>,
	do_parse!(
		/* version_number */ tag!([0x00, 0x0a]) >>
		length : u16!(Endianness::Big) >>
		export_time : u32!(Endianness::Big) >>
		sequence_number : u32!(Endianness::Big) >>
		observation_domain_id : u32!(Endianness::Big) >>
		(Message_Header{ version_number : 0x000au16, length, export_time, sequence_number, observation_domain_id })
	)
);

named!(set_header_parser<Set_Header>,
	do_parse!(
		set_id : u16!(Endianness::Big) >>
		length : u16!(Endianness::Big) >>
		(Set_Header{ set_id, length })
	)
);

named!(field_specifier_parser<Field_Specifier>,
	do_parse!(
		information_element_id : u16!(Endianness::Big) >>
		field_length : u16!(Endianness::Big) >>
		enterprise_number : cond!(information_element_id & 0x8000 != 0x0000, u32!(Endianness::Big)) >>
		(Field_Specifier{
			information_element_id : information_element_id & 0x7fff,
			field_length,
			enterprise_number
		})
	)
);

named_args!(pub template_records_parser(set_header : Set_Header)<Vec<Template_Record>>,
	length_value!(
		value!(set_header.length - SET_HEADER_LENGTH),
		many1!(complete!(template_record_parser))
	)
);

/* TODO
named_args!(pub options_template_records_parser(set_header : Set_Header)<Vec<Options_Template_Record>>,
	length_value!(
		value!(set_header.length - set_header_length),
		many1!(options_template_record_parser)
	)
);
*/

// needs explicit lifetimes because reference to cache
pub fn data_records_parser<'input>(input : &'input[u8], records_length : u16, template_size : u16) -> IResult<&'input[u8], Vec<Data_Record>> {
	length_value!(
		input,
		value!(records_length),
		many1!(
			complete!(map!(
				take!(template_size),
				|a : &[u8]| { Data_Record{ fields : a.to_vec() } }
			))
		)
	)
}

named!(template_record_parser<Template_Record>,
	do_parse!(
		header : template_record_header_parser >>
		fields : count!(
			field_specifier_parser,
			header.field_count as usize) >>
		(Template_Record{ header, fields })
	)
);

named!(template_record_header_parser<Template_Record_Header>,
	do_parse!(
		template_id : u16!(Endianness::Big) >>
		field_count : u16!(Endianness::Big) >>
		(Template_Record_Header{ template_id, field_count })
	)
);

/* TODO
named!(options_template_record_parser<Options_Template_Record>,
	do_parse!(
		header : options_template_record_header_parser >>
		fields : many_m_n!(
			header.field_count as usize,
			header.field_count as usize,
			field_specifier_parser) >>
		scope_fields : many_m_n!(
			header.scope_field_count as usize,
			header.scope_field_count as usize,
			field_specifier_parser) >>
		(Options_Template_Record{ header, fields, scope_fields })
	)
);
*/

/* TODO
named!(options_template_record_header_parser<Options_Template_Record_Header>,
	do_parse!(
		template_id : u16!(Endianness::Big) >>
		field_count : u16!(Endianness::Big) >>
		scope_field_count : u16!(Endianness::Big) >>
		(Options_Template_Record_Header{ template_id, field_count, scope_field_count })
	)
);
*/

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn message_parser_test() {
		let data : [u8; 56] = [
			0x00, 0x0a, 0x00, 56, // version, length
			0x5A, 0x88, 0x08, 0x2E, // time
			0xfe, 0xdc, 0xba, 0x98, // seq num
			0xde, 0xad, 0xbe, 0xef, // domain id
			0x00, 0x02, 0x00, 20, // set id, set length
			0x01, 0x23, 0x45, 0x67,
			0x89, 0xab, 0xcd, 0xef,
			0x01, 0x23, 0x45, 0x67,
			0x89, 0xab, 0xcd, 0xef,
			0x00, 0x03, 0x00, 20, // set id, set length
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
					Set_Header{ set_id : 2u16, length : 20u16 },
					&[
						0x01, 0x23, 0x45, 0x67,
						0x89, 0xab, 0xcd, 0xef,
						0x01, 0x23, 0x45, 0x67,
						0x89, 0xab, 0xcd, 0xef,
					]
				),
				(
					Set_Header{ set_id : 3u16, length : 20u16 },
					&[
						0x89, 0xab, 0xcd, 0xef,
						0x01, 0x23, 0x45, 0x67,
						0x89, 0xab, 0xcd, 0xef,
						0x01, 0x23, 0x45, 0x67,
					]
				),
			]
		};
		assert_eq!(
			message_parser(&data),
			Ok((&[][..], res))
		);
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
		assert_eq!(
			message_header_parser(&data),
			Ok((&b""[..], res))
		);
	}

	#[test]
	fn set_header_parser_test() {
		let data : [u8; 4] = [
			0x00, 0x02, 0x00, 4, // set id, set length
		];
		let res = Set_Header{
			set_id : 2u16,
			length : 4u16
		};
		assert_eq!(
			set_header_parser(&data),
			Ok((&[][..], res))
		);
	}

	#[test]
	fn template_records_parser_test() {
		let set_header = Set_Header {
			set_id : 2,
			length : 20,
		};
		let data : &[u8] = &[
			0x01, 0x02, 0x00, 0x01,
			0x00, 0x07, 0x00, 0x02,

			0x01, 0x02, 0x00, 0x01,
			0x00, 0x07, 0x00, 0x02,
		];
		let res = vec![
			Template_Record{
				header : Template_Record_Header {
					template_id : 0x0102u16,
					field_count : 0x0001u16,
				},
				fields : vec![
					Field_Specifier{
						information_element_id : 7u16,
						field_length : 2u16,
						enterprise_number : None,
					},
				],
			},
			Template_Record{
				header : Template_Record_Header {
					template_id : 0x0102u16,
					field_count : 0x0001u16,
				},
				fields : vec![
					Field_Specifier{
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
	fn template_record_parser_test() {
		let data : &[u8] = &[
			0x01, 0x02, 0x00, 0x04,
			0x00, 0x07, 0x00, 0x02,
			0x01, 0x37, 0x00, 0x08,
			0x00, 0xd2, 0x00, 0x04,
			0x81, 0x02, 0x00, 0x04, 0x00, 0x00, 0xC3, 0x3C,
		];
		let res = Template_Record{
			header : Template_Record_Header {
				template_id : 0x0102u16,
				field_count : 0x0004u16,
			},
			fields : vec![
				Field_Specifier{
					information_element_id : 7u16,
					field_length : 2u16,
					enterprise_number : None,
				},
				Field_Specifier{
					information_element_id : 311u16,
					field_length : 8u16,
					enterprise_number : None,
				},
				Field_Specifier{
					information_element_id : 210u16,
					field_length : 4,
					enterprise_number : None,
				},
				Field_Specifier{
					information_element_id : 258u16,
					field_length : 4,
					enterprise_number : Some(0xC33C),
				},
			],
		};
		assert_eq!(
			template_record_parser(&data),
			Ok((&[][..], res))
		);
	}

	#[test]
	fn field_specifier_parser_test() {
		let data : &[u8] = &[
			0x00, 0x07,
			0x00, 0x02,
		];
		let res = Field_Specifier{
			information_element_id : 7u16,
			field_length : 2u16,
			enterprise_number : None,
		};
		assert_eq!(
			field_specifier_parser(&data),
			Ok((&[][..], res))
		);

		let data : &[u8] = &[
			0x01, 0x37,
			0x00, 0x08,
		];
		let res = Field_Specifier{
			information_element_id : 311u16,
			field_length : 8u16,
			enterprise_number : None,
		};
		assert_eq!(
			field_specifier_parser(&data),
			Ok((&[][..], res))
		);

		let data : &[u8] = &[
			0x00, 0xd2,
			0x00, 0x04,
		];
		let res = Field_Specifier{
			information_element_id : 210u16,
			field_length : 4,
			enterprise_number : None,
		};
		assert_eq!(
			field_specifier_parser(&data),
			Ok((&[][..], res))
		);

		let data : &[u8] = &[
			0x81, 0x02,
			0x00, 0x04,
			0x00, 0x00, 0xC3, 0x3C,
		];
		let res = Field_Specifier{
			information_element_id : 258u16,
			field_length : 4,
			enterprise_number : Some(0xC33C),
		};
		assert_eq!(
			field_specifier_parser(&data),
			Ok((&[][..], res))
		);
	}
}
