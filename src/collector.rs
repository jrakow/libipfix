use super::*;

pub fn collect(input : &[u8]) {
	let mut input = input.clone();
	let mut cache = Template_Cache::default();

	let mut message_num = 0;
	while input != &b""[..] {
		let mut set_num = 0;

		let result = message_parser(&input[..]).unwrap();
		input = result.0;
		let message = result.1;

		debug!("message {}: {:?}", message_num, message.header);

		for (set_header, data) in message.sets {
			match set_header.set_id {
				2 | 3 => {
					let templates = template_records_parser(data, set_header).unwrap().1;
					for template in &templates {
						match verify_template(&template) {
							Ok(_) => cache.update_with(template.clone()),
							Err(e) => warn!("{}", e),
						}
					}
					trace!(
						"message {}: set {}: {:?}",
						message_num,
						set_num,
						(set_header, templates)
					);
				}
				256...0xffff => {
					let template = match cache.lookup(set_header.set_id) {
						None => {
							warn!("received data set without known template");
							continue;
						}
						Some(template) => template,
					};

					let opt =
						data_records_parser(data, set_header.length - SET_HEADER_LENGTH, template)
							.ok();

					if opt.is_none() || opt.as_ref().unwrap().0 != &b""[..] {
						error!("failed to parse data set");
						continue;
					}
					trace!(
						"message {}: set {}: {:?}",
						message_num,
						set_num,
						(set_header, opt.unwrap().1)
					);
				}
				id => error!("received set with reserved set id {}", id),
			}
			set_num += 1;
		}
		message_num += 1;
	}
}
