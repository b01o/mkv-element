use std::ops::Deref;

use crate::base::VInt64;
use crate::element::Element;
use crate::functional::*;

/// Void element, used for padding.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Void {
    /// Size of the void element in bytes.
    pub size: u64,
}
impl Element for Void {
    const ID: VInt64 = VInt64::from_encoded(0xEC);
    fn decode_body(buf: &mut &[u8]) -> crate::Result<Self> {
        let len = buf.len() as u64;
        buf.advance(buf.len());
        Ok(Self { size: len })
    }
    fn encode_body<B: BufMut>(&self, buf: &mut B) -> crate::Result<()> {
        buf.append_slice(&vec![0; self.size as usize]);
        Ok(())
    }
}

/// CRC-32 element, used for integrity checking.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Crc32(pub u32);
impl Deref for Crc32 {
    type Target = u32;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl Element for Crc32 {
    const ID: VInt64 = VInt64::from_encoded(0xBF);
    fn decode_body(buf: &mut &[u8]) -> crate::Result<Self> {
        Ok(Self(u32::decode_exact(buf, 4)?))
    }
    fn encode_body<B: BufMut>(&self, buf: &mut B) -> crate::Result<()> {
        buf.append_slice(&self.0.to_be_bytes());
        Ok(())
    }
}
