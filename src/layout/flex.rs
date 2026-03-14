//! Flexbox Layout
//!
//! Implements CSS Flexible Box Layout Module Level 1
//! Supports: flex-direction, justify-content, align-items, flex-wrap

use crate::types::{Rect, Length};
use crate::layout::box_model::{
    LayoutBox,
    calculate_width,
};
use crate::layout::style::ComputedStyle;
use crate::layout::flow::BlockFormattingContext;

/// Flex direction
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum FlexDirection {
    #[default]
    Row,
    RowReverse,
    Column,
    ColumnReverse,
}

/// Flex wrap mode
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum FlexWrap {
    #[default]
    Nowrap,
    Wrap,
    WrapReverse,
}

/// Justify content (main axis alignment)
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum JustifyContent {
    #[default]
    FlexStart,
    FlexEnd,
    Center,
    SpaceBetween,
    SpaceAround,
    SpaceEvenly,
}

/// Align items (cross axis alignment for flex items)
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum AlignItems {
    #[default]
    Stretch,
    FlexStart,
    FlexEnd,
    Center,
    Baseline,
}

/// Align content (multi-line alignment)
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum AlignContent {
    #[default]
    Stretch,
    FlexStart,
    FlexEnd,
    Center,
    SpaceBetween,
    SpaceAround,
    SpaceEvenly,
}

/// Flex container properties
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FlexContainer {
    /// Flex direction
    pub direction: FlexDirection,
    /// Flex wrap mode
    pub wrap: FlexWrap,
    /// Justify content (main axis)
    pub justify_content: JustifyContent,
    /// Align items (cross axis for items)
    pub align_items: AlignItems,
    /// Align content (cross axis for multi-line)
    pub align_content: AlignContent,
    /// Main axis is horizontal (row)
    pub is_horizontal: bool,
    /// Is reversed direction
    pub is_reversed: bool,
}

impl Default for FlexContainer {
    fn default() -> Self {
        Self {
            direction: FlexDirection::Row,
            wrap: FlexWrap::Nowrap,
            justify_content: JustifyContent::FlexStart,
            align_items: AlignItems::Stretch,
            align_content: AlignContent::Stretch,
            is_horizontal: true,
            is_reversed: false,
        }
    }
}

impl FlexContainer {
    /// Create from computed style
    pub fn from_style(_style: &ComputedStyle) -> Self {
        let mut container = Self::default();
        
        // Parse flex-direction (would come from style in full implementation)
        container.direction = FlexDirection::Row; // Default
        
        // Parse flex-wrap
        container.wrap = FlexWrap::Nowrap; // Default
        
        // Parse justify-content
        container.justify_content = JustifyContent::FlexStart; // Default
        
        // Parse align-items
        container.align_items = AlignItems::Stretch; // Default
        
        // Update orientation flags
        container.is_horizontal = matches!(
            container.direction,
            FlexDirection::Row | FlexDirection::RowReverse
        );
        container.is_reversed = matches!(
            container.direction,
            FlexDirection::RowReverse | FlexDirection::ColumnReverse
        );
        
        container
    }

    /// Get the main size (width for row, height for column)
    pub fn main_size(&self, size: crate::types::Size) -> f32 {
        if self.is_horizontal {
            size.width
        } else {
            size.height
        }
    }

    /// Get the cross size (height for row, width for column)
    pub fn cross_size(&self, size: crate::types::Size) -> f32 {
        if self.is_horizontal {
            size.height
        } else {
            size.width
        }
    }

    /// Get the main position (x for row, y for column)
    pub fn main_pos(&self, rect: Rect) -> f32 {
        if self.is_horizontal {
            rect.x
        } else {
            rect.y
        }
    }

    /// Get the cross position (y for row, x for column)
    pub fn cross_pos(&self, rect: Rect) -> f32 {
        if self.is_horizontal {
            rect.y
        } else {
            rect.x
        }
    }
}

/// Flex item properties
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct FlexItem {
    /// Flex grow factor
    pub flex_grow: f32,
    /// Flex shrink factor
    pub flex_shrink: f32,
    /// Flex basis (initial main size)
    pub flex_basis: FlexBasis,
    /// Align self (overrides align-items)
    pub align_self: Option<AlignItems>,
    /// Min main size constraint
    pub min_main_size: f32,
    /// Max main size constraint
    pub max_main_size: f32,
    /// Min cross size constraint
    pub min_cross_size: f32,
    /// Max cross size constraint
    pub max_cross_size: f32,
}

