#![allow(non_camel_case_types)]

#[macro_use]
extern crate nom;

mod parser;
use parser::*;
mod structs;
use structs::*;

use std::collections::HashMap;

#[derive(Default)]
pub struct Template_Cache {
	templates : HashMap<u16, Template_Record>,
	// TODO
//	options_templates : HashMap<u16, Options_Template_Record>,
}

impl Template_Cache {
	pub fn update_with(&mut self, template : Template_Record) {
		// template withdrawal
		if template.header.field_count == 0 {
			if self.templates.remove(&template.header.template_id).is_none() {
				// TODO spurious removal of unknown template
			}
		} else {
			if self.templates.contains_key(&template.header.template_id) {
				let existing_template = self.templates.get(&template.header.template_id).unwrap();
				if existing_template == &template {
					// ok, template known
				} else {
					// TODO spurious redefinition
				}
			} else {
				// ok, add unknown template
				self.templates.insert(template.header.template_id, template);
			}
		}
	}

	pub fn lookup(&self, id : u16) -> Option<&Template_Record> {
		self.templates.get(&id)
	}

	pub fn lookup_size(&self, id : u16) -> Option<u16> {
		self.lookup(id).map(
			|template| template.fields.iter().map(
				|field| field.field_length
			).sum()
		)
	}
}
