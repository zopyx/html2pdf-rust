//! PDF font handling with TrueType support and font metrics
//!
//! This module provides comprehensive font handling including:
//! - Standard PDF fonts (Type 1)
//! - TrueType font embedding
//! - Font metrics extraction from TTF files
//! - Font subsetting for reduced PDF size
//! - Font descriptor generation

use super::{PdfDictionary, PdfObject, PdfArray};
use std::collections::HashMap;

/// PDF Font representation
#[derive(Debug, Clone, PartialEq)]
pub struct PdfFont {
    pub name: String,
    pub base_font: String,
    pub subtype: FontSubtype,
    pub encoding: Option<String>,
    pub descriptor: Option<FontDescriptor>,
    /// Font metrics for layout calculations
    pub metrics: Option<FontMetrics>,
    /// For TrueType fonts: the widths array
    pub widths: Option<Vec<u16>>,
    /// First character code
    pub first_char: Option<u8>,
    /// Last character code
    pub last_char: Option<u8>,
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

/// Font descriptor - required for non-standard fonts
#[derive(Debug, Clone, PartialEq, Default)]
pub struct FontDescriptor {
    pub ascent: i32,
    pub descent: i32,
    pub cap_height: i32,
    pub flags: u32,
    pub font_bbox: (i32, i32, i32, i32),
    pub italic_angle: i32,
    pub stem_v: i32,
    pub x_height: i32,
    pub leading: i32,
    pub max_width: i32,
    pub avg_width: i32,
    pub missing_width: i32,
    /// Font file reference (for embedded fonts)
    pub font_file2: Option<u32>,
    /// Font name
    pub font_name: String,
    /// Character set (for Type 1 fonts)
    pub char_set: Option<String>,
}

impl FontDescriptor {
    /// Convert to PDF dictionary
    pub fn to_dictionary(&self) -> PdfDictionary {
        let mut dict = PdfDictionary::new();
        
        dict.insert("Type", PdfObject::Name("FontDescriptor".to_string()));
        dict.insert("FontName", PdfObject::Name(self.font_name.clone()));
        dict.insert("Ascent", self.ascent);
        dict.insert("Descent", self.descent);
        dict.insert("CapHeight", self.cap_height);
        dict.insert("Flags", self.flags as i32);
        
        // FontBBox: [left, bottom, right, top]
        let mut bbox = PdfArray::new();
        bbox.push(self.font_bbox.0);
        bbox.push(self.font_bbox.1);
        bbox.push(self.font_bbox.2);
        bbox.push(self.font_bbox.3);
        dict.insert("FontBBox", PdfObject::Array(bbox));
        
        dict.insert("ItalicAngle", self.italic_angle);
        dict.insert("StemV", self.stem_v);
        
        if self.x_height != 0 {
            dict.insert("XHeight", self.x_height);
        }
        
        if let Some(font_file2) = self.font_file2 {
            dict.insert("FontFile2", PdfObject::Reference(
                super::object::PdfReference::new(font_file2, 0)
            ));
        }
        
        if let Some(char_set) = &self.char_set {
            dict.insert("CharSet", PdfObject::String(char_set.as_bytes().to_vec()));
        }
        
        dict
    }
}

impl PdfFont {
    /// Create a standard PDF font
    pub fn standard(name: impl Into<String>, base_font: impl Into<String>) -> Self {
        let base_font_str = base_font.into();
        Self {
            name: name.into(),
            metrics: Some(FontMetrics::for_standard_font(&base_font_str)),
            base_font: base_font_str,
            subtype: FontSubtype::Type1,
            encoding: Some("WinAnsiEncoding".to_string()),
            descriptor: None,
            widths: None,
            first_char: None,
            last_char: None,
        }
    }

    /// Create a TrueType font with metrics
    pub fn truetype(
        name: impl Into<String>,
        base_font: impl Into<String>,
        metrics: FontMetrics,
        descriptor: FontDescriptor,
    ) -> Self {
        let first_char = 32u8; // Space
        let last_char = 255u8;
        
        // Build widths array for characters 32-255
        let mut widths = Vec::with_capacity(224);
        for c in first_char..=last_char {
            let width = metrics.glyph_widths.get(&(c as u32))
                .copied()
                .unwrap_or(metrics.default_width);
            widths.push(width);
        }
        
        Self {
            name: name.into(),
            base_font: base_font.into(),
            subtype: FontSubtype::TrueType,
            encoding: Some("WinAnsiEncoding".to_string()),
            descriptor: Some(descriptor),
            metrics: Some(metrics),
            widths: Some(widths),
            first_char: Some(first_char),
            last_char: Some(last_char),
        }
    }

