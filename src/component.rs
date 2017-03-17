use std::iter::Iterator;
use ::cursor::{Cursor, ByteSwappable};
use ::error::ParseResult;
use std::marker::{Sized, PhantomData};

trait ExifValueReader {
  fn read_exif_value(cursor: &mut Cursor) -> ParseResult<Self> where Self: Sized + Copy;
}

impl ExifValueReader for (u32, u32) {
  fn read_exif_value(cursor: &mut Cursor) -> ParseResult<Self> {
    let a : u32 = cursor.read_num_or_fail()?;
    let b : u32 = cursor.read_num_or_fail()?;
    Ok((a, b))
  }
}

impl ExifValueReader for (i32, i32) {
  fn read_exif_value(cursor: &mut Cursor) -> ParseResult<Self> {
    let a : i32 = cursor.read_num_or_fail()?;
    let b : i32 = cursor.read_num_or_fail()?;
    Ok((a, b))
  }
}

impl<T> ExifValueReader for T where T: ByteSwappable {
  fn read_exif_value(cursor: &mut Cursor) -> ParseResult<Self> {
    cursor.read_num_or_fail()
  }
}

pub struct ComponentIterator<'a, T> {
  value_cursor: Cursor<'a>,
  len: u32,
  i: u32,
  phantom_data: PhantomData<T>
}

impl<'a, T: ExifValueReader + Copy + Sized> ComponentIterator<'a, T> {
  pub fn new(cursor: Cursor<'a>, len: u32) -> ComponentIterator<'a, T> {
    ComponentIterator {
      value_cursor: cursor,
      len: len,
      i: 0,
      phantom_data: PhantomData
    }
  }
}

impl<'a, T: ExifValueReader + Copy + Sized> Iterator for ComponentIterator<'a, T> {

  type Item = T;

  fn next(&mut self) -> Option<Self::Item> {
    let remaining = self.len - self.i;

    if remaining == 0 {
      return None;
    }

    self.i += 1;

    let value = T::read_exif_value(&mut self.value_cursor);
    //unwrap should never panic since
    //we check the length of the value_cursor
    //in read_exif_tag
    return Some(value.unwrap());
  }

  fn size_hint(&self) -> (usize, Option<usize>) {
    let remaining = (self.len - self.i) as usize;
    (remaining, Some(remaining))
  }
}

