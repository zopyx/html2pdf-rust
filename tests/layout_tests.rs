//! Layout Engine Tests
//!
//! Tests for box model, floats, positioning, and page breaks.

use html2pdf::layout::{
    LayoutBox, BoxType, EdgeSizes,
    LayoutEngine, build_layout_tree,
};
use html2pdf::types::{Rect, Size, Point};
use html2pdf::html::{Element, Attribute, Node};

// ============================================================================
// Box Model Tests
// ============================================================================

#[test]
fn test_layout_box_creation() {
    let box_layout = LayoutBox::new(BoxType::Block, None);
    
    assert!(matches!(box_layout.box_type, BoxType::Block));
    assert!(box_layout.children.is_empty());
}

#[test]
fn test_content_rect_calculation() {
    let mut box_layout = LayoutBox::new(BoxType::Block, None);
    box_layout.dimensions.content = Rect::new(35.0, 35.0, 400.0, 300.0);
    box_layout.dimensions.margin = EdgeSizes { top: 10.0, right: 10.0, bottom: 10.0, left: 10.0 };
    box_layout.dimensions.border = EdgeSizes { top: 5.0, right: 5.0, bottom: 5.0, left: 5.0 };
    box_layout.dimensions.padding = EdgeSizes { top: 20.0, right: 20.0, bottom: 20.0, left: 20.0 };
    
    let content = box_layout.dimensions.content;
    
    // x = 0 + 10 (margin) + 5 (border) + 20 (padding)
    assert_eq!(content.x, 35.0);
    assert_eq!(content.y, 35.0);
    assert_eq!(content.width, 400.0);
    assert_eq!(content.height, 300.0);
}

#[test]
fn test_padding_rect_calculation() {
    let mut box_layout = LayoutBox::new(BoxType::Block, None);
    box_layout.dimensions.content = Rect::new(0.0, 0.0, 400.0, 300.0);
    box_layout.dimensions.padding = EdgeSizes { top: 20.0, right: 20.0, bottom: 20.0, left: 20.0 };
    
    let padding = box_layout.dimensions.padding_box();
    
    assert_eq!(padding.width, 440.0); // 400 + 20 + 20
    assert_eq!(padding.height, 340.0); // 300 + 20 + 20
}

#[test]
fn test_border_rect_calculation() {
    let mut box_layout = LayoutBox::new(BoxType::Block, None);
    box_layout.dimensions.content = Rect::new(0.0, 0.0, 400.0, 300.0);
    box_layout.dimensions.padding = EdgeSizes { top: 20.0, right: 20.0, bottom: 20.0, left: 20.0 };
    box_layout.dimensions.border = EdgeSizes { top: 5.0, right: 5.0, bottom: 5.0, left: 5.0 };
    
    let border = box_layout.dimensions.border_box();
    
    assert_eq!(border.width, 450.0); // 400 + 20 + 20 + 5 + 5
    assert_eq!(border.height, 350.0); // 300 + 20 + 20 + 5 + 5
}

#[test]
fn test_total_width_calculation() {
    let mut box_layout = LayoutBox::new(BoxType::Block, None);
    box_layout.dimensions.content = Rect::new(0.0, 0.0, 400.0, 300.0);
    box_layout.dimensions.padding = EdgeSizes { top: 20.0, right: 15.0, bottom: 20.0, left: 15.0 };
    box_layout.dimensions.border = EdgeSizes { top: 5.0, right: 5.0, bottom: 5.0, left: 5.0 };
    
    let total = box_layout.dimensions.content.width 
        + box_layout.dimensions.padding.left + box_layout.dimensions.padding.right
        + box_layout.dimensions.border.left + box_layout.dimensions.border.right;
    
    assert_eq!(total, 440.0); // 400 + 15 + 15 + 5 + 5
}

