use std::io::Result as Res;
use std::io::Write;

use crate::endian::{BitEndianness, BE, LE};

/// Writes most significant bits first.
pub type BEBitWriter<W> = BitWriter<BE, W>;
/// Writes least significant bits first.
pub type LEBitWriter<W> = BitWriter<LE, W>;

/**
	An error returned by `BitWriter::into_inner`.

	This is a clone of [`std::io::IntoInnerError`]. The semantics and API are the exact same. Ideally I'd use `std::io::IntoInnerError` directly, but its constructor is not public.

	See [`std::io::IntoInnerError`] for documentation.

	[`std::io::IntoInnerError`]: https://doc.rust-lang.org/std/io/struct.IntoInnerError.html
**/
#[derive(Debug)]
pub struct IntoInnerError<W>(W, std::io::Error);

impl<W> IntoInnerError<W> {
	pub fn error(&self) -> &std::io::Error { &self.1 }
	pub fn into_inner(self) -> W { self.0 }
}

/**
	Adds bit-level writing support to something implementing `std::io::Write`.

	This is accomplished through an internal buffer for storing partially read bytes. Note that this buffer is for correctness, not performance - if you want to improve performance by buffering, use [`std::io::BufWriter`] as the `BitWriter`'s write target.

	When the `BitWriter` is dropped, the partially written byte will be written out. However, any errors that happen in the process of flushing the buffer when the writer is dropped will be ignored. Code that wishes to handle such errors must manually call `flush` before the writer is dropped.
*/
#[derive(Debug)]
pub struct BitWriter<E: BitEndianness, W: Write> {
	/// Data to write to.
	inner: Option<W>,
	/// Offset of remaining bits in a byte, 0 <= bit_offset < 8.
	bit_offset: u8,
	/// Storage for remaining bits after an unaligned write operation.
	bit_buffer: u8,
	phantom: std::marker::PhantomData<E>,
}

impl<E: BitEndianness, W: Write> BitWriter<E, W> {
	/**
		Creates a new `BitWriter` from something implementing `Write`. This will be used as the underlying object to write to.

		# Examples

		Create a `BitWriter` writing to bytes in memory:

		```
		use endio_bit::BEBitWriter;

		let mut vec = vec![];
		let mut writer = BEBitWriter::new(vec);
		```
	*/
	pub fn new(inner: W) -> Self {
		Self {
			inner: Some(inner),
			bit_offset: 0,
			bit_buffer: 0,
			phantom: std::marker::PhantomData,
		}
	}

	/// Returns whether the writer is aligned to the byte boundary.
	#[inline(always)]
	pub fn is_aligned(&self) -> bool {
		self.bit_offset == 0
	}

	/// Aligns to byte boundary, skipping a partial byte if the `BitWriter` was not aligned.
	pub fn align(&mut self) -> Res<()> {
		if !self.is_aligned() {
			self.flush_buffer()?;
			self.bit_offset = 0;
		}
		Ok(())
	}

	/// Gets a reference to the underlying writer.
	pub fn get_ref(&self) -> &W {
		self.inner.as_ref().unwrap()
	}

	/**
		Gets a mutable reference to the underlying writer.

		Mutable operations on the underlying writer will corrupt this `BitWriter` if it is not aligned, so the reference is only returned if the `BitWriter` is aligned.

		Panics if the `BitWriter` is not aligned.
	*/
	pub fn get_mut(&mut self) -> &mut W {
		if !self.is_aligned() {
			panic!("BitWriter is not aligned");
		}
		self.inner.as_mut().unwrap()
	}

	/**
		Gets a mutable reference to the underlying writer.

		Use with care: Any writing/seeking/etc operation on the underlying writer will corrupt this `BitWriter` if it is not aligned.
	*/
	pub unsafe fn get_mut_unchecked(&mut self) -> &mut W {
		self.inner.as_mut().unwrap()
	}

	/**
		Unwraps this `BitWriter`, returning the underlying writer.

		The buffer for partial writes will be flushed before returning the writer. If an error occurs during the flushing it will be returned.
	*/
	pub fn into_inner(mut self) -> Result<W, IntoInnerError<Self>> {
		match self.align() {
			Ok(()) => Ok(self.inner.take().unwrap()),
			Err(e) => Err(IntoInnerError(self, e)),
		}
	}

	fn flush_buffer(&mut self) -> Res<()> {
		let mut temp = [0; 1];
		temp[0] = self.bit_buffer;
		unsafe { self.get_mut_unchecked() }.write(&temp)?;
		self.bit_buffer = 0;
		Ok(())
	}
}

impl<W: Write> BitWriter<BE, W> {
	/// Writes a single bit, writing 1 for true, 0 for false.
	pub fn write_bit(&mut self, bit: bool) -> Res<()> {
		if bit {
			self.bit_buffer |= 0x80 >> self.bit_offset;
		}
		self.bit_offset = (self.bit_offset + 1) % 8;
		if self.is_aligned() {
			self.flush_buffer()?;
		}
		Ok(())
	}

