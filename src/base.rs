use crate::error::Error;
use crate::functional::*;
use crate::io::ReadExt;
use crate::io::ReadFrom;
use std::fmt::Debug;
use std::fmt::Display;
use std::io::Read;
use std::ops::Deref;

/// A variable-length integer RFC 8794
#[derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct VInt64(pub u64);

impl Display for VInt64 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // write!(f, "{}", self.as_encoded())
        let encoded = self.as_encoded();
        if encoded <= 0xFF {
            write!(f, "0x{:02X}", encoded)
        } else if encoded <= 0xFFFF {
            write!(f, "0x{:04X}", encoded)
        } else if encoded <= 0xFFFFFF {
            write!(f, "0x{:06X}", encoded)
        } else if encoded <= 0xFFFFFFFF {
            write!(f, "0x{:08X}", encoded)
        } else if encoded <= 0xFFFFFFFFFF {
            write!(f, "0x{:010X}", encoded)
        } else if encoded <= 0xFFFFFFFFFFFF {
            write!(f, "0x{:012X}", encoded)
        } else if encoded <= 0xFFFFFFFFFFFFFF {
            write!(f, "0x{:014X}", encoded)
        } else {
            write!(f, "0x{:016X}", encoded)
        }
    }
}
impl Debug for VInt64 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("VInt64")
            .field(&self.0)
            .field(&format!("{}", self))
            .finish()
    }
}

