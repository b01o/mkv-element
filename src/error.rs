use crate::base_type::VInt64;


#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Invalid variable-length integer encoding, 8 leading zeros found...")]
    InvalidVInt,

    #[error("Attempted to read past the end of the buffer")]
    OutOfBounds,

    #[error("Element body over decode, ID: {0}")]
    OverDecode(VInt64),

    #[error("Short read: not all bytes were consumed")]
    ShortRead,

    #[error("Element body under decode, ID: {0}")]
    UnderDecode(VInt64),
}