	/**
		Writes 8 bits or less.

		The lowest `count` bits will be used, others will be ignored.

		Writing more than 8 bits is intentionally not supported to keep the interface simple and to avoid having to deal with endianness in any way. Writing more can be accomplished by writing bytes and then writing any leftover bits.

		# Panics

		Panics if `count` > 8.

		# Examples

		```
		use endio_bit::BEBitWriter;

		let mut vec = vec![];
		let mut writer = BEBitWriter::new(vec);
		writer.write_bits(31, 5);
		let vec = writer.into_inner().unwrap();
		//assert_eq!(vec[0], 0xf8);
		```
	*/
	pub fn write_bits(&mut self, bits: u8, count: u8) -> Res<()> {
		assert!(count <= 8);
		let bits = bits << (8 - count);
		self.bit_buffer |= bits >> self.bit_offset;
		if self.bit_offset + count >= 8 {
			self.flush_buffer()?;
		}
		if self.bit_offset + count > 8 {
			self.bit_buffer = bits << (8 - self.bit_offset);
		}
		self.bit_offset = (self.bit_offset + count) % 8;
		Ok(())
	}
}

impl<W: Write> BitWriter<LE, W> {
	/// Writes a single bit, writing 1 for true, 0 for false.
	pub fn write_bit(&mut self, bit: bool) -> Res<()> {
		if bit {
			self.bit_buffer |= 0x01 << self.bit_offset;
		}
		self.bit_offset = (self.bit_offset + 1) % 8;
		if self.is_aligned() {
			self.flush_buffer()?;
		}
		Ok(())
	}

	/**
		Writes 8 bits or less.

		The lowest `count` bits will be used, others will be ignored.

		Writing more than 8 bits is intentionally not supported to keep the interface simple and to avoid having to deal with endianness in any way. Writing more can be accomplished by writing bytes and then writing any leftover bits.

		# Panics

		Panics if `count` > 8.

		# Examples

		```
		use endio_bit::BEBitWriter;

		let mut vec = vec![];
		let mut writer = BEBitWriter::new(vec);
		writer.write_bits(31, 5);
		let vec = writer.into_inner().unwrap();
		//assert_eq!(vec[0], 0xf8);
		```
	*/
	pub fn write_bits(&mut self, bits: u8, count: u8) -> Res<()> {
		assert!(count <= 8);
		self.bit_buffer |= bits << self.bit_offset;
		if self.bit_offset + count >= 8 {
			self.flush_buffer()?;
		}
		if self.bit_offset + count > 8 {
			self.bit_buffer = bits >> (8 - self.bit_offset);
		}
		self.bit_offset = (self.bit_offset + count) % 8;
		Ok(())
	}
}

/**
	Write bytes to a `BitWriter` just like to `Write`, but with bit shifting support for unaligned writes.

	Note that in order to fulfill the contract of `Write` and write to the underlying object at most once, this function needs to heap-allocate a new buffer for bitshifting, which may be expensive for large buffers.

	Directly maps to `Write` for aligned writes.
*/
impl<W: Write> Write for BitWriter<BE, W> {
	fn write(&mut self, buf: &[u8]) -> Res<usize> {
		if self.is_aligned() {
			return unsafe { self.get_mut_unchecked() }.write(buf);
		}
		let mut shifted = vec![0; buf.len()];
		let mut last_byte = self.bit_buffer >> (8 - self.bit_offset);
		for (byte, new) in buf.iter().zip(shifted.iter_mut()) {
			*new = last_byte << (8 - self.bit_offset)  | *byte >> self.bit_offset;
			last_byte = *byte;
		}
		self.bit_buffer = last_byte << (8 - self.bit_offset);
		unsafe { self.get_mut_unchecked() }.write(&shifted)
	}

	fn flush(&mut self) -> Res<()> {
		if !self.is_aligned() {
			self.flush_buffer()?;
		}
		unsafe { self.get_mut_unchecked() }.flush()
	}
}

/**
	Write bytes to a `BitWriter` just like to `Write`, but with bit shifting support for unaligned writes.

	Note that in order to fulfill the contract of `Write` and write to the underlying object at most once, this function needs to heap-allocate a new buffer for bitshifting, which may be expensive for large buffers.

	Directly maps to `Write` for aligned writes.
*/
impl<W: Write> Write for BitWriter<LE, W> {
	fn write(&mut self, buf: &[u8]) -> Res<usize> {
		if self.is_aligned() {
			return unsafe { self.get_mut_unchecked() }.write(buf);
		}
		let mut shifted = vec![0; buf.len()];
		let mut last_byte = self.bit_buffer << (8 - self.bit_offset);
		for (byte, new) in buf.iter().zip(shifted.iter_mut()) {
			*new = last_byte >> (8 - self.bit_offset)  | *byte << self.bit_offset;
			last_byte = *byte;
		}
		self.bit_buffer = last_byte >> (8 - self.bit_offset);
		unsafe { self.get_mut_unchecked() }.write(&shifted)
	}

