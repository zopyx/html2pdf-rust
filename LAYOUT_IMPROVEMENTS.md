# Layout Engine Improvements Summary

This document summarizes the improvements made to the html2pdf-rs layout engine for better compatibility with real-world layouts.

## Files Modified

### 1. `src/layout/flow.rs` - Block Layout Improvements

#### Margin Collapsing
- **New Type**: `MarginCollapseContext` - Tracks context for margin collapsing
- **New Type**: `MarginState` - Maintains state for proper margin collapsing between siblings
- **New Function**: `collapse_margins()` - Implements CSS margin collapsing rules:
  - Both positive: takes maximum
  - Both negative: takes minimum (most negative)
  - Mixed signs: adds them together
- **Integration**: `layout_block_children()` now properly collapses margins between adjacent block boxes

#### Float Support
- **Enhanced**: `BlockFormattingContext` now includes:
  - Float tracking (`floats: Vec<FloatBox>`)
  - Float position finding (`find_float_position()`)
  - Available width calculation accounting for floats (`available_width()`)
  - Current X position calculation with float avoidance (`current_x()`)
  - Right edge calculation accounting for right floats (`right_edge()`)
- **New Function**: `layout_float_box()` - Positions floated elements
  - Supports left and right floats
  - Finds available space for floats
  - Handles float stacking when space is limited

#### Clear Property
- **Enhanced**: `clear_floats()` method on BFC
- **New Function**: `find_clear_position()` - Determines where cleared elements should be positioned
- **Support**: Left, Right, and Both clearance values

#### Positioned Elements
- **New Function**: `layout_positioned_box()` - Handles:
  - Absolute positioning (relative to nearest positioned ancestor)
  - Fixed positioning (relative to viewport)
- **New Function**: `apply_relative_offset()` - Applies relative positioning offsets
- **Integration**: `layout_block_children()` routes positioned elements to appropriate layout paths

#### Overflow Handling
- **New Function**: `should_establish_bfc()` - Determines if element establishes new BFC
- **Integration**: BFC establishment for:
  - `overflow: hidden`
  - `float` (non-none)
  - `position: absolute/fixed`
  - `display: inline-block/flex/inline-flex`

#### Clearfix
- **New Function**: `apply_clearfix()` - Forces container to expand to contain floated children

### 2. `src/layout/text.rs` - Inline Layout Improvements

#### Enhanced Text Metrics
- **Added to `TextMetrics`**:
  - `x_height` - Height of lowercase 'x' for vertical alignment

#### Word Breaking
- **New Enum**: `WordBreak` - Controls word breaking behavior:
  - `Normal` - Standard breaking
  - `BreakAll` - Break between any characters
  - `KeepAll` - Only break at explicit opportunities
  
- **New Enum**: `OverflowWrap` - Controls overflow handling:
  - `Normal` - Overflow allowed
  - `BreakWord` - Break words when needed
  - `Anywhere` - Break at any point

#### Vertical Alignment
- **New Enum**: `VerticalAlign` - Comprehensive vertical alignment:
  - `Baseline`, `Top`, `Bottom`, `Middle`
  - `Sub`, `Super`
  - `TextTop`, `TextBottom`
  - `Length(f32)`, `Percent(f32)`
- **New Function**: `calculate_vertical_align()` - Computes vertical offset
- **Integration**: `Line::apply_vertical_align()` applies offsets to fragments

#### Text Alignment
- **Enhanced**: `Line::apply_alignment()` now supports:
  - `Left`, `Center`, `Right` - Standard alignment
  - `Justify` - Distributes extra space between words

#### Line Breaking
- **Enhanced**: `LineBreaker` now supports:
  - Word breaking configuration via `WordBreaker`
  - Better handling of long words with `break_and_add_word()`
  - CJK character breaking support

### 3. `src/layout/flex.rs` - NEW Flexbox Layout Module

#### Flex Container Properties
- **New Enum**: `FlexDirection` - `Row`, `RowReverse`, `Column`, `ColumnReverse`
- **New Enum**: `FlexWrap` - `Nowrap`, `Wrap`, `WrapReverse`
- **New Enum**: `JustifyContent` - Main axis alignment:
  - `FlexStart`, `FlexEnd`, `Center`
  - `SpaceBetween`, `SpaceAround`, `SpaceEvenly`
- **New Enum**: `AlignItems` - Cross axis item alignment:
  - `Stretch`, `FlexStart`, `FlexEnd`, `Center`, `Baseline`
