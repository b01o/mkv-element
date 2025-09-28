use crate::Error;
use crate::base::*;
use crate::element::*;
use crate::functional::*;
use crate::leaf::*;
use crate::supplement::*;

// A helper for generating nested elements.
/* example:
nested! {
    required: [ EbmlMaxIdLength, EbmlMaxSizeLength ],
    optional: [ EbmlVersion, EbmlReadVersion, DocType, DocTypeVersion, DocTypeReadVersion ],
    multiple: [ ],
};
*/
macro_rules! nested {
    (required: [$($required:ident),*$(,)?], optional: [$($optional:ident),*$(,)?], multiple: [$($multiple:ident),*$(,)?],) => {
        paste::paste! {
            fn decode_body(buf: &mut &[u8]) -> crate::Result<Self> {
                let crc32 = Option::<Crc32>::decode(buf).ok().flatten();
                $( let mut [<$required:snake>] = None;)*
                $( let mut [<$optional:snake>] = None;)*
                $( let mut [<$multiple:snake>] = Vec::new();)*
                let mut void: Option<Void> = None;

                while let Ok(Some(header)) = Option::<Header>::decode(buf) {
                    match header.id {
                        $( $required::ID => {
                            if [<$required:snake>].is_some() {
                                return Err(Error::DuplicateElement { id: header.id, parent: Self::ID });
                            } else {
                                [<$required:snake>] = Some($required::decode_element(&header, buf)?)
                            }
                        } )*
                        $( $optional::ID => {
                            if [<$optional:snake>].is_some() {
                                return Err(Error::DuplicateElement { id: header.id, parent: Self::ID });
                            } else {
                                [<$optional:snake>] = Some($optional::decode_element(&header, buf)?)
                            }
                        } )*
                        $( $multiple::ID => {
                            [<$multiple:snake>].push($multiple::decode_element(&header, buf)?);
                        } )*
                        Void::ID => {
                            let v = Void::decode_element(&header, buf)?;
                            if let Some(previous) = void {
                                void = Some(Void { size: previous.size + v.size });
                            } else {
                                void = Some(v);
                            }
                            log::info!("Skipping Void element in Element {}, size: {}B", Self::ID, *header.size);
                        }
                        _ => {
                            buf.advance(*header.size as usize);
                            log::warn!("Unknown element {}({}b) in Element({})", header.id, *header.size, Self::ID);
                        }
                    }
                }

                if buf.has_remaining() {
                    return Err(Error::ShortRead);
                }

                Ok(Self {
                    crc32,
                    $( [<$required:snake>]: [<$required:snake>].or(if $required::HAS_DEFAULT_VALUE { Some($required::default()) } else { None }).ok_or(Error::MissingElement($required::ID))?, )*
                    $( [<$optional:snake>], )*
                    $( [<$multiple:snake>], )*
                    void,
                })
            }
            fn encode_body<B: BufMut>(&self, buf: &mut B) -> crate::Result<()> {
                self.crc32.encode(buf)?;

                $( self.[<$required:snake>].encode(buf)?; )*
                $( self.[<$optional:snake>].encode(buf)?; )*
                $( self.[<$multiple:snake>].encode(buf)?; )*

                self.void.encode(buf)?;

                Ok(())
            }
        }
    };
}

/// EBML element, the first top-level element in a Matroska file.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Ebml {
    /// Optional CRC-32 element for integrity checking.
    pub crc32: Option<Crc32>,
    /// void element, useful for reserving space during writing.
    pub void: Option<Void>,

    /// EBMLVersion element, indicates the version of EBML used.
    pub ebml_version: Option<EbmlVersion>,
    /// EBMLReadVersion element, indicates the minimum version of EBML required to read the file.
    pub ebml_read_version: Option<EbmlReadVersion>,
    /// EBMLMaxIDLength element, indicates the maximum length of an EBML ID in bytes.
    pub ebml_max_id_length: EbmlMaxIdLength,
    /// EBMLMaxSizeLength element, indicates the maximum length of an EBML size in bytes.
    pub ebml_max_size_length: EbmlMaxSizeLength,
    /// DocType element, indicates the type of document. For Matroska files, this is usually "matroska" or "webm".
    pub doc_type: Option<DocType>,
    /// DocTypeVersion element, indicates the version of the document type.
    pub doc_type_version: Option<DocTypeVersion>,
    /// DocTypeReadVersion element, indicates the minimum version of the document type required to read the file.
    pub doc_type_read_version: Option<DocTypeReadVersion>,
}

