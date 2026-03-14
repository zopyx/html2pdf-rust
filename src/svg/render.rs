//! SVG to PDF rendering
//!
//! Converts parsed SVG documents to PDF content stream operations.
//! This is the main rendering engine that handles all SVG elements.

use crate::pdf::PageContent;
use crate::types::{Color, PdfError, Point, Rect, Result};
use crate::svg::*;
use std::collections::HashMap;

/// SVG renderer
pub struct SvgRenderer<'a> {
    document: &'a SvgDocument,
    state_stack: Vec<RenderState>,
    current_state: RenderState,
}

/// Current rendering state
#[derive(Debug, Clone)]
struct RenderState {
    transform: Transform,
    style: SvgStyle,
    viewport: Rect,
}

impl RenderState {
    fn new(viewport: Rect) -> Self {
        Self {
            transform: Transform::identity(),
            style: SvgStyle::new(),
            viewport,
        }
    }
}

impl<'a> SvgRenderer<'a> {
    /// Create a new SVG renderer for the given document
    pub fn new(document: &'a SvgDocument) -> Self {
        let viewport = Rect::new(0.0, 0.0, document.width, document.height);
        let initial_state = RenderState::new(viewport);
        
        Self {
            document,
            state_stack: Vec::new(),
            current_state: initial_state,
        }
    }

    /// Render the SVG to PDF content
    /// 
    /// * `content` - The PDF page content to write to
    /// * `x`, `y` - Position on the PDF page (bottom-left corner in PDF coords)
    /// * `width`, `height` - Target dimensions
    pub fn render(
        &mut self,
        content: &mut PageContent,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
    ) -> Result<()> {
        // Save graphics state
        content.save_state();

        // Calculate the scale factors to fit SVG into target dimensions
        let scale_x = width / self.document.width;
        let scale_y = height / self.document.height;

        // Apply transform to position and scale the SVG
        // PDF coordinate system has Y going up, SVG has Y going down
        // We need to flip Y and adjust position
        let transform = Transform::new(
            scale_x, 0.0,
            0.0, -scale_y,  // Flip Y
            x, y + height   // Position at bottom-left + height, then flip draws downward
        );

        content.append_raw(&format!("{} cm\n", transform.to_pdf_string()));

        // Update viewport for the scaled coordinate system
        self.current_state.viewport = Rect::new(0.0, 0.0, self.document.width, self.document.height);

        // Render all children of the root element
        for child in &self.document.root.children {
            self.render_node(content, child)?;
        }

        // Restore graphics state
        content.restore_state();

        Ok(())
    }

    /// Render an SVG node
    fn render_node(&mut self, content: &mut PageContent, node: &SvgNode) -> Result<()> {
        match node {
            SvgNode::Element(element) => self.render_element(content, element)?,
            SvgNode::Text(_) => {} // Text is handled within text elements
        }
        Ok(())
    }

    /// Render an SVG element
    fn render_element(&mut self, content: &mut PageContent, element: &SvgElement) -> Result<()> {
        // Skip defs, title, desc, metadata elements
        match element.tag_name.as_str() {
            "defs" | "title" | "desc" | "metadata" => return Ok(()),
            _ => {}
        }

        // Update style from element attributes
        let mut element_style = SvgStyle::from_element_attributes(&element.attributes);
        if let Some(style_attr) = element.get_attr("style") {
            let inline_style = SvgStyle::from_inline_style(style_attr);
            element_style.merge(&inline_style);
        }

        // Check visibility
        if !element_style.is_visible() {
            return Ok(());
        }

        // Push state
        self.push_state();
        self.current_state.style.merge(&element_style);

        // Apply transform if present
        if let Some(transform_attr) = element.transform() {
            if let Ok(transform) = parse_transform(transform_attr) {
                self.current_state.transform = self.current_state.transform.multiply(&transform);
            }
        }

        // Render based on element type
        match element.tag_name.as_str() {
            "g" => self.render_group(content, element)?,
            "rect" => self.render_rect(content, element)?,
            "circle" => self.render_circle(content, element)?,
            "ellipse" => self.render_ellipse(content, element)?,
            "line" => self.render_line(content, element)?,
            "polyline" => self.render_polyline(content, element, false)?,
            "polygon" => self.render_polyline(content, element, true)?,
            "path" => self.render_path(content, element)?,
            "text" => self.render_text(content, element)?,
            "image" => self.render_image(content, element)?,
            "use" => self.render_use(content, element)?,
            "svg" => self.render_nested_svg(content, element)?,
            _ => {
                // Unknown element, render children
                for child in &element.children {
                    self.render_node(content, child)?;
                }
            }
        }

        // Pop state
        self.pop_state(content);

        Ok(())
    }

