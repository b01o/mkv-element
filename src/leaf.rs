#![allow(clippy::doc_lazy_continuation)] // auto-generated docs may have lazy continuation
use std::ops::Deref;

mod uint {
    #![allow(dead_code)]
    use std::ops::Deref;

    use crate::{base::VInt64, element::Element, functional::Buf};

    /// Bottom type for *unsigned integers*.
    #[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
    pub struct UnsignedInteger(pub u64);
    impl Deref for UnsignedInteger {
        type Target = u64;
        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }

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

mod sint {
    #![allow(dead_code)]
    use std::ops::Deref;

    use crate::{base::VInt64, element::Element, functional::Buf};

    /// Bottom type for *signed integers*.
    #[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
    pub struct SignedInteger(pub i64);
    impl Deref for SignedInteger {
        type Target = i64;
        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }

    impl Element for SignedInteger {
        const ID: VInt64 = VInt64::from_encoded(0x13);
        fn decode_body(buf: &mut &[u8]) -> crate::Result<Self> {
            if buf.is_empty() {
                return Ok(Self(0));
            }
            if buf.len() > 8 {
                return Err(crate::Error::UnderDecode(Self::ID));
            }
            let len = buf.len().min(8);
            let is_neg = (buf[0] & 0x80) != 0;
            let mut value = if is_neg { [0xFFu8; 8] } else { [0u8; 8] };
            value[8 - len..].copy_from_slice(&buf[..len]);
            buf.advance(len);
            Ok(Self(i64::from_be_bytes(value)))
        }
        fn encode_body<B: crate::functional::BufMut>(&self, buf: &mut B) -> crate::Result<()> {
            let bytes = self.0.to_be_bytes();
            if self.0 >= 0 {
                let first_non_zero = bytes
                    .iter()
                    .position(|&b| b != 0)
                    .unwrap_or(bytes.len() - 1);
                buf.append_slice(&bytes[first_non_zero..]);
                Ok(())
            } else {
                let first_non_ff = bytes
                    .iter()
                    .position(|&b| b != 0xFF)
                    .unwrap_or(bytes.len() - 1);
                buf.append_slice(&bytes[first_non_ff..]);
                Ok(())
            }
        }
    }
    #[cfg(test)]
    mod tests {
        use super::*;
        #[test]
        fn test_sint() {
            assert_eq!(-2i64.pow(15), -32768);

            let positive = |n: u32| 2i64.pow((n * 8) - 1) - 1;
            let negative = |n: u32| -2i64.pow((n * 8) - 1);

            let test_pair = [
                (vec![0u8], 0i64),
                (vec![1u8], 1i64),
                (vec![0xFF], -1i64),
                (vec![0x2A], 42),
                (vec![0xD6], -42),
                (vec![0x03, 0xE8], 1000),
                (vec![0xFC, 0x18], -1000),
                (vec![0x7F], 127),                                 // 2^7 - 1
                (vec![0x80], -128),                                // -2^7
                (vec![0x7F, 0xFF], positive(2)),                   // 2^15 - 1
                (vec![0x80, 0x00], negative(2)),                   // -2^15
                (vec![0x7F, 0xFF, 0xFF], positive(3)),             // 2^23 - 1
                (vec![0x80, 0x00, 0x00], negative(3)),             // -2^23
                (vec![0x7F, 0xFF, 0xFF, 0xFF], positive(4)),       // 2^31 - 1
                (vec![0x80, 0x00, 0x00, 0x00], negative(4)),       // -2^31
                (vec![0x7F, 0xFF, 0xFF, 0xFF, 0xFF], positive(5)), // 2^39 -1
                (vec![0x80, 0x00, 0x00, 0x00, 0x00], negative(5)), // -2^39
                (
                    vec![0x7F, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF],
                    i64::MAX,
                ),
                (
                    vec![0x80, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00],
                    i64::MIN,
                ),
            ];
            for (encoded, decoded) in test_pair {
                let v = SignedInteger::decode_body(&mut &*encoded).unwrap();
                assert_eq!(v, SignedInteger(decoded));

                let mut buf = vec![];
                SignedInteger(decoded).encode_body(&mut buf).unwrap();
                assert_eq!(buf, encoded);
            }
        }
    }
}

mod float {
    #![allow(dead_code)]
    use std::ops::Deref;

    use crate::{base::VInt64, element::Element, functional::Buf};

