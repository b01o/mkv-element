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
    /// A Top-Level Element of information with many tracks described.
    pub tracks: Option<Tracks>,
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
      // Tracks, Cues, Attachments, Chapters
      optional: [ Tracks ],
      multiple: [ SeekHead, Cluster ],
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

    /// A randomly generated unique ID to identify the Segment amongst many others (128 bits). It is equivalent to a UUID v4 \[@!RFC4122\] with all bits randomly (or pseudo-randomly) chosen. An actual UUID v4 value, where some bits are not random, **MAY** also be used. If the Segment is a part of a Linked Segment, then this Element is **REQUIRED**. The value of the unique ID **MUST** contain at least one bit set to 1.
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
    /// A unique ID that all Segments of a Linked Segment **MUST** share (128 bits). It is equivalent to a UUID v4 \[@!RFC4122\] with all bits randomly (or pseudo-randomly) chosen. An actual UUID v4 value, where some bits are not random, **MAY** also be used. If the Segment Info contains a `ChapterTranslate` element, this Element is **REQUIRED**.
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

/// A Top-Level Element of information with many tracks described.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Tracks {
    /// Optional CRC-32 element for integrity checking.
    pub crc32: Option<Crc32>,
    /// void element, useful for reserving space during writing.
    pub void: Option<Void>,

    /// Describes a track with all Elements.
    pub track_entry: Vec<TrackEntry>,
}

impl Element for Tracks {
    const ID: VInt64 = VInt64::from_encoded(0x1654AE6B);
    nested! {
      required: [ ],
      optional: [ ],
      multiple: [ TrackEntry ],
    }
}

/// Describes a track with all Elements.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct TrackEntry {
    /// Optional CRC-32 element for integrity checking.
    pub crc32: Option<Crc32>,
    /// void element, useful for reserving space during writing.
    pub void: Option<Void>,

    /// The track number as used in the Block Header.
    pub track_number: TrackNumber,
    /// A unique ID to identify the Track.
    pub track_uid: TrackUid,
    /// The `TrackType` defines the type of each frame found in the Track. The value **SHOULD** be stored on 1 octet.
    /// * 1 - video,
    /// * 2 - audio,
    /// * 3 - complex,
    /// * 16 - logo,
    /// * 17 - subtitle,
    /// * 18 - buttons,
    /// * 32 - control,
    /// * 33 - metadata
    pub track_type: TrackType,
    /// Set to 1 if the track is usable. It is possible to turn a not usable track into a usable track using chapter codecs or control tracks.
    pub flag_enabled: FlagEnabled,
    /// Set if that track (audio, video or subs) is eligible for automatic selection by the player; see default-track-selection for more details.
    pub flag_default: FlagDefault,
    /// Applies only to subtitles. Set if that track is eligible for automatic selection by the player if it matches the user's language preference, even if the user's preferences would normally not enable subtitles with the selected audio track; this can be used for tracks containing only translations of foreign-language audio or onscreen text. See default-track-selection for more details.
    pub flag_forced: FlagForced,
    /// Set to 1 if and only if that track is suitable for users with hearing impairments.
    pub flag_hearing_impaired: Option<FlagHearingImpaired>,
    /// Set to 1 if and only if that track is suitable for users with visual impairments.
    pub flag_visual_impaired: Option<FlagVisualImpaired>,
    /// Set to 1 if and only if that track contains textual descriptions of video content.
    pub flag_text_descriptions: Option<FlagTextDescriptions>,
    /// Set to 1 if and only if that track is in the content's original language.
    pub flag_original: Option<FlagOriginal>,
    /// Set to 1 if and only if that track contains commentary.
    pub flag_commentary: Option<FlagCommentary>,
    /// Set to 1 if the track **MAY** contain blocks using lacing. When set to 0 all blocks **MUST** have their lacing flags set to No lacing; see block-lacing on Block Lacing.
    pub flag_lacing: FlagLacing,
    /// Number of nanoseconds per frame, expressed in Matroska Ticks -- i.e., in nanoseconds; see timestamp-ticks (frame in the Matroska sense -- one Element put into a (Simple)Block).
    pub default_duration: Option<DefaultDuration>,
    /// The period between two successive fields at the output of the decoding process, expressed in Matroska Ticks -- i.e., in nanoseconds; see timestamp-ticks. see notes for more information
    pub default_decoded_field_duration: Option<DefaultDecodedFieldDuration>,
    /// The maximum value of BlockAddID (BlockAddID). A value 0 means there is no BlockAdditions (BlockAdditions) for this track.
    pub max_block_addition_id: MaxBlockAdditionId,
    /// Contains elements that extend the track format, by adding content either to each frame, with BlockAddID (BlockAddID), or to the track as a whole with BlockAddIDExtraData.
    pub block_addition_mapping: Vec<BlockAdditionMapping>,
    /// A human-readable track name.
    pub name: Option<Name>,
    /// The language of the track, in the Matroska languages form; see basics on language codes. This Element **MUST** be ignored if the LanguageBCP47 Element is used in the same TrackEntry.
    pub language: Language,
    /// The language of the track, in the \[@!BCP47\] form; see basics on language codes. If this Element is used, then any Language Elements used in the same TrackEntry **MUST** be ignored.
    pub language_bcp47: Option<LanguageBcp47>,
    /// An ID corresponding to the codec, see Matroska codec RFC for more info.
    pub codec_id: CodecId,
    /// Private data only known to the codec.
    pub codec_private: Option<CodecPrivate>,
    /// A human-readable string specifying the codec.
    pub codec_name: Option<CodecName>,
    /// CodecDelay is The codec-built-in delay, expressed in Matroska Ticks -- i.e., in nanoseconds; see timestamp-ticks. It represents the amount of codec samples that will be discarded by the decoder during playback. This timestamp value **MUST** be subtracted from each frame timestamp in order to get the timestamp that will be actually played. The value **SHOULD** be small so the muxing of tracks with the same actual timestamp are in the same Cluster.
    pub codec_delay: CodecDelay,
    /// After a discontinuity, SeekPreRoll is the duration of the data the decoder **MUST** decode before the decoded data is valid, expressed in Matroska Ticks -- i.e., in nanoseconds; see timestamp-ticks.
    pub seek_pre_roll: SeekPreRoll,
    /// The mapping between this `TrackEntry` and a track value in the given Chapter Codec. Chapter Codec may need to address content in specific track, but they may not know of the way to identify tracks in Matroska. This element and its child elements add a way to map the internal tracks known to the Chapter Codec to the track IDs in Matroska. This allows remuxing a file with Chapter Codec without changing the content of the codec data, just the track mapping.
    pub track_translate: Vec<TrackTranslate>,
    /// Video settings.
    pub video: Option<Video>,
    /// Audio settings.
    pub audio: Option<Audio>,
    /// Operation that needs to be applied on tracks to create this virtual track. For more details look at notes.
    pub track_operation: Option<TrackOperation>,
    /// Settings for several content encoding mechanisms like compression or encryption.
    pub content_encodings: Option<ContentEncodings>,
}

