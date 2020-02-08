/*!
	## Bit-level reading and writing

	`std::io::{Read, Write}` only allow reading and writing on the byte-level. This is not sufficient when working with protocols that use single bits or use structs that are not multiples of 8 bits in size. This crate provides wrappers for reading and writing, enabling bit-level I/O on any object implementing [`Read`]/[`Write`].

	The wrappers are modeled after [`std::io::BufReader`] and [`std::io::BufWriter`], so the semantics and interface should be familiar and robust.

	This crate is a minimal low-level abstraction focusing on bit-level I/O. I recommend using this crate together with [`endio`] if you also need byte-level I/O or (de-)serialization support. However, this crate is completely independent from [`endio`], and can be used standalone if you're only looking for `std::io` with bit support.

	### Goals of this crate

	- Reading and writing of single bits.
	- Reading and writing of bits that aren't a multiple of 8.
	- Reading and writing even if the underlying object is bitshifted.
	- Support for [bit endianness](https://en.wikipedia.org/wiki/Bit_numbering#Most-_vs_least-significant_bit_first) conversion/distinction.

	### Non-goals of this crate

	- Data type (de-)serialization. If you need this, use [`endio`] in combination with this crate.
	- [Byte endianness](https://en.wikipedia.org/wiki/Endianness) conversion/distinction. If you need this, use [`endio`] in combination with this crate.

	### Comparison with other crates

	Bit-level I/O is a common problem, and there are numerous crates on crates.io attempting to provide solutions. However, I haven't been able to find one that is completely satisfactory. Here's a list of related crates and how they differ from this one:

	- `av-bitstream` - Includes (de-)serialization and byte endianness. No bit-level writing. Undocumented.

	- `bit-io` - Does not implement `std::io::{Read, Write}`. No support for bit endianness. Undocumented.

	- `bitio` - No support for writing. No support for bit endianness.

	- `bitstream`, `bitstream_reader`, `bitter` - Include (de-)serialization and byte endianness. No support for writing.

	- `bitstream-io` - Includes (de-)serialization and byte endianness. Includes Huffman trees for some reason. Does not implement `std::io::{Read, Write}`.

	- `bitstream-rs` - Only has support for reading and writing single bits. Does not implement `std::io::{Read, Write}`.

	- `bitreader` - Only has support for reading. Includes deserialization in forced big endian, no little endian support.

	Therefore there is an opportunity to improve on the current library situation, which I hope to address through this crate.

	[`std::io::BufReader`]: https://doc.rust-lang.org/std/io/struct.BufReader.html
	[`std::io::BufWriter`]: https://doc.rust-lang.org/std/io/struct.BufWriter.html
	[`Read`]: https://doc.rust-lang.org/std/io/trait.Read.html
	[`Write`]: https://doc.rust-lang.org/std/io/trait.Write.html
	[`endio`]: https://crates.io/crates/endio
*/
mod endian;
mod read;
mod write;

pub use self::read::*;
pub use self::write::*;
