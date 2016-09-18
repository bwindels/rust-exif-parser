use std::iter::Iterator;
use ::cursor::Cursor;

pub struct JPEGSectionIterator<'a> {
    cursor: Cursor<'a>,
    had_error: bool
}

macro_rules! require {
    ($opt:expr, $default:expr) => {
        match $opt {
            Some(val) => val,
            None => return $default
        }
    }
}

impl<'a> JPEGSectionIterator<'a> {
    pub fn new(cursor: Cursor<'a>) -> JPEGSectionIterator<'a> {
        JPEGSectionIterator {cursor: cursor, had_error: false}
    }

    fn advance(&mut self) -> Result<Option<(u8, Cursor<'a>)>, &'static str> {
        let header_byte : u8 = require!(self.cursor.read_num(), Ok(None));

        if header_byte != 0xFF {
            self.had_error = true;
            return Err("Invalid JPEG section offset");
        }

        let marker_type : u8 = require!(self.cursor.read_num(), Ok(None));
        let section_has_data = (marker_type >= 0xD0 && marker_type <= 0xD9) ||
            marker_type == 0xDA;

        let len : u16 = if section_has_data {
            require!(self.cursor.read_num::<u16>(), Ok(None)) - 2
        } else {
            0
        };
        let section_cursor = require!(self.cursor.branch(len as usize), Ok(None));

        Ok(Some((marker_type, section_cursor)))
    }
}

impl<'a> Iterator for JPEGSectionIterator<'a> {
    type Item = Result<(u8, Cursor<'a>), &'static str>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.had_error {
            None
        } else {
            match self.advance() {
                Ok(Some(data)) => Some(Ok(data)),
                Ok(None) => None,
                Err(msg) => Some(Err(msg))
            }
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