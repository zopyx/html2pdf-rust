//! Layout Engine
//!
//! The layout engine takes a DOM Document and CSS Stylesheet,
//! builds a box tree, performs layout computation, and produces
//! positioned content ready for PDF rendering.
//!
//! # Architecture
//!
//! ```
//! DOM + CSS → Box Tree → Layout → Positioned Boxes → PDF
//! ```
//!
//! ## Components
//!
//! - **box_model**: CSS box model implementation (margin, border, padding, content)
//! - **style**: Style computation (cascade, specificity, inheritance)
//! - **text**: Text layout and line breaking
//! - **flow**: Normal flow layout (block and inline formatting contexts)

mod box_model;
mod flow;
mod style;
mod text;

pub use box_model::{
    LayoutBox, BoxType, Dimensions, EdgeSizes,
    build_box_tree, calculate_width, calculate_height, calculate_position,
};
pub use flow::{
    BlockFormattingContext, InlineFormattingContext,
    layout_block_children, layout_inline_children,
};
pub use style::{
    ComputedStyle, StyleResolver, Display, Position, Float, Clear,
    BorderStyle, FontWeight, FontStyle, LineHeight,
    TextAlign, TextDecoration, TextTransform,
    WhiteSpace, WordWrap, Visibility, Overflow,
    ZIndex, PageBreak, PageBreakInside,
};
pub use text::{
    TextLayout, LineBreaker, Line, TextFragment, TextMetrics,
    align_line, calculate_text_bounds,
};

use crate::css::Stylesheet;
use crate::html::{Document, Element, Node};
use crate::types::{Rect, Size, Point, PaperSize, Margins, Orientation, PdfError, Result};

/// Layout context holds global layout state
#[derive(Debug, Clone)]
pub struct LayoutContext {
    /// Page size in points
    pub page_size: Size,
    /// Page margins
    pub margins: Margins,
    /// Current page number
    pub page_number: u32,
    /// Total pages
    pub total_pages: u32,
    /// Base font size (for em/rem calculations)
    pub base_font_size: f32,
    /// Current containing block
    pub containing_block: Rect,
    /// Whether we're in a page break context
    pub in_page_break: bool,
}

impl LayoutContext {
    /// Create a new layout context with default settings
    pub fn new() -> Self {
        let (width, height) = PaperSize::A4.size();
        Self {
            page_size: Size::new(width, height),
            margins: Margins::all(72.0), // 1 inch margins
            page_number: 1,
            total_pages: 1,
            base_font_size: 12.0,
            containing_block: Rect::new(72.0, 72.0, width - 144.0, height - 144.0),
            in_page_break: false,
        }
    }

    /// Create layout context with custom page settings
    pub fn with_page_size(paper_size: PaperSize, orientation: Orientation) -> Self {
        let (mut width, mut height) = paper_size.size();
        
        if orientation == Orientation::Landscape {
            std::mem::swap(&mut width, &mut height);
        }

        let margins = Margins::all(72.0);
        
        Self {
            page_size: Size::new(width, height),
            margins,
            page_number: 1,
            total_pages: 1,
            base_font_size: 12.0,
            containing_block: Rect::new(
                margins.left,
                margins.top,
                width - margins.left - margins.right,
                height - margins.top - margins.bottom,
            ),
            in_page_break: false,
        }
    }

    /// Set margins
    pub fn with_margins(mut self, margins: Margins) -> Self {
        self.margins = margins;
        self.containing_block = Rect::new(
            margins.left,
            margins.top,
            self.page_size.width - margins.left - margins.right,
            self.page_size.height - margins.top - margins.bottom,
        );
        self
    }

    /// Get the content area rectangle for a page
    pub fn content_area(&self) -> Rect {
        self.containing_block
    }

    /// Get page width
    pub fn page_width(&self) -> f32 {
        self.page_size.width
    }

    /// Get page height
    pub fn page_height(&self) -> f32 {
        self.page_size.height
    }

