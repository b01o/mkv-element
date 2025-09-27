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
                let crc32 = Option::<Crc32>::decode(buf)?;
                $( let mut [<$required:snake>] = None;)*
                $( let mut [<$optional:snake>] = None;)*
                $( let mut [<$multiple:snake>] = Vec::new();)*

                while let Ok(Some(header)) = Option::<Header>::decode(buf) {
                    match header.id {
                        $( $required::ID => {
                            if [<$required:snake>].is_some() {
                                return Err(Error::DuplicateElement { id: header.id, parent: Self::ID });
                            } else {
                                [<$required:snake>] = Some($required::decode(buf)?)
                            }
                        } )*
                        $( $optional::ID => {
                            if [<$optional:snake>].is_some() {
                                return Err(Error::DuplicateElement { id: header.id, parent: Self::ID });
                            } else {
                                [<$optional:snake>] = Some($optional::decode(buf)?)
                            }
                        } )*
                        $( $multiple::ID => {
                            [<$multiple:snake>].push($multiple::decode(buf)?);
                        } )*
                        Void::ID => {
                            buf.advance(*header.size as usize);
                            log::info!("Skipping Void element in Element {}, size: {}B", Self::ID, *header.size);
                        }
                        _ => {
                            buf.advance(*header.size as usize);
                            log::warn!("Unknown element {}({}b) in Element({})", header.id, *header.size, Self::ID);
                            // return Err(Error::UnknownElement { id: header.id, size: *header.size, parent: Self::ID });
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
                })
            }
            fn encode_body<B: BufMut>(&self, buf: &mut B) -> crate::Result<()> {
                $( self.[<$required:snake>].encode(buf)?; )*
                $( self.[<$optional:snake>].encode(buf)?; )*
                $( self.[<$multiple:snake>].encode(buf)?; )*
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
