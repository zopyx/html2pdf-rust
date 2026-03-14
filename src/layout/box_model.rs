//! CSS Box Model
//!
//! Implements the CSS box model with content, padding, border, and margin areas.
//! Handles box tree construction from DOM and box dimensions calculations.

use crate::html::{Element, Node, TextNode};
use crate::types::{Rect, Length};

/// Type of layout box
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum BoxType {
    /// Block-level box
    #[default]
    Block,
    /// Inline-level box
    Inline,
    /// Inline-block box
    InlineBlock,
    /// Anonymous block box (for mixed content)
    Anonymous,
    /// Text run
    TextRun,
    /// Flex container
    Flex,
    /// Grid container
    Grid,
}

impl BoxType {
    /// Check if this is a block-level box type
    pub fn is_block_level(&self) -> bool {
        matches!(self, BoxType::Block | BoxType::Flex | BoxType::Grid)
    }

    /// Check if this is an inline-level box type
    pub fn is_inline_level(&self) -> bool {
        matches!(self, BoxType::Inline | BoxType::InlineBlock | BoxType::TextRun)
    }

    /// Check if this establishes a block formatting context
    pub fn establishes_bfc(&self) -> bool {
        matches!(self, BoxType::Block | BoxType::Flex | BoxType::Grid | BoxType::Anonymous)
    }
}

/// Edge sizes (margin, border, padding)
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct EdgeSizes {
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
    pub left: f32,
}

impl EdgeSizes {
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

    /// Get total horizontal edges
    pub fn horizontal(&self) -> f32 {
        self.left + self.right
    }

    /// Get total vertical edges
    pub fn vertical(&self) -> f32 {
        self.top + self.bottom
    }
}

/// Dimensions of a box (content + padding + border + margin)
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Dimensions {
    /// Content box position and size
    pub content: Rect,
    /// Padding area
    pub padding: EdgeSizes,
    /// Border area
    pub border: EdgeSizes,
    /// Margin area
    pub margin: EdgeSizes,
}

impl Dimensions {
    pub fn new(content: Rect) -> Self {
        Self {
            content,
            padding: EdgeSizes::default(),
            border: EdgeSizes::default(),
            margin: EdgeSizes::default(),
        }
    }

    /// Get the padding box rectangle
    pub fn padding_box(&self) -> Rect {
        Rect::new(
            self.content.x - self.padding.left,
            self.content.y - self.padding.top,
            self.content.width + self.padding.horizontal(),
            self.content.height + self.padding.vertical(),
        )
    }

    /// Get the border box rectangle
    pub fn border_box(&self) -> Rect {
        let padding = self.padding_box();
        Rect::new(
            padding.x - self.border.left,
            padding.y - self.border.top,
            padding.width + self.border.horizontal(),
            padding.height + self.border.vertical(),
        )
    }

    /// Get the margin box rectangle
    pub fn margin_box(&self) -> Rect {
        let border = self.border_box();
        Rect::new(
            border.x - self.margin.left,
            border.y - self.margin.top,
            border.width + self.margin.horizontal(),
            border.height + self.margin.vertical(),
        )
    }

    /// Get the total width including padding and border
    pub fn border_box_width(&self) -> f32 {
        self.content.width + self.padding.horizontal() + self.border.horizontal()
    }

    /// Get the total height including padding and border
    pub fn border_box_height(&self) -> f32 {
        self.content.height + self.padding.vertical() + self.border.vertical()
    }

    /// Get the total width including margins
    pub fn margin_box_width(&self) -> f32 {
        self.border_box_width() + self.margin.horizontal()
    }

    /// Get the total height including margins
    pub fn margin_box_height(&self) -> f32 {
        self.border_box_height() + self.margin.vertical()
    }
}

/// A layout box representing a node in the box tree
#[derive(Debug, Clone, PartialEq)]
pub struct LayoutBox {
    /// Type of box
    pub box_type: BoxType,
    /// Associated DOM node (None for anonymous boxes)
    pub node: Option<Node>,
    /// Computed dimensions
    pub dimensions: Dimensions,
    /// Child boxes
    pub children: Vec<LayoutBox>,
    /// Text content (for text runs)
    pub text_content: Option<String>,
    /// Whether this box has been laid out
    pub is_laid_out: bool,
}

