use ::tags::Section;
use ::tag::RawExifTag;

trait TagCombiner {
	fn add_raw_tag(&mut self, tag: &RawExifTag<'a>, section: Section) -> (bool, Option<Tag<'a>>);
}

pub enum Tag<'a> {
	ImageDescription(&'a str)
	Make(&'a str),
	Model(&'a str),
	ModifyDate(DateTime),
	DateTimeOriginal(DateTime),
	CreateDate(DateTime),
	GPSLatitude(f64),
	GPSLongitude(f64),
	Thumbnail(MimeType, &'a [u8]),

	Other(RawExifTag<'a>)
}

fn cast_text<'a>(raw_tag: &RawExifTag<'a>) -> ParseResult<&'a str> {
	match raw_tag.value {
		ExifVariant::Text(ref text) => Ok(text),
		_ => Err(ParseError::MalformedTag(tag.no, tag.format))
	}
}

pub fn simplify_raw_tag<'a>(raw_tag: RawExifTag<'a>) -> Tag {
	let simple_tag : Option<ParseResult<Tag<'a>>> = match (section, raw_tag.no) {
		(Section::IFD1, 0x0201) => {
			self.thumbnail.offset = Some(raw_tag);
			self.thumbnail.try_cast_value().map(|t| => Tag::Thumbnail(t))
		},
		(Section::IFD1, 0x0202) => {
			self.thumbnail.length = Some(raw_tag);
			self.thumbnail.try_cast_value().map(|t| => Tag::Thumbnail(t))
		},
		(Section::IFD0, 0x0103) => {
			self.thumbnail.compression = Some(raw_tag);
			self.thumbnail.try_cast_value().map(|t| => Tag::Thumbnail(t))
		},
		(Section::GPS, 0x0001) => {
			self.gps.latitude.reference = Some(raw_tag);
			self.gps.latitude.try_cast_value().map(|lat| => Tag::Latitude(lat))
		},
		(Section::GPS, 0x0002) => {
			self.gps.latitude.degrees = Some(raw_tag);
			self.gps.latitude.try_cast_value().map(|lat| => Tag::Latitude(lat))
		},
		(Section::GPS, 0x0003) => {
			self.gps.longitude.reference = Some(raw_tag);
			self.gps.longitude.try_cast_value().map(|lon| => Tag::Longitude(lon))
		},
		(Section::GPS, 0x0004) => {
			self.gps.longitude.degrees = Some(raw_tag);
			self.gps.longitude.try_cast_value().map(|lon| => Tag::Longitude(lon))
		},
		//Date tags
		(Section::SubIFD, 0x0132) => {
			Some(cast_datetime(&raw_tag).map(|dt| Tag::ModifyDate(dt)))
		},
		(Section::SubIFD, 0x9003) => {
			Some(cast_datetime(&raw_tag).map(|dt| Tag::DateTimeOriginal(dt)))
		},
		(Section::SubIFD, 0x9004) => {
			Some(cast_datetime(&raw_tag).map(|dt| Tag::CreateDate(dt)))
		},
		//Text values
		(Section::IFD0, 0x010e) => {
			Some(cast_text(&raw_tag).map(|text| Tag::ImageDescription(text)))
		},
		(Section::IFD0, 0x010f) => {
			Some(cast_text(&raw_tag).map(|text| Tag::Make(text)))
		},
		(Section::IFD0, 0x0110) => {
			Some(cast_text(&raw_tag).map(|text| Tag::Model(text)))
		},
		
		//Unsupported values
		_ => {
			Some(Tag::Other(section, raw_tag))
		},
	}
}