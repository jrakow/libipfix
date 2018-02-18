#![allow(non_camel_case_types)]

extern crate env_logger;
#[macro_use]
extern crate log;
#[macro_use]
extern crate nom;

pub mod parser;
pub use parser::*;
pub mod structs;
pub use structs::*;
pub mod template_management;
pub use template_management::*;
