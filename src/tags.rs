use ::section::{
  read_section,
  SectionIterator
};
use ::error::{
  ParseResult,
  ParseError
};
use ::cursor::{
  Cursor,
  Endianness
};
use ::tag::RawExifTag;

pub struct SectionOffsetIterator {
  has_consumed_ifd0: bool,
  ifd1_offset: Option<u32>,
  gps_offset: Option<u32>,
  sub_ifd_offset: Option<u32>,
  interop_offset: Option<u32>
}

impl SectionOffsetIterator {

  pub fn new() -> SectionOffsetIterator {
    SectionOffsetIterator {
      has_consumed_ifd0: false,
      //TODO: cleanup the 4 ... //Some(ifd0_length + 4u32)
      ifd1_offset: None,
      gps_offset: None,
      sub_ifd_offset: None,
      interop_offset: None
    }
  }

  pub fn set_ifd1_offset(&mut self, offset: u32) {
    self.ifd1_offset = Some(offset);
  }

  pub fn set_gps_offset(&mut self, offset: u32) {
    self.gps_offset = Some(offset);
  }

  pub fn set_sub_ifd_offset(&mut self, offset: u32) {
    self.sub_ifd_offset = Some(offset);
  }

  pub fn set_interop_offset(&mut self, offset: u32) {
    self.interop_offset = Some(offset);
  }
}

impl Iterator for SectionOffsetIterator {

  type Item = (u32, Section);

  fn next(&mut self) -> Option<Self::Item> {
    if !self.has_consumed_ifd0 {
      self.has_consumed_ifd0 = true;
      return Some((0, Section::IFD0));
    }
    if let Some(offset) = self.ifd1_offset {
      self.ifd1_offset = None;
      return Some((offset, Section::IFD1));
    }
    if let Some(offset) = self.gps_offset {
      self.gps_offset = None;
      return Some((offset, Section::GPS));
    }
    if let Some(offset) = self.sub_ifd_offset {
      self.sub_ifd_offset = None;
      return Some((offset, Section::SubIFD));
    }
    if let Some(offset) = self.interop_offset {
      self.interop_offset = None;
      return Some((offset, Section::InteropIFD));
    }
    else {
      return None;
    }
  }
}



pub struct ExifTagIterator<'a> {
  section_offsets: SectionOffsetIterator,
  current_section: Option<(SectionIterator<'a>, Section)>,
  tiff_marker: Cursor<'a>
}

impl<'a> ExifTagIterator<'a> {

  pub fn new(tiff_marker: Cursor<'a>) -> ExifTagIterator<'a> {
    ExifTagIterator {
      section_offsets: SectionOffsetIterator::new(),
      current_section: None,
      tiff_marker: tiff_marker
    }
  }

  fn open_section(&self, offset: u32) -> ParseResult<SectionIterator<'a>> {
    let cursor = self.tiff_marker.with_skip_or_fail(offset as usize)?;
    read_section(cursor, self.tiff_marker)
  }

}


fn update_offset_iter<'a>(offset_iter: &mut SectionOffsetIterator, tag: &RawExifTag<'a>) {
  match tag.tag_type {
    0x1234 => offset_iter.set_ifd1_offset(5u32),
    _ => ()
  }
}

impl<'a> Iterator for ExifTagIterator<'a> {

  type Item = ParseResult<(RawExifTag<'a>, Section)>;

  fn next(&mut self) -> Option<Self::Item> {
    loop {
      if let Some((ref mut section_it, ref id)) = self.current_section {
        if let Some(tag) = section_it.next() {

          if let Ok(ref t) = tag {
            update_offset_iter(&mut self.section_offsets, t);
          }

          let tag_with_section_id = tag.map(|t| (t, *id) );

          return Some(tag_with_section_id);
        }
      }

      match self.section_offsets.next() {
        None => return None,
        Some((offset, id)) => {
          let section = self.open_section(offset);
          match section {
            Ok(section_it) => self.current_section = Some( (section_it, id) ),
            Err(e) => return Some(Err(e))
          };
        }
      }
    }
  }

}

#[derive(PartialEq, Clone, Copy)]
pub enum Section {
  IFD0,
  IFD1,
  GPS,
  SubIFD,
  InteropIFD
}

pub fn read_tags<'a>(app1_cursor: Cursor<'a>) -> ParseResult<ExifTagIterator<'a>> {
  let tiff_marker = read_exif_header(app1_cursor)?;
  Ok(ExifTagIterator::new(tiff_marker))
}


fn read_exif_header<'a>(mut app1_cursor: Cursor<'a>) -> ParseResult<Cursor<'a>> {
  let header = app1_cursor.read_bytes_or_fail(6)?;

  if header != b"Exif\0\0" {
    let h = header;
    let header_array = [h[0], h[1], h[2], h[3], h[4], h[5]];
    return Err(ParseError::InvalidExifHeader{ header: header_array });
  }

  let tiff_header : u16 = app1_cursor.read_num_or_fail()?;

  let tiff_cursor = match tiff_header {
    0x4949 => app1_cursor.with_endianness(Endianness::Little),
    0x4D4D => app1_cursor.with_endianness(Endianness::Big),
    _ => return Err(ParseError::InvalidTiffHeader{ header: tiff_header })
  };

  //this is a marker in the data to check
  //the endianess has been properly detected
  //if not you'd read 0x2A00
  let tiff_data_marker : u16 = tiff_cursor.clone().read_num_or_fail()?;
  if tiff_data_marker != 0x002A {
    return Err(ParseError::InvalidTiffData{ data: tiff_data_marker });
  }

  return Ok(tiff_cursor);
}

#[cfg(test)]
mod tests {

  use ::cursor::{Cursor, Endianness};
  use ::test_fixtures::{JPEG_SAMPLE, JPEG_SAMPLE_EXIF_OFFSET};
  use super::{
    read_exif_header
  };

  #[test]
  fn test_read_exif_header() {
    let cursor = Cursor::new(JPEG_SAMPLE, Endianness::Little);
    assert!(read_exif_header(cursor).is_err());

    let cursor = Cursor::new(JPEG_SAMPLE, Endianness::Little);
    let cursor = cursor.with_skip_or_fail(JPEG_SAMPLE_EXIF_OFFSET).expect("EOF");
    let mut tiff_cursor = read_exif_header(cursor).unwrap();
    assert_eq!(tiff_cursor.read_num::<u16>(), Some(0x002A) );
  }

}