impl Element for Ebml {
    const ID: VInt64 = VInt64::from_encoded(0x1A45_DFA3);
    nested! {
        required: [ EbmlMaxIdLength, EbmlMaxSizeLength ],
        optional: [ EbmlVersion, EbmlReadVersion, DocType, DocTypeVersion, DocTypeReadVersion ],
        multiple: [ ],
    }
}

/// The Root Element that contains all other Top-Level Elements; see data-layout.
#[derive(Debug, Clone, PartialEq)]
pub struct Segment {
    /// Optional CRC-32 element for integrity checking.
    pub crc32: Option<Crc32>,
    /// void element, useful for reserving space during writing.
    pub void: Option<Void>,

    /// Contains seeking information of Top-Level Elements; see data-layout.
    pub seek_head: Vec<SeekHead>,
    /// Contains general information about the Segment.
    pub info: Info,
    /// The Top-Level Element containing the (monolithic) Block structure.
    pub cluster: Vec<Cluster>,
    // /// A Top-Level Element of information with many tracks described.
    // pub tracks: Option<Tracks>,
    // /// A Top-Level Element to speed seeking access. All entries are local to the Segment. This Element **SHOULD** be set when the Segment is not transmitted as a live stream (see #livestreaming).
    // pub cues: Option<Cues>,
    // /// Contain attached files.
    // pub attachments: Option<Attachments>,
    // /// A system to define basic menus and partition data. For more detailed information, look at the Chapters explanation in chapters.
    // pub chapters: Option<Chapters>,
    // /// Element containing metadata describing Tracks, Editions, Chapters, Attachments, or the Segment as a whole. A list of valid tags can be found in [Matroska tagging RFC](https://www.matroska.org/technical/tagging.html).
    // pub tags: Vec<Tags>,
}

impl Element for Segment {
    const ID: VInt64 = VInt64::from_encoded(0x18538067);
    nested! {
      required: [ Info ],
      optional: [ ],
      multiple: [ SeekHead, Cluster ],
    //   optional: [ Tracks, Cues, Attachments, Chapters ],
    //   multiple: [ SeekHead, Tags, Cluster ],
    }
}

/// Contains seeking information of Top-Level Elements; see data-layout.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SeekHead {
    /// Optional CRC-32 element for integrity checking.
    pub crc32: Option<Crc32>,
    /// void element, useful for reserving space during writing.
    pub void: Option<Void>,

    /// Contains a single seek entry to an EBML Element.
    pub seek: Vec<Seek>,
}

impl Element for SeekHead {
    const ID: VInt64 = VInt64::from_encoded(0x114D9B74);
    nested! {
      required: [ ],
      optional: [ ],
      multiple: [ Seek ],
    }
}

/// Contains a single seek entry to an EBML Element.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Seek {
    /// Optional CRC-32 element for integrity checking.
    pub crc32: Option<Crc32>,
    /// void element, useful for reserving space during writing.
    pub void: Option<Void>,

    /// The binary EBML ID of a Top-Level Element.
    pub seek_id: SeekId,
    /// The Segment Position (segment-position) of a Top-Level Element.
    pub seek_position: SeekPosition,
}

impl Element for Seek {
    const ID: VInt64 = VInt64::from_encoded(0x4DBB);
    nested! {
      required: [ SeekId, SeekPosition ],
      optional: [ ],
      multiple: [ ],
    }
}

