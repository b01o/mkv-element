use std::env;
use std::fs::File;
use std::io::Write;
use std::path::Path;

// ref:
//  #[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
//     pub struct UnsignedInteger(pub u64);
//     impl Deref for UnsignedInteger {
//         type Target = u64;
//         fn deref(&self) -> &Self::Target {
//             &self.0
//         }
//     }

//     impl Element for UnsignedInteger {
//         const ID: VInt64 = VInt64::from_encoded(0x12);
//         fn decode_body(buf: &mut &[u8]) -> crate::Result<Self> {
//             if buf.is_empty() {
//                 return Ok(Self(0));
//             }
//             if buf.len() > 8 {
//                 return Err(crate::Error::UnderDecode(Self::ID));
//             }
//             let len = buf.len().min(8);
//             let mut value = [0u8; 8];
//             value[8 - len..].copy_from_slice(&buf[..len]);
//             buf.advance(len);
//             Ok(Self(u64::from_be_bytes(value)))
//         }
//         fn encode_body<B: crate::functional::BufMut>(&self, buf: &mut B) -> crate::Result<()> {
//             let bytes = self.0.to_be_bytes();
//             let first_non_zero = bytes
//                 .iter()
//                 .position(|&b| b != 0)
//                 .unwrap_or(bytes.len() - 1);
//             buf.append_slice(&bytes[first_non_zero..]);
//             Ok(())
//         }
//     }
fn unsigned(file: &mut File, name: &str, id: &str, default: Option<&str>) {
    writeln!(
        file,
        "#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]"
    )
    .unwrap();
    writeln!(file, "pub struct {name}(pub u64);").unwrap();
    // Implement Deref to u64
    writeln!(file, "impl std::ops::Deref for {name} {{ type Target = u64; fn deref(&self) -> &Self::Target {{ &self.0 }} }}").unwrap();

    // Implement Element
    writeln!(file, "impl Element for {name} {{").unwrap();
    writeln!(file, "    const ID: VInt64 = VInt64::from_encoded({id});").unwrap();
    writeln!(
        file,
        "    fn decode_body(buf: &mut &[u8]) -> crate::Result<Self> {{"
    )
    .unwrap();
    writeln!(file, "        if buf.is_empty() {{").unwrap();

    if let Some(default_value) = default {
        writeln!(file, "            return Ok(Self({default_value}));").unwrap();
    } else {
        writeln!(file, "            return Ok(Self(0));").unwrap();
    }

    writeln!(file, "        }}").unwrap();
    writeln!(file, "        if buf.len() > 8 {{").unwrap();
    writeln!(
        file,
        "            return Err(crate::Error::UnderDecode(Self::ID));"
    )
    .unwrap();
    writeln!(file, "        }}").unwrap();
    writeln!(file, "        let len = buf.len().min(8);").unwrap();
    writeln!(file, "        let mut value = [0u8; 8];").unwrap();
    writeln!(
        file,
        "        value[8 - len..].copy_from_slice(&buf[..len]);"
    )
    .unwrap();
    writeln!(file, "        buf.advance(len);").unwrap();
    writeln!(file, "        Ok(Self(u64::from_be_bytes(value)))").unwrap();
    writeln!(file, "    }}").unwrap();
    writeln!(file, "    fn encode_body<B: crate::functional::BufMut>(&self, buf: &mut B) -> crate::Result<()> {{").unwrap();
    writeln!(file, "        let bytes = self.0.to_be_bytes();").unwrap();
    writeln!(file, "        let first_non_zero = bytes.iter().position(|&b| b != 0).unwrap_or(bytes.len() - 1);").unwrap();
    writeln!(file, "        buf.append_slice(&bytes[first_non_zero..]);").unwrap();
    writeln!(file, "        Ok(())").unwrap();
    writeln!(file, "    }}").unwrap();
    if default.is_some() {
        writeln!(file, "    const HAS_DEFAULT_VALUE: bool = true;").unwrap();
    }
    writeln!(file, "}}").unwrap();

    // Implement Default if default is Some
    writeln!(file, "impl Default for {name} {{").unwrap();
    writeln!(file, "    fn default() -> Self {{").unwrap();
    if let Some(default_value) = default {
        writeln!(file, "        Self({default_value})").unwrap();
    } else {
        writeln!(file, "        Self(0)").unwrap();
    }
    writeln!(file, "    }}").unwrap();
    writeln!(file, "}}").unwrap();
}