impl LayoutBox {
    pub fn new(box_type: BoxType, node: Option<Node>) -> Self {
        Self {
            box_type,
            node,
            dimensions: Dimensions::default(),
            children: Vec::new(),
            text_content: None,
            is_laid_out: false,
        }
    }

    /// Create a block box for an element
    pub fn block_box(element: &Element) -> Self {
        Self::new(BoxType::Block, Some(Node::Element(element.clone())))
    }

    /// Create an inline box for an element
    pub fn inline_box(element: &Element) -> Self {
        Self::new(BoxType::Inline, Some(Node::Element(element.clone())))
    }

    /// Create an anonymous block box
    pub fn anonymous_box() -> Self {
        Self::new(BoxType::Anonymous, None)
    }

    /// Create a text run box
    pub fn text_box(text: &TextNode) -> Self {
        let mut box_ = Self::new(BoxType::TextRun, Some(Node::Text(text.clone())));
        box_.text_content = Some(text.data.clone());
        box_
    }

    /// Append a child box
    pub fn append_child(&mut self, child: LayoutBox) {
        self.children.push(child);
    }

    /// Get the content width
    pub fn content_width(&self) -> f32 {
        self.dimensions.content.width
    }

    /// Get the content height
    pub fn content_height(&self) -> f32 {
        self.dimensions.content.height
    }

    /// Set content width
    pub fn set_content_width(&mut self, width: f32) {
        self.dimensions.content.width = width;
    }

    /// Set content height
    pub fn set_content_height(&mut self, height: f32) {
        self.dimensions.content.height = height;
    }

    /// Set content position
    pub fn set_content_position(&mut self, x: f32, y: f32) {
        self.dimensions.content.x = x;
        self.dimensions.content.y = y;
    }

    /// Check if this box contains a block-level child
    pub fn has_block_child(&self) -> bool {
        self.children.iter().any(|c| c.box_type.is_block_level())
    }

    /// Get the element if this box represents an element
    pub fn element(&self) -> Option<&Element> {
        self.node.as_ref()?.as_element()
    }

    /// Get the tag name if this box represents an element
    pub fn tag_name(&self) -> Option<&str> {
        Some(self.element()?.tag_name())
    }
}

/// Build a box tree from a DOM element
pub fn build_box_tree(element: &Element, display_resolver: &dyn Fn(&Element) -> BoxType) -> LayoutBox {
    let box_type = display_resolver(element);
    let mut box_ = LayoutBox::new(box_type, Some(Node::Element(element.clone())));

    match box_type {
        BoxType::Block | BoxType::Flex | BoxType::Grid => {
            build_block_children(element, &mut box_, display_resolver);
        }
        BoxType::Inline => {
            build_inline_children(element, &mut box_, display_resolver);
        }
        BoxType::InlineBlock => {
            // Inline-block contains both inline and block content
            build_inline_children(element, &mut box_, display_resolver);
        }
        _ => {}
    }

    box_
}

/// Build children for a block container
fn build_block_children(
    element: &Element,
    parent_box: &mut LayoutBox,
    display_resolver: &dyn Fn(&Element) -> BoxType,
) {
    let mut inline_buffer: Vec<Node> = Vec::new();

    for child in element.children() {
        match child {
            Node::Element(child_el) => {
                let child_display = display_resolver(child_el);

                if child_display.is_block_level() {
                    // Flush any buffered inline content as anonymous block
                    if !inline_buffer.is_empty() {
                        let anon = build_anonymous_box(&inline_buffer, display_resolver);
                        parent_box.append_child(anon);
                        inline_buffer.clear();
                    }
                    // Add block child
                    let child_box = build_box_tree(child_el, display_resolver);
                    parent_box.append_child(child_box);
                } else {
                    // Collect inline content
                    inline_buffer.push(child.clone());
                }
            }
            Node::Text(text) => {
                // Only add non-whitespace text or if we're in an inline context
                if !text.data.trim().is_empty() {
                    inline_buffer.push(child.clone());
                }
            }
            _ => {}
        }
    }

    // Flush remaining inline content
    if !inline_buffer.is_empty() {
        let anon = build_anonymous_box(&inline_buffer, display_resolver);
        parent_box.append_child(anon);
    }
}