- **New Enum**: `AlignContent` - Multi-line alignment
- **New Type**: `FlexContainer` - Holds all flex container properties

#### Flex Item Properties
- **New Type**: `FlexItem` - Flex item configuration:
  - `flex_grow`, `flex_shrink` - Flex factors
  - `flex_basis` - Initial main size
  - `align_self` - Item-specific cross alignment
  - Min/max size constraints
- **New Enum**: `FlexBasis` - `Auto`, `Content`, `Length(f32)`

#### Flex Layout Algorithm
- **New Type**: `FlexContext` - Layout context for flex containers
- **New Type**: `FlexLine` - Represents a line of flex items (for wrapping)
- **Main Function**: `layout_flex_container()` - Full flexbox layout algorithm:
  1. Determine available main/cross size
  2. Calculate flex base size for each item
  3. Collect items into flex lines
  4. Resolve flexible lengths (grow/shrink)
  5. Determine cross size of each line
  6. Position items within lines

### 4. `src/layout/mod.rs` - Layout Engine Integration

#### Stacking Context
- **New Type**: `StackingContext` - For z-index handling:
  - Z-index tracking
  - Nested stacking contexts
  - Sorting by painting order

#### Positioned Elements Collection
- **New Type**: `PositionedElement` - Stores info about positioned elements
- **Integration**: `LayoutEngine` now collects positioned elements for z-index ordering

#### Exports
- Added exports for all new types and functions:
  - Flexbox types
  - Margin collapsing types
  - Float handling types
  - Text layout improvements

### 5. `src/layout/style.rs` - Style System Updates

#### New Properties Added to ComputedStyle
- `font_stretch: FontStretch` - Font width expansion/condensation
- `font_variant: FontVariant` - Small caps, etc.
- `font_variant_caps: FontVariantCaps` - Detailed caps control

#### New Enums
- **FontStretch**: `Normal`, `UltraCondensed` through `UltraExpanded`
- **FontVariant**: `Normal`, `SmallCaps`
- **FontVariantCaps**: `Normal`, `SmallCaps`, `AllSmallCaps`, `PetiteCaps`, etc.

## Test Examples Created

### `examples/test_margin_collapsing.html`
Tests margin collapsing behavior between adjacent blocks, negative margins, and BFC containment.

### `examples/test_floats.html`
Tests float positioning (left/right), text wrapping, clear property, and clearfix behavior.

### `examples/test_positioning.html`
Tests static, relative, absolute, and fixed positioning, along with z-index stacking.

### `examples/test_flexbox.html`
Tests flex-direction, justify-content, align-items, flex-wrap, and nested flex containers.

### `examples/test_text_layout.html`
Tests text-align (including justify), white-space variants, word-breaking, and vertical-align.

### `examples/test_comprehensive.html`
Combines multiple layout features to test real-world scenarios with:
- Header with navigation (flexbox)
- Two-column layout with sidebar
- Float-based article with image
- Positioned badge
- Multi-row flex features
- Footer layout

## Architecture Improvements

### Separation of Concerns
- Flex layout isolated in its own module
- Text layout enhancements are self-contained
- Flow layout handles all positioning modes consistently

### Extensibility
- New layout modes can be added following the established patterns
- Style system supports additional properties
- Metrics system allows for font-specific measurements

### CSS Spec Compliance
- Margin collapsing follows CSS 2.1 spec
- Float positioning handles edge cases
- Flexbox implements Level 1 specification
- Text layout respects Unicode line breaking

## Known Limitations

1. **Flexbox**: Advanced features like `order`, `flex-flow` shorthand, and complex alignment not yet implemented
2. **Positioning**: Sticky positioning not implemented
3. **Text**: Complex text shaping (Arabic, Indic scripts) not yet supported
4. **Line Breaking**: Full Unicode line breaking algorithm not implemented
5. **Pagination**: Page break handling in layout is basic

## Performance Considerations

- Float positioning has a max iteration limit to prevent infinite loops
- Flex layout uses single-pass algorithm where possible
- Text layout minimizes allocations during line breaking
- Margin state tracks previous values rather than recalculating

## Future Enhancements

1. CSS Grid layout module
2. CSS Multi-column layout
3. Advanced justification with Knuth-Plass algorithm
4. Proper baseline alignment across inline elements
5. Shrink-to-fit width calculations
6. Min/max-content width constraints
