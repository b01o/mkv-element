#![allow(clippy::doc_lazy_continuation)] // auto-generated docs may have lazy continuation
use std::ops::Deref;

mod uint {
    #![allow(dead_code)]
    use crate::{base::VInt64, element::Element, functional::Buf};

    /// Bottom type for *unsigned integers*.
    #[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
    pub struct UnsignedInteger(pub u64);
    impl Element for UnsignedInteger {
        const ID: VInt64 = VInt64::from_encoded(0x12);
        fn decode_body(buf: &mut &[u8]) -> crate::Result<Self> {
            if buf.is_empty() {
                return Ok(Self(0));
            }
            if buf.len() > 8 {
                return Err(crate::Error::UnderDecode(Self::ID));
            }
            let len = buf.len().min(8);
            let mut value = [0u8; 8];
            value[8 - len..].copy_from_slice(&buf[..len]);
            buf.advance(len);
            Ok(Self(u64::from_be_bytes(value)))
        }
        fn encode_body<B: crate::functional::BufMut>(&self, buf: &mut B) -> crate::Result<()> {
            let bytes = self.0.to_be_bytes();
            let first_non_zero = bytes
                .iter()
                .position(|&b| b != 0)
                .unwrap_or(bytes.len() - 1);
            buf.append_slice(&bytes[first_non_zero..]);
            Ok(())
        }
    }
    #[cfg(test)]
    mod tests {
        use super::*;
        #[test]
        fn test_uint() {
            let test_pair = [
                (vec![1u8], 1u64),
                (vec![0u8], 0u64),
                (vec![0xFFu8], 255u64),
                (vec![0x01u8, 0], 256u64),
                (vec![0x01u8, 0xFF], 256u64 + 255),
                (vec![0xFFu8, 0xFFu8], 2u64.pow(16) - 1),
                (vec![1, 0, 0], 2u64.pow(16)),
                (vec![1, 0, 0, 0], 2u64.pow(24)),
                (vec![1, 0, 0, 0, 0, 0, 0, 0], 2u64.pow(56)),
                (vec![0xFF; 8], u64::MAX),
            ];
            for (encoded, decoded) in test_pair {
                let v = UnsignedInteger::decode_body(&mut &*encoded).unwrap();
                assert_eq!(v, UnsignedInteger(decoded));

                let mut buf = vec![];
                UnsignedInteger(decoded).encode_body(&mut buf).unwrap();
                assert_eq!(buf, encoded);
            }
        }
    }
}

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
