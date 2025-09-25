#![warn(missing_docs)]
#![doc = include_str!("../README.md")]


mod functional;
/// Error types for this crate.
mod error;
/// I/O utilities.
mod io;
/// Leaf types for Matroska elements. ie. types except `Master` elements.
pub mod leaf_elements;
/// Variable-length integer types and utilities.
pub mod base_type;


/// Result type for this crate.
pub type Result<T> = std::result::Result<T, error::Error>;

mod element;
