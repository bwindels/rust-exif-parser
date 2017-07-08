use std::mem;
use std::ptr;
use std::str;
use std::cmp;
use ::error::{ParseError, ParseResult};

#[derive(Debug, Clone, Copy)]
pub enum Endianness {
  Little,
  Big,
}

pub trait ByteSwappable : Clone + PartialEq + PartialOrd {
  fn swap_bytes(self) -> Self;
}

impl ByteSwappable for u8 {
  fn swap_bytes(self) -> Self {
    self
  }
}

impl ByteSwappable for i8 {
  fn swap_bytes(self) -> Self {
    self
  }
}

impl ByteSwappable for f32 {
  fn swap_bytes(self) -> Self {
    let mut u : u32 = unsafe { mem::transmute(self) };
    u = u.swap_bytes();
    let f : f32 = unsafe { mem::transmute(u) };
    f
  }
}

impl ByteSwappable for f64 {
  fn swap_bytes(self) -> Self {
    let mut u : u64 = unsafe { mem::transmute(self) };
    u = u.swap_bytes();
    let f : f64 = unsafe { mem::transmute(u) };
    f
  }
}

impl ByteSwappable for u16 {
  fn swap_bytes(self) -> Self {
    self.swap_bytes()
  }
}

impl ByteSwappable for i16 {
  fn swap_bytes(self) -> Self {
    self.swap_bytes()
  }
}

impl ByteSwappable for u32 {
  fn swap_bytes(self) -> Self {
    self.swap_bytes()
  }
}

impl ByteSwappable for i32 {
  fn swap_bytes(self) -> Self {
    self.swap_bytes()
  }
}


impl ByteSwappable for u64 {
  fn swap_bytes(self) -> Self {
    self.swap_bytes()
  }
}

#[cfg(target_endian = "big")]
#[inline(always)]
fn to_le<T: ByteSwappable>(n: T) -> T {
  n.swap_bytes()
}

#[cfg(target_endian = "little")]
#[inline(always)]
fn to_le<T: ByteSwappable>(n: T) -> T {
  n
}

#[cfg(target_endian = "big")]
#[inline(always)]
fn to_be<T: ByteSwappable>(n: T) -> T {
  n
}

#[cfg(target_endian = "little")]
#[inline(always)]
fn to_be<T: ByteSwappable>(n: T) -> T {
  n.swap_bytes()
}

#[derive(Clone, Copy)]
pub struct Cursor<'a> {
  data: &'a [u8],
  endianness: Endianness
}

impl<'a> Cursor<'a> {
  pub fn new(data: &'a [u8], init_endian: Endianness) -> Cursor<'a> {
    Cursor {
      data: data,
      endianness: init_endian
    }
  }

  pub fn offset_from(&self, parent: &Cursor<'a>) -> usize {
    self.data.as_ptr() as usize - parent.data.as_ptr() as usize
  }

  pub fn len(&self) -> usize {
    self.data.len()
  }

  pub fn read_num<T: ByteSwappable>(&mut self) -> Option<T> {
    let size = mem::size_of::<T>();

    if self.data.len() >= size {
      let n : T = unsafe {
        mem::uninitialized()
      };
      //TODO: taking a mut pointer can be done safely IIRC
      let dst : *mut u8 = unsafe {
        mem::transmute(&n)
      };
      let src = self.data.as_ptr();
      unsafe {
        ptr::copy_nonoverlapping(src, dst, size);
      };
      let adjusted_n = match self.endianness {
        Endianness::Little => to_le(n),
        Endianness::Big => to_be(n)
      };

      self.data = &self.data[size .. ];

      Some(adjusted_n)
    }
    else {
      None
    }
  }

  pub fn read_num_or_fail<T: ByteSwappable>(&mut self) -> ParseResult<T> {
    self.read_num().ok_or(ParseError::UnexpectedEOF)
  }

  pub fn with_endianness(&mut self, end: Endianness) -> Cursor<'a> {
    Cursor {
      data: self.data,
      endianness: end
    }
  }

  pub fn endianness(&self) -> Endianness {
    self.endianness
  }

  fn read_bytes_without_advancing(&self, length: usize) -> Option<&'a [u8]> {
    if self.data.len() >= length {
      let byte_slice = &self.data[0 .. length];
      return Some(byte_slice);
    }
    return None
  }

  pub fn read_bytes(&mut self, length: usize) -> Option<&'a [u8]> {
    return self.read_bytes_without_advancing(length).map(|b| {
      self.data = &self.data[length .. ];
      return b;
    });
  }

  pub fn read_bytes_or_fail(&mut self, length: usize) -> ParseResult<&'a [u8]> {
    self.read_bytes(length).ok_or(ParseError::UnexpectedEOF)
  }

  pub fn read_str(&mut self, length: usize) -> Option<&'a str> {
    let bytes = self.read_bytes_without_advancing(length);
    if let Some(slice) = bytes {
      let str_slice = str::from_utf8(slice).ok();
      if str_slice.is_some() {
        self.data = &self.data[length .. ];
      }
      return str_slice;
    } else {
      return None;
    }
  }

  pub fn read_str_or_fail(&mut self, length: usize) -> ParseResult<&'a str> {
    self.read_str(length).ok_or(ParseError::UnexpectedEOF)
  }

  pub fn with_skip_or_fail(&self, offset: usize) -> ParseResult<Cursor<'a>> {
    if self.data.len() >= offset {
      Ok(Cursor {
        data: &self.data[offset ..],
        endianness: self.endianness
      })
    }
    else {
      Err(ParseError::UnexpectedEOF)
    }
  }

  pub fn with_max_len(&self, max_len: usize) -> Cursor<'a> {
    let max_len = cmp::min(self.len(), max_len);
    let truncated_data = &self.data[0 .. max_len];
    Cursor {
      data: truncated_data,
      endianness: self.endianness
    }
  }
}

