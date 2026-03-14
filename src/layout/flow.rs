//! Normal Flow Layout
//!
//! Implements block formatting contexts and inline formatting contexts
//! for normal document flow according to CSS 2.1 specification.

use crate::types::{Rect, Length};
use crate::layout::box_model::{
    LayoutBox, BoxType, Dimensions, EdgeSizes,
    calculate_width, calculate_height, calculate_position,
};
use crate::layout::style::{ComputedStyle, Display, Position, Float, Clear};
use crate::layout::text::{TextLayout, LineBreaker, Line, align_line};

/// Block formatting context
/// 
/// A BFC is a region in which block boxes are laid out according to
/// the block formatting rules.
#[derive(Debug, Clone)]
pub struct BlockFormattingContext {
    /// Containing block for this context
    pub containing_block: Rect,
    /// Current vertical position
    pub current_y: f32,
    /// Floats affecting this context
    pub floats: Vec<FloatBox>,
    /// Whether this context has been cleared
    pub is_cleared: bool,
}

impl BlockFormattingContext {
    pub fn new(containing_block: Rect) -> Self {
        Self {
            containing_block,
            current_y: containing_block.y,
            floats: Vec::new(),
            is_cleared: false,
        }
    }

    /// Get the available width at the current position
    pub fn available_width(&self) -> f32 {
        self.containing_block.width
    }

    /// Get the remaining height in the containing block
    pub fn remaining_height(&self) -> f32 {
        (self.containing_block.y + self.containing_block.height - self.current_y).max(0.0)
    }

    /// Advance the current Y position
    pub fn advance(&mut self, amount: f32) {
        self.current_y += amount;
    }

    /// Check if there's room for a box of given height
    pub fn has_room_for(&self, height: f32) -> bool {
        self.remaining_height() >= height
    }

    /// Clear floats
    pub fn clear_floats(&mut self, clear: Clear) {
        match clear {
            Clear::Left | Clear::Right | Clear::Both => {
                // Find the bottom of all relevant floats
                let clear_y = self.floats.iter()
                    .filter(|f| {
                        match clear {
                            Clear::Both => true,
                            Clear::Left => f.is_left,
                            Clear::Right => !f.is_left,
                            Clear::None => false,
                        }
                    })
                    .map(|f| f.rect.y + f.rect.height)
                    .fold(0.0, f32::max);
                
                if clear_y > self.current_y {
                    self.current_y = clear_y;
                }
                self.floats.clear();
            }
            Clear::None => {}
        }
    }
}

/// A floated box
#[derive(Debug, Clone)]
pub struct FloatBox {
    pub rect: Rect,
    pub is_left: bool,
    pub box_index: usize,
}

/// Inline formatting context
///
/// An IFC is a region in which inline boxes are laid out horizontally
/// according to the inline formatting rules.
#[derive(Debug, Clone)]
pub struct InlineFormattingContext {
    /// Containing block width
    pub available_width: f32,
    /// Current line boxes
    pub line_boxes: Vec<LineBox>,
    /// Current line being built
    pub current_line: LineBox,
    /// Current x position in the line
    pub current_x: f32,
    /// Line height
    pub line_height: f32,
    /// Vertical offset from containing block top
    pub vertical_offset: f32,
}

/// A line box in an inline formatting context
#[derive(Debug, Clone, Default)]
pub struct LineBox {
    /// Inline boxes in this line
    pub fragments: Vec<InlineFragment>,
    /// Line width
    pub width: f32,
    /// Line height
    pub height: f32,
    /// Baseline position
    pub baseline: f32,
    /// Line top position
    pub top: f32,
}

/// A fragment of an inline box
#[derive(Debug, Clone)]
pub struct InlineFragment {
    /// Reference to the source box
    pub box_index: usize,
    /// Width of this fragment
    pub width: f32,
    /// Height of this fragment
    pub height: f32,
    /// Position within line
    pub x: f32,
    /// Whether this is a text fragment
    pub is_text: bool,
    /// Text content (if text)
    pub text: Option<String>,
}

impl InlineFormattingContext {
    pub fn new(available_width: f32, line_height: f32) -> Self {
        Self {
            available_width,
            line_boxes: Vec::new(),
            current_line: LineBox::default(),
            current_x: 0.0,
            line_height,
            vertical_offset: 0.0,
        }
    }

    /// Add an inline fragment
    pub fn add_fragment(&mut self, fragment: InlineFragment) {
        let fragment_width = fragment.width;
        self.current_line.fragments.push(fragment);
        self.current_line.width += fragment_width;
        self.current_x += fragment_width;
    }

    /// Check if there's room for a fragment of given width
    pub fn has_room_for(&self, width: f32) -> bool {
        self.current_x + width <= self.available_width
    }

