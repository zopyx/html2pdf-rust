//! Font Loading and Management
//!
//! This module provides comprehensive font support:
//! - System font detection
//! - Web font loading (@font-face)
//! - Font fallback chains
//! - Font caching
//! - @font-face parsing from CSS

use crate::css::{Declaration, CssValue};
use crate::pdf::font::{FontMetrics, PdfFont, FontCache};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

pub mod loader;
pub mod system;

pub use loader::FontLoader;
pub use system::SystemFontFinder;

/// A loaded font with its metadata
#[derive(Debug, Clone)]
pub struct LoadedFont {
    /// Font name/family
    pub family: String,
    /// Full font name (including weight/style)
    pub full_name: String,
    /// Font weight
    pub weight: FontWeight,
    /// Font style
    pub style: FontStyle,
    /// Font format
    pub format: FontFormat,
    /// Path to the font file (if loaded from disk)
    pub path: Option<PathBuf>,
    /// Raw font data
    pub data: Option<Vec<u8>>,
    /// Font metrics
    pub metrics: Option<FontMetrics>,
    /// PDF font reference
    pub pdf_font: Option<PdfFont>,
}

/// Font weight
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum FontWeight {
    Thin = 100,
    ExtraLight = 200,
    Light = 300,
    Normal = 400,
    Medium = 500,
    SemiBold = 600,
    Bold = 700,
    ExtraBold = 800,
    Black = 900,
}

impl FontWeight {
    /// Parse font weight from CSS value
    pub fn from_css(value: &CssValue) -> Self {
        match value {
            CssValue::Ident(s) => match s.as_str() {
                "normal" => FontWeight::Normal,
                "bold" => FontWeight::Bold,
                "lighter" => FontWeight::Light,
                "bolder" => FontWeight::Bold,
                _ => FontWeight::Normal,
            },
            CssValue::Number(n) => FontWeight::from_number(*n as u16),
            _ => FontWeight::Normal,
        }
    }

    /// Create from numeric weight
    pub fn from_number(n: u16) -> Self {
        match n {
            100 => FontWeight::Thin,
            200 => FontWeight::ExtraLight,
            300 => FontWeight::Light,
            400 | 500 => FontWeight::Normal,
            600 => FontWeight::SemiBold,
            700 => FontWeight::Bold,
            800 => FontWeight::ExtraBold,
            900 => FontWeight::Black,
            _ => FontWeight::Normal,
        }
    }

    /// Get the numeric value
    pub fn to_number(&self) -> u16 {
        *self as u16
    }

    /// Check if this is bold
    pub fn is_bold(&self) -> bool {
        *self >= FontWeight::Bold
    }
}

impl Default for FontWeight {
    fn default() -> Self {
        FontWeight::Normal
    }
}

/// Font style
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FontStyle {
    #[default]
    Normal,
    Italic,
    Oblique,
}

impl FontStyle {
    /// Parse from CSS value
    pub fn from_css(value: &CssValue) -> Self {
        match value {
            CssValue::Ident(s) => match s.as_str() {
                "normal" => FontStyle::Normal,
                "italic" => FontStyle::Italic,
                "oblique" => FontStyle::Oblique,
                _ => FontStyle::Normal,
            },
            _ => FontStyle::Normal,
        }
    }

    /// Check if this is italic/oblique
    pub fn is_italic(&self) -> bool {
        matches!(self, FontStyle::Italic | FontStyle::Oblique)
    }
}

/// Font format
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FontFormat {
    /// TrueType (.ttf)
    TrueType,
    /// OpenType (.otf)
    OpenType,
    /// WOFF (Web Open Font Format)
    Woff,
    /// WOFF2
    Woff2,
    /// Embedded OpenType
    Eot,
    /// SVG font
    Svg,
    /// Unknown format
    Unknown,
}

impl FontFormat {
    /// Detect format from file extension
    pub fn from_extension(path: &Path) -> Self {
        match path.extension().and_then(|s| s.to_str()) {
            Some("ttf") => FontFormat::TrueType,
            Some("otf") => FontFormat::OpenType,
            Some("woff") => FontFormat::Woff,
            Some("woff2") => FontFormat::Woff2,
            Some("eot") => FontFormat::Eot,
            Some("svg") => FontFormat::Svg,
            _ => FontFormat::Unknown,
        }
    }

