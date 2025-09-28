//! Encoding and decoding Element or other types from buffers in memory.

use super::*;
use crate::{Result, base::Header, element::Element, error::Error};

/// Decode an element from a buffer.
pub trait Decode: Sized {
    /// Decode an element from the buffer.
    fn decode(buf: &mut &[u8]) -> Result<Self>;

    /// Helper: Decode exactly size bytes from the buffer.
    fn decode_exact(buf: &mut &[u8], size: usize) -> Result<Self> {
        if buf.remaining() < size {
            return Err(Error::OutOfBounds);
        }
        let mut inner = buf.slice(size);
        let res = Self::decode(&mut inner)?;
        if inner.has_remaining() {
            return Err(Error::ShortRead);
        }
        buf.advance(size);
        Ok(res)
    }
}

/// Decode an element using the provided header
pub trait DecodeElement: Sized + Element {
    /// Decode an element using the provided header
    /// implemented for all `Element`s.
    fn decode_element(header: &Header, buf: &mut &[u8]) -> Result<Self> {
        let size = *header.size as usize;
        if size > buf.remaining() {
            return Err(crate::error::Error::OutOfBounds);
        }
        let mut body = buf.slice(size);
        let element = match Self::decode_body(&mut body) {
            Ok(e) => e,
            Err(Error::OutOfBounds) => return Err(Error::OverDecode(Self::ID)),
            Err(Error::ShortRead) => return Err(Error::UnderDecode(Self::ID)),
            Err(e) => return Err(e),
        };

        if body.has_remaining() {
            return Err(Error::UnderDecode(Self::ID));
        }

        buf.advance(size);
        Ok(element)
    }
}
impl<T: Element> DecodeElement for T {}

impl<const N: usize> Decode for [u8; N] {
    fn decode(buf: &mut &[u8]) -> Result<Self> {
        if buf.len() < N {
            return Err(Error::OutOfBounds);
        }
        let mut v = [0u8; N];
        v.copy_from_slice(buf.slice(N));
        buf.advance(N);
        Ok(v)
    }
}

impl Decode for u8 {
    fn decode(buf: &mut &[u8]) -> Result<Self> {
        Ok(Self::from_be_bytes(<[u8; 1]>::decode(buf)?))
    }
}

impl Decode for i8 {
    fn decode(buf: &mut &[u8]) -> Result<Self> {
        Ok(Self::from_be_bytes(<[u8; 1]>::decode(buf)?))
    }
}

impl Decode for u16 {
    fn decode(buf: &mut &[u8]) -> Result<Self> {
        Ok(Self::from_be_bytes(<[u8; 2]>::decode(buf)?))
    }
}

impl Decode for i16 {
    fn decode(buf: &mut &[u8]) -> Result<Self> {
        Ok(Self::from_be_bytes(<[u8; 2]>::decode(buf)?))
    }
}

impl Decode for u32 {
    fn decode(buf: &mut &[u8]) -> Result<Self> {
        Ok(Self::from_be_bytes(<[u8; 4]>::decode(buf)?))
    }
}

impl Decode for i32 {
    fn decode(buf: &mut &[u8]) -> Result<Self> {
        Ok(Self::from_be_bytes(<[u8; 4]>::decode(buf)?))
    }
}

impl Decode for u64 {
    fn decode(buf: &mut &[u8]) -> Result<Self> {
        Ok(Self::from_be_bytes(<[u8; 8]>::decode(buf)?))
    }
}

impl Decode for i64 {
    fn decode(buf: &mut &[u8]) -> Result<Self> {
        Ok(Self::from_be_bytes(<[u8; 8]>::decode(buf)?))
    }
}

impl<T: Decode> Decode for Vec<T> {
    fn decode(buf: &mut &[u8]) -> Result<Self> {
        let mut vec = Vec::new();
        while buf.has_remaining() {
            let item = T::decode(buf)?;
            vec.push(item);
        }
        Ok(vec)
    }
}

/// Encode an element to a buffer.
pub trait Encode {
    /// Encode self to the buffer.
    fn encode<B: BufMut>(&self, buf: &mut B) -> Result<()>;
}

impl Encode for u8 {
    fn encode<B: BufMut>(&self, buf: &mut B) -> Result<()> {
        self.to_be_bytes().encode(buf)
    }
}

impl Encode for i8 {
    fn encode<B: BufMut>(&self, buf: &mut B) -> Result<()> {
        self.to_be_bytes().encode(buf)
    }
}

impl Encode for i16 {
    fn encode<B: BufMut>(&self, buf: &mut B) -> Result<()> {
        self.to_be_bytes().encode(buf)
    }
}

impl Encode for u16 {
    fn encode<B: BufMut>(&self, buf: &mut B) -> Result<()> {
        self.to_be_bytes().encode(buf)
    }
}

impl Encode for u32 {
    fn encode<B: BufMut>(&self, buf: &mut B) -> Result<()> {
        self.to_be_bytes().encode(buf)
    }
}

impl Encode for i32 {
    fn encode<B: BufMut>(&self, buf: &mut B) -> Result<()> {
        self.to_be_bytes().encode(buf)
    }
}

impl Encode for u64 {
    fn encode<B: BufMut>(&self, buf: &mut B) -> Result<()> {
        self.to_be_bytes().encode(buf)
    }
}

impl Encode for i64 {
    fn encode<B: BufMut>(&self, buf: &mut B) -> Result<()> {
        self.to_be_bytes().encode(buf)
    }
}

impl<const N: usize> Encode for [u8; N] {
    fn encode<B: BufMut>(&self, buf: &mut B) -> Result<()> {
        buf.append_slice(self);
        Ok(())
    }
}

impl<T: Encode> Encode for &[T] {
    fn encode<B: BufMut>(&self, buf: &mut B) -> Result<()> {
        for item in self.iter() {
            item.encode(buf)?;
        }

        Ok(())
    }
}

impl<T: Encode> Encode for Option<T> {
    fn encode<B: BufMut>(&self, buf: &mut B) -> Result<()> {
        match self {
            Some(v) => v.encode(buf),
            None => Ok(()),
        }
    }
}

impl Encode for &str {
    fn encode<B: BufMut>(&self, buf: &mut B) -> Result<()> {
        self.as_bytes().encode(buf)?;
        0u8.encode(buf)?;
        Ok(())
    }
}

impl<T: Encode> Encode for Vec<T> {
    fn encode<B: BufMut>(&self, buf: &mut B) -> Result<()> {
        for item in self.iter() {
            item.encode(buf)?;
        }

        Ok(())
    }
}
