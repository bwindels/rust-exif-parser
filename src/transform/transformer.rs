use tags::{Section, ExifTagIterator};
use tag::RawExifTag;
use super::gps_degree::GpsDegreeCombiner;
use super::TagCombiner;
use super::datetime::to_datetime;
use super::text::to_text;
use super::thumbnail::ThumbnailCombiner;
use error::ParseResult;
use super::tag::Tag;

fn option_to_tag<'a, T, F>(option: Option<ParseResult<T>>, map: F) -> Option<ParseResult<Tag<'a>>> where F: FnOnce(T) -> Tag<'a> {
  option.map(|r| r.map(map))
}

struct GpsCombiners<'a> {
  latitude: GpsDegreeCombiner<'a>,
  longitude: GpsDegreeCombiner<'a>
}

pub struct TagTransformer<'a> {
  raw_tag_it: ExifTagIterator<'a>,
  thumbnail: ThumbnailCombiner<'a>,
  gps: GpsCombiners<'a>,
  raw_tags: ExifTagIterator<'a>
}

enum Matched<'a, T> {
  Matched(T),
  Unmatched(RawExifTag<'a>)
}

use self::Matched::*;
use self::Section::*;

impl<'a> TagTransformer<'a> {

  fn transform_thumbnail_tags<'b: 'a>(&mut self, raw_tag: RawExifTag<'b>, section: Section) -> Matched<Option<ParseResult<Tag<'b>>>> {
    let r = match (section, raw_tag.no) {
      (IFD1, 0x0201) => {
        self.thumbnail.offset = Some(raw_tag);
        option_to_tag(self.thumbnail.try_combine_tags(), |thumb| Tag::Thumbnail(thumb))
      },
      (IFD1, 0x0202) => {
        self.thumbnail.length = Some(raw_tag);
        option_to_tag(self.thumbnail.try_combine_tags(), |thumb| Tag::Thumbnail(thumb))
      },
      (IFD0, 0x0103) => {
        self.thumbnail.compression = Some(raw_tag);
        option_to_tag(self.thumbnail.try_combine_tags(), |thumb| Tag::Thumbnail(thumb))
      },
      _ => return Unmatched(raw_tag),
    };
    return Matched(r);
  }

  fn transform_gps_tags<'b: 'a>(&mut self, raw_tag: RawExifTag<'b>, section: Section) -> Matched<Option<ParseResult<Tag<'b>>>> {
    let r = match (section, raw_tag.no) {
      (GPS, 0x0001) => {
        self.gps.latitude.reference = Some(raw_tag);
        option_to_tag(self.gps.latitude.try_combine_tags(), |lat| Tag::GPSLatitude(lat))
      },
      (GPS, 0x0002) => {
        self.gps.latitude.degrees = Some(raw_tag);
        option_to_tag(self.gps.latitude.try_combine_tags(), |lat| Tag::GPSLatitude(lat))
      },
      (GPS, 0x0003) => {
        self.gps.longitude.reference = Some(raw_tag);
        option_to_tag(self.gps.longitude.try_combine_tags(), |lon| Tag::GPSLongitude(lon))
      },
      (GPS, 0x0004) => {
        self.gps.longitude.degrees = Some(raw_tag);
        option_to_tag(self.gps.longitude.try_combine_tags(), |lon| Tag::GPSLongitude(lon))
      },
      _ => return Unmatched(raw_tag)
    };
    return Matched(r);
  }

  fn transform_date_tags<'b: 'a>(&mut self, raw_tag: RawExifTag<'b>, section: Section) -> Matched<Option<ParseResult<Tag<'b>>>> {
    let r = match (section, raw_tag.no) {
      (SubIFD, 0x0132) => {
        Some(to_datetime(&raw_tag).map(|dt| Tag::ModifyDate(dt)))
      },
      (SubIFD, 0x9003) => {
        Some(to_datetime(&raw_tag).map(|dt| Tag::DateTimeOriginal(dt)))
      },
      (SubIFD, 0x9004) => {
        Some(to_datetime(&raw_tag).map(|dt| Tag::CreateDate(dt)))
      },
      _ => return Unmatched(raw_tag)
    };
    return Matched(r);
  }

  fn transform_text_tags<'b: 'a>(&mut self, raw_tag: RawExifTag<'b>, section: Section) -> Matched<Option<ParseResult<Tag<'b>>>> {
    let r = match (section, raw_tag.no) {
      (IFD0, 0x010e) => {
        Some(to_text(&raw_tag).map(|text| Tag::ImageDescription(text)))
      },
      (IFD0, 0x010f) => {
        Some(to_text(&raw_tag).map(|text| Tag::Make(text)))
      },
      (IFD0, 0x0110) => {
        Some(to_text(&raw_tag).map(|text| Tag::Model(text)))
      },
      _ => return Unmatched(raw_tag)
    };
    return Matched(r);
  }

  fn transform_raw_tag<'b: 'a>(&mut self, raw_tag: RawExifTag<'b>, section: Section) -> Option<ParseResult<Tag<'b>>> {
    let raw_tag = match self.transform_gps_tags(raw_tag, section) {
      Matched(tag_option) => return tag_option,
      Unmatched(raw_tag) => raw_tag
    };
    let raw_tag = match self.transform_thumbnail_tags(raw_tag, section) {
      Matched(tag_option) => return tag_option,
      Unmatched(raw_tag) => raw_tag
    };
    let raw_tag = match self.transform_date_tags(raw_tag, section) {
      Matched(tag_option) => return tag_option,
      Unmatched(raw_tag) => raw_tag
    };
    let raw_tag = match self.transform_text_tags(raw_tag, section) {
      Matched(tag_option) => return tag_option,
      Unmatched(raw_tag) => raw_tag
    };
    return Some(Ok(Tag::Other(raw_tag)));
  }
}

impl<'a> Iterator for TagTransformer<'a> {
  type Item = ParseResult<Tag<'a>>;

  fn next(&mut self) -> Option<Self::Item> {
    None
  }
}
