//! Font Loading Implementation
//!
//! Handles loading fonts from various sources:
//! - Local files (TTF, OTF)
//! - HTTP URLs for @font-face
//! - Data URIs
//! - System fonts

use super::{LoadedFont, FontFormat, FontWeight, FontStyle, FontFaceRule, FontSource};
use crate::pdf::font::{FontMetrics, TtfFontLoader, FontDescriptor};
use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

/// Font loader for fetching and parsing fonts
#[derive(Debug, Default)]
pub struct FontLoader {
    /// Base URL for resolving relative URLs
    base_url: Option<String>,
    /// Cache of loaded fonts by URL
    cache: HashMap<String, LoadedFont>,
    /// Font loading options
    options: FontLoadOptions,
}

/// Font loading options
#[derive(Debug, Clone)]
pub struct FontLoadOptions {
    /// Maximum font file size in bytes (default: 10MB)
    pub max_size: usize,
    /// Timeout for network requests in seconds
    pub timeout_seconds: u64,
    /// Whether to allow loading from URLs
    pub allow_network: bool,
    /// User agent for HTTP requests
    pub user_agent: String,
}

impl Default for FontLoadOptions {
    fn default() -> Self {
        Self {
            max_size: 10 * 1024 * 1024, // 10MB
            timeout_seconds: 30,
            allow_network: true,
            user_agent: "html2pdf-rs/0.1.0".to_string(),
        }
    }
}

impl FontLoader {
    /// Create a new font loader
    pub fn new() -> Self {
        Self::default()
    }

    /// Create with options
    pub fn with_options(options: FontLoadOptions) -> Self {
        Self {
            options,
            ..Default::default()
        }
    }

    /// Set base URL for resolving relative URLs
    pub fn set_base_url(&mut self, url: impl Into<String>) {
        self.base_url = Some(url.into());
    }

    /// Load a font from a @font-face rule
    pub fn load_font_face(&mut self, rule: &FontFaceRule) -> Result<LoadedFont, FontLoadError> {
        // Try each source in order
        for source in &rule.sources {
            match self.load_source(source) {
                Ok(data) => {
                    if let Some(font) = self.parse_font_data(&data, &rule.font_family)? {
                        let loaded = LoadedFont {
                            family: rule.font_family.clone(),
                            full_name: format!("{}-{}-{}", 
                                rule.font_family,
                                weight_to_name(rule.font_weight),
                                style_to_name(rule.font_style)
                            ),
                            weight: rule.font_weight,
                            style: rule.font_style,
                            format: source.format.unwrap_or(FontFormat::TrueType),
                            path: None,
                            data: Some(data),
                            metrics: font.0,
                            pdf_font: None,
                        };
                        return Ok(loaded);
                    }
                }
                Err(e) => {
                    // Log error and try next source
                    eprintln!("Failed to load font source {}: {}", source.url, e);
                    continue;
                }
            }
        }
        
        Err(FontLoadError::NoValidSource)
    }

    /// Load font data from a source
    fn load_source(&self, source: &FontSource) -> Result<Vec<u8>, FontLoadError> {
        if source.is_local {
            // Try to find local font
            self.load_local_font(&source.url)
        } else if source.url.starts_with("data:") {
            // Data URI
            self.load_data_uri(&source.url)
        } else if source.url.starts_with("http://") || source.url.starts_with("https://") {
            // Absolute URL
            if self.options.allow_network {
                self.load_from_url(&source.url)
            } else {
                Err(FontLoadError::NetworkDisabled)
            }
        } else {
            // Relative URL or file path
            self.load_from_path(&source.url)
        }
    }

    /// Load a local font by name
    fn load_local_font(&self, name: &str) -> Result<Vec<u8>, FontLoadError> {
        // Try to find font in system directories
        let system_dirs = self.get_system_font_directories();
        
        for dir in system_dirs {
            // Search for font files matching the name
            if let Ok(entries) = std::fs::read_dir(&dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if let Some(stem) = path.file_stem() {
                        if stem.to_string_lossy().eq_ignore_ascii_case(name) {
                            if let Ok(data) = fs::read(&path) {
                                if data.len() <= self.options.max_size {
                                    return Ok(data);
                                }
                            }
                        }
                    }
                }
            }
        }
        
