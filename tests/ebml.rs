use mkv_element::{coding::*, leaf::*, master::*};

#[test]
fn read_ebml() {
    let ebml_hex = [
        0x1a, 0x45, 0xDF, 0xA3, 0x93, 0x42, 0x82, 0x88, 0x6D, 0x61, 0x74, 0x72, 0x6F, 0x73, 0x6B,
        0x61, 0x42, 0x87, 0x81, 0x01, 0x42, 0x85, 0x81, 0x01,
    ];
    let ebml = Ebml::decode(&mut ebml_hex.as_slice()).unwrap();
    let ebml_expected = Ebml {
        crc32: None,
        ebml_version: None,
        ebml_read_version: None,
        ebml_max_id_length: EbmlMaxIdLength(4),
        ebml_max_size_length: EbmlMaxSizeLength(8),
        doc_type: Some(DocType("matroska".to_string())),
        doc_type_version: Some(DocTypeVersion(1)),
        doc_type_read_version: Some(DocTypeReadVersion(1)),
        void: None,
    };
    assert_eq!(ebml, ebml_expected);
}
