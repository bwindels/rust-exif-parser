use std::iter::Iterator;
use ::cursor::Cursor;

pub struct JPEGSectionIterator<'a> {
	cursor: Cursor<'a>
}

impl<'a> JPEGSectionIterator<'a> {
	pub fn new(cursor: Cursor<'a>) -> JPEGSectionIterator<'a> {
		JPEGSectionIterator {cursor: cursor}
	}
}

impl<'a> Iterator for JPEGSectionIterator<'a> {
	type Item = Result<(u8, Cursor<'a>), &'static str>;

	fn next(&mut self) -> Option<Self::Item> {
		None
	}
}

#[cfg(test)]
mod tests {
	use ::cursor::{Cursor, Endianness};

	#[test]
	fn test_empty() {
		let data : [u8; 0] = [];
		let cursor = Cursor::new(&data, Endianness::Little);
		let mut it = super::JPEGSectionIterator::new(cursor);
		assert!(it.next().is_none());
	}
}