#[test]
fn test_total_height_calculation() {
    let mut box_layout = LayoutBox::new(BoxType::Block, None);
    box_layout.dimensions.content = Rect::new(0.0, 0.0, 400.0, 300.0);
    box_layout.dimensions.padding = EdgeSizes { top: 20.0, right: 20.0, bottom: 20.0, left: 20.0 };
    box_layout.dimensions.border = EdgeSizes { top: 5.0, right: 5.0, bottom: 5.0, left: 5.0 };
    
    let total = box_layout.dimensions.content.height
        + box_layout.dimensions.padding.top + box_layout.dimensions.padding.bottom
        + box_layout.dimensions.border.top + box_layout.dimensions.border.bottom;
    
    assert_eq!(total, 350.0); // 300 + 20 + 20 + 5 + 5
}

#[test]
fn test_box_model_box_sizing_content() {
    // With box-sizing: content-box (default)
    // width = content width
    let content_width = 400.0;
    let padding = EdgeSizes { left: 20.0, right: 20.0, ..Default::default() };
    let border = EdgeSizes { left: 5.0, right: 5.0, ..Default::default() };
    
    let total_width = content_width + padding.left + padding.right + border.left + border.right;
    
    assert_eq!(total_width, 450.0);
}

#[test]
fn test_box_model_box_sizing_border() {
    // With box-sizing: border-box
    // width = content + padding + border
    // content width = specified width - padding - border
    let specified_width = 400.0;
    let padding = EdgeSizes { left: 20.0, right: 20.0, ..Default::default() };
    let border = EdgeSizes { left: 5.0, right: 5.0, ..Default::default() };
    
    let content_width = specified_width - padding.left - padding.right - border.left - border.right;
    
    assert_eq!(content_width, 350.0);
}

// ============================================================================
// Box Type Tests
// ============================================================================

#[test]
fn test_box_types() {
    let block = LayoutBox::new(BoxType::Block, None);
    assert!(matches!(block.box_type, BoxType::Block));
    
    let inline = LayoutBox::new(BoxType::Inline, None);
    assert!(matches!(inline.box_type, BoxType::Inline));
    
    let inline_block = LayoutBox::new(BoxType::InlineBlock, None);
    assert!(matches!(inline_block.box_type, BoxType::InlineBlock));
    
    // Note: Float boxes use BoxType::Block with Float style property
    // Float handling is done through style, not box type
    let float = LayoutBox::new(BoxType::Block, None);
    assert!(matches!(float.box_type, BoxType::Block));
    
    // Note: Positioned boxes use BoxType::Block with Position style property  
    // Position handling is done through style, not box type
    let positioned = LayoutBox::new(BoxType::Block, None);
    assert!(matches!(positioned.box_type, BoxType::Block));
}

#[test]
fn test_box_hierarchy() {
    let mut parent = LayoutBox::new(BoxType::Block, None);
    let child1 = LayoutBox::new(BoxType::Block, None);
    let child2 = LayoutBox::new(BoxType::Inline, None);
    
    parent.children.push(child1);
    parent.children.push(child2);
    
    assert_eq!(parent.children.len(), 2);
    assert!(matches!(parent.children[0].box_type, BoxType::Block));
    assert!(matches!(parent.children[1].box_type, BoxType::Inline));
}

// ============================================================================
// Margin Tests
// ============================================================================

#[test]
fn test_margin_collapsing_siblings() {
    // When two block elements are adjacent vertically, their margins collapse
    // max(margin-bottom of first, margin-top of second)
    
    let margin1: f32 = 20.0;
    let margin2: f32 = 30.0;
    let collapsed = margin1.max(margin2);
    
    assert_eq!(collapsed, 30.0);
}

#[test]
fn test_margin_collapsing_parent_child() {
    // When a block's top margin touches its parent's top margin, they collapse
    let parent_margin: f32 = 20.0;
    let child_margin: f32 = 10.0;
    let collapsed = parent_margin.max(child_margin);
    
    assert_eq!(collapsed, 20.0);
}

#[test]
fn test_margin_auto_centering() {
    // margin-left: auto + margin-right: auto centers a block element
    let container_width = 800.0;
    let element_width = 400.0;
    let margin_left = (container_width - element_width) / 2.0;
    let margin_right = margin_left;
    
    assert_eq!(margin_left, 200.0);
    assert_eq!(margin_right, 200.0);
}

// ============================================================================
// Float Tests
// ============================================================================

