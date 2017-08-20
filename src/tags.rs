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
use ::tag::{
  RawExifTag,
  ExifVariant
};

pub struct SectionOffsetIterator {
  ifd0_offset: Option<u32>,
  ifd1_offset: Option<u32>,
  gps_offset: Option<u32>,
  sub_ifd_offset: Option<u32>,
  interop_offset: Option<u32>
}

impl SectionOffsetIterator {

  pub fn new(ifd0_offset: u32) -> SectionOffsetIterator {
    SectionOffsetIterator {
      ifd0_offset: Some(ifd0_offset),
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
    if let Some(offset) = self.ifd0_offset {
      self.ifd0_offset = None;
      return Some((offset, Section::IFD0));
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
  current_section: Option<(SectionIterator<'a>, (u32, Section))>,
  tiff_marker: Cursor<'a>
}

impl<'a> ExifTagIterator<'a> {

  pub fn new(tiff_marker: Cursor<'a>, ifd0_offset: u32) -> ExifTagIterator<'a> {
    ExifTagIterator {
      section_offsets: SectionOffsetIterator::new(ifd0_offset),
      current_section: None,
      tiff_marker: tiff_marker
    }
  }

  fn open_section(&self, offset: u32) -> ParseResult<SectionIterator<'a>> {
    let cursor = self.tiff_marker.with_skip_or_fail(offset as usize)?;
    read_section(cursor, self.tiff_marker)
  }

}

/** get the offset from a tag meant to contain the offset to another IFD */
fn offset_from_tag<'a>(tag: &RawExifTag<'a>) -> Option<u32> {
  match tag.value {
    ExifVariant::UInt(ref components) => components.iter().nth(0),
    _ => None
  }
}

/** The offset to other IFDs are (apart from IFD1) in tags themselves.
    Here we update the offset iterator with offsets it finds in tags as we go */
fn update_offset_iter<'a>(offset_iter: &mut SectionOffsetIterator,
                          section_id: Section, tag: &RawExifTag<'a>) {

  match (section_id, tag.tag_type) {
    (Section::IFD0, 0x8825) =>
      if let Some(offset) = offset_from_tag(&tag) {
        offset_iter.set_gps_offset(offset)
      },
    (Section::IFD0, 0x8769) =>
      if let Some(offset) = offset_from_tag(&tag) {
        offset_iter.set_sub_ifd_offset(offset)
      },
    (Section::SubIFD, 0xA005) =>
      if let Some(offset) = offset_from_tag(&tag) {
        offset_iter.set_interop_offset(offset)
      },
    _ => ()
  }
}

/** The IFD1 offset does not come in a tag,
    but as an offset right after the IFD0 section. */
fn update_offset_iter_with_idf1<'a>(
  offset_iter: &mut SectionOffsetIterator,
  ifd0_offset: usize,
  ifd0_size: usize,
  tiff_cursor: Cursor<'a>
) {

  //offset where to read the offset of IFD1 off,
  //just behind IFD0
  let ifd1_offset_offset = ifd0_offset + ifd0_size;

  if let Some(mut ifd1_offset_cursor) = tiff_cursor.with_skip(ifd1_offset_offset) {
    if let Some(ifd1_offset) = ifd1_offset_cursor.read_num::<u32>() {
      if ifd1_offset != 0 {
        offset_iter.set_ifd1_offset(ifd1_offset);
      }
    }
  }
}

impl<'a> Iterator for ExifTagIterator<'a> {

  type Item = ParseResult<(RawExifTag<'a>, Section)>;

  fn next(&mut self) -> Option<Self::Item> {
    loop {
      //if currently going through a section ...
      if let Some((ref mut section_it, (offset, id))) = self.current_section {
        //if this section still has tags in it ...
        if let Some(tag) = section_it.next() {
          //see if the tag contains an offset to another IFD
          if let Ok(ref t) = tag {
            update_offset_iter(&mut self.section_offsets, id, t);
          }
          //include the section enum value in the result,
          //because tag numbers are only unique inside a section
          let tag_with_section_id = tag.map(|t| (t, id) );

          return Some(tag_with_section_id);
        }
        //handle IFD1 offset just after IFD0 section
        else if id == Section::IFD0 {
          update_offset_iter_with_idf1(
            &mut self.section_offsets,
            offset as usize,
            section_it.byte_size(),
            self.tiff_marker);
        }
      }
      //if we got here (either current section came to end,
      //or we haven't entered first section yet),
      //we try to go into a new section
      match self.section_offsets.next() {
        None => return None,
        Some((offset, id)) => {
          let section = self.open_section(offset);
          match section {
            Ok(section_it) =>
              self.current_section = Some( (section_it, (offset, id)) ),
            Err(e) =>
              return Some(Err(e))
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
  let ifd0_offset : u32 = tiff_marker.clone()
    .with_skip_or_fail(2 + 2)?  //skip tiff header and tiff data
    .read_num_or_fail()?;
  Ok(ExifTagIterator::new(tiff_marker, ifd0_offset))
}


fn read_exif_header<'a>(mut app1_cursor: Cursor<'a>) -> ParseResult<Cursor<'a>> {
  let header = app1_cursor.read_bytes_or_fail(6)?;

  if header != b"Exif\0\0" {
    let h = header;
    let header_array = [h[0], h[1], h[2], h[3], h[4], h[5]];
    return Err(ParseError::InvalidExifHeader{ header: header_array });
  }

  let tiff_cursor = app1_cursor.clone();
  let tiff_header : u16 = app1_cursor.read_num_or_fail()?;

  let app1_cursor = match tiff_header {
    0x4949 => Ok(app1_cursor.with_endianness(Endianness::Little)),
    0x4D4D => Ok(app1_cursor.with_endianness(Endianness::Big)),
    _ => Err(ParseError::InvalidTiffHeader{ header: tiff_header })
  };
  let mut app1_cursor = app1_cursor?;

  //this is a marker in the data to check
  //the endianness has been properly detected
  //if not you'd read 0x2A00
  let tiff_data_marker : u16 = app1_cursor.read_num_or_fail()?;
  if tiff_data_marker != 0x002A {
    return Err(ParseError::InvalidTiffData{ data: tiff_data_marker });
  }

  let tiff_cursor = tiff_cursor.with_endianness(app1_cursor.endianness());

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
    let tiff_cursor = read_exif_header(cursor).unwrap();
    let tiff_data = tiff_cursor
      .with_skip_or_fail(2).unwrap()  //skip tiff header
      .read_num::<u16>();
    assert_eq!(tiff_data, Some(0x002A) );
  }

}
