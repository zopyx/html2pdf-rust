//! PDF object types (PDF 1.4 specification)

use std::collections::BTreeMap;
use std::fmt;

/// PDF object types
#[derive(Debug, Clone, PartialEq)]
pub enum PdfObject {
    Null,
    Bool(bool),
    Integer(i32),
    Real(f32),
    String(Vec<u8>),
    HexString(Vec<u8>),
    Name(String),
    Array(PdfArray),
    Dictionary(PdfDictionary),
    Stream(Vec<u8>), // Raw stream data
    Reference(PdfReference),
}

impl PdfObject {
    /// Write object to PDF output
    pub fn write<W: fmt::Write>(&self, writer: &mut W) -> fmt::Result {
        match self {
            PdfObject::Null => write!(writer, "null"),
            PdfObject::Bool(b) => write!(writer, "{}", if *b { "true" } else { "false" }),
            PdfObject::Integer(i) => write!(writer, "{}", i),
            PdfObject::Real(f) => write!(writer, "{:.6}", f),
            PdfObject::String(s) => {
                // Literal string - escape special characters
                writer.write_str("(")?;
                for &byte in s {
                    match byte {
                        b'\\' | b'(' | b')' => {
                            writer.write_str("\\")?;
                            writer.write_char(byte as char)?;
                        }
                        b'\n' => writer.write_str("\\n")?,
                        b'\r' => writer.write_str("\\r")?,
                        b'\t' => writer.write_str("\\t")?,
                        c => writer.write_char(c as char)?,
                    }
                }
                writer.write_str(")")
            }
            PdfObject::HexString(s) => {
                writer.write_str("<")?;
                for byte in s {
                    write!(writer, "{:02X}", byte)?;
                }
                writer.write_str(">")
            }
            PdfObject::Name(n) => {
                writer.write_str("/")?;
                // Escape special characters in name
                for c in n.chars() {
                    if c.is_ascii_alphanumeric() || c == '_' || c == '.' || c == '-' {
                        writer.write_char(c)?;
                    } else {
                        write!(writer, "#{:02X}", c as u8)?;
                    }
                }
                Ok(())
            }
            PdfObject::Array(a) => a.write(writer),
            PdfObject::Dictionary(d) => d.write(writer),
            PdfObject::Stream(_) => {
                // Streams are handled specially with their length
                panic!("Stream objects should be written separately")
            }
            PdfObject::Reference(r) => write!(writer, "{} {} R", r.object_number, r.generation),
        }
    }

    /// Get object as integer
    pub fn as_integer(&self) -> Option<i32> {
        match self {
            PdfObject::Integer(i) => Some(*i),
            _ => None,
        }
    }

    /// Get object as string
    pub fn as_string(&self) -> Option<&str> {
        match self {
            PdfObject::String(s) => std::str::from_utf8(s).ok(),
            _ => None,
        }
    }

    /// Get object as name
    pub fn as_name(&self) -> Option<&str> {
        match self {
            PdfObject::Name(n) => Some(n),
            _ => None,
        }
    }
}

impl From<bool> for PdfObject {
    fn from(b: bool) -> Self {
        PdfObject::Bool(b)
    }
}

impl From<i32> for PdfObject {
    fn from(i: i32) -> Self {
        PdfObject::Integer(i)
    }
}

impl From<f32> for PdfObject {
    fn from(f: f32) -> Self {
        PdfObject::Real(f)
    }
}

impl From<&str> for PdfObject {
    fn from(s: &str) -> Self {
        PdfObject::String(s.as_bytes().to_vec())
    }
}

impl From<String> for PdfObject {
    fn from(s: String) -> Self {
        PdfObject::String(s.into_bytes())
    }
}

/// PDF object reference
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct PdfReference {
    pub object_number: u32,
    pub generation: u16,
}

impl PdfReference {
    pub const fn new(object_number: u32, generation: u16) -> Self {
        Self {
            object_number,
            generation,
        }
    }
}

/// PDF array
#[derive(Debug, Clone, PartialEq, Default)]
pub struct PdfArray {
    items: Vec<PdfObject>,
}