/// Flex basis value
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FlexBasis {
    Auto,
    Content,
    Length(f32),
}

impl Default for FlexBasis {
    fn default() -> Self {
        FlexBasis::Auto
    }
}

/// A line of flex items (for multi-line flex containers)
#[derive(Debug, Clone, Default)]
pub struct FlexLine {
    /// Items in this line
    pub items: Vec<usize>,
    /// Total flex grow of items in this line
    pub total_flex_grow: f32,
    /// Total flex shrink of items in this line
    pub total_flex_shrink: f32,
    /// Main size used by items (before growing/shrinking)
    pub used_main_size: f32,
    /// Cross size of this line
    pub cross_size: f32,
}

/// Flex layout context
#[derive(Debug)]
pub struct FlexContext {
    /// Container properties
    pub container: FlexContainer,
    /// Available main size
    pub available_main_size: f32,
    /// Available cross size
    pub available_cross_size: f32,
    /// Flex lines
    pub lines: Vec<FlexLine>,
}

impl FlexContext {
    pub fn new(container: FlexContainer, available_size: crate::types::Size) -> Self {
        Self {
            available_main_size: container.main_size(available_size),
            available_cross_size: container.cross_size(available_size),
            container,
            lines: Vec::new(),
        }
    }
}

/// Layout a flex container and its items
pub fn layout_flex_container(
    box_: &mut LayoutBox,
    bfc: &mut BlockFormattingContext,
    style_resolver: &dyn Fn(&crate::html::Element) -> ComputedStyle,
    base_font_size: f32,
) {
    let style = box_.element()
        .map(|el| style_resolver(el))
        .unwrap_or_default();

    let container = FlexContainer::from_style(&style);
    
    // Get container dimensions
    let content_width = box_.dimensions.content.width;
    let content_height = box_.dimensions.content.height;
    
    let available_size = crate::types::Size::new(content_width, content_height);
    let _context = FlexContext::new(container, available_size);

    // Collect flex items
    let flex_items: Vec<(usize, FlexItem)> = box_.children
        .iter()
        .enumerate()
        .map(|(i, child)| {
            let item_style = child.element()
                .map(|el| style_resolver(el))
                .unwrap_or_default();
            (i, create_flex_item(&item_style, base_font_size))
        })
        .collect();

    if flex_items.is_empty() {
        box_.is_laid_out = true;
        return;
    }

    // Perform flex layout algorithm
    // 1. Determine available main and cross size
    let container_main_size = if container.is_horizontal {
        content_width
    } else {
        content_height
    };
    
    let container_cross_size = if container.is_horizontal {
        content_height
    } else {
        content_width
    };

    // 2. Calculate flex base size and hypothetical main size for each item
    let mut item_main_sizes: Vec<f32> = Vec::with_capacity(flex_items.len());
    
    for (child_index, item) in &flex_items {
        let child = &box_.children[*child_index];
        let base_size = calculate_flex_base_size(
            child,
            item,
            container_main_size,
            container.is_horizontal,
            style_resolver,
            base_font_size,
        );
        item_main_sizes.push(base_size);
    }

    // 3. Collect flex items into flex lines
    let lines = collect_flex_lines(
        &flex_items,
        &item_main_sizes,
        container_main_size,
        container.wrap,
    );

    // 4. Resolve flexible lengths (grow/shrink items to fill available space)
    let resolved_main_sizes = resolve_flexible_lengths(
        &lines,
        &flex_items,
        &item_main_sizes,
        container_main_size,
    );

    // 5. Determine cross size of each flex line
    let line_cross_sizes = calculate_line_cross_sizes(
        &lines,
        &flex_items,
        &resolved_main_sizes,
        container_cross_size,
        container.is_horizontal,
        box_,
        style_resolver,
        base_font_size,
    );

    // 6. Position items within lines
    position_flex_items(
        box_,
        &lines,
        &flex_items,
        &resolved_main_sizes,
        &line_cross_sizes,
        &container,
        container_main_size,
        container_cross_size,
        bfc,
        style_resolver,
        base_font_size,
    );

    // 7. Calculate final container size
    let total_main_size: f32 = resolved_main_sizes.iter().sum();
    let total_cross_size: f32 = line_cross_sizes.iter().sum();
    
    if container.is_horizontal {
        box_.dimensions.content.width = total_main_size;
        box_.dimensions.content.height = total_cross_size;
    } else {
        box_.dimensions.content.width = total_cross_size;
        box_.dimensions.content.height = total_main_size;
    }

    box_.is_laid_out = true;
}

