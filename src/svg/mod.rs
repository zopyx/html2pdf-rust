//! SVG rendering support for HTML2PDF
//!
//! This module provides SVG parsing and conversion to PDF vector graphics.
//! It supports a comprehensive subset of SVG features including:
//!
//! - Basic shapes: rect, circle, ellipse, line, polyline, polygon
//! - Paths with full path command support
//! - Text rendering
//! - Gradients (linear and radial)
//! - Coordinate transforms
//! - Styling attributes
//!
//! The implementation converts SVG directly to PDF graphics operations
//! for optimal quality and file size.

use crate::pdf::PageContent;
use crate::types::{Color, Point, Rect, Result, PdfError};
use std::collections::HashMap;
use std::str::FromStr;

mod parser;
mod path;
mod render;
mod transform;
mod style;

pub use parser::{SvgDocument, SvgElement, SvgNode};
pub use path::{PathCommand, PathParser};
pub use render::SvgRenderer;
pub use transform::Transform;
pub use style::{SvgStyle, FillRule, LineCap, LineJoin};

/// SVG namespace URI
const SVG_NAMESPACE: &str = "http://www.w3.org/2000/svg";

/// SVG length unit
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SvgUnit {
    Px,
    Pt,
    In,
    Cm,
    Mm,
    Em,
    Ex,
    Percent,
}

/// SVG length value with unit
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SvgLength {
    pub value: f32,
    pub unit: SvgUnit,
}

impl SvgLength {
    /// Convert to points (PDF uses points as default unit)
    pub fn to_points(&self, viewport_width: f32, viewport_height: f32, font_size: f32) -> f32 {
        match self.unit {
            SvgUnit::Px => self.value * 0.75, // 96 DPI
            SvgUnit::Pt => self.value,
            SvgUnit::In => self.value * 72.0,
            SvgUnit::Cm => self.value * 28.346_46,
            SvgUnit::Mm => self.value * 2.834_646,
            SvgUnit::Em => self.value * font_size,
            SvgUnit::Ex => self.value * font_size * 0.5, // Approximate
            SvgUnit::Percent => self.value * viewport_width / 100.0,
        }
    }

    /// Convert to points using specific reference dimension for percentages
    pub fn to_points_with_reference(&self, reference: f32, font_size: f32) -> f32 {
        match self.unit {
            SvgUnit::Percent => self.value * reference / 100.0,
            _ => self.to_points(reference, reference, font_size),
        }
    }
}

impl Default for SvgLength {
    fn default() -> Self {
        Self {
            value: 0.0,
            unit: SvgUnit::Px,
        }
    }
}

/// Parse an SVG length value from string
pub fn parse_length(s: &str) -> Option<SvgLength> {
    let s = s.trim();
    if s.is_empty() {
        return None;
    }

    // Find where the number ends
    let num_end = s
        .find(|c: char| !c.is_ascii_digit() && c != '.' && c != '-' && c != '+')
        .unwrap_or(s.len());

    let value = s[..num_end].parse::<f32>().ok()?;
    let unit_str = &s[num_end..];

    let unit = match unit_str.trim() {
        "px" | "" => SvgUnit::Px,
        "pt" => SvgUnit::Pt,
        "in" => SvgUnit::In,
        "cm" => SvgUnit::Cm,
        "mm" => SvgUnit::Mm,
        "em" => SvgUnit::Em,
        "ex" => SvgUnit::Ex,
        "%" => SvgUnit::Percent,
        _ => SvgUnit::Px,
    };

    Some(SvgLength { value, unit })
}

/// SVG viewBox definition
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ViewBox {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl ViewBox {
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    /// Parse viewBox from string "x y width height"
    pub fn parse(s: &str) -> Option<Self> {
        let parts: Vec<&str> = s.split_whitespace().collect();
        if parts.len() != 4 {
            return None;
        }
        let x = parts[0].parse().ok()?;
        let y = parts[1].parse().ok()?;
        let width = parts[2].parse().ok()?;
        let height = parts[3].parse().ok()?;
        Some(Self::new(x, y, width, height))
    }

    /// Get the aspect ratio
    pub fn aspect_ratio(&self) -> f32 {
        if self.height == 0.0 {
            1.0
        } else {
            self.width / self.height
        }
    }
}

impl Default for ViewBox {
    fn default() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            width: 100.0,
            height: 100.0,
        }
    }
}

/// SVG gradient stop
#[derive(Debug, Clone, PartialEq)]
pub struct GradientStop {
    pub offset: f32, // 0.0 to 1.0
    pub color: Color,
    pub opacity: f32,
}

