use std::io::Result as Res;
use std::io::Read;

/**
	Adds bit-level reading support to something implementing `std::io::Read`.

	This is accomplished through an internal buffer for storing partially read bytes.
*/
pub struct BitReader<R> {
	/// Data to read from.
	inner: R,
	/// Offset of remaining bits in a byte, 0 <= bit_offset < 8.
	bit_offset: u8,
	/// Storage for remaining bits after an unaligned read operation.
	bit_buffer: u8,
}

impl<R: Read> BitReader<R> {
	/**
		Creates a new `BitReader` from something implementing `Read`. This will be used as the underlying object to read from.

		# Examples

		Create a `BitReader` reading from bytes in memory:

		```
		use endio_bit::BitReader;

		let data = b"\xcf\xfe\xf3\x2c";
		let data_reader = &data[..];
		let mut reader = BitReader::new(data_reader);
		```
	*/
	pub fn new(inner: R) -> BitReader<R> {
		BitReader {
			inner,
			bit_offset: 0,
			bit_buffer: 0,
		}
	}

	/// Reads a single bit, returning true for 1, false for 0.
	pub fn read_bit(&mut self) -> Res<bool> {
		if self.bit_offset == 0 {
			self.fill_buffer()?;
		}
		let val = self.bit_buffer & (0x80 >> self.bit_offset) != 0;
		self.bit_offset = if self.bit_offset == 7 { 0 } else { self.bit_offset + 1 };
		Ok(val)
	}

	/**
		Reads 8 bits or less.

		The lowest `count` bits will be filled by this, the others will be zero.

		Reading more than 8 bits is intentionally not supported to keep the interface simple and to avoid having to deal with endianness in any way. Reading more can be accomplished by reading bytes and then reading any leftover bits.

		# Panics

		Panics if `count` > 8.

		# Examples

		```
		use endio_bit::BitReader;

		let data = &b"\xf8"[..];
		let mut reader = BitReader::new(data);

		let value = reader.read_bits(5).unwrap();
		assert_eq!(value, 31);
		```
	*/
	pub fn read_bits(&mut self, count: u8) -> Res<u8> {
		assert!(count <= 8);
		if self.bit_offset == 0 {
			self.fill_buffer()?;
		}
		let mut res = self.bit_buffer << self.bit_offset;
		if count > 8 - self.bit_offset {
			self.fill_buffer()?;
			res |= self.bit_buffer >> (8 - self.bit_offset);
		}
		res >>= 8 - count;
		self.bit_offset = (self.bit_offset + count) % 8;
		Ok(res)
	}

	/// Returns whether the reader is aligned to the byte boundary.
	pub fn is_aligned(&self) -> bool {
		self.bit_offset == 0
	}

	/// Aligns to byte boundary, discarding a partial byte if the `BitReader` was not aligned.
	pub fn align(&mut self) {
		self.bit_offset = 0;
		self.bit_buffer = 0;
	}

	/// Gets a reference to the underlying reader.
	pub fn get_ref(&self) -> &R {
		&self.inner
	}

	/**
		Gets a mutable reference to the underlying reader.

		Mutable operations on the underlying reader will corrupt this `BitReader` if it is not aligned, so the reference is only returned if the `BitReader` is aligned.

		Panics if the `BitReader` is not aligned.
	*/
	pub fn get_mut(&mut self) -> &mut R {
		if !self.is_aligned() {
			panic!("BitReader is not aligned");
		}
		&mut self.inner
	}

	/**
		Gets a mutable reference to the underlying reader.

		Use with care: Any reading/seeking/etc operation on the underlying reader will corrupt this `BitReader` if it is not aligned.
	*/
	pub fn get_mut_unchecked(&mut self) -> &mut R {
		&mut self.inner
	}

	/**
		Unwraps this `BitReader`, returning the underlying reader.

		Note that any partially read byte is lost.
	*/
	pub fn into_inner(self) -> R {
		self.inner
	}

	fn fill_buffer(&mut self) -> Res<()> {
		let mut temp = [0; 1];
		self.inner.read_exact(&mut temp)?;
		self.bit_buffer = temp[0];
		Ok(())
	}
}


