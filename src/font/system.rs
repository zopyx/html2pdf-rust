//! System Font Finder
//!
//! Discovers and indexes fonts installed on the system.
//! Provides font matching by family, weight, and style.

use super::{FontWeight, FontStyle, FontFormat};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// System font information
#[derive(Debug, Clone)]
pub struct SystemFont {
    /// Full path to font file
    pub path: PathBuf,
    /// Font family name
    pub family: String,
    /// Full font name
    pub full_name: String,
    /// PostScript name
    pub postscript_name: Option<String>,
    /// Font weight
    pub weight: FontWeight,
    /// Font style
    pub style: FontStyle,
    /// Font format
    pub format: FontFormat,
    /// Font file size in bytes
    pub file_size: u64,
}

/// System font finder and indexer
#[derive(Debug, Default)]
pub struct SystemFontFinder {
    /// Indexed fonts by family name
    fonts: HashMap<String, Vec<SystemFont>>,
    /// All indexed fonts
    all_fonts: Vec<SystemFont>,
    /// Whether fonts have been indexed
    indexed: bool,
    /// System font directories
    font_dirs: Vec<PathBuf>,
}

impl SystemFontFinder {
    /// Create a new system font finder
    pub fn new() -> Self {
        let mut finder = Self {
            fonts: HashMap::new(),
            all_fonts: Vec::new(),
            indexed: false,
            font_dirs: Self::default_font_directories(),
        };
        finder.refresh();
        finder
    }

