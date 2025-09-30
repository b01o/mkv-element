#![cfg_attr(docsrs, feature(doc_cfg))]
#![warn(missing_docs)]
#![doc = include_str!("../README.md")]

mod base; // base types for Matroska elements. ie. `VInt64`, `Header`, etc.
mod element; // Element body definitions and traits.
mod error;
mod frame;
mod functional;
mod lacer;
mod leaf; // Leaf elements in Matroska.
mod master; // Master elements in Matroska.
mod supplement; // Supplementary elements in Matroska. Void elements, CRC-32, etc.

// following modules are public
pub mod io;

// Re-export common types
pub use error::*;

/// A prelude for common types and traits.
pub mod prelude {
    pub use crate::base::*;
    pub use crate::element::*;
    pub use crate::frame::*;
    pub use crate::lacer::*;
    pub use crate::leaf::*;
    pub use crate::master::*;
    pub use crate::supplement::*;
}