/// Gradient definition
#[derive(Debug, Clone, PartialEq)]
pub enum Gradient {
    Linear {
        x1: f32,
        y1: f32,
        x2: f32,
        y2: f32,
        stops: Vec<GradientStop>,
        units: GradientUnits,
        spread_method: SpreadMethod,
    },
    Radial {
        cx: f32,
        cy: f32,
        r: f32,
        fx: f32,
        fy: f32,
        stops: Vec<GradientStop>,
        units: GradientUnits,
        spread_method: SpreadMethod,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum GradientUnits {
    #[default]
    ObjectBoundingBox,
    UserSpaceOnUse,
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum SpreadMethod {
    #[default]
    Pad,
    Reflect,
    Repeat,
}

/// SVG pattern definition
#[derive(Debug, Clone, PartialEq)]
pub struct Pattern {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub units: GradientUnits,
    pub content: Vec<SvgElement>,
}

/// Parse a color from SVG color value
pub fn parse_color(s: &str) -> Option<Color> {
    let s = s.trim();
    
    // Named colors
    match s.to_ascii_lowercase().as_str() {
        "none" | "transparent" => return Some(Color::TRANSPARENT),
        "black" => return Some(Color::BLACK),
        "white" => return Some(Color::WHITE),
        "red" => return Some(Color::new(255, 0, 0)),
        "green" => return Some(Color::new(0, 128, 0)),
        "blue" => return Some(Color::new(0, 0, 255)),
        "yellow" => return Some(Color::new(255, 255, 0)),
        "cyan" => return Some(Color::new(0, 255, 255)),
        "magenta" => return Some(Color::new(255, 0, 255)),
        "orange" => return Some(Color::new(255, 165, 0)),
        "purple" => return Some(Color::new(128, 0, 128)),
        "pink" => return Some(Color::new(255, 192, 203)),
        "gray" | "grey" => return Some(Color::new(128, 128, 128)),
        "lightgray" | "lightgrey" => return Some(Color::new(211, 211, 211)),
        "darkgray" | "darkgrey" => return Some(Color::new(169, 169, 169)),
        _ => {}
    }
    
    // Hex colors
    if let Some(color) = Color::from_hex(s) {
        return Some(color);
    }
    
    // rgb() function
    if s.starts_with("rgb(") && s.ends_with(')') {
        let inner = &s[4..s.len()-1];
        let parts: Vec<&str> = inner.split(',').collect();
        if parts.len() == 3 {
            let r = parts[0].trim().parse::<u8>().ok()?;
            let g = parts[1].trim().parse::<u8>().ok()?;
            let b = parts[2].trim().parse::<u8>().ok()?;
            return Some(Color::new(r, g, b));
        }
    }
    
    // rgba() function
    if s.starts_with("rgba(") && s.ends_with(')') {
        let inner = &s[5..s.len()-1];
        let parts: Vec<&str> = inner.split(',').collect();
        if parts.len() == 4 {
            let r = parts[0].trim().parse::<u8>().ok()?;
            let g = parts[1].trim().parse::<u8>().ok()?;
            let b = parts[2].trim().parse::<u8>().ok()?;
            let a = (parts[3].trim().parse::<f32>().ok()? * 255.0) as u8;
            return Some(Color::new_rgba(r, g, b, a));
        }
    }
    
    None
}

/// Parse opacity value (number or percentage)
pub fn parse_opacity(s: &str) -> Option<f32> {
    let s = s.trim();
    if s.ends_with('%') {
        s[..s.len()-1].parse::<f32>().ok().map(|v| v / 100.0)
    } else {
        s.parse::<f32>().ok()
    }
}

/// Convert an SVG document to PDF and render to PageContent
/// 
/// This is the main entry point for SVG to PDF conversion
pub fn render_svg_to_pdf(
    svg_data: &[u8],
    content: &mut PageContent,
    x: f32,
    y: f32,
    target_width: f32,
    target_height: f32,
) -> Result<()> {
    let svg_str = std::str::from_utf8(svg_data)
        .map_err(|_| PdfError::Image("Invalid SVG: not valid UTF-8".to_string()))?;
    
    // Parse the SVG document
    let doc = parser::parse_svg(svg_str)?;
    
    // Create renderer and render
    let mut renderer = SvgRenderer::new(&doc);
    renderer.render(content, x, y, target_width, target_height)?;
    
    Ok(())
}

/// Get SVG dimensions without full rendering
pub fn get_svg_dimensions(svg_data: &[u8]) -> Result<(f32, f32)> {
    let svg_str = std::str::from_utf8(svg_data)
        .map_err(|_| PdfError::Image("Invalid SVG: not valid UTF-8".to_string()))?;
    
    let doc = parser::parse_svg(svg_str)?;
    
    Ok((doc.width, doc.height))
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod unit_tests {
    use super::*;

    #[test]
    fn test_parse_length() {
        assert_eq!(parse_length("100").unwrap().value, 100.0);
        assert_eq!(parse_length("100px").unwrap().unit, SvgUnit::Px);
        assert_eq!(parse_length("72pt").unwrap().unit, SvgUnit::Pt);
        assert_eq!(parse_length("1in").unwrap().unit, SvgUnit::In);
        assert_eq!(parse_length("2.54cm").unwrap().unit, SvgUnit::Cm);
        assert_eq!(parse_length("10mm").unwrap().unit, SvgUnit::Mm);
        assert_eq!(parse_length("50%").unwrap().unit, SvgUnit::Percent);
    }

    #[test]
    fn test_parse_color() {
        assert_eq!(parse_color("#FF0000"), Some(Color::new(255, 0, 0)));
        assert_eq!(parse_color("red"), Some(Color::new(255, 0, 0)));
        assert_eq!(parse_color("rgb(0, 128, 0)"), Some(Color::new(0, 128, 0)));
        assert_eq!(parse_color("none"), Some(Color::TRANSPARENT));
    }

    #[test]
    fn test_parse_viewbox() {
        let vb = ViewBox::parse("0 0 100 200").unwrap();
        assert_eq!(vb.x, 0.0);
        assert_eq!(vb.y, 0.0);
        assert_eq!(vb.width, 100.0);
        assert_eq!(vb.height, 200.0);
    }

    #[test]
    fn test_length_to_points() {
        let px = SvgLength { value: 96.0, unit: SvgUnit::Px };
        assert!((px.to_points(100.0, 100.0, 12.0) - 72.0).abs() < 0.1);

        let pt = SvgLength { value: 72.0, unit: SvgUnit::Pt };
        assert_eq!(pt.to_points(100.0, 100.0, 12.0), 72.0);

        let percent = SvgLength { value: 50.0, unit: SvgUnit::Percent };
        assert_eq!(percent.to_points(200.0, 100.0, 12.0), 100.0);
    }
}