impl Element for TrackEntry {
    const ID: VInt64 = VInt64::from_encoded(0xAE);
    nested! {
      required: [ TrackNumber, TrackUid, TrackType, FlagEnabled,
                  FlagDefault, FlagForced, FlagLacing, MaxBlockAdditionId,
                  Language, CodecId, CodecDelay, SeekPreRoll ],
      optional: [ FlagHearingImpaired, FlagVisualImpaired, FlagTextDescriptions,
                  FlagOriginal, FlagCommentary, DefaultDuration,
                  DefaultDecodedFieldDuration, Name, LanguageBcp47, CodecPrivate,
                  CodecName, Video, Audio, TrackOperation, ContentEncodings ],
      multiple: [ BlockAdditionMapping, TrackTranslate ],
    }
}

/// Contains elements that extend the track format, by adding content either to each frame, with BlockAddID (BlockAddID), or to the track as a whole with BlockAddIDExtraData.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct BlockAdditionMapping {
    /// Optional CRC-32 element for integrity checking.
    pub crc32: Option<Crc32>,
    /// void element, useful for reserving space during writing.
    pub void: Option<Void>,

    /// If the track format extension needs content beside frames, the value refers to the BlockAddID (BlockAddID), value being described. To keep MaxBlockAdditionID as low as possible, small values **SHOULD** be used.
    pub block_add_id_value: Option<BlockAddIdValue>,
    /// A human-friendly name describing the type of BlockAdditional data, as defined by the associated Block Additional Mapping.
    pub block_add_id_name: Option<BlockAddIdName>,
    /// Stores the registered identifier of the Block Additional Mapping to define how the BlockAdditional data should be handled. If BlockAddIDType is 0, the BlockAddIDValue and corresponding BlockAddID values **MUST** be 1.
    pub block_add_id_type: BlockAddIdType,
    /// Extra binary data that the BlockAddIDType can use to interpret the BlockAdditional data. The interpretation of the binary data depends on the BlockAddIDType value and the corresponding Block Additional Mapping.
    pub block_add_id_extra_data: Option<BlockAddIdExtraData>,
}
impl Element for BlockAdditionMapping {
    const ID: VInt64 = VInt64::from_encoded(0x41E4);
    nested! {
        required: [ BlockAddIdType ],
        optional: [ BlockAddIdValue, BlockAddIdName, BlockAddIdExtraData ],
        multiple: [ ],
    }
}

