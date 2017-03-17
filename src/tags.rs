

pub struct ExifIterator<'a> {
  gps_offset: Option<u32>,
  sub_ifd_offset: Option<u32>,
  interop_offset: Option<u32>,
  current_section: SectionIterator<'a>,
  current_ifd_marker: IFD,
  tiff_marker: Cursor<'a>
}

impl<'a> Iterator for ExifIterator<'a> {
  type Item = ParseResult<(RawExifTag<'a>, IFD)>;

  fn next(&mut self) -> Option<Self::Item> {
    let tag = self.current_section.next()
    let tag = if tag.is_none() {
      let next_section = self.find_next_section();
      self.current_section = match next_section {
        Some(s) => s,
        None => return None;
      };
      self.current_section.next()
    } else {
      tag
    };
    let tag_and_ifd = 
      tag.map(|tag| (tag, self.current_ifd_marker));

    Some(tag_and_ifd)
  }

  fn find_next_section(&mut self) -> (IFD, SectionIterator<'a>) {
    if self.gps_offset.is_some() {
      let section = (GPS, open_section(self.gps_offset.unwrap()));
      self.gps_offset 
    }
  }

  fn open_section(section_offset: &mut Option<u32>, &Cursor<'a> tiff_marker) -> ParseResult<SectionIterator<'a>> {
    
    let cursor = self.tiff_marker.with_skip_or_fail(offset)?;
    read_ifd_section(cursor, &self.tiff_marker)
  }

  fn size_hint(&self) -> (usize, Option<usize>) {
    let min_len = self.current_section.size_hint().0;
    (min_len, None)
  }

}

pub enum IFD {
  IFD0,
  IFD1,
  GPS,
  SubIFD,
  InteropIFD
}

pub fn read_tags(cursor: &mut Cursor<'a>) -> ExifIterator<'a> {
  //read header, read ifd0
}


pub fn read_exif_header<'a>(app1_cursor: &mut Cursor<'a>) -> ParseResult<Cursor<'a>> {
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