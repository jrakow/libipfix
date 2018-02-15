pub struct Message {
	header : Message_Header,
	records : Vec<Set>,
}

pub struct Message_Header {
	version_number : u16,
	length : u16,
	export_time : u32,
	sequence_number : u32,
	observation_domain_id : u32,
}

pub struct Field_Specifier {
	information_element_id : u16,
	field_length : u16,
	enterprise_number : Option<u32>,
}

pub struct Set {
	header : Set_Header,
	records : Vec<Record>,
}

pub struct Set_Header {
	set_id : u16,
	length : u16,
}

enum Record {
	Template_Record,
	Options_Template_Record,
	Data_Record,
}

pub struct Template_Record {
	header : Template_Record_Header,
	fields : Vec<Field_Specifier>,
}

pub struct Template_Record_Header {
	template_id : u16,
	field_count : u16,
}

pub struct Options_Template_Record {
	header : Options_Template_Record_Header,
	fields : Vec<Field_Specifier>,
}

pub struct Options_Template_Record_Header {
	template_id : u16,
	field_count : u16,
	scope_field_count : u16,
}

pub struct Data_Record {
	fields : Vec<u8>,
}
