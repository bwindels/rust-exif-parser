use cursor::Cursor;
use tag::RawExifTag;
use super::TagCombiner;
use error::ParseResult;

pub enum MimeType {}

pub struct ThumbnailInfo {
  mimetype: MimeType,
  offset: usize,
  length: usize,
}

pub struct ThumbnailCombiner<'a> {
  tiff_cursor: Cursor<'a>,
  pub offset: Option<RawExifTag<'a>>,
  pub length: Option<RawExifTag<'a>>,
  pub compression: Option<RawExifTag<'a>>
}


impl<'a> TagCombiner<ThumbnailInfo> for ThumbnailCombiner<'a> {
  fn try_combine_tags(&self) -> Option<ParseResult<ThumbnailInfo>> {
    None
  }
  /*
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
  */
}


