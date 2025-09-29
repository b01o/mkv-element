use crate::{
    base::VInt64,
    functional::{Decode, Encode},
    leaf::SimpleBlock,
    master::{BlockGroup, Cluster},
};

/// A Matroska encoded frame.
pub struct Frame<'a> {
    /// in matroska timestamp units
    pub data: &'a [u8],
    /// whether the frame is a keyframe
    pub is_keyframe: bool,
    /// whether the frame is invisible (mostly for subtitle tracks)
    pub is_invisible: bool,
    /// whether the frame is discardable (for video tracks, e.g. non-reference frames)
    pub is_discardable: bool,
    /// track number the frame belongs to
    pub track_number: u64,
    /// timestamp of the frame, in the same timescale as the Cluster timestamp
    pub timestamp: i64,
}

/// A block in a Cluster, either a SimpleBlock or a BlockGroup.
///
/// This is a convenience enum to allow handling both types of blocks uniformly.
/// * when reading: often we just want to iterate over all blocks in a cluster, regardless of type.
/// * when writing: we may want to write a list of blocks of mixed types.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClusterBlock {
    /// A SimpleBlock
    Simple(SimpleBlock),
    /// A BlockGroup
    Group(BlockGroup),
}
impl ClusterBlock {
    fn block_ref(&self) -> BlockRef<'_> {
        match self {
            ClusterBlock::Simple(b) => BlockRef::Simple(b),
            ClusterBlock::Group(b) => BlockRef::Group(b),
        }
    }
}
impl From<SimpleBlock> for ClusterBlock {
    fn from(b: SimpleBlock) -> Self {
        ClusterBlock::Simple(b)
    }
}
impl From<BlockGroup> for ClusterBlock {
    fn from(b: BlockGroup) -> Self {
        ClusterBlock::Group(b)
    }
}

impl Encode for ClusterBlock {
    fn encode<B: crate::functional::BufMut>(&self, buf: &mut B) -> crate::Result<()> {
        match self {
            ClusterBlock::Simple(b) => b.encode(buf),
            ClusterBlock::Group(b) => b.encode(buf),
        }
    }
}

enum BlockRef<'a> {
    Simple(&'a crate::leaf::SimpleBlock),
    Group(&'a crate::master::BlockGroup),
}

impl<'a> BlockRef<'a> {
    fn into_frames(self, cluster_ts: u64) -> impl Iterator<Item = crate::Result<Frame<'a>>> + 'a {
        match self {
            BlockRef::Simple(block) => {
                let body_buf = &mut &block[..];

                let track_number = match VInt64::decode(body_buf) {
                    Ok(num) => num,
                    Err(e) => return std::iter::once(Err(e)),
                };

                let relative_timestamp = match i16::decode(body_buf) {
                    Ok(ts) => ts,
                    Err(e) => return std::iter::once(Err(e)),
                };

                let flag = match u8::decode(body_buf) {
                    Ok(f) => f,
                    Err(e) => return std::iter::once(Err(e)),
                };

                let data = *body_buf;

                let lacing = (flag >> 1) & 0x03;

                // TODO handle lacing
                if lacing == 0 {
                    // no lacing, single frame
                    std::iter::once(Ok(Frame {
                        data,
                        is_keyframe: (flag & 0x80) != 0,
                        is_invisible: (flag & 0x08) != 0,
                        is_discardable: (flag & 0x01) != 0,
                        track_number: *track_number,
                        timestamp: cluster_ts as i64 + relative_timestamp as i64,
                    }))
                } else {
                    unimplemented!()
                }
            }
            BlockRef::Group(g) => {
                let block = &g.block;
                let body_buf = &mut &block[..];

                let track_number = match VInt64::decode(body_buf) {
                    Ok(num) => num,
                    Err(e) => return std::iter::once(Err(e)),
                };

                let relative_timestamp = match i16::decode(body_buf) {
                    Ok(ts) => ts,
                    Err(e) => return std::iter::once(Err(e)),
                };

                let flag = match u8::decode(body_buf) {
                    Ok(f) => f,
                    Err(e) => return std::iter::once(Err(e)),
                };

                let data = *body_buf;
                let lacing = (flag >> 1) & 0x03;
                if lacing == 0 {
                    // no lacing
                    std::iter::once(Ok(Frame {
                        data,
                        is_keyframe: g.reference_block.is_empty(),
                        is_invisible: flag & 0x08 != 0,
                        is_discardable: false,
                        track_number: *track_number,
                        timestamp: cluster_ts as i64 + relative_timestamp as i64,
                    }))
                } else {
                    unimplemented!()
                }
            }
        }
    }
}

impl<'a> From<&'a crate::leaf::SimpleBlock> for BlockRef<'a> {
    fn from(b: &'a crate::leaf::SimpleBlock) -> Self {
        BlockRef::Simple(b)
    }
}
impl<'a> From<&'a crate::master::BlockGroup> for BlockRef<'a> {
    fn from(b: &'a crate::master::BlockGroup) -> Self {
        BlockRef::Group(b)
    }
}

impl Cluster {
    /// frames in the cluster.
    pub fn frames(&self) -> impl Iterator<Item = crate::Result<Frame<'_>>> + '_ {
        self.blocks
            .iter()
            .map(|b| b.block_ref())
            .flat_map(|b| b.into_frames(*self.timestamp))
    }
}