    /// Detect from MIME type or format hint
    pub fn from_format_hint(hint: &str) -> Self {
        let lower = hint.to_ascii_lowercase();
        match lower.as_str() {
            "truetype" | "ttf" => FontFormat::TrueType,
            "opentype" | "otf" => FontFormat::OpenType,
            "woff" => FontFormat::Woff,
            "woff2" => FontFormat::Woff2,
            "eot" => FontFormat::Eot,
            "svg" => FontFormat::Svg,
            _ => FontFormat::Unknown,
        }
    }

    /// Check if this format is supported for embedding
    pub fn is_supported(&self) -> bool {
        matches!(self, FontFormat::TrueType | FontFormat::OpenType)
    }
}

/// @font-face rule parsed from CSS
#[derive(Debug, Clone, Default)]
pub struct FontFaceRule {
    /// Font family name
    pub font_family: String,
    /// Source URLs
    pub sources: Vec<FontSource>,
    /// Font style
    pub font_style: FontStyle,
    /// Font weight
    pub font_weight: FontWeight,
    /// Unicode range (optional)
    pub unicode_range: Option<String>,
    /// Font display
    pub font_display: FontDisplay,
    /// Font stretch
    pub font_stretch: Option<String>,
    /// Font variant
    pub font_variant: Option<String>,
}

/// Font display property
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FontDisplay {
    #[default]
    Auto,
    Block,
    Swap,
    Fallback,
    Optional,
}

impl FontDisplay {
    pub fn from_css(value: &CssValue) -> Self {
        match value {
            CssValue::Ident(s) => match s.as_str() {
                "auto" => FontDisplay::Auto,
                "block" => FontDisplay::Block,
                "swap" => FontDisplay::Swap,
                "fallback" => FontDisplay::Fallback,
                "optional" => FontDisplay::Optional,
                _ => FontDisplay::Auto,
            },
            _ => FontDisplay::Auto,
        }
    }
}

/// Font source from @font-face
#[derive(Debug, Clone)]
pub struct FontSource {
    /// URL or local font name
    pub url: String,
    /// Format hint
    pub format: Option<FontFormat>,
    /// Whether this is a local() reference
    pub is_local: bool,
}

impl FontFaceRule {
    /// Parse from CSS declarations
    pub fn from_declarations(declarations: &[Declaration]) -> Option<Self> {
        let mut rule = Self::default();
        
        for decl in declarations {
            match decl.name.as_str() {
                "font-family" => {
                    rule.font_family = match &decl.value {
                        CssValue::String(s) => s.clone(),
                        CssValue::Ident(s) => s.clone(),
                        _ => continue,
                    };
                }
                "src" => {
                    rule.sources = Self::parse_src(&decl.value);
                }
                "font-style" => {
                    rule.font_style = FontStyle::from_css(&decl.value);
                }
                "font-weight" => {
                    rule.font_weight = FontWeight::from_css(&decl.value);
                }
                "font-display" => {
                    rule.font_display = FontDisplay::from_css(&decl.value);
                }
                "unicode-range" => {
                    if let CssValue::String(s) = &decl.value {
                        rule.unicode_range = Some(s.clone());
                    }
                }
                _ => {}
            }
        }
        
        if rule.font_family.is_empty() || rule.sources.is_empty() {
            None
        } else {
            Some(rule)
        }
    }

    /// Parse the src property
    fn parse_src(value: &CssValue) -> Vec<FontSource> {
        let mut sources = Vec::new();
        
        // Handle list of sources
        let values = match value {
            CssValue::List(list) => list.clone(),
            _ => vec![value.clone()],
        };
        
        let mut i = 0;
        while i < values.len() {
            match &values[i] {
                CssValue::Function(f) if f.name == "url" => {
                    if let Some(CssValue::String(url)) = f.args.first() {
                        let mut format = None;
                        // Check for format() function after URL
                        if i + 1 < values.len() {
                            if let CssValue::Function(fmt) = &values[i + 1] {
                                if fmt.name == "format" {
                                    if let Some(CssValue::String(fmt_str)) = fmt.args.first() {
                                        format = Some(FontFormat::from_format_hint(fmt_str));
                                    }
                                    i += 1;
                                }
                            }
                        }
                        sources.push(FontSource {
                            url: url.clone(),
                            format,
                            is_local: false,
                        });
                    }
                }
                CssValue::Function(f) if f.name == "local" => {
                    if let Some(CssValue::String(name)) = f.args.first() {
                        sources.push(FontSource {
                            url: name.clone(),
                            format: None,
                            is_local: true,
                        });
                    }
                }
                _ => {}
            }
            i += 1;
        }
        
        sources
    }
}

