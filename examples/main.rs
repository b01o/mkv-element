use std::ops::Deref;

use mkv_element::base_type::VInt64;


fn main() {
    let v = VInt64::from_encoded(0x42F2);
    println!("v: 0x{v}");



    return ;
    // let file = std::fs::File::open("matroska-specification/ebml_matroska.xml").unwrap();
    let content = std::fs::read_to_string("matroska-specification/ebml_matroska.xml").unwrap();
    let doc = roxmltree::Document::parse(&content).unwrap();

     doc.descendants()
        .filter(|n| n.has_tag_name("element"))
        .filter(|n| n.attribute("type") != Some("master"))
        .for_each(|n|{
        let name = n.attribute("name").unwrap();
        let id = n.attribute("id").unwrap();
        let ty = match n.attribute("type").unwrap() {
            "uinteger" => "UnsignedInteger",
            "integer" => "SignedInteger",
            "string" => "Text",
            "utf-8" => "Text",
            "binary" => "Bin",
            "float" => "Float",
            "date" => "Date",
            other => panic!("Unknown type: {other}"),
        };
        let default = n.attribute("default");

        let documentation = n.children().find(|n| n.has_tag_name("documentation")).and_then(|n| n.text());

                println!("{name},\t{id},\t{ty},\t{default:?}: {documentation:?}");

        });

    let n = doc.descendants()
        .filter(|n| n.has_tag_name("element"))
        .filter(|n| n.attribute("type") != Some("master"))
        .next().unwrap();

        let name = n.attribute("name").unwrap();
        let id = n.attribute("id").unwrap();
        let ty = match n.attribute("type").unwrap() {
            "uinteger" => "UnsignedInteger",
            "integer" => "SignedInteger",
            "string" => "Text",
            "utf-8" => "Text",
            "binary" => "Bin",
            "float" => "Float",
            "date" => "Date", // TODO: DateTime
            other => panic!("Unknown type: {other}"),
        };
        let default = n.attribute("default");

                // println!("{name},\t{id},\t{ty},\t{default:?}");

    //  println!("pub type {} = {};", name, ty);


}

