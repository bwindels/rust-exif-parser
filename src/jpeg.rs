use std::iter::Iterator;
use ::cursor::{Cursor, Endianness};

pub struct JPEGSectionIterator<'a> {
	cursor: Cursor<'a>
}

impl<'a> JPEGSectionIterator<'a> {
	pub fn new(cursor: Cursor<'a>) -> JPEGSectionIterator<'a> {
		JPEGSectionIterator {cursor: cursor}
	}
}

impl<'a> Iterator for JPEGSectionIterator<'a> {
	type Item = (u8, Cursor<'a>);

	fn next(&mut self) -> Option<Self::Item> {
		None
	}
}

#[cfg(test)]
mod tests {

	#[test]
	fn test_empty() {
		let data : [u8; 0] = [];
		let cursor = ::Cursor::new(&data, ::Endianness::Little);
		let mut it = super::JPEGSectionIterator::new(cursor);
		assert!(it.next().is_none());
	}
}