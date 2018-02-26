#![no_main]
#[macro_use] extern crate libfuzzer_sys;
extern crate libipfix;

fuzz_target!(|data: &[u8]| {
	let _res = libipfix::message_parser(data);
});
