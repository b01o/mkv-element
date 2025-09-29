use crate::base::*;
use crate::error::Error;
use crate::functional::*;
use crate::io::blocking::*;

/// A Matroska element.
pub trait Element: Sized {
    /// EBML ID of the element.
    const ID: VInt64;
    /// Whether the element has a default value, as per Matroska specification.
    /// If true, and the element is missing in a master element, it should be treated as if it were present with the default value.
    /// If false, and the element is missing in a master element, it should be treated as an error.
    const HAS_DEFAULT_VALUE: bool = false;

    /// Decode the body of the element from a buffer.
    fn decode_body(buf: &mut &[u8]) -> crate::Result<Self>;

    /// Encode the body of the element to a buffer.
    fn encode_body<B: BufMut>(&self, buf: &mut B) -> crate::Result<()>;
}

impl<T: Element> Decode for T {
    fn decode(buf: &mut &[u8]) -> crate::Result<Self> {
        let header = Header::decode(buf)?;
        let body_size = *header.size as usize;
        if buf.remaining() < body_size {
            return Err(crate::error::Error::OutOfBounds);
        }
        let mut body = buf.slice(body_size);
        let element = match T::decode_body(&mut body) {
            Ok(e) => e,
            Err(Error::OutOfBounds) => return Err(Error::OverDecode(Self::ID)),
            Err(Error::ShortRead) => return Err(Error::UnderDecode(Self::ID)),
            Err(e) => return Err(e),
        };

        if body.has_remaining() {
            return Err(Error::UnderDecode(Self::ID));
        }

        buf.advance(body_size);
        Ok(element)
    }
}

impl<T: Element> Encode for T {
    fn encode<B: BufMut>(&self, buf: &mut B) -> crate::Result<()> {
        let mut body_buf = Vec::new();
        self.encode_body(&mut body_buf)?;
        let header = Header {
            id: T::ID,
            size: VInt64::new(body_buf.len() as u64),
        };
        header.encode(buf)?;
        buf.append_slice(&body_buf);
        Ok(())
    }
}

impl<T: Element> ReadFrom for T {
    fn read_from<R: std::io::Read>(r: &mut R) -> crate::Result<Self> {
        let header = Header::read_from(r)?;
        let body = header.read_body(r)?;
        let element = match T::decode_body(&mut &body[..]) {
            Ok(e) => e,
            Err(Error::OutOfBounds) => return Err(Error::OverDecode(Self::ID)),
            Err(Error::ShortRead) => return Err(Error::UnderDecode(Self::ID)),
            Err(e) => return Err(e),
        };
        Ok(element)
    }
}

#[cfg(feature = "tokio")]
impl<T: Element> crate::io::tokio_impl::AsyncReadFrom for T {
    async fn async_read_from<R: tokio::io::AsyncRead + Unpin>(r: &mut R) -> crate::Result<Self> {
        let header = Header::async_read_from(r).await?;
        let body = header.read_body_tokio(r).await?;
        let element = match T::decode_body(&mut &body[..]) {
            Ok(e) => e,
            Err(Error::OutOfBounds) => return Err(Error::OverDecode(Self::ID)),
            Err(Error::ShortRead) => return Err(Error::UnderDecode(Self::ID)),
            Err(e) => return Err(e),
        };
        Ok(element)
    }
}