    /// Bottom type for *floating point numbers*.
    #[derive(Debug, Copy, Clone, PartialEq, PartialOrd)]
    pub struct Float(pub f64);
    impl Deref for Float {
        type Target = f64;
        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }

    impl Element for Float {
        const ID: VInt64 = VInt64::from_encoded(0x14);
        fn decode_body(buf: &mut &[u8]) -> crate::Result<Self> {
            match buf.len() {
                0 => Ok(Self(0.0)),
                4 => {
                    let mut value = [0u8; 4];
                    value.copy_from_slice(&buf[..4]);
                    buf.advance(4);
                    Ok(Self(f32::from_be_bytes(value) as f64))
                }
                8 => {
                    let mut value = [0u8; 8];
                    value.copy_from_slice(&buf[..8]);
                    buf.advance(8);
                    Ok(Self(f64::from_be_bytes(value)))
                }
                _ => Err(crate::Error::UnderDecode(Self::ID)),
            }
        }
        fn encode_body<B: crate::functional::BufMut>(&self, buf: &mut B) -> crate::Result<()> {
            fn can_represent_as_f32(value: f64) -> bool {
                if value.is_infinite() || value.is_nan() {
                    return false;
                }

                if value.abs() > f32::MAX as f64
                    || (value != 0.0 && value.abs() < f32::MIN_POSITIVE as f64)
                {
                    return false;
                }

                let f32_value = value as f32;
                f32_value as f64 == value
            }

            if can_represent_as_f32(self.0) {
                buf.append_slice(&(self.0 as f32).to_be_bytes());
                Ok(())
            } else {
                buf.append_slice(&self.0.to_be_bytes());
                Ok(())
            }
        }
    }
    #[cfg(test)]
    mod tests {
        use super::*;
        #[test]
        fn test_float() {
            let test_pair = [
                0f64,
                -1.0,
                1.0,
                f32::MIN_POSITIVE as f64,
                f32::MIN as f64,
                f32::MAX as f64,
                f64::MIN_POSITIVE,
                f64::MIN,
                f64::MAX,
            ]
            .iter()
            .map(|&v| (v.to_be_bytes().to_vec(), v));

            for (encoded, decoded) in test_pair {
                let v = Float::decode_body(&mut &*encoded).unwrap();
                assert_eq!(v, Float(decoded));

                let mut buf = vec![];
                Float(decoded).encode_body(&mut buf).unwrap();
                let new_v = Float::decode_body(&mut &*buf).unwrap();
                assert_eq!(new_v, Float(decoded));
            }
        }
    }
}

mod text {
    #![allow(dead_code)]
    use std::ops::Deref;

    use crate::{base::VInt64, element::Element, functional::Buf};

    /// Bottom type for *text strings*.
    #[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
    pub struct Text(pub String);
    impl Deref for Text {
        type Target = str;
        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }

    impl Element for Text {
        const ID: VInt64 = VInt64::from_encoded(0x15);
        fn decode_body(buf: &mut &[u8]) -> crate::Result<Self> {
            let first_zero = buf.iter().position(|&b| b == 0).unwrap_or(buf.len());
            let result = Self(String::from_utf8_lossy(&buf[..first_zero]).to_string());
            buf.advance(buf.len());
            Ok(result)
        }
        fn encode_body<B: crate::functional::BufMut>(&self, buf: &mut B) -> crate::Result<()> {
            buf.append_slice(self.0.as_bytes());
            Ok(())
        }
    }
    #[cfg(test)]
    mod tests {
        use super::*;
        #[test]
        fn test_text() {
            let test_pair = [
                (vec![], ""),
                (vec![b'h', b'e', b'y'], "hey"),
                ("testing utf8 ✓".as_bytes().to_vec(), "testing utf8 ✓"),
                ("こんにちは".as_bytes().to_vec(), "こんにちは"),
                (vec![b'h', b'e', b'y', 0, b'w'], "hey"),
            ];

            for (encoded, decoded) in test_pair {
                // Decode the text
                let v = Text::decode_body(&mut &*encoded).unwrap();
                assert_eq!(v, Text(decoded.to_string()));

                let mut buf = vec![];
                Text(decoded.to_string()).encode_body(&mut buf).unwrap();
                if !encoded.contains(&0) {
                    assert_eq!(buf, encoded);
                }
                let new_decoded = Text::decode_body(&mut &*buf).unwrap();
                assert_eq!(new_decoded, Text(decoded.to_string()));
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
