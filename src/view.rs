//! A View of a Matroska file, parsing w/o loading clusters into memory.

use crate::element::Element;
use crate::master::*;

/// View of a Matroska file, parsing the EBML and Segment headers, but not loading Clusters.
#[derive(Debug, Clone, PartialEq)]
pub struct MatroskaView {
    /// The EBML header.
    pub ebml: Ebml,
    /// The Segment views, as there can be multiple segments in a Matroska file.
    pub segment: Vec<SegmentView>,
}

impl MatroskaView {
    /// Create a new MatroskaView by parsing the EBML header and all Segment headers,
    /// but skipping Cluster data to avoid loading it into memory.
    pub fn new<R>(reader: &mut R) -> crate::Result<Self>
    where
        R: std::io::Read + std::io::Seek,
    {
        use crate::io::blocking_impl::*;

        // Read the EBML header
        let ebml = Ebml::read_from(reader)?;

        let mut segments = Vec::new();

        // Parse all segments in the file
        while let Ok(segment) = SegmentView::new(reader) {
            segments.push(segment);
        }

        // At least one segment is required
        if segments.is_empty() {
            return Err(crate::Error::MissingElement(Segment::ID));
        }

        Ok(MatroskaView {
            ebml,
            segment: segments,
        })
    }

    /// Create a new MatroskaView by parsing the EBML header and all Segment headers,
    /// but skipping Cluster data to avoid loading it into memory.
    #[cfg(feature = "tokio")]
    #[cfg_attr(docsrs, doc(cfg(feature = "tokio")))]
    pub async fn new_async<R>(reader: &mut R) -> crate::Result<Self>
    where
        R: tokio::io::AsyncRead + tokio::io::AsyncSeek + Unpin,
    {
        use crate::io::tokio_impl::*;

        // Read the EBML header
        let ebml = Ebml::async_read_from(reader).await?;

        let mut segments = Vec::new();

        // Parse all segments in the file
        while let Ok(segment) = SegmentView::new_async(reader).await {
            segments.push(segment);
        }

        // At least one segment is required
        if segments.is_empty() {
            return Err(crate::Error::MissingElement(Segment::ID));
        }

        Ok(MatroskaView {
            ebml,
            segment: segments,
        })
    }
}

/// View of a Segment, parsing the Segment header, but not loading Clusters.
#[derive(Debug, Clone, PartialEq)]
pub struct SegmentView {
    /// Contains seeking information of Top-Level Elements; see data-layout.
    pub seek_head: Vec<SeekHead>,
    /// Contains general information about the Segment.
    pub info: Info,
    /// A Top-Level Element of information with many tracks described.
    pub tracks: Option<Tracks>,
    /// A Top-Level Element to speed seeking access. All entries are local to the Segment. This Element **SHOULD** be set when the Segment is not transmitted as a live stream (see #livestreaming).
    pub cues: Option<Cues>,
    /// Contain attached files.
    pub attachments: Option<Attachments>,
    /// A system to define basic menus and partition data. For more detailed information, look at the Chapters explanation in chapters.
    pub chapters: Option<Chapters>,
    /// Element containing metadata describing Tracks, Editions, Chapters, Attachments, or the Segment as a whole. A list of valid tags can be found in [Matroska tagging RFC](https://www.matroska.org/technical/tagging.html).
    pub tags: Vec<Tags>,
    /// The position of the first Cluster in the Segment.
    pub first_cluster_position: u64,
    /// The position of the Segment data (after the Segment header).
    pub segment_data_position: u64,
}

