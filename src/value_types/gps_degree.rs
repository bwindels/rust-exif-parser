fn collect_slice<T>(&mut slice: [T], it: ComponentIterator<'a, T>) -> ParseResult<()> {

}


pub struct GpsDegreeTagCombiner {
	degrees: Option<RawExifTag<'a>>,
	reference: Option<RawExifTag<'a>>
}

impl GpsDegreesCombiner {
	pub fn new() {
    GpsDegreesCombiner {
      degrees: None,
      reference: None
    }
  }
}

impl TagCombiner<f64> for GpsDegreesCombiner {
  pub fn try_cast_value(&mut self) -> Option<ParseResult<f64>> {
    if let (Some(ref degrees), Some(ref reference)) = (self.degrees, self.reference) {
      
      let degrees_slice = if let Double(ref components) = degrees.value {
        let degree_values : [f64; 3];
        collect_slice(&mut degree_values, self.degrees.value).map(|| => degree_values)
      }
      else {
        ParseResult::Malformed(&self.degrees)
      };

      let reference_positive = if let ExifVariant::Text(ref text) = reference.value {
        match text {
          "N" | "E" => Ok(true),
          "S" | "W" => Ok(false),
          _ => ParseResult::Malformed(&self.reference)
        }
      }
      else {
        ParseResult::Malformed(&self.reference)
      }
      //is there a better way of joining these two results? ...
      if let (Ok(degrees_slice), Ok(reference_positive)) = (degrees_slice, reference_positive) {
        let degree_sign : f64 = reference_positive ? 1 : -1;
        let degree = (degrees_slice[0] + (degrees_slice[1] / 60f64) + (degrees_slice[2] / 3600f64)) * degree_sign;
        Ok(degree)
      }
      // ... because this way it's hard get the error of the result that failed and cast it to ParseResult<f64>
      else {
        //
      }

      Some(result)
    }
    None
  }
}