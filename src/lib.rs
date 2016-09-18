mod cursor;
mod jpeg;

pub struct Size {
	pub height: u16,
	pub width: u16
}

pub use cursor::{Cursor, Endianness};