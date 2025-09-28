[![crates.io](https://img.shields.io/crates/v/mkv-element)](https://crates.io/crates/mkv-element)
[![docs.rs](https://img.shields.io/docsrs/mkv-element)](https://docs.rs/mkv-element)
[![MIT License](https://img.shields.io/badge/license-MIT-blue.svg)](./LICENSE)

# mkv-element (WORK IN PROGRESS)
A Rust library for reading and writing Matroska/WebM (MKV) elements.

## Elements
MKV files are made of elements, which can be of different types:
- Master elements: containers for other elements (like folders)
- Leaf elements: contain a single value of a specific type:
    - Unsigned integers
    - Signed integers
    - Floating point numbers
    - Strings (UTF-8/ASCII)
    - Binary data
    - Dates (timestamps in nanoseconds offset to 2001-01-01T00:00:00.000000000 UTC)

See the [Matroska specifications](https://www.matroska.org/technical/elements.html).

## Example
```rust
    // TODO add example here when ready
    println!("it works!");
```

## Highlights
- Efficient in-memory parsing and serialization of MKV elements
- Flexible I/O support for both synchronous and asynchronous operations
- High-performance zero-copy data handling with minimal allocations
- Type-safe leaf elements automatically generated from official Matroska specifications for guaranteed correctness.

## Acknowledgements
Some of the ideas and code snippets were inspired by ~or stolen from~ the following sources, thanks to their authors:
- [mp4-atom](https://github.com/kixelated/mp4-atom) by *kixelated*
- [Network protocols, sans I/O](https://sans-io.readthedocs.io/)

#### License
<sup>
This project is licensed under the MIT License.
See the <a href="LICENSE">LICENSE</a> file for details.
</sup>