/// Contains general information about the Segment.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Info {
    /// Optional CRC-32 element for integrity checking.
    pub crc32: Option<Crc32>,
    /// void element, useful for reserving space during writing.
    pub void: Option<Void>,

    /// A randomly generated unique ID to identify the Segment amongst many others (128 bits). It is equivalent to a UUID v4 [@!RFC4122] with all bits randomly (or pseudo-randomly) chosen. An actual UUID v4 value, where some bits are not random, **MAY** also be used. If the Segment is a part of a Linked Segment, then this Element is **REQUIRED**. The value of the unique ID **MUST** contain at least one bit set to 1.
    pub segment_uuid: Option<SegmentUuid>,
    /// A filename corresponding to this Segment.
    pub segment_filename: Option<SegmentFilename>,
    /// An ID to identify the previous Segment of a Linked Segment. If the Segment is a part of a Linked Segment that uses Hard Linking (hard-linking), then either the PrevUUID or the NextUUID Element is **REQUIRED**. If a Segment contains a PrevUUID but not a NextUUID, then it **MAY** be considered as the last Segment of the Linked Segment. The PrevUUID **MUST NOT** be equal to the SegmentUUID.
    pub prev_uuid: Option<PrevUuid>,
    /// A filename corresponding to the file of the previous Linked Segment. Provision of the previous filename is for display convenience, but PrevUUID **SHOULD** be considered authoritative for identifying the previous Segment in a Linked Segment.
    pub prev_filename: Option<PrevFilename>,
    /// An ID to identify the next Segment of a Linked Segment. If the Segment is a part of a Linked Segment that uses Hard Linking (hard-linking), then either the PrevUUID or the NextUUID Element is **REQUIRED**. If a Segment contains a NextUUID but not a PrevUUID, then it **MAY** be considered as the first Segment of the Linked Segment. The NextUUID **MUST NOT** be equal to the SegmentUUID.
    pub next_uuid: Option<NextUuid>,
    /// A filename corresponding to the file of the next Linked Segment. Provision of the next filename is for display convenience, but NextUUID **SHOULD** be considered authoritative for identifying the Next Segment.
    pub next_filename: Option<NextFilename>,
    /// A unique ID that all Segments of a Linked Segment **MUST** share (128 bits). It is equivalent to a UUID v4 [@!RFC4122] with all bits randomly (or pseudo-randomly) chosen. An actual UUID v4 value, where some bits are not random, **MAY** also be used. If the Segment Info contains a `ChapterTranslate` element, this Element is **REQUIRED**.
    pub segment_family: Vec<SegmentFamily>,
    /// The mapping between this `Segment` and a segment value in the given Chapter Codec. Chapter Codec may need to address different segments, but they may not know of the way to identify such segment when stored in Matroska. This element and its child elements add a way to map the internal segments known to the Chapter Codec to the Segment IDs in Matroska. This allows remuxing a file with Chapter Codec without changing the content of the codec data, just the Segment mapping.
    pub chapter_translate: Vec<ChapterTranslate>,
    /// Base unit for Segment Ticks and Track Ticks, in nanoseconds. A TimestampScale value of 1000000 means scaled timestamps in the Segment are expressed in milliseconds; see timestamps on how to interpret timestamps.
    pub timestamp_scale: TimestampScale,
    /// Duration of the Segment, expressed in Segment Ticks which is based on TimestampScale; see timestamp-ticks.
    pub duration: Option<Duration>,
    /// The date and time that the Segment was created by the muxing application or library.
    pub date_utc: Option<DateUtc>,
    /// General name of the Segment
    pub title: Option<Title>,
    /// Muxing application or library (example: "libmatroska-0.4.3"). Include the full name of the application or library followed by the version number.
    pub muxing_app: MuxingApp,
    /// Writing application (example: "mkvmerge-0.3.3"). Include the full name of the application followed by the version number.
    pub writing_app: WritingApp,
}

impl Element for Info {
    const ID: VInt64 = VInt64::from_encoded(0x1549A966);
    nested! {
      required: [ TimestampScale, MuxingApp, WritingApp ],
      optional: [ SegmentUuid, SegmentFilename, PrevUuid, PrevFilename, NextUuid, NextFilename, Duration, DateUtc, Title ],
      multiple: [ SegmentFamily, ChapterTranslate ],
    }
}

