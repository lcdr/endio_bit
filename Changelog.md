
# Changelog

## Version 0.2.0

### Added
- Support for little endian bit endianness was added.

### Changed
- Breaking change: To support different bit endiannesses, `BitReader` and `BitWriter` have been split into `BEBitReader`/`LEBitReader` and `BEBitWriter`/`LEBitWriter`. Use the big endianness variants to keep previous behavior.
- Breaking change: `get_mut_unchecked` is now marked as unsafe, as modifying the underlying object can lead to inconsistent operation when the stream is not byte-aligned.
- The `Read` implementation of `BitReader` has been optimized to avoid frequent read calls to the data source.