/// Font family with fallback chain
#[derive(Debug, Clone)]
pub struct FontFamily {
    /// Primary font family name
    pub primary: String,
    /// Fallback fonts in order of preference
    pub fallbacks: Vec<String>,
    /// Generic family type
    pub generic_family: GenericFontFamily,
}

/// Generic font family types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum GenericFontFamily {
    #[default]
    Serif,
    SansSerif,
    Monospace,
    Cursive,
    Fantasy,
    SystemUI,
    UISerif,
    UISansSerif,
    UIMonospace,
}

impl GenericFontFamily {
    /// Get the default font for this generic family
    pub fn default_font_name(&self) -> &'static str {
        match self {
            GenericFontFamily::Serif | GenericFontFamily::UISerif => "Times-Roman",
            GenericFontFamily::SansSerif | GenericFontFamily::SystemUI | GenericFontFamily::UISansSerif => "Helvetica",
            GenericFontFamily::Monospace | GenericFontFamily::UIMonospace => "Courier",
            _ => "Helvetica",
        }
    }

    /// Parse from CSS value
    pub fn from_css(value: &str) -> Option<Self> {
        match value.to_ascii_lowercase().as_str() {
            "serif" => Some(GenericFontFamily::Serif),
            "sans-serif" => Some(GenericFontFamily::SansSerif),
            "monospace" => Some(GenericFontFamily::Monospace),
            "cursive" => Some(GenericFontFamily::Cursive),
            "fantasy" => Some(GenericFontFamily::Fantasy),
            "system-ui" => Some(GenericFontFamily::SystemUI),
            "ui-serif" => Some(GenericFontFamily::UISerif),
            "ui-sans-serif" => Some(GenericFontFamily::UISansSerif),
            "ui-monospace" => Some(GenericFontFamily::UIMonospace),
            _ => None,
        }
    }
}

/// Font manager - central registry for all fonts
#[derive(Debug, Default)]
pub struct FontManager {
    /// Loaded fonts by family name
    fonts: HashMap<String, Vec<LoadedFont>>,
    /// @font-face rules
    font_faces: Vec<FontFaceRule>,
    /// Font cache for standard fonts
    standard_cache: FontCache,
    /// System font finder
    system_finder: SystemFontFinder,
}

impl FontManager {
    /// Create a new font manager
    pub fn new() -> Self {
        Self {
            fonts: HashMap::new(),
            font_faces: Vec::new(),
            standard_cache: FontCache::with_standard_fonts(),
            system_finder: SystemFontFinder::new(),
        }
    }

    /// Initialize with standard fonts
    pub fn with_standard_fonts() -> Self {
        let mut manager = Self::new();
        manager.standard_cache = FontCache::with_standard_fonts();
        manager
    }

    /// Add a font face rule from CSS
    pub fn add_font_face(&mut self, rule: FontFaceRule) {
        self.font_faces.push(rule);
    }

    /// Parse and add @font-face from declarations
    pub fn add_font_face_from_declarations(&mut self, declarations: &[Declaration]) {
        if let Some(rule) = FontFaceRule::from_declarations(declarations) {
            self.add_font_face(rule);
        }
    }

    /// Register a loaded font
    pub fn register_font(&mut self, font: LoadedFont) {
        let family = font.family.to_ascii_lowercase();
        self.fonts.entry(family).or_default().push(font);
    }

    /// Find a font matching the given criteria
    pub fn find_font(
        &self,
        family: &str,
        weight: FontWeight,
        style: FontStyle,
    ) -> Option<&LoadedFont> {
        let family_lower = family.to_ascii_lowercase();
        
        // Check loaded fonts first
        if let Some(fonts) = self.fonts.get(&family_lower) {
            // Find best match
            return fonts.iter().min_by_key(|f| {
                let weight_diff = (f.weight.to_number() as i16 - weight.to_number() as i16).abs();
                let style_match = if f.style == style { 0 } else { 1 };
                (weight_diff, style_match)
            });
        }
        
        None
    }

