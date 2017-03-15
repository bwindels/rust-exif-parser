#[derive(Debug, PartialEq)]
pub enum ParseError {
  InvalidValueFormat{ format: u16 },
  InvalidExifHeader{ header: [u8;6] },
  InvalidTiffHeader{ header: u16 },
  InvalidTiffData{ data: u16 },
  UnexpectedEOF,
  InvalidJPEGSegmentHeader{ header: u8 }
}

pub type ParseResult<T> = Result<T, ParseError>;