    /// Finish the current line and start a new one
    pub fn finish_line(&mut self) {
        if !self.current_line.fragments.is_empty() {
            // Calculate line height based on fragments
            let max_height = self.current_line.fragments.iter()
                .map(|f| f.height)
                .fold(0.0, f32::max)
                .max(self.line_height);
            
            self.current_line.height = max_height;
            self.current_line.baseline = max_height * 0.8; // Approximate baseline
            self.current_line.top = self.vertical_offset;
            
            self.line_boxes.push(self.current_line.clone());
            self.vertical_offset += max_height;
        }
        
        self.current_line = LineBox::default();
        self.current_x = 0.0;
    }

    /// Get total height of laid out lines
    pub fn total_height(&self) -> f32 {
        self.vertical_offset + if self.current_line.fragments.is_empty() {
            0.0
        } else {
            self.current_line.height
        }
    }

    /// Complete layout
    pub fn finish(mut self) -> Vec<LineBox> {
        self.finish_line();
        self.line_boxes
    }
}

/// Layout block-level children
pub fn layout_block_children(
    box_: &mut LayoutBox,
    bfc: &mut BlockFormattingContext,
    style_resolver: &dyn Fn(&crate::html::Element) -> ComputedStyle,
    base_font_size: f32,
) {
    let content_width = box_.dimensions.content.width;
    let content_x = box_.dimensions.content.x;
    let mut content_height = 0.0;

    // Create a child BFC
    let child_containing_block = Rect::new(
        content_x + box_.dimensions.padding.left + box_.dimensions.border.left,
        bfc.current_y + box_.dimensions.padding.top + box_.dimensions.border.top,
        content_width - box_.dimensions.padding.horizontal() - box_.dimensions.border.horizontal(),
        f32::MAX, // Unlimited height for now
    );

    let mut child_bfc = BlockFormattingContext::new(child_containing_block);

    for child_index in 0..box_.children.len() {
        let child = &mut box_.children[child_index];
        
        // Handle clearance
        if let Some(element) = child.element() {
            let style = style_resolver(element);
            child_bfc.clear_floats(style.clear);
        }

        // Position the child
        let child_y = child_bfc.current_y - bfc.current_y;
        child.dimensions.content.x = child_containing_block.x;
        child.dimensions.content.y = child_bfc.current_y;

        // Layout the child based on its type
        match child.box_type {
            BoxType::Block | BoxType::Anonymous => {
                layout_block_box(child, &mut child_bfc, style_resolver, base_font_size);
            }
            BoxType::Inline | BoxType::InlineBlock => {
                // Inline content in block context - should have been wrapped in anonymous block
                layout_inline_children(child, &mut child_bfc, style_resolver, base_font_size);
            }
            BoxType::TextRun => {
                // Text runs should be inside inline containers
                layout_text_run(child, &mut child_bfc, style_resolver, base_font_size);
            }
            _ => {}
        }

        // Account for margin collapse (simplified - only collapse top margins)
        let margin_top = child.dimensions.margin.top;
        let box_height = child.dimensions.margin_box_height();
        
        content_height = child_y + box_height;
        child_bfc.advance(box_height);
    }

    // Set the content height (accounting for padding and border)
    box_.dimensions.content.height = content_height.max(0.0);
}

/// Layout a single block box
fn layout_block_box(
    box_: &mut LayoutBox,
    bfc: &mut BlockFormattingContext,
    style_resolver: &dyn Fn(&crate::html::Element) -> ComputedStyle,
    base_font_size: f32,
) {
    // Get computed style for this element
    let style = if let Some(element) = box_.element() {
        style_resolver(element)
    } else {
        ComputedStyle::default()
    };

    let containing_block_width = bfc.available_width();

    // Calculate width
    calculate_width(
        box_,
        containing_block_width,
        Some(style.width),
        (style.margin_left, style.margin_right),
        (style.padding_left, style.padding_right, style.padding_top, style.padding_bottom),
        (
            Length::Px(style.border_left_width.to_pt(base_font_size)),
            Length::Px(style.border_right_width.to_pt(base_font_size)),
            Length::Px(style.border_top_width.to_pt(base_font_size)),
            Length::Px(style.border_bottom_width.to_pt(base_font_size)),
        ),
        base_font_size,
    );

    // Position horizontally
    let margin_left = box_.dimensions.margin.left;
    box_.dimensions.content.x = bfc.containing_block.x + margin_left + box_.dimensions.border.left + box_.dimensions.padding.left;

    // Layout children
    if !box_.children.is_empty() {
        layout_block_children(box_, bfc, style_resolver, base_font_size);
    }

    // Calculate height (auto height based on content)
    let content_height = box_.dimensions.content.height;
    calculate_height(
        box_,
        f32::MAX, // No containing block height constraint
        Some(style.height),
        (style.margin_top, style.margin_bottom),
        (
            Length::Px(style.padding_top.to_pt(base_font_size)),
            Length::Px(style.padding_bottom.to_pt(base_font_size)),
            Length::Px(0.0),
            Length::Px(0.0),
        ),
        (
            Length::Px(style.border_top_width.to_pt(base_font_size)),
            Length::Px(style.border_bottom_width.to_pt(base_font_size)),
            Length::Px(0.0),
            Length::Px(0.0),
        ),
        base_font_size,
        Some(content_height),
    );

    box_.is_laid_out = true;
}