/// Build children for an inline container
fn build_inline_children(
    element: &Element,
    parent_box: &mut LayoutBox,
    display_resolver: &dyn Fn(&Element) -> BoxType,
) {
    for child in element.children() {
        match child {
            Node::Element(child_el) => {
                let child_box = build_box_tree(child_el, display_resolver);
                parent_box.append_child(child_box);
            }
            Node::Text(text) => {
                if !text.data.is_empty() {
                    let text_box = LayoutBox::text_box(text);
                    parent_box.append_child(text_box);
                }
            }
            _ => {}
        }
    }
}

/// Build an anonymous block box from inline content
fn build_anonymous_box(
    nodes: &[Node],
    display_resolver: &dyn Fn(&Element) -> BoxType,
) -> LayoutBox {
    let mut anon = LayoutBox::anonymous_box();

    for node in nodes {
        match node {
            Node::Element(el) => {
                let child_box = build_box_tree(el, display_resolver);
                anon.append_child(child_box);
            }
            Node::Text(text) => {
                let text_box = LayoutBox::text_box(text);
                anon.append_child(text_box);
            }
            _ => {}
        }
    }

    anon
}

/// Calculate width based on containing block and computed values
pub fn calculate_width(
    box_: &mut LayoutBox,
    containing_block_width: f32,
    specified_width: Option<Length>,
    margins: (Length, Length),
    padding: (Length, Length, Length, Length),
    borders: (Length, Length, Length, Length),
    base_font_size: f32,
) {
    let dims = &mut box_.dimensions;

    // Convert padding
    dims.padding.left = padding.0.to_pt_with_container(base_font_size, containing_block_width);
    dims.padding.right = padding.1.to_pt_with_container(base_font_size, containing_block_width);
    dims.padding.top = padding.2.to_pt_with_container(base_font_size, containing_block_width);
    dims.padding.bottom = padding.3.to_pt_with_container(base_font_size, containing_block_width);

    // Convert borders
    dims.border.left = borders.0.to_pt(base_font_size);
    dims.border.right = borders.1.to_pt(base_font_size);
    dims.border.top = borders.2.to_pt(base_font_size);
    dims.border.bottom = borders.3.to_pt(base_font_size);

    // Calculate available width
    let padding_border_width = dims.padding.horizontal() + dims.border.horizontal();

    // Handle width calculation
    let width = if let Some(w) = specified_width {
        if w.is_auto() {
            // Width depends on margins and containing block
            let margin_left = margins.0.to_pt_with_container(base_font_size, containing_block_width);
            let margin_right = margins.1.to_pt_with_container(base_font_size, containing_block_width);
            
            // For auto width: width = containing_block - margins - padding - border
            let available = containing_block_width - margin_left - margin_right - padding_border_width;
            
            dims.margin.left = margin_left;
            dims.margin.right = margin_right;
            available.max(0.0)
        } else {
            // Fixed width
            dims.margin.left = margins.0.to_pt_with_container(base_font_size, containing_block_width);
            dims.margin.right = margins.1.to_pt_with_container(base_font_size, containing_block_width);
            w.to_pt_with_container(base_font_size, containing_block_width)
        }
    } else {
        // Default to auto
        let margin_left = margins.0.to_pt_with_container(base_font_size, containing_block_width);
        let margin_right = margins.1.to_pt_with_container(base_font_size, containing_block_width);
        let available = containing_block_width - margin_left - margin_right - padding_border_width;
        
        dims.margin.left = margin_left;
        dims.margin.right = margin_right;
        available.max(0.0)
    };

    dims.content.width = width.max(0.0);
}