/// The mapping between this `Segment` and a segment value in the given Chapter Codec. Chapter Codec may need to address different segments, but they may not know of the way to identify such segment when stored in Matroska. This element and its child elements add a way to map the internal segments known to the Chapter Codec to the Segment IDs in Matroska. This allows remuxing a file with Chapter Codec without changing the content of the codec data, just the Segment mapping.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChapterTranslate {
    /// Optional CRC-32 element for integrity checking.
    pub crc32: Option<Crc32>,
    /// void element, useful for reserving space during writing.
    pub void: Option<Void>,

    /// The binary value used to represent this Segment in the chapter codec data. The format depends on the ChapProcessCodecID used; see [ChapProcessCodecID](https://www.matroska.org/technical/elements.html#chapprocesscodecid-element).
    pub chapter_translate_id: ChapterTranslateId,
    /// This `ChapterTranslate` applies to this chapter codec of the given chapter edition(s); see ChapProcessCodecID.
    /// * 0 - Matroska Script,
    /// * 1 - DVD-menu
    pub chapter_translate_codec: ChapterTranslateCodec,
    /// Specify a chapter edition UID on which this `ChapterTranslate` applies. When no `ChapterTranslateEditionUID` is specified in the `ChapterTranslate`, the `ChapterTranslate` applies to all chapter editions found in the Segment using the given `ChapterTranslateCodec`.
    pub chapter_translate_edition_uid: Vec<ChapterTranslateEditionUid>,
}

impl Element for ChapterTranslate {
    const ID: VInt64 = VInt64::from_encoded(0x6924);
    nested! {
        required: [ ChapterTranslateId, ChapterTranslateCodec ],
        optional: [ ],
        multiple: [ ChapterTranslateEditionUid ],
    }
}

/// The Top-Level Element containing the (monolithic) Block structure.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Cluster {
    /// Optional CRC-32 element for integrity checking.
    pub crc32: Option<Crc32>,
    /// void element, useful for reserving space during writing.
    pub void: Option<Void>,

    /// Absolute timestamp of the cluster, expressed in Segment Ticks which is based on TimestampScale; see timestamp-ticks. This element **SHOULD** be the first child element of the Cluster it belongs to, or the second if that Cluster contains a CRC-32 element (crc-32).
    pub timestamp: Timestamp,
    /// The Segment Position of the Cluster in the Segment (0 in live streams). It might help to resynchronise offset on damaged streams.
    pub position: Option<Position>,
    /// Size of the previous Cluster, in octets. Can be useful for backward playing.
    pub prev_size: Option<PrevSize>,
    /// Similar to Block, see [basics](https://www.matroska.org/technical/basics.html#block-structure), but without all the extra information, mostly used to reduced overhead when no extra feature is needed; see basics on SimpleBlock Structure.
    pub simple_block: Vec<SimpleBlock>,
    /// Basic container of information containing a single Block and information specific to that Block.
    pub block_group: Vec<BlockGroup>,
}

impl Element for Cluster {
    const ID: VInt64 = VInt64::from_encoded(0x1F43B675);
    nested! {
      required: [ Timestamp ],
      optional: [ Position, PrevSize ],
      multiple: [ SimpleBlock, BlockGroup ],
    }
}

