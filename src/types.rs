//! Core types for HTML2PDF

/// A 2D point
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

impl Point {
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

/// A 2D size
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Size {
    pub width: f32,
    pub height: f32,
}

impl Size {
    pub const fn new(width: f32, height: f32) -> Self {
        Self { width, height }
    }

    pub const fn zero() -> Self {
        Self {
            width: 0.0,
            height: 0.0,
        }
    }
}

/// A rectangle
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl Rect {
    pub const fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    pub const fn from_origin(size: Size) -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            width: size.width,
            height: size.height,
        }
    }

    pub fn contains(&self, point: Point) -> bool {
        point.x >= self.x
            && point.x <= self.x + self.width
            && point.y >= self.y
            && point.y <= self.y + self.height
    }

    pub fn right(&self) -> f32 {
        self.x + self.width
    }

    pub fn bottom(&self) -> f32 {
        self.y + self.height
    }
}

/// CSS length value
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum Length {
    Px(f32),
    Pt(f32),
    Mm(f32),
    Cm(f32),
    In(f32),
    Em(f32),
    Rem(f32),
    Percent(f32),
    #[default]
    Auto,
}

impl Length {
    /// Convert to points (1/72 inch)
    pub fn to_pt(&self, base_font_size: f32) -> f32 {
        match *self {
            Length::Px(v) => v * 0.75, // 96 DPI: 1px = 0.75pt
            Length::Pt(v) => v,
            Length::Mm(v) => v * 2.834_646, // 1mm = 2.834646pt
            Length::Cm(v) => v * 28.346_46,
            Length::In(v) => v * 72.0,
            Length::Em(v) => v * base_font_size,
            Length::Rem(v) => v * base_font_size,
            Length::Percent(_) => 0.0, // Requires context
            Length::Auto => 0.0,
        }
    }

    /// Convert to points with a container size for percentages
    pub fn to_pt_with_container(&self, base_font_size: f32, container_size: f32) -> f32 {
        match *self {
            Length::Percent(p) => container_size * p / 100.0,
            _ => self.to_pt(base_font_size),
        }
    }

    pub fn is_auto(&self) -> bool {
        matches!(self, Length::Auto)
    }
}

/// Color in RGBA
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 255 }
    }

    pub const fn new_rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    pub fn from_hex(hex: &str) -> Option<Self> {
        let hex = hex.trim_start_matches('#');
        match hex.len() {
            3 => {
                let r = u8::from_str_radix(&hex[0..1].repeat(2), 16).ok()?;
                let g = u8::from_str_radix(&hex[1..2].repeat(2), 16).ok()?;
                let b = u8::from_str_radix(&hex[2..3].repeat(2), 16).ok()?;
                Some(Self::new(r, g, b))
            }
            6 => {
                let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
                let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
                let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
                Some(Self::new(r, g, b))
            }
            8 => {
                let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
                let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
                let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
                let a = u8::from_str_radix(&hex[6..8], 16).ok()?;
                Some(Self::new_rgba(r, g, b, a))
            }
            _ => None,
        }
    }

    /// Convert to PDF color values (0.0 - 1.0)
    pub fn to_pdf(&self) -> (f32, f32, f32) {
        (
            self.r as f32 / 255.0,
            self.g as f32 / 255.0,
            self.b as f32 / 255.0,
        )
    }

    pub const BLACK: Self = Self::new(0, 0, 0);
    pub const WHITE: Self = Self::new(255, 255, 255);
    pub const RED: Self = Self::new(255, 0, 0);
    pub const GREEN: Self = Self::new(0, 128, 0);
    pub const BLUE: Self = Self::new(0, 0, 255);
    pub const TRANSPARENT: Self = Self::new_rgba(0, 0, 0, 0);
}

/// Paper size definitions
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum PaperSize {
    A0,
    A1,
    A2,
    A3,
    #[default]
    A4,
    A5,
    A6,
    Letter,
    Legal,
    Tabloid,
    Custom { width: f32, height: f32 },
}

impl PaperSize {
    /// Get size in points (width, height)
    pub fn size(&self) -> (f32, f32) {
        match *self {
            PaperSize::A0 => (2383.94, 3370.39),
            PaperSize::A1 => (1683.78, 2383.94),
            PaperSize::A2 => (1190.55, 1683.78),
            PaperSize::A3 => (841.89, 1190.55),
            PaperSize::A4 => (595.28, 841.89),
            PaperSize::A5 => (419.53, 595.28),
            PaperSize::A6 => (297.64, 419.53),
            PaperSize::Letter => (612.0, 792.0),
            PaperSize::Legal => (612.0, 1008.0),
            PaperSize::Tabloid => (792.0, 1224.0),
            PaperSize::Custom { width, height } => (width, height),
        }
    }
}

/// Page orientation
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum Orientation {
    #[default]
    Portrait,
    Landscape,
}

/// Page margins
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Margins {
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
    pub left: f32,
}

impl Margins {
    pub const fn new(top: f32, right: f32, bottom: f32, left: f32) -> Self {
        Self {
            top,
            right,
            bottom,
            left,
        }
    }

    pub const fn all(value: f32) -> Self {
        Self {
            top: value,
            right: value,
            bottom: value,
            left: value,
        }
    }

    pub const fn symmetric(vertical: f32, horizontal: f32) -> Self {
        Self {
            top: vertical,
            right: horizontal,
            bottom: vertical,
            left: horizontal,
        }
    }
}

/// PDF error type
#[derive(Debug, thiserror::Error)]
pub enum PdfError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Parse error: {0}")]
    Parse(String),

    #[error("Layout error: {0}")]
    Layout(String),

    #[error("Font error: {0}")]
    Font(String),

    #[error("Image error: {0}")]
    Image(String),

    #[error("Invalid color: {0}")]
    InvalidColor(String),
}

/// Result type alias
pub type Result<T> = std::result::Result<T, PdfError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_from_hex() {
        assert_eq!(Color::from_hex("#FF0000"), Some(Color::new(255, 0, 0)));
        assert_eq!(Color::from_hex("#00FF00"), Some(Color::new(0, 255, 0)));
        assert_eq!(Color::from_hex("#0000FF"), Some(Color::new(0, 0, 255)));
        assert_eq!(Color::from_hex("#F00"), Some(Color::new(255, 0, 0)));
    }

    #[test]
    fn test_length_conversion() {
        let mm = Length::Mm(10.0);
        assert!((mm.to_pt(12.0) - 28.346).abs() < 0.1);

        let px = Length::Px(96.0);
        assert!((px.to_pt(12.0) - 72.0).abs() < 0.1);
    }

    #[test]
    fn test_paper_sizes() {
        let (w, h) = PaperSize::A4.size();
        assert!((w - 595.28).abs() < 0.1);
        assert!((h - 841.89).abs() < 0.1);
    }
}
