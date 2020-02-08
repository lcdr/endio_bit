/**
	Specifies the bit endianness of a `BitReader` or `BitWriter`.

	You can't implement this trait, it only exists as a trait bound.
*/
pub trait BitEndianness: private::Sealed {}

pub struct BigEndian;
pub struct LittleEndian;

impl BitEndianness for BigEndian {}
impl BitEndianness for LittleEndian {}

pub type BE = BigEndian;
pub type LE = LittleEndian;

// ensures no one else implements the trait
mod private {
	pub trait Sealed {}

	impl Sealed for super::BigEndian {}
	impl Sealed for super::LittleEndian {}
}