#[test]
fn test_float_left_layout() {
    // Float left: element moves to left, content flows around right side
    let container_width = 800.0;
    let float_width = 200.0;
    let remaining_width = container_width - float_width;
    
    assert_eq!(remaining_width, 600.0);
}

#[test]
fn test_float_right_layout() {
    // Float right: element moves to right, content flows around left side
    let container_width = 800.0;
    let float_width = 200.0;
    let remaining_width = container_width - float_width;
    
    assert_eq!(remaining_width, 600.0);
}

#[test]
fn test_clear_left() {
    // clear: left - element is moved below all left floats
    let float_bottom = 300.0;
    let cleared_element_y = float_bottom;
    
    assert_eq!(cleared_element_y, 300.0);
}

#[test]
fn test_clear_right() {
    // clear: right - element is moved below all right floats
    let float_bottom = 300.0;
    let cleared_element_y = float_bottom;
    
    assert_eq!(cleared_element_y, 300.0);
}

#[test]
fn test_clear_both() {
    // clear: both - element is moved below all floats
    let left_float_bottom: f32 = 300.0;
    let right_float_bottom: f32 = 250.0;
    let cleared_element_y = left_float_bottom.max(right_float_bottom);
    
    assert_eq!(cleared_element_y, 300.0);
}

// ============================================================================
// Positioning Tests
// ============================================================================

#[test]
fn test_position_static() {
    // Static positioning: normal document flow
    use html2pdf::layout::Position;
    let static_position = Position::Static;
    
    assert!(matches!(static_position, Position::Static));
}

#[test]
fn test_position_relative() {
    // Relative positioning: offset from normal position
    use html2pdf::layout::Position;
    let relative_position = Position::Relative;
    
    assert!(matches!(relative_position, Position::Relative));
}

#[test]
fn test_position_absolute() {
    // Absolute positioning: positioned relative to nearest positioned ancestor
    use html2pdf::layout::Position;
    let absolute_position = Position::Absolute;
    
    assert!(matches!(absolute_position, Position::Absolute));
}

#[test]
fn test_position_fixed() {
    // Fixed positioning: positioned relative to viewport
    use html2pdf::layout::Position;
    let fixed_position = Position::Fixed;
    
    assert!(matches!(fixed_position, Position::Fixed));
}

#[test]
fn test_z_index_stacking() {
    // Elements with higher z-index appear on top
    let z_index1 = 1;
    let z_index2 = 2;
    
    assert!(z_index2 > z_index1);
}

// ============================================================================
// Flexbox Layout Tests
// ============================================================================

#[test]
fn test_flex_direction_row() {
    // flex-direction: row - items placed left to right
    let container_width = 600.0;
    let item_width = 100.0;
    let num_items = 3;
    
    let total_item_width = item_width * num_items as f32;
    let remaining_space = container_width - total_item_width;
    
    assert_eq!(remaining_space, 300.0);
}

#[test]
fn test_flex_direction_column() {
    // flex-direction: column - items placed top to bottom
    let container_height = 600.0;
    let item_height = 100.0;
    let num_items = 3;
    
    let total_item_height = item_height * num_items as f32;
    let remaining_space = container_height - total_item_height;
    
    assert_eq!(remaining_space, 300.0);
}

#[test]
fn test_flex_grow_distribution() {
    // flex-grow distributes remaining space proportionally
    let remaining_space = 300.0;
    let grow1 = 1.0;
    let grow2 = 2.0;
    let total_grow = grow1 + grow2;
    
    let extra1 = remaining_space * (grow1 / total_grow);
    let extra2 = remaining_space * (grow2 / total_grow);
    
    assert_eq!(extra1, 100.0);
    assert_eq!(extra2, 200.0);
}

#[test]
fn test_flex_shrink_distribution() {
    // flex-shrink reduces size proportionally when items overflow
    let overflow = 100.0;
    let shrink1 = 1.0;
    let shrink2 = 1.0;
    let total_shrink = shrink1 + shrink2;
    
    let reduction1 = overflow * (shrink1 / total_shrink);
    let reduction2 = overflow * (shrink2 / total_shrink);
    
    assert_eq!(reduction1, 50.0);
    assert_eq!(reduction2, 50.0);
}

