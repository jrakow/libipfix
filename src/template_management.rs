use information_element;
use std;
use structs::*;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Template_Cache {
	templates : std::collections::HashMap<u16, Template_Record>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Update_Ok {
	Addition,
	Redefinition,
	Withdrawal,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Update_Err {
	Redefinition_Different,
	Withdrawal_Unknown,
}

impl Template_Cache {
	pub fn update_with(&mut self, template : Template_Record) -> Result<Update_Ok, Update_Err> {
		use std::collections::hash_map::Entry::*;

		assert!(template.header.template_id >= 256);
		assert!(template.header.scope_field_count as usize == template.scope_fields.len());
		assert!(
			template.header.field_count as usize - template.header.scope_field_count as usize
				== template.fields.len()
		);

		if template.header.field_count == 0 {
			// template withdrawal
			self.templates
				.remove(&template.header.template_id)
				.map(|_| Update_Ok::Withdrawal)
				.ok_or(Update_Err::Withdrawal_Unknown)
		} else {
			match self.templates.entry(template.header.template_id) {
				Occupied(ref entry) if &template == entry.get() => Ok(Update_Ok::Redefinition),
				Occupied(entry) => {
					// template != entry
					entry.remove();
					Err(Update_Err::Redefinition_Different)
				}
				Vacant(entry) => {
					entry.insert(template);
					Ok(Update_Ok::Addition)
				}
			}
		}
	}

	pub fn lookup(&self, id : u16) -> Option<&Template_Record> {
		self.templates.get(&id)
	}
}

#[cfg(test)]
mod template_cache_tests {
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
				scope_field_count : 0,
			},
			scope_fields : vec![],
			fields : vec![DUMMY_FIELD],
		};
		assert_eq!(cache.update_with(template.clone()), Ok(Update_Ok::Addition));
		assert_eq!(cache.lookup(256).unwrap(), &template);

		// identical redefinition
		assert_eq!(
			cache.update_with(template.clone()),
			Ok(Update_Ok::Redefinition)
		);
		assert_eq!(cache.lookup(256).unwrap(), &template);

		let removal = Template_Record {
			header : Template_Record_Header {
				template_id : 256,
				field_count : 0,
				scope_field_count : 0,
			},
			scope_fields : vec![],
			fields : vec![],
		};
		assert_eq!(
			cache.update_with(removal.clone()),
			Ok(Update_Ok::Withdrawal)
		);
		assert!(cache.lookup(256).is_none());
	}

	#[test]
	fn different_redefinition() {
		let mut cache = Template_Cache::default();

		let template = Template_Record {
			header : Template_Record_Header {
				template_id : 256,
				field_count : 1,
				scope_field_count : 0,
			},
			scope_fields : vec![],
			fields : vec![DUMMY_FIELD],
		};
		let mut template2 = template.clone();
		assert_eq!(cache.update_with(template.clone()), Ok(Update_Ok::Addition));
		template2.header.field_count = 2;
		template2.fields.push(DUMMY_FIELD);
		assert_eq!(
			cache.update_with(template2.clone()),
			Err(Update_Err::Redefinition_Different)
		);

		assert!(cache.lookup(256).is_none());
	}

	#[test]
	fn spurious_withdrawal() {
		let mut cache = Template_Cache::default();
		assert_eq!(
			cache.update_with(Template_Record {
				header : Template_Record_Header {
					template_id : 256,
					field_count : 0,
					scope_field_count : 0,
				},
				scope_fields : vec![],
				fields : vec![],
			}),
			Err(Update_Err::Withdrawal_Unknown)
		);
		assert!(cache.lookup(256).is_none());
	}

	#[test]
	#[should_panic(expected = "assertion failed")]
	fn panic_on_nontemplate() {
		let mut cache = Template_Cache::default();
		let _res = cache.update_with(Template_Record {
			header : Template_Record_Header {
				template_id : 0,
				field_count : 0,
				scope_field_count : 0,
			},
			scope_fields : vec![],
			fields : vec![],
		});
	}

	#[test]
	#[should_panic(expected = "assertion failed")]
	fn panic_on_wrong_size() {
		let mut cache = Template_Cache::default();
		let _res = cache.update_with(Template_Record {
			header : Template_Record_Header {
				template_id : 0,
				field_count : 1,
				scope_field_count : 0,
			},
			scope_fields : vec![],
			fields : vec![],
		});
	}
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Verify_Template_Error {
	Field_Count_Invalid(u16),
	Scope_Field_Count_Mismatch {
		count_header : u16,
		len : usize,
	},
	Field_Count_Mismatch {
		field_count_header : u16,
		scope_field_count_header : u16,
		fields_len : usize,
	},
	Information_Element_Id_Not_Found(u16),
	Field_Length_Invalid(u16),
	Field_Length_Mismatch {
		length : u16,
		type_ : Abstract_Data_Type,
	},
	Field_Length_Not_Implemented {
		length : u16,
		type_ : Abstract_Data_Type,
	},
	Type_Not_Implemented(Abstract_Data_Type),
	Enterprise_Numbers_Not_Implemented,
}