    /// Get content width
    pub fn content_width(&self) -> f32 {
        self.containing_block.width
    }

    /// Get content height
    pub fn content_height(&self) -> f32 {
        self.containing_block.height
    }
}

impl Default for LayoutContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Layout engine that processes documents
#[derive(Debug)]
pub struct LayoutEngine {
    context: LayoutContext,
    style_resolver: StyleResolver,
    viewport_width: f32,
}

impl LayoutEngine {
    /// Create a new layout engine
    pub fn new() -> Self {
        Self {
            context: LayoutContext::new(),
            style_resolver: StyleResolver::new(),
            viewport_width: 800.0, // Default viewport width in pixels
        }
    }

    /// Create layout engine with custom context
    pub fn with_context(context: LayoutContext) -> Self {
        Self {
            viewport_width: context.content_width(),
            context,
            style_resolver: StyleResolver::new(),
        }
    }

    /// Add a stylesheet to the engine
    pub fn add_stylesheet(&mut self, stylesheet: Stylesheet) {
        self.style_resolver.add_stylesheet(stylesheet);
    }

    /// Set the viewport width for media queries
    pub fn set_viewport_width(&mut self, width: f32) {
        self.viewport_width = width;
    }

    /// Layout a complete document
    pub fn layout_document(&mut self, document: &Document) -> Result<LayoutBox> {
        // Get the body element
        let body = document.body_element();

        // Build the box tree from the DOM
        let mut root_box = build_box_tree(body, &|element| {
            self.style_resolver.resolve_display(element)
        });

        // Create the initial block formatting context
        let content_area = self.context.content_area();
        let mut bfc = BlockFormattingContext::new(content_area);

        // Perform layout
        self.layout_box_tree(&mut root_box, &mut bfc)?;

        Ok(root_box)
    }

    /// Layout a box tree starting from the root
    fn layout_box_tree(
        &mut self,
        root: &mut LayoutBox,
        bfc: &mut BlockFormattingContext,
    ) -> Result<()> {
        // Set up the root box dimensions
        root.dimensions.content.x = bfc.containing_block.x;
        root.dimensions.content.y = bfc.containing_block.y;
        root.dimensions.content.width = bfc.containing_block.width;

        // Layout children
        layout_block_children(
            root,
            bfc,
            &|element| self.style_resolver.compute_style(element, None),
            self.context.base_font_size,
        );

        // Calculate final height based on content
        if root.dimensions.content.height == 0.0 {
            let content_height: f32 = root.children.iter()
                .map(|child| child.dimensions.margin_box_height())
                .sum();
            root.dimensions.content.height = content_height;
        }

        root.is_laid_out = true;
        Ok(())
    }

    /// Layout a specific element (for fragments)
    pub fn layout_element(&mut self, element: &Element, containing_block: Rect) -> Result<LayoutBox> {
        // Build box tree for this element
        let mut box_ = build_box_tree(element, &|el| {
            self.style_resolver.resolve_display(el)
        });

        // Layout in a new BFC
        let mut bfc = BlockFormattingContext::new(containing_block);
        
        // Compute style
        let style = self.style_resolver.compute_style(element, None);

        // Calculate dimensions
        calculate_width(
            &mut box_,
            containing_block.width,
            Some(style.width),
            (style.margin_left, style.margin_right),
            (style.padding_left, style.padding_right, style.padding_top, style.padding_bottom),
            (
                crate::types::Length::Px(style.border_left_width.to_pt(self.context.base_font_size)),
                crate::types::Length::Px(style.border_right_width.to_pt(self.context.base_font_size)),
                crate::types::Length::Px(style.border_top_width.to_pt(self.context.base_font_size)),
                crate::types::Length::Px(style.border_bottom_width.to_pt(self.context.base_font_size)),
            ),
            self.context.base_font_size,
        );

        // Position
        box_.dimensions.content.x = containing_block.x + box_.dimensions.margin.left + box_.dimensions.border.left + box_.dimensions.padding.left;
        box_.dimensions.content.y = containing_block.y + box_.dimensions.margin.top + box_.dimensions.border.top + box_.dimensions.padding.top;

        // Layout children
        if !box_.children.is_empty() {
            layout_block_children(
                &mut box_,
                &mut bfc,
                &|el| self.style_resolver.compute_style(el, Some(&style)),
                self.context.base_font_size,
            );
        }

        // Calculate height
        let content_height = box_.dimensions.content.height;
        calculate_height(
            &mut box_,
            containing_block.height,
            Some(style.height),
            (style.margin_top, style.margin_bottom),
            (
                crate::types::Length::Px(style.padding_top.to_pt(self.context.base_font_size)),
                crate::types::Length::Px(style.padding_bottom.to_pt(self.context.base_font_size)),
                crate::types::Length::Px(0.0),
                crate::types::Length::Px(0.0),
            ),
            (
                crate::types::Length::Px(style.border_top_width.to_pt(self.context.base_font_size)),
                crate::types::Length::Px(style.border_bottom_width.to_pt(self.context.base_font_size)),
                crate::types::Length::Px(0.0),
                crate::types::Length::Px(0.0),
            ),
            self.context.base_font_size,
            Some(content_height),
        );

        box_.is_laid_out = true;
        Ok(box_)
    }

