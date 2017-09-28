use error::ParseResult;

mod datetime;
mod text;
mod gps_degree;
mod transformer;
mod tag;
mod thumbnail;

pub trait TagCombiner<T> {
  fn try_combine_tags(&self) -> Option<ParseResult<T>>;
}