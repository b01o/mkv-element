use mkv_element::{
    io::ReadFrom,
    master::{Ebml, Segment},
};

// test1-tag.xml
//
// <?xml version="1.0" encoding="ISO-8859-1"?>
// <!DOCTYPE Tags SYSTEM "matroskatags.dtd">
// <Tags>
//   <!-- movie -->
//   <Tag>
//     <Targets>
//       <TargetTypeValue>50</TargetTypeValue>
//     </Targets>
//     <Simple>
//       <Name>TITLE</Name>
//       <String>Big Buck Bunny - test 1</String>
//     </Simple>
//     <Simple>
//       <Name>DATE_RELEASED</Name>
//       <String>2010</String>
//     </Simple>
//     <Simple>
//       <Name>COMMENT</Name>
//       <String>Matroska Validation File1, basic MPEG4.2 and MP3 with only SimpleBlock</String>
//     </Simple>
//   </Tag>
// </Tags>
#[test]
fn ietf_test_1() {
    let mut file = std::fs::File::open("matroska-test-files/test_files/test1.mkv").unwrap();
    let _ebml_head = Ebml::read_from(&mut file).unwrap();
    let segment = Segment::read_from(&mut file).unwrap();
    let tags = segment.tags.first().unwrap();
    let tag = tags.tag.first().unwrap();
    let target_tag = tag.targets.target_type_value;
    assert_eq!(*target_tag, 50);
    let title = tag
        .simple_tag
        .iter()
        .find(|s| &*s.tag_name == "TITLE")
        .map(|s| s.tag_string.as_deref());
    assert_eq!(title, Some(Some("Big Buck Bunny - test 1")));
    let date_released = tag
        .simple_tag
        .iter()
        .find(|s| &*s.tag_name == "DATE_RELEASED")
        .map(|s| s.tag_string.as_deref());
    assert_eq!(date_released, Some(Some("2010")));
    let comment = tag
        .simple_tag
        .iter()
        .find(|s| &*s.tag_name == "COMMENT")
        .map(|s| s.tag_string.as_deref());
    assert_eq!(
        comment,
        Some(Some(
            "Matroska Validation File1, basic MPEG4.2 and MP3 with only SimpleBlock"
        ))
    );

    assert!(
        segment.cluster.iter().all(|c| c.block_group.is_empty()),
        "All clusters should use SimpleBlock only"
    );
}

// <?xml version="1.0" encoding="ISO-8859-1"?>
// <!DOCTYPE Tags SYSTEM "matroskatags.dtd">
// <Tags>
//   <!-- movie -->
//   <Tag>
//     <Targets>
//       <TargetTypeValue>50</TargetTypeValue>
//     </Targets>
//     <Simple>
//       <Name>TITLE</Name>
//       <String>Elephant Dream - test 2</String>
//     </Simple>
//     <Simple>
//       <Name>DATE_RELEASED</Name>
//       <String>2010</String>
//     </Simple>
//     <Simple>
//       <Name>COMMENT</Name>
//       <String>Matroska Validation File 2, 100,000 timecode scale, odd aspect ratio, and CRC-32. Codecs are AVC and AAC</String>
//     </Simple>
//   </Tag>
// </Tags>
#[test]
fn ietf_test_2() {
    let mut file = std::fs::File::open("matroska-test-files/test_files/test2.mkv").unwrap();
    let _ebml_head = Ebml::read_from(&mut file).unwrap();
    let segment = Segment::read_from(&mut file).unwrap();
    let tags = segment.tags.first().unwrap();
    let tag = tags.tag.first().unwrap();
    let target_tag = tag.targets.target_type_value;
    assert_eq!(*target_tag, 50);
    let title = tag
        .simple_tag
        .iter()
        .find(|s| &*s.tag_name == "TITLE")
        .map(|s| s.tag_string.as_deref());
    assert_eq!(title, Some(Some("Elephant Dream - test 2")));
    let date_released = tag
        .simple_tag
        .iter()
        .find(|s| &*s.tag_name == "DATE_RELEASED")
        .map(|s| s.tag_string.as_deref());
    assert_eq!(date_released, Some(Some("2010")));
    let comment = tag
        .simple_tag
        .iter()
        .find(|s| &*s.tag_name == "COMMENT")
        .map(|s| s.tag_string.as_deref());
    assert_eq!(
        comment,
        Some(Some(
            "Matroska Validation File 2, 100,000 timecode scale, odd aspect ratio, and CRC-32. Codecs are AVC and AAC"
        ))
    );
    assert!(*segment.info.timestamp_scale == 100_000);
}
