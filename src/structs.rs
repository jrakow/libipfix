use std;
pub use std::net::{Ipv4Addr, Ipv6Addr};

use serde::ser::{Serialize, SerializeMap, SerializeTupleStruct, Serializer};

pub const MESSAGE_HEADER_LENGTH : u16 = 16;
pub const SET_HEADER_LENGTH : u16 = 4;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Message<'a> {
	pub header : MessageHeader,
	pub sets : Vec<(SetHeader, &'a [u8])>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MessageHeader {
	pub version_number : u16,
	pub length : u16,
	pub export_time : u32,
	pub sequence_number : u32,
	pub observation_domain_id : u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FieldSpecifier {
	pub information_element_id : u16,
	pub field_length : u16,
	pub enterprise_number : Option<u32>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SetHeader {
	pub set_id : u16,
	pub length : u16,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TemplateRecord {
	pub header : TemplateRecordHeader,
	pub scope_fields : Vec<FieldSpecifier>,
	pub fields : Vec<FieldSpecifier>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TemplateRecordHeader {
	pub template_id : u16,
	pub field_count : u16,
	pub scope_field_count : u16,
}

#[derive(Clone, Debug, PartialEq)]
pub struct DataRecord {
	pub fields : Vec<DataValue>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AbstractDataType {
	Unsigned8,
	Unsigned16,
	Unsigned32,
	Unsigned64,
	Signed8,
	Signed16,
	Signed32,
	Signed64,
	Float32,
	Float64,
	Boolean,
	MacAddress,
	OctetArray,
	String,
	DateTimeSeconds,
	DateTimeMilliseconds,
	DateTimeMicroseconds,
	DateTimeNanoseconds,
	Ipv4Address,
	Ipv6Address,
	// TODO
	BasicList,
	SubTemplateList,
	SubTemplateMultiList,
}

impl std::fmt::Display for AbstractDataType {
	fn fmt(&self, f : &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
		write!(f, "{:?}", self)
	}
}

#[derive(Clone, Debug, PartialEq)]
pub enum DataValue {
	Unsigned8(u8),
	Unsigned16(u16),
	Unsigned32(u32),
	Unsigned64(u64),
	Signed8(i8),
	Signed16(i16),
	Signed32(i32),
	Signed64(i64),
	Float32(f32),
	Float64(f64),
	Boolean(bool),
	MacAddress(Vec<u8>), // always length 6
	OctetArray(Vec<u8>),
	String(String),
	DateTimeSeconds(u32),
	DateTimeMilliseconds(u64),
	DateTimeMicroseconds { seconds : u32, fraction : u32 },
	DateTimeNanoseconds { seconds : u32, fraction : u32 },
	Ipv4Address(Ipv4Addr),
	Ipv6Address(Ipv6Addr),
	// TODO
	BasicList,
	SubTemplateList,
	SubTemplateMultiList,
}

impl Serialize for DataValue {
	fn serialize<S>(&self, s : S) -> Result<S::Ok, S::Error>
	where
		S : Serializer,
	{
		use DataValue::*;

		match *self {
			Unsigned8(u) => s.serialize_u8(u),
			Unsigned16(u) => s.serialize_u16(u),
			Unsigned32(u) | DateTimeSeconds(u) => s.serialize_u32(u),
			Unsigned64(u) | DateTimeMilliseconds(u) => s.serialize_u64(u),
			Signed8(u) => s.serialize_i8(u),
			Signed16(u) => s.serialize_i16(u),
			Signed32(u) => s.serialize_i32(u),
			Signed64(u) => s.serialize_i64(u),
			Float32(u) => s.serialize_f32(u),
			Float64(u) => s.serialize_f64(u),
			Boolean(u) => s.serialize_bool(u),
			MacAddress(ref addr) => s.serialize_str(&format!(
				"{:02X}-{:02X}-{:02X}-{:02X}-{:02X}-{:02X}",
				addr[0], addr[1], addr[2], addr[3], addr[4], addr[5]
			)),
			OctetArray(ref arr) => s.serialize_bytes(arr),
			String(ref st) => s.serialize_str(st),
			DateTimeMicroseconds { seconds, fraction }
			| DateTimeNanoseconds { seconds, fraction } => {
				let mut ts = s.serialize_tuple_struct("", 2)?;
				ts.serialize_field(&seconds)?;
				ts.serialize_field(&fraction)?;
				ts.end()
			}
			Ipv4Address(addr) => s.serialize_str(&format!("{}", addr)),
			Ipv6Address(addr) => s.serialize_str(&format!("{}", addr)),
			// TODO
			BasicList | SubTemplateList | SubTemplateMultiList => unimplemented!(),
		}
	}
}

pub struct TypedDataRecord<'a> {
	data : &'a DataRecord,
	template : &'a TemplateRecord,
}

impl<'a> Serialize for TypedDataRecord<'a> {
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
		use DataValue::*;

		assert_eq!(to_string(&Unsigned8(255)).unwrap(), "255");
		assert_eq!(to_string(&Unsigned16(65535)).unwrap(), "65535");
		assert_eq!(to_string(&Unsigned32(4294967295)).unwrap(), "4294967295");
		assert_eq!(
			to_string(&Unsigned64(18446744073709551615)).unwrap(),
			"18446744073709551615"
		);

		assert_eq!(to_string(&Signed8(-5)).unwrap(), "-5");
		assert_eq!(to_string(&Signed16(-500)).unwrap(), "-500");
		assert_eq!(to_string(&Signed32(-500000000)).unwrap(), "-500000000");
		assert_eq!(
			to_string(&Signed64(-5000000000000)).unwrap(),
			"-5000000000000"
		);

		assert_eq!(to_string(&Float32(32.0)).unwrap(), "32.0");
		assert_eq!(to_string(&Float64(64.0)).unwrap(), "64.0");

		assert_eq!(to_string(&Boolean(true)).unwrap(), "true");
		assert_eq!(to_string(&Boolean(false)).unwrap(), "false");

		assert_eq!(
			to_string(&MacAddress([0x00, 0x01, 0x02, 0x03, 0x04, 0x05].to_vec())).unwrap(),
			"\"00-01-02-03-04-05\""
		);
		assert_eq!(
			to_string(&OctetArray([0x00, 0x01, 0x02, 0x03, 0x04, 0x05].to_vec())).unwrap(),
			"[0,1,2,3,4,5]"
		);

		assert_eq!(to_string(&String("💖".to_string())).unwrap(), "\"💖\"");

		assert_eq!(to_string(&DateTimeSeconds(3600)).unwrap(), "3600");
		assert_eq!(
			to_string(&DateTimeMilliseconds(3_600_000)).unwrap(),
			"3600000"
		);

		assert_eq!(
			to_string(&DateTimeMicroseconds {
				seconds : 3600,
				fraction : 1,
			}).unwrap(),
			"[3600,1]"
		);
		assert_eq!(
			to_string(&DateTimeNanoseconds {
				seconds : 3600,
				fraction : 1,
			}).unwrap(),
			"[3600,1]"
		);

		assert_eq!(
			to_string(&Ipv4Address(std::net::Ipv4Addr::new(127, 0, 0, 1))).unwrap(),
			"\"127.0.0.1\""
		);
		assert_eq!(
			to_string(&Ipv6Address(std::net::Ipv6Addr::new(
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
		BasicList,
		SubTemplateList,
		SubTemplateMultiList,
		*/
	}

	#[test]
	pub fn data_record_json_test() {
		let template = TemplateRecord {
			header : TemplateRecordHeader {
				template_id : 256,
				field_count : 4,
				scope_field_count : 0,
			},
			fields : vec![
				FieldSpecifier {
					information_element_id : 210,
					field_length : 4,
					enterprise_number : None,
				},
				FieldSpecifier {
					information_element_id : 210,
					field_length : 4,
					enterprise_number : None,
				},
			],
			scope_fields : vec![],
		};
		let record = DataRecord {
			fields : vec![
				DataValue::OctetArray(vec![0x00, 0x00, 0x00, 0x00]),
				DataValue::OctetArray(vec![0x00, 0x00, 0x00, 0x00]),
			],
		};
		let typed = TypedDataRecord {
			template : &template,
			data : &record,
		};
		assert_eq!(
			serde_json::to_string(&typed).unwrap(),
			"{\"210\":[0,0,0,0],\"210\":[0,0,0,0]}"
		);
	}
}
