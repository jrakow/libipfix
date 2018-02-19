extern crate libipfix;
use libipfix::*;

extern crate env_logger;

#[test]
fn ipfix_dump() {
	env_logger::init();

	// downloaded from http://www7.informatik.uni-erlangen.de/~limmer/files/ipfix.dump.gz
	collect(include_bytes!("ipfix.dump"));
}
