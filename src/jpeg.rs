use std::iter::Iterator;
use ::cursor::Cursor;

pub struct JPEGSectionIterator<'a> {
    cursor: Cursor<'a>,
    had_error: bool
}

impl<'a> JPEGSectionIterator<'a> {
    pub fn new(cursor: Cursor<'a>) -> JPEGSectionIterator<'a> {
        JPEGSectionIterator {cursor: cursor, had_error: false}
    }
}

impl<'a> Iterator for JPEGSectionIterator<'a> {
    type Item = Result<(u8, Cursor<'a>), &'static str>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.had_error {
            return None;
        }

        let header_byte : u8 = self.cursor.read_num();

        if header_byte != 0xFF {
            self.had_error = true;
            return Some(Err("Invalid JPEG section offset"));
        }
        
        let marker_type : u8 = self.cursor.read_num();
        let section_has_data = (marker_type >= 0xD0 && marker_type <= 0xD9) || 
                               marker_type == 0xDA;
        
        let len : u16 = if section_has_data {
            self.cursor.read_num() - 2
        } else {
            0
        };
        let section_cursor = self.cursor.branch(len);
        
        section_cursor.map(|c| Some((marker_type, c)))
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