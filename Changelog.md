
# Changelog

## Version 0.2.0

### Added
- Support for little endian bit endianness was added.

### Changed
- Breaking change: To support different bit endiannesses, `BitReader` and `BitWriter` have been split into `BEBitReader`/`LEBitReader` and `BEBitWriter`/`LEBitWriter`. Use the big endianness variants to keep previous behavior.
- The `Read` implementation of `BitReader` has been optimized to avoid frequent read calls to the data source.