/// The mapping between this `TrackEntry` and a track value in the given Chapter Codec. Chapter Codec may need to address content in specific track, but they may not know of the way to identify tracks in Matroska. This element and its child elements add a way to map the internal tracks known to the Chapter Codec to the track IDs in Matroska. This allows remuxing a file with Chapter Codec without changing the content of the codec data, just the track mapping.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TrackTranslate {
    /// Optional CRC-32 element for integrity checking.
    pub crc32: Option<Crc32>,
    /// void element, useful for reserving space during writing.
    pub void: Option<Void>,

    /// The binary value used to represent this `TrackEntry` in the chapter codec data. The format depends on the `ChapProcessCodecID` used; see ChapProcessCodecID.
    pub track_translate_track_id: TrackTranslateTrackId,
    /// This `TrackTranslate` applies to this chapter codec of the given chapter edition(s); see ChapProcessCodecID.
    /// * 0 - Matroska Script,
    /// * 1 - DVD-menu
    pub track_translate_codec: TrackTranslateCodec,
    /// Specify a chapter edition UID on which this `TrackTranslate` applies. When no `TrackTranslateEditionUID` is specified in the `TrackTranslate`, the `TrackTranslate` applies to all chapter editions found in the Segment using the given `TrackTranslateCodec`.
    pub track_translate_edition_uid: Vec<TrackTranslateEditionUid>,
}

impl Element for TrackTranslate {
    const ID: VInt64 = VInt64::from_encoded(0x6624);
    nested! {
        required: [ TrackTranslateTrackId, TrackTranslateCodec ],
        optional: [ ],
        multiple: [ TrackTranslateEditionUid ],
    }
}

/// Video settings.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Video {
    /// Optional CRC-32 element for integrity checking.
    pub crc32: Option<Crc32>,
    /// void element, useful for reserving space during writing.
    pub void: Option<Void>,

    /// Specify whether the video frames in this track are interlaced.
    /// * 0 - undetermined,
    /// * 1 - interlaced,
    /// * 2 - progressive
    pub flag_interlaced: FlagInterlaced,
    /// Specify the field ordering of video frames in this track. If FlagInterlaced is not set to 1, this Element **MUST** be ignored.
    /// * 0 - progressive,
    /// * 1 - tff,
    /// * 2 - undetermined,
    /// * 6 - bff,
    /// * 9 - bff(swapped),
    /// * 14 - tff(swapped)
    pub field_order: FieldOrder,
    /// Stereo-3D video mode. There are some more details in notes.
    /// * 0 - mono,
    /// * 1 - side by side (left eye first),
    /// * 2 - top - bottom (right eye is first),
    /// * 3 - top - bottom (left eye is first),
    /// * 4 - checkboard (right eye is first),
    /// * 5 - checkboard (left eye is first),
    /// * 6 - row interleaved (right eye is first),
    /// * 7 - row interleaved (left eye is first),
    /// * 8 - column interleaved (right eye is first),
    /// * 9 - column interleaved (left eye is first),
    /// * 10 - anaglyph (cyan/red),
    /// * 11 - side by side (right eye first),
    /// * 12 - anaglyph (green/magenta),
    /// * 13 - both eyes laced in one Block (left eye is first),
    /// * 14 - both eyes laced in one Block (right eye is first)
    pub stereo_mode: StereoMode,
    /// Indicate whether the BlockAdditional Element with BlockAddID of "1" contains Alpha data, as defined by to the Codec Mapping for the `CodecID`. Undefined values **SHOULD NOT** be used as the behavior of known implementations is different (considered either as 0 or 1).
    /// * 0 - none,
    /// * 1 - present
    pub alpha_mode: AlphaMode,
    /// Width of the encoded video frames in pixels.
    pub pixel_width: PixelWidth,
    /// Height of the encoded video frames in pixels.
    pub pixel_height: PixelHeight,
    /// The number of video pixels to remove at the bottom of the image.
    pub pixel_crop_bottom: PixelCropBottom,
    /// The number of video pixels to remove at the top of the image.
    pub pixel_crop_top: PixelCropTop,
    /// The number of video pixels to remove on the left of the image.
    pub pixel_crop_left: PixelCropLeft,
    /// The number of video pixels to remove on the right of the image.
    pub pixel_crop_right: PixelCropRight,
    /// Width of the video frames to display. Applies to the video frame after cropping (PixelCrop* Elements). If the DisplayUnit of the same TrackEntry is 0, then the default value for DisplayWidth is equal to PixelWidth - PixelCropLeft - PixelCropRight, else there is no default value.
    pub display_width: Option<DisplayWidth>,
    /// Height of the video frames to display. Applies to the video frame after cropping (PixelCrop* Elements). If the DisplayUnit of the same TrackEntry is 0, then the default value for DisplayHeight is equal to PixelHeight - PixelCropTop - PixelCropBottom, else there is no default value.
    pub display_height: Option<DisplayHeight>,
    /// How DisplayWidth & DisplayHeight are interpreted.
    /// * 0 - pixels,
    /// * 1 - centimeters,
    /// * 2 - inches,
    /// * 3 - display aspect ratio,
    /// * 4 - unknown
    pub display_unit: DisplayUnit,
    /// Specify the uncompressed pixel format used for the Track's data as a FourCC. This value is similar in scope to the biCompression value of AVI's `BITMAPINFO` \[@?AVIFormat\]. There is no definitive list of FourCC values, nor an official registry. Some common values for YUV pixel formats can be found at \[@?MSYUV8\], \[@?MSYUV16\] and \[@?FourCC-YUV\]. Some common values for uncompressed RGB pixel formats can be found at \[@?MSRGB\] and \[@?FourCC-RGB\]. UncompressedFourCC **MUST** be set in TrackEntry, when the CodecID Element of the TrackEntry is set to "V_UNCOMPRESSED".
    pub uncompressed_fourcc: Option<UncompressedFourcc>,
    /// Settings describing the colour format.
    pub colour: Option<Colour>,
    /// Describes the video projection details. Used to render spherical, VR videos or flipping videos horizontally/vertically.
    pub projection: Option<Projection>,
}
impl Element for Video {
    const ID: VInt64 = VInt64::from_encoded(0xE0);
    nested! {
      required: [ FlagInterlaced, FieldOrder, StereoMode, AlphaMode,
                  PixelWidth, PixelHeight, PixelCropBottom, PixelCropTop,
                  PixelCropLeft, PixelCropRight, DisplayUnit ],
      optional: [ DisplayWidth, DisplayHeight, UncompressedFourcc,
                  Colour, Projection ],
      multiple: [ ],
    }
}