/// Calculate height based on content or specified value
#[allow(clippy::too_many_arguments)]
pub fn calculate_height(
    box_: &mut LayoutBox,
    containing_block_height: f32,
    specified_height: Option<Length>,
    margins: (Length, Length),
    padding: (Length, Length, Length, Length),
    borders: (Length, Length, Length, Length),
    base_font_size: f32,
    content_height: Option<f32>,
) {
    let dims = &mut box_.dimensions;

    // Convert vertical padding
    dims.padding.top = padding.0.to_pt_with_container(base_font_size, containing_block_height);
    dims.padding.bottom = padding.1.to_pt_with_container(base_font_size, containing_block_height);

    // Convert vertical borders
    dims.border.top = borders.0.to_pt(base_font_size);
    dims.border.bottom = borders.1.to_pt(base_font_size);

    // Convert margins
    dims.margin.top = margins.0.to_pt_with_container(base_font_size, containing_block_height);
    dims.margin.bottom = margins.1.to_pt_with_container(base_font_size, containing_block_height);

    // Calculate height
    let height = if let Some(h) = specified_height {
        if h.is_auto() {
            // Height depends on content
            content_height.unwrap_or(0.0)
        } else {
            h.to_pt_with_container(base_font_size, containing_block_height)
        }
    } else {
        // Default to content height
        content_height.unwrap_or(0.0)
    };

    dims.content.height = height.max(0.0);
}

/// Calculate position based on normal flow
pub fn calculate_position(
    box_: &mut LayoutBox,
    containing_block: &Rect,
    x: f32,
    y: f32,
) {
    box_.dimensions.content.x = containing_block.x + x;
    box_.dimensions.content.y = containing_block.y + y;
}

/// Shrink-to-fit width calculation
#[allow(dead_code)]
pub fn shrink_to_fit_width(box_: &LayoutBox, available_width: f32) -> f32 {
    // For block boxes with auto width in certain contexts
    // Returns the preferred minimum width based on content
    let content_width = box_.children.iter()
        .map(|child| child.dimensions.margin_box_width())
        .fold(0.0, f32::max);
    
    content_width.min(available_width).max(0.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_edge_sizes() {
        let edges = EdgeSizes::all(10.0);
        assert_eq!(edges.horizontal(), 20.0);
        assert_eq!(edges.vertical(), 20.0);

        let edges = EdgeSizes::symmetric(10.0, 20.0);
        assert_eq!(edges.horizontal(), 40.0);
        assert_eq!(edges.vertical(), 20.0);
    }

    #[test]
    fn test_dimensions() {
        let mut dims = Dimensions::new(Rect::new(0.0, 0.0, 100.0, 50.0));
        dims.padding = EdgeSizes::all(10.0);
        dims.border = EdgeSizes::all(5.0);
        dims.margin = EdgeSizes::all(15.0);

        // Content: 100x50
        assert_eq!(dims.content.width, 100.0);
        assert_eq!(dims.content.height, 50.0);

        // Padding box: (100 + 20) x (50 + 20) = 120 x 70
        let padding = dims.padding_box();
        assert_eq!(padding.width, 120.0);
        assert_eq!(padding.height, 70.0);

        // Border box: (120 + 10) x (70 + 10) = 130 x 80
        let border = dims.border_box();
        assert_eq!(border.width, 130.0);
        assert_eq!(border.height, 80.0);

        // Margin box: (130 + 30) x (80 + 30) = 160 x 110
        let margin = dims.margin_box();
        assert_eq!(margin.width, 160.0);
        assert_eq!(margin.height, 110.0);
    }

    #[test]
    fn test_box_type() {
        assert!(BoxType::Block.is_block_level());
        assert!(!BoxType::Block.is_inline_level());
        
        assert!(BoxType::Inline.is_inline_level());
        assert!(!BoxType::Inline.is_block_level());
        
        assert!(BoxType::Block.establishes_bfc());
        assert!(BoxType::Anonymous.establishes_bfc());
    }

    #[test]
    fn test_layout_box_creation() {
        let el = Element::new("div", vec![]);
        let box_ = LayoutBox::block_box(&el);
        
        assert_eq!(box_.box_type, BoxType::Block);
        assert!(box_.element().is_some());
        assert_eq!(box_.tag_name(), Some("div"));
    }

    #[test]
    fn test_text_box() {
        let text = TextNode::new("Hello World");
        let box_ = LayoutBox::text_box(&text);
        
        assert_eq!(box_.box_type, BoxType::TextRun);
        assert_eq!(box_.text_content, Some("Hello World".to_string()));
    }
}