/// Basic container of information containing a single Block and information specific to that Block.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct BlockGroup {
    /// Optional CRC-32 element for integrity checking.
    pub crc32: Option<Crc32>,
    /// void element, useful for reserving space during writing.
    pub void: Option<Void>,

    /// Block containing the actual data to be rendered and a timestamp relative to the Cluster Timestamp; see [basics](https://www.matroska.org/technical/basics.html#block-structure) on Block Structure.
    pub block: Block,
    /// Contain additional binary data to complete the main one; see Codec BlockAdditions section of [Matroska codec RFC](https://www.matroska.org/technical/codec_specs.html) for more information. An EBML parser that has no knowledge of the Block structure could still see and use/skip these data.
    pub block_additions: Option<BlockAdditions>,
    /// The duration of the Block, expressed in Track Ticks; see timestamp-ticks.
    /// The BlockDuration Element can be useful at the end of a Track to define the duration of the last frame (as there is no subsequent Block available),
    /// or when there is a break in a track like for subtitle tracks.
    /// When not written and with no DefaultDuration, the value is assumed to be the difference between the timestamp of this Block and the timestamp of the next Block in "display" order (not coding order). BlockDuration **MUST** be set if the associated TrackEntry stores a DefaultDuration value.
    pub block_duration: Option<BlockDuration>,
    /// This frame is referenced and has the specified cache priority. In cache only a frame of the same or higher priority can replace this frame. A value of 0 means the frame is not referenced.
    pub reference_priority: ReferencePriority,
    /// A timestamp value, relative to the timestamp of the Block in this BlockGroup, expressed in Track Ticks; see timestamp-ticks. This is used to reference other frames necessary to decode this frame. The relative value **SHOULD** correspond to a valid `Block` this `Block` depends on. Historically Matroska Writer didn't write the actual `Block(s)` this `Block` depends on, but *some* `Block` in the past. The value "0" **MAY** also be used to signify this `Block` cannot be decoded on its own, but without knownledge of which `Block` is necessary. In this case, other `ReferenceBlock` **MUST NOT** be found in the same `BlockGroup`. If the `BlockGroup` doesn't have any `ReferenceBlock` element, then the `Block` it contains can be decoded without using any other `Block` data.
    pub reference_block: Vec<ReferenceBlock>,
    /// The new codec state to use. Data interpretation is private to the codec. This information **SHOULD** always be referenced by a seek entry.
    pub codec_state: Option<CodecState>,
    /// Duration of the silent data added to the Block, expressed in Matroska Ticks -- i.e., in nanoseconds; see timestamp-ticks (padding at the end of the Block for positive value, at the beginning of the Block for negative value). The duration of DiscardPadding is not calculated in the duration of the TrackEntry and **SHOULD** be discarded during playback.
    pub discard_padding: Option<DiscardPadding>,
}

impl Element for BlockGroup {
    const ID: VInt64 = VInt64::from_encoded(0xA0);
    nested! {
      required: [ Block, ReferencePriority ],
      optional: [ BlockAdditions, BlockDuration, CodecState, DiscardPadding ],
      multiple: [ ReferenceBlock ],
    }
}
/// Contain additional binary data to complete the main one; see Codec BlockAdditions section of [Matroska codec RFC](https://www.matroska.org/technical/codec_specs.html) for more information. An EBML parser that has no knowledge of the Block structure could still see and use/skip these data.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct BlockAdditions {
    /// Optional CRC-32 element for integrity checking.
    pub crc32: Option<Crc32>,
    /// void element, useful for reserving space during writing.
    pub void: Option<Void>,

    /// Contain the BlockAdditional and some parameters.
    pub block_more: Vec<BlockMore>,
}

impl Element for BlockAdditions {
    const ID: VInt64 = VInt64::from_encoded(0x75A1);
    nested! {
      required: [ ],
      optional: [ ],
      multiple: [ BlockMore ],
    }
}

/// Contain the BlockAdditional and some parameters.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct BlockMore {
    /// Optional CRC-32 element for integrity checking.
    pub crc32: Option<Crc32>,
    /// void element, useful for reserving space during writing.
    pub void: Option<Void>,

    /// Interpreted by the codec as it wishes (using the BlockAddID).
    pub block_additional: BlockAdditional,
    /// An ID to identify how to interpret the BlockAdditional data; see Codec BlockAdditions section of [Matroska codec RFC](https://www.matroska.org/technical/codec_specs.html) for more information. A value of 1 indicates that the meaning of the BlockAdditional data is defined by the codec. Any other value indicates the meaning of the BlockAdditional data is found in the BlockAddIDType found in the TrackEntry. Each BlockAddID value **MUST** be unique between all BlockMore elements found in a BlockAdditions.To keep MaxBlockAdditionID as low as possible, small values **SHOULD** be used.
    pub block_add_id: BlockAddId,
}

impl Element for BlockMore {
    const ID: VInt64 = VInt64::from_encoded(0xA6);
    nested! {
      required: [ BlockAdditional, BlockAddId ],
      optional: [ ],
      multiple: [ ],
    }
}
