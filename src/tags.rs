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

pub struct ExifTagIterator<'a> {
  gps_offset: Option<u32>,
  sub_ifd_offset: Option<u32>,
  interop_offset: Option<u32>,
  current_section: SectionIterator<'a>,
  current_ifd_marker: Section,
  tiff_marker: Cursor<'a>
}

impl<'a> ExifTagIterator<'a> {

  fn find_next_section(&mut self) -> Option<ParseResult<(SectionIterator<'a>, Section)>> {
    if self.current_ifd_marker == Section::IFD0 {
      //TODO: cleanup the 4 ...
      let ifd1_offset = 4usize + self.current_section.byte_size();
      let section = self.open_section(ifd1_offset);
      return Some(section.map(|s| (s, Section::IFD1)));
    }
    else if self.gps_offset.is_some() {
      let section = self.open_section(self.gps_offset.unwrap() as usize);
      self.gps_offset = None;
      return Some(section.map(|s| (s, Section::GPS)));
    }
    else if self.sub_ifd_offset.is_some() {
      let section = self.open_section(self.sub_ifd_offset.unwrap() as usize);
      self.sub_ifd_offset = None;
      return Some(section.map(|s| (s, Section::SubIFD)));
    }
    else if self.interop_offset.is_some() {
      let section = self.open_section(self.interop_offset.unwrap() as usize);
      self.interop_offset = None;
      return Some(section.map(|s| (s, Section::InteropIFD)));
    }
    return None;
  }

  fn open_section(&self, offset: usize) -> ParseResult<SectionIterator<'a>> {
    let cursor = self.tiff_marker.with_skip_or_fail(offset)?;
    read_section(cursor, &self.tiff_marker)
  }

}

impl<'a> Iterator for ExifTagIterator<'a> {
  type Item = ParseResult<(RawExifTag<'a>, Section)>;

  fn next(&mut self) -> Option<Self::Item> {
    let mut tag = self.current_section.next();

    while tag.is_none() {
      let next_section = self.find_next_section();
      match next_section {
        Some(result) => {
          self.current_section = it;
          self.current_ifd_marker = s;
        },
        //no more sections, end iterator
        None => return None
      };
      tag = self.current_section.next();
    }

    let tag_and_ifd = tag.map(|tag| (tag, self.current_ifd_marker));

    Some(tag_and_ifd)
  }

  fn size_hint(&self) -> (usize, Option<usize>) {
    let min_len = self.current_section.size_hint().0;
    (min_len, None)
  }

}

#[derive(PartialEq)]
pub enum Section {
  IFD0,
  IFD1,
  GPS,
  SubIFD,
  InteropIFD
}

pub fn read_tags<'a>(mut app1_cursor: Cursor<'a>) -> ParseResult<ExifTagIterator<'a>> {
  let tiff_marker = read_exif_header(&mut app1_cursor)?;
  let mut ifd0_cursor = app1_cursor;
  let ifd0_section = read_section(ifd0_cursor.clone(), &tiff_marker)?;
  
  Ok(ExifTagIterator {
    gps_offset: None,
    sub_ifd_offset: None,
    interop_offset: None,
    current_section: ifd0_section,
    current_ifd_marker: Section::IFD0,
    tiff_marker: tiff_marker
  })
}


fn read_exif_header<'a>(app1_cursor: &mut Cursor<'a>) -> ParseResult<Cursor<'a>> {
  let header = app1_cursor.read_bytes_or_fail(6)?;

  if header != b"Exif\0\0" {
    let h = header;
    let header_array = [h[0], h[1], h[2], h[3], h[4], h[5]];
    return Err(ParseError::InvalidExifHeader{ header: header_array });
  }

  let mut tiff_marker = app1_cursor.clone();
  let tiff_header : u16 = app1_cursor.read_num_or_fail()?;

  if tiff_header == 0x4949 {
    app1_cursor.set_endianness(Endianness::Little);
  }
  else if tiff_header == 0x4D4D {
    app1_cursor.set_endianness(Endianness::Big);
  }
  else {
    return Err(ParseError::InvalidTiffHeader{ header: tiff_header });
  }

  let tiff_data_marker : u16 = app1_cursor.read_num_or_fail()?;

  if tiff_data_marker != 0x002A {
    return Err(ParseError::InvalidTiffData{ data: tiff_data_marker });
  }

  tiff_marker.set_endianness(app1_cursor.endianness());

  return Ok(tiff_marker);
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
    let mut cursor = Cursor::new(JPEG_SAMPLE, Endianness::Little);
    assert!(read_exif_header(&mut cursor).is_err());
    let mut cursor = Cursor::new(JPEG_SAMPLE, Endianness::Little);
    cursor = cursor.with_skip_or_fail(JPEG_SAMPLE_EXIF_OFFSET).expect("EOF");
    assert!(read_exif_header(&mut cursor).is_ok());
  }

}