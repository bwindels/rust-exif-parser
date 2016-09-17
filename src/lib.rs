struct BufferStream<'a> {
	data: &'a [u8],
	offset: usize
}

impl<'a> BufferStream<'a> {
	
	pub fn new(data: &'a [u8]) -> BufferStream<'a> {
		BufferStream {
			data: data,
			offset: 0
		}
	}

	pub fn len(&self) -> usize {
		self.data.len() - self.offset
	}

	pub fn read_u8(&mut self) -> Option<u8> {
		if self.len() >= 1 {
			let result = Some(self.data[self.offset]);
			self.offset += 1;
			result
		}
		else {
			None
		}
	}
}

#[cfg(test)]
mod tests {
	const DATA : &'static [u8] = &[0xDEu8, 0xADu8, 0xCAu8, 0xFEu8];

	#[test]
	fn test_len() {
		let stream = ::BufferStream::new(&DATA);
		assert_eq!(stream.len(), 4);
	}

	#[test]
	fn test_read_u8() {
		let mut stream = ::BufferStream::new(&DATA);
		assert_eq!(stream.read_u8(), Some(0xDEu8));
		assert_eq!(stream.read_u8(), Some(0xADu8));
		assert_eq!(stream.read_u8(), Some(0xCAu8));
		assert_eq!(stream.read_u8(), Some(0xFEu8));
		assert_eq!(stream.read_u8(), None);
		
	}
}