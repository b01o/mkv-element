use std::io::{Read, Write};
use crate::{functional::Encode, base_type::Header};
use super::*;

/// Read a type from a reader.
pub trait ReadFrom: Sized {
    fn read_from<R: Read>(r: &mut R) -> Result<Self>;
}

/// Read an element from a reader provided the header.
/// 
/// Useful for two-step reading.
pub trait ReadElement: Sized {
    fn read_element<R: Read>(header: &Header, r: &mut R) -> Result<Self>;
}

/// Read until Self is found
pub trait ReadUntil: Sized {
    fn read_until<R: Read>(r: &mut R) -> Result<Self>;
}

/// Write to a writer.
pub trait WriteTo {
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
    fn read_u8(&mut self) -> Result<u8> {
        let mut buf = [0u8; 1];
        self.read_exact(&mut buf)?;
        Ok(buf[0])
    }
}
impl<T: Read> ReadExt for T {}