    /// Convert to PDF dictionary
    pub fn to_dictionary(&self) -> PdfDictionary {
        let mut dict = PdfDictionary::new();
        dict.insert("Type", PdfObject::Name("Font".to_string()));
        dict.insert("Subtype", PdfObject::Name(self.subtype.as_str().to_string()));
        dict.insert("BaseFont", PdfObject::Name(self.base_font.clone()));
        
        if let Some(encoding) = &self.encoding {
            dict.insert("Encoding", PdfObject::Name(encoding.clone()));
        }
        
        // Add widths for non-standard fonts
        if let Some(ref widths) = self.widths {
            if let (Some(first), Some(last)) = (self.first_char, self.last_char) {
                dict.insert("FirstChar", first as i32);
                dict.insert("LastChar", last as i32);
                
                let mut widths_array = PdfArray::new();
                for w in widths {
                    widths_array.push(*w as i32);
                }
                dict.insert("Widths", PdfObject::Array(widths_array));
            }
        }
        
        // Add font descriptor for non-standard fonts
        if let Some(ref descriptor) = self.descriptor {
            dict.insert("FontDescriptor", PdfObject::Dictionary(descriptor.to_dictionary()));
        }
        
        dict
    }

    /// Get font metrics
    pub fn metrics(&self) -> Option<&FontMetrics> {
        self.metrics.as_ref()
    }

    /// Calculate the width of a string in points at the given font size
    pub fn string_width(&self, text: &str, font_size: f32) -> f32 {
        match &self.metrics {
            Some(metrics) => metrics.string_width(text, font_size),
            None => {
                // Fallback: estimate based on font size
                text.len() as f32 * font_size * 0.5
            }
        }
    }
}

/// Standard PDF font names
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

/// Standard font families
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum StandardFontFamily {
    Helvetica,
    Times,
    Courier,
    Symbol,
    ZapfDingbats,
}

impl StandardFontFamily {
    /// Get the font name for a given weight and style
    pub fn font_name(&self, weight: FontWeight, style: FontStyle) -> &'static str {
        match self {
            StandardFontFamily::Helvetica => match (weight, style) {
                (FontWeight::Bold, FontStyle::Italic) => "Helvetica-BoldOblique",
                (FontWeight::Bold, _) => "Helvetica-Bold",
                (_, FontStyle::Italic) => "Helvetica-Oblique",
                (_, FontStyle::Oblique) => "Helvetica-Oblique",
                _ => "Helvetica",
            },
            StandardFontFamily::Times => match (weight, style) {
                (FontWeight::Bold, FontStyle::Italic) => "Times-BoldItalic",
                (FontWeight::Bold, _) => "Times-Bold",
                (_, FontStyle::Italic) => "Times-Italic",
                (_, FontStyle::Oblique) => "Times-Italic",
                _ => "Times-Roman",
            },
            StandardFontFamily::Courier => match (weight, style) {
                (FontWeight::Bold, FontStyle::Italic) => "Courier-BoldOblique",
                (FontWeight::Bold, _) => "Courier-Bold",
                (_, FontStyle::Italic) => "Courier-Oblique",
                (_, FontStyle::Oblique) => "Courier-Oblique",
                _ => "Courier",
            },
            StandardFontFamily::Symbol => "Symbol",
            StandardFontFamily::ZapfDingbats => "ZapfDingbats",
        }
    }
}

/// Font weight for standard font selection
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FontWeight {
    Normal,
    Bold,
}

/// Font style for standard font selection
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FontStyle {
    Normal,
    Italic,
    Oblique,
}

/// Check if a font is a standard PDF font
pub fn is_standard_font(name: &str) -> bool {
    STANDARD_FONTS.contains(&name)
}

/// Resolve a font family name to a standard font family
pub fn resolve_standard_font_family(name: &str) -> Option<StandardFontFamily> {
    let lower = name.to_ascii_lowercase();
    if lower.contains("helvetica") || lower.contains("arial") || lower.contains("sans") {
        Some(StandardFontFamily::Helvetica)
    } else if lower.contains("times") || lower.contains("serif") {
        Some(StandardFontFamily::Times)
    } else if lower.contains("courier") || lower.contains("mono") {
        Some(StandardFontFamily::Courier)
    } else {
        None
    }
}

