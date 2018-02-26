extern crate libipfix;
use libipfix::*;

extern crate env_logger;

#[test]
fn message_parser_fuzz_test() {
	let data = include_bytes!("fuzz_results/minimized-from-e36226af5aafeaab3dd1dabb4ae0e7f96dd78212");
	let _res = message_parser(data);
}

#[test]
fn data_records_parser_fuzz_test() {
	let template = Template_Record {
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
	};
	let data = include_bytes!("fuzz_results/crash-da39a3ee5e6b4b0d3255bfef95601890afd80709");
	let _res = data_records_parser(data, 0xffff, &template);
}