	fn flush(&mut self) -> Res<()> {
		if !self.is_aligned() {
			self.flush_buffer()?;
		}
		unsafe { self.get_mut_unchecked() }.flush()
	}
}

/// Flushes the buffer for unaligned writes before the `BitWriter` is dropped.
impl<E: BitEndianness, W: Write> Drop for BitWriter<E, W> {
	fn drop(&mut self) {
		self.align().unwrap();
	}
}

#[cfg(test)]
mod tests_common {
	use crate::BEBitWriter;

	#[test]
	fn align() {
		let mut vec = vec![];{
		let mut writer = BEBitWriter::new(&mut vec);
		writer.write_bits(31, 5).unwrap();
		assert_eq!(writer.is_aligned(), false);
		writer.align().unwrap();
		assert_eq!(writer.is_aligned(), true);
		writer.write_bit(true).unwrap();}
		assert_eq!(vec, b"\xf8\x80");
	}

	#[test]
	#[should_panic]
	fn get_mut_unaligned() {
		let mut writer = BEBitWriter::new(vec![]);
		writer.write_bits(0x0a, 4).unwrap();
		writer.get_mut();
	}
}

#[cfg(test)]
mod tests_be {
	use crate::BEBitWriter;

	#[test]
	fn write_shifted() {
		use std::io::Write;
		let mut vec = vec![];{
		let mut writer = BEBitWriter::new(&mut vec);
		writer.write_bit(true).unwrap();
		writer.write_bit(false).unwrap();
		writer.write_bit(true).unwrap();
		assert_eq!(writer.write(b"Test").unwrap(), 4);}
		assert_eq!(vec, b"\xaa\x8c\xae\x6e\x80");
	}

	#[test]
	fn write_bit() {
		let mut vec = vec![];{
		let mut writer = BEBitWriter::new(&mut vec);
		writer.write_bit(false).unwrap();
		writer.write_bit(false).unwrap();
		writer.write_bit(true).unwrap();
		writer.write_bit(false).unwrap();
		writer.write_bit(true).unwrap();
		writer.write_bit(false).unwrap();
		writer.write_bit(true).unwrap();
		writer.write_bit(false).unwrap();}
		assert_eq!(vec, b"\x2a");
	}

	#[test]
	fn write_bits() {
		let mut vec = vec![];{
		let mut writer = BEBitWriter::new(&mut vec);
		writer.write_bits(0x0a, 4).unwrap();
		writer.write_bits(0xbc, 8).unwrap();}
		assert_eq!(vec, b"\xab\xc0");
	}

	#[test]
	#[should_panic]
	fn write_too_many_bits() {
		let mut vec = vec![];
		let mut writer = BEBitWriter::new(&mut vec);
		writer.write_bits(0xff, 9).unwrap();
	}
}

#[cfg(test)]
mod tests_le {
	use crate::LEBitWriter;

	#[test]
	fn write_shifted() {
		use std::io::Write;
		let mut vec = vec![];{
		let mut writer = LEBitWriter::new(&mut vec);
		writer.write_bit(true).unwrap();
		writer.write_bit(false).unwrap();
		writer.write_bit(true).unwrap();
		assert_eq!(writer.write(b"Test").unwrap(), 4);}
		assert_eq!(vec, b"\xa5\x2a\x9b\xa3\x03");
	}

	#[test]
	fn write_bit() {
		let mut vec = vec![];{
		let mut writer = LEBitWriter::new(&mut vec);
		writer.write_bit(false).unwrap();
		writer.write_bit(false).unwrap();
		writer.write_bit(true).unwrap();
		writer.write_bit(false).unwrap();
		writer.write_bit(true).unwrap();
		writer.write_bit(false).unwrap();
		writer.write_bit(true).unwrap();
		writer.write_bit(false).unwrap();}
		assert_eq!(vec, b"\x54");
	}

	#[test]
	fn write_bits() {
		let mut vec = vec![];{
		let mut writer = LEBitWriter::new(&mut vec);
		writer.write_bits(0x0a, 4).unwrap();
		writer.write_bits(0xbc, 8).unwrap();}
		assert_eq!(vec, b"\xca\x0b");
	}

	#[test]
	#[should_panic]
	fn write_too_many_bits() {
		let mut vec = vec![];
		let mut writer = LEBitWriter::new(&mut vec);
		writer.write_bits(0xff, 9).unwrap();
	}
}