/// Settings describing the colour format.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Colour {
    /// Optional CRC-32 element for integrity checking.
    pub crc32: Option<Crc32>,
    /// void element, useful for reserving space during writing.
    pub void: Option<Void>,

    /// The Matrix Coefficients of the video used to derive luma and chroma values from red, green, and blue color primaries. For clarity, the value and meanings for MatrixCoefficients are adopted from Table 4 of ISO/IEC 23001-8:2016 or ITU-T H.273.
    /// * 0 - Identity,
    /// * 1 - ITU-R BT.709,
    /// * 2 - unspecified,
    /// * 3 - reserved,
    /// * 4 - US FCC 73.682,
    /// * 5 - ITU-R BT.470BG,
    /// * 6 - SMPTE 170M,
    /// * 7 - SMPTE 240M,
    /// * 8 - YCoCg,
    /// * 9 - BT2020 Non-constant Luminance,
    /// * 10 - BT2020 Constant Luminance,
    /// * 11 - SMPTE ST 2085,
    /// * 12 - Chroma-derived Non-constant Luminance,
    /// * 13 - Chroma-derived Constant Luminance,
    /// * 14 - ITU-R BT.2100-0
    pub matrix_coefficients: MatrixCoefficients,
    /// Number of decoded bits per channel. A value of 0 indicates that the BitsPerChannel is unspecified.
    pub bits_per_channel: BitsPerChannel,
    /// The amount of pixels to remove in the Cr and Cb channels for every pixel not removed horizontally. Example: For video with 4:2:0 chroma subsampling, the ChromaSubsamplingHorz **SHOULD** be set to 1.
    pub chroma_subsampling_horz: Option<ChromaSubsamplingHorz>,
    /// The amount of pixels to remove in the Cr and Cb channels for every pixel not removed vertically. Example: For video with 4:2:0 chroma subsampling, the ChromaSubsamplingVert **SHOULD** be set to 1.
    pub chroma_subsampling_vert: Option<ChromaSubsamplingVert>,
    /// The amount of pixels to remove in the Cb channel for every pixel not removed horizontally. This is additive with ChromaSubsamplingHorz. Example: For video with 4:2:1 chroma subsampling, the ChromaSubsamplingHorz **SHOULD** be set to 1 and CbSubsamplingHorz **SHOULD** be set to 1.
    pub cb_subsampling_horz: Option<CbSubsamplingHorz>,
    /// The amount of pixels to remove in the Cb channel for every pixel not removed vertically. This is additive with ChromaSubsamplingVert.
    pub cb_subsampling_vert: Option<CbSubsamplingVert>,
    /// How chroma is subsampled horizontally.
    /// * 0 - unspecified,
    /// * 1 - left collocated,
    /// * 2 - half
    pub chroma_siting_horz: ChromaSitingHorz,
    /// How chroma is subsampled vertically.
    /// * 0 - unspecified,
    /// * 1 - top collocated,
    /// * 2 - half
    pub chroma_siting_vert: ChromaSitingVert,
    /// Clipping of the color ranges.
    /// * 0 - unspecified,
    /// * 1 - broadcast range,
    /// * 2 - full range (no clipping),
    /// * 3 - defined by MatrixCoefficients / TransferCharacteristics
    pub range: Range,
    /// The transfer characteristics of the video. For clarity, the value and meanings for TransferCharacteristics are adopted from Table 3 of ISO/IEC 23091-4 or ITU-T H.273.
    /// * 0 - reserved,
    /// * 1 - ITU-R BT.709,
    /// * 2 - unspecified,
    /// * 3 - reserved2,
    /// * 4 - Gamma 2.2 curve - BT.470M,
    /// * 5 - Gamma 2.8 curve - BT.470BG,
    /// * 6 - SMPTE 170M,
    /// * 7 - SMPTE 240M,
    /// * 8 - Linear,
    /// * 9 - Log,
    /// * 10 - Log Sqrt,
    /// * 11 - IEC 61966-2-4,
    /// * 12 - ITU-R BT.1361 Extended Colour Gamut,
    /// * 13 - IEC 61966-2-1,
    /// * 14 - ITU-R BT.2020 10 bit,
    /// * 15 - ITU-R BT.2020 12 bit,
    /// * 16 - ITU-R BT.2100 Perceptual Quantization,
    /// * 17 - SMPTE ST 428-1,
    /// * 18 - ARIB STD-B67 (HLG)
    pub transfer_characteristics: TransferCharacteristics,
    /// The colour primaries of the video. For clarity, the value and meanings for Primaries are adopted from Table 2 of ISO/IEC 23091-4 or ITU-T H.273.
    /// * 0 - reserved,
    /// * 1 - ITU-R BT.709,
    /// * 2 - unspecified,
    /// * 3 - reserved2,
    /// * 4 - ITU-R BT.470M,
    /// * 5 - ITU-R BT.470BG - BT.601 625,
    /// * 6 - ITU-R BT.601 525 - SMPTE 170M,
    /// * 7 - SMPTE 240M,
    /// * 8 - FILM,
    /// * 9 - ITU-R BT.2020,
    /// * 10 - SMPTE ST 428-1,
    /// * 11 - SMPTE RP 432-2,
    /// * 12 - SMPTE EG 432-2,
    /// * 22 - EBU Tech. 3213-E - JEDEC P22 phosphors
    pub primaries: Primaries,
    /// Maximum brightness of a single pixel (Maximum Content Light Level) in candelas per square meter (cd/m^2^).
    pub max_cll: Option<MaxCll>,
    /// Maximum brightness of a single full frame (Maximum Frame-Average Light Level) in candelas per square meter (cd/m^2^).
    pub max_fall: Option<MaxFall>,
    /// SMPTE 2086 mastering data.
    pub mastering_metadata: Option<MasteringMetadata>,
}

