struct BufferStream<'a> {
	data: &'a [u8]
}

impl<'a> BufferStream<'a> {
	
	pub fn new(data: &'a [u8]) -> BufferStream<'a> {
		BufferStream {
			data: data
		}
	}

	pub fn len(&self) -> usize {
		self.data.len()
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
	
}