/// Get standard font name with weight and style
pub fn get_standard_font_name(
    family: &str,
    weight: FontWeight,
    style: FontStyle,
) -> Option<String> {
    resolve_standard_font_family(family)
        .map(|f| f.font_name(weight, style).to_string())
}

/// Font metrics for layout and rendering
#[derive(Debug, Clone, PartialEq)]
pub struct FontMetrics {
    /// Glyph widths indexed by character code
    pub glyph_widths: HashMap<u32, u16>,
    /// Default width for missing glyphs
    pub default_width: u16,
    /// Ascent in font units
    pub ascent: i16,
    /// Descent in font units (negative value)
    pub descent: i16,
    /// X-height (height of lowercase x)
    pub x_height: i16,
    /// Capital height
    pub cap_height: i16,
    /// Units per em (font scale factor)
    pub units_per_em: u16,
    /// Line gap
    pub line_gap: i16,
    /// Italic angle in degrees (negative for italic)
    pub italic_angle: f32,
    /// Whether this is a monospace font
    pub is_monospace: bool,
}

impl Default for FontMetrics {
    fn default() -> Self {
        Self {
            glyph_widths: HashMap::new(),
            default_width: 500,
            ascent: 800,
            descent: -200,
            x_height: 500,
            cap_height: 700,
            units_per_em: 1000,
            line_gap: 0,
            italic_angle: 0.0,
            is_monospace: false,
        }
    }
}

impl FontMetrics {
    /// Create default metrics for standard fonts
    pub fn for_standard_font(name: &str) -> Self {
        let lower = name.to_ascii_lowercase();
        
        if lower.contains("helvetica") {
            Self::helvetica()
        } else if lower.contains("times") {
            Self::times_roman()
        } else if lower.contains("courier") {
            Self::courier()
        } else {
            Self::default()
        }
    }

    /// Create metrics for Helvetica (approximate values)
    pub fn helvetica() -> Self {
        let mut widths = HashMap::new();
        
        // Approximate widths for common characters (in 1000 unit em)
        for c in 'a'..='z' {
            widths.insert(c as u32, match c {
                'i' | 'l' | 'j' => 280,
                'f' | 'r' | 't' => 300,
                'm' | 'w' => 840,
                _ => 560,
            });
        }
        for c in 'A'..='Z' {
            widths.insert(c as u32, match c {
                'I' => 360,
                'M' | 'W' => 920,
                _ => 720,
            });
        }
        for c in '0'..='9' {
            widths.insert(c as u32, 560);
        }
        widths.insert(' ' as u32, 280);
        widths.insert('.' as u32, 280);
        widths.insert(',' as u32, 280);
        widths.insert('-' as u32, 340);
        widths.insert('_' as u32, 560);
        
        Self {
            glyph_widths: widths,
            default_width: 560,
            ascent: 718,
            descent: -207,
            x_height: 523,
            cap_height: 718,
            units_per_em: 1000,
            line_gap: 0,
            italic_angle: 0.0,
            is_monospace: false,
        }
    }

    /// Create metrics for Times Roman
    pub fn times_roman() -> Self {
        let mut widths = HashMap::new();
        
        for c in 'a'..='z' {
            widths.insert(c as u32, match c {
                'i' | 'l' => 280,
                'j' => 280,
                'm' | 'w' => 780,
                _ => 500,
            });
        }
        for c in 'A'..='Z' {
            widths.insert(c as u32, match c {
                'I' => 360,
                'M' | 'W' => 940,
                _ => 720,
            });
        }
        for c in '0'..='9' {
            widths.insert(c as u32, 500);
        }
        widths.insert(' ' as u32, 250);
        widths.insert('.' as u32, 250);
        widths.insert(',' as u32, 250);
        widths.insert('-' as u32, 333);
        widths.insert('_' as u32, 500);
        
        Self {
            glyph_widths: widths,
            default_width: 500,
            ascent: 683,
            descent: -217,
            x_height: 450,
            cap_height: 662,
            units_per_em: 1000,
            line_gap: 0,
            italic_angle: 0.0,
            is_monospace: false,
        }
    }

    /// Create metrics for Courier (monospace)
    pub fn courier() -> Self {
        let mut widths = HashMap::new();
        
        // All characters have same width in monospace
        for c in 32u32..=255 {
            widths.insert(c, 600);
        }
        
        Self {
            glyph_widths: widths,
            default_width: 600,
            ascent: 629,
            descent: -157,
            x_height: 441,
            cap_height: 562,
            units_per_em: 1000,
            line_gap: 0,
            italic_angle: 0.0,
            is_monospace: true,
        }
    }