impl Element for Colour {
    const ID: VInt64 = VInt64::from_encoded(0x55B0);
    nested! {
      required: [ MatrixCoefficients, BitsPerChannel, ChromaSitingHorz,
                  ChromaSitingVert, Range, TransferCharacteristics, Primaries ],
      optional: [ ChromaSubsamplingHorz, ChromaSubsamplingVert,
                  CbSubsamplingHorz, CbSubsamplingVert, MaxCll,
                  MaxFall, MasteringMetadata ],
      multiple: [ ],
    }
}

/// SMPTE 2086 mastering data.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct MasteringMetadata {
    /// Optional CRC-32 element for integrity checking.
    pub crc32: Option<Crc32>,
    /// void element, useful for reserving space during writing.
    pub void: Option<Void>,

    /// Red X chromaticity coordinate, as defined by \[@!CIE-1931\].
    pub primary_r_chromaticity_x: Option<PrimaryRChromaticityX>,
    /// Red Y chromaticity coordinate, as defined by \[@!CIE-1931\].
    pub primary_r_chromaticity_y: Option<PrimaryRChromaticityY>,
    /// Green X chromaticity coordinate, as defined by \[@!CIE-1931\].
    pub primary_g_chromaticity_x: Option<PrimaryGChromaticityX>,
    /// Green Y chromaticity coordinate, as defined by \[@!CIE-1931\].
    pub primary_g_chromaticity_y: Option<PrimaryGChromaticityY>,
    /// Blue X chromaticity coordinate, as defined by \[@!CIE-1931\].
    pub primary_b_chromaticity_x: Option<PrimaryBChromaticityX>,
    /// Blue Y chromaticity coordinate, as defined by \[@!CIE-1931\].
    pub primary_b_chromaticity_y: Option<PrimaryBChromaticityY>,
    /// White point X chromaticity coordinate, as defined by \[@!CIE-1931\].
    pub white_point_chromaticity_x: Option<WhitePointChromaticityX>,
    /// White point Y chromaticity coordinate, as defined by \[@!CIE-1931\].
    pub white_point_chromaticity_y: Option<WhitePointChromaticityY>,
    /// Maximum luminance. Represented in candelas per square meter (cd/m^2^).
    pub luminance_max: Option<LuminanceMax>,
    /// Minimum luminance. Represented in candelas per square meter (cd/m^2^).
    pub luminance_min: Option<LuminanceMin>,
}

