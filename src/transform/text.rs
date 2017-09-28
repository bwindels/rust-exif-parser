use tag::{RawExifTag, ExifVariant};
use error::{ParseError, ParseResult};

pub fn to_text<'a>(raw_tag: &RawExifTag<'a>) -> ParseResult<&'a str> {
	match raw_tag.value {
		ExifVariant::Text(ref text) => Ok(text),
		_ => Err(ParseError::MalformedTag)
	}
}