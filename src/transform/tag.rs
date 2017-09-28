use super::datetime::DateTime;
use super::thumbnail::ThumbnailInfo;
use super::gps_degree::GpsPosition;
use tag::RawExifTag;

pub enum Tag<'a> {
  ImageDescription(&'a str),
  Make(&'a str),
  Model(&'a str),
  ModifyDate(DateTime),
  DateTimeOriginal(DateTime),
  CreateDate(DateTime),
  GpsPosition(GpsPosition),
  Thumbnail(ThumbnailInfo),
  Other(RawExifTag<'a>)
}