impl std::fmt::Display for Verify_Template_Error {
	fn fmt(&self, f : &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
		use Verify_Template_Error::*;

		match *self {
			Field_Count_Invalid(c) => write!(f, "field count {} is invalid", c),
			Scope_Field_Count_Mismatch { count_header, len } => write!(
				f,
				"scope field count is {}, but template has {} scope fields",
				count_header, len
			),
			Field_Count_Mismatch {
				field_count_header,
				scope_field_count_header,
				fields_len,
			} => write!(
				f,
				"field count is {} and scope field count is {}.\
				 Expected {} non-scope fields, but template has {} non-scope fields",
				field_count_header,
				scope_field_count_header,
				field_count_header - scope_field_count_header,
				fields_len,
			),
			Information_Element_Id_Not_Found(id) => {
				write!(f, "information element with id {} not found", id)
			}
			Field_Length_Invalid(len) => write!(f, "field length {} is invalid", len),
			Field_Length_Mismatch { length, type_ } => write!(
				f,
				"type {} may not be encoded with length {}",
				type_, length
			),
			Field_Length_Not_Implemented { length, type_ } => {
				write!(f, "length {} not implemented for type {}", type_, length)
			}
			Type_Not_Implemented(type_) => write!(f, "type {} not implemented", type_),
			Enterprise_Numbers_Not_Implemented => {
				write!(f, "enterprise numbers are not implemented")
			}
		}
	}
}

pub fn verify_template(template : &Template_Record) -> Result<(), Verify_Template_Error> {
	use Verify_Template_Error::*;

	if template.header.field_count == 0 {
		return Err(Field_Count_Invalid(template.header.field_count));
	}

	if template.header.scope_field_count as usize != template.scope_fields.len() {
		return Err(Scope_Field_Count_Mismatch {
			count_header : template.header.scope_field_count,
			len : template.scope_fields.len(),
		});
	}
	if template.header.field_count as usize - template.header.scope_field_count as usize
		!= template.fields.len()
	{
		return Err(Field_Count_Mismatch {
			field_count_header : template.header.field_count,
			scope_field_count_header : template.header.scope_field_count,
			fields_len : template.fields.len(),
		});
	}

	for field in template.scope_fields.iter().chain(template.fields.iter()) {
		verify_field_specifier(field)?;
	}

	Ok(())
}