    /// Create metrics from TrueType font data
    pub fn from_ttf(font_data: &[u8]) -> Option<Self> {
        use ttf_parser::Face;
        
        let face = Face::parse(font_data, 0).ok()?;
        
        let units_per_em = face.units_per_em()?;
        let ascent = face.ascender();
        let descent = face.descender();
        let x_height = face.x_height().unwrap_or((ascent as f32 * 0.5) as i16);
        let cap_height = face.capital_height().unwrap_or(ascent);
        
        // Collect glyph widths
        let mut glyph_widths = HashMap::new();
        for c in 32u32..=255 {
            if let Some(glyph_id) = face.glyph_index(char::from_u32(c)?) {
                if let Some(advance) = face.glyph_hor_advance(glyph_id) {
                    glyph_widths.insert(c, advance);
                }
            }
        }
        
        // Check if monospace by comparing widths
        let is_monospace = face.is_monospaced();
        
        Some(Self {
            glyph_widths,
            default_width: glyph_widths.get(&('m' as u32)).copied().unwrap_or(500),
            ascent,
            descent,
            x_height,
            cap_height,
            units_per_em,
            line_gap: face.line_gap(),
            italic_angle: face.italic_angle().unwrap_or(0.0) as f32,
            is_monospace,
        })
    }

    /// Get the scale factor for converting font units to points
    pub fn scale_factor(&self, font_size: f32) -> f32 {
        font_size / self.units_per_em as f32
    }

    /// Convert font units to points at a given font size
    pub fn to_points(&self, units: i16, font_size: f32) -> f32 {
        units as f32 * self.scale_factor(font_size)
    }

    /// Get the line height at a given font size
    pub fn line_height(&self, font_size: f32, line_height_ratio: f32) -> f32 {
        let metrics_height = (self.ascent - self.descent) as f32 * self.scale_factor(font_size);
        metrics_height.max(font_size * line_height_ratio)
    }

    /// Get the default line height (1.2 * font-size)
    pub fn default_line_height(&self, font_size: f32) -> f32 {
        self.line_height(font_size, 1.2)
    }

    /// Get width of a string in thousandths of a unit
    pub fn string_width(&self, s: &str, font_size: f32) -> f32 {
        let mut total = 0u32;
        for c in s.chars() {
            total += self.glyph_widths.get(&(c as u32))
                .copied()
                .unwrap_or(self.default_width) as u32;
        }
        (total as f32 / self.units_per_em as f32) * font_size
    }

    /// Get width of a single character
    pub fn char_width(&self, c: char, font_size: f32) -> f32 {
        let width = self.glyph_widths.get(&(c as u32))
            .copied()
            .unwrap_or(self.default_width);
        (width as f32 / self.units_per_em as f32) * font_size
    }

    /// Get the ascent in points
    pub fn ascent_pt(&self, font_size: f32) -> f32 {
        self.to_points(self.ascent, font_size)
    }

    /// Get the descent in points (absolute value)
    pub fn descent_pt(&self, font_size: f32) -> f32 {
        self.to_points(self.descent.abs() as i16, font_size)
    }

    /// Get the x-height in points
    pub fn x_height_pt(&self, font_size: f32) -> f32 {
        self.to_points(self.x_height, font_size)
    }

    /// Get the cap height in points
    pub fn cap_height_pt(&self, font_size: f32) -> f32 {
        self.to_points(self.cap_height, font_size)
    }
}

/// TrueType font loader and parser
#[derive(Debug)]
pub struct TtfFontLoader;

impl TtfFontLoader {
    /// Parse a TrueType font and extract metrics
    pub fn parse(font_data: &[u8]) -> Option<(FontMetrics, FontDescriptor)> {
        Self::parse_with_ttf_parser(font_data)
    }

