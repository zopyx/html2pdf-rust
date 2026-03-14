//! SVG XML parser
//!
//! Parses SVG XML documents into an internal representation
//! that can be rendered to PDF.

use super::*;
use crate::types::{PdfError, Result};

/// An SVG document
#[derive(Debug, Clone, PartialEq)]
pub struct SvgDocument {
    pub width: f32,
    pub height: f32,
    pub view_box: ViewBox,
    pub root: SvgElement,
    /// Gradient and pattern definitions
    pub definitions: HashMap<String, Definition>,
}

impl SvgDocument {
    pub fn new() -> Self {
        Self {
            width: 300.0,
            height: 150.0,
            view_box: ViewBox::default(),
            root: SvgElement::new("svg"),
            definitions: HashMap::new(),
        }
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

impl Default for SvgDocument {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Definition {
    Gradient(Gradient),
    Pattern(Pattern),
}

/// An SVG element
#[derive(Debug, Clone, PartialEq)]
pub struct SvgElement {
    pub tag_name: String,
    pub attributes: HashMap<String, String>,
    pub children: Vec<SvgNode>,
    pub styles: HashMap<String, String>,
}

impl SvgElement {
    pub fn new(tag_name: impl Into<String>) -> Self {
        Self {
            tag_name: tag_name.into(),
            attributes: HashMap::new(),
            children: Vec::new(),
            styles: HashMap::new(),
        }
    }

    pub fn get_attr(&self, name: &str) -> Option<&str> {
        self.attributes.get(name).map(|s| s.as_str())
    }

    pub fn get_style(&self, name: &str) -> Option<&str> {
        self.styles.get(name).map(|s| s.as_str())
    }

    pub fn has_attr(&self, name: &str) -> bool {
        self.attributes.contains_key(name)
    }

    /// Get id attribute
    pub fn id(&self) -> Option<&str> {
        self.get_attr("id")
    }

    /// Get fill attribute (from attribute or style)
    pub fn fill(&self) -> Option<&str> {
        self.get_style("fill")
            .or_else(|| self.get_attr("fill"))
    }

    /// Get stroke attribute
    pub fn stroke(&self) -> Option<&str> {
        self.get_style("stroke")
            .or_else(|| self.get_attr("stroke"))
    }

    /// Get stroke-width
    pub fn stroke_width(&self) -> Option<f32> {
        self.get_style("stroke-width")
            .or_else(|| self.get_attr("stroke-width"))
            .and_then(|s| s.parse().ok())
    }

    /// Get opacity
    pub fn opacity(&self) -> Option<f32> {
        self.get_style("opacity")
            .or_else(|| self.get_attr("opacity"))
            .and_then(parse_opacity)
    }

    /// Get transform attribute
    pub fn transform(&self) -> Option<&str> {
        self.get_attr("transform")
    }
}

/// SVG node type (element or text)
#[derive(Debug, Clone, PartialEq)]
pub enum SvgNode {
    Element(SvgElement),
    Text(String),
}

/// Parse an SVG string into a document
pub fn parse_svg(svg_str: &str) -> Result<SvgDocument> {
    let mut parser = SvgXmlParser::new(svg_str);
    parser.parse()
}

/// Simple XML parser for SVG
struct SvgXmlParser<'a> {
    input: &'a str,
    pos: usize,
}

impl<'a> SvgXmlParser<'a> {
    fn new(input: &'a str) -> Self {
        Self { input, pos: 0 }
    }

    fn parse(&mut self) -> Result<SvgDocument> {
        self.skip_whitespace();
        
        // Skip XML declaration if present
        if self.peek_str("<?xml") {
            self.skip_until("?>");
            self.advance(2); // skip ?>
            self.skip_whitespace();
        }

        // Skip DOCTYPE if present
        if self.peek_str("<!DOCTYPE") {
            self.skip_until(">");
            self.advance(1); // skip >
            self.skip_whitespace();
        }

        // Parse root svg element
        let root = self.parse_element()?;
        
        if root.tag_name != "svg" {
            return Err(PdfError::Parse("Root element must be svg".to_string()));
        }

        // Build document from root
        let mut doc = SvgDocument::new();
        doc.root = root;

        // Extract dimensions
        Self::extract_dimensions(&mut doc)?;

        // Extract definitions
        Self::extract_definitions(&mut doc)?;

        Ok(doc)
    }

    fn extract_dimensions(doc: &mut SvgDocument) -> Result<()> {
        let root = &doc.root;

        // Parse width
        if let Some(width_attr) = root.get_attr("width") {
            if let Some(length) = parse_length(width_attr) {
                doc.width = length.to_points(100.0, 100.0, 12.0);
            }
        }

        // Parse height
        if let Some(height_attr) = root.get_attr("height") {
            if let Some(length) = parse_length(height_attr) {
                doc.height = length.to_points(100.0, 100.0, 12.0);
            }
        }

        // Parse viewBox
        if let Some(viewbox_attr) = root.get_attr("viewBox") {
            if let Some(view_box) = ViewBox::parse(viewbox_attr) {
                doc.view_box = view_box;
                // If no width/height, use viewBox dimensions
                if !root.has_attr("width") {
                    doc.width = view_box.width;
                }
                if !root.has_attr("height") {
                    doc.height = view_box.height;
                }
            }
        }

        // Apply reasonable limits
        doc.width = doc.width.max(1.0).min(2000.0);
        doc.height = doc.height.max(1.0).min(2000.0);

        Ok(())
    }

    fn extract_definitions(doc: &mut SvgDocument) -> Result<()> {
        // Find defs elements and extract gradients/patterns
        Self::extract_defs_from_element(&doc.root.clone(), &mut doc.definitions);
        Ok(())
    }

    fn extract_defs_from_element(element: &SvgElement, defs: &mut HashMap<String, Definition>) {
        if element.tag_name == "defs" {
            for child in &element.children {
                if let SvgNode::Element(el) = child {
                    if let Some(id) = el.id() {
                        if let Some(def) = Self::parse_definition(el) {
                            defs.insert(id.to_string(), def);
                        }
                    }
                }
            }
        }

        // Also check children recursively
        for child in &element.children {
            if let SvgNode::Element(el) = child {
                Self::extract_defs_from_element(el, defs);
            }
        }
    }

    fn parse_definition(el: &SvgElement) -> Option<Definition> {
        match el.tag_name.as_str() {
            "linearGradient" => {
                Self::parse_linear_gradient(el).map(Definition::Gradient)
            }
            "radialGradient" => {
                Self::parse_radial_gradient(el).map(Definition::Gradient)
            }
            "pattern" => {
                Self::parse_pattern(el).map(Definition::Pattern)
            }
            _ => None,
        }
    }

    fn parse_linear_gradient(el: &SvgElement) -> Option<Gradient> {
        let x1 = el.get_attr("x1").and_then(parse_length)
            .map(|l| l.value).unwrap_or(0.0);
        let y1 = el.get_attr("y1").and_then(parse_length)
            .map(|l| l.value).unwrap_or(0.0);
        let x2 = el.get_attr("x2").and_then(parse_length)
            .map(|l| l.value).unwrap_or(1.0);
        let y2 = el.get_attr("y2").and_then(parse_length)
            .map(|l| l.value).unwrap_or(0.0);

        let stops = Self::parse_gradient_stops(el);

        Some(Gradient::Linear {
            x1, y1, x2, y2,
            stops,
            units: GradientUnits::ObjectBoundingBox,
            spread_method: SpreadMethod::Pad,
        })
    }

    fn parse_radial_gradient(el: &SvgElement) -> Option<Gradient> {
        let cx = el.get_attr("cx").and_then(parse_length)
            .map(|l| l.value).unwrap_or(0.5);
        let cy = el.get_attr("cy").and_then(parse_length)
            .map(|l| l.value).unwrap_or(0.5);
        let r = el.get_attr("r").and_then(parse_length)
            .map(|l| l.value).unwrap_or(0.5);
        let fx = el.get_attr("fx").and_then(parse_length)
            .map(|l| l.value).unwrap_or(cx);
        let fy = el.get_attr("fy").and_then(parse_length)
            .map(|l| l.value).unwrap_or(cy);

        let stops = Self::parse_gradient_stops(el);

        Some(Gradient::Radial {
            cx, cy, r, fx, fy,
            stops,
            units: GradientUnits::ObjectBoundingBox,
            spread_method: SpreadMethod::Pad,
        })
    }

    fn parse_gradient_stops(el: &SvgElement) -> Vec<GradientStop> {
        let mut stops = Vec::new();

        for child in &el.children {
            if let SvgNode::Element(stop_el) = child {
                if stop_el.tag_name == "stop" {
                    let offset = stop_el.get_attr("offset")
                        .and_then(|s| {
                            if s.ends_with('%') {
                                s[..s.len()-1].parse::<f32>().ok().map(|v| v / 100.0)
                            } else {
                                s.parse::<f32>().ok()
                            }
                        })
                        .unwrap_or(0.0);

                    let color = stop_el.get_attr("stop-color")
                        .and_then(parse_color)
                        .unwrap_or(Color::BLACK);

                    let opacity = stop_el.get_attr("stop-opacity")
                        .and_then(parse_opacity)
                        .unwrap_or(1.0);

                    stops.push(GradientStop {
                        offset: offset.clamp(0.0, 1.0),
                        color,
                        opacity,
                    });
                }
            }
        }

        stops
    }

    fn parse_pattern(_el: &SvgElement) -> Option<Pattern> {
        // Simplified pattern parsing
        // Full implementation would parse pattern units, content, etc.
        None
    }

    fn parse_element(&mut self) -> Result<SvgElement> {
        self.expect('<')?;
        
        let tag_name = self.parse_identifier()?;
        let mut element = SvgElement::new(tag_name);

        // Parse attributes
        loop {
            self.skip_whitespace();
            
            if self.peek() == '>' || self.peek_str("/>") {
                break;
            }

            if self.is_eof() {
                return Err(PdfError::Parse("Unexpected end of input in element".to_string()));
            }

            let attr_name = self.parse_identifier()?;
            self.skip_whitespace();
            self.expect('=')?;
            self.skip_whitespace();
            let attr_value = self.parse_attribute_value()?;
            
            element.attributes.insert(attr_name, attr_value);
        }

        // Parse style attribute into individual styles
        if let Some(style_str) = element.get_attr("style") {
            element.styles = Self::parse_inline_styles(style_str);
        }

        // Check for self-closing tag
        if self.peek_str("/>") {
            self.advance(2);
            return Ok(element);
        }

        // Expect >
        self.expect('>')?;

        // Parse children
        loop {
            self.skip_whitespace();

            if self.peek_str("</") {
                break;
            }

            if self.is_eof() {
                return Err(PdfError::Parse("Unexpected end of input looking for closing tag".to_string()));
            }

            if self.peek() == '<' {
                let child = self.parse_element()?;
                element.children.push(SvgNode::Element(child));
            } else {
                let text = self.parse_text()?;
                if !text.trim().is_empty() {
                    element.children.push(SvgNode::Text(text));
                }
            }
        }

        // Parse closing tag
        self.expect_str("</")?;
        let closing_name = self.parse_identifier()?;
        if closing_name != element.tag_name {
            return Err(PdfError::Parse(
                format!("Mismatched tags: <{}> and </{}>", element.tag_name, closing_name)
            ));
        }
        self.skip_whitespace();
        self.expect('>')?;

        Ok(element)
    }

    fn parse_inline_styles(style_str: &str) -> HashMap<String, String> {
        let mut styles = HashMap::new();
        
        for declaration in style_str.split(';') {
            let declaration = declaration.trim();
            if let Some(pos) = declaration.find(':') {
                let property = declaration[..pos].trim().to_string();
                let value = declaration[pos+1..].trim().to_string();
                styles.insert(property, value);
            }
        }

        styles
    }

    fn parse_identifier(&mut self) -> Result<String> {
        let start = self.pos;
        
        while !self.is_eof() {
            let c = self.peek();
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == ':' {
                self.advance(1);
            } else {
                break;
            }
        }

        if self.pos == start {
            return Err(PdfError::Parse("Expected identifier".to_string()));
        }

        Ok(self.input[start..self.pos].to_string())
    }

    fn parse_attribute_value(&mut self) -> Result<String> {
        let quote = self.peek();
        if quote != '"' && quote != '\'' {
            return Err(PdfError::Parse("Expected quote".to_string()));
        }
        
        self.advance(1);
        let start = self.pos;
        
        while !self.is_eof() && self.peek() != quote {
            self.advance(1);
        }

        let value = self.input[start..self.pos].to_string();
        self.advance(1); // skip closing quote

        Ok(value)
    }

    fn parse_text(&mut self) -> Result<String> {
        let start = self.pos;
        
        while !self.is_eof() && self.peek() != '<' {
            self.advance(1);
        }

        Ok(self.input[start..self.pos].to_string())
    }

    fn peek(&self) -> char {
        self.input[self.pos..].chars().next().unwrap_or('\0')
    }

    fn peek_str(&self, s: &str) -> bool {
        self.input[self.pos..].starts_with(s)
    }

    fn advance(&mut self, n: usize) {
        self.pos = (self.pos + n).min(self.input.len());
    }

    fn expect(&mut self, expected: char) -> Result<()> {
        if self.peek() == expected {
            self.advance(1);
            Ok(())
        } else {
            Err(PdfError::Parse(
                format!("Expected '{}', found '{}'", expected, self.peek())
            ))
        }
    }

    fn expect_str(&mut self, expected: &str) -> Result<()> {
        if self.peek_str(expected) {
            self.advance(expected.len());
            Ok(())
        } else {
            Err(PdfError::Parse(
                format!("Expected '{}'", expected)
            ))
        }
    }

    fn skip_whitespace(&mut self) {
        while !self.is_eof() && self.peek().is_ascii_whitespace() {
            self.advance(1);
        }
    }

    fn skip_until(&mut self, pattern: &str) {
        while !self.is_eof() && !self.peek_str(pattern) {
            self.advance(1);
        }
    }

    fn is_eof(&self) -> bool {
        self.pos >= self.input.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_svg() {
        let svg = r#"<svg width="100" height="200"></svg>"#;
        let doc = parse_svg(svg).unwrap();
        
        assert_eq!(doc.width, 75.0); // 100px * 0.75 = 75pt
        assert_eq!(doc.height, 150.0); // 200px * 0.75 = 150pt
    }

    #[test]
    fn test_parse_with_viewbox() {
        let svg = r#"<svg viewBox="0 0 800 600"></svg>"#;
        let doc = parse_svg(svg).unwrap();
        
        assert_eq!(doc.view_box.width, 800.0);
        assert_eq!(doc.view_box.height, 600.0);
        assert_eq!(doc.width, 800.0);
        assert_eq!(doc.height, 600.0);
    }

    #[test]
    fn test_parse_element_with_attributes() {
        let svg = r#"<svg width="100" height="100">
            <rect x="10" y="20" width="30" height="40" fill="red"/>
        </svg>"#;
        let doc = parse_svg(svg).unwrap();
        
        let rect = &doc.root.children[0];
        if let SvgNode::Element(el) = rect {
            assert_eq!(el.tag_name, "rect");
            assert_eq!(el.get_attr("x"), Some("10"));
            assert_eq!(el.get_attr("fill"), Some("red"));
        } else {
            panic!("Expected element");
        }
    }

    #[test]
    fn test_parse_inline_styles() {
        let svg = r#"<svg width="100" height="100">
            <rect style="fill: blue; stroke: red; stroke-width: 2"/>
        </svg>"#;
        let doc = parse_svg(svg).unwrap();
        
        let rect = &doc.root.children[0];
        if let SvgNode::Element(el) = rect {
            assert_eq!(el.get_style("fill"), Some("blue"));
            assert_eq!(el.get_style("stroke"), Some("red"));
            assert_eq!(el.get_style("stroke-width"), Some("2"));
        } else {
            panic!("Expected element");
        }
    }

    #[test]
    fn test_parse_gradient() {
        let svg = r#"<svg width="100" height="100">
            <defs>
                <linearGradient id="grad1" x1="0%" y1="0%" x2="100%" y2="0%">
                    <stop offset="0%" stop-color="red"/>
                    <stop offset="100%" stop-color="blue"/>
                </linearGradient>
            </defs>
        </svg>"#;
        let doc = parse_svg(svg).unwrap();
        
        assert!(doc.definitions.contains_key("grad1"));
    }
}