impl Element for MasteringMetadata {
    const ID: VInt64 = VInt64::from_encoded(0x55D0);
    nested! {
      required: [ ],
      optional: [ PrimaryRChromaticityX, PrimaryRChromaticityY,
                 PrimaryGChromaticityX, PrimaryGChromaticityY,
                 PrimaryBChromaticityX, PrimaryBChromaticityY,
                 WhitePointChromaticityX, WhitePointChromaticityY,
                 LuminanceMax, LuminanceMin ],
      multiple: [ ],
    }
}

/// Describes the video projection details. Used to render spherical, VR videos or flipping videos horizontally/vertically.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Projection {
    /// Optional CRC-32 element for integrity checking.
    pub crc32: Option<Crc32>,
    /// void element, useful for reserving space during writing.
    pub void: Option<Void>,

    /// Describes the projection used for this video track.
    /// * 0 - rectangular,
    /// * 1 - equirectangular,
    /// * 2 - cubemap,
    /// * 3 - mesh
    pub projection_type: ProjectionType,
    /// Private data that only applies to a specific projection. * If `ProjectionType` equals 0 (Rectangular), then this element **MUST NOT** be present. * If `ProjectionType` equals 1 (Equirectangular), then this element **MUST** be present and contain the same binary data that would be stored inside an ISOBMFF Equirectangular Projection Box ('equi'). * If `ProjectionType` equals 2 (Cubemap), then this element **MUST** be present and contain the same binary data that would be stored inside an ISOBMFF Cubemap Projection Box ('cbmp'). * If `ProjectionType` equals 3 (Mesh), then this element **MUST** be present and contain the same binary data that would be stored inside an ISOBMFF Mesh Projection Box ('mshp'). ISOBMFF box size and fourcc fields are not included in the binary data, but the FullBox version and flag fields are. This is to avoid redundant framing information while preserving versioning and semantics between the two container formats
    pub projection_private: Option<ProjectionPrivate>,
    /// Specifies a yaw rotation to the projection. Value represents a clockwise rotation, in degrees, around the up vector. This rotation must be applied before any `ProjectionPosePitch` or `ProjectionPoseRoll` rotations. The value of this element **MUST** be in the -180 to 180 degree range, both included. Setting `ProjectionPoseYaw` to 180 or -180 degrees, with the `ProjectionPoseRoll` and `ProjectionPosePitch` set to 0 degrees flips the image horizontally.
    pub projection_pose_yaw: ProjectionPoseYaw,
    /// Specifies a pitch rotation to the projection. Value represents a counter-clockwise rotation, in degrees, around the right vector. This rotation must be applied after the `ProjectionPoseYaw` rotation and before the `ProjectionPoseRoll` rotation. The value of this element **MUST** be in the -90 to 90 degree range, both included.
    pub projection_pose_pitch: ProjectionPosePitch,
    /// Specifies a roll rotation to the projection. Value represents a counter-clockwise rotation, in degrees, around the forward vector. This rotation must be applied after the `ProjectionPoseYaw` and `ProjectionPosePitch` rotations. The value of this element **MUST** be in the -180 to 180 degree range, both included. Setting `ProjectionPoseRoll` to 180 or -180 degrees, the `ProjectionPoseYaw` to 180 or -180 degrees with `ProjectionPosePitch` set to 0 degrees flips the image vertically. Setting `ProjectionPoseRoll` to 180 or -180 degrees, with the `ProjectionPoseYaw` and `ProjectionPosePitch` set to 0 degrees flips the image horizontally and vertically.
    pub projection_pose_roll: ProjectionPoseRoll,
}

impl Element for Projection {
    const ID: VInt64 = VInt64::from_encoded(0x7670);
    nested! {
      required: [ ProjectionType, ProjectionPoseYaw, ProjectionPosePitch, ProjectionPoseRoll ],
      optional: [ ProjectionPrivate ],
      multiple: [ ],
    }
}

/// Audio settings.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Audio {
    /// Optional CRC-32 element for integrity checking.
    pub crc32: Option<Crc32>,
    /// void element, useful for reserving space during writing.
    pub void: Option<Void>,

    /// Sampling frequency in Hz.
    pub sampling_frequency: SamplingFrequency,
    /// Real output sampling frequency in Hz (used for SBR techniques). The default value for OutputSamplingFrequency of the same TrackEntry is equal to the SamplingFrequency.
    pub output_sampling_frequency: Option<OutputSamplingFrequency>,
    /// Numbers of channels in the track.
    pub channels: Channels,
    /// Bits per sample, mostly used for PCM.
    pub bit_depth: Option<BitDepth>,
    /// Audio emphasis applied on audio samples. The player **MUST** apply the inverse emphasis to get the proper audio samples.
    /// * 0 - No emphasis,
    /// * 1 - CD audio,
    /// * 2 - reserved,
    /// * 3 - CCIT J.17,
    /// * 4 - FM 50,
    /// * 5 - FM 75,
    /// * 10 - Phono RIAA,
    /// * 11 - Phono IEC N78,
    /// * 12 - Phono TELDEC,
    /// * 13 - Phono EMI,
    /// * 14 - Phono Columbia LP,
    /// * 15 - Phono LONDON,
    /// * 16 - Phono NARTB
    pub emphasis: Emphasis,
}

