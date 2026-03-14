//! PDF generation from scratch
//!
//! This module implements PDF generation without external dependencies,
//! supporting the full PDF 1.4 specification needed for document output.

mod object;
mod stream;
mod writer;
mod font;
mod image;

pub use object::{PdfObject, PdfReference, PdfDictionary, PdfArray};
pub use stream::{PdfStream, FlateEncode};
pub use writer::PdfWriter;
pub use font::PdfFont;
pub use image::PdfImage;

use crate::types::{Color, Rect, Point, Size};

/// A PDF page content builder
pub struct PageContent {
    content: Vec<u8>,
    current_font: Option<(String, f32)>,
    current_color: Option<Color>,
}

impl PageContent {
    pub fn new() -> Self {
        Self {
            content: Vec::new(),
            current_font: None,
            current_color: None,
        }
    }

    /// Start a new text object
    pub fn begin_text(&mut self) {
        self.content.extend_from_slice(b"BT\n");
    }

    /// End text object
    pub fn end_text(&mut self) {
        self.content.extend_from_slice(b"ET\n");
    }

    /// Set text position
    pub fn text_position(&mut self, x: f32, y: f32) {
        self.content.extend_from_slice(format!("{:.3} {:.3} Td\n", x, y).as_bytes());
    }

    /// Set font
    pub fn set_font(&mut self, font_name: &str, size: f32) {
        self.current_font = Some((font_name.to_string(), size));
        self.content.extend_from_slice(
            format!("/{} {:.3} Tf\n", font_name, size).as_bytes()
        );
    }

    /// Show text
    pub fn show_text(&mut self, text: &str) {
        // Escape special characters
        let escaped = text
            .replace('\\', "\\\\")
            .replace('(', "\\(")
            .replace(')', "\\)")
            .replace('\n', "\\n")
            .replace('\r', "\\r")
            .replace('\t', "\\t");
        self.content.extend_from_slice(format!("({}) Tj\n", escaped).as_bytes());
    }

    /// Set fill color
    pub fn set_fill_color(&mut self, color: Color) {
        self.current_color = Some(color);
        let (r, g, b) = color.to_pdf();
        self.content.extend_from_slice(
            format!("{:.3} {:.3} {:.3} rg\n", r, g, b).as_bytes()
        );
    }

    /// Set stroke color
    pub fn set_stroke_color(&mut self, color: Color) {
        let (r, g, b) = color.to_pdf();
        self.content.extend_from_slice(
            format!("{:.3} {:.3} {:.3} RG\n", r, g, b).as_bytes()
        );
    }

    /// Draw a rectangle
    pub fn draw_rect(&mut self, rect: Rect) {
        self.content.extend_from_slice(
            format!("{:.3} {:.3} {:.3} {:.3} re\n", rect.x, rect.y, rect.width, rect.height).as_bytes()
        );
    }

    /// Fill path
    pub fn fill(&mut self) {
        self.content.extend_from_slice(b"f\n");
    }

    /// Stroke path
    pub fn stroke(&mut self) {
        self.content.extend_from_slice(b"S\n");
    }

    /// Fill and stroke
    pub fn fill_and_stroke(&mut self) {
        self.content.extend_from_slice(b"B\n");
    }

    /// Draw a line
    pub fn draw_line(&mut self, from: Point, to: Point) {
        self.content.extend_from_slice(
            format!("{:.3} {:.3} m\n", from.x, from.y).as_bytes()
        );
        self.content.extend_from_slice(
            format!("{:.3} {:.3} l\n", to.x, to.y).as_bytes()
        );
    }

    /// Set line width
    pub fn set_line_width(&mut self, width: f32) {
        self.content.extend_from_slice(format!("{:.3} w\n", width).as_bytes());
    }

    /// Draw an image
    pub fn draw_image(&mut self, image_name: &str, x: f32, y: f32, width: f32, height: f32) {
        // Save graphics state
        self.content.extend_from_slice(b"q\n");
        // Concatenate matrix for positioning and scaling
        self.content.extend_from_slice(
            format!("{:.3} 0 0 {:.3} {:.3} {:.3} cm\n", width, height, x, y).as_bytes()
        );
        // Draw image
        self.content.extend_from_slice(format!("/{} Do\n", image_name).as_bytes());
        // Restore graphics state
        self.content.extend_from_slice(b"Q\n");
    }

    /// Save graphics state
    pub fn save_state(&mut self) {
        self.content.extend_from_slice(b"q\n");
    }

    /// Restore graphics state
    pub fn restore_state(&mut self) {
        self.content.extend_from_slice(b"Q\n");
    }

    /// Get the content as bytes
    pub fn into_bytes(self) -> Vec<u8> {
        self.content
    }

    /// Get content length
    pub fn len(&self) -> usize {
        self.content.len()
    }

    pub fn is_empty(&self) -> bool {
        self.content.is_empty()
    }
}

impl Default for PageContent {
    fn default() -> Self {
        Self::new()
    }
}

/// Text alignment options
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum TextAlign {
    #[default]
    Left,
    Center,
    Right,
    Justify,
}

/// Vertical alignment
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum VerticalAlign {
    #[default]
    Baseline,
    Top,
    Middle,
    Bottom,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_page_content() {
        let mut content = PageContent::new();
        content.begin_text();
        content.set_font("F1", 12.0);
        content.text_position(100.0, 700.0);
        content.show_text("Hello World");
        content.end_text();

        let bytes = content.into_bytes();
        let output = String::from_utf8(bytes).unwrap();
        assert!(output.contains("BT"));
        assert!(output.contains("ET"));
        assert!(output.contains("Hello World"));
    }

    #[test]
    fn test_text_escaping() {
        let mut content = PageContent::new();
        content.show_text("(test)");
        
        let bytes = content.into_bytes();
        let output = String::from_utf8(bytes).unwrap();
        assert!(output.contains("\\(test\\)"));
    }
}