/// Layout inline children
pub fn layout_inline_children(
    box_: &mut LayoutBox,
    bfc: &mut BlockFormattingContext,
    style_resolver: &dyn Fn(&crate::html::Element) -> ComputedStyle,
    base_font_size: f32,
) {
    // Get style
    let style = if let Some(element) = box_.element() {
        style_resolver(element)
    } else {
        ComputedStyle::default()
    };

    let available_width = bfc.available_width() - box_.dimensions.padding.horizontal() - box_.dimensions.border.horizontal();
    let line_height = match style.line_height {
        crate::layout::style::LineHeight::Number(n) => style.font_size.to_pt(base_font_size) * n,
        crate::layout::style::LineHeight::Length(l) => l.to_pt(base_font_size),
        crate::layout::style::LineHeight::Normal => style.font_size.to_pt(base_font_size) * 1.2,
    };

    let mut ifc = InlineFormattingContext::new(available_width, line_height);

    // Layout all inline children
    for (i, child) in box_.children.iter_mut().enumerate() {
        match child.box_type {
            BoxType::TextRun => {
                layout_inline_text(child, &mut ifc, &style, base_font_size);
            }
            BoxType::Inline => {
                layout_inline_box(child, &mut ifc, style_resolver, base_font_size);
            }
            BoxType::InlineBlock => {
                layout_inline_block_box(child, &mut ifc, style_resolver, base_font_size);
            }
            _ => {}
        }
    }

    // Finish layout
    let line_boxes = ifc.finish();
    
    // Calculate total dimensions
    let total_height: f32 = line_boxes.iter().map(|l| l.height).sum();
    let max_width = line_boxes.iter().map(|l| l.width).fold(0.0, f32::max);

    box_.dimensions.content.width = max_width;
    box_.dimensions.content.height = total_height;

    // Store line boxes for rendering
    // (In a full implementation, these would be attached to the box)
    box_.is_laid_out = true;
}

/// Layout text as inline content
fn layout_inline_text(
    box_: &mut LayoutBox,
    ifc: &mut InlineFormattingContext,
    parent_style: &ComputedStyle,
    base_font_size: f32,
) {
    let text = box_.text_content.as_deref().unwrap_or("");
    if text.is_empty() {
        return;
    }

    let font_size = parent_style.font_size.to_pt(base_font_size);
    let text_layout = TextLayout::new(font_size);

    // Layout text with wrapping
    let lines = text_layout.layout_text(text, ifc.available_width, parent_style);

    // Create fragments for each line
    for line in lines {
        // Check if we need to start a new line
        if !ifc.has_room_for(line.width) && !ifc.current_line.fragments.is_empty() {
            ifc.finish_line();
        }

        // Add fragment for this line's content
        let fragment = InlineFragment {
            box_index: 0, // Will be set by caller
            width: line.width,
            height: line.height,
            x: ifc.current_x,
            is_text: true,
            text: Some(line.fragments.iter().map(|f| f.text.clone()).collect::<String>()),
        };

        ifc.add_fragment(fragment);
    }
}

/// Layout an inline box
fn layout_inline_box(
    box_: &mut LayoutBox,
    ifc: &mut InlineFormattingContext,
    style_resolver: &dyn Fn(&crate::html::Element) -> ComputedStyle,
    base_font_size: f32,
) {
    let style = if let Some(element) = box_.element() {
        style_resolver(element)
    } else {
        ComputedStyle::default()
    };

    // For inline boxes, layout children inline
    let mut inline_width = 0.0;
    let mut inline_height: f32 = 0.0;

    for child in box_.children.iter_mut() {
        match child.box_type {
            BoxType::TextRun => {
                let font_size = style.font_size.to_pt(base_font_size);
                let text_layout = TextLayout::new(font_size);
                let (width, height) = text_layout.measure_text(
                    child.text_content.as_deref().unwrap_or("")
                );
                
                inline_width += width;
                inline_height = inline_height.max(height);

                let fragment = InlineFragment {
                    box_index: 0,
                    width,
                    height,
                    x: ifc.current_x + inline_width - width,
                    is_text: true,
                    text: child.text_content.clone(),
                };
                ifc.add_fragment(fragment);
            }
            BoxType::Inline => {
                // Recursively layout nested inline
                let mut child_ifc = InlineFormattingContext::new(ifc.available_width, ifc.line_height);
                layout_inline_box(child, &mut child_ifc, style_resolver, base_font_size);
                inline_height = inline_height.max(child_ifc.total_height());
            }
            _ => {}
        }
    }

    box_.dimensions.content.width = inline_width;
    box_.dimensions.content.height = inline_height;
}

