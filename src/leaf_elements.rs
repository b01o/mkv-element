use std::ops::Deref;

/// Bottom type for *unsigned integers*.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct UnsignedInteger<const ID: u64>(u64);

/// Bottom type for *signed integers*.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct SignedInteger<const ID: u64>(i64);

/// Bottom type for *floating point numbers*.
#[derive(Debug, Copy, Clone, PartialEq, PartialOrd)]
pub struct Float<const ID: u64>(f64);

/// Bottom type for *text strings*.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Text<const ID: u64>(String);

/// Bottom type for *date/time values*.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Date<const ID: u64>(Vec<u8>);

/// Bottom type for *binary data*.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Bin<const ID: u64>(Vec<u8>);

impl<const ID: u64> Deref for UnsignedInteger<ID> {
    type Target = u64;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl<const ID: u64> Deref for SignedInteger<ID> {
    type Target = i64;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl<const ID: u64> Deref for Float<ID> {
    type Target = f64;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl<const ID: u64> Deref for Text<ID> {
    type Target = str;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl<const ID: u64> Deref for Date<ID> {
    type Target = [u8];
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl<const ID: u64> Deref for Bin<ID> {
    type Target = [u8];
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

// Auto-generated element types.
include!(concat!(env!("OUT_DIR"), "/generated_types.rs"));