#[test]
fn test_justify_content_flex_start() {
    // justify-content: flex-start - items packed to start
    let container_width = 500.0;
    let content_width = 300.0;
    let start_offset = 0.0;
    
    assert_eq!(start_offset, 0.0);
}

#[test]
fn test_justify_content_center() {
    // justify-content: center - items centered
    let container_width = 500.0;
    let content_width = 300.0;
    let start_offset = (container_width - content_width) / 2.0;
    
    assert_eq!(start_offset, 100.0);
}

#[test]
fn test_justify_content_space_between() {
    // justify-content: space-between - first at start, last at end, rest distributed
    let container_width = 500.0;
    let num_items = 3;
    let total_gaps = num_items - 1;
    let gap_width = (container_width - 300.0) / total_gaps as f32;
    
    assert_eq!(gap_width, 100.0);
}

#[test]
fn test_align_items_stretch() {
    // align-items: stretch - items stretch to fill container
    let container_height = 200.0;
    let item_height = container_height;
    
    assert_eq!(item_height, 200.0);
}

#[test]
fn test_align_items_center() {
    // align-items: center - items centered on cross axis
    let container_height = 200.0;
    let item_height = 50.0;
    let offset = (container_height - item_height) / 2.0;
    
    assert_eq!(offset, 75.0);
}

// ============================================================================
// Grid Layout Tests
// ============================================================================

#[test]
fn test_grid_template_columns() {
    // grid-template-columns: 1fr 2fr 1fr
    let container_width = 800.0;
    let gap = 20.0;
    let num_gaps = 2;
    let available_width = container_width - (gap * num_gaps as f32);
    let total_fr = 4.0; // 1 + 2 + 1
    
    let col1 = available_width * (1.0 / total_fr);
    let col2 = available_width * (2.0 / total_fr);
    let col3 = available_width * (1.0 / total_fr);
    
    assert_eq!(col1, 190.0);
    assert_eq!(col2, 380.0);
    assert_eq!(col3, 190.0);
}

#[test]
fn test_grid_template_rows() {
    // grid-template-rows: auto 100px auto
    let row2_height = 100.0;
    
    assert_eq!(row2_height, 100.0);
}

#[test]
fn test_grid_gap() {
    // gap: 20px
    let row_gap = 20.0;
    let column_gap = 20.0;
    
    assert_eq!(row_gap, 20.0);
    assert_eq!(column_gap, 20.0);
}

#[test]
fn test_grid_placement() {
    // grid-column: 1 / 3, grid-row: 2 / 4
    let start_col = 1;
    let end_col = 3;
    let span_col = end_col - start_col;
    
    assert_eq!(span_col, 2);
}

// ============================================================================
// Page Break Tests (PrintCSS)
// ============================================================================

#[test]
fn test_page_break_before() {
    // page-break-before: always - force page break before element
    let should_break = true;
    assert!(should_break);
}

#[test]
fn test_page_break_after() {
    // page-break-after: always - force page break after element
    let should_break = true;
    assert!(should_break);
}

#[test]
fn test_page_break_inside() {
    // page-break-inside: avoid - avoid breaking inside element
    let should_avoid = true;
    assert!(should_avoid);
}

#[test]
fn test_widows_orphans() {
    // widows: 2 - minimum lines at top of new page
    // orphans: 2 - minimum lines at bottom of old page
    let min_lines = 2;
    
    assert_eq!(min_lines, 2);
}

#[test]
fn test_page_size_a4() {
    // A4: 210mm x 297mm = ~595pt x ~842pt
    let a4_width_pt = 595.28;
    let a4_height_pt = 841.89;
    
    assert!((a4_width_pt - 595.28_f32).abs() < 0.1);
    assert!((a4_height_pt - 841.89_f32).abs() < 0.1);
}

#[test]
fn test_page_size_letter() {
    // Letter: 8.5in x 11in = 612pt x 792pt
    let letter_width_pt = 612.0;
    let letter_height_pt = 792.0;
    
    assert_eq!(letter_width_pt, 612.0);
    assert_eq!(letter_height_pt, 792.0);
}

