#![allow(non_camel_case_types)]

#[macro_use]
extern crate nom;
#[macro_use]
extern crate log;
extern crate env_logger;

mod parser;
use parser::*;
mod structs;
use structs::*;
mod template_management;
use template_management::*;