// ref:
// #[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
// pub struct SignedInteger(pub i64);
// impl Deref for SignedInteger {
//     type Target = i64;
//     fn deref(&self) -> &Self::Target {
//         &self.0
//     }
// }

// impl Element for SignedInteger {
//     const ID: VInt64 = VInt64::from_encoded(0x13);
//     fn decode_body(buf: &mut &[u8]) -> crate::Result<Self> {
//         if buf.is_empty() {
//             return Ok(Self(0));
//         }
//         if buf.len() > 8 {
//             return Err(crate::Error::UnderDecode(Self::ID));
//         }
//         let len = buf.len().min(8);
//         let is_neg = (buf[0] & 0x80) != 0;
//         let mut value = if is_neg { [0xFFu8; 8] } else { [0u8; 8] };
//         value[8 - len..].copy_from_slice(&buf[..len]);
//         buf.advance(len);
//         Ok(Self(i64::from_be_bytes(value)))
//     }
//     fn encode_body<B: crate::functional::BufMut>(&self, buf: &mut B) -> crate::Result<()> {
//         let bytes = self.0.to_be_bytes();
//         if self.0 >= 0 {
//             let first_non_zero = bytes
//                 .iter()
//                 .position(|&b| b != 0)
//                 .unwrap_or(bytes.len() - 1);
//             buf.append_slice(&bytes[first_non_zero..]);
//             Ok(())
//         } else {
//             let first_non_ff = bytes
//                 .iter()
//                 .position(|&b| b != 0xFF)
//                 .unwrap_or(bytes.len() - 1);
//             buf.append_slice(&bytes[first_non_ff..]);
//             Ok(())
//         }
//     }
// }
fn signed(file: &mut File, name: &str, id: &str, default: Option<&str>) {
    writeln!(
        file,
        "#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]"
    )
    .unwrap();
    writeln!(file, "pub struct {name}(pub i64);").unwrap();
    // Implement Deref to i64
    writeln!(file, "impl std::ops::Deref for {name} {{ type Target = i64; fn deref(&self) -> &Self::Target {{ &self.0 }} }}").unwrap();
    // Implement Element
    writeln!(file, "impl Element for {name} {{").unwrap();
    writeln!(file, "    const ID: VInt64 = VInt64::from_encoded({id});").unwrap();
    writeln!(
        file,
        "    fn decode_body(buf: &mut &[u8]) -> crate::Result<Self> {{"
    )
    .unwrap();
    writeln!(file, "        if buf.is_empty() {{").unwrap();
    if let Some(default_value) = default {
        writeln!(file, "            return Ok(Self({default_value}));").unwrap();
    } else {
        writeln!(file, "            return Ok(Self(0));").unwrap();
    }
    writeln!(file, "        }}").unwrap();
    writeln!(file, "        if buf.len() > 8 {{").unwrap();
    writeln!(
        file,
        "            return Err(crate::Error::UnderDecode(Self::ID));"
    )
    .unwrap();
    writeln!(file, "        }}").unwrap();
    writeln!(file, "        let len = buf.len().min(8);").unwrap();
    writeln!(file, "        let is_neg = (buf[0] & 0x80) != 0;").unwrap();
    writeln!(
        file,
        "        let mut value = if is_neg {{ [0xFFu8; 8] }} else {{ [0u8; 8] }};"
    )
    .unwrap();
    writeln!(
        file,
        "        value[8 - len..].copy_from_slice(&buf[..len]);"
    )
    .unwrap();
    writeln!(file, "        buf.advance(len);").unwrap();
    writeln!(file, "        Ok(Self(i64::from_be_bytes(value)))").unwrap();
    writeln!(file, "    }}").unwrap();
    writeln!(file, "    fn encode_body<B: crate::functional::BufMut>(&self, buf: &mut B) -> crate::Result<()> {{").unwrap();
    writeln!(file, "        let bytes = self.0.to_be_bytes();").unwrap();
    writeln!(file, "        if self.0 >= 0 {{").unwrap();
    writeln!(file, "            let first_non_zero = bytes.iter().position(|&b| b != 0).unwrap_or(bytes.len() - 1);").unwrap();
    writeln!(
        file,
        "            buf.append_slice(&bytes[first_non_zero..]);"
    )
    .unwrap();
    writeln!(file, "            Ok(())").unwrap();
    writeln!(file, "        }} else {{").unwrap();
    writeln!(file, "            let first_non_ff = bytes.iter().position(|&b| b != 0xFF).unwrap_or(bytes.len() - 1);").unwrap();
    writeln!(
        file,
        "            buf.append_slice(&bytes[first_non_ff..]);"
    )
    .unwrap();
    writeln!(file, "            Ok(())").unwrap();
    writeln!(file, "        }}").unwrap();
    writeln!(file, "    }}").unwrap();
    if default.is_some() {
        writeln!(file, "    const HAS_DEFAULT_VALUE: bool = true;").unwrap();
    }
    writeln!(file, "}}").unwrap();

    // Implement Default if default is Some
    writeln!(file, "impl Default for {name} {{").unwrap();
    writeln!(file, "    fn default() -> Self {{").unwrap();
    if let Some(default_value) = default {
        writeln!(file, "        Self({default_value})").unwrap();
    } else {
        writeln!(file, "        Self(0)").unwrap();
    }
    writeln!(file, "    }}").unwrap();
    writeln!(file, "}}").unwrap();
}
// ref:
// #[derive(Debug, Copy, Clone, PartialEq, PartialOrd)]
// pub struct Float(pub f64);
// impl Deref for Float {
//     type Target = f64;
//     fn deref(&self) -> &Self::Target {
//         &self.0
//     }
// }