impl Deref for VInt64 {
    type Target = u64;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl VInt64 {
    /// Create a VInt64 from an already encoded u64 value.
    pub const fn from_encoded(enc: u64) -> Self {
        Self(enc & (u64::MAX >> (enc.leading_zeros() + 1)))
    }

    /// Create a VInt64 from an already encoded u64 value.
    pub fn as_encoded(&self) -> u64 {
        let size = VInt64::encode_size(self.0);
        let mut sbuf = [0u8; 8];
        let slice = &mut sbuf[8 - size..];
        slice.copy_from_slice(&self.0.to_be_bytes()[8 - size..]);
        slice[0] |= 1u8 << (8 - size);
        u64::from_be_bytes(sbuf)
    }

    /// Get the size in bytes of the encoded representation of a u64 value.
    pub const fn encode_size(value: u64) -> usize {
        let leading_zeros = value.leading_zeros() as usize;
        let total_bits = 64 - leading_zeros;
        if total_bits == 0 {
            1
        } else {
            (total_bits + 6).div_euclid(7)
        }
    }
}

impl ReadFrom for VInt64 {
    fn read_from<R: std::io::Read>(r: &mut R) -> crate::Result<Self> {
        let first_byte = r.read_u8()?;
        let leading_zeros = first_byte.leading_zeros() as usize;
        if leading_zeros >= 8 {
            return Err(crate::error::Error::InvalidVInt);
        }

        if leading_zeros == 0 {
            Ok(VInt64((first_byte & 0b0111_1111) as u64))
        } else {
            let mut buf = [0u8; 8];
            let read_buf = &mut buf[8 - leading_zeros..];
            r.read_exact(read_buf)?;
            if leading_zeros != 7 {
                buf[8 - leading_zeros - 1] = first_byte & (0xFF >> (leading_zeros + 1));
            }
            Ok(VInt64(u64::from_be_bytes(buf)))
        }
    }
}

impl Decode for VInt64 {
    fn decode(buf: &mut &[u8]) -> crate::Result<Self> {
        if !buf.has_remaining() {
            return Err(Error::OutOfBounds);
        }
        let first_byte = u8::decode(buf)?;
        if first_byte == 0 {
            return Err(Error::InvalidVInt);
        }
        let leading_zeros = first_byte.leading_zeros() as usize;

        if leading_zeros == 0 {
            Ok(VInt64((first_byte & 0b0111_1111) as u64))
        } else {
            if buf.remaining() < leading_zeros {
                return Err(Error::OutOfBounds);
            }
            let mut bytes = [0u8; 8];
            let read_buf = &mut bytes[8 - leading_zeros..];
            read_buf.copy_from_slice(buf.slice(leading_zeros));

            if leading_zeros != 7 {
                bytes[8 - leading_zeros - 1] = first_byte & (0xFF >> (leading_zeros + 1));
            }
            buf.advance(leading_zeros);
            Ok(VInt64(u64::from_be_bytes(bytes)))
        }
    }
}

impl Encode for VInt64 {
    fn encode<B: BufMut>(&self, buf: &mut B) -> crate::Result<()> {
        let size = VInt64::encode_size(self.0);
        let mut sbuf = [0u8; 8];
        let slice = &mut sbuf[8 - size..];
        slice.copy_from_slice(&self.0.to_be_bytes()[8 - size..]);
        slice[0] |= 1u8 << (8 - size);
        buf.append_slice(slice);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::functional::{Decode, Encode};

    use super::*;
    use std::convert::TryInto;

    #[test]
    fn test_encode_size() {
        let test_pair = [
            (vec![0b1000_0000], 0),
            (vec![0b1000_0001], 1),
            (vec![0b1111_1111], 0b0111_1111),
            (vec![0b0100_0000, 0xFF], 0xFF),
            (vec![0b0100_0001, 0xFF], 0b1_1111_1111),
            (vec![0b0111_1111, 0xFF], 0b11_1111_1111_1111),
            (vec![0b0010_0000, 0b0111_1111, 0xFF], 0b111_1111_1111_1111),
            (vec![0b0010_0000, 0xFF, 0xFF], 0xFFFF),
            (vec![0b0011_1111, 0xFF, 0xFF], 0b1_1111_1111_1111_1111_1111),
            (
                vec![1, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF],
                0xFF_FFFF_FFFF_FFFF,
            ),
        ];
        for (encoded, val) in test_pair {
            assert_eq!(VInt64::encode_size(val), encoded.len());
        }
    }

    #[test]
    fn test_encode() {
        let test_pair = [
            (vec![0b1000_0000], 0),
            (vec![0b1000_0001], 1),
            (vec![0b1111_1111], 0b0111_1111),
            (vec![0b0100_0000, 0xFF], 0xFF),
            (vec![0b0100_0001, 0xFF], 0b1_1111_1111),
            (vec![0b0111_1111, 0xFF], 0b11_1111_1111_1111),
            (vec![0b0010_0000, 0b0111_1111, 0xFF], 0b111_1111_1111_1111),
            (vec![0b0010_0000, 0xFF, 0xFF], 0xFFFF),
            (vec![0b0011_1111, 0xFF, 0xFF], 0b1_1111_1111_1111_1111_1111),
            (
                vec![1, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF],
                0xFF_FFFF_FFFF_FFFF,
            ),
        ];
        for (encoded, val) in test_pair {
            let v = VInt64(val);
            let mut out = vec![];
            v.encode(&mut out).unwrap();
            assert_eq!(encoded, out);

            let encoded_num = v.as_encoded();
            let mut enc8 = vec![0u8; 8 - encoded.len()];
            enc8.extend_from_slice(&encoded);
            let encoded_from = u64::from_be_bytes(enc8.try_into().unwrap());
            assert_eq!(encoded_num, encoded_from);
        }
    }

    #[test]
    fn test_decode() {
        let test_pair = [
            (vec![0b1000_0000], 0),
            (vec![0b1000_0001], 1),
            (vec![0b1111_1111], 0b0111_1111),
            (vec![0b0100_0000, 0xFF], 0xFF),
            (vec![0b0100_0001, 0xFF], 0b1_1111_1111),
            (vec![0b0111_1111, 0xFF], 0b11_1111_1111_1111),
            (vec![0b0010_0000, 0b0111_1111, 0xFF], 0b111_1111_1111_1111),
            (vec![0b0010_0000, 0xFF, 0xFF], 0xFFFF),
            (vec![0b0011_1111, 0xFF, 0xFF], 0b1_1111_1111_1111_1111_1111),
            (
                vec![1, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF],
                0xFF_FFFF_FFFF_FFFF,
            ),
        ];
        for (encoded, val) in test_pair {
            // test read
            let mut c = std::io::Cursor::new(encoded.clone());
            let vint = VInt64::read_from(&mut c).unwrap();
            assert_eq!(*vint, val);

            // test decode
            let ecoded2 = encoded.clone();
            let mut slice_encoded2 = &ecoded2[..];
            let vint_decoded = VInt64::decode(&mut slice_encoded2).unwrap();
            assert_eq!(*vint_decoded, val);

            // test from_encoded
            let mut enc8 = vec![0u8; 8 - encoded.len()];
            enc8.extend_from_slice(&encoded);
            let v = VInt64::from_encoded(u64::from_be_bytes(enc8.try_into().unwrap()));
            assert_eq!(*v, val);
        }
    }
}

/// EBML element header, consisting of an ID and a size.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Header {
    /// EBML ID of the element.
    pub id: VInt64,
    /// Size of the element's data, excluding the header itself.
    pub size: VInt64,
}

impl Header {
    pub(crate) fn read_body<R: Read>(&self, r: &mut R) -> crate::Result<Vec<u8>> {
        // we allocate 4096 bytes upfront and grow as needed
        let size = self.size.0;
        let cap = size.min(4096) as usize;
        let mut buf = Vec::with_capacity(cap);
        let n = std::io::copy(&mut r.take(size), &mut buf)?;
        if size != n {
            return Err(Error::OutOfBounds);
        }
        Ok(buf)
    }
}

impl ReadFrom for Header {
    fn read_from<R: std::io::Read>(reader: &mut R) -> crate::Result<Self> {
        let id = VInt64::read_from(reader)?;
        let size = VInt64::read_from(reader)?;
        Ok(Self { id, size })
    }
}

impl Decode for Header {
    fn decode(buf: &mut &[u8]) -> crate::Result<Self> {
        let id = VInt64::decode(buf)?;
        let size = VInt64::decode(buf)?;
        Ok(Self { id, size })
    }
}

impl Encode for Header {
    fn encode<B: BufMut>(&self, buf: &mut B) -> crate::Result<()> {
        self.id.encode(buf)?;
        self.size.encode(buf)?;
        Ok(())
    }
}