    /// Parse using ttf-parser crate
    #[cfg(feature = "ttf-parser")]
    fn parse_with_ttf_parser(font_data: &[u8]) -> Option<(FontMetrics, FontDescriptor)> {
        use ttf_parser::Face;
        
        let face = Face::parse(font_data, 0).ok()?;
        
        let units_per_em = face.units_per_em()?;
        let ascent = face.ascender();
        let descent = face.descender();
        let x_height = face.x_height().unwrap_or((ascent as f32 * 0.5) as i16);
        let cap_height = face.capital_height().unwrap_or(ascent);
        let line_gap = face.line_gap();
        let italic_angle = face.italic_angle().unwrap_or(0.0);
        
        // Collect glyph widths and calculate average
        let mut glyph_widths = HashMap::new();
        let mut total_width: u32 = 0;
        let mut count: u32 = 0;
        
        for c in 32u32..=255 {
            if let Some(char) = char::from_u32(c) {
                if let Some(glyph_id) = face.glyph_index(char) {
                    if let Some(advance) = face.glyph_hor_advance(glyph_id) {
                        glyph_widths.insert(c, advance);
                        total_width += advance as u32;
                        count += 1;
                    }
                }
            }
        }
        
        let avg_width = if count > 0 { (total_width / count) as i32 } else { 500 };
        let default_width = glyph_widths.get(&('m' as u32)).copied().unwrap_or(500) as i32;
        
        // Get font bounding box
        let bbox = face.global_bounding_box();
        let font_bbox = (
            bbox.x_min as i32,
            bbox.y_min as i32,
            bbox.x_max as i32,
            bbox.y_max as i32,
        );
        
        // Calculate flags
        let mut flags: u32 = 0;
        if face.is_monospaced() {
            flags |= 1 << 0; // FixedPitch
        }
        if face.has_character_map() {
            flags |= 1 << 2; // Symbolic (has non-standard encoding)
        }
        // flags |= 1 << 5; // Nonsymbolic (standard Latin encoding)
        
        let metrics = FontMetrics {
            glyph_widths,
            default_width: default_width as u16,
            ascent,
            descent,
            x_height,
            cap_height,
            units_per_em,
            line_gap,
            italic_angle: italic_angle as f32,
            is_monospace: face.is_monospaced(),
        };
        
        let font_name = face.names().into_iter()
            .find(|n| n.name_id == ttf_parser::name_ids::FULL_NAME)
            .and_then(|n| n.to_string())
            .unwrap_or_else(|| "Unknown".to_string())
            .replace(" ", "-");
        
        let descriptor = FontDescriptor {
            ascent: ascent as i32,
            descent: descent as i32,
            cap_height: cap_height as i32,
            flags,
            font_bbox,
            italic_angle: (italic_angle * 10.0) as i32, // PDF uses tenths of degrees
            stem_v: (avg_width / 10).max(50), // Approximate stem width
            x_height: x_height as i32,
            leading: line_gap as i32,
            max_width: face.global_bounding_box().x_max as i32,
            avg_width,
            missing_width: default_width,
            font_file2: None,
            font_name,
            char_set: None,
        };
        
        Some((metrics, descriptor))
    }

    /// Fallback parsing without ttf-parser (uses hardcoded defaults)
    #[cfg(not(feature = "ttf-parser"))]
    fn parse_with_ttf_parser(_font_data: &[u8]) -> Option<(FontMetrics, FontDescriptor)> {
        None
    }
}

/// Font subsetting for reduced PDF size
#[derive(Debug)]
pub struct FontSubset {
    /// The subset name
    pub name: String,
    /// Characters included in this subset
    pub chars: Vec<char>,
    /// Glyph IDs in the original font
    pub glyph_ids: Vec<u16>,
}

impl FontSubset {
    /// Create a new font subset for the given text
    pub fn new(name: impl Into<String>, text: &str) -> Self {
        let name = name.into();
        let mut chars: Vec<char> = text.chars().collect();
        chars.sort_unstable();
        chars.dedup();
        
        // Add space if not present
        if !chars.contains(&' ') {
            chars.push(' ');
        }
        
        // Add basic punctuation if not present
        for c in ['.', ',', '-', '_', '(', ')', '[', ']', '{', '}', ':', ';', '!', '?', '"', '\''] {
            if !chars.contains(&c) {
                chars.push(c);
            }
        }
        
        Self {
            name,
            chars,
            glyph_ids: Vec::new(),
        }
    }

    /// Check if a character is in this subset
    pub fn contains(&self, c: char) -> bool {
        self.chars.contains(&c)
    }

    /// Get the number of glyphs in this subset
    pub fn len(&self) -> usize {
        self.chars.len()
    }

    pub fn is_empty(&self) -> bool {
        self.chars.is_empty()
    }
}

/// Font cache for managing loaded fonts
#[derive(Debug, Default)]
pub struct FontCache {
    fonts: HashMap<String, PdfFont>,
    metrics: HashMap<String, FontMetrics>,
}