// impl Element for Float {
//     const ID: VInt64 = VInt64::from_encoded(0x14);
//     fn decode_body(buf: &mut &[u8]) -> crate::Result<Self> {
//         match buf.len() {
//             0 => Ok(Self(0.0)),
//             4 => {
//                 let mut value = [0u8; 4];
//                 value.copy_from_slice(&buf[..4]);
//                 buf.advance(4);
//                 Ok(Self(f32::from_be_bytes(value) as f64))
//             }
//             8 => {
//                 let mut value = [0u8; 8];
//                 value.copy_from_slice(&buf[..8]);
//                 buf.advance(8);
//                 Ok(Self(f64::from_be_bytes(value)))
//             }
//             _ => Err(crate::Error::UnderDecode(Self::ID)),
//         }
//     }
//     fn encode_body<B: crate::functional::BufMut>(&self, buf: &mut B) -> crate::Result<()> {
//         fn can_represent_as_f32(value: f64) -> bool {
//             if value.is_infinite() || value.is_nan() {
//                 return false;
//             }

//             if value.abs() > f32::MAX as f64
//                 || (value != 0.0 && value.abs() < f32::MIN_POSITIVE as f64)
//             {
//                 return false;
//             }

//             let f32_value = value as f32;
//             f32_value as f64 == value
//         }

