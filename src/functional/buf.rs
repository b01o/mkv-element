use std::ops::RangeBounds;

/// A contiguous buffer of bytes.
pub trait Buf: std::fmt::Debug {
    fn remaining(&self) -> usize;
    fn slice(&self, size: usize) -> &[u8];
    fn advance(&mut self, n: usize);
    fn has_remaining(&self) -> bool {
        self.remaining() > 0
    }
}

impl Buf for &[u8] {
    fn remaining(&self) -> usize {
        self.len()
    }

    fn slice(&self, size: usize) -> &[u8] {
        &self[..size]
    }

    fn advance(&mut self, n: usize) {
        *self = &self[n..];
    }
}

/// A mutable contiguous buffer of bytes.
// We're not using bytes::BufMut because it doesn't allow seeking backwards (to set the size).
pub trait BufMut: std::fmt::Debug {
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
    fn append_slice(&mut self, val: &[u8]);
    fn set_slice(&mut self, pos: usize, val: &[u8]);
    fn offset_within(&mut self, src: impl RangeBounds<usize>, offset: usize);
}

impl BufMut for Vec<u8> {
    fn len(&self) -> usize {
        self.len()
    }

    fn append_slice(&mut self, val: &[u8]) {
        self.extend_from_slice(val);
    }

    fn set_slice(&mut self, pos: usize, val: &[u8]) {
        self[pos..pos + val.len()].copy_from_slice(val);
    }

    fn offset_within(&mut self, src: impl RangeBounds<usize>, offset: usize) {
        let start = match src.start_bound() {
            std::ops::Bound::Included(&start) => start,
            std::ops::Bound::Excluded(&start) => start + 1,
            std::ops::Bound::Unbounded => 0,
        };
        self.copy_within(src, start + offset);
    }
}

impl<T: BufMut + ?Sized> BufMut for &mut T {
    fn len(&self) -> usize {
        (**self).len()
    }

    fn append_slice(&mut self, val: &[u8]) {
        (**self).append_slice(val)
    }

    fn set_slice(&mut self, pos: usize, val: &[u8]) {
        (**self).set_slice(pos, val)
    }

    fn offset_within(&mut self, src: impl RangeBounds<usize>, offset: usize) {
        (**self).offset_within(src, offset)
    }
}