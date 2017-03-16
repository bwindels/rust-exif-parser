use std::iter::Iterator;
use ::cursor::{Cursor, Endianness, ByteSwappable};
use ::error::{ParseError, ParseResult};
use std::marker::{Sized, PhantomData};

pub enum ExifVariant<'a> {
  Text(&'a str),
  Bytes(&'a [u8]),
  SignedByte(ValueIterator<'a, i8>),
  UShort(ValueIterator<'a, u16>),
  UInt(ValueIterator<'a, u32>),
  UIntFraction(ValueIterator<'a, (u32, u32)>),
  Short(ValueIterator<'a, i16>),
  Int(ValueIterator<'a, i32>),
  IntFraction(ValueIterator<'a, (i32, i32)>),
  Float(ValueIterator<'a, f32>),
  Double(ValueIterator<'a, f64>)
}

#[derive(PartialEq, Copy, Clone, Debug)]
pub enum ExifFormat {
  UByte,
  Text,
  UShort,
  UInt,
  UIntFraction,
  SignedByte,
  Binary,
  Short,
  Int,
  IntFraction,
  Float,
  Double
}

impl ExifFormat {
  fn from(format: u16) -> ParseResult<ExifFormat> {
    match format {
      1 => Ok(ExifFormat::UByte),
      2 => Ok(ExifFormat::Text),
      3 => Ok(ExifFormat::UShort),
      4 => Ok(ExifFormat::UInt),
      5 => Ok(ExifFormat::UIntFraction),
      6 => Ok(ExifFormat::SignedByte),
      7 => Ok(ExifFormat::Binary),
      8 => Ok(ExifFormat::Short),
      9 => Ok(ExifFormat::Int),
      10 => Ok(ExifFormat::IntFraction),
      11 => Ok(ExifFormat::Float),
      12 => Ok(ExifFormat::Double),
      _ => Err(ParseError::InvalidValueFormat{ format: format } )
    }
  }

  fn bytes_per_component(self) -> usize {
    match self {
      ExifFormat::UByte |
      ExifFormat::Text |
      ExifFormat::SignedByte |
      ExifFormat::Binary => 1,

      ExifFormat::UShort |
      ExifFormat::Short => 2,

      ExifFormat::UInt |
      ExifFormat::Int |
      ExifFormat::Float => 4,

      ExifFormat::UIntFraction |
      ExifFormat::IntFraction |
      ExifFormat::Double => 8
    }
  }

  fn variant_from_cursor<'a>(self, mut value_cursor: Cursor<'a>, len: u32)
  -> ParseResult<ExifVariant<'a>>
  {
    let variant = match self {
      ExifFormat::UByte | ExifFormat::Binary =>
      ExifVariant::Bytes(value_cursor.read_bytes_or_fail(len as usize)?),
      ExifFormat::Text =>
      ExifVariant::Text(value_cursor.read_str_or_fail(len as usize)?),
      ExifFormat::UShort =>
      ExifVariant::UShort(ValueIterator::<u16>::new(value_cursor, len)),
      ExifFormat::UInt =>
      ExifVariant::UInt(ValueIterator::<u32>::new(value_cursor, len)),
      ExifFormat::UIntFraction =>
      ExifVariant::UIntFraction(ValueIterator::<(u32, u32)>::new(value_cursor, len)),
      ExifFormat::SignedByte =>
      ExifVariant::SignedByte(ValueIterator::<i8>::new(value_cursor, len)),
      ExifFormat::Short =>
      ExifVariant::Short(ValueIterator::<i16>::new(value_cursor, len)),
      ExifFormat::Int =>
      ExifVariant::Int(ValueIterator::<i32>::new(value_cursor, len)),
      ExifFormat::IntFraction =>
      ExifVariant::IntFraction(ValueIterator::<(i32, i32)>::new(value_cursor, len)),
      ExifFormat::Float =>
      ExifVariant::Float(ValueIterator::<f32>::new(value_cursor, len)),
      ExifFormat::Double =>
      ExifVariant::Double(ValueIterator::<f64>::new(value_cursor, len))
    };
    Ok(variant)
  }
}

pub struct RawExifTag<'a> {
  pub tag_type: u16,
  pub format: ExifFormat,
  pub value: ExifVariant<'a>
}

trait ExifValueReader {
  fn read_exif_value(cursor: &mut Cursor) -> ParseResult<Self> where Self: Sized + Copy;
}

impl ExifValueReader for (u32, u32) {
  fn read_exif_value(cursor: &mut Cursor) -> ParseResult<Self> {
    let a : u32 = cursor.read_num_or_fail()?;
    let b : u32 = cursor.read_num_or_fail()?;
    Ok((a, b))
  }
}

impl ExifValueReader for (i32, i32) {
  fn read_exif_value(cursor: &mut Cursor) -> ParseResult<Self> {
    let a : i32 = cursor.read_num_or_fail()?;
    let b : i32 = cursor.read_num_or_fail()?;
    Ok((a, b))
  }
}

impl<T> ExifValueReader for T where T: ByteSwappable {
  fn read_exif_value(cursor: &mut Cursor) -> ParseResult<Self> {
    cursor.read_num_or_fail()
  }
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

pub fn read_exif_tag<'a>(cursor: &mut Cursor<'a>, tiff_cursor: &Cursor<'a>) -> ParseResult<RawExifTag<'a>> {
  let tag_type : u16 = cursor.read_num_or_fail()?;
  let format_num : u16 = cursor.read_num_or_fail()?;
  let components : u32 = cursor.read_num_or_fail()?;
  let format = ExifFormat::from(format_num)?;
  let total_values_bytes = format.bytes_per_component() * components as usize;

  let value_cursor = if total_values_bytes > 4 {
    let tiff_offset : u32 = cursor.read_num_or_fail()?;
    tiff_cursor.with_skip_or_fail(tiff_offset as usize)?
  } else {
    let c = cursor.clone();
    //move the cursor ref we got past this exif value
    //so cursor is at the next value
    *cursor = cursor.with_skip_or_fail(4)?;
    c
  };

  let variant = format.variant_from_cursor(
    value_cursor, components)?;

  let tag = RawExifTag {
    tag_type: tag_type,
    format: format,
    value: variant
  };

  Ok(tag)
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
    read_exif_header,
    read_exif_tag,
    ExifFormat,
    ExifVariant
  };

	#[test]
	fn test_read_exif_header() {
		let mut cursor = Cursor::new(JPEG_SAMPLE, Endianness::Little);
		assert!(read_exif_header(&mut cursor).is_err());
		let mut cursor = Cursor::new(JPEG_SAMPLE, Endianness::Little);
		cursor = cursor.with_skip_or_fail(JPEG_SAMPLE_EXIF_OFFSET).expect("EOF");
		assert!(read_exif_header(&mut cursor).is_ok());
	}

  #[test]
  fn test_simple_uint_exif_tag() {
    const EXIF_TAG : &'static [u8] = &[0u8, 200u8, 0u8, 4u8, 0u8, 0u8, 0u8, 1u8, 0u8, 0u8, 0u8, 240u8];
    const EXIF_POINTER_AREA : &'static [u8] = &[];

    let mut cursor = Cursor::new(EXIF_TAG, Endianness::Big);
    let tag = read_exif_tag(&mut cursor,
      &mut Cursor::new(EXIF_POINTER_AREA, Endianness::Big));
    let tag = tag.expect("tag should be ok");
    assert_eq!(tag.tag_type, 200);
    assert_eq!(tag.format, ExifFormat::UInt);
    match tag.value {
      ExifVariant::UInt(mut it) => {
        assert_eq!(it.next().expect("first value in iterator should be ok"), 240);
        assert!(it.next().is_none());
      },
      _ => panic!("tag value should be of type {}", EXIF_TAG[3])
    };
  }

  #[test]
  fn test_extended_uint_exif_tag() {
    const EXIF_TAG : &'static [u8] = &[
      0u8, 210u8,//tag
      0u8, 4u8, //uint
      0u8, 0u8, 0u8, 2u8, //2 values
      0u8, 0u8, 0u8, 6u8]; //offset to data area;
    /* first 4 bytes are skipped by offset in exif tag value */
    const EXIF_POINTER_AREA : &'static [u8] = &[
      0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
      0u8, 0u8, 0u8, 120u8,
      0u8, 0u8, 0u8, 160u8];

    let mut cursor = Cursor::new(EXIF_TAG, Endianness::Big);
    let tag = read_exif_tag(&mut cursor,
      &mut Cursor::new(EXIF_POINTER_AREA, Endianness::Big));
    let tag = tag.expect("tag should be ok");
    assert_eq!(tag.tag_type, 210);
    assert_eq!(tag.format, ExifFormat::UInt);
    match tag.value {
      ExifVariant::UInt(mut it) => {
        assert_eq!(it.next().expect("first value in iterator should be ok"), 120);
        assert_eq!(it.next().expect("second value in iterator should be ok"), 160);
        assert!(it.next().is_none(), "there should only be 2 values");
      },
      _ => panic!("tag value should be of type {}", EXIF_TAG[3])
    };
  }
}