        Err(FontLoadError::FontNotFound(name.to_string()))
    }

    /// Load from a data URI
    fn load_data_uri(&self, uri: &str) -> Result<Vec<u8>, FontLoadError> {
        // Parse data URI: data:[<mediatype>][;base64],<data>
        let prefix = "data:";
        if !uri.starts_with(prefix) {
            return Err(FontLoadError::InvalidDataUri);
        }
        
        let rest = &uri[prefix.len()..];
        
        // Find the comma separator
        let comma_pos = rest.find(',').ok_or(FontLoadError::InvalidDataUri)?;
        let header = &rest[..comma_pos];
        let data = &rest[comma_pos + 1..];
        
        // Check if base64 encoded
        if header.contains("base64") {
            base64::decode(data).map_err(|_| FontLoadError::InvalidDataUri)
        } else {
            // URL encoded
            urlencoding::decode(data)
                .map(|s| s.as_bytes().to_vec())
                .map_err(|_| FontLoadError::InvalidDataUri)
        }
    }

    /// Load from HTTP URL
    fn load_from_url(&self, _url: &str) -> Result<Vec<u8>, FontLoadError> {
        // For now, we return an error since we don't have an HTTP client
        // In a full implementation, this would use reqwest or similar
        Err(FontLoadError::NetworkNotImplemented)
    }

    /// Load from a file path
    fn load_from_path(&self, path: &str) -> Result<Vec<u8>, FontLoadError> {
        let path = Path::new(path);
        
        if !path.exists() {
            // Try resolving relative to base URL
            if let Some(ref base) = self.base_url {
                let base_path = Path::new(base);
                let resolved = base_path.parent().unwrap_or(base_path).join(path);
                if resolved.exists() {
                    return self.read_file(&resolved);
                }
            }
            return Err(FontLoadError::FontNotFound(path.to_string_lossy().to_string()));
        }
        
        self.read_file(path)
    }

    /// Read a font file
    fn read_file(&self, path: &Path) -> Result<Vec<u8>, FontLoadError> {
        let data = fs::read(path)?;
        
        if data.len() > self.options.max_size {
            return Err(FontLoadError::FontTooLarge(data.len()));
        }
        
        Ok(data)
    }

    /// Parse font data and extract metrics
    fn parse_font_data(&self, data: &[u8], _family: &str) -> Result<Option<(Option<FontMetrics>, Option<FontDescriptor>)>, FontLoadError> {
        // Try to parse as TrueType/OpenType
        if let Some((metrics, descriptor)) = TtfFontLoader::parse(data) {
            return Ok(Some((Some(metrics), Some(descriptor))));
        }
        
        // Fallback: create basic metrics
        Ok(Some((Some(FontMetrics::default()), None)))
    }

    /// Load a font file directly
    pub fn load_file(&mut self, path: &Path) -> Result<LoadedFont, FontLoadError> {
        let data = self.read_file(path)?;
        let format = FontFormat::from_extension(path);
        let family = path.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("Unknown")
            .to_string();
        
        let (metrics, _) = self.parse_font_data(&data, &family)?
            .unwrap_or((Some(FontMetrics::default()), None));
        
        let loaded = LoadedFont {
            family: family.clone(),
            full_name: family.clone(),
            weight: FontWeight::Normal,
            style: FontStyle::Normal,
            format,
            path: Some(path.to_path_buf()),
            data: Some(data),
            metrics,
            pdf_font: None,
        };
        
        Ok(loaded)
    }

    /// Get system font directories
    fn get_system_font_directories(&self) -> Vec<PathBuf> {
        let mut dirs = Vec::new();
        
        #[cfg(target_os = "macos")]
        {
            dirs.push(PathBuf::from("/System/Library/Fonts"));
            dirs.push(PathBuf::from("/Library/Fonts"));
            if let Some(home) = std::env::var_os("HOME") {
                dirs.push(PathBuf::from(home).join("Library/Fonts"));
            }
        }
        
        #[cfg(target_os = "linux")]
        {
            dirs.push(PathBuf::from("/usr/share/fonts"));
            dirs.push(PathBuf::from("/usr/local/share/fonts"));
            if let Some(home) = std::env::var_os("HOME") {
                dirs.push(PathBuf::from(home).join(".fonts"));
                dirs.push(PathBuf::from(home).join(".local/share/fonts"));
            }
        }
        
        #[cfg(target_os = "windows")]
        {
            if let Some(windir) = std::env::var_os("WINDIR") {
                dirs.push(PathBuf::from(windir).join("Fonts"));
            }
        }
        
        dirs
    }

    /// Get a cached font
    pub fn get_cached(&self, url: &str) -> Option<&LoadedFont> {
        self.cache.get(url)
    }

    /// Add a font to the cache
    pub fn cache_font(&mut self, url: impl Into<String>, font: LoadedFont) {
        self.cache.insert(url.into(), font);
    }
}

/// Font loading errors
#[derive(Debug)]
pub enum FontLoadError {
    Io(io::Error),
    FontNotFound(String),
    FontTooLarge(usize),
    InvalidDataUri,
    NetworkDisabled,
    NetworkNotImplemented,
    NoValidSource,
    ParseError(String),
}