//         if can_represent_as_f32(self.0) {
//             buf.append_slice(&(self.0 as f32).to_be_bytes());
//             Ok(())
//         } else {
//             buf.append_slice(&self.0.to_be_bytes());
//             Ok(())
//         }
//     }
// }
fn float(file: &mut File, name: &str, id: &str, default: Option<&str>) {
    writeln!(file, "#[derive(Debug, Copy, Clone, PartialEq, PartialOrd)]").unwrap();
    writeln!(file, "pub struct {name}(pub f64);").unwrap();
    // Implement Deref to f64
    writeln!(file, "impl std::ops::Deref for {name} {{ type Target = f64; fn deref(&self) -> &Self::Target {{ &self.0 }} }}").unwrap();
    // Implement Element
    writeln!(file, "impl Element for {name} {{").unwrap();
    writeln!(file, "    const ID: VInt64 = VInt64::from_encoded({id});").unwrap();
    writeln!(
        file,
        "    fn decode_body(buf: &mut &[u8]) -> crate::Result<Self> {{"
    )
    .unwrap();
    writeln!(file, "        match buf.len() {{").unwrap();
    if let Some(default_value) = default {
        writeln!(
            file,
            "            0 => Ok(Self(hexf::hexf64!(\"{default_value}\"))),"
        )
        .unwrap();
    } else {
        writeln!(file, "            0 => Ok(Self(0.0)),").unwrap();
    }
    writeln!(file, "            4 => {{").unwrap();
    writeln!(file, "                let mut value = [0u8; 4];").unwrap();
    writeln!(file, "                value.copy_from_slice(&buf[..4]);").unwrap();
    writeln!(file, "                buf.advance(4);").unwrap();
    writeln!(
        file,
        "                Ok(Self(f32::from_be_bytes(value) as f64))"
    )
    .unwrap();
    writeln!(file, "            }},").unwrap();
    writeln!(file, "            8 => {{").unwrap();
    writeln!(file, "                let mut value = [0u8; 8];").unwrap();
    writeln!(file, "                value.copy_from_slice(&buf[..8]);").unwrap();
    writeln!(file, "                buf.advance(8);").unwrap();
    writeln!(file, "                Ok(Self(f64::from_be_bytes(value)))").unwrap();
    writeln!(file, "            }},").unwrap();
    writeln!(
        file,
        "            _ => Err(crate::Error::UnderDecode(Self::ID)),"
    )
    .unwrap();
    writeln!(file, "        }}").unwrap();
    writeln!(file, "    }}").unwrap();
    writeln!(file, "    fn encode_body<B: crate::functional::BufMut>(&self, buf: &mut B) -> crate::Result<()> {{").unwrap();
    writeln!(
        file,
        "        fn can_represent_as_f32(value: f64) -> bool {{"
    )
    .unwrap();
    writeln!(
        file,
        "            if value.is_infinite() || value.is_nan() {{"
    )
    .unwrap();
    writeln!(file, "                return false;").unwrap();
    writeln!(file, "            }}").unwrap();
    writeln!(file, "            if value.abs() > f32::MAX as f64 || (value != 0.0 && value.abs() < f32::MIN_POSITIVE as f64) {{").unwrap();
    writeln!(file, "                return false;").unwrap();
    writeln!(file, "            }}").unwrap();
    writeln!(file, "            let f32_value = value as f32;").unwrap();
    writeln!(file, "            f32_value as f64 == value").unwrap();
    writeln!(file, "        }}").unwrap();
    writeln!(file, "        if can_represent_as_f32(self.0) {{").unwrap();
    writeln!(
        file,
        "            buf.append_slice(&(self.0 as f32).to_be_bytes());"
    )
    .unwrap();
    writeln!(file, "            Ok(())").unwrap();
    writeln!(file, "        }} else {{").unwrap();
    writeln!(file, "            buf.append_slice(&self.0.to_be_bytes());").unwrap();
    writeln!(file, "            Ok(())").unwrap();
    writeln!(file, "        }}").unwrap();
    writeln!(file, "    }}").unwrap();
    if default.is_some() {
        writeln!(file, "    const HAS_DEFAULT_VALUE: bool = true;").unwrap();
    }
    writeln!(file, "}}").unwrap();

    // Implement Default if default is Some
    writeln!(file, "impl Default for {name} {{").unwrap();
    writeln!(file, "    fn default() -> Self {{").unwrap();
    if let Some(default_value) = default {
        writeln!(file, "        Self(hexf::hexf64!(\"{default_value}\"))").unwrap();
    } else {
        writeln!(file, "        Self(0.0)").unwrap();
    }
    writeln!(file, "    }}").unwrap();
    writeln!(file, "}}").unwrap();
}

// ref:
//  #[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
//     pub struct Text(pub String);
//     impl Deref for Text {
//         type Target = str;
//         fn deref(&self) -> &Self::Target {
//             &self.0
//         }
//     }

