use error::ParseResult;

mod datetime;
mod text;
mod gps;
mod transformer;
mod tag;
mod thumbnail;
mod number;

pub trait TagCombiner<T> {
  fn try_combine_tags(&self) -> Option<ParseResult<T>>;
}