fn transform_text<'a>(raw_tag: &RawExifTag<'a>) -> ParseResult<&'a str> {
	match raw_tag.value {
		ExifVariant::Text(ref text) => Ok(text),
		_ => Err(ParseError::MalformedTag(tag.no, tag.format))
	}
}