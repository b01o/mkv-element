#![cfg_attr(docsrs, feature(doc_cfg))]
#![warn(missing_docs)]
#![doc = include_str!("../README.md")]

/// Error types for this crate.
mod error;
mod functional;
pub use error::*;

/// I/O utilities.
pub mod io;

/// base types for Matroska elements. ie. `VInt64`, `Header`, etc.
mod base;
mod frame;
/// Leaf elements in Matroska.
mod leaf;
/// Master elements in Matroska.
mod master;
/// Supplementary elements in Matroska. Void elements, CRC-32, etc.
///
/// These elements are not from the Matroska specification, but Matroska specifications inherit them from EBML specifications.
mod supplement;
// Element body definitions and traits.
mod element;

/// A prelude for common types and traits.
pub mod prelude {
    pub use crate::base::*;
    pub use crate::element::*;
    pub use crate::frame::*;
    pub use crate::leaf::*;
    pub use crate::master::*;
    pub use crate::supplement::*;
}
