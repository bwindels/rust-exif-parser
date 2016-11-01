use std::iter::Iterator;
use ::cursor::Cursor;
use ::Size;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SegmentMarker {
    SOI,
    DHT,
    DQT,
    DRI,
    SOS,
    COM,
    EOI,
    APP(u8),
    SOF(u8),
    RST(u8),
    Other(u8)
}

impl SegmentMarker {
    fn has_size(self) -> bool {
        match self {
            SegmentMarker::RST(_) | 
            SegmentMarker::EOI | 
            SegmentMarker::SOI | 
            SegmentMarker::SOS => false,
            _ => true
        }
    }
}

impl From<u8> for SegmentMarker {

    fn from(value: u8) -> SegmentMarker {
        match value {
            0xD8 => SegmentMarker::SOI,
            0xC4 => SegmentMarker::DHT,
            0xDB => SegmentMarker::DQT,
            0xDD => SegmentMarker::DRI,
            0xDA => SegmentMarker::SOS,
            0xFE => SegmentMarker::COM,
            0xD9 => SegmentMarker::EOI,
            0xE0 ... 0xEF => SegmentMarker::APP(value - 0xE0),
            0xD0 ... 0xD7 => SegmentMarker::RST(value - 0xD0),
            0xC0 ... 0xC3 | 
            0xC5 ... 0xC7 | 
            0xC9 ... 0xCF => SegmentMarker::SOF(value - 0xC0),
            _ => SegmentMarker::Other(value)
        }
    }
}

pub struct JPEGSegmentIterator<'a> {
    cursor: Cursor<'a>,
    at_end: bool,
    next_skip: u16
}

impl<'a> JPEGSegmentIterator<'a> {
    pub fn new(cursor: Cursor<'a>) -> JPEGSegmentIterator<'a> {
        JPEGSegmentIterator {
            cursor: cursor,
            at_end: false,
            next_skip: 0
        }
    }

    fn try_next(&mut self) -> Result<Option<(SegmentMarker, Cursor<'a>)>, &'static str> {
        if self.next_skip != 0 {
            self.cursor = require!(self.cursor.skip(self.next_skip as usize), Ok(None));
        }

        let header_byte : u8 = require!(self.cursor.read_num(), Ok(None));

        if header_byte != 0xFF {
            self.next_skip = 0;
            self.at_end = true;
            return Err("Invalid JPEG segment offset");
        }

        let marker = SegmentMarker::from(
            require!(self.cursor.read_num::<u8>(), Ok(None)));

        //stop reading the stream at the SOS (Start of Stream) marker,
        //because its length is not stored in the header so we can't
        //know where to jump to. The only marker after that is just EOI (End Of Image) anyway
        if marker == SegmentMarker::SOS {
            self.at_end = true;
        }
        //don't read size from markers that have no datas
        let len : u16 = if marker.has_size() {
            require!(self.cursor.read_num::<u16>(), Ok(None)) - 2
        } else {
            0
        };

        let segment_cursor = require!(self.cursor.branch(len as usize), Ok(None));
        self.next_skip = len;

        Ok(Some((marker, segment_cursor)))
    }
}

impl<'a> Iterator for JPEGSegmentIterator<'a> {
    type Item = Result<(SegmentMarker, Cursor<'a>), &'static str>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.at_end {
            None
        } else {
            match self.try_next() {
                Ok(Some(data)) => Some(Ok(data)),
                Ok(None) => None,
                Err(msg) => Some(Err(msg))
            }
        }
    }
}

pub fn read_image_size_from_sof<'a>(sof_cursor: &Cursor<'a>) -> Option<Size> {
    let mut sof_cursor = require!(sof_cursor.skip(1), None);
    let height : u16 = require!(sof_cursor.read_num(), None);
    let width : u16 = require!(sof_cursor.read_num(), None);

    Some(Size { height: height, width: width })
}

#[cfg(test)]
mod tests {
    use ::cursor::{
        Cursor,
        Endianness
    };
    use super::{
        JPEGSegmentIterator,
        SegmentMarker,
        read_image_size_from_sof
    };
    use ::test_fixtures::{
        JPEG_SAMPLE,
        expected_segments
    };

    #[test]
    fn test_empty() {
        let data : [u8; 0] = [];
        let cursor = Cursor::new(&data, Endianness::Little);
        let mut it = JPEGSegmentIterator::new(cursor);
        assert!(it.next().is_none());
    }

    #[test]
    fn test_app_segments_content() {
        let expected = expected_segments();
        let cursor = Cursor::new(JPEG_SAMPLE, Endianness::Big);
        let it = JPEGSegmentIterator::new(cursor);

        assert_eq!(
            it.take_while(Result::is_ok).count(),
            expected.len()
        );

        let cursor = Cursor::new(JPEG_SAMPLE, Endianness::Big);
        let it = JPEGSegmentIterator::new(cursor);

        let mapped = it
            .map(|r| r.unwrap())
            .map(|(marker, cursor)| (marker, cursor.len()));
        
        let zipped = mapped.zip(&expected);

        for (given, expected) in zipped {
            assert_eq!(given.0, expected.0);
            assert_eq!(given.1, expected.1);
        }
    }

    #[test]
    fn test_size() {
        let cursor = Cursor::new(JPEG_SAMPLE, Endianness::Big);
        let it = JPEGSegmentIterator::new(cursor);
        let mut sof0_cursor = it
            .map(|res| res.unwrap())
            .find(|&(marker, _)| marker == SegmentMarker::SOF(0))
            .map(|(_, cursor)| cursor).unwrap();
        let size = read_image_size_from_sof(&mut sof0_cursor).unwrap();
        assert_eq!(size.width, 2);
        assert_eq!(size.height, 1);
    }
}