/// Create a FlexItem from computed style
fn create_flex_item(style: &ComputedStyle, _base_font_size: f32) -> FlexItem {
    FlexItem {
        flex_grow: 0.0,  // Would parse from flex property
        flex_shrink: 1.0, // Default shrink
        flex_basis: FlexBasis::Auto,
        align_self: None,
        min_main_size: 0.0,
        max_main_size: f32::INFINITY,
        min_cross_size: 0.0,
        max_cross_size: f32::INFINITY,
    }
}

/// Calculate the flex base size for an item
fn calculate_flex_base_size(
    child: &LayoutBox,
    item: &FlexItem,
    container_main_size: f32,
    is_horizontal: bool,
    style_resolver: &dyn Fn(&crate::html::Element) -> ComputedStyle,
    base_font_size: f32,
) -> f32 {
    // Use flex-basis if specified
    match item.flex_basis {
        FlexBasis::Length(size) => size,
        FlexBasis::Auto | FlexBasis::Content => {
            // Calculate based on content
            let _style = child.element()
                .map(|el| style_resolver(el))
                .unwrap_or_default();
            
            if is_horizontal {
                // Calculate preferred width
                let width = style.width.to_pt_with_container(base_font_size, container_main_size);
                if width > 0.0 {
                    width
                } else {
                    // Estimate from content
                    estimate_content_width(child, base_font_size)
                }
            } else {
                // Calculate preferred height
                let height = style.height.to_pt_with_container(base_font_size, container_main_size);
                if height > 0.0 {
                    height
                } else {
                    estimate_content_height(child, base_font_size)
                }
            }
        }
    }
}

/// Estimate content width from children
fn estimate_content_width(child: &LayoutBox, _base_font_size: f32) -> f32 {
    // Sum children's widths for inline, max for block
    if child.children.is_empty() {
        // Estimate from text content
        child.text_content.as_ref()
            .map(|text| text.len() as f32 * 6.0) // Approximate char width
            .unwrap_or(50.0)
    } else {
        child.children.iter()
            .map(|c| c.dimensions.content.width)
            .fold(0.0, f32::max)
    }
}

/// Estimate content height from children
fn estimate_content_height(child: &LayoutBox, _base_font_size: f32) -> f32 {
    if child.children.is_empty() {
        20.0 // Default line height approximation
    } else {
        child.children.iter()
            .map(|c| c.dimensions.content.height)
            .sum()
    }
}

/// Collect flex items into lines based on wrapping
fn collect_flex_lines(
    items: &[(usize, FlexItem)],
    item_main_sizes: &[f32],
    container_main_size: f32,
    wrap: FlexWrap,
) -> Vec<FlexLine> {
    if items.is_empty() {
        return Vec::new();
    }

    let mut lines: Vec<FlexLine> = Vec::new();
    let mut current_line = FlexLine::default();
    let mut current_line_size: f32 = 0.0;

    for (_i, ((child_index, item), main_size)) in items.iter().zip(item_main_sizes.iter()).enumerate() {
        let item_size = *main_size;
        
        // Check if we need to wrap
        let needs_wrap = match wrap {
            FlexWrap::Nowrap => false,
            FlexWrap::Wrap => current_line_size + item_size > container_main_size && !current_line.items.is_empty(),
            FlexWrap::WrapReverse => current_line_size + item_size > container_main_size && !current_line.items.is_empty(),
        };

        if needs_wrap {
            // Finish current line
            if !current_line.items.is_empty() {
                lines.push(current_line);
                current_line = FlexLine::default();
                current_line_size = 0.0;
            }
        }

        // Add item to current line
        current_line.items.push(*child_index);
        current_line.total_flex_grow += item.flex_grow;
        current_line.total_flex_shrink += item.flex_shrink;
        current_line.used_main_size += item_size;
        current_line_size += item_size;
    }

    // Don't forget the last line
    if !current_line.items.is_empty() {
        lines.push(current_line);
    }

    lines
}

