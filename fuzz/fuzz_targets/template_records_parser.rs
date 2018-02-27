#![no_main]
#[macro_use] extern crate libfuzzer_sys;
extern crate libipfix;

use libipfix::*;

fuzz_target!(|data: &[u8]| {
	let set_header = Set_Header {
		set_id : 3,
		length : 256,
	};
	let _res = template_records_parser(data, set_header);
});
