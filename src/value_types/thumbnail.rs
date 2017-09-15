
pub struct ThumbnailCombiner<'a> {
	tiff_cursor: Cursor<'a>,
	thumbnail_offset: Option<usize>,
	thumbnail_len: Option<usize>,
	thumbnail_type: Option<u8>
}

impl<'a> TagCombiner for ThumbnailCombiner<'a> {
	fn add_raw_tag(&mut self, tag: &RawExifTag<'a>, section: Section) -> (bool, Option<Tag<'a>>) {
		let tag_comsumed = match tag.no {
			//ThumbnailOffset
			0x0201 => {
				self.thumbnail_offset = Some();
				true
			},
			//ThumbnailLength
			0x0202 => {
				self.thumbnail_len = Some();
				true
			},
			//Compression
			//TODO: is this specific to the thumbnail only?
			0x0103 => {
				self.thumbnail_type = Some();
				true
			}
			_ => false
		};
		if let (Some(offset), Some(len)) = (self.thumbnail_offset, self.thumbnail_len) {
			if let Some(thumbnail_cursor) = self.tiff_cursor.with_skip(offset) {
				if let Some(buffer) = thumbnail_cursor.read_bytes(len) {
					return (tag_comsumed, Some(Tag::Thumbnail(buffer))));
				}
			}
		}
		return (tag_comsumed, None);
	}
}

