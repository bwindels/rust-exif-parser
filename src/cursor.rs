use std::mem;
use std::ptr;
use std::str;

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

impl ByteSwappable for u32 {
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

pub struct Cursor<'a> {
    data: &'a [u8],
    offset: usize,
    endianness: Endianness
}

impl<'a> Cursor<'a> {
    pub fn new(data: &'a [u8], init_endian: Endianness) -> Cursor<'a> {
        Cursor {
            data: data,
            offset: 0,
            endianness: init_endian
        }
    }

    pub fn len(&self) -> usize {
        self.data.len() - self.offset
    }

    pub fn read_num<T: ByteSwappable>(&mut self) -> Option<T> {
        let size = mem::size_of::<T>();

        if self.len() >= size {
            let n : T = unsafe {
                mem::uninitialized()
            };
            let dst : *mut u8 = unsafe {
                mem::transmute(&n)
            };
            let src = unsafe {
                self.data.as_ptr().offset(self.offset as isize)
            };
            unsafe {
                ptr::copy_nonoverlapping(src, dst, size);
            };
            let adjusted_n = match self.endianness {
                Endianness::Little => to_le(n),
                Endianness::Big => to_be(n)
            };

            self.offset += size;

            Some(adjusted_n)
        }
        else {
            None
        }
    }

    pub fn set_endianness(&mut self, end: Endianness) {
        self.endianness = end;
    }

    fn read_bytes_without_advancing(&mut self, length: usize) -> Option<&'a [u8]> {
        if self.len() >= length {
            let end_index = self.offset + length;
            let byte_slice = &self.data[self.offset .. end_index];
            return Some(byte_slice);
        }
        return None
    }

    pub fn read_bytes(&mut self, length: usize) -> Option<&'a [u8]> {
    	return self.read_bytes_without_advancing(length).map(|b| {
    	   self.offset = self.offset + length;
    	   return b;
    	});
    }

    pub fn read_str(&mut self, length: usize) -> Option<&'a str> {
        let bytes = self.read_bytes_without_advancing(length);
        if let Some(slice) = bytes {
		    let str_slice = str::from_utf8(slice).ok();
		    if str_slice.is_some() {
			    self.offset = self.offset + length;
		    }
		    return str_slice;
        } else {
        	return None;
        }
    }

    pub fn branch(&self, length: usize) -> Option<Cursor<'a>> {
        if self.len() >= length {
            let data_subset = &self.data[self.offset .. self.offset + length];
            Some(Cursor::new(data_subset, self.endianness))
        } else {
            None
        }
    }
    
    pub fn skip(&self, length: usize) -> Option<Cursor<'a>> {
    	if self.len() >= length {
    		let new_offset = self.offset + length;
    		let slice = &self.data[new_offset ..];
    		Some(Cursor::new(slice, self.endianness))
    	}
    	else {
    		None
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
        stream.set_endianness(::Endianness::Little);
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
    fn test_branch() {
        let mut stream = ::Cursor::new(DATA, ::Endianness::Big);
        let mut branched_stream = stream.branch(2).unwrap();
        let result = Some(0xDEAD);

        assert_eq!(stream.read_num::<u16>(), result);
        assert_eq!(branched_stream.read_num::<u16>(), result);

        assert!(stream.read_num::<u16>().is_some());
        assert!(branched_stream.read_num::<u16>().is_none());
    }

    #[test]
    fn test_branch_length_check() {
        let stream = ::Cursor::new(DATA, ::Endianness::Big);

        assert!(stream.branch(4).is_some());
        assert!(stream.branch(5).is_none());
    }

    #[test]
    fn test_skip() {
        let mut stream = ::Cursor::new(&DATA, ::Endianness::Big);
        stream = stream.skip(2).unwrap();
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
