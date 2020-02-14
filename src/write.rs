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
	Adds bit-level writing support to something implementing [`std::io::Write`].

	This is accomplished through an internal buffer for storing partially read bytes. Note that this buffer is for correctness, not performance - if you want to improve performance by buffering, use [`std::io::BufWriter`] as the `BitWriter`'s write target.

	When the `BitWriter` is dropped, the partially written byte will be written out. However, any errors that happen in the process of flushing the buffer when the writer is dropped will be ignored. Code that wishes to handle such errors must manually call `flush` before the writer is dropped.

	To use this writer, you'll have to choose a bit endianness to write in. The bit endianness determines the direction in which bits in a byte will be written. Note that this is distinct from byte endianness, and e.g. a format which is little endian at the byte level is not necessarily little endian at the bit level.

	If you don't already know which bit endianness you need, chances are you need big endian bit numbering. In that case, just use `endio_bit::BEBitWriter`. Otherwise use `endio_bit::LEBitWriter`.

	[`std::io::Write`]: https://doc.rust-lang.org/std/io/trait.Write.html
	[`std::io::BufWriter`]: https://doc.rust-lang.org/std/io/struct.BufWriter.html
*/
#[derive(Debug)]
pub struct BitWriter<E: BitEndianness, W: Write> {
	/// Data to write to.
	inner: Option<W>,
	/// Offset of remaining bits in a byte, 0 <= bit_offset < 8.
	bit_offset: u8,
	/// Storage for remaining bits after an unaligned write operation.
	bit_buffer: u8,
	buffer: Vec<u8>,
	phantom: std::marker::PhantomData<E>,
}

impl<E: BitEndianness, W: Write> BitWriter<E, W> {
	/**
		Creates a new `BitWriter` from something implementing [`Write`]. This will be used as the underlying object to write to.

		The default capacity for the buffer used in the `Write` implementation is currently 16 bytes, but this may change in the future.

		# Examples

		Create a `BitWriter` writing to bytes in memory:

		```
		use endio_bit::BEBitWriter;

		let mut vec = vec![];
		let mut writer = BEBitWriter::new(vec);
		```

		[`Write`]: https://doc.rust-lang.org/std/io/trait.Write.html
	*/
	pub fn new(inner: W) -> Self {
		Self::with_capacity(16, inner)
	}

