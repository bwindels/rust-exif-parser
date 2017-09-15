use std::str::FromStr;

pub struct DateTime {
	year: u16,
	month: u8,
	day: u8,
	hour: u8,
	minute: u8,
	second: u8
}

impl DateTime {
	//parses a date in format `2013:05:10 15:21:35`
	pub fn parse(date_str: & str) -> Option<DateTime> {
		let year_str = date_str.get(0 .. 4);
		let month_str = date_str.get(5 .. 7);
		let day_str = date_str.get(8 .. 10);
		let hour_str = date_str.get(11 .. 13);
		let minute_str = date_str.get(14 .. 16);
		let second_str = date_str.get(17 .. 19);

		if let (Some(year_str), Some(month_str), Some(day_str),
				    Some(hour_str), Some(minute_str), Some(second_str)) = 
       (year_str, month_str, day_str, hour_str, minute_str, second_str)
    {
  		let year = u16::from_str(year_str);
  		let month = u8::from_str(month_str);
  		let day = u8::from_str(day_str);
  		let hour = u8::from_str(hour_str);
  		let minute = u8::from_str(minute_str);
  		let second = u8::from_str(second_str);
  		
  		if let (Ok(year), Ok(month), Ok(day),
  			    Ok(hour), Ok(minute), Ok(second)) = 
  		       (year, month, day, hour, minute, second)
  		{
  		   	return Some(DateTime {year, month, day, hour, minute, second});
  		}
    }
		return None;
	}
}

#[cfg(test)]
mod tests {
	#[test]
	fn test_datetime_parse_success() {
		let d = super::DateTime::parse("2013:05:10 15:21:35").unwrap();
		assert_eq!(d.year, 2013);
		assert_eq!(d.month, 5);
		assert_eq!(d.day, 10);
		assert_eq!(d.hour, 15);
		assert_eq!(d.minute, 21);
		assert_eq!(d.second, 35);
	}

	#[test]
	fn test_datetime_parse_too_short() {
		let d = super::DateTime::parse("2013:05:10 15:21:3");
		assert!(d.is_none());
	}

	#[test]
	fn test_datetime_parse_only_decimal() {
		let d = super::DateTime::parse("aaaaaaaaaaaaaaaaaaa");
		assert!(d.is_none());
	}

	#[test]
	fn test_datetime_parse_no_panic_with_multibyte() {
		let d = super::DateTime::parse("👨‍👨‍👦‍👦👮🏻👩🏿👨🏼🚶🐗🍀👨‍👨‍👦‍👦👮🏻👩🏿👨🏼🚶🐗🍀");
		assert!(d.is_none());
	}

}