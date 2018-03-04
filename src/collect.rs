use std;

use parser::*;
use structs::*;
use template_management::*;

use nom;
use nom::Needed;
use serde_json;

pub fn collect<Reader>(reader : &mut Reader)
where
	Reader : std::io::Read,
{
	let mut buffer = Vec::<u8>::new();
	let mut cache = TemplateCache::default();

	let mut message_num = 0;
	loop {
		match message_parser(&buffer.clone()[..]) {
			Ok((rest, message)) => {
				trace!("message header {}: {:?}", message_num, message.header);
				collect_message_body(&mut cache, message, message_num);
				message_num += 1;

				// discard consumed front of buffer
				let consumed_len = buffer.len() - rest.len();
				buffer = buffer.split_off(consumed_len);
			}
			Err(nom::Err::Incomplete(needed)) => {
				let old_len = buffer.len();
				let length = match needed {
					Needed::Size(length) => length,
					Needed::Unknown => 1,
				};

				// get more input
				buffer.resize(old_len + length, 0x00);
				if reader.read_exact(&mut buffer[old_len..]).is_err() {
					return;
				}
			}
			// parser error and failure
			Err(e) => {
				error!("message {} unparseable", message_num);
				println!("error {:?}", e);
				println!("input = {:?}", buffer);
				return;
			}
		}
	}
}

fn collect_message_body(cache : &mut TemplateCache, message : Message, message_num : usize) {
	let mut set_num = 0;
	for (set_header, data) in message.sets {
		trace!("set header {}.{}: {:?}", message_num, set_num, set_header);

		match set_header.set_id {
			2 | 3 => {
				let templates = match template_records_parser(data, set_header) {
					Ok((_, t)) => t,
					Err(_) => {
						error!("template set {}.{} unparseable", message_num, set_num);
						continue;
					}
				};
				for template in &templates {
					trace!(
						"template {}.{}.{}: {:?}",
						message_num,
						set_num,
						template.header.template_id,
						template
					);
					if let Err(e) = verify_template(template) {
						error!("{:?}", e);
						continue;
					}
					if let Err(e) = cache.update_with(template.clone()) {
						error!("{:?}", e);
						continue;
					}
				}
			}
			256...0xffff => {
				let template = match cache.lookup(set_header.set_id) {
					None => {
						error!("received data set without known template");
						continue;
					}
					Some(template) => template,
				};

				let records = match data_records_parser(
					data,
					set_header.length - SET_HEADER_LENGTH,
					template,
				) {
					Ok(o) => o,
					Err(e) => {
						error!("{:?}", e);
						continue;
					}
				};
				for (record_num, record) in records.1.iter().enumerate() {
					println!(
						"{}",
						serde_json::to_string(&TypedDataRecord {
							data : record,
							template,
						}).unwrap()
					);
					trace!(
						"data record {}.{}.{}: {:?}",
						message_num,
						set_num,
						record_num,
						record
					);
				}
			}
			id => error!("received set with reserved set id {}", id),
		}
		set_num += 1;
	}
}
