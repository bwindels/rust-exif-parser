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

    fn foo(&mut self) -> Result<(u8, Cursor<'a>), &'static str> {
        let header_byte : u8 = try!(self.cursor.read_num().ok_or("Could not read header"));

        if header_byte != 0xFF {
            self.had_error = true;
            return Err("Invalid JPEG section offset");
        }

        let marker_type : u8 = try!(self.cursor.read_num().ok_or("Could not read header type"));
        let section_has_data = (marker_type >= 0xD0 && marker_type <= 0xD9) ||
            marker_type == 0xDA;

        let len : u16 = if section_has_data {
            try!(self.cursor.read_num::<u16>().ok_or("Could not read section data length")) - 2
        } else {
            0
        };
        let section_cursor = try!(self.cursor.branch(len as usize).ok_or("Fooo"));

        Ok((marker_type, section_cursor))
    }
}

impl<'a> Iterator for JPEGSectionIterator<'a> {
    type Item = Result<(u8, Cursor<'a>), &'static str>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.had_error {
            None
        } else {
            Some(self.foo())
        }
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