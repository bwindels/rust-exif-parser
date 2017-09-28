use error::{ParseError, ParseResult};
use tag::{RawExifTag, ExifVariant};

pub fn to_text<'a>(raw_tag: &RawExifTag<'a>) -> ParseResult<&'a str> {
  match raw_tag.value {
    ExifVariant::Text(ref text) => Ok(text),
    _ => Err(ParseError::MalformedTag)
  }
}

pub fn to_u32(raw_tag: &RawExifTag) -> ParseResult<u32> {
  match raw_tag.value {
    ExifVariant::UInt(ref components) => {
      components.iter().next().ok_or(ParseError::MalformedTag)
    },
    _ => Err(ParseError::MalformedTag)
  }
}

pub fn to_u16(raw_tag: &RawExifTag) -> ParseResult<u16> {
  match raw_tag.value {
    ExifVariant::UShort(ref components) => {
      components.iter().next().ok_or(ParseError::MalformedTag)
    },
    _ => Err(ParseError::MalformedTag)
  }
}

