#![warn(missing_docs)]
#![doc = include_str!("../README.md")]

mod functional;
pub use functional::coding::*;
/// Error types for this crate.
mod error;
pub use error::*;

/// Variable-length integer types and utilities.
pub mod base_type;
/// I/O utilities.
pub mod io;
/// Leaf types for Matroska elements. ie. types except `Master` elements.
pub mod leaf_elements;

mod element;
