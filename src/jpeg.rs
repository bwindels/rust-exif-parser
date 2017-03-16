use std::iter::Iterator;
use ::cursor::Cursor;
use ::Size;
use ::error::{ParseError, ParseResult};

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

  fn try_next(&mut self) -> Result<Option<(SegmentMarker, Cursor<'a>)>, ParseError> {
    if self.next_skip != 0 {
      self.cursor = self.cursor.with_skip_or_fail(self.next_skip as usize)?;
    }

    let header_byte : u8 = self.cursor.read_num_or_fail()?;

    if header_byte != 0xFF {
      self.next_skip = 0;
      self.at_end = true;
      return Err(ParseError::InvalidJPEGSegmentHeader {header: header_byte} );
    }

    let marker = SegmentMarker::from(
      self.cursor.read_num_or_fail::<u8>()?);

    //stop reading the stream at the SOS (Start of Stream) marker,
    //because its length is not stored in the header so we can't
    //know where to jump to. The only marker after that is just EOI (End Of Image) anyway
    if marker == SegmentMarker::SOS {
      self.at_end = true;
    }
    //don't read size from markers that have no datas
    let len : u16 = if marker.has_size() {
      self.cursor.read_num_or_fail::<u16>()? - 2
    } else {
      0
    };
    println!("len: {}", len);
    let segment_cursor = self.cursor.with_max_len(len as usize);

    self.next_skip = len;

    Ok(Some((marker, segment_cursor)))
  }
}

impl<'a> Iterator for JPEGSegmentIterator<'a> {
  type Item = Result<(SegmentMarker, Cursor<'a>), ParseError>;

  fn next(&mut self) -> Option<Self::Item> {
    if self.at_end {
      None
    } else {
      match self.try_next() {
        Ok(Some(data)) => Some(Ok(data)),
        Ok(None) => None,
        Err(err) => Some(Err(err))
      }
    }
  }
}

pub fn read_image_size_from_sof<'a>(sof_cursor: &Cursor<'a>) -> ParseResult<Size> {
  let mut sof_cursor = sof_cursor.with_skip_or_fail(1)?;
  let height : u16 = sof_cursor.read_num_or_fail()?;
  let width : u16 = sof_cursor.read_num_or_fail()?;

  Ok(Size { height: height, width: width })
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
  use ::error::ParseError;

  #[test]
  fn test_empty() {
    let data : [u8; 0] = [];
    let cursor = Cursor::new(&data, Endianness::Little);
    let mut it = JPEGSegmentIterator::new(cursor);
    assert!(it.next().unwrap().err().unwrap() == ParseError::UnexpectedEOF);
  }

  #[test]
  fn test_app_segments_content() {
    let expected = expected_segments();
    let cursor = Cursor::new(JPEG_SAMPLE, Endianness::Big);
    let it = JPEGSegmentIterator::new(cursor);

    let mapped = it.map(|r| r.unwrap());
    let zipped = mapped.zip(&expected);

    for ((marker, cursor), expected) in zipped {
      assert_eq!(marker, expected.marker);
      assert_eq!(cursor.offset(), expected.offset);
      assert_eq!(cursor.len(), expected.len);
    }
  }

  #[test]
  fn test_app_segments_len() {
    let expected = expected_segments();
    let cursor = Cursor::new(JPEG_SAMPLE, Endianness::Big);
    let it = JPEGSegmentIterator::new(cursor);

    assert_eq!(
      it.take_while(Result::is_ok).count(),
      expected.len()
      );
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
