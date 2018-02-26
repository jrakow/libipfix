extern crate libipfix;
use libipfix::*;

extern crate env_logger;

#[test]
fn message_parser_fuzz_test() {
	let data = include_bytes!("fuzz_results/minimized-from-e36226af5aafeaab3dd1dabb4ae0e7f96dd78212");
	let _res = message_parser(data);
}
