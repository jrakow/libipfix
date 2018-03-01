extern crate env_logger;
extern crate libipfix;

use std::io::{self, Read};

fn main() {
	env_logger::init();

	let mut buffer = Vec::<u8>::new();
	io::stdin().read_to_end(&mut buffer).unwrap();
	libipfix::collect(&buffer);
}
