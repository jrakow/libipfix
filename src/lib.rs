#![allow(non_camel_case_types)]

#[macro_use]
extern crate nom;
#[macro_use]
extern crate log;
extern crate env_logger;

pub mod parser;
pub use parser::*;
pub mod structs;
pub use structs::*;
pub mod template_management;
pub use template_management::*;