/// Resolve flexible lengths (grow/shrink items)
fn resolve_flexible_lengths(
    lines: &[FlexLine],
    items: &[(usize, FlexItem)],
    item_main_sizes: &[f32],
    container_main_size: f32,
) -> Vec<f32> {
    let mut resolved_sizes = item_main_sizes.to_vec();

    for line in lines {
        let available_space = container_main_size - line.used_main_size;
        
        if available_space > 0.0 && line.total_flex_grow > 0.0 {
            // Grow items
            let grow_factor = available_space / line.total_flex_grow;
            
            for &child_index in &line.items {
                if let Some((_, item)) = items.iter().find(|(i, _)| *i == child_index) {
                    if let Some(size) = resolved_sizes.get_mut(child_index) {
                        *size += grow_factor * item.flex_grow;
                    }
                }
            }
        } else if available_space < 0.0 && line.total_flex_shrink > 0.0 {
            // Shrink items
            let shrink_factor = (-available_space) / line.total_flex_shrink;
            
            for &child_index in &line.items {
                if let Some((_, item)) = items.iter().find(|(i, _)| *i == child_index) {
                    if let Some(size) = resolved_sizes.get_mut(child_index) {
                        *size = (*size - shrink_factor * item.flex_shrink).max(0.0);
                    }
                }
            }
        }
    }

    resolved_sizes
}

/// Calculate cross size for each flex line
fn calculate_line_cross_sizes(
    lines: &[FlexLine],
    items: &[(usize, FlexItem)],
    resolved_main_sizes: &[f32],
    container_cross_size: f32,
    is_horizontal: bool,
    box_: &LayoutBox,
    style_resolver: &dyn Fn(&crate::html::Element) -> ComputedStyle,
    base_font_size: f32,
) -> Vec<f32> {
    let mut line_cross_sizes = Vec::with_capacity(lines.len());

    for line in lines {
        let mut max_cross_size: f32 = 0.0;

        for &child_index in &line.items {
            if let Some(child) = box_.children.get(child_index) {
                let style = child.element()
                    .map(|el| style_resolver(el))
                    .unwrap_or_default();

                // Estimate cross size based on content
                let cross_size = if is_horizontal {
                    // Height for row
                    let specified_height = style.height.to_pt(base_font_size);
                    if specified_height > 0.0 {
                        specified_height
                    } else {
                        estimate_content_height(child, base_font_size)
                    }
                } else {
                    // Width for column
                    let specified_width = style.width.to_pt(base_font_size);
                    if specified_width > 0.0 {
                        specified_width
                    } else {
                        estimate_content_width(child, base_font_size)
                    }
                };

                max_cross_size = max_cross_size.max(cross_size);
            }
        }

        line_cross_sizes.push(max_cross_size);
    }

    // If container has fixed cross size and align-content is stretch
    if container_cross_size > 0.0 {
        let total_cross: f32 = line_cross_sizes.iter().sum();
        if total_cross < container_cross_size {
            // Stretch lines to fill container
            let extra_per_line = (container_cross_size - total_cross) / lines.len() as f32;
            for size in &mut line_cross_sizes {
                *size += extra_per_line;
            }
        }
    }

    line_cross_sizes
}

/// Position flex items within the container
fn position_flex_items(
    box_: &mut LayoutBox,
    lines: &[FlexLine],
    items: &[(usize, FlexItem)],
    resolved_main_sizes: &[f32],
    line_cross_sizes: &[f32],
    container: &FlexContainer,
    container_main_size: f32,
    _container_cross_size: f32,
    bfc: &mut BlockFormattingContext,
    style_resolver: &dyn Fn(&crate::html::Element) -> ComputedStyle,
    base_font_size: f32,
) {
    let _line_main_start: f32 = 0.0;
    let mut line_cross_start: f32 = 0.0;

    // Reverse line order if wrap-reverse
    let line_iter: Box<dyn Iterator<Item = (usize, &FlexLine)>> = if container.wrap == FlexWrap::WrapReverse {
        Box::new(lines.iter().enumerate().rev())
    } else {
        Box::new(lines.iter().enumerate())
    };

    for (line_idx, line) in line_iter {
        let line_cross_size = line_cross_sizes[line_idx];
        
        // Calculate main-axis distribution
        let (main_start_offset, gap) = calculate_main_axis_distribution(
            line.used_main_size,
            container_main_size,
            container.justify_content,
            line.items.len(),
        );

        let mut current_main: f32 = main_start_offset;

        // Position items in this line
        for &child_index in &line.items {
            if let Some(child) = box_.children.get_mut(child_index) {
                let main_size = resolved_main_sizes.get(child_index).copied().unwrap_or(50.0);
                
                // Get item's align-self or container's align-items
                let item = items.iter().find(|(i, _)| *i == child_index).map(|(_, item)| item);
                let align = item.and_then(|i| i.align_self).unwrap_or(container.align_items);
                
                // Calculate cross position based on alignment
                let cross_pos = calculate_cross_position(
                    line_cross_size,
                    child.dimensions.content.height,
                    align,
                );

                // Set position
                if container.is_horizontal {
                    // Row layout
                    let x = if container.is_reversed {
                        container_main_size - current_main - main_size
                    } else {
                        current_main
                    };
                    let y = line_cross_start + cross_pos;
                    
                    child.dimensions.content.x = bfc.containing_block.x + x;
                    child.dimensions.content.y = bfc.containing_block.y + y;
                    child.dimensions.content.width = main_size;
                    
                    // Layout child content
                    layout_child_content(
                        child,
                        main_size,
                        line_cross_size,
                        container.is_horizontal,
                        style_resolver,
                        base_font_size,
                    );
                } else {
                    // Column layout
                    let x = line_cross_start + cross_pos;
                    let y = if container.is_reversed {
                        container_main_size - current_main - main_size
                    } else {
                        current_main
                    };
                    
                    child.dimensions.content.x = bfc.containing_block.x + x;
                    child.dimensions.content.y = bfc.containing_block.y + y;
                    child.dimensions.content.height = main_size;
                    
                    // Layout child content
                    layout_child_content(
                        child,
                        line_cross_size,
                        main_size,
                        container.is_horizontal,
                        style_resolver,
                        base_font_size,
                    );
                }

                child.is_laid_out = true;
                current_main += main_size + gap;
            }
        }

        line_cross_start += line_cross_size;
    }
}

