use mkv_element::prelude::*;

#[test]
fn read_ebml() {
    use mkv_element::io::blocking::*;
    let ebml_hex = [
        0x1a, 0x45, 0xDF, 0xA3, 0x93, 0x42, 0x82, 0x88, 0x6D, 0x61, 0x74, 0x72, 0x6F, 0x73, 0x6B,
        0x61, 0x42, 0x87, 0x81, 0x01, 0x42, 0x85, 0x81, 0x01,
    ];
    let mut ebml_hex = std::io::Cursor::new(ebml_hex);
    let ebml = Ebml::read_from(&mut ebml_hex).unwrap();
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

#[test]
fn write_ebml() {
    use mkv_element::io::blocking::*;
    let ebml = Ebml {
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
    let mut ebml_buf = Vec::new();
    ebml.write_to(&mut ebml_buf).unwrap();
    let ebml_read = Ebml::read_from(&mut &ebml_buf[..]).unwrap();
    assert_eq!(ebml, ebml_read);
}

#[tokio::test]
async fn read_ebml_tokio() {
    use mkv_element::io::tokio_impl::*;

    let ebml_hex = [
        0x1a, 0x45, 0xDF, 0xA3, 0x93, 0x42, 0x82, 0x88, 0x6D, 0x61, 0x74, 0x72, 0x6F, 0x73, 0x6B,
        0x61, 0x42, 0x87, 0x81, 0x01, 0x42, 0x85, 0x81, 0x01,
    ];
    let mut ebml_hex = std::io::Cursor::new(ebml_hex);
    let ebml = Ebml::async_read_from(&mut ebml_hex).await.unwrap();
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

#[tokio::test]
async fn write_ebml_tokio() {
    use mkv_element::io::tokio_impl::*;
    let ebml = Ebml {
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
    let mut ebml_buf = Vec::new();
    ebml.async_write_to(&mut ebml_buf).await.unwrap();
    let ebml_read = Ebml::async_read_from(&mut &ebml_buf[..]).await.unwrap();
    assert_eq!(ebml, ebml_read);
}
