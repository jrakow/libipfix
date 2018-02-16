use nom::*;
use structs::*;
use super::*;

// TODO ignore first bit at enterprise_number
// TODO subtract header size from length field

named!(message_parser<Message>,
	do_parse!(
		header : message_header_parser >>
		sets : flat_map!(
			take!(header.length),
			separated_list!(/* dummy */ value!(0), set_parser)
		) >>
		(Message{ header, sets })
	)
);

named!(message_header_parser<Message_Header>,
	do_parse!(
		version_number : u16!(Endianness::Big) >>
		length : u16!(Endianness::Big) >>
		export_time : u32!(Endianness::Big) >>
		sequence_number : u32!(Endianness::Big) >>
		observation_domain_id : u32!(Endianness::Big) >>
		(Message_Header{ version_number, length, export_time, sequence_number, observation_domain_id })
	)
);

named!(field_specifier_parser<Field_Specifier>,
	do_parse!(
		information_element_id : u16!(Endianness::Big) >>
		field_length : u16!(Endianness::Big) >>
		enterprise_number : cond!(information_element_id & 0x8000 != 0x0000, u32!(Endianness::Big)) >>
		(Field_Specifier{ information_element_id, field_length, enterprise_number })
	)
);

named!(set_parser<Set>,
	do_parse!(
		header : set_header_parser >>
		records : alt_complete!(
			cond_reduce!(header.set_id == 2,
				map!(complete!(flat_map!(
					take!(header.length),
					many0!(template_record_parser)
				)), |records : Vec<Template_Record>| Records::Template_Records(records))
			)
			| cond_reduce!(header.set_id == 3,
				map!(complete!(flat_map!(
					take!(header.length),
					many0!(options_template_record_parser)
				)), |records : Vec<Options_Template_Record>| Records::Options_Template_Records(records))
			)
			| cond_reduce!(header.set_id >= 256,
				map!(complete!(flat_map!(
					take!(header.length),
					many0!(
						cond_reduce!(
							template_size(header.set_id).is_some(),
							map!(
								take!(template_size(header.set_id).unwrap()),
								|fields : &[u8]| { Data_Record{ fields : fields.to_vec() } }
							)
						)
					)
				)), |records : Vec<Data_Record>| Records::Data_Records(records))
			)
		) >>
		(Set{ header, records })
	)
);

named!(set_header_parser<Set_Header>,
	do_parse!(
		set_id : u16!(Endianness::Big) >>
		length : u16!(Endianness::Big) >>
		(Set_Header{ set_id, length })
	)
);

named!(template_record_parser<Template_Record>,
	do_parse!(
		header : template_record_header_parser >>
		fields : many_m_n!(
			header.field_count as usize,
			header.field_count as usize,
			field_specifier_parser) >>
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

named!(options_template_record_parser<Options_Template_Record>,
	do_parse!(
		header : options_template_record_header_parser >>
		fields : many_m_n!(
			(header.field_count + header.scope_field_count) as usize,
			(header.field_count + header.scope_field_count) as usize,
			field_specifier_parser) >>
		(Options_Template_Record{ header, fields })
	)
);

named!(options_template_record_header_parser<Options_Template_Record_Header>,
	do_parse!(
		template_id : u16!(Endianness::Big) >>
		field_count : u16!(Endianness::Big) >>
		scope_field_count : u16!(Endianness::Big) >>
		(Options_Template_Record_Header{ template_id, field_count, scope_field_count })
	)
);
