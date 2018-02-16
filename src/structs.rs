pub struct Message {
	pub header : Message_Header,
	pub sets : Vec<Set>,
}

pub struct Message_Header {
	pub version_number : u16,
	pub length : u16,
	pub export_time : u32,
	pub sequence_number : u32,
	pub observation_domain_id : u32,
}

pub struct Field_Specifier {
	pub information_element_id : u16,
	pub field_length : u16,
	pub enterprise_number : Option<u32>,
}

pub struct Set {
	pub header : Set_Header,
	pub records : Records,
}

pub struct Set_Header {
	pub set_id : u16,
	pub length : u16,
}

pub enum Records {
	Template_Records(Vec<Template_Record>),
	Options_Template_Records(Vec<Options_Template_Record>),
	Data_Records(Vec<Data_Record>),
}

pub struct Template_Record {
	pub header : Template_Record_Header,
	pub fields : Vec<Field_Specifier>,
}

pub struct Template_Record_Header {
	pub template_id : u16,
	pub field_count : u16,
}

pub struct Options_Template_Record {
	pub header : Options_Template_Record_Header,
	pub fields : Vec<Field_Specifier>,
}

pub struct Options_Template_Record_Header {
	pub template_id : u16,
	pub field_count : u16,
	pub scope_field_count : u16,
}

pub struct Data_Record {
	pub fields : Vec<u8>,
}