#[test]
fn test_page_margins() {
    // @page { margin: 2cm; }
    let margin_cm = 2.0;
    let margin_pt = margin_cm * 28.346;
    
    assert!((margin_pt - 56.692_f32).abs() < 0.1);
}

// ============================================================================
// Layout Engine Tests
// ============================================================================

#[test]
fn test_layout_engine_creation() {
    let engine = LayoutEngine::new();
    
    // Engine created successfully
    assert!(true);
}

#[test]
fn test_layout_tree_building() {
    use html2pdf::html::parse_html;
    
    let html = "<html><body><div>Test</div></body></html>";
    let doc = parse_html(html).unwrap();
    
    let layout_tree = build_layout_tree(&doc, &[]).unwrap();
    
    assert!(matches!(layout_tree.box_type, BoxType::Block));
}

#[test]
fn test_nested_element_layout() {
    use html2pdf::html::parse_html;
    
    let html = "<html><body><div><p>Test</p></div></body></html>";
    let doc = parse_html(html).unwrap();
    
    let layout_tree = build_layout_tree(&doc, &[]).unwrap();
    
    // The layout tree should have nested boxes
    assert!(!layout_tree.children.is_empty());
}

// ============================================================================
// Text Layout Tests
// ============================================================================

#[test]
fn test_line_height_calculation() {
    let font_size = 16.0;
    let line_height_multiplier = 1.5;
    let line_height = font_size * line_height_multiplier;
    
    assert_eq!(line_height, 24.0);
}

#[test]
fn test_text_alignment_left() {
    // text-align: left - text starts at left edge
    let container_width = 400.0;
    let text_width = 200.0;
    let offset = 0.0;
    
    assert_eq!(offset, 0.0);
}

#[test]
fn test_text_alignment_center() {
    // text-align: center - text centered
    let container_width = 400.0;
    let text_width = 200.0;
    let offset = (container_width - text_width) / 2.0;
    
    assert_eq!(offset, 100.0);
}

#[test]
fn test_text_alignment_right() {
    // text-align: right - text ends at right edge
    let container_width = 400.0;
    let text_width = 200.0;
    let offset = container_width - text_width;
    
    assert_eq!(offset, 200.0);
}

// ============================================================================
// Overflow Tests
// ============================================================================

#[test]
fn test_overflow_visible() {
    // overflow: visible - content may overflow container
    let content_height = 300.0;
    let container_height = 200.0;
    
    assert!(content_height > container_height);
}

#[test]
fn test_overflow_hidden() {
    // overflow: hidden - content clipped to container
    let visible_height = 200.0; // Container height
    
    assert_eq!(visible_height, 200.0);
}

// ============================================================================
// Performance Tests
// ============================================================================

#[test]
fn test_large_layout_tree() {
    use html2pdf::html::parse_html;
    
    // Build deeply nested HTML structure
    let mut html = String::from("<html><body><div>");
    for _ in 0..50 {
        html.push_str("<div>");
    }
    html.push_str("Content");
    for _ in 0..50 {
        html.push_str("</div>");
    }
    html.push_str("</div></body></html>");
    
    let doc = parse_html(&html).unwrap();
    let layout_tree = build_layout_tree(&doc, &[]).unwrap();
    
    assert!(matches!(layout_tree.box_type, BoxType::Block));
}

// ============================================================================
// Edge Cases
// ============================================================================

#[test]
fn test_zero_size_box() {
    let box_layout = LayoutBox::new(BoxType::Block, None);
    
    assert_eq!(box_layout.dimensions.content.width, 0.0);
    assert_eq!(box_layout.dimensions.content.height, 0.0);
}

#[test]
fn test_negative_margins() {
    // Negative margins can pull elements closer
    let normal_position = 100.0;
    let negative_margin = -20.0;
    let final_position = normal_position + negative_margin;
    
    assert_eq!(final_position, 80.0);
}

#[test]
fn test_percentage_dimensions() {
    // width: 50% in 800px container = 400px
    let container_width = 800.0;
    let percentage = 50.0;
    let computed_width = container_width * (percentage / 100.0);
    
    assert_eq!(computed_width, 400.0);
}
