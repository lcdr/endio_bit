/**
	Specifies the bit endianness of a `BitReader` or `BitWriter`.

	You can't implement this trait, it only exists as a trait bound.
*/
pub trait BitEndianness: private::Sealed {
	/// Shifts towards the most significant bit.
	fn shift_msb(val: u8, by: u8) -> u8;
	/// Shifts towards the least significant bit.
	fn shift_lsb(val: u8, by: u8) -> u8;
	/// Aligns right.
	fn align_right(val: u8, count: u8) -> u8;
}

#[derive(Debug)]
pub struct BigEndian;
#[derive(Debug)]
pub struct LittleEndian;

impl BitEndianness for BigEndian {
	fn shift_msb(val: u8, by: u8) -> u8 { val << by }
	fn shift_lsb(val: u8, by: u8) -> u8 { val >> by }
	fn align_right(val: u8, _count: u8) -> u8 { val }
}
impl BitEndianness for LittleEndian {
	fn shift_msb(val: u8, by: u8) -> u8 { val >> by }
	fn shift_lsb(val: u8, by: u8) -> u8 { val << by }
	fn align_right(val: u8, count: u8) -> u8 { Self::shift_msb(val, 8 - count) }
}

pub type BE = BigEndian;
pub type LE = LittleEndian;

// ensures no one else implements the trait
mod private {
	pub trait Sealed {}

	impl Sealed for super::BigEndian {}
	impl Sealed for super::LittleEndian {}
}
