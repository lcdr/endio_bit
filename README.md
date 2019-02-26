
# endio_bit

## Bit-level reading and writing

`std::io::{Read, Write}` only allow reading and writing on the byte-level. This is not sufficient when working with protocols that use single bits or use structs that are not multiples of 8 bits in size. This crate provides wrappers for reading and writing, enabling bit-level I/O on any object implementing `std::io::Read`/`std::io::Write`.

The wrappers are modeled after `std::io::BufReader` and `std::io::BufWriter`, so the semantics and interface should be familiar and robust.

This crate is a minimal low-level abstraction focusing on bit-level I/O. I recommend using this crate together with `endio` for a higher level of abstraction and ergonomics. However, this crate is completely independent from `endio`, and can be used standalone if you're only looking for `std::io` with bit support.

### Goals of this crate

- Reading and writing of single bits.
- Reading and writing of bits that aren't a multiple of 8.
- Reading and writing even if the underlying object is bitshifted.

### Non-goals of this crate

- Any data type (de-)serialization. If you need this, use `endio` in combination with this crate.
- Any endianness conversion/distinction. If you need this, use `endio` in combination with this crate.

### Comparison with other crates

Bit-level I/O is a common problem, and there are numerous crates on crates.io attempting to provide solutions. However, I haven't been able to find one that is completely satisfactory. Here's a list of related crates and how they differ from this one:

- `av-bitstream` - Includes (de-)serialization and endianness. The entire crate is undocumented.

- `bit-io` - Does not implement `std::io::{Read, Write}` on its abstractions, thus not being suitable for code requiring these traits.

- `bitio` - Only has support for reading.

- `bitstream`, `bitstream_reader`, `bitter` - Only have support for reading. Include deserialization and endianness instead of being a low-level abstraction.

- `bitstream-io` - Includes (de-)serialization and endianness. Includes Huffman trees for some reason. Does not implement `std::io::{Read, Write}` on its abstractions.

- `bitstream-rs` - Only has support for reading and writing single bits. Does not implement `std::io::{Read, Write}` on its abstractions.

- `bitreader` - Only has support for reading. Includes deserialization in forced big endian, no little endian support.

Therefore there is an opportunity to improve on the current library situation, which I hope to address through this crate.
