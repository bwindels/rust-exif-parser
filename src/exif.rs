use ::cursor::{Cursor, Endianness};

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
