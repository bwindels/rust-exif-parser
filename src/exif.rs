use std::iter::Iterator;
use ::cursor::{Cursor, Endianness, ByteSwappable};
use ::error::{ParseError, ParseResult};
use std::marker::{Sized, PhantomData};

trait ExifValueReader {
  fn read_exif_value(cursor: &mut Cursor) -> ParseResult<Self> where Self: Sized + Copy;
}

impl ExifValueReader for (u32, u32) {
  fn read_exif_value(cursor: &mut Cursor) -> ParseResult<Self> {
    let a : u32 = try!(cursor.read_num_or_fail());
    let b : u32 = try!(cursor.read_num_or_fail());
    Ok((a, b))
  }
}

impl ExifValueReader for (i32, i32) {
  fn read_exif_value(cursor: &mut Cursor) -> ParseResult<Self> {
    let a : i32 = try!(cursor.read_num_or_fail());
    let b : i32 = try!(cursor.read_num_or_fail());
    Ok((a, b))
  }
}

impl<T> ExifValueReader for T where T: ByteSwappable {
  fn read_exif_value(cursor: &mut Cursor) -> ParseResult<Self> {
    cursor.read_num_or_fail()
  }
}

pub struct RawExifTag<'a> {
  pub tag_type: u16,
  pub format: u16,
  pub value: ExifVariant<'a>
}

pub struct ValueIterator<'a, T> {
  value_cursor: Cursor<'a>,
  len: u32,
  i: u32,
  phantom_data: PhantomData<T>
}

impl<'a, T: ExifValueReader + Copy + Sized> ValueIterator<'a, T> {
  fn new(cursor: Cursor<'a>, len: u32) -> ValueIterator<'a, T> {
    ValueIterator {
      value_cursor: cursor,
      len: len,
      i: 0,
      phantom_data: PhantomData
    }
  }
}

impl<'a, T: ExifValueReader + Copy + Sized> Iterator for ValueIterator<'a, T> {

  type Item = T;

  fn next(&mut self) -> Option<Self::Item> {
    let remaining = self.len - self.i;

    if remaining == 0 {
      return None;
    }

    self.i = remaining;

    let value = T::read_exif_value(&mut self.value_cursor);
    return Some(value.unwrap());
  }

  fn size_hint(&self) -> (usize, Option<usize>) {
    let remaining = (self.len - self.i) as usize;
    (remaining, Some(remaining))
  }
}

fn format_bytes_per_component(format: u16) -> ParseResult<usize> {
  match format {
    1 | 2 | 6 | 7 => Ok(1),
    3 | 8 => Ok(2),
    4 | 9 | 11 => Ok(4),
    5 | 10 | 12 => Ok(8),
    _ => Err(ParseError::InvalidValueFormat{ format: format })
  }
}

pub fn read_exif_tag<'a>(cursor: &mut Cursor<'a>, tiff_cursor: &Cursor<'a>) -> ParseResult<RawExifTag<'a>> {
  let tag_type : u16 = try!(cursor.read_num_or_fail());
  let format : u16 = try!(cursor.read_num_or_fail());
  let components : u32 = try!(cursor.read_num_or_fail());
  let bytes_per_component = try!(format_bytes_per_component(format));
  let total_values_bytes = bytes_per_component * components as usize;

  let mut value_cursor = if total_values_bytes > 4 {
    let tiff_offset : u32 = try!(cursor.read_num_or_fail());
    try!(tiff_cursor.branch_with_offset_or_fail(tiff_offset as usize))
  } else {
    try!(cursor.branch_with_offset_or_fail(0))
  };

  //move the cursor ref we got past this exif value
  //so cursor is at the next value
  if let Some(err) = cursor.skip_or_fail(4) {
    return Err(err);
  }

  let len = components;
  let variant = match format {
    1 | 7 => ExifVariant::Bytes(try!(value_cursor.read_bytes_or_fail(len as usize))),
    2 => ExifVariant::Text(try!(value_cursor.read_str_or_fail(len as usize))),
    3 => ExifVariant::UShort(ValueIterator::<u16>::new(value_cursor, len)),
    4 => ExifVariant::UInt(ValueIterator::<u32>::new(value_cursor, len)),
    5 => ExifVariant::UIntFraction(ValueIterator::<(u32, u32)>::new(value_cursor, len)),
    6 => ExifVariant::Byte(ValueIterator::<i8>::new(value_cursor, len)),
    8 => ExifVariant::Short(ValueIterator::<i16>::new(value_cursor, len)),
    9 => ExifVariant::Int(ValueIterator::<i32>::new(value_cursor, len)),
    10 => ExifVariant::IntFraction(ValueIterator::<(i32, i32)>::new(value_cursor, len)),
    11 => ExifVariant::Float(ValueIterator::<f32>::new(value_cursor, len)),
    12 => ExifVariant::Double(ValueIterator::<f64>::new(value_cursor, len)),
    _ => return Err(ParseError::InvalidValueFormat{ format: format } )
  };

  let tag = RawExifTag {
    tag_type: tag_type,
    format: format,
    value: variant
  };
  Ok(tag)
}

pub enum ExifVariant<'a> {
  Text(&'a str),
  Bytes(&'a [u8]),
  UByte(ValueIterator<'a, u8>),
  UShort(ValueIterator<'a, u16>),
  UInt(ValueIterator<'a, u32>),
  UIntFraction(ValueIterator<'a, (u32, u32)>),
  Byte(ValueIterator<'a, i8>),
  Short(ValueIterator<'a, i16>),
  Int(ValueIterator<'a, i32>),
  IntFraction(ValueIterator<'a, (i32, i32)>),
  Float(ValueIterator<'a, f32>),
  Double(ValueIterator<'a, f64>)
}

pub fn read_exif_header<'a>(app1_cursor: &mut Cursor<'a>) -> ParseResult<Cursor<'a>> {
	let header = try!(app1_cursor.read_bytes_or_fail(6));

	if header != b"Exif\0\0" {
	  let h = header;
	  let header_array = [h[0], h[1], h[2], h[3], h[4], h[5]];
		return Err(ParseError::InvalidExifHeader{ header: header_array });
	}

	let mut tiff_marker = try!(app1_cursor.branch_or_fail(app1_cursor.len()));
	let tiff_header : u16 = try!(app1_cursor.read_num_or_fail());

	if tiff_header == 0x4949 {
		app1_cursor.set_endianness(Endianness::Little);
	}
	else if tiff_header == 0x4D4D {
		app1_cursor.set_endianness(Endianness::Big);
	}
	else {
		return Err(ParseError::InvalidTiffHeader{ header: tiff_header });
	}

	let tiff_data_marker : u16 = try!(app1_cursor.read_num_or_fail());

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
	use super::read_exif_header;

	#[test]
	fn test_read_exif_header() {
		let mut cursor = Cursor::new(JPEG_SAMPLE, Endianness::Little);
		assert!(read_exif_header(&mut cursor).is_err());
		let mut cursor = Cursor::new(JPEG_SAMPLE, Endianness::Little);
		cursor = cursor.skip(JPEG_SAMPLE_EXIF_OFFSET).unwrap();
		assert!(read_exif_header(&mut cursor).is_ok());
	}
}