/// Calculate main-axis distribution (justify-content)
fn calculate_main_axis_distribution(
    used_main_size: f32,
    container_main_size: f32,
    justify_content: JustifyContent,
    item_count: usize,
) -> (f32, f32) {
    let remaining_space = container_main_size - used_main_size;
    
    match justify_content {
        JustifyContent::FlexStart => (0.0, 0.0),
        JustifyContent::FlexEnd => (remaining_space.max(0.0), 0.0),
        JustifyContent::Center => (remaining_space.max(0.0) / 2.0, 0.0),
        JustifyContent::SpaceBetween => {
            if item_count <= 1 {
                (0.0, 0.0)
            } else {
                let gap = remaining_space.max(0.0) / (item_count - 1) as f32;
                (0.0, gap)
            }
        }
        JustifyContent::SpaceAround => {
            let gap = if item_count > 0 {
                remaining_space.max(0.0) / item_count as f32
            } else {
                0.0
            };
            (gap / 2.0, gap)
        }
        JustifyContent::SpaceEvenly => {
            let gap = if item_count > 0 {
                remaining_space.max(0.0) / (item_count + 1) as f32
            } else {
                0.0
            };
            (gap, gap)
        }
    }
}

/// Calculate cross-axis position based on alignment
fn calculate_cross_position(
    line_cross_size: f32,
    item_cross_size: f32,
    align: AlignItems,
) -> f32 {
    match align {
        AlignItems::FlexStart => 0.0,
        AlignItems::FlexEnd => (line_cross_size - item_cross_size).max(0.0),
        AlignItems::Center => ((line_cross_size - item_cross_size) / 2.0).max(0.0),
        AlignItems::Stretch => 0.0, // Item will be stretched to fill
        AlignItems::Baseline => 0.0, // Simplified - would need baseline calculation
    }
}

/// Layout child content within flex item
fn layout_child_content(
    child: &mut LayoutBox,
    main_size: f32,
    cross_size: f32,
    is_horizontal: bool,
    style_resolver: &dyn Fn(&crate::html::Element) -> ComputedStyle,
    base_font_size: f32,
) {
    let style = child.element()
        .map(|el| style_resolver(el))
        .unwrap_or_default();

    // Set dimensions
    if is_horizontal {
        child.dimensions.content.height = cross_size;
    } else {
        child.dimensions.content.width = main_size;
    }

    // Layout children if any
    if !child.children.is_empty() {
        let containing_block = Rect::new(
            child.dimensions.content.x,
            child.dimensions.content.y,
            child.dimensions.content.width,
            child.dimensions.content.height,
        );
        let mut child_bfc = BlockFormattingContext::new(containing_block);
        
        // Recursively layout children
        for child_box in &mut child.children {
            layout_child_box(child_box, &mut child_bfc, style_resolver, base_font_size);
        }
    }
}

