use std::mem;
use std::ptr;

#[derive(Debug)]
enum Endianness {
    Little,
    Big,
}

trait Number : Clone {
	fn swap_bytes(self) -> Self;
}

impl Number for u8 {
	fn swap_bytes(self) -> Self {
		self
	}
}

impl Number for u16 {
	fn swap_bytes(self) -> Self {
		self.swap_bytes()
	}
}

impl Number for u32 {
	fn swap_bytes(self) -> Self {
		self.swap_bytes()
	}
}

impl Number for u64 {
	fn swap_bytes(self) -> Self {
		self.swap_bytes()
	}
}

#[cfg(target_endian = "big")]
fn to_le<T: Number>(n: T) -> T {
	n.swap_bytes()
}

#[cfg(target_endian = "little")]
fn to_le<T: Number>(n: T) -> T {
	n
}

#[cfg(target_endian = "big")]
fn to_be<T: Number>(n: T) -> T {
	n
}

#[cfg(target_endian = "little")]
fn to_be<T: Number>(n: T) -> T {
	n.swap_bytes()
}

struct BufferStream<'a> {
	data: &'a [u8],
	offset: usize,
	endianness: Endianness
}

impl<'a> BufferStream<'a> {
	
	pub fn new(data: &'a [u8], init_endian: Endianness) -> BufferStream<'a> {
		BufferStream {
			data: data,
			offset: 0,
			endianness: init_endian
		}
	}

	pub fn len(&self) -> usize {
		self.data.len() - self.offset
	}

	pub fn read_num<T: Number>(&mut self) -> Option<T> {
		
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
}

#[cfg(test)]
mod tests {
	const DATA : &'static [u8] = &[0xDE, 0xAD, 0xCA, 0xFE];
	const ENDIANNESS : ::Endianness = ::Endianness::Big;
	#[test]
	fn test_len() {
		let stream = ::BufferStream::new(&DATA, ENDIANNESS);
		assert_eq!(stream.len(), 4);
	}

	#[test]
	fn test_read_u8() {
		let mut stream = ::BufferStream::new(&DATA, ENDIANNESS);
		assert_eq!(stream.read_num::<u8>(), Some(0xDE));
		assert_eq!(stream.read_num::<u8>(), Some(0xAD));
		assert_eq!(stream.read_num::<u8>(), Some(0xCA));
		assert_eq!(stream.read_num::<u8>(), Some(0xFE));
		assert_eq!(stream.read_num::<u8>(), None);
	}

	#[test]
	fn test_read_u16() {
		let mut stream = ::BufferStream::new(&DATA, ENDIANNESS);
		assert_eq!(stream.read_num::<u16>(), Some(0xDEAD));
		assert_eq!(stream.read_num::<u16>(), Some(0xCAFE));
		assert_eq!(stream.read_num::<u16>(), None);
	}

	#[test]
	fn test_read_u32() {
		let mut stream = ::BufferStream::new(&DATA, ENDIANNESS);
		assert_eq!(stream.read_num::<u32>(), Some(0xDEADCAFE));
		assert_eq!(stream.read_num::<u32>(), None);
	}

}