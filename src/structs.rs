use std;
pub use std::net::{Ipv4Addr, Ipv6Addr};

use serde::ser::{Serialize, SerializeMap, SerializeTupleStruct, Serializer};

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
	dateTimeMicroseconds { seconds : u32, fraction : u32 },
	dateTimeNanoseconds { seconds : u32, fraction : u32 },
	ipv4Address(Ipv4Addr),
	ipv6Address(Ipv6Addr),
	// TODO
	basicList,
	subTemplateList,
	subTemplateMultiList,
}

impl Serialize for Data_Value {
	fn serialize<S>(&self, s : S) -> Result<S::Ok, S::Error>
	where
		S : Serializer,
	{
		use Data_Value::*;

		match self {
			&unsigned8(u) => s.serialize_u8(u),
			&unsigned16(u) => s.serialize_u16(u),
			&unsigned32(u) | &dateTimeSeconds(u) => s.serialize_u32(u),
			&unsigned64(u) | &dateTimeMilliseconds(u) => s.serialize_u64(u),
			&signed8(u) => s.serialize_i8(u),
			&signed16(u) => s.serialize_i16(u),
			&signed32(u) => s.serialize_i32(u),
			&signed64(u) => s.serialize_i64(u),
			&float32(u) => s.serialize_f32(u),
			&float64(u) => s.serialize_f64(u),
			&boolean(u) => s.serialize_bool(u),
			&macAddress(ref addr) => s.serialize_str(&format!(
				"{:02X}-{:02X}-{:02X}-{:02X}-{:02X}-{:02X}",
				addr[0], addr[1], addr[2], addr[3], addr[4], addr[5]
			)),
			&octetArray(ref arr) => s.serialize_bytes(&arr),
			&string(ref st) => s.serialize_str(&st),
			&dateTimeMicroseconds { seconds, fraction }
			| &dateTimeNanoseconds { seconds, fraction } => {
				let mut ts = s.serialize_tuple_struct("", 2)?;
				ts.serialize_field(&seconds)?;
				ts.serialize_field(&fraction)?;
				ts.end()
			}
			&ipv4Address(addr) => s.serialize_str(&format!("{}", addr)),
			&ipv6Address(addr) => s.serialize_str(&format!("{}", addr)),
			// TODO
			&basicList | &subTemplateList | &subTemplateMultiList => unimplemented!(),
		}
	}
}

pub struct Typed_Data_Record<'a> {
	data : &'a Data_Record,
	template : &'a Template_Record,
}

impl<'a> Serialize for Typed_Data_Record<'a> {
	fn serialize<S>(&self, s : S) -> Result<S::Ok, S::Error>
	where
		S : Serializer,
	{
		let mut map = s.serialize_map(Some(self.template.fields.len()))?;

		for (specifier, value) in self.template
			.scope_fields
			.iter()
			.chain(self.template.fields.iter())
			.zip(self.data.fields.iter())
		{
			map.serialize_key(&specifier.information_element_id)?;
			map.serialize_value(value)?;
		}
		map.end()
	}
}

#[cfg(test)]
mod tests {
	extern crate serde_json;
	use self::serde_json::to_string;

	use super::*;

	#[test]
	pub fn data_value_json_test() {
		use Data_Value::*;

		assert_eq!(to_string(&unsigned8(255)).unwrap(), "255");
		assert_eq!(to_string(&unsigned16(65535)).unwrap(), "65535");
		assert_eq!(to_string(&unsigned32(4294967295)).unwrap(), "4294967295");
		assert_eq!(
			to_string(&unsigned64(18446744073709551615)).unwrap(),
			"18446744073709551615"
		);

		assert_eq!(to_string(&signed8(-5)).unwrap(), "-5");
		assert_eq!(to_string(&signed16(-500)).unwrap(), "-500");
		assert_eq!(to_string(&signed32(-500000000)).unwrap(), "-500000000");
		assert_eq!(
			to_string(&signed64(-5000000000000)).unwrap(),
			"-5000000000000"
		);

		assert_eq!(to_string(&float32(32.0)).unwrap(), "32.0");
		assert_eq!(to_string(&float32(64.0)).unwrap(), "64.0");

		assert_eq!(to_string(&boolean(true)).unwrap(), "true");
		assert_eq!(to_string(&boolean(false)).unwrap(), "false");

		assert_eq!(
			to_string(&macAddress([0x00, 0x01, 0x02, 0x03, 0x04, 0x05].to_vec())).unwrap(),
			"\"00-01-02-03-04-05\""
		);
		assert_eq!(
			to_string(&octetArray([0x00, 0x01, 0x02, 0x03, 0x04, 0x05].to_vec())).unwrap(),
			"[0,1,2,3,4,5]"
		);

		assert_eq!(to_string(&string("💖".to_string())).unwrap(), "\"💖\"");

		assert_eq!(to_string(&dateTimeSeconds(3600)).unwrap(), "3600");
		assert_eq!(
			to_string(&dateTimeMilliseconds(3_600_000)).unwrap(),
			"3600000"
		);

		assert_eq!(
			to_string(&dateTimeMicroseconds {
				seconds : 3600,
				fraction : 1,
			}).unwrap(),
			"[3600,1]"
		);
		assert_eq!(
			to_string(&dateTimeNanoseconds {
				seconds : 3600,
				fraction : 1,
			}).unwrap(),
			"[3600,1]"
		);

		assert_eq!(
			to_string(&ipv4Address(std::net::Ipv4Addr::new(127, 0, 0, 1))).unwrap(),
			"\"127.0.0.1\""
		);
		assert_eq!(
			to_string(&ipv6Address(std::net::Ipv6Addr::new(
				0,
				0,
				0,
				0,
				0,
				0,
				0,
				1
			))).unwrap(),
			"\"::1\""
		);
		/* TODO
		basicList,
		subTemplateList,
		subTemplateMultiList,
		*/
	}

	#[test]
	pub fn data_record_json_test() {
		let template = Template_Record {
			header : Template_Record_Header {
				template_id : 256,
				field_count : 4,
				scope_field_count : 0,
			},
			fields : vec![
				Field_Specifier {
					information_element_id : 210,
					field_length : 4,
					enterprise_number : None,
				},
				Field_Specifier {
					information_element_id : 210,
					field_length : 4,
					enterprise_number : None,
				},
			],
			scope_fields : vec![],
		};
		let record = Data_Record {
			fields : vec![
				Data_Value::octetArray(vec![0x00, 0x00, 0x00, 0x00]),
				Data_Value::octetArray(vec![0x00, 0x00, 0x00, 0x00]),
			],
		};
		let typed = Typed_Data_Record {
			template : &template,
			data : &record,
		};
		assert_eq!(
			serde_json::to_string(&typed).unwrap(),
			"{\"210\":[0,0,0,0],\"210\":[0,0,0,0]}"
		);
	}
}
