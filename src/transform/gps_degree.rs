use component::ComponentIterator;
use error::{ParseResult, ParseError};
use tag::{RawExifTag, ExifVariant};
use super::TagCombiner;
use super::text::to_text;
use component::ExifValueReader;

fn collect_slice<'a, T>(slice: &mut [T], it: ComponentIterator<'a, T>) -> ParseResult<()> where T: Copy + ExifValueReader {
  for i in 0..slice.len() {
    match it.next() {
      Some(v) => slice[i] = v,
      None => return Err(ParseError::MalformedTag)
    }
  }
  return Ok(())
}

pub struct GpsDegreeCombiner<'a> {
	pub degrees: Option<RawExifTag<'a>>,
	pub reference: Option<RawExifTag<'a>>
}

impl<'a> GpsDegreeCombiner<'a> {
	pub fn new() -> GpsDegreeCombiner<'a> {
    GpsDegreeCombiner {
      degrees: None,
      reference: None
    }
  }
}

impl<'a> TagCombiner<f64> for GpsDegreeCombiner<'a> {
  fn try_combine_tags(&self) -> Option<ParseResult<f64>> {
    if let (&Some(ref degrees), &Some(ref reference)) = (&self.degrees, &self.reference) {
      let hour_min_sec = if let ExifVariant::Double(ref components) = degrees.value {
        let mut degree_values = [0f64; 3];
        collect_slice(&mut degree_values, components.iter()).map(|_| degree_values)
      }
      else {
        Err(ParseError::MalformedTag) //self.degrees
      };

      let sign = to_text(&reference).and_then(|text| {
        match text {
          "N" | "E" => Ok(1f64),
          "S" | "W" => Ok(-1f64),
          _ => Err(ParseError::MalformedTag)
        }
      });

      let hms_and_sign = hour_min_sec.and_then(|hms| sign.map(|s| (hms, s)));
      let decimal_degrees = hms_and_sign.map(|(hour_min_sec, sign)| {
        (hour_min_sec[0] + (hour_min_sec[1] / 60f64) + (hour_min_sec[2] / 3600f64)) * sign
      });

      return Some(decimal_degrees);
    }
    return None;
  }
}