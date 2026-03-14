//! PDF generation from scratch
//!
//! This module implements PDF generation without external dependencies,
//! supporting the full PDF 1.4 specification needed for document output.

mod object;
mod stream;
mod writer;
pub mod font;
mod image;
pub mod print_css;
#[cfg(test)]
mod print_css_tests;

pub use object::{PdfObject, PdfReference, PdfDictionary, PdfArray};
pub use stream::{PdfStream, FlateEncode};
pub use writer::PdfWriter;
pub use font::PdfFont;
pub use image::PdfImage;
pub use print_css::{
    PageContext, PageSize, PageMaster, MarginBoxContent, MarginContentPart,
    TextAlign as MarginTextAlign, VerticalAlign as MarginVerticalAlign,
    BreakType, BreakInside, RunningElement, StringSet, StringSetValue,
    PageCounter, Bookmark, get_margin_box_rect,
    parse_break_value, parse_break_inside_value, parse_orphans_widows_value,
};

use crate::types::{Color, Rect, Point};
use crate::layout::form::{FormBox, FormControlType};

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

    /// Fill and stroke (non-zero winding rule)
    pub fn fill_and_stroke(&mut self) {
        self.content.extend_from_slice(b"B\n");
    }

    /// Fill and stroke with even-odd rule
    pub fn fill_and_stroke_even_odd(&mut self) {
        self.content.extend_from_slice(b"B*\n");
    }

    /// Fill with even-odd rule
    pub fn fill_even_odd(&mut self) {
        self.content.extend_from_slice(b"f*\n");
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

    /// Append raw PDF content
    pub fn append_raw(&mut self, s: &str) {
        self.content.extend_from_slice(s.as_bytes());
    }

    /// Draw a form control
    pub fn draw_form_control(&mut self, form_box: &FormBox, x: f32, y: f32, width: f32, height: f32) {
        match form_box.control_type {
            FormControlType::Text | FormControlType::Password | FormControlType::Email |
            FormControlType::Number | FormControlType::Date | FormControlType::Time |
            FormControlType::File => {
                self.draw_text_input(form_box, x, y, width, height);
            }
            FormControlType::Textarea => {
                self.draw_textarea(form_box, x, y, width, height);
            }
            FormControlType::Select => {
                self.draw_select(form_box, x, y, width, height);
            }
            FormControlType::Checkbox => {
                self.draw_checkbox(form_box, x, y, width, height);
            }
            FormControlType::Radio => {
                self.draw_radio(form_box, x, y, width, height);
            }
            FormControlType::Button | FormControlType::Submit | FormControlType::Reset => {
                self.draw_button(form_box, x, y, width, height);
            }
            FormControlType::Label => {
                self.draw_label(form_box, x, y, width, height);
            }
            FormControlType::Hidden => {
                // Hidden inputs are not rendered
            }
        }
    }

    /// Draw a text input field
    fn draw_text_input(&mut self, form_box: &FormBox, x: f32, y: f32, width: f32, height: f32) {
        // Save state
        self.save_state();

        // Draw background
        let bg_color = if form_box.disabled {
            Color::new(240, 240, 240)
        } else {
            Color::WHITE
        };
        self.set_fill_color(bg_color);
        self.draw_rect(Rect::new(x, y, width, height));
        self.fill();

        // Draw border
        let border_color = if form_box.disabled {
            Color::new(200, 200, 200)
        } else {
            Color::new(150, 150, 150)
        };
        self.set_stroke_color(border_color);
        self.set_line_width(1.0);
        self.draw_rect(Rect::new(x, y, width, height));
        self.stroke();

        // Draw text content
        let text = form_box.display_text();
        if !text.is_empty() {
            let text_color = if form_box.disabled {
                Color::new(128, 128, 128)
            } else {
                Color::BLACK
            };
            self.set_fill_color(text_color);
            self.begin_text();
            self.text_position(x + 4.0, y + height / 2.0 + 4.0);
            self.show_text(&text);
            self.end_text();
        }

        // Restore state
        self.restore_state();
    }

    /// Draw a textarea
    fn draw_textarea(&mut self, form_box: &FormBox, x: f32, y: f32, width: f32, height: f32) {
        // Similar to text input but typically larger
        self.save_state();

        // Draw background
        let bg_color = if form_box.disabled {
            Color::new(240, 240, 240)
        } else {
            Color::WHITE
        };
        self.set_fill_color(bg_color);
        self.draw_rect(Rect::new(x, y, width, height));
        self.fill();

        // Draw border
        let border_color = if form_box.disabled {
            Color::new(200, 200, 200)
        } else {
            Color::new(150, 150, 150)
        };
        self.set_stroke_color(border_color);
        self.set_line_width(1.0);
        self.draw_rect(Rect::new(x, y, width, height));
        self.stroke();

        // Draw text content (simplified - just show first line)
        let text = form_box.display_text();
        if !text.is_empty() {
            let text_color = if form_box.disabled {
                Color::new(128, 128, 128)
            } else {
                Color::BLACK
            };
            self.set_fill_color(text_color);
            self.begin_text();
            // Show text (limited to visible area)
            let display_text: String = text.chars().take(100).collect();
            self.text_position(x + 4.0, y + height - 8.0);
            self.show_text(&display_text);
            self.end_text();
        }

        self.restore_state();
    }

    /// Draw a select dropdown
    fn draw_select(&mut self, form_box: &FormBox, x: f32, y: f32, width: f32, height: f32) {
        self.save_state();

        // Draw background
        let bg_color = if form_box.disabled {
            Color::new(240, 240, 240)
        } else {
            Color::WHITE
        };
        self.set_fill_color(bg_color);
        self.draw_rect(Rect::new(x, y, width, height));
        self.fill();

        // Draw border
        let border_color = if form_box.disabled {
            Color::new(200, 200, 200)
        } else {
            Color::new(150, 150, 150)
        };
        self.set_stroke_color(border_color);
        self.set_line_width(1.0);
        self.draw_rect(Rect::new(x, y, width, height));
        self.stroke();

        // Draw selected text
        let text = form_box.display_text();
        if !text.is_empty() {
            let text_color = if form_box.disabled {
                Color::new(128, 128, 128)
            } else {
                Color::BLACK
            };
            self.set_fill_color(text_color);
            self.begin_text();
            self.text_position(x + 4.0, y + height / 2.0 + 4.0);
            self.show_text(&text);
            self.end_text();
        }

        // Draw dropdown arrow
        let arrow_x = x + width - 12.0;
        let arrow_y = y + height / 2.0;
        self.set_stroke_color(Color::new(100, 100, 100));
        self.set_line_width(1.0);
        self.draw_line(Point::new(arrow_x, arrow_y - 2.0), Point::new(arrow_x + 4.0, arrow_y + 2.0));
        self.draw_line(Point::new(arrow_x + 4.0, arrow_y + 2.0), Point::new(arrow_x + 8.0, arrow_y - 2.0));
        self.stroke();

        self.restore_state();
    }

    /// Draw a checkbox
    fn draw_checkbox(&mut self, form_box: &FormBox, x: f32, y: f32, width: f32, height: f32) {
        self.save_state();

        // Draw box
        let size = width.min(height).min(16.0);
        let box_x = x;
        let box_y = y + (height - size) / 2.0;

        // Background
        self.set_fill_color(Color::WHITE);
        self.draw_rect(Rect::new(box_x, box_y, size, size));
        self.fill();

        // Border
        let border_color = if form_box.disabled {
            Color::new(200, 200, 200)
        } else {
            Color::new(100, 100, 100)
        };
        self.set_stroke_color(border_color);
        self.set_line_width(1.0);
        self.draw_rect(Rect::new(box_x, box_y, size, size));
        self.stroke();

        // Draw checkmark if checked
        if form_box.checked {
            let check_color = if form_box.disabled {
                Color::new(150, 150, 150)
            } else {
                Color::new(50, 50, 50)
            };
            self.set_stroke_color(check_color);
            self.set_line_width(2.0);
            // Draw simple checkmark
            let offset = size * 0.2;
            let check_x = box_x + offset;
            let check_y = box_y + size / 2.0;
            self.draw_line(Point::new(check_x, check_y), Point::new(check_x + size * 0.15, check_y - size * 0.15));
            self.draw_line(Point::new(check_x + size * 0.15, check_y - size * 0.15), Point::new(check_x + size * 0.4, check_y + size * 0.2));
            self.stroke();
        }

        self.restore_state();
    }

    /// Draw a radio button
    fn draw_radio(&mut self, form_box: &FormBox, x: f32, y: f32, width: f32, height: f32) {
        self.save_state();

        // Draw circle
        let size = width.min(height).min(16.0);
        let center_x = x + size / 2.0;
        let center_y = y + height / 2.0;
        let radius = size / 2.0;

        // Draw circle border (using PDF circle approximation with bezier curves)
        let border_color = if form_box.disabled {
            Color::new(200, 200, 200)
        } else {
            Color::new(100, 100, 100)
        };
        self.set_stroke_color(border_color);
        self.set_fill_color(Color::WHITE);
        self.set_line_width(1.0);
        
        // Approximate circle with 4 bezier curves
        let c = 0.55228475 * radius; // Control point offset for circle approximation
        self.content.extend_from_slice(format!("{:.3} {:.3} m\n", center_x + radius, center_y).as_bytes());
        self.content.extend_from_slice(format!("{:.3} {:.3} {:.3} {:.3} {:.3} {:.3} c\n", 
            center_x + radius, center_y + c, center_x + c, center_y + radius, center_x, center_y + radius).as_bytes());
        self.content.extend_from_slice(format!("{:.3} {:.3} {:.3} {:.3} {:.3} {:.3} c\n", 
            center_x - c, center_y + radius, center_x - radius, center_y + c, center_x - radius, center_y).as_bytes());
        self.content.extend_from_slice(format!("{:.3} {:.3} {:.3} {:.3} {:.3} {:.3} c\n", 
            center_x - radius, center_y - c, center_x - c, center_y - radius, center_x, center_y - radius).as_bytes());
        self.content.extend_from_slice(format!("{:.3} {:.3} {:.3} {:.3} {:.3} {:.3} c\n", 
            center_x + c, center_y - radius, center_x + radius, center_y - c, center_x + radius, center_y).as_bytes());
        self.fill_and_stroke();

        // Draw filled circle if selected
        if form_box.checked {
            let fill_color = if form_box.disabled {
                Color::new(150, 150, 150)
            } else {
                Color::new(50, 50, 50)
            };
            self.set_fill_color(fill_color);
            let inner_radius = radius * 0.4;
            let c = 0.55228475 * inner_radius;
            self.content.extend_from_slice(format!("{:.3} {:.3} m\n", center_x + inner_radius, center_y).as_bytes());
            self.content.extend_from_slice(format!("{:.3} {:.3} {:.3} {:.3} {:.3} {:.3} c\n", 
                center_x + inner_radius, center_y + c, center_x + c, center_y + inner_radius, center_x, center_y + inner_radius).as_bytes());
            self.content.extend_from_slice(format!("{:.3} {:.3} {:.3} {:.3} {:.3} {:.3} c\n", 
                center_x - c, center_y + inner_radius, center_x - inner_radius, center_y + c, center_x - inner_radius, center_y).as_bytes());
            self.content.extend_from_slice(format!("{:.3} {:.3} {:.3} {:.3} {:.3} {:.3} c\n", 
                center_x - inner_radius, center_y - c, center_x - c, center_y - inner_radius, center_x, center_y - inner_radius).as_bytes());
            self.content.extend_from_slice(format!("{:.3} {:.3} {:.3} {:.3} {:.3} {:.3} c\n", 
                center_x + c, center_y - inner_radius, center_x + inner_radius, center_y - c, center_x + inner_radius, center_y).as_bytes());
            self.fill();
        }

        self.restore_state();
    }

    /// Draw a button
    fn draw_button(&mut self, form_box: &FormBox, x: f32, y: f32, width: f32, height: f32) {
        self.save_state();

        // Button colors
        let bg_color = if form_box.disabled {
            Color::new(220, 220, 220)
        } else {
            Color::new(240, 240, 240)
        };
        let border_color = if form_box.disabled {
            Color::new(180, 180, 180)
        } else {
            Color::new(150, 150, 150)
        };

        // Draw button background
        self.set_fill_color(bg_color);
        self.draw_rect(Rect::new(x, y, width, height));
        self.fill();

        // Draw border
        self.set_stroke_color(border_color);
        self.set_line_width(1.0);
        self.draw_rect(Rect::new(x, y, width, height));
        self.stroke();

        // Draw button text centered
        let text = &form_box.value;
        if !text.is_empty() {
            let text_color = if form_box.disabled {
                Color::new(128, 128, 128)
            } else {
                Color::BLACK
            };
            self.set_fill_color(text_color);
            self.begin_text();
            // Simple centering approximation
            let text_width = text.len() as f32 * 6.0; // Approximate
            let text_x = x + (width - text_width) / 2.0;
            let text_y = y + (height / 2.0) + 4.0;
            self.text_position(text_x, text_y);
            self.show_text(text);
            self.end_text();
        }

        self.restore_state();
    }

    /// Draw a label
    fn draw_label(&mut self, form_box: &FormBox, x: f32, y: f32, _width: f32, height: f32) {
        let text = &form_box.value;
        if !text.is_empty() {
            self.save_state();
            self.set_fill_color(Color::BLACK);
            self.begin_text();
            let text_y = y + (height / 2.0) + 4.0;
            self.text_position(x, text_y);
            self.show_text(text);
            self.end_text();
            self.restore_state();
        }
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