/// Layout an inline-block box
fn layout_inline_block_box(
    box_: &mut LayoutBox,
    ifc: &mut InlineFormattingContext,
    style_resolver: &dyn Fn(&crate::html::Element) -> ComputedStyle,
    base_font_size: f32,
) {
    let style = if let Some(element) = box_.element() {
        style_resolver(element)
    } else {
        ComputedStyle::default()
    };

    // Inline-block is laid out as a block but flows inline
    let containing_block = Rect::new(0.0, 0.0, ifc.available_width, f32::MAX);
    let mut bfc = BlockFormattingContext::new(containing_block);

    // Layout as block
    layout_block_children(box_, &mut bfc, style_resolver, base_font_size);

    // Create a fragment for the inline-block
    let mut fragment = InlineFragment {
        box_index: 0,
        width: box_.dimensions.border_box_width(),
        height: box_.dimensions.border_box_height(),
        x: ifc.current_x,
        is_text: false,
        text: None,
    };

    // Check if it fits
    if !ifc.has_room_for(fragment.width) && !ifc.current_line.fragments.is_empty() {
        ifc.finish_line();
        fragment.x = 0.0;
    }

    ifc.add_fragment(fragment);
}

/// Layout a text run in block context
fn layout_text_run(
    box_: &mut LayoutBox,
    bfc: &mut BlockFormattingContext,
    style_resolver: &dyn Fn(&crate::html::Element) -> ComputedStyle,
    base_font_size: f32,
) {
    // Text runs in block context should be wrapped
    // This is a simplified implementation
    let text = box_.text_content.as_deref().unwrap_or("");
    
    let text_layout = TextLayout::new(base_font_size);
    let style = ComputedStyle::default(); // Use default for anonymous boxes

    let lines = text_layout.layout_text(text, bfc.available_width(), &style);
    
    let total_height: f32 = lines.iter().map(|l| l.height).sum();
    let max_width = lines.iter().map(|l| l.width).fold(0.0, f32::max);

    box_.dimensions.content.width = max_width;
    box_.dimensions.content.height = total_height;
    box_.dimensions.content.x = bfc.containing_block.x;
    box_.dimensions.content.y = bfc.current_y;
}

/// Calculate clearance for floated elements
pub fn calculate_clearance(
    float_boxes: &[FloatBox],
    clear: Clear,
    current_y: f32,
) -> f32 {
    match clear {
        Clear::None => 0.0,
        Clear::Left => {
            float_boxes.iter()
                .filter(|f| f.is_left)
                .map(|f| f.rect.y + f.rect.height - current_y)
                .fold(0.0, f32::max)
        }
        Clear::Right => {
            float_boxes.iter()
                .filter(|f| !f.is_left)
                .map(|f| f.rect.y + f.rect.height - current_y)
                .fold(0.0, f32::max)
        }
        Clear::Both => {
            float_boxes.iter()
                .map(|f| f.rect.y + f.rect.height - current_y)
                .fold(0.0, f32::max)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::html::Element;

    fn default_style_resolver(_: &crate::html::Element) -> ComputedStyle {
        ComputedStyle::default()
    }

    #[test]
    fn test_block_formatting_context() {
        let containing = Rect::new(0.0, 0.0, 500.0, 800.0);
        let mut bfc = BlockFormattingContext::new(containing);

        assert_eq!(bfc.available_width(), 500.0);
        assert!(bfc.has_room_for(100.0));

        bfc.advance(100.0);
        assert_eq!(bfc.current_y, 100.0);
    }

    #[test]
    fn test_inline_formatting_context() {
        let mut ifc = InlineFormattingContext::new(200.0, 20.0);

        let fragment = InlineFragment {
            box_index: 0,
            width: 50.0,
            height: 20.0,
            x: 0.0,
            is_text: true,
            text: Some("Hello".to_string()),
        };

        ifc.add_fragment(fragment);
        assert_eq!(ifc.current_x, 50.0);
        assert!(ifc.has_room_for(100.0));
        assert!(!ifc.has_room_for(200.0));
    }

    #[test]
    fn test_clearance() {
        let floats = vec![
            FloatBox {
                rect: Rect::new(0.0, 0.0, 100.0, 50.0),
                is_left: true,
                box_index: 0,
            },
        ];

        let clearance = calculate_clearance(&floats, Clear::Left, 0.0);
        assert_eq!(clearance, 50.0);

        let clearance = calculate_clearance(&floats, Clear::Right, 0.0);
        assert_eq!(clearance, 0.0);
    }
}
