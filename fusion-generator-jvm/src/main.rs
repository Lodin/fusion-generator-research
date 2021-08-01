use std::convert::TryFrom;
use std::error::Error;
use std::fs::File;
use std::io::Read;
use std::path::Path;

use coffer::member::MethodAttribute;
use coffer::ty::Type;
use coffer::{read_from, Class, ReadWrite};

mod cli;
mod constant;
mod error;
mod resolver;
mod translator;
mod utils;

fn main() -> Result<(), Box<dyn Error>> {
  Ok(())
}
