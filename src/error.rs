#[derive(Debug, PartialEq)]
pub enum ParseError {
  InvalidValueFormat{ format: u16 },
  InvalidExifHeader{ header: [u8;6] },
  InvalidTiffHeader{ header: u16 },
  InvalidTiffData{ data: u16 },
  UnexpectedEOF,
  ValueOutOfBounds { offset: usize },
  InvalidJPEGSegmentHeader{ header: u8 },
  MalformedTag
}

pub type ParseResult<T> = Result<T, ParseError>;