    /// Get the computed style for an element
    pub fn compute_style(&self, element: &Element, parent: Option<&ComputedStyle>) -> ComputedStyle {
        self.style_resolver.compute_style(element, parent)
    }

    /// Get the current layout context
    pub fn context(&self) -> &LayoutContext {
        &self.context
    }

    /// Get mutable access to the layout context
    pub fn context_mut(&mut self) -> &mut LayoutContext {
        &mut self.context
    }
}

impl Default for LayoutEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Layout a document and return the positioned box tree
pub fn layout_document(
    document: &Document,
    stylesheets: &[Stylesheet],
    context: Option<LayoutContext>,
) -> Result<LayoutBox> {
    let mut engine = match context {
        Some(ctx) => LayoutEngine::with_context(ctx),
        None => LayoutEngine::new(),
    };

    for stylesheet in stylesheets {
        engine.add_stylesheet(stylesheet.clone());
    }

    engine.layout_document(document)
}

/// Create a layout box tree without performing layout
pub fn build_layout_tree(document: &Document, stylesheets: &[Stylesheet]) -> Result<LayoutBox> {
    let mut resolver = StyleResolver::new();
    
    for stylesheet in stylesheets {
        resolver.add_stylesheet(stylesheet.clone());
    }

    let body = document.body_element();
    let root_box = build_box_tree(body, &|element| {
        resolver.resolve_display(element)
    });

    Ok(root_box)
}

/// Collect all positioned boxes from a layout tree
pub fn collect_positioned_boxes(root: &LayoutBox) -> Vec<&LayoutBox> {
    let mut result = Vec::new();
    collect_boxes_recursive(root, &mut result);
    result
}

fn collect_boxes_recursive<'a>(box_: &'a LayoutBox, result: &mut Vec<&'a LayoutBox>) {
    if box_.is_laid_out {
        result.push(box_);
    }

    for child in &box_.children {
        collect_boxes_recursive(child, result);
    }
}

/// Print layout tree for debugging
pub fn print_layout_tree(box_: &LayoutBox, indent: usize) {
    let indent_str = "  ".repeat(indent);
    let dims = &box_.dimensions;
    
    println!(
        "{}Box({:?}): content={:.1}x{:.1}@({:.1},{:.1}), margin={:.1},{:.1},{:.1},{:.1}",
        indent_str,
        box_.box_type,
        dims.content.width, dims.content.height,
        dims.content.x, dims.content.y,
        dims.margin.top, dims.margin.right, dims.margin.bottom, dims.margin.left
    );

    for child in &box_.children {
        print_layout_tree(child, indent + 1);
    }
}