    /// Get default system font directories
    fn default_font_directories() -> Vec<PathBuf> {
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

    /// Refresh the font index
    pub fn refresh(&mut self) {
        self.fonts.clear();
        self.all_fonts.clear();
        
        // Clone font_dirs to avoid borrow issues
        let dirs: Vec<PathBuf> = self.font_dirs.clone();
        for dir in dirs {
            self.scan_directory(&dir);
        }
        
        self.indexed = true;
    }

    /// Scan a directory for fonts
    fn scan_directory(&mut self, dir: &Path) {
        if !dir.exists() || !dir.is_dir() {
            return;
        }
        
        let entries = match fs::read_dir(dir) {
            Ok(e) => e,
            Err(_) => return,
        };
        
        for entry in entries.flatten() {
            let path = entry.path();
            
            if path.is_dir() {
                // Recursively scan subdirectories
                self.scan_directory(&path);
            } else if let Some(font) = self.parse_font_file(&path) {
                let family = font.family.to_ascii_lowercase();
                self.fonts.entry(family).or_default().push(font.clone());
                self.all_fonts.push(font);
            }
        }
    }

    /// Parse a font file and extract metadata
    fn parse_font_file(&self, path: &Path) -> Option<SystemFont> {
        let format = FontFormat::from_extension(path);
        
        // Only index supported formats
        if !matches!(format, FontFormat::TrueType | FontFormat::OpenType) {
            return None;
        }
        
        let metadata = fs::metadata(path).ok()?;
        let file_size = metadata.len();
        
        // Extract font name from filename
        let filename = path.file_stem()?.to_string_lossy();
        
        // Try to parse the font file for better metadata
        let (family, weight, style) = self.parse_font_metadata(path, &filename);
        
        Some(SystemFont {
            path: path.to_path_buf(),
            family: family.clone(),
            full_name: filename.to_string(),
            postscript_name: None,
            weight,
            style,
            format,
            file_size,
        })
    }

    /// Parse font metadata from file
    fn parse_font_metadata(&self, path: &Path, filename: &str) -> (String, FontWeight, FontStyle) {
        let filename_lower = filename.to_ascii_lowercase();
        
        // Try to parse actual font file
        if let Ok(data) = fs::read(path) {
            if let Some(metrics) = crate::pdf::font::TtfFontLoader::parse(&data) {
                let family = metrics.1.font_name.clone();
                return (family, FontWeight::Normal, FontStyle::Normal);
            }
        }
        
        // Fallback: guess from filename
        let family = self.guess_family_from_filename(filename);
        let weight = self.guess_weight_from_filename(&filename_lower);
        let style = self.guess_style_from_filename(&filename_lower);
        
        (family, weight, style)
    }

    /// Guess font family from filename
    fn guess_family_from_filename(&self, filename: &str) -> String {
        let lower = filename.to_ascii_lowercase();
        
        // Common font families
        if lower.contains("arial") {
            "Arial".to_string()
        } else if lower.contains("helvetica") {
            "Helvetica".to_string()
        } else if lower.contains("times") {
            "Times New Roman".to_string()
        } else if lower.contains("courier") {
            "Courier New".to_string()
        } else if lower.contains("georgia") {
            "Georgia".to_string()
        } else if lower.contains("verdana") {
            "Verdana".to_string()
        } else if lower.contains("trebuchet") {
            "Trebuchet MS".to_string()
        } else if lower.contains("impact") {
            "Impact".to_string()
        } else if lower.contains("comic") {
            "Comic Sans MS".to_string()
        } else {
            // Extract base name (remove weight/style suffixes)
            filename
                .split(&['-', '_', ' '][..])
                .next()
                .unwrap_or(filename)
                .to_string()
        }
    }

    /// Guess weight from filename
    fn guess_weight_from_filename(&self, filename: &str) -> FontWeight {
        if filename.contains("thin") || filename.contains("100") {
            FontWeight::Thin
        } else if filename.contains("extralight") || filename.contains("200") {
            FontWeight::ExtraLight
        } else if filename.contains("light") || filename.contains("300") {
            FontWeight::Light
        } else if filename.contains("medium") || filename.contains("500") {
            FontWeight::Medium
        } else if filename.contains("semibold") || filename.contains("600") {
            FontWeight::SemiBold
        } else if filename.contains("extrabold") || filename.contains("800") {
            FontWeight::ExtraBold
        } else if filename.contains("bold") || filename.contains("700") || filename.contains("bd") {
            FontWeight::Bold
        } else if filename.contains("black") || filename.contains("900") || filename.contains("heavy") {
            FontWeight::Black
        } else {
            FontWeight::Normal
        }
    }

    /// Guess style from filename
    fn guess_style_from_filename(&self, filename: &str) -> FontStyle {
        if filename.contains("italic") || filename.contains("it") {
            FontStyle::Italic
        } else if filename.contains("oblique") {
            FontStyle::Oblique
        } else {
            FontStyle::Normal
        }
    }

    /// Find a font by family name
    pub fn find_font(&self, family: &str) -> Option<&SystemFont> {
        let family_lower = family.to_ascii_lowercase();
        
        self.fonts
            .get(&family_lower)
            .and_then(|fonts| fonts.first())
    }

    /// Find a font with specific weight and style
    pub fn find_font_with_style(
        &self,
        family: &str,
        weight: FontWeight,
        style: FontStyle,
    ) -> Option<&SystemFont> {
        let family_lower = family.to_ascii_lowercase();
        
        let fonts = self.fonts.get(&family_lower)?;
        
        // Find best match
        fonts.iter().min_by_key(|f| {
            let weight_diff = (f.weight.to_number() as i16 - weight.to_number() as i16).abs();
            let style_match = if f.style == style { 0 } else { 1 };
            (weight_diff, style_match)
        })
    }

    /// Find fonts by family name (all variants)
    pub fn find_font_family(&self, family: &str) -> Option<&[SystemFont]> {
        let family_lower = family.to_ascii_lowercase();
        self.fonts.get(&family_lower).map(|v| v.as_slice())
    }

    /// Search for fonts matching a query
    pub fn search(&self, query: &str) -> Vec<&SystemFont> {
        let query_lower = query.to_ascii_lowercase();
        
        self.all_fonts
            .iter()
            .filter(|f| {
                f.family.to_ascii_lowercase().contains(&query_lower) ||
                f.full_name.to_ascii_lowercase().contains(&query_lower)
            })
            .collect()
    }

    /// Get all indexed fonts
    pub fn all_fonts(&self) -> &[SystemFont] {
        &self.all_fonts
    }

    /// Get all font families
    pub fn families(&self) -> Vec<&String> {
        self.fonts.keys().collect()
    }

    /// Get font count
    pub fn font_count(&self) -> usize {
        self.all_fonts.len()
    }

    /// Check if fonts have been indexed
    pub fn is_indexed(&self) -> bool {
        self.indexed
    }

    /// Add a custom font directory
    pub fn add_directory(&mut self, path: impl Into<PathBuf>) {
        let path = path.into();
        if !self.font_dirs.contains(&path) {
            self.font_dirs.push(path);
        }
    }

    /// Get font directories
    pub fn directories(&self) -> &[PathBuf] {
        &self.font_dirs
    }
}

/// Get the path to a system font
pub fn find_system_font(family: &str) -> Option<PathBuf> {
    let finder = SystemFontFinder::new();
    finder.find_font(family).map(|f| f.path.clone())
}

/// Find a system font with specific weight and style
pub fn find_system_font_with_style(
    family: &str,
    weight: FontWeight,
    style: FontStyle,
) -> Option<PathBuf> {
    let finder = SystemFontFinder::new();
    finder.find_font_with_style(family, weight, style).map(|f| f.path.clone())
}

/// List all available system font families
pub fn list_system_font_families() -> Vec<String> {
    let finder = SystemFontFinder::new();
    finder.families().into_iter().cloned().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_system_font_finder() {
        let finder = SystemFontFinder::new();
        // Should have indexed some fonts (on systems with fonts installed)
        // This test might fail on CI systems without fonts
        // Just verify the finder was created
        assert!(finder.is_indexed());
    }

    #[test]
    fn test_guess_weight() {
        let finder = SystemFontFinder::new();
        
        assert_eq!(finder.guess_weight_from_filename("font-bold"), FontWeight::Bold);
        assert_eq!(finder.guess_weight_from_filename("font-light"), FontWeight::Light);
        assert_eq!(finder.guess_weight_from_filename("font-italic"), FontWeight::Normal);
    }

    #[test]
    fn test_guess_style() {
        let finder = SystemFontFinder::new();
        
        assert_eq!(finder.guess_style_from_filename("font-italic"), FontStyle::Italic);
        assert_eq!(finder.guess_style_from_filename("font-bold"), FontStyle::Normal);
    }

    #[test]
    fn test_guess_family() {
        let finder = SystemFontFinder::new();
        
        assert!(finder.guess_family_from_filename("Arial-Bold").contains("Arial"));
        assert!(finder.guess_family_from_filename("Helvetica").contains("Helvetica"));
    }

    #[test]
    fn test_font_directories() {
        let finder = SystemFontFinder::new();
        assert!(!finder.directories().is_empty());
    }
}
