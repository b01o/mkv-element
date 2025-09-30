use crate::{
    base::VInt64,
    functional::{Decode, Encode},
    lacer::Lacer,
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
        // Without automatic sum types or generators, it's kind of amusing to write an iterator
        // FIXME: Replace this workaround with a generator or sum type iterator when Rust stabilizes generators (see https://github.com/rust-lang/rust/issues/43122)
        enum Output<T1, T2, T3, T4, T5, T6, T7> {
            Once(T1),
            Xiph(T2),
            Xiph2(T3),
            Ebml(T4),
            Ebml2(T5),
            FixedSize(T6),
            FixedSize2(T7),
        }

        impl<O, T1, T2, T3, T4, T5, T6, T7> Iterator for Output<T1, T2, T3, T4, T5, T6, T7>
        where
            T1: Iterator<Item = O>,
            T2: Iterator<Item = O>,
            T3: Iterator<Item = O>,
            T4: Iterator<Item = O>,
            T5: Iterator<Item = O>,
            T6: Iterator<Item = O>,
            T7: Iterator<Item = O>,
        {
            type Item = O;
            fn next(&mut self) -> Option<Self::Item> {
                match self {
                    Output::Once(it) => it.next(),
                    Output::Xiph(it) => it.next(),
                    Output::Xiph2(it) => it.next(),
                    Output::Ebml(it) => it.next(),
                    Output::Ebml2(it) => it.next(),
                    Output::FixedSize(it) => it.next(),
                    Output::FixedSize2(it) => it.next(),
                }
            }
        }

        match self {
            BlockRef::Simple(block) => {
                let body_buf = &mut &block[..];

                let track_number = match VInt64::decode(body_buf) {
                    Ok(num) => num,
                    Err(e) => return Output::Once(std::iter::once(Err(e))),
                };

                let relative_timestamp = match i16::decode(body_buf) {
                    Ok(ts) => ts,
                    Err(e) => return Output::Once(std::iter::once(Err(e))),
                };

                let flag = match u8::decode(body_buf) {
                    Ok(f) => f,
                    Err(e) => return Output::Once(std::iter::once(Err(e))),
                };

                let data = *body_buf;

                let lacing = (flag >> 1) & 0x03;

                if lacing == 0 {
                    // no lacing, single frame
                    Output::Once(std::iter::once(Ok(Frame {
                        data,
                        is_keyframe: (flag & 0x80) != 0,
                        is_invisible: (flag & 0x08) != 0,
                        is_discardable: (flag & 0x01) != 0,
                        track_number: *track_number,
                        timestamp: cluster_ts as i64 + relative_timestamp as i64,
                    })))
                } else if lacing == 0b01 {
                    let data = match Lacer::Xiph.delace(data) {
                        Ok(frames) => frames,
                        Err(e) => return Output::Once(std::iter::once(Err(e))),
                    };

                    Output::Xiph(data.into_iter().map(move |d| {
                        Ok(Frame {
                            data: d,
                            is_keyframe: (flag & 0x80) != 0,
                            is_invisible: (flag & 0x08) != 0,
                            is_discardable: (flag & 0x01) != 0,
                            track_number: *track_number,
                            timestamp: cluster_ts as i64 + relative_timestamp as i64,
                        })
                    }))
                } else if lacing == 0b11 {
                    // EBML lacing
                    let data = match Lacer::Ebml.delace(data) {
                        Ok(frames) => frames,
                        Err(e) => return Output::Once(std::iter::once(Err(e))),
                    };
                    Output::Ebml(data.into_iter().map(move |d| {
                        Ok(Frame {
                            data: d,
                            is_keyframe: (flag & 0x80) != 0,
                            is_invisible: (flag & 0x08) != 0,
                            is_discardable: (flag & 0x01) != 0,
                            track_number: *track_number,
                            timestamp: cluster_ts as i64 + relative_timestamp as i64,
                        })
                    }))
                } else {
                    // fixed-size lacing
                    let data = match Lacer::FixedSize.delace(data) {
                        Ok(frames) => frames,
                        Err(e) => return Output::Once(std::iter::once(Err(e))),
                    };
                    Output::FixedSize(data.into_iter().map(move |d| {
                        Ok(Frame {
                            data: d,
                            is_keyframe: (flag & 0x80) != 0,
                            is_invisible: (flag & 0x08) != 0,
                            is_discardable: (flag & 0x01) != 0,
                            track_number: *track_number,
                            timestamp: cluster_ts as i64 + relative_timestamp as i64,
                        })
                    }))
                }
            }
            BlockRef::Group(g) => {
                let block = &g.block;
                let body_buf = &mut &block[..];

                let track_number = match VInt64::decode(body_buf) {
                    Ok(num) => num,
                    Err(e) => return Output::Once(std::iter::once(Err(e))),
                };

                let relative_timestamp = match i16::decode(body_buf) {
                    Ok(ts) => ts,
                    Err(e) => return Output::Once(std::iter::once(Err(e))),
                };

                let flag = match u8::decode(body_buf) {
                    Ok(f) => f,
                    Err(e) => return Output::Once(std::iter::once(Err(e))),
                };

                let data = *body_buf;
                let lacing = (flag >> 1) & 0x03;
                if lacing == 0 {
                    // no lacing
                    Output::Once(std::iter::once(Ok(Frame {
                        data,
                        is_keyframe: g.reference_block.is_empty(),
                        is_invisible: flag & 0x08 != 0,
                        is_discardable: false,
                        track_number: *track_number,
                        timestamp: cluster_ts as i64 + relative_timestamp as i64,
                    })))
                } else if lacing == 0b01 {
                    let data = match Lacer::Xiph.delace(data) {
                        Ok(frames) => frames,
                        Err(e) => return Output::Once(std::iter::once(Err(e))),
                    };

                    Output::Xiph2(data.into_iter().map(move |d| {
                        Ok(Frame {
                            data: d,
                            is_keyframe: g.reference_block.is_empty(),
                            is_invisible: flag & 0x08 != 0,
                            is_discardable: false,
                            track_number: *track_number,
                            timestamp: cluster_ts as i64 + relative_timestamp as i64,
                        })
                    }))
                } else if lacing == 0b11 {
                    let data = match Lacer::Ebml.delace(data) {
                        Ok(frames) => frames,
                        Err(e) => return Output::Once(std::iter::once(Err(e))),
                    };
                    Output::Ebml2(data.into_iter().map(move |d| {
                        Ok(Frame {
                            data: d,
                            is_keyframe: g.reference_block.is_empty(),
                            is_invisible: flag & 0x08 != 0,
                            is_discardable: false,
                            track_number: *track_number,
                            timestamp: cluster_ts as i64 + relative_timestamp as i64,
                        })
                    }))
                } else {
                    let data = match Lacer::FixedSize.delace(data) {
                        Ok(frames) => frames,
                        Err(e) => return Output::Once(std::iter::once(Err(e))),
                    };
                    Output::FixedSize2(data.into_iter().map(move |d| {
                        Ok(Frame {
                            data: d,
                            is_keyframe: g.reference_block.is_empty(),
                            is_invisible: flag & 0x08 != 0,
                            is_discardable: false,
                            track_number: *track_number,
                            timestamp: cluster_ts as i64 + relative_timestamp as i64,
                        })
                    }))
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
