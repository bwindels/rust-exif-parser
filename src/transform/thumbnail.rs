use cursor::Cursor;
use tag::RawExifTag;
use super::TagCombiner;
use error::ParseResult;
use super::number::{to_u32, to_u16};

pub enum MimeType {
  Jpeg,
  Tiff,
  Unknown
}

fn parse_compression(raw_tag: &RawExifTag) -> ParseResult<MimeType> {
  to_u16(raw_tag).map(|n| {
    match n {
      6 => MimeType::Jpeg,
      _ => MimeType::Unknown
    }
  })
}

pub struct ThumbnailInfo {
  mimetype: MimeType,
  offset: u32,
  length: u32,
}

pub struct ThumbnailCombiner<'a> {
  pub offset: Option<RawExifTag<'a>>,
  pub length: Option<RawExifTag<'a>>,
  pub compression: Option<RawExifTag<'a>>
}

impl<'a> ThumbnailCombiner<'a> {
  pub fn new() -> ThumbnailCombiner<'a> {
    ThumbnailCombiner {
      offset: None,
      length: None,
      compression: None
    }
  }
}

impl<'a> TagCombiner<ThumbnailInfo> for ThumbnailCombiner<'a> {
  fn try_combine_tags(&self) -> Option<ParseResult<ThumbnailInfo>> {
    if let (&Some(ref offset_tag), &Some(ref length_tag), &Some(ref compression_tag)) =
      (&self.offset, &self.length, &self.compression)
    {
      let offset = to_u32(&offset_tag);
      let length = to_u32(&length_tag);
      let mimetype = parse_compression(&compression_tag);

      match (offset, length, mimetype) {
        (Ok(offset), Ok(length), Ok(mimetype)) => {
          Some(Ok(ThumbnailInfo {
            length: length,
            offset: offset,
            mimetype: mimetype
          }))
        },
        (Err(err), _, _) |
        (Ok(_), Err(err), _) |
        (Ok(_), Ok(_), Err(err)) => {
          Some(Err(err))
        }
      }
    }
    else {
      None
    }
  }
}