    /// Render a group element
    fn render_group(&mut self, content: &mut PageContent, element: &SvgElement) -> Result<()> {
        // Just render children, the group already applied styles and transform
        for child in &element.children {
            self.render_node(content, child)?;
        }
        Ok(())
    }

    /// Render a rectangle
    fn render_rect(&mut self, content: &mut PageContent, element: &SvgElement) -> Result<()> {
        let x = element.get_attr("x").and_then(|s| s.parse().ok()).unwrap_or(0.0);
        let y = element.get_attr("y").and_then(|s| s.parse().ok()).unwrap_or(0.0);
        let width = element.get_attr("width").and_then(|s| s.parse().ok()).unwrap_or(0.0);
        let height = element.get_attr("height").and_then(|s| s.parse().ok()).unwrap_or(0.0);

        let rx = element.get_attr("rx").and_then(|s| s.parse().ok());
        let ry = element.get_attr("ry").and_then(|s| s.parse().ok());

        // Build path data for rectangle
        let path_data = if let (Some(rx), Some(ry)) = (rx, ry) {
            if rx > 0.0 && ry > 0.0 {
                self.rounded_rect_path(x, y, width, height, rx, ry)
            } else {
                self.rect_path(x, y, width, height)
            }
        } else if let Some(r) = rx {
            if r > 0.0 {
                self.rounded_rect_path(x, y, width, height, r, r)
            } else {
                self.rect_path(x, y, width, height)
            }
        } else {
            self.rect_path(x, y, width, height)
        };

        self.render_path_data(content, &path_data)
    }

    fn rect_path(&self, x: f32, y: f32, width: f32, height: f32) -> String {
        format!(
            "M {} {} h {} v {} h {} Z",
            x, y, width, height, -width
        )
    }

    fn rounded_rect_path(&self, x: f32, y: f32, width: f32, height: f32, rx: f32, ry: f32) -> String {
        // Clamp radii
        let rx = rx.min(width / 2.0);
        let ry = ry.min(height / 2.0);

        format!(
            "M {} {} h {} a {} {} 0 0 1 {} {} v {} a {} {} 0 0 1 {} {} h {} a {} {} 0 0 1 {} {} v {} a {} {} 0 0 1 {} {} Z",
            x + rx, y,
            width - 2.0 * rx,
            rx, ry, rx, ry,
            height - 2.0 * ry,
            rx, ry, -rx, ry,
            -(width - 2.0 * rx),
            rx, ry, -rx, -ry,
            -(height - 2.0 * ry),
            rx, ry, rx, -ry
        )
    }

    /// Render a circle
    fn render_circle(&mut self, content: &mut PageContent, element: &SvgElement) -> Result<()> {
        let cx = element.get_attr("cx").and_then(|s| s.parse().ok()).unwrap_or(0.0);
        let cy = element.get_attr("cy").and_then(|s| s.parse().ok()).unwrap_or(0.0);
        let r = element.get_attr("r").and_then(|s| s.parse().ok()).unwrap_or(0.0);

        if r > 0.0 {
            let path_data = self.circle_path(cx, cy, r);
            self.render_path_data(content, &path_data)?;
        }

        Ok(())
    }