impl PdfArray {
    pub fn new() -> Self {
        Self { items: Vec::new() }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            items: Vec::with_capacity(capacity),
        }
    }

    pub fn push(&mut self, obj: impl Into<PdfObject>) {
        self.items.push(obj.into());
    }

    pub fn push_ref(&mut self, reference: PdfReference) {
        self.items.push(PdfObject::Reference(reference));
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn write<W: fmt::Write>(&self, writer: &mut W) -> fmt::Result {
        writer.write_str("[")?;
        for (i, item) in self.items.iter().enumerate() {
            if i > 0 {
                writer.write_str(" ")?;
            }
            item.write(writer)?;
        }
        writer.write_str("]")
    }
}

impl FromIterator<PdfObject> for PdfArray {
    fn from_iter<T: IntoIterator<Item = PdfObject>>(iter: T) -> Self {
        Self {
            items: iter.into_iter().collect(),
        }
    }
}

/// PDF dictionary (ordered by key for consistency)
#[derive(Debug, Clone, PartialEq, Default)]
pub struct PdfDictionary {
    entries: BTreeMap<String, PdfObject>,
}

impl PdfDictionary {
    pub fn new() -> Self {
        Self {
            entries: BTreeMap::new(),
        }
    }

    pub fn insert(&mut self, key: impl Into<String>, value: impl Into<PdfObject>) {
        self.entries.insert(key.into(), value.into());
    }

    pub fn insert_ref(&mut self, key: impl Into<String>, reference: PdfReference) {
        self.entries.insert(key.into(), PdfObject::Reference(reference));
    }

    pub fn get(&self, key: &str) -> Option<&PdfObject> {
        self.entries.get(key)
    }

    pub fn contains(&self, key: &str) -> bool {
        self.entries.contains_key(key)
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn write<W: fmt::Write>(&self, writer: &mut W) -> fmt::Result {
        writer.write_str("<<")?;
        for (key, value) in &self.entries {
            write!(writer, "/{}", key)?;
            writer.write_str(" ")?;
            value.write(writer)?;
        }
        writer.write_str(">>")
    }

    /// Merge another dictionary into this one
    pub fn merge(&mut self, other: &PdfDictionary) {
        for (key, value) in &other.entries {
            self.entries.insert(key.clone(), value.clone());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_object_writing() {
        let mut output = String::new();
        
        PdfObject::Null.write(&mut output).unwrap();
        assert_eq!(output, "null");
        
        output.clear();
        PdfObject::Bool(true).write(&mut output).unwrap();
        assert_eq!(output, "true");
        
        output.clear();
        PdfObject::Integer(42).write(&mut output).unwrap();
        assert_eq!(output, "42");
        
        output.clear();
        PdfObject::Real(3.14159).write(&mut output).unwrap();
        assert!(output.starts_with("3.1415"));
    }

    #[test]
    fn test_string_escaping() {
        let mut output = String::new();
        PdfObject::String(b"(test)".to_vec()).write(&mut output).unwrap();
        assert_eq!(output, "(\\(test\\))");
    }

    #[test]
    fn test_dictionary() {
        let mut dict = PdfDictionary::new();
        dict.insert("Type", PdfObject::Name("Catalog".to_string()));
        dict.insert("Pages", PdfObject::Reference(PdfReference::new(1, 0)));
        
        let mut output = String::new();
        dict.write(&mut output).unwrap();
        assert!(output.contains("/Type /Catalog"));
        assert!(output.contains("/Pages 1 0 R"));
    }

    #[test]
    fn test_array() {
        let mut arr = PdfArray::new();
        arr.push(1i32);
        arr.push(2.5f32);
        arr.push("hello");
        
        let mut output = String::new();
        arr.write(&mut output).unwrap();
        assert_eq!(output, "[1 2.500000 (hello)]");
    }

    #[test]
    fn test_reference() {
        let mut output = String::new();
        PdfObject::Reference(PdfReference::new(5, 0)).write(&mut output).unwrap();
        assert_eq!(output, "5 0 R");
    }
}