#[cfg(test)]
mod tests {
  const DATA : &'static [u8] = &[0xDE, 0xAD, 0xCA, 0xFE];

  #[test]
  fn test_len() {
    let stream = ::Cursor::new(&DATA, ::Endianness::Big);
    assert_eq!(stream.len(), 4);
  }

  #[test]
  fn test_read_u8() {
    let mut stream = ::Cursor::new(&DATA, ::Endianness::Big);
    assert_eq!(stream.read_num::<u8>(), Some(0xDE));
    assert_eq!(stream.read_num::<u8>(), Some(0xAD));
    assert_eq!(stream.read_num::<u8>(), Some(0xCA));
    assert_eq!(stream.read_num::<u8>(), Some(0xFE));
    assert_eq!(stream.read_num::<u8>(), None);
  }

  #[test]
  fn test_read_u16_big_endian() {
    let data = &[0xFD, 0xE8];
    let mut stream = ::Cursor::new(data, ::Endianness::Big);
    assert_eq!(stream.read_num::<u16>(), Some(65000));
    assert_eq!(stream.read_num::<u16>(), None);
  }

  #[test]
  fn test_read_u16_little_endian() {
    let data = &[0xE8, 0xFD];
    let mut stream = ::Cursor::new(data, ::Endianness::Little);
    assert_eq!(stream.read_num::<u16>(), Some(65000));
    assert_eq!(stream.read_num::<u16>(), None);
  }

  #[test]
  fn test_read_u16_switch_endianness() {
    let mut stream = ::Cursor::new(&DATA, ::Endianness::Big);
    assert_eq!(stream.read_num::<u16>(), Some(0xDEAD));
    let mut stream = stream.with_endianness(::Endianness::Little);
    assert_eq!(stream.read_num::<u16>(), Some(0xFECA));
  }

  #[test]
  fn test_read_u32_big_endian() {
    let mut stream = ::Cursor::new(&DATA, ::Endianness::Big);
    assert_eq!(stream.read_num::<u32>(), Some(0xDEADCAFE));
    assert_eq!(stream.read_num::<u32>(), None);
  }

  #[test]
  fn test_read_u32_little_endian() {
    let mut stream = ::Cursor::new(&DATA, ::Endianness::Little);
    assert_eq!(stream.read_num::<u32>(), Some(0xFECAADDE));
    assert_eq!(stream.read_num::<u32>(), None);
  }

  #[test]
  fn test_read_f32_little_endian() {
    let data = &[0xD8, 0xF, 0x49, 0x40];
    let mut stream = ::Cursor::new(data, ::Endianness::Little);
    assert_eq!(stream.read_num::<f32>(), Some(3.141592));
    assert_eq!(stream.read_num::<f32>(), None);
  }

  #[test]
  fn test_read_f32_big_endian() {
    let data = &[0x40, 0x49, 0xF, 0xD8];
    let mut stream = ::Cursor::new(data, ::Endianness::Big);
    assert_eq!(stream.read_num::<f32>(), Some(3.141592));
    assert_eq!(stream.read_num::<f32>(), None);
  }

  #[test]
  fn test_read_f64_little_endian() {
    let data = &[0xEA, 0x2E, 0x44, 0x54, 0xFB, 0x21, 0x9, 0x40];
    let mut stream = ::Cursor::new(data, ::Endianness::Little);
    assert_eq!(stream.read_num::<f64>(), Some(3.14159265359));
    assert_eq!(stream.read_num::<f64>(), None);
  }

  #[test]
  fn test_read_f64_big_endian() {
    let data = &[0x40, 0x9, 0x21, 0xFB, 0x54, 0x44, 0x2E, 0xEA];
    let mut stream = ::Cursor::new(data, ::Endianness::Big);
    assert_eq!(stream.read_num::<f64>(), Some(3.14159265359));
    assert_eq!(stream.read_num::<f64>(), None);
  }

  #[test]
  fn test_read_str() {
    let data = &[0x68, 0x65, 0x6C, 0x6C, 0x6F];
    let mut stream = ::Cursor::new(data, ::Endianness::Little);
    assert_eq!(stream.read_str(5), Some("hello"));
    assert_eq!(stream.read_str(1), None);
  }

  #[test]
  fn test_with_max_len() {
    let mut stream = ::Cursor::new(DATA, ::Endianness::Big);
    let mut truncated_stream = stream.with_max_len(2);
    let result = Some(0xDEAD);

    assert_eq!(stream.read_num::<u16>(), result);
    assert_eq!(truncated_stream.read_num::<u16>(), result);

    assert!(stream.read_num::<u16>().is_some());
    assert!(truncated_stream.read_num::<u16>().is_none());
  }

  #[test]
  fn test_with_max_len_check() {
    let stream = ::Cursor::new(DATA, ::Endianness::Big);
    assert_eq!(stream.with_max_len(5).len(), 4);
  }

  #[test]
  fn test_skip() {
    let mut stream = ::Cursor::new(&DATA, ::Endianness::Big);
    stream = stream.with_skip_or_fail(2).expect("EOF");
    assert_eq!(stream.read_num::<u16>(), Some(0xCAFE));
    assert_eq!(stream.read_num::<u16>(), None);
  }

  #[test]
  fn test_read_bytes() {
    let mut cursor = ::Cursor::new(b"\0\x10\x20\x30", ::Endianness::Big);
    assert_eq!(cursor.read_bytes(2), Some(&b"\0\x10"[0 .. 2]));
    assert_eq!(cursor.read_bytes(2), Some(&b"\x20\x30"[0 .. 2]));
    assert_eq!(cursor.read_bytes(1), None);

  }
}
