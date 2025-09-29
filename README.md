[![crates.io](https://img.shields.io/crates/v/mkv-element)](https://crates.io/crates/mkv-element)
[![docs.rs](https://img.shields.io/docsrs/mkv-element)](https://docs.rs/mkv-element)
[![MIT License](https://img.shields.io/badge/license-MIT-blue.svg)](./LICENSE)

# mkv-element
A Rust library for reading and writing Matroska/WebM (MKV) elements.

## Highlights
- *Simple* API to work with
- Efficient in-memory parsing and serialization of MKV elements
- Flexible I/O support for both synchronous and asynchronous operations
- High-performance zero-copy data handling with minimal allocations

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

## Matroska/Webm Container aka. EBML

Mkv/WebM files start with an EBML header, followed by one or more segments containing the actual media data and metadata.
Roughly, the structure looks like:

``` text
┌────────────────── MKV Structure ─────────────────┐
│ ┌────────────── EBML ──────────────┐             │
│ │ Header (Version, ReadVersion)    │             │
│ └──────────────────────────────────┘             │
│ ┌────────────── Segment(s) ────────┐             │
│ │ ┌──────────── Info ──────────┐   │             │
│ │ │ Metadata (Duration, Title) │   │             │
│ │ └────────────────────────────┘   │             │
│ │ ┌──────────── Tracks ────────┐   │             │
│ │ │ Audio/Video Tracks         │   │             │
│ │ └────────────────────────────┘   │             │
│ │ ┌──────────── SeekHead ──────┐   │             │
│ │ │ Index for Seeking          │   │             │
│ │ └────────────────────────────┘   │             │
│ │ ┌──────────── Cluster(s) ────┐   │             │
│ │ │ Media Data (Frames)        │   │             │
│ │ └────────────────────────────┘   │             │
│ │ ┌──────────── Others ────────┐   │             │
│ │ │ Cues, Chapters, Tags...    │   │             │
│ │ └────────────────────────────┘   │             │
│ └──────────────────────────────────┘             │
└──────────────────────────────────────────────────┘
```

### Blocking I/O

1. Reading elements from types implementing `std::io::Read` .
2. Writing elements to types implementing `std::io::Write`.

```rust
use mkv_element::prelude::*; // prelude brings all the types into scope
use mkv_element::io::blocking_impl::*; // use blocking_impl for blocking I/O

/// Create a simple EBML header element
let ebml = Ebml {
    crc32: None,
    ebml_version: None,
    ebml_read_version: None,
    ebml_max_id_length: EbmlMaxIdLength(4),
    ebml_max_size_length: EbmlMaxSizeLength(8),
    doc_type: Some(DocType("matroska".to_string())),
    doc_type_version: Some(DocTypeVersion(1)),
    doc_type_read_version: Some(DocTypeReadVersion(1)),
    void: None,
};

// Write the EBML element to a type implementing std::io::Write

// 1. to a Vec<u8>
let mut buffer = Vec::new();
ebml.write_to(&mut buffer).unwrap();

// 2. to a file
let mut file = std::io::sink(); // replace with actual file, ie. std::fs::File::create("path/to/file.mkv").unwrap();
ebml.write_to(&mut file).unwrap();


// Reading a element can be done from either using the element's `read_from()` method
// or reading out the header first followed by a `read_element()`.
// the latter is useful when you don't know the element type in advance.

// 1. using `read_from()`
let mut buf_cursor = std::io::Cursor::new(&buffer);
let ebml_read_1 = Ebml::read_from(&mut buf_cursor).unwrap();
// or directly from a slice
let ebml_read_2 = Ebml::read_from(&mut &buffer[..]).unwrap();
// or from a file
// let mut file = std::fs::File::open("path/to/file.mkv").unwrap();
// let ebml_read_3 = Ebml::read_from(&mut file).unwrap();
assert_eq!(ebml, ebml_read_1);
assert_eq!(ebml, ebml_read_2);

// 2. using `read_element()`
let mut buf_cursor = std::io::Cursor::new(&buffer);
let header = Header::read_from(&mut buf_cursor).unwrap();
assert_eq!(header.id, Ebml::ID);
let ebml_read_4 = Ebml::read_element(&header, &mut buf_cursor).unwrap();
assert_eq!(ebml, ebml_read_4);

```



### Asynchronous I/O

With features `tokio` enabled, async I/O from tokio is supported.

```rust
# tokio_test::block_on(async {

use mkv_element::prelude::*; // prelude brings all the types into scope
use mkv_element::io::tokio_impl::*; // use tokio_impl for async I/O

/// Create a simple EBML header element
let ebml = Ebml {
    crc32: None,
    ebml_version: None,
    ebml_read_version: None,
    ebml_max_id_length: EbmlMaxIdLength(4),
    ebml_max_size_length: EbmlMaxSizeLength(8),
    doc_type: Some(DocType("matroska".to_string())),
    doc_type_version: Some(DocTypeVersion(1)),
    doc_type_read_version: Some(DocTypeReadVersion(1)),
    void: None,
};

// Write the EBML element to a type implementing std::io::Write

// 1. to a Vec<u8>
let mut buffer = Vec::new();
ebml.async_write_to(&mut buffer).await.unwrap();

// 2. to a file
let mut file = tokio::io::sink(); // replace with actual file, ie. tokio::fs::File::create("path/to/file.mkv").unwrap();
ebml.async_write_to(&mut file).await.unwrap();


// Reading a element can be done from either using the element's `read_from()` method
// or reading out the header first followed by a `read_element()`.
// the latter is useful when you don't know the element type in advance.

// 1. using `read_from()`
let mut buf_cursor = std::io::Cursor::new(&buffer);
let ebml_read_1 = Ebml::async_read_from(&mut buf_cursor).await.unwrap();
// or directly from a slice
let ebml_read_2 = Ebml::async_read_from(&mut &buffer[..]).await.unwrap();
// or from a file
// let mut file = tokio::fs::File::open("path/to/file.mkv").unwrap();
// let ebml_read_3 = Ebml::async_read_from(&mut file).await.unwrap();
assert_eq!(ebml, ebml_read_1);
assert_eq!(ebml, ebml_read_2);

// 2. using `read_element()`
let mut buf_cursor = std::io::Cursor::new(&buffer);
let header = Header::async_read_from(&mut buf_cursor).await.unwrap();
assert_eq!(header.id, Ebml::ID);
let ebml_read_4 = Ebml::async_read_element(&header, &mut buf_cursor).await.unwrap();
assert_eq!(ebml, ebml_read_4);

# })
```


## Note
1. if you need to work with actual MKV files, don't read a whole segment into memory at once, read only the parts you need instead. Real world MKV files can be very large.
2. According to the Matroska specifications, segments and clusters can have an "unknown" size (all size bytes set to 1). In that case, the segment/cluster extends to the end of the file or until the next segment/cluster. This needs to handle by the user. Trying to read such elements with this library will result in an `ElementBodySizeUnknown` error.
3. This library does not attempt to recover from malformed/corrupted data. If such behavior is desired, extra logic can be added on top of this library.
4. Output of this library MAY NOT be the same as input, but should be semantically equivalent and valid. For example, output order of elements may differ from input order, as the order is not strictly enforced by the Matroska specifications.


## Acknowledgements
Some of the ideas and code snippets were inspired by ~or stolen from~ the following sources, thanks to their authors:
- [mp4-atom](https://github.com/kixelated/mp4-atom) by *kixelated*
- [Network protocols, sans I/O](https://sans-io.readthedocs.io/)

#### License
<sup>
This project is licensed under the MIT License.
See the <a href="LICENSE">LICENSE</a> file for details.
</sup>