/**
	Read bytes from a `BitReader` just like from `Read`, but with bit shifting support for unaligned reads.

	Directly maps to `Read` for aligned reads.
*/
impl<R: Read> Read for BitReader<R> {
	fn read(&mut self, buf: &mut [u8]) -> Res<usize> {
		if self.bit_offset == 0 {
			return self.inner.read(buf);
		}

		let first;
		let rest;
		match buf.split_first_mut() {
			Some(x) => {
				first = x.0;
				rest = x.1
			}
			None => { return Ok(0); }
		}
		let mut count_read = 0;

		*first = self.bit_buffer << self.bit_offset;
		let mut temp = [0; 1];
		count_read += self.inner.read(&mut temp)?;
		*first |= temp[0] >> (8 - self.bit_offset);
		for b in rest {
			*b = temp[0] << self.bit_offset;
			count_read += self.inner.read(&mut temp)?;
			*b |= temp[0] >> (8 - self.bit_offset);
		}
		self.bit_buffer = temp[0];
		Ok(count_read)
	}
}

#[cfg(test)]
mod tests {
	use crate::BitReader;

	#[test]
	fn read_shifted() {
		use std::io::Read;
		let data = &b"\xaa\x8c\xaen\x80"[..];
		let mut reader = BitReader::new(data);
		let mut val: bool;
		val = reader.read_bit().unwrap();
		assert_eq!(val, true);
		val = reader.read_bit().unwrap();
		assert_eq!(val, false);
		val = reader.read_bit().unwrap();
		assert_eq!(val, true);
		let mut buf = [0; 4];
		assert_eq!(reader.read(&mut buf).unwrap(), 4);
		assert_eq!(&buf, b"Test");
	}

	#[test]
	fn read_bit() {
		let b = &b"\x80"[..];
		let mut reader = BitReader::new(b);
		let bit: bool = reader.read_bit().unwrap();
		assert_eq!(bit, true);
	}

	#[test]
	fn read_bit_multiple() {
		let b = &b"\x2a"[..];
		let mut reader = BitReader::new(b);
		let mut bit: bool;
		bit = reader.read_bit().unwrap();
		assert_eq!(bit, false);
		bit = reader.read_bit().unwrap();
		assert_eq!(bit, false);
		bit = reader.read_bit().unwrap();
		assert_eq!(bit, true);
		bit = reader.read_bit().unwrap();
		assert_eq!(bit, false);
		bit = reader.read_bit().unwrap();
		assert_eq!(bit, true);
		bit = reader.read_bit().unwrap();
		assert_eq!(bit, false);
		bit = reader.read_bit().unwrap();
		assert_eq!(bit, true);
		bit = reader.read_bit().unwrap();
		assert_eq!(bit, false);
	}

	#[test]
	fn read_bits() {
		let data = &b"\xa5"[..];
		let mut reader = BitReader::new(data);
		let bits_1 = reader.read_bits(4).unwrap();
		let bits_2 = reader.read_bits(4).unwrap();
		assert_eq!(bits_1, 0x0a);
		assert_eq!(bits_2, 0x05);
	}

	#[test]
	fn read_max_bits() {
		let data = &b"\xff\xa5"[..];
		let mut reader = BitReader::new(data);
		let bits = reader.read_bits(8).unwrap();
		assert_eq!(bits, 0xff);
	}
	#[test]
	#[should_panic]
	fn read_too_many_bits() {
		let data = &b"\x2a\xa5"[..];
		let mut reader = BitReader::new(data);
		reader.read_bits(9).unwrap();
	}

	#[test]
	fn align() {
		let data = &b"\xf8\x80"[..];
		let mut reader = BitReader::new(data);
		let bits = reader.read_bits(5).unwrap();
		assert_eq!(reader.is_aligned(), false);
		reader.align();
		assert_eq!(reader.is_aligned(), true);
		let bit: bool = reader.read_bit().unwrap();
		assert_eq!(bits, 31);
		assert_eq!(bit, true);
	}

	#[test]
	#[should_panic]
	fn get_mut_unaligned() {
		let data = &b"\xff"[..];
		let mut reader = BitReader::new(data);
		reader.read_bits(4).unwrap();
		reader.get_mut();
	}
}