//     impl Element for Text {
//         const ID: VInt64 = VInt64::from_encoded(0x15);
//         fn decode_body(buf: &mut &[u8]) -> crate::Result<Self> {
//             let first_zero = buf.iter().position(|&b| b == 0).unwrap_or(buf.len());
//             let result = Self(String::from_utf8_lossy(&buf[..first_zero]).to_string());
//             buf.advance(buf.len());
//             Ok(result)
//         }
//         fn encode_body<B: crate::functional::BufMut>(&self, buf: &mut B) -> crate::Result<()> {
//             buf.append_slice(self.0.as_bytes());
//             Ok(())
//         }
//     }
fn text(file: &mut File, name: &str, id: &str, default: Option<&str>) {
    writeln!(
        file,
        "#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]"
    )
    .unwrap();
    writeln!(file, "pub struct {name}(pub String);").unwrap();
    // Implement Deref to str
    writeln!(file, "impl std::ops::Deref for {name} {{ type Target = str; fn deref(&self) -> &Self::Target {{ &self.0 }} }}").unwrap();
    // Implement Element
    writeln!(file, "impl Element for {name} {{").unwrap();
    writeln!(file, "    const ID: VInt64 = VInt64::from_encoded({id});").unwrap();
    writeln!(
        file,
        "    fn decode_body(buf: &mut &[u8]) -> crate::Result<Self> {{"
    )
    .unwrap();
    writeln!(file, "    if buf.is_empty() {{").unwrap();
    if let Some(default_value) = default {
        writeln!(file, "        return Ok(Self(\"{default_value}\".into()));").unwrap();
    } else {
        writeln!(file, "        return Ok(Self(\"\".into()));").unwrap();
    }
    writeln!(file, "    }}").unwrap();
    writeln!(
        file,
        "        let first_zero = buf.iter().position(|&b| b == 0).unwrap_or(buf.len());"
    )
    .unwrap();
    writeln!(
        file,
        "        let result = Self(String::from_utf8_lossy(&buf[..first_zero]).to_string());"
    )
    .unwrap();
    writeln!(file, "        buf.advance(buf.len());").unwrap();
    writeln!(file, "        Ok(result)").unwrap();
    writeln!(file, "    }}").unwrap();
    writeln!(file, "    fn encode_body<B: crate::functional::BufMut>(&self, buf: &mut B) -> crate::Result<()> {{").unwrap();
    writeln!(file, "        buf.append_slice(self.0.as_bytes());").unwrap();
    writeln!(file, "        Ok(())").unwrap();
    writeln!(file, "    }}").unwrap();
    if default.is_some() {
        writeln!(file, "    const HAS_DEFAULT_VALUE: bool = true;").unwrap();
    }
    writeln!(file, "}}").unwrap();

    // Implement Default if default is Some
    writeln!(file, "impl Default for {name} {{").unwrap();
    writeln!(file, "    fn default() -> Self {{").unwrap();
    if let Some(default_value) = default {
        writeln!(file, "        Self(\"{default_value}\".to_string())").unwrap();
    } else {
        writeln!(file, "        Self(String::new())").unwrap();
    }
    writeln!(file, "    }}").unwrap();
    writeln!(file, "}}").unwrap();
}

// ref:
// #[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
//     pub struct Bin(pub Vec<u8>);
//     impl Deref for Bin {
//         type Target = [u8];
//         fn deref(&self) -> &Self::Target {
//             &self.0
//         }
//     }