impl SegmentView {
    /// Create a new SegmentView by parsing the Segment header and metadata elements,
    /// but skipping Cluster data to avoid loading it into memory.
    pub fn new<R>(reader: &mut R) -> crate::Result<Self>
    where
        R: std::io::Read + std::io::Seek,
    {
        use crate::io::blocking_impl::*;

        // Read the Segment header
        let segment_header = crate::base::Header::read_from(reader)?;
        if segment_header.id != Segment::ID {
            return Err(crate::Error::MissingElement(Segment::ID));
        }

        let segment_data_position = reader.stream_position()?;

        let mut seek_head = Vec::new();
        let mut info = None;
        let mut tracks = None;
        let mut cues = None;
        let mut attachments = None;
        let mut chapters = None;
        let mut tags = Vec::new();
        let mut first_cluster_position = None;

        // Parse segment elements
        loop {
            let current_position = reader.stream_position()?;

            // Check if we've reached the end of the segment
            if let Ok(header) = crate::base::Header::read_from(reader) {
                match header.id {
                    SeekHead::ID => {
                        let element = SeekHead::read_element(&header, reader)?;
                        seek_head.push(element);
                    }
                    Info::ID => {
                        let element = Info::read_element(&header, reader)?;
                        info = Some(element);
                    }
                    Tracks::ID => {
                        let element = Tracks::read_element(&header, reader)?;
                        tracks = Some(element);
                    }
                    Cues::ID => {
                        let element = Cues::read_element(&header, reader)?;
                        cues = Some(element);
                    }
                    Attachments::ID => {
                        let element = Attachments::read_element(&header, reader)?;
                        attachments = Some(element);
                    }
                    Chapters::ID => {
                        let element = Chapters::read_element(&header, reader)?;
                        chapters = Some(element);
                    }
                    Tags::ID => {
                        let element = Tags::read_element(&header, reader)?;
                        tags.push(element);
                    }
                    Cluster::ID => {
                        // Found the first cluster, record its position and stop parsing
                        if first_cluster_position.is_none() {
                            first_cluster_position = Some(current_position);
                        }
                        break;
                    }
                    _ => {
                        use log::warn;
                        use std::io::Read;
                        // Skip unknown elements, here we read and discard the data for efficiency
                        std::io::copy(&mut reader.take(*header.size), &mut std::io::sink())?;
                        warn!("Skipped unknown element with ID: {}", header.id);
                    }
                }
            } else {
                // End of stream or error reading header
                break;
            }
        }

        // Info is required in a valid Matroska file
        let info = info.ok_or(crate::Error::MissingElement(Info::ID))?;

        Ok(SegmentView {
            seek_head,
            info,
            tracks,
            cues,
            attachments,
            chapters,
            tags,
            first_cluster_position: first_cluster_position.unwrap_or(0),
            segment_data_position,
        })
    }

    /// Create a new SegmentView by parsing the Segment header and metadata elements,
    /// but skipping Cluster data to avoid loading it into memory.
    #[cfg(feature = "tokio")]
    #[cfg_attr(docsrs, doc(cfg(feature = "tokio")))]
    pub async fn new_async<R>(reader: &mut R) -> crate::Result<Self>
    where
        R: tokio::io::AsyncRead + tokio::io::AsyncSeek + Unpin,
    {
        use crate::io::tokio_impl::*;
        use tokio::io::AsyncSeekExt;

        // Read the Segment header
        let segment_header = crate::base::Header::async_read_from(reader).await?;
        if segment_header.id != Segment::ID {
            return Err(crate::Error::MissingElement(Segment::ID));
        }

        let segment_data_position = reader.stream_position().await?;

        let mut seek_head = Vec::new();
        let mut info = None;
        let mut tracks = None;
        let mut cues = None;
        let mut attachments = None;
        let mut chapters = None;
        let mut tags = Vec::new();
        let mut first_cluster_position = None;

        // Parse segment elements
        loop {
            let current_position = reader.stream_position().await?;

            // Check if we've reached the end of the segment
            if let Ok(header) = crate::base::Header::async_read_from(reader).await {
                match header.id {
                    SeekHead::ID => {
                        let element = SeekHead::async_read_element(&header, reader).await?;
                        seek_head.push(element);
                    }
                    Info::ID => {
                        let element = Info::async_read_element(&header, reader).await?;
                        info = Some(element);
                    }
                    Tracks::ID => {
                        let element = Tracks::async_read_element(&header, reader).await?;
                        tracks = Some(element);
                    }
                    Cues::ID => {
                        let element = Cues::async_read_element(&header, reader).await?;
                        cues = Some(element);
                    }
                    Attachments::ID => {
                        let element = Attachments::async_read_element(&header, reader).await?;
                        attachments = Some(element);
                    }
                    Chapters::ID => {
                        let element = Chapters::async_read_element(&header, reader).await?;
                        chapters = Some(element);
                    }
                    Tags::ID => {
                        let element = Tags::async_read_element(&header, reader).await?;
                        tags.push(element);
                    }
                    Cluster::ID => {
                        // Found the first cluster, record its position and stop parsing
                        if first_cluster_position.is_none() {
                            first_cluster_position = Some(current_position);
                        }
                        break;
                    }
                    _ => {
                        use log::warn;
                        use tokio::io::AsyncReadExt;
                        // Skip unknown elements, here we read and discard the data for efficiency
                        tokio::io::copy(&mut reader.take(*header.size), &mut tokio::io::sink())
                            .await?;
                        warn!("Skipped unknown element with ID: {}", header.id);
                    }
                }
            } else {
                // End of stream or error reading header
                break;
            }
        }

        // Info is required in a valid Matroska file
        let info = info.ok_or(crate::Error::MissingElement(Info::ID))?;

        Ok(SegmentView {
            seek_head,
            info,
            tracks,
            cues,
            attachments,
            chapters,
            tags,
            first_cluster_position: first_cluster_position.unwrap_or(0),
            segment_data_position,
        })
    }
}
