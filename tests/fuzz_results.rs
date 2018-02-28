extern crate libipfix;
use libipfix::*;

extern crate env_logger;

#[test]
fn message_parser_fuzz_test() {
	let data =
		include_bytes!("fuzz_results/minimized-from-e36226af5aafeaab3dd1dabb4ae0e7f96dd78212");
	let _res = message_parser(data);
}

#[test]
fn data_records_parser_fuzz_test() {}

#[test]
fn template_records_parser_fuzz_test() {
	let set_header = Set_Header {
		set_id : 3,
		length : 256,
	};
	let _res = template_records_parser(
		include_bytes!("fuzz_results/minimized-from-7fa7c999c258a01dabd846a7bc49ec3dd9bab049"),
		set_header,
	);
}
