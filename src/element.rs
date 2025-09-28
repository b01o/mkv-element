use crate::base::*;
use crate::error::Error;
use crate::functional::*;
use crate::io::ReadFrom;

/// A Matroska element.
pub trait Element: Sized {
    const ID: VInt64;
    const HAS_DEFAULT_VALUE: bool = false;
    fn decode_body(buf: &mut &[u8]) -> crate::Result<Self>;
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
            size: VInt64(body_buf.len() as u64),
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
