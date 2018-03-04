extern crate libipfix;

extern crate env_logger;
#[macro_use]
extern crate log;

use std::io::BufReader;
use std::net;

fn main() {
	env_logger::init();

	let listener = net::TcpListener::bind("127.0.0.1:8080").unwrap();
	info!("listening on 127.0.0.1:8080");
	let stream = listener.accept().unwrap().0;
	libipfix::collect(&mut BufReader::new(stream));
}