fn verify_field_specifier(field : &Field_Specifier) -> Result<(), Verify_Template_Error> {
	use Abstract_Data_Type::*;
	use Verify_Template_Error::*;

	let information_element = information_element::lookup(field.information_element_id)
		.ok_or_else(|| (Information_Element_Id_Not_Found(field.information_element_id)))?;

	if field.field_length == 0 {
		return Err(Field_Length_Invalid(field.field_length));
	}

	let type_ = information_element.abstract_data_type;
	let length = field.field_length;

	// check length
	match type_ {
		// different lengths not implemented
		unsigned8 | signed8 | boolean => match length {
			1 => {}
			_ => return Err(Field_Length_Mismatch { length, type_ }),
		},
		unsigned16 | signed16 => match length {
			1 | 2 => {}
			_ => return Err(Field_Length_Mismatch { length, type_ }),
		},
		unsigned32 | signed32 => match length {
			1 | 2 | 4 => {}
			3 => return Err(Field_Length_Not_Implemented { length, type_ }),
			_ => return Err(Field_Length_Mismatch { length, type_ }),
		},
		unsigned64 | signed64 => match length {
			1 | 2 | 4 | 8 => {}
			3 | 5 | 6 | 7 => return Err(Field_Length_Not_Implemented { length, type_ }),
			_ => return Err(Field_Length_Mismatch { length, type_ }),
		},
		float32 | dateTimeSeconds | ipv4Address => match length {
			4 => {}
			_ => return Err(Field_Length_Mismatch { length, type_ }),
		},
		float64 => match length {
			4 | 8 => {}
			_ => return Err(Field_Length_Mismatch { length, type_ }),
		},
		macAddress => match length {
			6 => {}
			_ => return Err(Field_Length_Mismatch { length, type_ }),
		},
		octetArray | string => {}
		dateTimeMilliseconds | dateTimeMicroseconds | dateTimeNanoseconds => match length {
			8 => {}
			_ => return Err(Field_Length_Mismatch { length, type_ }),
		},
		ipv6Address => match length {
			16 => {}
			_ => return Err(Field_Length_Mismatch { length, type_ }),
		},
		basicList | subTemplateList | subTemplateMultiList => {
			return Err(Type_Not_Implemented(type_))
		}
	};

	match field.enterprise_number {
		Some(_) => Err(Enterprise_Numbers_Not_Implemented),
		None => Ok(()),
	}
}

#[cfg(test)]
mod verify_template_tests {
	use super::*;
	const DUMMY_FIELD : Field_Specifier = Field_Specifier {
		information_element_id : 210,
		field_length : 4,
		enterprise_number : None,
	};

	#[test]
	fn verify_template_test() {
		let template = Template_Record {
			header : Template_Record_Header {
				template_id : 256,
				scope_field_count : 0,
				field_count : 1,
			},
			scope_fields : vec![],
			fields : vec![DUMMY_FIELD],
		};
		assert!(verify_template(&template).is_ok());

		let template = Template_Record {
			header : Template_Record_Header {
				template_id : 256,
				scope_field_count : 1,
				field_count : 1,
			},
			scope_fields : vec![DUMMY_FIELD],
			fields : vec![],
		};
		assert!(verify_template(&template).is_ok());

		let template = Template_Record {
			header : Template_Record_Header {
				template_id : 256,
				scope_field_count : 1,
				field_count : 2,
			},
			scope_fields : vec![DUMMY_FIELD],
			fields : vec![DUMMY_FIELD],
		};
		assert!(verify_template(&template).is_ok());

		let template = Template_Record {
			header : Template_Record_Header {
				template_id : 256,
				scope_field_count : 0,
				field_count : 0,
			},
			scope_fields : vec![],
			fields : vec![],
		};
		assert!(verify_template(&template).is_err());

		let template = Template_Record {
			header : Template_Record_Header {
				template_id : 256,
				scope_field_count : 2,
				field_count : 1,
			},
			scope_fields : vec![],
			fields : vec![],
		};
		assert!(verify_template(&template).is_err());

		let template = Template_Record {
			header : Template_Record_Header {
				template_id : 256,
				scope_field_count : 1,
				field_count : 2,
			},
			scope_fields : vec![DUMMY_FIELD],
			fields : vec![],
		};
		assert!(verify_template(&template).is_err());
	}

	#[test]
	fn verify_field_specifier_test() {
		assert!(verify_field_specifier(&DUMMY_FIELD).is_ok());

		let field = Field_Specifier {
			information_element_id : 0xffff,
			field_length : 1,
			enterprise_number : None,
		};
		assert!(verify_field_specifier(&field).is_err());

		let field = Field_Specifier {
			information_element_id : 210,
			field_length : 0,
			enterprise_number : None,
		};
		assert!(verify_field_specifier(&field).is_err());

		let field = Field_Specifier {
			information_element_id : 210,
			field_length : 1,
			enterprise_number : Some(32473),
		};
		assert!(verify_field_specifier(&field).is_err());

		let field = Field_Specifier {
			information_element_id : 1,
			field_length : 256,
			enterprise_number : None,
		};
		assert!(verify_field_specifier(&field).is_err());
	}
}