	/// Creates a new `BitWriter` with an explicitly specified capacity for the buffer used in the `Write` implementation.
	pub fn with_capacity(capacity: usize, inner: W) -> Self {
		Self {
			inner: Some(inner),
			bit_offset: 0,
			bit_buffer: 0,
			buffer: vec![0; capacity],
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

	/**
		Gets a reference to the underlying writer.

		```compile_fail
		# use endio_bit::BEBitWriter;
		# let mut writer = BEBitWriter::new(vec![]);
		# let inner = writer.get_ref();
		# inner.clear();
		```
	**/
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

	/**
		Writes a single bit, writing 1 for true, 0 for false.

		# Examples

		```
		# use endio_bit::BEBitWriter;
		let mut writer = BEBitWriter::new(vec![]);
		writer.write_bit(true).unwrap();
		let vec = writer.into_inner().unwrap();
		assert_eq!(vec[0], 0x80);
		```

		```
		# use endio_bit::LEBitWriter;
		let mut writer = LEBitWriter::new(vec![]);
		writer.write_bit(true).unwrap();
		let vec = writer.into_inner().unwrap();
		assert_eq!(vec[0], 0x01);
		```
	**/
	pub fn write_bit(&mut self, bit: bool) -> Res<()> {
		if bit {
			self.bit_buffer |= E::shift_lsb(E::shift_msb(0xff, 7), self.bit_offset);
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

		Writing more than 8 bits is intentionally not supported to keep the interface simple. Writing more can be accomplished by writing bytes and then writing any leftover bits.

		# Panics

		Panics if `count` > 8.

		# Examples

		```
		# use endio_bit::BEBitWriter;
		let mut writer = BEBitWriter::new(vec![]);
		writer.write_bits(31, 5);
		let vec = writer.into_inner().unwrap();
		assert_eq!(vec[0], 0xf8);
		```

		```
		# use endio_bit::LEBitWriter;
		let mut writer = LEBitWriter::new(vec![]);
		writer.write_bits(31, 5);
		let vec = writer.into_inner().unwrap();
		assert_eq!(vec[0], 0x1f);
		```
	*/
	pub fn write_bits(&mut self, bits: u8, count: u8) -> Res<()> {
		assert!(count <= 8);
		let start = self.bit_offset;
		let end = start + count;
		let bits = bits << (8 - count);
		let bits = E::align_right(bits, count);
		self.bit_buffer |= E::shift_lsb(bits, start);
		if end >= 8 {
			self.flush_buffer()?;
		}
		if end > 8 {
			self.bit_buffer = E::shift_msb(bits, 8 - start);
		}
		self.bit_offset = end % 8;
		Ok(())
	}
}

/**
	Write bytes to a `BitWriter` just like to [`Write`], but with bit shifting support for unaligned writes.

	Note that in order to fulfill the contract of [`Write`] and write to the underlying object at most once, this function uses a buffer for bitshifting. You can adjust the size of the buffer by creating the `BitWriter` using the `with_capacity` constructor.

	Directly maps to [`Write`] for aligned writes.

	[`Write`]: https://doc.rust-lang.org/std/io/trait.Write.html
*/
impl<E: BitEndianness, W: Write> Write for BitWriter<E, W> {
	fn write(&mut self, buf: &[u8]) -> Res<usize> {
		if self.is_aligned() {
			return unsafe { self.get_mut_unchecked() }.write(buf);
		}
		let mut last_byte = E::shift_lsb(self.bit_buffer, 8 - self.bit_offset);
		for (byte, new) in buf.iter().zip(self.buffer.iter_mut()) {
			*new = E::shift_msb(last_byte, 8 - self.bit_offset)  | E::shift_lsb(*byte, self.bit_offset);
			last_byte = *byte;
		}
		self.bit_buffer = E::shift_msb(last_byte, 8 - self.bit_offset);
		let len = std::cmp::min(buf.len(), self.buffer.len());
		self.inner.as_mut().unwrap().write(&self.buffer[0..len])
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
		let _ = self.align();
	}
}

#[cfg(test)]
mod tests_common {
	use crate::BEBitWriter;

	#[test]
	fn get_ref() {
		let writer = BEBitWriter::new(vec![]);
		let inner = writer.get_ref();
		assert_eq!(inner.len(), 0);
	}

	#[test]
	fn get_mut() {
		let mut writer = BEBitWriter::new(vec![]);
		let inner = writer.get_mut();
		inner.clear();
	}

	#[test]
	#[should_panic]
	fn get_mut_unaligned() {
		let mut writer = BEBitWriter::new(vec![]);
		writer.write_bits(0x0a, 4).unwrap();
		writer.get_mut();
	}

	#[test]
	fn into_inner() {
		let writer = BEBitWriter::new(vec![]);
		let inner = writer.into_inner().unwrap();
		inner.into_boxed_slice();
	}

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
}

#[cfg(test)]
mod tests_be {
	use std::io::Write;
	use crate::BEBitWriter;

	#[test]
	fn write_aligned() {
		let mut vec = vec![];{
		let mut writer = BEBitWriter::new(&mut vec);
		assert_eq!(writer.write(b"Test").unwrap(), 4);}
		assert_eq!(vec, b"Test");
	}

	#[test]
	fn write_shifted() {
		let mut vec = vec![];{
		let mut writer = BEBitWriter::with_capacity(8, &mut vec);
		writer.write_bit(true).unwrap();
		writer.write_bit(false).unwrap();
		writer.write_bit(true).unwrap();
		assert_eq!(writer.write(b"Test").unwrap(), 4);}
		assert_eq!(vec, b"\xaa\x8c\xae\x6e\x80");
	}

	#[test]
	fn flush() {
		let mut writer = BEBitWriter::new(vec![]);
		writer.write_bit(true).unwrap();
		assert_eq!(writer.get_ref(), b"");
		writer.flush().unwrap();
		assert_eq!(writer.get_ref(), b"\x80");
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
		writer.write_bits(0xfa, 4).unwrap();
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
	use std::io::Write;
	use crate::LEBitWriter;

	#[test]
	fn write_aligned() {
		let mut vec = vec![];{
		let mut writer = LEBitWriter::new(&mut vec);
		assert_eq!(writer.write(b"Test").unwrap(), 4);}
		assert_eq!(vec, b"Test");
	}

	#[test]
	fn write_shifted() {
		let mut vec = vec![];{
		let mut writer = LEBitWriter::with_capacity(8, &mut vec);
		writer.write_bit(true).unwrap();
		writer.write_bit(false).unwrap();
		writer.write_bit(true).unwrap();
		assert_eq!(writer.write(b"Test").unwrap(), 4);}
		assert_eq!(vec, b"\xa5\x2a\x9b\xa3\x03");
	}

	#[test]
	fn flush() {
		let mut writer = LEBitWriter::new(vec![]);
		writer.write_bit(true).unwrap();
		assert_eq!(writer.get_ref(), b"");
		writer.flush().unwrap();
		assert_eq!(writer.get_ref(), b"\x01");
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
		writer.write_bits(0xfa, 4).unwrap();
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