impl Element for Audio {
    const ID: VInt64 = VInt64::from_encoded(0xE1);
    nested! {
      required: [ SamplingFrequency, Channels, Emphasis ],
      optional: [ OutputSamplingFrequency, BitDepth ],
      multiple: [ ],
    }
}

/// Operation that needs to be applied on tracks to create this virtual track. For more details look at notes.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TrackOperation {
    /// Optional CRC-32 element for integrity checking.
    pub crc32: Option<Crc32>,
    /// void element, useful for reserving space during writing.
    pub void: Option<Void>,

    /// Contains the list of all video plane tracks that need to be combined to create this 3D track
    pub track_combine_planes: Option<TrackCombinePlanes>,
    /// Contains the list of all tracks whose Blocks need to be combined to create this virtual track
    pub track_join_blocks: Option<TrackJoinBlocks>,
}

impl Element for TrackOperation {
    const ID: VInt64 = VInt64::from_encoded(0xE2);
    nested! {
      required: [ ],
      optional: [ TrackCombinePlanes, TrackJoinBlocks ],
      multiple: [ ],
    }
}

/// Contains the list of all video plane tracks that need to be combined to create this 3D track
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TrackCombinePlanes {
    /// Optional CRC-32 element for integrity checking.
    pub crc32: Option<Crc32>,
    /// void element, useful for reserving space during writing.
    pub void: Option<Void>,

    /// Contains a video plane track that need to be combined to create this 3D track
    pub track_plane: Vec<TrackPlane>,
}

impl Element for TrackCombinePlanes {
    const ID: VInt64 = VInt64::from_encoded(0xE3);
    nested! {
        required: [ ],
        optional: [ ],
        multiple: [ TrackPlane ],
    }
}

/// Contains a video plane track that need to be combined to create this 3D track
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TrackPlane {
    /// Optional CRC-32 element for integrity checking.
    pub crc32: Option<Crc32>,
    /// void element, useful for reserving space during writing.
    pub void: Option<Void>,

    /// The trackUID number of the track representing the plane.
    pub track_plane_uid: TrackPlaneUid,
    /// The kind of plane this track corresponds to.
    /// * 0 - left eye,
    /// * 1 - right eye,
    /// * 2 - background
    pub track_plane_type: TrackPlaneType,
}

impl Element for TrackPlane {
    const ID: VInt64 = VInt64::from_encoded(0xE4);
    nested! {
        required: [ TrackPlaneUid, TrackPlaneType ],
        optional: [ ],
        multiple: [ ],
    }
}

/// Contains the list of all tracks whose Blocks need to be combined to create this virtual track
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TrackJoinBlocks {
    /// Optional CRC-32 element for integrity checking.
    pub crc32: Option<Crc32>,
    /// void element, useful for reserving space during writing.
    pub void: Option<Void>,

    /// The trackUID number of a track whose blocks are used to create this virtual track.
    pub track_join_uid: Vec<TrackJoinUid>,
}

impl Element for TrackJoinBlocks {
    const ID: VInt64 = VInt64::from_encoded(0xE9);
    nested! {
        required: [ ],
        optional: [ ],
        multiple: [ TrackJoinUid ],
    }
}

/// Settings for several content encoding mechanisms like compression or encryption.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ContentEncodings {
    /// Optional CRC-32 element for integrity checking.
    pub crc32: Option<Crc32>,
    /// void element, useful for reserving space during writing.
    pub void: Option<Void>,

    /// Settings for one content encoding like compression or encryption.
    pub content_encoding: Vec<ContentEncoding>,
}

impl Element for ContentEncodings {
    const ID: VInt64 = VInt64::from_encoded(0x6D80);
    nested! {
        required: [ ],
        optional: [ ],
        multiple: [ ContentEncoding ],
    }
}

