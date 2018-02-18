extern crate lib_ipfix_rs;
use lib_ipfix_rs::*;

extern crate env_logger;
#[macro_use]
extern crate log;

use std::fs::File;
use std::io::prelude::*;

pub fn main() {
	env_logger::init();
	let mut cache = Template_Cache::default();

	let mut input_vec = Vec::<u8>::default();
	File::open("/dev/stdin")
		.unwrap()
		.read_to_end(&mut input_vec)
		.unwrap();
	let input_vec = input_vec;
	let mut input = &input_vec[..];

	let mut message_num = 0;
	while input != &b""[..] {
		let result = message_parser(&input[..]).unwrap();
		input = result.0;
		let message = result.1;

		debug!("message {}: {:?}", message_num, message);
		message_num += 1;

		for (set_header, data) in message.sets {
			match set_header.set_id {
				2 | 3 => {
					let templates = template_records_parser(data, set_header).unwrap().1;
					for template in templates {
						cache.update_with(template);
					}
				}
				256...65535 => {
					let template = match cache.lookup(set_header.set_id) {
						None => {
							warn!("received data set without known template");
							continue;
						}
						Some(template) => template,
					};

					let opt = data_records_parser(
						data,
						set_header.length - SET_HEADER_LENGTH,
						template.size(),
					).ok();

					if opt.is_none() || opt.as_ref().unwrap().0 == &b""[..] {
						error!("failed to parse data set");
						continue;
					}
				}
				id => error!("received set with reserved set id {}", id),
			}
		}
	}
}