    fn circle_path(&self, cx: f32, cy: f32, r: f32) -> String {
        // Use four cubic bezier curves to approximate a circle
        // kappa = 4 * (sqrt(2) - 1) / 3 for a good approximation
        let k = 0.5522847498 * r;

        format!(
            "M {} {} C {} {}, {} {}, {} {} C {} {}, {} {}, {} {} C {} {}, {} {}, {} {} C {} {}, {} {}, {} {} Z",
            cx, cy - r,
            cx + k, cy - r, cx + r, cy - k, cx + r, cy,
            cx + r, cy + k, cx + k, cy + r, cx, cy + r,
            cx - k, cy + r, cx - r, cy + k, cx - r, cy,
            cx - r, cy - k, cx - k, cy - r, cx, cy - r
        )
    }

    /// Render an ellipse
    fn render_ellipse(&mut self, content: &mut PageContent, element: &SvgElement) -> Result<()> {
        let cx = element.get_attr("cx").and_then(|s| s.parse().ok()).unwrap_or(0.0);
        let cy = element.get_attr("cy").and_then(|s| s.parse().ok()).unwrap_or(0.0);
        let rx = element.get_attr("rx").and_then(|s| s.parse().ok()).unwrap_or(0.0);
        let ry = element.get_attr("ry").and_then(|s| s.parse().ok()).unwrap_or(0.0);

        if rx > 0.0 && ry > 0.0 {
            let path_data = self.ellipse_path(cx, cy, rx, ry);
            self.render_path_data(content, &path_data)?;
        }

        Ok(())
    }

    fn ellipse_path(&self, cx: f32, cy: f32, rx: f32, ry: f32) -> String {
        let kx = 0.5522847498 * rx;
        let ky = 0.5522847498 * ry;

        format!(
            "M {} {} C {} {}, {} {}, {} {} C {} {}, {} {}, {} {} C {} {}, {} {}, {} {} C {} {}, {} {}, {} {} Z",
            cx, cy - ry,
            cx + kx, cy - ry, cx + rx, cy - ky, cx + rx, cy,
            cx + rx, cy + ky, cx + kx, cy + ry, cx, cy + ry,
            cx - kx, cy + ry, cx - rx, cy + ky, cx - rx, cy,
            cx - rx, cy - ky, cx - kx, cy - ry, cx, cy - ry
        )
    }

    /// Render a line
    fn render_line(&mut self, content: &mut PageContent, element: &SvgElement) -> Result<()> {
        let x1 = element.get_attr("x1").and_then(|s| s.parse().ok()).unwrap_or(0.0);
        let y1 = element.get_attr("y1").and_then(|s| s.parse().ok()).unwrap_or(0.0);
        let x2 = element.get_attr("x2").and_then(|s| s.parse().ok()).unwrap_or(0.0);
        let y2 = element.get_attr("y2").and_then(|s| s.parse().ok()).unwrap_or(0.0);

        let path_data = format!("M {} {} L {} {}", x1, y1, x2, y2);
        self.render_path_data(content, &path_data)?;

        Ok(())
    }

    /// Render polyline or polygon
    fn render_polyline(&mut self, content: &mut PageContent, element: &SvgElement, close: bool) -> Result<()> {
        let points_str = element.get_attr("points").unwrap_or("");
        let points = self.parse_points(points_str);

        if points.len() < 2 {
            return Ok(());
        }

        let mut path_data = format!("M {} {}", points[0].x, points[0].y);
        for point in &points[1..] {
            path_data.push_str(&format!(" L {} {}", point.x, point.y));
        }

        if close {
            path_data.push_str(" Z");
        }

        self.render_path_data(content, &path_data)?;

        Ok(())
    }

    fn parse_points(&self, s: &str) -> Vec<Point> {
        let mut points = Vec::new();
        let parts: Vec<&str> = s.split(|c: char| c == ',' || c.is_ascii_whitespace()).collect();
        
        let mut i = 0;
        while i + 1 < parts.len() {
            if let (Ok(x), Ok(y)) = (parts[i].parse::<f32>(), parts[i + 1].parse::<f32>()) {
                points.push(Point::new(x, y));
                i += 2;
            } else {
                i += 1;
            }
        }

        points
    }

    /// Render a path element
    fn render_path(&mut self, content: &mut PageContent, element: &SvgElement) -> Result<()> {
        let d = element.get_attr("d").unwrap_or("");
        self.render_path_data(content, d)?;
        Ok(())
    }

