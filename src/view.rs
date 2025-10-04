//! A View of a Matroska file, parsing w/o loading clusters into memory.

use crate::master::*;

/// View of a Matroska file, parsing the EBML and Segment headers, but not loading Clusters.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MatroskaView {
    /// The EBML header.
    pub ebml: Ebml,
    /// The Segment views, as there can be multiple segments in a Matroska file.
    pub segment: Vec<SegmentView>,
}

/// View of a Segment, parsing the Segment header, but not loading Clusters.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SegmentView;