    /// Resolve a font family to a font name
    pub fn resolve_font_family(
        &self,
        families: &[String],
        weight: FontWeight,
        style: FontStyle,
    ) -> String {
        for family in families {
            // Check if it's a generic family
            if let Some(generic) = GenericFontFamily::from_css(family) {
                return generic.default_font_name().to_string();
            }
            
            // Check loaded fonts
            if let Some(font) = self.find_font(family, weight, style) {
                return font.full_name.clone();
            }
            
            // Check @font-face rules
            for rule in &self.font_faces {
                if rule.font_family.eq_ignore_ascii_case(family) {
                    // This font is available as @font-face
                    return rule.font_family.clone();
                }
            }
            
            // Check standard fonts
            if let Some(name) = crate::pdf::font::get_standard_font_name(family, 
                if weight.is_bold() { crate::pdf::font::FontWeight::Bold } else { crate::pdf::font::FontWeight::Normal },
                if style.is_italic() { crate::pdf::font::FontStyle::Italic } else { crate::pdf::font::FontStyle::Normal }) {
                return name;
            }
        }
        
        // Fallback to Helvetica
        "Helvetica".to_string()
    }

    /// Get standard font cache
    pub fn standard_cache(&self) -> &FontCache {
        &self.standard_cache
    }

    /// Get mutable standard font cache
    pub fn standard_cache_mut(&mut self) -> &mut FontCache {
        &mut self.standard_cache
    }

    /// Get all @font-face rules
    pub fn font_faces(&self) -> &[FontFaceRule] {
        &self.font_faces
    }

    /// Get system font finder
    pub fn system_finder(&self) -> &SystemFontFinder {
        &self.system_finder
    }

    /// Load system fonts into cache
    pub fn load_system_fonts(&mut self) {
        self.system_finder.refresh();
    }
}

/// Parse a font-family CSS value into a list of family names
pub fn parse_font_family_list(value: &CssValue) -> Vec<String> {
    match value {
        CssValue::List(values) => values
            .iter()
            .filter_map(|v| match v {
                CssValue::String(s) => Some(s.clone()),
                CssValue::Ident(s) => Some(s.clone()),
                _ => None,
            })
            .collect(),
        CssValue::String(s) => vec![s.clone()],
        CssValue::Ident(s) => vec![s.clone()],
        _ => vec!["serif".to_string()],
    }
}

/// Get the default font for a generic family
pub fn get_generic_font_family(family: &str) -> Option<GenericFontFamily> {
    GenericFontFamily::from_css(family)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_font_weight() {
        assert_eq!(FontWeight::from_number(400), FontWeight::Normal);
        assert_eq!(FontWeight::from_number(700), FontWeight::Bold);
        assert!(FontWeight::Bold.is_bold());
        assert!(!FontWeight::Normal.is_bold());
    }

    #[test]
    fn test_font_style() {
        assert!(FontStyle::Italic.is_italic());
        assert!(FontStyle::Oblique.is_italic());
        assert!(!FontStyle::Normal.is_italic());
    }

    #[test]
    fn test_font_format_from_extension() {
        assert_eq!(FontFormat::from_extension(Path::new("font.ttf")), FontFormat::TrueType);
        assert_eq!(FontFormat::from_extension(Path::new("font.otf")), FontFormat::OpenType);
        assert_eq!(FontFormat::from_extension(Path::new("font.woff")), FontFormat::Woff);
    }

    #[test]
    fn test_font_face_parse_src() {
        let mut func = crate::css::CssFunction::new("url");
        func.add_argument(CssValue::String("font.woff2".to_string()));
        let src = CssValue::Function(func);
        
        let sources = FontFaceRule::parse_src(&src);
        assert_eq!(sources.len(), 1);
        assert_eq!(sources[0].url, "font.woff2");
    }

    #[test]
    fn test_generic_font_family() {
        assert_eq!(
            GenericFontFamily::from_css("serif"),
            Some(GenericFontFamily::Serif)
        );
        assert_eq!(
            GenericFontFamily::Serif.default_font_name(),
            "Times-Roman"
        );
    }

    #[test]
    fn test_font_manager() {
        let manager = FontManager::with_standard_fonts();
        
        // Should resolve standard fonts
        let name = manager.resolve_font_family(
            &vec!["Helvetica".to_string()],
            FontWeight::Normal,
            FontStyle::Normal,
        );
        assert_eq!(name, "Helvetica");
    }
}
