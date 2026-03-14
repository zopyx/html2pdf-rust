//! PDF font handling

use super::PdfDictionary;


/// PDF Font representation
#[derive(Debug, Clone, PartialEq)]
pub struct PdfFont {
    pub name: String,
    pub base_font: String,
    pub subtype: FontSubtype,
    pub encoding: Option<String>,
    pub descriptor: Option<FontDescriptor>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FontSubtype {
    Type1,
    Type3,
    TrueType,
    CIDFontType0,
    CIDFontType2,
}

impl FontSubtype {
    pub fn as_str(&self) -> &'static str {
        match self {
            FontSubtype::Type1 => "Type1",
            FontSubtype::Type3 => "Type3",
            FontSubtype::TrueType => "TrueType",
            FontSubtype::CIDFontType0 => "CIDFontType0",
            FontSubtype::CIDFontType2 => "CIDFontType2",
        }
    }
}

/// Font descriptor
#[derive(Debug, Clone, PartialEq, Default)]
pub struct FontDescriptor {
    pub ascent: i32,
    pub descent: i32,
    pub cap_height: i32,
    pub flags: u32,
    pub font_bbox: (i32, i32, i32, i32),
    pub italic_angle: i32,
    pub stem_v: i32,
}

impl PdfFont {
    /// Create a standard PDF font
    pub fn standard(name: impl Into<String>, base_font: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            base_font: base_font.into(),
            subtype: FontSubtype::Type1,
            encoding: Some("WinAnsiEncoding".to_string()),
            descriptor: None,
        }
    }

    /// Convert to PDF dictionary
    pub fn to_dictionary(&self) -> PdfDictionary {
        let mut dict = PdfDictionary::new();
        dict.insert("Type", super::PdfObject::Name("Font".to_string()));
        dict.insert("Subtype", super::PdfObject::Name(self.subtype.as_str().to_string()));
        dict.insert("BaseFont", super::PdfObject::Name(self.base_font.clone()));
        
        if let Some(encoding) = &self.encoding {
            dict.insert("Encoding", super::PdfObject::Name(encoding.clone()));
        }
        
        // TODO: Add font descriptor for non-standard fonts
        
        dict
    }
}

/// Standard PDF font names
#[allow(dead_code)]
pub const STANDARD_FONTS: &[&str] = &[
    "Courier",
    "Courier-Bold",
    "Courier-Oblique",
    "Courier-BoldOblique",
    "Helvetica",
    "Helvetica-Bold",
    "Helvetica-Oblique",
    "Helvetica-BoldOblique",
    "Times-Roman",
    "Times-Bold",
    "Times-Italic",
    "Times-BoldItalic",
    "Symbol",
    "ZapfDingbats",
];

/// Check if a font is a standard PDF font
#[allow(dead_code)]
pub fn is_standard_font(name: &str) -> bool {
    STANDARD_FONTS.contains(&name)
}

/// Font metrics (simplified - in production would load from AFM files)
#[allow(dead_code)]
pub struct FontMetrics {
    pub widths: std::collections::HashMap<char, u16>,
    pub default_width: u16,
    pub ascent: i16,
    pub descent: i16,
    pub x_height: i16,
    pub cap_height: i16,
}

#[allow(dead_code)]
impl FontMetrics {
    /// Create metrics for Helvetica (approximate values)
    pub fn helvetica() -> Self {
        Self {
            widths: std::collections::HashMap::new(),
            default_width: 500,
            ascent: 718,
            descent: -207,
            x_height: 523,
            cap_height: 718,
        }
    }

    /// Create metrics for Times Roman
    pub fn times_roman() -> Self {
        Self {
            widths: std::collections::HashMap::new(),
            default_width: 500,
            ascent: 683,
            descent: -217,
            x_height: 450,
            cap_height: 662,
        }
    }

    /// Create metrics for Courier
    pub fn courier() -> Self {
        Self {
            widths: std::collections::HashMap::new(),
            default_width: 600,
            ascent: 629,
            descent: -157,
            x_height: 441,
            cap_height: 562,
        }
    }

    /// Get width of a string in thousandths of a unit
    pub fn string_width(&self, s: &str, font_size: f32) -> f32 {
        let mut total = 0u32;
        for c in s.chars() {
            total += *self.widths.get(&c).unwrap_or(&self.default_width) as u32;
        }
        (total as f32 / 1000.0) * font_size
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_standard_fonts() {
        assert!(is_standard_font("Helvetica"));
        assert!(is_standard_font("Times-Roman"));
        assert!(!is_standard_font("NonExistent"));
    }

    #[test]
    fn test_font_metrics() {
        let metrics = FontMetrics::helvetica();
        let width = metrics.string_width("Hello", 12.0);
        assert!(width > 0.0);
    }
}
