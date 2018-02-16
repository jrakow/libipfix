#![allow(non_camel_case_types)]

#[macro_use]
extern crate nom;

mod parser;
mod structs;

fn template_size(template_id : u16) -> Option<u16> {
	if template_id == 256 {
		Some(2)
	} else {
		None
	}
}
