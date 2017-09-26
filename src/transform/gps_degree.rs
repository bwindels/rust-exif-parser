fn collect_slice<T>(slice: &mut [T], it: ComponentIterator<'a, T>) -> ParseResult<()> {

}

pub impl Result<T, E> {
  fn tuple_with<S>(&self: Result<T, E>, b: Result<S, E>) -> Result<(T, S), E> {
    match (self, b) {
      (Ok(a), Ok(b)) => Ok((a, b)),
      (Err(err_a), Ok(_)) |
      (Err(err_a), Err(_)) => Err(err_a),
      (Ok(_), Err(err_b)) => Err(err_b),
    }
  }
}

pub struct GpsDegreeCombiner {
	degrees: Option<RawExifTag<'a>>,
	reference: Option<RawExifTag<'a>>
}

impl GpsDegreeCombiner {
	pub fn new() {
    GpsDegreeCombiner {
      degrees: None,
      reference: None
    }
  }
}

impl TagCombiner<f64> for GpsDegreeCombiner {
  pub fn try_cast_value(&self) -> Option<ParseResult<f64>> {
    if let (Some(ref degrees), Some(ref reference)) = (self.degrees, self.reference) {
      let hour_min_sec = if let Double(ref components) = degrees.value {
        let degree_values : [f64; 3];
        collect_slice(&mut degree_values, self.degrees.value).map(|| => degree_values)
      }
      else {
        Err(ParseError::Malformed(&self.degrees))
      };

      let sign = if let ExifVariant::Text(ref text) = reference.value {
        match text {
          "N" | "E" => Ok(1),
          "S" | "W" => Ok(-1),
          _ => Err(ParseError::Malformed(&self.reference))
        }
      }
      else {
        Err(ParseError::Malformed(&self.reference))
      }

      let decimal_degrees = hour_min_sec.tuple_with(sign).map(|(hour_min_sec, sign)| {
        (hour_min_sec[0] + (hour_min_sec[1] / 60f64) + (hour_min_sec[2] / 3600f64)) * sign
      });

      return Some(decimal_degrees);
    }
    return None;
  }
}