/// Settings for one content encoding like compression or encryption.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ContentEncoding {
    /// Optional CRC-32 element for integrity checking.
    pub crc32: Option<Crc32>,
    /// void element, useful for reserving space during writing.
    pub void: Option<Void>,

    /// Tell in which order to apply each `ContentEncoding` of the `ContentEncodings`. The decoder/demuxer **MUST** start with the `ContentEncoding` with the highest `ContentEncodingOrder` and work its way down to the `ContentEncoding` with the lowest `ContentEncodingOrder`. This value **MUST** be unique over for each `ContentEncoding` found in the `ContentEncodings` of this `TrackEntry`.
    pub content_encoding_order: ContentEncodingOrder,
    /// A bit field that describes which Elements have been modified in this way. Values (big-endian) can be OR'ed.
    /// * 1 - Block,
    /// * 2 - Private,
    /// * 4 - Next
    pub content_encoding_scope: ContentEncodingScope,
    /// A value describing what kind of transformation is applied.
    /// * 0 - Compression,
    /// * 1 - Encryption
    pub content_encoding_type: ContentEncodingType,
    /// Settings describing the compression used. This Element **MUST** be present if the value of ContentEncodingType is 0 and absent otherwise. Each block **MUST** be decompressable even if no previous block is available in order not to prevent seeking.
    pub content_compression: Option<ContentCompression>,
    /// Settings describing the encryption used. This Element **MUST** be present if the value of `ContentEncodingType` is 1 (encryption) and **MUST** be ignored otherwise. A Matroska Player **MAY** support encryption.
    pub content_encryption: Option<ContentEncryption>,
}

impl Element for ContentEncoding {
    const ID: VInt64 = VInt64::from_encoded(0x6240);
    nested! {
        required: [ ContentEncodingOrder, ContentEncodingScope, ContentEncodingType ],
        optional: [ ContentCompression, ContentEncryption ],
        multiple: [ ],
    }
}

/// Settings describing the compression used. This Element **MUST** be present if the value of ContentEncodingType is 0 and absent otherwise. Each block **MUST** be decompressable even if no previous block is available in order not to prevent seeking.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ContentCompression {
    /// Optional CRC-32 element for integrity checking.
    pub crc32: Option<Crc32>,
    /// void element, useful for reserving space during writing.
    pub void: Option<Void>,

    /// The compression algorithm used. Compression method "1" (bzlib) and "2" (lzo1x) are lacking proper documentation on the format which limits implementation possibilities. Due to licensing conflicts on commonly available libraries compression methods "2" (lzo1x) does not offer widespread interoperability. A Matroska Writer **SHOULD NOT** use these compression methods by default. A Matroska Reader **MAY** support methods "1" and "2" as possible, and **SHOULD** support other methods.
    /// * 0 - zlib,
    /// * 1 - bzlib,
    /// * 2 - lzo1x,
    /// * 3 - Header Stripping
    pub content_comp_algo: ContentCompAlgo,

    /// Settings that might be needed by the decompressor. For Header Stripping (`ContentCompAlgo`=3), the bytes that were removed from the beginning of each frames of the track.
    pub content_comp_settings: Option<ContentCompSettings>,
}
impl Element for ContentCompression {
    const ID: VInt64 = VInt64::from_encoded(0x5034);
    nested! {
        required: [ ContentCompAlgo ],
        optional: [ ContentCompSettings ],
        multiple: [ ],
    }
}

/// Settings describing the encryption used. This Element **MUST** be present if the value of `ContentEncodingType` is 1 (encryption) and **MUST** be ignored otherwise. A Matroska Player **MAY** support encryption.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ContentEncryption {
    /// Optional CRC-32 element for integrity checking.
    pub crc32: Option<Crc32>,
    /// void element, useful for reserving space during writing.
    pub void: Option<Void>,

    /// The encryption algorithm used.
    /// * 0 - Not encrypted,
    /// * 1 - DES,
    /// * 2 - 3DES,
    /// * 3 - Twofish,
    /// * 4 - Blowfish,
    /// * 5 - AES
    pub content_enc_algo: ContentEncAlgo,
    /// For public key algorithms this is the ID of the public key the the data was encrypted with.
    pub content_enc_key_id: Option<ContentEncKeyId>,
    /// Settings describing the encryption algorithm used.
    pub content_enc_aes_settings: Option<ContentEncAesSettings>,
}
impl Element for ContentEncryption {
    const ID: VInt64 = VInt64::from_encoded(0x5035);
    nested! {
        required: [ ContentEncAlgo ],
        optional: [ ContentEncKeyId, ContentEncAesSettings ],
        multiple: [ ],
    }
}

/// Settings describing the encryption algorithm used.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ContentEncAesSettings {
    /// Optional CRC-32 element for integrity checking.
    pub crc32: Option<Crc32>,
    /// void element, useful for reserving space during writing.
    pub void: Option<Void>,

    /// The AES cipher mode used in the encryption.
    /// * 1 - AES-CTR,
    /// * 2 - AES-CBC
    pub aes_settings_cipher_mode: AesSettingsCipherMode,
}

impl Element for ContentEncAesSettings {
    const ID: VInt64 = VInt64::from_encoded(0x47E7);
    nested! {
        required: [ AesSettingsCipherMode ],
        optional: [ ],
        multiple: [ ],
    }
}