//     impl Element for Bin {
//         const ID: VInt64 = VInt64::from_encoded(0x16);
//         fn decode_body(buf: &mut &[u8]) -> crate::Result<Self> {
//             let result = Self(buf.to_vec());
//             buf.advance(buf.len());
//             Ok(result)
//         }
//         fn encode_body<B: crate::functional::BufMut>(&self, buf: &mut B) -> crate::Result<()> {
//             buf.append_slice(&self.0);
//             Ok(())
//         }
//     }
fn bin(file: &mut File, name: &str, id: &str, _default: Option<&str>) {
    writeln!(
        file,
        "#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]"
    )
    .unwrap();
    writeln!(file, "pub struct {name}(pub Vec<u8>);").unwrap();
    // Implement Deref to [u8]
    writeln!(file, "impl std::ops::Deref for {name} {{ type Target = [u8]; fn deref(&self) -> &Self::Target {{ &self.0 }} }}").unwrap();
    // Implement Element
    writeln!(file, "impl Element for {name} {{").unwrap();
    writeln!(file, "    const ID: VInt64 = VInt64::from_encoded({id});").unwrap();
    writeln!(
        file,
        "    fn decode_body(buf: &mut &[u8]) -> crate::Result<Self> {{"
    )
    .unwrap();
    writeln!(file, "        let result = Self(buf.to_vec());").unwrap();
    writeln!(file, "        buf.advance(buf.len());").unwrap();
    writeln!(file, "        Ok(result)").unwrap();
    writeln!(file, "    }}").unwrap();
    writeln!(file, "    fn encode_body<B: crate::functional::BufMut>(&self, buf: &mut B) -> crate::Result<()> {{").unwrap();
    writeln!(file, "        buf.append_slice(&self.0);").unwrap();
    writeln!(file, "        Ok(())").unwrap();
    writeln!(file, "    }}").unwrap();
    writeln!(file, "}}").unwrap();

    writeln!(file, "impl Default for {name} {{").unwrap();
    writeln!(file, "    fn default() -> Self {{").unwrap();
    writeln!(file, "        Self(Vec::new())").unwrap();
    writeln!(file, "    }}").unwrap();
    writeln!(file, "}}").unwrap();
}

// ref:
//  #[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
//     pub struct Date(pub i64);
//     impl Deref for Date {
//         type Target = i64;
//         fn deref(&self) -> &Self::Target {
//             &self.0
//         }
//     }

