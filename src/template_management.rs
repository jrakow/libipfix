use information_element;
use std;
use structs::*;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct TemplateCache {
	templates : std::collections::HashMap<u16, TemplateRecord>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UpdateOk {
	Addition,
	Redefinition,
	Withdrawal,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UpdateErr {
	RedefinitionDifferent,
	WithdrawalUnknown,
}

impl TemplateCache {
	pub fn update_with(&mut self, template : TemplateRecord) -> Result<UpdateOk, UpdateErr> {
		use std::collections::hash_map::Entry::*;

		assert!(template.header.template_id >= FIRST_TEMPLATE_ID);
		assert!(template.header.scope_field_count as usize == template.scope_fields.len());
		assert!(
			template.header.field_count as usize - template.header.scope_field_count as usize
				== template.fields.len()
		);

		if template.header.field_count == 0 {
			// template withdrawal
			self.templates
				.remove(&template.header.template_id)
				.map(|_| UpdateOk::Withdrawal)
				.ok_or(UpdateErr::WithdrawalUnknown)
		} else {
			match self.templates.entry(template.header.template_id) {
				Occupied(ref entry) if &template == entry.get() => Ok(UpdateOk::Redefinition),
				Occupied(entry) => {
					// template != entry
					entry.remove();
					Err(UpdateErr::RedefinitionDifferent)
				}
				Vacant(entry) => {
					entry.insert(template);
					Ok(UpdateOk::Addition)
				}
			}
		}
	}

	pub fn lookup(&self, id : u16) -> Option<&TemplateRecord> {
		self.templates.get(&id)
	}
}

#[cfg(test)]
mod template_cache_tests {
	use super::*;
	const DUMMY_FIELD : FieldSpecifier = FieldSpecifier {
		information_element_id : 210,
		field_length : 4,
		enterprise_number : None,
	};

	#[test]
	fn normal_case() {
		let mut cache = TemplateCache::default();
		let template = TemplateRecord {
			header : TemplateRecordHeader {
				template_id : FIRST_TEMPLATE_ID,
				field_count : 1,
				scope_field_count : 0,
			},
			scope_fields : vec![],
			fields : vec![DUMMY_FIELD],
		};
		assert_eq!(cache.update_with(template.clone()), Ok(UpdateOk::Addition));
		assert_eq!(cache.lookup(FIRST_TEMPLATE_ID).unwrap(), &template);

		// identical redefinition
		assert_eq!(
			cache.update_with(template.clone()),
			Ok(UpdateOk::Redefinition)
		);
		assert_eq!(cache.lookup(FIRST_TEMPLATE_ID).unwrap(), &template);

		let removal = TemplateRecord {
			header : TemplateRecordHeader {
				template_id : FIRST_TEMPLATE_ID,
				field_count : 0,
				scope_field_count : 0,
			},
			scope_fields : vec![],
			fields : vec![],
		};
		assert_eq!(cache.update_with(removal.clone()), Ok(UpdateOk::Withdrawal));
		assert!(cache.lookup(FIRST_TEMPLATE_ID).is_none());
	}

	#[test]
	fn different_redefinition() {
		let mut cache = TemplateCache::default();

		let template = TemplateRecord {
			header : TemplateRecordHeader {
				template_id : FIRST_TEMPLATE_ID,
				field_count : 1,
				scope_field_count : 0,
			},
			scope_fields : vec![],
			fields : vec![DUMMY_FIELD],
		};
		let mut template2 = template.clone();
		assert_eq!(cache.update_with(template.clone()), Ok(UpdateOk::Addition));
		template2.header.field_count = 2;
		template2.fields.push(DUMMY_FIELD);
		assert_eq!(
			cache.update_with(template2.clone()),
			Err(UpdateErr::RedefinitionDifferent)
		);

		assert!(cache.lookup(FIRST_TEMPLATE_ID).is_none());
	}

	#[test]
	fn spurious_withdrawal() {
		let mut cache = TemplateCache::default();
		assert_eq!(
			cache.update_with(TemplateRecord {
				header : TemplateRecordHeader {
					template_id : FIRST_TEMPLATE_ID,
					field_count : 0,
					scope_field_count : 0,
				},
				scope_fields : vec![],
				fields : vec![],
			}),
			Err(UpdateErr::WithdrawalUnknown)
		);
		assert!(cache.lookup(FIRST_TEMPLATE_ID).is_none());
	}

	#[test]
	#[should_panic(expected = "assertion failed")]
	fn panic_on_nontemplate() {
		let mut cache = TemplateCache::default();
		let _res = cache.update_with(TemplateRecord {
			header : TemplateRecordHeader {
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
		let mut cache = TemplateCache::default();
		let _res = cache.update_with(TemplateRecord {
			header : TemplateRecordHeader {
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
pub enum VerifyTemplateError {
	FieldCountInvalid(u16),
	ScopeFieldCountMismatch {
		count_header : u16,
		len : usize,
	},
	FieldCountMismatch {
		field_count_header : u16,
		scope_field_count_header : u16,
		fields_len : usize,
	},
	InformationElementIdNotFound(u16),
	FieldLengthInvalid(u16),
	FieldLengthMismatch {
		length : u16,
		type_ : AbstractDataType,
	},
	FieldLengthNotImplemented {
		length : u16,
		type_ : AbstractDataType,
	},
	TypeNotImplemented(AbstractDataType),
	EnterpriseNumbersNotImplemented,
}

impl std::fmt::Display for VerifyTemplateError {
	fn fmt(&self, f : &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
		use VerifyTemplateError::*;

		match *self {
			FieldCountInvalid(c) => write!(f, "field count {} is invalid", c),
			ScopeFieldCountMismatch { count_header, len } => write!(
				f,
				"scope field count is {}, but template has {} scope fields",
				count_header, len
			),
			FieldCountMismatch {
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
			InformationElementIdNotFound(id) => {
				write!(f, "information element with id {} not found", id)
			}
			FieldLengthInvalid(len) => write!(f, "field length {} is invalid", len),
			FieldLengthMismatch { length, type_ } => write!(
				f,
				"type {} may not be encoded with length {}",
				type_, length
			),
			FieldLengthNotImplemented { length, type_ } => {
				write!(f, "length {} not implemented for type {}", type_, length)
			}
			TypeNotImplemented(type_) => write!(f, "type {} not implemented", type_),
			EnterpriseNumbersNotImplemented => write!(f, "enterprise numbers are not implemented"),
		}
	}
}

pub fn verify_template(template : &TemplateRecord) -> Result<(), VerifyTemplateError> {
	use VerifyTemplateError::*;

	if template.header.field_count == 0 {
		return Err(FieldCountInvalid(template.header.field_count));
	}

	if template.header.scope_field_count as usize != template.scope_fields.len() {
		return Err(ScopeFieldCountMismatch {
			count_header : template.header.scope_field_count,
			len : template.scope_fields.len(),
		});
	}
	if template.header.field_count as usize - template.header.scope_field_count as usize
		!= template.fields.len()
	{
		return Err(FieldCountMismatch {
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

fn verify_field_specifier(field : &FieldSpecifier) -> Result<(), VerifyTemplateError> {
	use AbstractDataType::*;
	use VerifyTemplateError::*;

	let information_element = information_element::lookup(field.information_element_id)
		.ok_or_else(|| (InformationElementIdNotFound(field.information_element_id)))?;

	if field.field_length == 0 {
		return Err(FieldLengthInvalid(field.field_length));
	}

	let type_ = information_element.abstract_data_type;
	let length = field.field_length;

	// check length
	match type_ {
		// different lengths not implemented
		Unsigned8 | Signed8 | Boolean => match length {
			1 => {}
			_ => return Err(FieldLengthMismatch { length, type_ }),
		},
		Unsigned16 | Signed16 => match length {
			1 | 2 => {}
			_ => return Err(FieldLengthMismatch { length, type_ }),
		},
		Unsigned32 | Signed32 => match length {
			1 | 2 | 4 => {}
			3 => return Err(FieldLengthNotImplemented { length, type_ }),
			_ => return Err(FieldLengthMismatch { length, type_ }),
		},
		Unsigned64 | Signed64 => match length {
			1 | 2 | 4 | 8 => {}
			3 | 5 | 6 | 7 => return Err(FieldLengthNotImplemented { length, type_ }),
			_ => return Err(FieldLengthMismatch { length, type_ }),
		},
		Float32 | DateTimeSeconds | Ipv4Address => match length {
			4 => {}
			_ => return Err(FieldLengthMismatch { length, type_ }),
		},
		Float64 => match length {
			4 | 8 => {}
			_ => return Err(FieldLengthMismatch { length, type_ }),
		},
		MacAddress => match length {
			6 => {}
			_ => return Err(FieldLengthMismatch { length, type_ }),
		},
		OctetArray | String => {}
		DateTimeMilliseconds | DateTimeMicroseconds | DateTimeNanoseconds => match length {
			8 => {}
			_ => return Err(FieldLengthMismatch { length, type_ }),
		},
		Ipv6Address => match length {
			16 => {}
			_ => return Err(FieldLengthMismatch { length, type_ }),
		},
		BasicList | SubTemplateList | SubTemplateMultiList => return Err(TypeNotImplemented(type_)),
	};

	match field.enterprise_number {
		Some(_) => Err(EnterpriseNumbersNotImplemented),
		None => Ok(()),
	}
}

#[cfg(test)]
mod verify_template_tests {
	use super::*;
	const DUMMY_FIELD : FieldSpecifier = FieldSpecifier {
		information_element_id : 210,
		field_length : 4,
		enterprise_number : None,
	};

	#[test]
	fn verify_template_test() {
		let template = TemplateRecord {
			header : TemplateRecordHeader {
				template_id : FIRST_TEMPLATE_ID,
				scope_field_count : 0,
				field_count : 1,
			},
			scope_fields : vec![],
			fields : vec![DUMMY_FIELD],
		};
		assert!(verify_template(&template).is_ok());

		let template = TemplateRecord {
			header : TemplateRecordHeader {
				template_id : FIRST_TEMPLATE_ID,
				scope_field_count : 1,
				field_count : 1,
			},
			scope_fields : vec![DUMMY_FIELD],
			fields : vec![],
		};
		assert!(verify_template(&template).is_ok());

		let template = TemplateRecord {
			header : TemplateRecordHeader {
				template_id : FIRST_TEMPLATE_ID,
				scope_field_count : 1,
				field_count : 2,
			},
			scope_fields : vec![DUMMY_FIELD],
			fields : vec![DUMMY_FIELD],
		};
		assert!(verify_template(&template).is_ok());

		let template = TemplateRecord {
			header : TemplateRecordHeader {
				template_id : FIRST_TEMPLATE_ID,
				scope_field_count : 0,
				field_count : 0,
			},
			scope_fields : vec![],
			fields : vec![],
		};
		assert!(verify_template(&template).is_err());

		let template = TemplateRecord {
			header : TemplateRecordHeader {
				template_id : FIRST_TEMPLATE_ID,
				scope_field_count : 2,
				field_count : 1,
			},
			scope_fields : vec![],
			fields : vec![],
		};
		assert!(verify_template(&template).is_err());

		let template = TemplateRecord {
			header : TemplateRecordHeader {
				template_id : FIRST_TEMPLATE_ID,
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

		let field = FieldSpecifier {
			information_element_id : 0xffff,
			field_length : 1,
			enterprise_number : None,
		};
		assert!(verify_field_specifier(&field).is_err());

		let field = FieldSpecifier {
			information_element_id : 210,
			field_length : 0,
			enterprise_number : None,
		};
		assert!(verify_field_specifier(&field).is_err());

		let field = FieldSpecifier {
			information_element_id : 210,
			field_length : 1,
			enterprise_number : Some(32473),
		};
		assert!(verify_field_specifier(&field).is_err());

		let field = FieldSpecifier {
			information_element_id : 1,
			field_length : FIRST_TEMPLATE_ID,
			enterprise_number : None,
		};
		assert!(verify_field_specifier(&field).is_err());
	}
}
