#![warn(missing_docs)]
#![doc = include_str!("../README.md")]

mod functional;
pub use functional::coding;
/// Error types for this crate.
mod error;
pub use error::*;

/// base types for Matroska elements. ie. `VInt64`, `Header`, etc.
pub mod base;
/// I/O utilities.
pub mod io;

/// Leaf elements in Matroska.
pub mod leaf;
/// Master elements in Matroska.
pub mod master;
/// Supplementary elements in Matroska. Void elements, CRC-32, etc.
///
/// These elements are not from the Matroska specification, but Matroska specifications inherit them from EBML specifications.
pub mod supplement;

mod element;
