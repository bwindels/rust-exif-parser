use std::iter::Iterator;
use ::cursor::Cursor;
use ::error::ParseResult;
use ::tag::{
  read_exif_tag,
  RawExifTag,
  EXIF_TAG_SIZE
};

pub struct SectionIterator<'a> {
	cursor: Cursor<'a>,
  tiff_marker: Cursor<'a>,
  len: u32,
  i: u32
}

impl<'a> SectionIterator<'a> {

  pub fn len(&self) -> usize {
    self.len as usize
  }

} 

impl<'a> Iterator for SectionIterator<'a> {
  type Item = ParseResult<RawExifTag<'a>>;

  fn next(&mut self) -> Option<Self::Item> {
    if self.i >= self.len {
      return None;
    }
    //skip only when we really need to so we fail as late as possible
    if self.i != 0 {
      let skipped_cursor = self.cursor.with_skip_or_fail(EXIF_TAG_SIZE);
      self.cursor = match skipped_cursor {
        Ok(c) => c,
        Err(err) => return Some(Err(err))
      };
    }

    let tag = read_exif_tag(self.cursor, self.tiff_marker);
    self.i += 1;

    return Some(tag);
  }

  fn size_hint(&self) -> (usize, Option<usize>) {
    let size = (self.len - self.i) as usize;
    (size, Some(size))
  }
}

pub fn read_section<'a>(mut cursor: Cursor<'a>, tiff_marker: Cursor<'a>) -> ParseResult<SectionIterator<'a>> {
	let len : u32 = cursor.read_num_or_fail()?;
  Ok(SectionIterator {
    cursor: cursor,
    tiff_marker: tiff_marker,
    len: len,
    i: 0
  })
}

#[cfg(test)]
mod tests {
  use super::read_section;
  use ::cursor::{Cursor, Endianness};

  #[test]
  fn read_simple_section() {
    const EXIF_SECTION : &'static [u8] = &[
      0u8, 0u8, 0u8, 2u8, //2 tags in this section
      //first tag
      0u8, 209u8,//tag
      0u8, 4u8, //uint
      0u8, 0u8, 0u8, 1u8, //1 values
      0u8, 0u8, 0u8, 130u8, //ABCD
      //second tag
      0u8, 210u8,//tag
      0u8, 4u8, //uint
      0u8, 0u8, 0u8, 1u8, //1 values
      0u8, 0u8, 0u8, 120u8, //120
      ]; //offset to data area;
    const EXIF_POINTER_AREA : &'static [u8] = &[];
    let cursor = Cursor::new(EXIF_SECTION, Endianness::Big);
    let data_cursor = Cursor::new(EXIF_POINTER_AREA, Endianness::Big);
    let mut section = read_section(cursor, data_cursor)
      .expect("read should not fail");
    let first_tag = section.next()
      .expect("first value should be some")
      .expect("first tag should be ok");
    assert_eq!(first_tag.tag_type, 209);
    let second_tag = section.next()
      .expect("second value should be some")
      .expect("second tag should be ok");
    assert_eq!(second_tag.tag_type, 210);
    assert!(section.next().is_none(), "section only contains 2 tags");
  }
}