/// Layout a single child box (simplified)
fn layout_child_box(
    box_: &mut LayoutBox,
    bfc: &mut BlockFormattingContext,
    style_resolver: &dyn Fn(&crate::html::Element) -> ComputedStyle,
    base_font_size: f32,
) {
    let style = box_.element()
        .map(|el| style_resolver(el))
        .unwrap_or_default();

    // Calculate width
    calculate_width(
        box_,
        bfc.available_width(),
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

    box_.dimensions.content.x = bfc.containing_block.x + box_.dimensions.margin.left;
    box_.dimensions.content.y = bfc.current_y + box_.dimensions.margin.top;

    // Simple height calculation
    let content_height = if box_.children.is_empty() {
        style.font_size.to_pt(base_font_size)
    } else {
        box_.children.iter().map(|c| c.dimensions.content.height).sum()
    };

    box_.dimensions.content.height = content_height;
    box_.is_laid_out = true;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_flex_container_defaults() {
        let container = FlexContainer::default();
        assert_eq!(container.direction, FlexDirection::Row);
        assert_eq!(container.wrap, FlexWrap::Nowrap);
        assert_eq!(container.justify_content, JustifyContent::FlexStart);
        assert_eq!(container.align_items, AlignItems::Stretch);
        assert!(container.is_horizontal);
        assert!(!container.is_reversed);
    }

    #[test]
    fn test_flex_direction_column() {
        let container = FlexContainer {
            direction: FlexDirection::Column,
            ..Default::default()
        };
        assert!(!container.is_horizontal);
        assert!(!container.is_reversed);
    }

    #[test]
    fn test_justify_content_calculations() {
        // FlexStart
        let (offset, gap) = calculate_main_axis_distribution(300.0, 500.0, JustifyContent::FlexStart, 3);
        assert_eq!(offset, 0.0);
        assert_eq!(gap, 0.0);

        // FlexEnd
        let (offset, gap) = calculate_main_axis_distribution(300.0, 500.0, JustifyContent::FlexEnd, 3);
        assert_eq!(offset, 200.0);
        assert_eq!(gap, 0.0);

        // Center
        let (offset, gap) = calculate_main_axis_distribution(300.0, 500.0, JustifyContent::Center, 3);
        assert_eq!(offset, 100.0);
        assert_eq!(gap, 0.0);

        // SpaceBetween
        let (offset, gap) = calculate_main_axis_distribution(300.0, 500.0, JustifyContent::SpaceBetween, 3);
        assert_eq!(offset, 0.0);
        assert_eq!(gap, 100.0);
    }

    #[test]
    fn test_cross_alignments() {
        let line_size = 100.0;
        let item_size = 50.0;

        assert_eq!(calculate_cross_position(line_size, item_size, AlignItems::FlexStart), 0.0);
        assert_eq!(calculate_cross_position(line_size, item_size, AlignItems::FlexEnd), 50.0);
        assert_eq!(calculate_cross_position(line_size, item_size, AlignItems::Center), 25.0);
        assert_eq!(calculate_cross_position(line_size, item_size, AlignItems::Stretch), 0.0);
    }

    #[test]
    fn test_flex_line_collection() {
        let items = vec![
            (0, FlexItem { flex_grow: 0.0, flex_shrink: 1.0, ..Default::default() }),
            (1, FlexItem { flex_grow: 0.0, flex_shrink: 1.0, ..Default::default() }),
            (2, FlexItem { flex_grow: 0.0, flex_shrink: 1.0, ..Default::default() }),
        ];
        let sizes = vec![100.0, 100.0, 100.0];

        // No wrapping
        let lines = collect_flex_lines(&items, &sizes, 500.0, FlexWrap::Nowrap);
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0].items.len(), 3);

        // With wrapping
        let lines = collect_flex_lines(&items, &sizes, 150.0, FlexWrap::Wrap);
        assert!(lines.len() > 1);
    }

    #[test]
    fn test_flexible_length_resolution() {
        let lines = vec![
            FlexLine {
                items: vec![0, 1],
                total_flex_grow: 2.0,
                total_flex_shrink: 2.0,
                used_main_size: 100.0,
                cross_size: 0.0,
            }
        ];
        let items = vec![
            (0, FlexItem { flex_grow: 1.0, flex_shrink: 1.0, ..Default::default() }),
            (1, FlexItem { flex_grow: 1.0, flex_shrink: 1.0, ..Default::default() }),
        ];
        let sizes = vec![50.0, 50.0];

        // Grow to fill 200px container
        let resolved = resolve_flexible_lengths(&lines, &items, &sizes, 200.0);
        assert_eq!(resolved.len(), 2);
        // Each item should grow by 50px (100px remaining / 2 grow factor each)
        assert!(resolved[0] > 50.0);
        assert!(resolved[1] > 50.0);
    }
}
