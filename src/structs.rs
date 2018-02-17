pub const MESSAGE_HEADER_LENGTH : u16 = 16;
pub const SET_HEADER_LENGTH : u16 = 4;

#[derive(Debug, PartialEq)]
pub struct Message<'a> {
	pub header : Message_Header,
	pub sets : Vec<(Set_Header, &'a[u8])>,
}

#[derive(Debug, PartialEq)]
pub struct Message_Header {
	pub version_number : u16,
	pub length : u16,
	pub export_time : u32,
	pub sequence_number : u32,
	pub observation_domain_id : u32,
}

#[derive(Debug, PartialEq)]
pub struct Field_Specifier {
	pub information_element_id : u16,
	pub field_length : u16,
	pub enterprise_number : Option<u32>,
}

#[derive(Debug, PartialEq)]
pub struct Set_Header {
	pub set_id : u16,
	pub length : u16,
}

#[derive(Debug, PartialEq)]
pub struct Template_Record {
	pub header : Template_Record_Header,
	pub fields : Vec<Field_Specifier>,
}

#[derive(Debug, PartialEq)]
pub struct Template_Record_Header {
	pub template_id : u16,
	pub field_count : u16,
}

/* TODO
pub struct Options_Template_Record {
	pub header : Options_Template_Record_Header,
	pub fields : Vec<Field_Specifier>,
	pub scope_fields : Vec<Field_Specifier>
}
*/

/* TODO
pub struct Options_Template_Record_Header {
	pub template_id : u16,
	pub field_count : u16,
	pub scope_field_count : u16,
}
*/

#[derive(Debug, PartialEq)]
pub struct Data_Record {
	pub fields : Vec<u8>,
}
