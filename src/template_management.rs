use std::collections::HashMap;

use structs::*;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Template_Cache {
	templates : HashMap<u16, Template_Record>,
	// TODO
//	options_templates : HashMap<u16, Options_Template_Record>,
}

impl Template_Cache {
	pub fn update_with(&mut self, template : Template_Record) {
		assert!(template.header.template_id >= 256);
		assert!(template.header.field_count as usize == template.fields.len());

		if template.header.field_count == 0 {
			// template withdrawal
			if self.templates.remove(&template.header.template_id).is_none() {
				warn!("spurious withdrawal of unknown template with id {}", template.header.template_id);
			}
		} else {
			// template definition
			if self.templates.contains_key(&template.header.template_id) {
				if self.templates.get(&template.header.template_id).unwrap() == &template {
					info!("identical definition of known template with id {}", template.header.template_id);
					// ok, template known
				} else {
					warn!("spurious redefinition of known template with id {}", template.header.template_id);
					info!("removing both templates to avoid ambiguity");
					self.templates.remove(&template.header.template_id);
				}
			} else {
				info!("adding template with id {}", template.header.template_id);
				self.templates.insert(template.header.template_id, template);
			}
		}
	}

	pub fn lookup(&self, id : u16) -> Option<&Template_Record> {
		self.templates.get(&id)
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	const DUMMY_FIELD : Field_Specifier = Field_Specifier {
		information_element_id : 210,
		field_length : 4,
		enterprise_number : None,
	};

	#[test]
	fn normal_case() {
		let mut cache = Template_Cache::default();
		let template = Template_Record {
			header : Template_Record_Header {
				template_id : 256,
				field_count : 1,
			},
			fields : vec![ DUMMY_FIELD ]
		};
		cache.update_with(template.clone());
		assert_eq!(cache.lookup(256).unwrap(), &template);

		// identical redefinition
		cache.update_with(template.clone());
		assert_eq!(cache.lookup(256).unwrap(), &template);

		let removal = Template_Record {
			header : Template_Record_Header {
				template_id : 256,
				field_count : 0,
			},
			fields : vec![]
		};
		cache.update_with(removal);
		assert!(cache.lookup(256).is_none());
	}

	#[test]
	fn spurious_withdrawal() {
		let mut cache = Template_Cache::default();
		cache.update_with(
			Template_Record{
				header : Template_Record_Header {
					template_id : 256,
					field_count : 0,
				},
				fields : vec![]
			}
		);
		assert!(cache.lookup(256).is_none());
	}

	#[test]
	#[should_panic(expected = "assertion failed")]
	fn panic_on_nontemplate() {
		let mut cache = Template_Cache::default();
		cache.update_with(
			Template_Record{
				header : Template_Record_Header {
					template_id : 0,
					field_count : 0,
				},
				fields : vec![]
			}
		);
	}

	#[test]
	#[should_panic(expected = "assertion failed")]
	fn panic_on_wrong_size() {
		let mut cache = Template_Cache::default();
		cache.update_with(
			Template_Record{
				header : Template_Record_Header {
					template_id : 0,
					field_count : 1,
				},
				fields : vec![]
			}
		);
	}
}
