use ::cursor::{Cursor, Endianness};

fn read_exif_header<'a>(app1_cursor: &mut Cursor<'a>) -> Result<Cursor<'a>, &'static str> {
	let header = try!(app1_cursor.read_bytes(6)
		.ok_or("Unexpected EOF 1"));

	if header != b"Exif\0\0" {
		return Err("Invalid exif header");
	}

	let mut tiff_marker = app1_cursor.branch(app1_cursor.len()).unwrap();
	let tiff_header : u16 = try!(app1_cursor.read_num()
		.ok_or("Unexpected EOF 2"));

	if tiff_header == 0x4949 {
		app1_cursor.set_endianness(Endianness::Little);
	}
	else if tiff_header == 0x4D4D {
		app1_cursor.set_endianness(Endianness::Big);
	}
	else {
		return Err("Invalid tiff header");
	}

	let tiff_data_marker : u16 = try!(app1_cursor.read_num()
		.ok_or("Unexpected EOF 3"));

	if tiff_data_marker != 0x002A {
		return Err("Invalid tiff data");
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
