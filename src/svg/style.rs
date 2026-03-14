//! SVG styling support
//!
//! Handles SVG presentation attributes and CSS styling including:
//! - Fill properties (color, opacity, rule)
//! - Stroke properties (color, width, opacity, dash, cap, join)
//! - Opacity
//! - Display

use crate::types::Color;
use super::parse_color;

/// SVG fill rule for determining interior of complex paths
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum FillRule {
    #[default]
    NonZero,
    EvenOdd,
}

impl FillRule {
    /// Parse from string value
    pub fn parse(s: &str) -> Self {
        match s.trim() {
            "evenodd" => FillRule::EvenOdd,
            _ => FillRule::NonZero,
        }
    }

    /// Get PDF fill rule operator
    pub fn pdf_operator(&self) -> &'static str {
        match self {
            FillRule::NonZero => "f",
            FillRule::EvenOdd => "f*",
        }
    }
}

/// Line cap style for strokes
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum LineCap {
    #[default]
    Butt,
    Round,
    Square,
}

impl LineCap {
    /// Parse from string value
    pub fn parse(s: &str) -> Self {
        match s.trim() {
            "round" => LineCap::Round,
            "square" => LineCap::Square,
            _ => LineCap::Butt,
        }
    }

    /// Get PDF line cap integer value
    pub fn to_pdf_int(&self) -> i32 {
        match self {
            LineCap::Butt => 0,
            LineCap::Round => 1,
            LineCap::Square => 2,
        }
    }
}

/// Line join style for strokes
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum LineJoin {
    #[default]
    Miter,
    Round,
    Bevel,
}

impl LineJoin {
    /// Parse from string value
    pub fn parse(s: &str) -> Self {
        match s.trim() {
            "round" => LineJoin::Round,
            "bevel" => LineJoin::Bevel,
            _ => LineJoin::Miter,
        }
    }

    /// Get PDF line join integer value
    pub fn to_pdf_int(&self) -> i32 {
        match self {
            LineJoin::Miter => 0,
            LineJoin::Round => 1,
            LineJoin::Bevel => 2,
        }
    }
}

/// SVG style properties
#[derive(Debug, Clone, PartialEq)]
pub struct SvgStyle {
    // Fill properties
    pub fill: Option<Fill>,
    pub fill_opacity: f32,
    pub fill_rule: FillRule,

    // Stroke properties
    pub stroke: Option<Stroke>,
    pub stroke_width: f32,
    pub stroke_opacity: f32,
    pub stroke_linecap: LineCap,
    pub stroke_linejoin: LineJoin,
    pub stroke_miterlimit: f32,
    pub stroke_dasharray: Option<Vec<f32>>,
    pub stroke_dashoffset: f32,

    // General properties
    pub opacity: f32,
    pub display: Display,
    pub visibility: Visibility,