impl std::fmt::Display for FontLoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FontLoadError::Io(e) => write!(f, "IO error: {}", e),
            FontLoadError::FontNotFound(name) => write!(f, "Font not found: {}", name),
            FontLoadError::FontTooLarge(size) => write!(f, "Font too large: {} bytes", size),
            FontLoadError::InvalidDataUri => write!(f, "Invalid data URI"),
            FontLoadError::NetworkDisabled => write!(f, "Network loading disabled"),
            FontLoadError::NetworkNotImplemented => write!(f, "Network loading not implemented"),
            FontLoadError::NoValidSource => write!(f, "No valid font source found"),
            FontLoadError::ParseError(msg) => write!(f, "Parse error: {}", msg),
        }
    }
}

impl std::error::Error for FontLoadError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            FontLoadError::Io(e) => Some(e),
            _ => None,
        }
    }
}

impl From<io::Error> for FontLoadError {
    fn from(e: io::Error) -> Self {
        FontLoadError::Io(e)
    }
}

/// Convert weight to name string
fn weight_to_name(weight: FontWeight) -> &'static str {
    match weight {
        FontWeight::Thin => "Thin",
        FontWeight::ExtraLight => "ExtraLight",
        FontWeight::Light => "Light",
        FontWeight::Normal => "Regular",
        FontWeight::Medium => "Medium",
        FontWeight::SemiBold => "SemiBold",
        FontWeight::Bold => "Bold",
        FontWeight::ExtraBold => "ExtraBold",
        FontWeight::Black => "Black",
    }
}

/// Convert style to name string
fn style_to_name(style: FontStyle) -> &'static str {
    match style {
        FontStyle::Normal => "Normal",
        FontStyle::Italic => "Italic",
        FontStyle::Oblique => "Oblique",
    }
}

/// Simple base64 decoder (for data URIs)
mod base64 {
    pub fn decode(input: &str) -> Result<Vec<u8>, ()> {

        
        let chars: Vec<char> = input.chars().collect();
        let mut result = Vec::new();
        
        let mut buffer: u32 = 0;
        let mut bits_collected: u8 = 0;
        
        for &c in &chars {
            let value = match c {
                'A'..='Z' => c as u32 - 'A' as u32,
                'a'..='z' => c as u32 - 'a' as u32 + 26,
                '0'..='9' => c as u32 - '0' as u32 + 52,
                '+' => 62,
                '/' => 63,
                '=' => break, // Padding
                _ => continue, // Ignore whitespace
            };
            
            buffer = (buffer << 6) | value;
            bits_collected += 6;
            
            if bits_collected >= 8 {
                bits_collected -= 8;
                result.push((buffer >> bits_collected) as u8);
                buffer &= (1 << bits_collected) - 1;
            }
        }
        
        Ok(result)
    }
}

/// URL decoding (simplified)
mod urlencoding {
    pub fn decode(input: &str) -> Result<String, ()> {
        let mut result = String::new();
        let chars: Vec<char> = input.chars().collect();
        let mut i = 0;
        
        while i < chars.len() {
            if chars[i] == '%' && i + 2 < chars.len() {
                let hex = &input[i+1..i+3];
                if let Ok(byte) = u8::from_str_radix(hex, 16) {
                    result.push(byte as char);
                    i += 3;
                    continue;
                }
            } else if chars[i] == '+' {
                result.push(' ');
            } else {
                result.push(chars[i]);
            }
            i += 1;
        }
        
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_font_loader_new() {
        let loader = FontLoader::new();
        assert!(loader.get_cached("test").is_none());
    }

    #[test]
    fn test_base64_decode() {
        let encoded = "SGVsbG8gV29ybGQ=";
        let decoded = base64::decode(encoded).unwrap();
        assert_eq!(String::from_utf8(decoded).unwrap(), "Hello World");
    }

    #[test]
    fn test_url_decode() {
        assert_eq!(urlencoding::decode("Hello%20World").unwrap(), "Hello World");
        assert_eq!(urlencoding::decode("Hello+World").unwrap(), "Hello World");
    }

    #[test]
    fn test_font_format_detection() {
        assert_eq!(FontFormat::from_extension(Path::new("test.ttf")), FontFormat::TrueType);
        assert_eq!(FontFormat::from_extension(Path::new("test.otf")), FontFormat::OpenType);
    }

    #[test]
    fn test_weight_to_name() {
        assert_eq!(weight_to_name(FontWeight::Normal), "Regular");
        assert_eq!(weight_to_name(FontWeight::Bold), "Bold");
    }
}
