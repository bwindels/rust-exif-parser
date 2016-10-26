#[macro_use]
mod require;
mod cursor;
mod jpeg;
mod exif;
mod error;
#[cfg(test)]
mod test_fixtures;

pub struct Size {
	pub height: u16,
	pub width: u16
}

pub use cursor::{Cursor, Endianness};
pub use error::ParseError;
