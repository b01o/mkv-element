use crate::base::VInt64;

/// Error types for this crate.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// I/O error, from `std::io::Error`.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Invalid variable-length integer encoding, incidicates a vint longer than 8 bytes.
    #[error("Invalid variable-length integer encoding, 8 leading zeros found...")]
    InvalidVInt,

    /// Attempted to read past the end of the buffer.
    #[error("Attempted to read past the end of the buffer")]
    OutOfBounds,

    /// Attempted to read past the end of the buffer during element body decoding.
    #[error("Element body over decode, ID: {0}")]
    OverDecode(VInt64),

    /// Not all bytes were consumed in a element body
    #[error("Short read: not all bytes were consumed")]
    ShortRead,

    /// Not all bytes were consumed in a element body during element body decoding.
    #[error("Element body under decode, ID: {0}")]
    UnderDecode(VInt64),
}

/// Result type for this crate.
pub type Result<T> = std::result::Result<T, Error>;