impl FontCache {
    /// Create a new font cache
    pub fn new() -> Self {
        Self {
            fonts: HashMap::new(),
            metrics: HashMap::new(),
        }
    }

    /// Add a font to the cache
    pub fn add_font(&mut self, name: impl Into<String>, font: PdfFont) {
        let name = name.into();
        if let Some(ref metrics) = font.metrics {
            self.metrics.insert(name.clone(), metrics.clone());
        }
        self.fonts.insert(name, font);
    }

    /// Get a font by name
    pub fn get_font(&self, name: &str) -> Option<&PdfFont> {
        self.fonts.get(name)
    }

    /// Get metrics by font name
    pub fn get_metrics(&self, name: &str) -> Option<&FontMetrics> {
        self.metrics.get(name)
    }

    /// Check if a font exists
    pub fn has_font(&self, name: &str) -> bool {
        self.fonts.contains_key(name)
    }

    /// Get all fonts
    pub fn fonts(&self) -> &HashMap<String, PdfFont> {
        &self.fonts
    }

    /// Create a font cache with standard fonts pre-loaded
    pub fn with_standard_fonts() -> Self {
        let mut cache = Self::new();
        
        // Helvetica variants
        cache.add_font("Helvetica", PdfFont::standard("F1", "Helvetica"));
        cache.add_font("Helvetica-Bold", PdfFont::standard("F2", "Helvetica-Bold"));
        cache.add_font("Helvetica-Oblique", PdfFont::standard("F3", "Helvetica-Oblique"));
        cache.add_font("Helvetica-BoldOblique", PdfFont::standard("F4", "Helvetica-BoldOblique"));
        
        // Times variants
        cache.add_font("Times-Roman", PdfFont::standard("F5", "Times-Roman"));
        cache.add_font("Times-Bold", PdfFont::standard("F6", "Times-Bold"));
        cache.add_font("Times-Italic", PdfFont::standard("F7", "Times-Italic"));
        cache.add_font("Times-BoldItalic", PdfFont::standard("F8", "Times-BoldItalic"));
        
        // Courier variants
        cache.add_font("Courier", PdfFont::standard("F9", "Courier"));
        cache.add_font("Courier-Bold", PdfFont::standard("F10", "Courier-Bold"));
        cache.add_font("Courier-Oblique", PdfFont::standard("F11", "Courier-Oblique"));
        cache.add_font("Courier-BoldOblique", PdfFont::standard("F12", "Courier-BoldOblique"));
        
        // Symbol and ZapfDingbats
        cache.add_font("Symbol", PdfFont::standard("F13", "Symbol"));
        cache.add_font("ZapfDingbats", PdfFont::standard("F14", "ZapfDingbats"));
        
        cache
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
        
        // Test character widths
        let i_width = metrics.char_width('i', 12.0);
        let m_width = metrics.char_width('m', 12.0);
        assert!(i_width < m_width);
    }

    #[test]
    fn test_standard_font_family() {
        assert_eq!(
            StandardFontFamily::Helvetica.font_name(FontWeight::Normal, FontStyle::Normal),
            "Helvetica"
        );
        assert_eq!(
            StandardFontFamily::Helvetica.font_name(FontWeight::Bold, FontStyle::Normal),
            "Helvetica-Bold"
        );
        assert_eq!(
            StandardFontFamily::Times.font_name(FontWeight::Normal, FontStyle::Italic),
            "Times-Italic"
        );
    }

    #[test]
    fn test_font_resolution() {
        assert_eq!(
            resolve_standard_font_family("Arial"),
            Some(StandardFontFamily::Helvetica)
        );
        assert_eq!(
            resolve_standard_font_family("Times New Roman"),
            Some(StandardFontFamily::Times)
        );
    }

    #[test]
    fn test_font_subset() {
        let subset = FontSubset::new("subset1", "Hello World!");
        assert!(subset.contains('H'));
        assert!(subset.contains('e'));
        assert!(subset.contains(' '));
        assert!(!subset.is_empty());
    }

    #[test]
    fn test_font_cache() {
        let cache = FontCache::with_standard_fonts();
        assert!(cache.has_font("Helvetica"));
        assert!(cache.has_font("Times-Roman"));
        assert!(cache.get_metrics("Helvetica").is_some());
    }

    #[test]
    fn test_line_height_calculation() {
        let metrics = FontMetrics::helvetica();
        let lh = metrics.line_height(12.0, 1.5);
        assert!(lh >= 12.0 * 1.5);
    }
}