/// Convert layout boxes to PDF page content
/// 
/// This is the bridge between layout and PDF rendering.
pub fn boxes_to_pdf_content(boxes: &[&LayoutBox]) -> Vec<PdfBox> {
    boxes.iter().map(|b| PdfBox::from_layout_box(b)).collect()
}

/// A simplified box representation for PDF output
#[derive(Debug, Clone)]
pub struct PdfBox {
    /// Position in PDF coordinates (origin at bottom-left)
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    /// Box type
    pub box_type: BoxType,
    /// Text content if applicable
    pub text: Option<String>,
    /// Background color
    pub background_color: Option<crate::types::Color>,
    /// Border widths
    pub border: EdgeSizes,
    /// Whether this box needs to be rendered
    pub is_visible: bool,
}

impl PdfBox {
    /// Convert a layout box to a PDF box
    pub fn from_layout_box(box_: &LayoutBox) -> Self {
        let border_box = box_.dimensions.border_box();
        
        Self {
            x: border_box.x,
            y: border_box.y,
            width: border_box.width,
            height: border_box.height,
            box_type: box_.box_type,
            text: box_.text_content.clone(),
            background_color: None, // Would come from computed style
            border: box_.dimensions.border,
            is_visible: box_.is_laid_out && box_.box_type != BoxType::Anonymous,
        }
    }

    /// Convert from PDF coordinates (origin at bottom-left) to layout coordinates
    pub fn to_pdf_coordinates(&self, page_height: f32) -> Self {
        let mut result = self.clone();
        // PDF origin is at bottom-left, layout origin is at top-left
        result.y = page_height - self.y - self.height;
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::html;

    #[test]
    fn test_layout_context() {
        let ctx = LayoutContext::new();
        assert!(ctx.page_width() > 0.0);
        assert!(ctx.page_height() > 0.0);
        assert!(ctx.content_width() > 0.0);
        assert!(ctx.content_height() > 0.0);
    }

    #[test]
    fn test_layout_context_with_page_size() {
        let ctx = LayoutContext::with_page_size(PaperSize::Letter, Orientation::Portrait);
        assert_eq!(ctx.page_width(), 612.0);
        assert_eq!(ctx.page_height(), 792.0);
    }

    #[test]
    fn test_layout_engine() {
        let mut engine = LayoutEngine::new();
        
        let html = "<html><body><div>Hello</div></body></html>";
        let doc = html::parse_html(html).unwrap();
        
        let result = engine.layout_document(&doc);
        assert!(result.is_ok());
        
        let root = result.unwrap();
        assert!(root.is_laid_out);
    }

    #[test]
    fn test_layout_document() {
        let html = "<html><body><p>Hello World</p></body></html>";
        let doc = html::parse_html(html).unwrap();
        
        let result = layout_document(&doc, &[], None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_collect_positioned_boxes() {
        let mut root = LayoutBox::new(BoxType::Block, None);
        root.is_laid_out = true;
        
        let child1 = LayoutBox::new(BoxType::Block, None);
        let mut child2 = LayoutBox::new(BoxType::Block, None);
        child2.is_laid_out = true;
        
        root.children.push(child1);
        root.children.push(child2);
        
        let boxes = collect_positioned_boxes(&root);
        assert_eq!(boxes.len(), 2); // root and child2
    }

    #[test]
    fn test_pdf_box_conversion() {
        let mut box_ = LayoutBox::new(BoxType::Block, None);
        box_.dimensions.content = Rect::new(10.0, 10.0, 100.0, 50.0);
        box_.dimensions.padding = EdgeSizes::all(5.0);
        box_.dimensions.border = EdgeSizes::all(2.0);
        box_.is_laid_out = true;
        
        let pdf_box = PdfBox::from_layout_box(&box_);
        
        // Border box should be: content + padding + border
        // Width: 100 + 10 + 4 = 114
        // Height: 50 + 10 + 4 = 64
        assert_eq!(pdf_box.width, 114.0);
        assert_eq!(pdf_box.height, 64.0);
    }
}
