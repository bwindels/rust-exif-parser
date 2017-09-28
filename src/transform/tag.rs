use super::datetime::DateTime;
use super::thumbnail::ThumbnailInfo;
use tag::RawExifTag;

pub enum Tag<'a> {
  ImageDescription(&'a str),
  Make(&'a str),
  Model(&'a str),
  ModifyDate(DateTime),
  DateTimeOriginal(DateTime),
  CreateDate(DateTime),
  GPSLatitude(f64),
  GPSLongitude(f64),
  Thumbnail(ThumbnailInfo),

  Other(RawExifTag<'a>)
}