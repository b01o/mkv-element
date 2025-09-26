use super::*;
use crate::{base_type::Header, element::Element, functional::Encode};
use std::io::{Read, Write};

/// Read a type from a reader.
/// Can be implemented for types that can be read without knowing their size beforehand.
pub trait ReadFrom: Sized {
    /// Read Self from a reader.
    fn read_from<R: Read>(r: &mut R) -> Result<Self>;
}

/// implemented for all `Element`s.
pub trait ReadElement: Sized + Element {
    /// Read an element from a reader provided the header.
    fn read_element<R: Read>(header: &Header, r: &mut R) -> Result<Self> {
        let body = header.read_body(r)?;
        Self::decode_body(&mut &body[..])
    }
}
impl<T: Element> ReadElement for T {}

/// Read until Self is found
pub trait ReadUntil: Sized {
    /// Read until Self is found
    fn read_until<R: Read>(r: &mut R) -> Result<Self>;
}

/// Write to a writer.
pub trait WriteTo {
    /// Write an element to a writer.
    fn write_to<W: Write>(&self, w: &mut W) -> Result<()>;
}

impl<T: Encode> WriteTo for T {
    fn write_to<W: Write>(&self, w: &mut W) -> Result<()> {
        //TODO should avoid the extra allocation here
        let mut buf = vec![];
        self.encode(&mut buf)?;
        w.write_all(&buf)?;
        Ok(())
    }
}

/// Extension trait for `std::io::Read` to read primitive types.
pub trait ReadExt: Read {
    /// Read a single byte.
    fn read_u8(&mut self) -> Result<u8> {
        let mut buf = [0u8; 1];
        self.read_exact(&mut buf)?;
        Ok(buf[0])
    }
}
impl<T: Read> ReadExt for T {}
