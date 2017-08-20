use ::cursor::{Cursor};
use ::component::TagComponents;
use ::error::{ParseError, ParseResult};

pub const EXIF_TAG_SIZE : usize = 2 + 2 + 4 + 4;

pub enum ExifVariant<'a> {
  Text(&'a str),
  Bytes(&'a [u8]),
  SignedByte(TagComponents<'a, i8>),
  UShort(TagComponents<'a, u16>),
  UInt(TagComponents<'a, u32>),
  UIntFraction(TagComponents<'a, (u32, u32)>),
  Short(TagComponents<'a, i16>),
  Int(TagComponents<'a, i32>),
  IntFraction(TagComponents<'a, (i32, i32)>),
  Float(TagComponents<'a, f32>),
  Double(TagComponents<'a, f64>)
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
      ExifVariant::UShort(TagComponents::<u16>::new(value_cursor, len)),
      ExifFormat::UInt =>
      ExifVariant::UInt(TagComponents::<u32>::new(value_cursor, len)),
      ExifFormat::UIntFraction =>
      ExifVariant::UIntFraction(TagComponents::<(u32, u32)>::new(value_cursor, len)),
      ExifFormat::SignedByte =>
      ExifVariant::SignedByte(TagComponents::<i8>::new(value_cursor, len)),
      ExifFormat::Short =>
      ExifVariant::Short(TagComponents::<i16>::new(value_cursor, len)),
      ExifFormat::Int =>
      ExifVariant::Int(TagComponents::<i32>::new(value_cursor, len)),
      ExifFormat::IntFraction =>
      ExifVariant::IntFraction(TagComponents::<(i32, i32)>::new(value_cursor, len)),
      ExifFormat::Float =>
      ExifVariant::Float(TagComponents::<f32>::new(value_cursor, len)),
      ExifFormat::Double =>
      ExifVariant::Double(TagComponents::<f64>::new(value_cursor, len))
    };
    Ok(variant)
  }
}

pub struct RawExifTag<'a> {
  pub tag_type: u16,
  pub format: ExifFormat,
  pub value: ExifVariant<'a>
}

pub fn read_exif_tag<'a>(mut cursor: Cursor<'a>, tiff_cursor: Cursor<'a>) -> ParseResult<RawExifTag<'a>> {
  let tag_type : u16 = cursor.read_num_or_fail()?;
  let format_num : u16 = cursor.read_num_or_fail()?;
  let components : u32 = cursor.read_num_or_fail()?;

  let format = ExifFormat::from(format_num)?;
  let total_values_bytes = format.bytes_per_component() * components as usize;

  let value_cursor = if total_values_bytes > 4 {
    //TODO: have an option to be more forgiving about EOF
    //here since this will inhibit us from discovering
    //the tags to come with embedded values.
    let tiff_offset : u32 = cursor.read_num_or_fail()?;
    tiff_cursor.with_skip_or_fail(tiff_offset as usize)?
  } else {
    cursor
  };

  //to reduce the complexity of reading a single tag,
  //we fail early if we'll hit an EOF. It doesn't make
  //sense to read half of an exif tag,
  //and deal with errors in the ComponentIterator
  if value_cursor.len() < total_values_bytes {
    return Err(ParseError::UnexpectedEOF);
  }

  let variant = format.variant_from_cursor(
    value_cursor, components)?;

  let tag = RawExifTag {
    tag_type: tag_type,
    format: format,
    value: variant
  };

  return Ok(tag);
}

#[cfg(test)]
mod tests {

	use ::cursor::{Cursor, Endianness};
	use super::{
    read_exif_tag,
    ExifFormat,
    ExifVariant
  };

  #[test]
  fn test_simple_uint_tag() {
    const EXIF_TAG : &'static [u8] = &[
      0u8, 200u8,
      0u8, 4u8,
      0u8, 0u8, 0u8, 1u8,
      0u8, 0u8, 0u8, 240u8
    ];
    const EXIF_POINTER_AREA : &'static [u8] = &[];

    let cursor = Cursor::new(EXIF_TAG, Endianness::Big);
    let tag = read_exif_tag(cursor,
      Cursor::new(EXIF_POINTER_AREA, Endianness::Big));
    let tag = tag.expect("tag should be ok");
    assert_eq!(tag.tag_type, 200);
    assert_eq!(tag.format, ExifFormat::UInt);
    match tag.value {
      ExifVariant::UInt(ref components) => {
        let mut it = components.iter();
        assert_eq!(it.next().expect("first value in iterator should be ok"), 240);
        assert!(it.next().is_none());
      },
      _ => panic!("tag value should be of type {}", EXIF_TAG[3])
    };
  }

  #[test]
  fn read_simple_text_tag() {
    const EXIF_TAG : &'static [u8] = &[
      0u8, 210u8,//tag
      0u8, 2u8, //string
      0u8, 0u8, 0u8, 3u8, //3 values
      65u8, 66u8, 67u8, 0u8, //ABC
    ]; //offset to data area;
    const EXIF_POINTER_AREA : &'static [u8] = &[];
    let cursor = Cursor::new(EXIF_TAG, Endianness::Big);
    let data_cursor = Cursor::new(EXIF_POINTER_AREA, Endianness::Big);
    let tag = read_exif_tag(cursor, data_cursor)
      .expect("read should not fail");
    assert_eq!(tag.tag_type, 210);
    assert_eq!(tag.format, ExifFormat::Text);
    match tag.value {
      ExifVariant::Text(text) => assert_eq!(text, "ABC"),
      _ => panic!("value should be text")
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

    let cursor = Cursor::new(EXIF_TAG, Endianness::Big);
    let tag = read_exif_tag(cursor,
      Cursor::new(EXIF_POINTER_AREA, Endianness::Big));
    let tag = tag.expect("tag should be ok");
    assert_eq!(tag.tag_type, 210);
    assert_eq!(tag.format, ExifFormat::UInt);
    match tag.value {
      ExifVariant::UInt(ref components) => {
        let mut it = components.iter();
        assert_eq!(it.next().expect("first value in iterator should be ok"), 120);
        assert_eq!(it.next().expect("second value in iterator should be ok"), 160);
        assert!(it.next().is_none(), "there should only be 2 values");
      },
      _ => panic!("tag value should be of type {}", EXIF_TAG[3])
    };
  }
}