    /// Render path data
    fn render_path_data(&mut self, content: &mut PageContent, path_data: &str) -> Result<()> {
        // Parse path data
        let commands = match path::parse_path_data(path_data) {
            Ok(cmds) => cmds,
            Err(_) => return Ok(()), // Silently skip invalid paths
        };

        if commands.is_empty() {
            return Ok(());
        }

        // Apply current transform
        if !self.current_state.transform.is_identity() {
            content.save_state();
            content.append_raw(&format!("{} cm\n", self.current_state.transform.to_pdf_string()));
        }

        // Set up graphics state
        self.setup_graphics_state(content);

        // Generate PDF path operators
        let pdf_path = path::path_commands_to_pdf(&commands);
        content.append_raw(&pdf_path);

        // Fill and/or stroke
        let style = &self.current_state.style;
        let has_fill = style.has_fill();
        let has_stroke = style.has_stroke();

        match (has_fill, has_stroke) {
            (true, true) => {
                if let Some(color) = style.effective_fill_color() {
                    content.set_fill_color(color);
                }
                if let Some(color) = style.effective_stroke_color() {
                    content.set_stroke_color(color);
                }
                content.set_line_width(style.stroke_width);
                match style.fill_rule {
                    FillRule::NonZero => content.fill_and_stroke(),
                    FillRule::EvenOdd => content.append_raw("B*\n"),
                }
            }
            (true, false) => {
                if let Some(color) = style.effective_fill_color() {
                    content.set_fill_color(color);
                }
                match style.fill_rule {
                    FillRule::NonZero => content.fill(),
                    FillRule::EvenOdd => content.append_raw("f*\n"),
                }
            }
            (false, true) => {
                if let Some(color) = style.effective_stroke_color() {
                    content.set_stroke_color(color);
                }
                content.set_line_width(style.stroke_width);
                content.stroke();
            }
            (false, false) => {}
        }

        if !self.current_state.transform.is_identity() {
            content.restore_state();
        }

        Ok(())
    }

    /// Set up graphics state for stroke properties
    fn setup_graphics_state(&self, content: &mut PageContent) {
        let style = &self.current_state.style;

        // Line cap
        if style.stroke_linecap != LineCap::default() {
            content.append_raw(&format!("{} J\n", style.stroke_linecap.to_pdf_int()));
        }

        // Line join
        if style.stroke_linejoin != LineJoin::default() {
            content.append_raw(&format!("{} j\n", style.stroke_linejoin.to_pdf_int()));
        }

        // Miter limit
        if style.stroke_miterlimit != 4.0 {
            content.append_raw(&format!("{} M\n", style.stroke_miterlimit));
        }

        // Dash array
        if let Some(ref dash) = style.stroke_dasharray {
            if !dash.is_empty() {
                let dash_str = dash.iter()
                    .map(|d| format!("{:.3}", d))
                    .collect::<Vec<_>>()
                    .join(" ");
                content.append_raw(&format!("[{}] {} d\n", dash_str, style.stroke_dashoffset));
            }
        }
    }

    /// Render text element
    fn render_text(&mut self, content: &mut PageContent, element: &SvgElement) -> Result<()> {
        let x = element.get_attr("x").and_then(|s| s.parse().ok()).unwrap_or(0.0);
        let y = element.get_attr("y").and_then(|s| s.parse().ok()).unwrap_or(0.0);

        // Collect text content from children
        let mut text_content = String::new();
        for child in &element.children {
            if let SvgNode::Text(text) = child {
                text_content.push_str(text);
            }
        }

        if text_content.trim().is_empty() {
            return Ok(());
        }

        let style = &self.current_state.style;

        // Only render if visible
        if !style.is_visible() {
            return Ok(());
        }

        // Apply transform
        if !self.current_state.transform.is_identity() {
            content.save_state();
            content.append_raw(&format!("{} cm\n", self.current_state.transform.to_pdf_string()));
        }

        // Set up text rendering
        content.begin_text();

        // Set font (use a standard PDF font)
        let font_size = style.font_size;
        content.set_font("F1", font_size);

        // Position text
        // SVG text baseline is different from PDF, adjust y position
        let baseline_offset = font_size * 0.25; // Approximate
        content.text_position(x, y - baseline_offset);

        // Set fill color
        if let Some(color) = style.effective_fill_color() {
            content.set_fill_color(color);
        }

        // Show text
        content.show_text(&text_content);

        content.end_text();

        if !self.current_state.transform.is_identity() {
            content.restore_state();
        }

        Ok(())
    }

