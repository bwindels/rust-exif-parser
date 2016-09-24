use ::cursor::Cursor;

fn has_exif_header<'a>(app1_cursor: &mut Cursor<'a>) -> bool {
	let header = require!(app1_cursor.read_str(6), false);
	header == "Exif\0\0"
}

#[cfg(test)]
mod tests {

	use ::cursor::{Cursor, Endianness};
	use ::test_fixtures::{JPEG_SAMPLE, JPEG_SAMPLE_EXIF_OFFSET};
	use super::{
		has_exif_header
	};

	#[test]
	fn test_has_exif_header() {
		let mut cursor = Cursor::new(JPEG_SAMPLE, Endianness::Little);
		assert!(!has_exif_header(&mut cursor));
		let mut cursor = cursor.skip(JPEG_SAMPLE_EXIF_OFFSET).unwrap();
		assert!(has_exif_header(&mut cursor));
		
	}


}