    // Font properties (for text)
    pub font_family: Option<String>,
    pub font_size: f32,
    pub font_weight: FontWeight,
    pub font_style: FontStyle,
    pub text_anchor: TextAnchor,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Fill {
    Color(Color),
    Url(String), // Reference to gradient or pattern
    None,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Stroke {
    Color(Color),
    Url(String),
    None,
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum Display {
    #[default]
    Inline,
    None,
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum Visibility {
    #[default]
    Visible,
    Hidden,
    Collapse,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FontWeight {
    Normal,
    Bold,
    Bolder,
    Lighter,
    Weight(u16),
}

impl Default for FontWeight {
    fn default() -> Self {
        FontWeight::Normal
    }
}

impl FontWeight {
    pub fn parse(s: &str) -> Self {
        match s.trim() {
            "bold" => FontWeight::Bold,
            "bolder" => FontWeight::Bolder,
            "lighter" => FontWeight::Lighter,
            "normal" => FontWeight::Normal,
            n => n.parse::<u16>().map(FontWeight::Weight).unwrap_or(FontWeight::Normal),
        }
    }

    pub fn is_bold(&self) -> bool {
        match self {
            FontWeight::Bold | FontWeight::Bolder => true,
            FontWeight::Weight(w) => *w >= 700,
            _ => false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum FontStyle {
    #[default]
    Normal,
    Italic,
    Oblique,
}

impl FontStyle {
    pub fn parse(s: &str) -> Self {
        match s.trim() {
            "italic" => FontStyle::Italic,
            "oblique" => FontStyle::Oblique,
            _ => FontStyle::Normal,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum TextAnchor {
    #[default]
    Start,
    Middle,
    End,
}

impl TextAnchor {
    pub fn parse(s: &str) -> Self {
        match s.trim() {
            "middle" => TextAnchor::Middle,
            "end" => TextAnchor::End,
            _ => TextAnchor::Start,
        }
    }
}

impl SvgStyle {
    /// Create a new style with default values
    pub fn new() -> Self {
        Self {
            fill: Some(Fill::Color(Color::BLACK)),
            fill_opacity: 1.0,
            fill_rule: FillRule::default(),
            stroke: None,
            stroke_width: 1.0,
            stroke_opacity: 1.0,
            stroke_linecap: LineCap::default(),
            stroke_linejoin: LineJoin::default(),
            stroke_miterlimit: 4.0,
            stroke_dasharray: None,
            stroke_dashoffset: 0.0,
            opacity: 1.0,
            display: Display::default(),
            visibility: Visibility::default(),
            font_family: None,
            font_size: 12.0,
            font_weight: FontWeight::default(),
            font_style: FontStyle::default(),
            text_anchor: TextAnchor::default(),
        }
    }

    /// Check if element should be rendered
    pub fn is_visible(&self) -> bool {
        matches!(self.display, Display::Inline)
            && matches!(self.visibility, Visibility::Visible)
            && self.opacity > 0.0
    }

    /// Check if element has fill
    pub fn has_fill(&self) -> bool {
        matches!(self.fill, Some(Fill::Color(_)))
            && self.fill_opacity > 0.0
            && self.opacity > 0.0
    }

    /// Check if element has stroke
    pub fn has_stroke(&self) -> bool {
        matches!(self.stroke, Some(Stroke::Color(_)))
            && self.stroke_width > 0.0
            && self.stroke_opacity > 0.0
            && self.opacity > 0.0
    }

    /// Get effective fill color with opacity applied
    pub fn effective_fill_color(&self) -> Option<Color> {
        match &self.fill {
            Some(Fill::Color(c)) => {
                let alpha = ((c.a as f32 / 255.0) * self.fill_opacity * self.opacity * 255.0) as u8;
                Some(Color::new_rgba(c.r, c.g, c.b, alpha))
            }
            _ => None,
        }
    }

    /// Get effective stroke color with opacity applied
    pub fn effective_stroke_color(&self) -> Option<Color> {
        match &self.stroke {
            Some(Stroke::Color(c)) => {
                let alpha = ((c.a as f32 / 255.0) * self.stroke_opacity * self.opacity * 255.0) as u8;
                Some(Color::new_rgba(c.r, c.g, c.b, alpha))
            }
            _ => None,
        }
    }

    /// Parse style from SVG element attributes
    pub fn from_element_attributes(attrs: &std::collections::HashMap<String, String>) -> Self {
        let mut style = Self::new();

        // Parse fill
        if let Some(fill_str) = attrs.get("fill") {
            style.fill = parse_fill(fill_str);
        }

        // Parse fill-opacity
        if let Some(opacity_str) = attrs.get("fill-opacity") {
            style.fill_opacity = opacity_str.parse().unwrap_or(1.0);
        }

        // Parse fill-rule
        if let Some(rule_str) = attrs.get("fill-rule") {
            style.fill_rule = FillRule::parse(rule_str);
        }

        // Parse stroke
        if let Some(stroke_str) = attrs.get("stroke") {
            style.stroke = parse_stroke(stroke_str);
        }

        // Parse stroke-width
        if let Some(width_str) = attrs.get("stroke-width") {
            style.stroke_width = parse_length_value(width_str).unwrap_or(1.0);
        }

        // Parse stroke-opacity
        if let Some(opacity_str) = attrs.get("stroke-opacity") {
            style.stroke_opacity = opacity_str.parse().unwrap_or(1.0);
        }

        // Parse stroke-linecap
        if let Some(cap_str) = attrs.get("stroke-linecap") {
            style.stroke_linecap = LineCap::parse(cap_str);
        }

        // Parse stroke-linejoin
        if let Some(join_str) = attrs.get("stroke-linejoin") {
            style.stroke_linejoin = LineJoin::parse(join_str);
        }

        // Parse stroke-miterlimit
        if let Some(limit_str) = attrs.get("stroke-miterlimit") {
            style.stroke_miterlimit = limit_str.parse().unwrap_or(4.0);
        }

        // Parse stroke-dasharray
        if let Some(dash_str) = attrs.get("stroke-dasharray") {
            style.stroke_dasharray = parse_dasharray(dash_str);
        }

        // Parse stroke-dashoffset
        if let Some(offset_str) = attrs.get("stroke-dashoffset") {
            style.stroke_dashoffset = parse_length_value(offset_str).unwrap_or(0.0);
        }

        // Parse opacity
        if let Some(opacity_str) = attrs.get("opacity") {
            style.opacity = opacity_str.parse().unwrap_or(1.0);
        }

        // Parse display
        if let Some(display_str) = attrs.get("display") {
            style.display = if display_str.trim() == "none" {
                Display::None
            } else {
                Display::Inline
            };
        }

        // Parse visibility
        if let Some(vis_str) = attrs.get("visibility") {
            style.visibility = match vis_str.trim() {
                "hidden" => Visibility::Hidden,
                "collapse" => Visibility::Collapse,
                _ => Visibility::Visible,
            };
        }

        // Parse font properties
        if let Some(family_str) = attrs.get("font-family") {
            style.font_family = Some(family_str.clone());
        }

        if let Some(size_str) = attrs.get("font-size") {
            style.font_size = parse_length_value(size_str).unwrap_or(12.0);
        }

        if let Some(weight_str) = attrs.get("font-weight") {
            style.font_weight = FontWeight::parse(weight_str);
        }

        if let Some(style_str) = attrs.get("font-style") {
            style.font_style = FontStyle::parse(style_str);
        }

        if let Some(anchor_str) = attrs.get("text-anchor") {
            style.text_anchor = TextAnchor::parse(anchor_str);
        }

        style
    }

    /// Parse inline style string (e.g., "fill: red; stroke: blue")
    pub fn from_inline_style(style_str: &str) -> Self {
        let mut style = Self::new();

        for declaration in style_str.split(';') {
            let declaration = declaration.trim();
            if let Some(pos) = declaration.find(':') {
                let property = declaration[..pos].trim();
                let value = declaration[pos + 1..].trim();

                match property {
                    "fill" => style.fill = parse_fill(value),
                    "fill-opacity" => style.fill_opacity = value.parse().unwrap_or(1.0),
                    "fill-rule" => style.fill_rule = FillRule::parse(value),
                    "stroke" => style.stroke = parse_stroke(value),
                    "stroke-width" => style.stroke_width = parse_length_value(value).unwrap_or(1.0),
                    "stroke-opacity" => style.stroke_opacity = value.parse().unwrap_or(1.0),
                    "stroke-linecap" => style.stroke_linecap = LineCap::parse(value),
                    "stroke-linejoin" => style.stroke_linejoin = LineJoin::parse(value),
                    "stroke-miterlimit" => style.stroke_miterlimit = value.parse().unwrap_or(4.0),
                    "stroke-dasharray" => style.stroke_dasharray = parse_dasharray(value),
                    "stroke-dashoffset" => style.stroke_dashoffset = parse_length_value(value).unwrap_or(0.0),
                    "opacity" => style.opacity = value.parse().unwrap_or(1.0),
                    "display" => style.display = if value == "none" { Display::None } else { Display::Inline },
                    "visibility" => style.visibility = match value {
                        "hidden" => Visibility::Hidden,
                        "collapse" => Visibility::Collapse,
                        _ => Visibility::Visible,
                    },
                    "font-family" => style.font_family = Some(value.to_string()),
                    "font-size" => style.font_size = parse_length_value(value).unwrap_or(12.0),
                    "font-weight" => style.font_weight = FontWeight::parse(value),
                    "font-style" => style.font_style = FontStyle::parse(value),
                    "text-anchor" => style.text_anchor = TextAnchor::parse(value),
                    _ => {}
                }
            }
        }

        style
    }

    /// Merge another style into this one (other takes precedence for non-default values)
    pub fn merge(&mut self, other: &SvgStyle) {
        if other.fill.is_some() {
            self.fill = other.fill.clone();
        }
        if other.fill_opacity != 1.0 {
            self.fill_opacity = other.fill_opacity;
        }
        self.fill_rule = other.fill_rule;

        if other.stroke.is_some() {
            self.stroke = other.stroke.clone();
        }
        if other.stroke_width != 1.0 {
            self.stroke_width = other.stroke_width;
        }
        if other.stroke_opacity != 1.0 {
            self.stroke_opacity = other.stroke_opacity;
        }
        if other.stroke_linecap != LineCap::default() {
            self.stroke_linecap = other.stroke_linecap;
        }
        if other.stroke_linejoin != LineJoin::default() {
            self.stroke_linejoin = other.stroke_linejoin;
        }
        if other.stroke_miterlimit != 4.0 {
            self.stroke_miterlimit = other.stroke_miterlimit;
        }
        if other.stroke_dasharray.is_some() {
            self.stroke_dasharray = other.stroke_dasharray.clone();
        }
        if other.stroke_dashoffset != 0.0 {
            self.stroke_dashoffset = other.stroke_dashoffset;
        }

        if other.opacity != 1.0 {
            self.opacity = other.opacity;
        }
        if !matches!(other.display, Display::Inline) {
            self.display = other.display;
        }
        if !matches!(other.visibility, Visibility::Visible) {
            self.visibility = other.visibility;
        }

        if other.font_family.is_some() {
            self.font_family = other.font_family.clone();
        }
        if other.font_size != 12.0 {
            self.font_size = other.font_size;
        }
        if !matches!(other.font_weight, FontWeight::Normal) {
            self.font_weight = other.font_weight;
        }
        if !matches!(other.font_style, FontStyle::Normal) {
            self.font_style = other.font_style;
        }
        if !matches!(other.text_anchor, TextAnchor::Start) {
            self.text_anchor = other.text_anchor;
        }
    }
}

impl Default for SvgStyle {
    fn default() -> Self {
        Self::new()
    }
}

/// Parse fill value
fn parse_fill(s: &str) -> Option<Fill> {
    let s = s.trim();
    if s == "none" {
        Some(Fill::None)
    } else if s.starts_with("url(") && s.ends_with(')') {
        let url = &s[4..s.len()-1];
        let url = url.trim_matches('#');
        Some(Fill::Url(url.to_string()))
    } else {
        parse_color(s).map(Fill::Color)
    }
}

/// Parse stroke value
fn parse_stroke(s: &str) -> Option<Stroke> {
    let s = s.trim();
    if s == "none" {
        Some(Stroke::None)
    } else if s.starts_with("url(") && s.ends_with(')') {
        let url = &s[4..s.len()-1];
        let url = url.trim_matches('#');
        Some(Stroke::Url(url.to_string()))
    } else {
        parse_color(s).map(Stroke::Color)
    }
}

/// Parse a length value (simplified - just extracts the number)
fn parse_length_value(s: &str) -> Option<f32> {
    let s = s.trim();
    
    // Remove units
    let num_str: String = s
        .chars()
        .take_while(|c| c.is_ascii_digit() || *c == '.' || *c == '-')
        .collect();
    
    num_str.parse().ok()
}

/// Parse stroke-dasharray value
fn parse_dasharray(s: &str) -> Option<Vec<f32>> {
    let s = s.trim();
    if s == "none" {
        return None;
    }

    let mut dashes = Vec::new();
    for part in s.split(',') {
        let part = part.trim();
        if let Some(val) = parse_length_value(part) {
            dashes.push(val);
        }
    }

    if dashes.is_empty() {
        None
    } else {
        Some(dashes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_style() {
        let style = SvgStyle::new();
        assert!(matches!(style.fill, Some(Fill::Color(_))));
        assert_eq!(style.fill_opacity, 1.0);
        assert_eq!(style.stroke_width, 1.0);
    }

    #[test]
    fn test_fill_rule() {
        assert_eq!(FillRule::parse("nonzero"), FillRule::NonZero);
        assert_eq!(FillRule::parse("evenodd"), FillRule::EvenOdd);
        assert_eq!(FillRule::parse("evenOdd"), FillRule::EvenOdd);
    }

    #[test]
    fn test_line_cap() {
        assert_eq!(LineCap::parse("butt"), LineCap::Butt);
        assert_eq!(LineCap::parse("round"), LineCap::Round);
        assert_eq!(LineCap::parse("square"), LineCap::Square);
    }

    #[test]
    fn test_line_join() {
        assert_eq!(LineJoin::parse("miter"), LineJoin::Miter);
        assert_eq!(LineJoin::parse("round"), LineJoin::Round);
        assert_eq!(LineJoin::parse("bevel"), LineJoin::Bevel);
    }

    #[test]
    fn test_parse_inline_style() {
        let style = SvgStyle::from_inline_style("fill: red; stroke: blue; stroke-width: 2");
        
        assert_eq!(style.fill, Some(Fill::Color(Color::new(255, 0, 0))));
        assert_eq!(style.stroke, Some(Stroke::Color(Color::new(0, 0, 255))));
        assert_eq!(style.stroke_width, 2.0);
    }

    #[test]
    fn test_parse_url_fill() {
        let style = SvgStyle::from_inline_style("fill: url(#gradient1)");
        assert_eq!(style.fill, Some(Fill::Url("gradient1".to_string())));
    }

    #[test]
    fn test_effective_colors() {
        let mut style = SvgStyle::new();
        style.fill = Some(Fill::Color(Color::new_rgba(255, 0, 0, 255)));
        style.fill_opacity = 0.5;
        style.opacity = 0.5;

        let effective = style.effective_fill_color().unwrap();
        assert!(effective.a < 255);
    }

    #[test]
    fn test_visibility() {
        let style = SvgStyle::from_inline_style("display: none");
        assert!(!style.is_visible());

        let style = SvgStyle::from_inline_style("visibility: hidden");
        assert!(!style.is_visible());

        let style = SvgStyle::from_inline_style("opacity: 0");
        assert!(!style.has_fill());
    }

    #[test]
    fn test_font_weight() {
        assert!(FontWeight::parse("bold").is_bold());
        assert!(FontWeight::parse("700").is_bold());
        assert!(!FontWeight::parse("normal").is_bold());
        assert!(!FontWeight::parse("400").is_bold());
    }

    #[test]
    fn test_parse_dasharray() {
        assert_eq!(parse_dasharray("5, 3, 2"), Some(vec![5.0, 3.0, 2.0]));
        assert_eq!(parse_dasharray("none"), None);
        assert_eq!(parse_dasharray(""), None);
    }
}
