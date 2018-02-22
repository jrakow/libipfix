use super::*;

pub fn collect(input : &[u8]) {
	let mut input = input.clone();
	let mut cache = Template_Cache::default();

	let mut message_num = 0;
	while input != &b""[..] {
		let mut set_num = 0;

		let result = match message_parser(&input[..]) {
			Ok(o) => o,
			Err(_) => {
				error!("message {} unparseable", message_num);
				continue;
			}
		};

		input = result.0;
		let message = result.1;

		trace!("message header {}: {:?}", message_num, message.header);

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
						if let Err(e) = verify_template(&template) {
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
		message_num += 1;
	}
}
