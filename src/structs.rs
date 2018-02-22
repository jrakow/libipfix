use std;

pub const MESSAGE_HEADER_LENGTH : u16 = 16;
pub const SET_HEADER_LENGTH : u16 = 4;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Message<'a> {
	pub header : Message_Header,
	pub sets : Vec<(Set_Header, &'a [u8])>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Message_Header {
	pub version_number : u16,
	pub length : u16,
	pub export_time : u32,
	pub sequence_number : u32,
	pub observation_domain_id : u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Field_Specifier {
	pub information_element_id : u16,
	pub field_length : u16,
	pub enterprise_number : Option<u32>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Set_Header {
	pub set_id : u16,
	pub length : u16,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Template_Record {
	pub header : Template_Record_Header,
	pub scope_fields : Vec<Field_Specifier>,
	pub fields : Vec<Field_Specifier>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Template_Record_Header {
	pub template_id : u16,
	pub field_count : u16,
	pub scope_field_count : u16,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Data_Record {
	pub fields : Vec<Data_Value>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Abstract_Data_Type {
	unsigned8,
	unsigned16,
	unsigned32,
	unsigned64,
	signed8,
	signed16,
	signed32,
	signed64,
	float32,
	float64,
	boolean,
	macAddress,
	octetArray,
	string,
	dateTimeSeconds,
	dateTimeMilliseconds,
	dateTimeMicroseconds,
	dateTimeNanoseconds,
	ipv4Address,
	ipv6Address,
	// TODO
	basicList,
	subTemplateList,
	subTemplateMultiList,
}

impl std::fmt::Display for Abstract_Data_Type {
	fn fmt(&self, f : &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
		write!(f, "{:?}", self)
	}
}

#[derive(Clone, Debug, PartialEq)]
pub enum Data_Value {
	unsigned8(u8),
	unsigned16(u16),
	unsigned32(u32),
	unsigned64(u64),
	signed8(i8),
	signed16(i16),
	signed32(i32),
	signed64(i64),
	float32(f32),
	float64(f64),
	boolean(bool),
	macAddress(Vec<u8>), // always length 6
	octetArray(Vec<u8>),
	string(String),
	dateTimeSeconds(u32),
	dateTimeMilliseconds(u64),
	dateTimeMicroseconds(u32, u32),
	dateTimeNanoseconds(u32, u32),
	ipv4Address(Vec<u8>), // always length 4
	ipv6Address(Vec<u8>), // always length 16
	// TODO
	basicList,
	subTemplateList,
	subTemplateMultiList,
}

impl std::fmt::Display for Data_Value {
	fn fmt(&self, f : &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
		use Data_Value::*;

		match *self {
			unsigned8(u) => write!(f, "{}", u),
			unsigned16(u) => write!(f, "{}", u),
			unsigned32(u) | dateTimeSeconds(u) => write!(f, "{}", u),
			unsigned64(u) | dateTimeMilliseconds(u) => write!(f, "{}", u),
			signed8(i) => write!(f, "{}", i),
			signed16(i) => write!(f, "{}", i),
			signed32(i) => write!(f, "{}", i),
			signed64(i) => write!(f, "{}", i),
			float32(g) => write!(f, "{}", g),
			float64(g) => write!(f, "{}", g),
			boolean(b) => write!(f, "{}", b),
			macAddress(ref arr) => {
				for i in (1..6).rev() {
					match write!(f, "{:X}-", arr[i]) {
						Ok(_) => {}
						Err(e) => return Err(e),
					}
				}
				write!(f, "{:X}", arr[0])
			}
			octetArray(ref arr) => write!(f, "{:?}", arr),
			string(ref s) => write!(f, "{}", s),
			dateTimeMicroseconds(sec, frac) | dateTimeNanoseconds(sec, frac) => write!(
				f,
				r#"{{
					"seconds" : {},
					"fraction" : {}
				}}"#,
				sec, frac
			),
			ipv4Address(ref arr) => {
				for i in (1..4).rev() {
					match write!(f, "{}.", arr[i]) {
						Ok(_) => {}
						Err(e) => return Err(e),
					}
				}
				write!(f, "{}", arr[0])
			}
			ipv6Address(ref arr) => {
				for i in (1..16).rev() {
					match write!(f, "{:x}:", arr[i]) {
						Ok(_) => {}
						Err(e) => return Err(e),
					}
				}
				write!(f, "{:x}", arr[0])
			}
			basicList | subTemplateList | subTemplateMultiList => Err(std::fmt::Error),
		}
	}
}
