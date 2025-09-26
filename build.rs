use std::env;
use std::fs::File;
use std::io::Write;
use std::path::Path;
fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("generated_types.rs");
    let mut file = File::create(&dest_path).unwrap();

    let content = std::fs::read_to_string("matroska-specification/ebml_matroska.xml").unwrap();
    let doc = roxmltree::Document::parse(&content).unwrap();

    for element in doc
        .descendants()
        .filter(|n| n.has_tag_name("element"))
        .filter(|n| n.attribute("type") != Some("master"))
    {
        let name = element.attribute("name").unwrap();
        let id = element.attribute("id").unwrap();
        let (ty, rep_ty) = match element.attribute("type").unwrap() {
            "uinteger" => ("UnsignedInteger", "u64"),
            "integer" => ("SignedInteger", "i64"),
            "string" => ("Text", "&'static str"),
            "utf-8" => ("Text", "&'static str"),
            "binary" => ("Bin", "&'static [u8]"),
            "float" => ("Float", "f64"),
            "date" => ("Date", "&'static [u8]"), // TODO: DateTime
            other => panic!("Unknown type: {other}"),
        };
        let default_value = element.attribute("default");
        let documentation = element
            .children()
            .find(|n| n.has_tag_name("documentation"))
            .and_then(|n| n.text());

        if let Some(doc) = documentation {
            for line in doc.lines() {
                writeln!(file, "/// {}", line.trim()).unwrap();
            }
        } else {
            writeln!(file, "/// {name} in ebml").unwrap();
        }
        writeln!(file, "pub type {name} = {ty}<{id}>;").unwrap();
        writeln!(file, "impl {name} {{").unwrap();
        if let Some(default) = default_value {
            writeln!(file, "/// Default value for the element if not present.").unwrap();
            if ty == "Float" {
                writeln!(
                    file,
                    "    pub const DEFAULT: Option<{rep_ty}> = Some(hexf::hexf64!(\"{default}\"));"
                )
                .unwrap();
            } else if ty == "Text" {
                writeln!(
                    file,
                    "    pub const DEFAULT: Option<{rep_ty}> = Some(\"{default}\");"
                )
                .unwrap();
            } else {
                writeln!(
                    file,
                    "    pub const DEFAULT: Option<{rep_ty}> = Some({default});"
                )
                .unwrap();
            }
        } else {
            writeln!(file, "/// Does not have a default value").unwrap();
            writeln!(file, "    pub const DEFAULT: Option<{rep_ty}> = None;").unwrap();
        }
        writeln!(file, "}}").unwrap();
    }
}
