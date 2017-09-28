#![allow(dead_code)]

mod cursor;
mod error;
mod jpeg;
mod component;
mod tag;
mod section; 
mod tags;
mod transform;
#[cfg(test)]
mod test_fixtures;

pub struct Size {
	pub height: u16,
	pub width: u16
}

pub use cursor::{Cursor, Endianness};
pub use error::{ParseError, ParseResult};