//     impl Element for Date {
//         const ID: VInt64 = VInt64::from_encoded(0x17);
//         fn decode_body(buf: &mut &[u8]) -> crate::Result<Self> {
//             if buf.len() != 8 {
//                 return Err(crate::Error::UnderDecode(Self::ID));
//             }
//             let result = i64::from_be_bytes(buf[..8].try_into().unwrap());
//             buf.advance(8);
//             Ok(Self(result))
//         }
//         fn encode_body<B: crate::functional::BufMut>(&self, buf: &mut B) -> crate::Result<()> {
//             buf.append_slice(&self.0.to_be_bytes());
//             Ok(())
//         }
//     }
fn date(file: &mut File, name: &str, id: &str, default: Option<&str>) {
    writeln!(
        file,
        "#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]"
    )
    .unwrap();
    writeln!(file, "pub struct {name}(pub i64);").unwrap();
    // Implement Deref to i64
    writeln!(file, "impl std::ops::Deref for {name} {{ type Target = i64; fn deref(&self) -> &Self::Target {{ &self.0 }} }}").unwrap();
    // Implement Element
    writeln!(file, "impl Element for {name} {{").unwrap();
    writeln!(file, "    const ID: VInt64 = VInt64::from_encoded({id});").unwrap();
    writeln!(
        file,
        "    fn decode_body(buf: &mut &[u8]) -> crate::Result<Self> {{"
    )
    .unwrap();
    writeln!(file, "        if buf.len() != 8 {{").unwrap();
    writeln!(
        file,
        "            return Err(crate::Error::UnderDecode(Self::ID));"
    )
    .unwrap();
    writeln!(file, "        }}").unwrap();
    writeln!(
        file,
        "        let result = i64::from_be_bytes(buf[..8].try_into().unwrap());"
    )
    .unwrap();
    writeln!(file, "        buf.advance(8);").unwrap();
    writeln!(file, "        Ok(Self(result))").unwrap();
    writeln!(file, "    }}").unwrap();
    writeln!(file, "    fn encode_body<B: crate::functional::BufMut>(&self, buf: &mut B) -> crate::Result<()> {{").unwrap();
    writeln!(file, "        buf.append_slice(&self.0.to_be_bytes());").unwrap();
    writeln!(file, "        Ok(())").unwrap();
    writeln!(file, "    }}").unwrap();
    writeln!(file, "}}").unwrap();

    // Implement Default if default is Some
    writeln!(file, "impl Default for {name} {{").unwrap();
    writeln!(file, "    fn default() -> Self {{").unwrap();
    if let Some(default_value) = default {
        writeln!(file, "        Self({default_value})").unwrap();
    } else {
        writeln!(file, "        Self(0)").unwrap();
    }
    writeln!(file, "    }}").unwrap();
    writeln!(file, "}}").unwrap();
}

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
        let default_value = element.attribute("default");

        let documentation = element
            .children()
            .find(|n| n.has_tag_name("documentation"))
            .and_then(|n| n.text());
        if let Some(doc) = documentation {
            for line in doc.lines() {
                writeln!(
                    file,
                    "/// {}",
                    line.trim().replace("[", "\\[").replace("]", "\\]")
                )
                .unwrap();
            }
        } else {
            writeln!(file, "/// {name} in ebml").unwrap();
        }

        // name adjustments
        let name = match name {
            "EBMLMaxIDLength" => "EbmlMaxIdLength",
            "EBMLMaxSizeLength" => "EbmlMaxSizeLength",
            "SeekID" => "SeekId",
            "SegmentUUID" => "SegmentUuid",
            "PrevUUID" => "PrevUuid",
            "NextUUID" => "NextUuid",
            "DateUTC" => "DateUtc",
            "ChapterTranslateID" => "ChapterTranslateId",
            "ChapterTranslateEditionUID" => "ChapterTranslateEditionUid",
            "BlockAddID" => "BlockAddId",
            "TrackUID" => "TrackUid",
            "LanguageBCP47" => "LanguageBcp47",
            "CodecID" => "CodecId",
            "MaxBlockAdditionID" => "MaxBlockAdditionId",
            "BlockAddIDType" => "BlockAddIdType",
            "BlockAddIDValue" => "BlockAddIdValue",
            "BlockAddIDExtraData" => "BlockAddIdExtraData",
            "BlockAddIDName" => "BlockAddIdName",
            "TrackTranslateTrackID" => "TrackTranslateTrackId",
            "TrackTranslateEditionUID" => "TrackTranslateEditionUid",
            "UncompressedFourCC" => "UncompressedFourcc",
            "MaxCLL" => "MaxCll",
            "MaxFALL" => "MaxFall",
            "TrackPlaneUID" => "TrackPlaneUid",
            "TrackJoinUID" => "TrackJoinUid",
            "ContentEncKeyID" => "ContentEncKeyId",
            "AESSettingsCipherMode" => "AesSettingsCipherMode",
            _ => name,
        };

        match element.attribute("type").unwrap() {
            "uinteger" => unsigned(&mut file, name, id, default_value),
            "integer" => signed(&mut file, name, id, default_value),
            "string" => text(&mut file, name, id, default_value),
            "utf-8" => text(&mut file, name, id, default_value),
            "binary" => bin(&mut file, name, id, default_value),
            "float" => float(&mut file, name, id, default_value),
            "date" => date(&mut file, name, id, default_value),
            other => panic!("Unknown type: {other}"),
        };
    }

    writeln!(
        file,
        "/// EBMLVersion element, indicates the version of EBML used."
    )
    .unwrap();
    unsigned(&mut file, "EbmlVersion", "0x4286", Some("1"));

    writeln!(
        file,
        "/// EBMLReadVersion element, indicates the read version of EBML used."
    )
    .unwrap();
    unsigned(&mut file, "EbmlReadVersion", "0x42f7", Some("1"));

    writeln!(
        file,
        "/// DocType element, indicates the type of the document."
    )
    .unwrap();
    text(&mut file, "DocType", "0x4282", Some("matroska"));

    writeln!(
        file,
        "/// DocTypeVersion element, indicates the version of the document type."
    )
    .unwrap();
    unsigned(&mut file, "DocTypeVersion", "0x4287", Some("1"));

    writeln!(
        file,
        "/// DocTypeReadVersion element, indicates the read version of the document type."
    )
    .unwrap();
    unsigned(&mut file, "DocTypeReadVersion", "0x4285", Some("1"));
}
