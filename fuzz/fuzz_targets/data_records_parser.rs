#![no_main]
#[macro_use] extern crate libfuzzer_sys;
extern crate libipfix;

use libipfix::*;

fuzz_target!(|data: &[u8]| {
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
	let _res = data_records_parser(data, 0xffff, &template);
});