    /// Render image element
    fn render_image(&mut self, _content: &mut PageContent, _element: &SvgElement) -> Result<()> {
        // Image rendering would require loading and embedding the image
        // For now, this is a placeholder
        Ok(())
    }

    /// Render use element
    fn render_use(&mut self, content: &mut PageContent, element: &SvgElement) -> Result<()> {
        let href = element.get_attr("href")
            .or_else(|| element.get_attr("xlink:href"))
            .unwrap_or("");

        let x = element.get_attr("x").and_then(|s| s.parse().ok()).unwrap_or(0.0);
        let y = element.get_attr("y").and_then(|s| s.parse().ok()).unwrap_or(0.0);

        // Look up referenced element in definitions
        if let Some(id) = href.strip_prefix('#') {
            if let Some(def) = self.document.definitions.get(id) {
                // Apply translation for x, y attributes
                self.push_state();
                let translate = Transform::translate(x, y);
                self.current_state.transform = self.current_state.transform.multiply(&translate);

                // Render the referenced definition
                // This would need proper handling based on definition type
                
                self.pop_state(content);
            }
        }

        Ok(())
    }

    /// Render nested SVG element
    fn render_nested_svg(&mut self, content: &mut PageContent, element: &SvgElement) -> Result<()> {
        // Get dimensions
        let x = element.get_attr("x").and_then(|s| s.parse().ok()).unwrap_or(0.0);
        let y = element.get_attr("y").and_then(|s| s.parse().ok()).unwrap_or(0.0);
        let width = element.get_attr("width").and_then(|s| s.parse().ok()).unwrap_or(100.0);
        let height = element.get_attr("height").and_then(|s| s.parse().ok()).unwrap_or(100.0);

        // Save state and set up new viewport
        self.push_state();

        // Translate to position
        let translate = Transform::translate(x, y);
        self.current_state.transform = self.current_state.transform.multiply(&translate);

        // Update viewport
        self.current_state.viewport = Rect::new(0.0, 0.0, width, height);

        // Render children
        for child in &element.children {
            self.render_node(content, child)?;
        }

        self.pop_state(content);

        Ok(())
    }

    /// Push current state onto stack
    fn push_state(&mut self) {
        self.state_stack.push(self.current_state.clone());
    }

    /// Pop state from stack
    fn pop_state(&mut self, content: &mut PageContent) {
        if let Some(state) = self.state_stack.pop() {
            self.current_state = state;
        }
    }
}

/// Extension methods for SVG rendering on PageContent
/// 
/// These are implemented as inherent methods via the pdf module
use crate::pdf::PageContent;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_renderer_creation() {
        let doc = SvgDocument::new();
        let renderer = SvgRenderer::new(&doc);
        
        assert_eq!(renderer.current_state.viewport.width, doc.width);
        assert_eq!(renderer.current_state.viewport.height, doc.height);
    }

    #[test]
    fn test_parse_points() {
        let doc = SvgDocument::new();
        let renderer = SvgRenderer::new(&doc);
        
        let points = renderer.parse_points("0,0 100,0 100,100");
        assert_eq!(points.len(), 3);
        assert_eq!(points[0], Point::new(0.0, 0.0));
        assert_eq!(points[1], Point::new(100.0, 0.0));
        assert_eq!(points[2], Point::new(100.0, 100.0));
    }

    #[test]
    fn test_circle_path() {
        let doc = SvgDocument::new();
        let renderer = SvgRenderer::new(&doc);
        
        let path = renderer.circle_path(50.0, 50.0, 25.0);
        assert!(path.starts_with("M"));
        assert!(path.contains("C"));
        assert!(path.ends_with("Z"));
    }
}
