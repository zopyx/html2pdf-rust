# HTML2PDF Architecture Documentation

This document provides an in-depth look at the technical architecture of html2pdf-rs, a from-scratch Rust HTML to PDF converter.

## Table of Contents

1. [System Overview](#system-overview)
2. [Module Descriptions](#module-descriptions)
3. [Data Flow](#data-flow)
4. [Key Design Decisions](#key-design-decisions)
5. [Extension Points](#extension-points)

## System Overview

HTML2PDF is a library and CLI tool that converts HTML documents to PDF format. It implements:

- **HTML5 Parser**: Complete WHATWG specification compliance
- **CSS3 Parser**: Full CSS3 support including PrintCSS/Paged Media extensions
- **Layout Engine**: Box model, formatting contexts, and pagination
- **PDF Generator**: Native PDF 1.4 implementation without external dependencies

### High-Level Architecture

```
┌─────────────────────────────────────────────────────────────────────────┐
│                              Input Layer                                 │
│     (File / URL / Stdin / String) → Input enum abstraction              │
└───────────────────────────────────┬─────────────────────────────────────┘
                                    │
                                    ▼
┌─────────────────────────────────────────────────────────────────────────┐
│                           HTML5 Parser (src/html/)                       │
│  ┌──────────────┐    ┌──────────────┐    ┌──────────────────────┐      │
│  │  Tokenizer   │───▶│ Tree Builder │───▶│      DOM Types       │      │
│  │ (tokenizer)  │    │(tree_builder)│    │  (Document, Element) │      │
│  └──────────────┘    └──────────────┘    └──────────────────────┘      │
└───────────────────────────────────┬─────────────────────────────────────┘
                                    │ Document
                                    ▼
┌─────────────────────────────────────────────────────────────────────────┐
│                         CSS Parser (src/css/)                            │
│  ┌──────────────┐    ┌──────────────┐    ┌──────────────────────┐      │
│  │  Tokenizer   │───▶│    Parser    │───▶│    Stylesheet        │      │
│  │(css/tokenizer│    │ (css/parser) │    │  (Rules, Selectors)  │      │
│  └──────────────┘    └──────────────┘    └──────────────────────┘      │
│                                                                          │
│  Additional modules: selectors.rs, at_rules.rs, values.rs               │
└───────────────────────────────────┬─────────────────────────────────────┘
                                    │ Stylesheet[]
                                    ▼
┌─────────────────────────────────────────────────────────────────────────┐
│                         Layout Engine (src/layout/)                      │
│  ┌──────────────┐    ┌──────────────┐    ┌──────────────────────┐      │
│  │   Box Tree   │───▶│    Layout    │───▶│  Positioned Boxes    │      │
│  │ (box_model)  │    │   (flow)     │    │   (LayoutBox tree)   │      │
│  └──────────────┘    └──────────────┘    └──────────────────────┘      │
│         ▲                                           │                    │
│         │                                           │ PdfBox             │
│  ┌──────────────┐                          ┌──────────────┐             │
│  │    Style     │                          │  Text Layout │             │
│  │  Resolver    │                          │    (text)    │             │
│  │   (style)    │                          └──────────────┘             │
│  └──────────────┘                                                        │
└───────────────────────────────────┬─────────────────────────────────────┘
                                    │
                                    ▼
┌─────────────────────────────────────────────────────────────────────────┐
│                        PDF Generation (src/pdf/)                         │
│  ┌──────────────┐    ┌──────────────┐    ┌──────────────────────┐      │
│  │   Objects    │───▶│   Streams    │───▶│   PDF Document       │      │
│  │   (object)   │    │   (stream)   │    │     (writer)         │      │
│  └──────────────┘    └──────────────┘    └──────────────────────┘      │
│                                                                          │
│  Supporting modules: font.rs, image.rs                                  │
└─────────────────────────────────────────────────────────────────────────┘
```

## Module Descriptions

### Core Types (`src/types.rs`)

Fundamental types used throughout the library:

- **`Point`**: 2D coordinate (x, y)
- **`Size`**: 2D dimensions (width, height)
- **`Rect`**: Rectangle with position and size
- **`Length`**: CSS length values (px, pt, mm, em, %, etc.)
- **`Color`**: RGBA color representation
- **`PaperSize`**: Standard paper sizes (A4, Letter, etc.)
- **`Orientation`**: Portrait or Landscape
- **`Margins`**: Page margins (top, right, bottom, left)
- **`PdfError`**: Error type for all operations

### HTML Module (`src/html/`)

#### `tokenizer.rs`
- **Purpose**: Tokenizes HTML source into tokens
- **Key Types**: `HtmlTokenizer`, `Token`
- **Features**: 
  - HTML5 tokenization rules
  - Character references (entities)
  - CDATA sections
  - Script/data state handling

#### `tree_builder.rs`
- **Purpose**: Builds DOM tree from tokens
- **Key Types**: `TreeBuilder`
- **Features**:
  - HTML5 tree construction
  - Foster parenting
  - Adoption agency algorithm
  - Fragment parsing support

#### `dom.rs`
- **Purpose**: DOM data structures
- **Key Types**: `Document`, `Element`, `Node`, `TextNode`, `Attribute`
- **Features**:
  - Standard DOM operations
  - Element traversal and querying
  - HTML serialization

#### `mod.rs`
- **Public API**: `parse_html()`, `parse_fragment()`
- **Utilities**: Element classification (void, block, inline)

### CSS Module (`src/css/`)

#### `tokenizer.rs`
- **Purpose**: Tokenizes CSS source
- **Key Types**: `CssTokenizer`, `CssToken`
- **Features**:
  - CSS Syntax Module Level 3
  - At-rule handling
  - Comment stripping

#### `parser.rs`
- **Purpose**: Parses tokens into stylesheet structure
- **Key Types**: `CssParser`, `Stylesheet`, `Rule`, `StyleRule`, `Declaration`
- **Features**:
  - Rule parsing (style, at-rules)
  - Declaration parsing
  - Nested at-rules

#### `selectors.rs`
- **Purpose**: CSS selector representation and matching
- **Key Types**: `Selector`, `SelectorPart`, `Combinator`, `AttributeOp`
- **Features**:
  - All CSS selector types
  - Specificity calculation
  - Selector matching

#### `at_rules.rs`
- **Purpose**: CSS at-rule handling
- **Key Types**: `AtRule`, `PageRule`, `PageMarginBox`, `MarginBoxType`
- **Features**:
  - `@page` rules
  - Page selectors (:first, :left, :right)
  - Page margin boxes (@top-center, etc.)
  - PrintCSS extensions

#### `values.rs`
- **Purpose**: CSS value types
- **Key Types**: `CssValue`, `CssFunction`, `Unit`
- **Features**:
  - All CSS units
  - Functions (calc, var, etc.)
  - Color values

#### `mod.rs`
- **Public API**: `parse_stylesheet()`, `parse_rule()`, `parse_value()`, `parse_selector()`
- **Utilities**: Property validation, length parsing

### Layout Module (`src/layout/`)

#### `box_model.rs`
- **Purpose**: CSS box model implementation
- **Key Types**: `LayoutBox`, `BoxType`, `Dimensions`, `EdgeSizes`
- **Features**:
  - Box tree construction
  - Width/height calculation
  - Margin collapsing
  - Box type determination

#### `flow.rs`
- **Purpose**: Normal flow layout
- **Key Types**: `BlockFormattingContext`, `InlineFormattingContext`
- **Features**:
  - Block formatting context
  - Inline formatting context
  - Float handling
  - Clear handling

#### `style.rs`
- **Purpose**: Style computation
- **Key Types**: `ComputedStyle`, `StyleResolver`, `Display`, `Position`
- **Features**:
  - Cascade
  - Specificity
  - Inheritance
  - Initial values

#### `text.rs`
- **Purpose**: Text layout and line breaking
- **Key Types**: `TextLayout`, `LineBreaker`, `Line`, `TextFragment`
- **Features**:
  - Line breaking
  - Text metrics
  - Text alignment

#### `mod.rs`
- **Key Types**: `LayoutContext`, `LayoutEngine`, `PdfBox`
- **Public API**: `layout_document()`, `build_layout_tree()`, `collect_positioned_boxes()`
- **Features**:
  - Layout context management
  - Document-level layout orchestration
  - PDF box generation

### PDF Module (`src/pdf/`)

#### `object.rs`
- **Purpose**: PDF object types
- **Key Types**: `PdfObject`, `PdfReference`, `PdfDictionary`, `PdfArray`
- **Features**:
  - All PDF primitive types
  - Object serialization
  - Reference management

#### `stream.rs`
- **Purpose**: PDF streams and compression
- **Key Types**: `PdfStream`, `FlateEncode`
- **Features**:
  - Stream objects
  - Flate (zlib) compression
  - Filter chains

#### `writer.rs`
- **Purpose**: PDF document assembly
- **Key Types**: `PdfWriter`
- **Features**:
  - Document structure
  - Page management
  - Font embedding
  - Image embedding
  - Cross-reference table

#### `font.rs`
- **Purpose**: Font handling
- **Key Types**: `PdfFont`
- **Features**:
  - Standard 14 fonts
  - TrueType embedding
  - Font metrics

#### `image.rs`
- **Purpose**: Image embedding
- **Key Types**: `PdfImage`
- **Features**:
  - PNG support
  - JPEG support
  - Color space handling

#### `mod.rs`
- **Key Types**: `PageContent`, `TextAlign`, `VerticalAlign`
- **Features**:
  - Page content stream building
  - Drawing operations
  - Text operations
  - Graphics state

### CLI (`src/cli.rs`, `src/main.rs`)

- **Purpose**: Command-line interface
- **Features**:
  - Argument parsing with clap
  - Input handling (file, URL, stdin)
  - Configuration management
  - Progress reporting
  - Error handling

## Data Flow

### HTML to PDF Conversion Flow

```
Input String
    │
    ▼
┌─────────────────┐
│  HtmlTokenizer  │ ← Tokenizes input into HTML tokens
│   (src/html)    │
└────────┬────────┘
         │ Vec<Token>
         ▼
┌─────────────────┐
│   TreeBuilder   │ ← Builds DOM tree from tokens
│   (src/html)    │
└────────┬────────┘
         │ Document
         ▼
┌─────────────────┐
│   CssTokenizer  │ ← Tokenizes CSS from <style> tags
│    (src/css)    │
└────────┬────────┘
         │ Vec<CssToken>
         ▼
┌─────────────────┐
│    CssParser    │ ← Parses tokens into stylesheet
│    (src/css)    │
└────────┬────────┘
         │ Stylesheet
         ▼
┌─────────────────┐
│   StyleResolver │ ← Computes styles (cascade + specificity)
│  (src/layout)   │
└────────┬────────┘
         │ ComputedStyle
         ▼
┌─────────────────┐
│   build_box_tree│ ← Creates box tree from DOM
│ (src/layout/box)│
└────────┬────────┘
         │ LayoutBox
         ▼
┌─────────────────┐
│  layout_block/  │ ← Computes positions and sizes
│  layout_inline  │
└────────┬────────┘
         │ LayoutBox (positioned)
         ▼
┌─────────────────┐
│  PdfBox::from_  │ ← Converts to PDF representation
│   layout_box()  │
└────────┬────────┘
         │ PdfBox
         ▼
┌─────────────────┐
│   PageContent   │ ← Builds PDF content stream
│   (src/pdf/mod) │
└────────┬────────┘
         │ Vec<u8>
         ▼
┌─────────────────┐
│   PdfWriter     │ ← Assembles PDF document
│ (src/pdf/writer)│
└────────┬────────┘
         │ Vec<u8> (PDF bytes)
         ▼
    Output File
```

### Style Computation Flow

```
Element
    │
    ▼
┌─────────────────┐     ┌─────────────────┐
│  Match Selectors│────▶│  Get Rule Set   │
│                 │     │  (sorted by     │
│                 │     │  specificity)   │
└────────┬────────┘     └────────┬────────┘
         │                       │
         ▼                       ▼
┌─────────────────┐     ┌─────────────────┐
│   Apply Cascade │◀────│  User Agent     │
│   (user styles  │     │  Stylesheet     │
│    override UA) │     │                 │
└────────┬────────┘     └─────────────────┘
         │
         ▼
┌─────────────────┐     ┌─────────────────┐
│  Inherit from   │◀────│  Parent Style   │
│  Parent (for    │     │                 │
│  inherited      │     │                 │
│  properties)    │     │                 │
└────────┬────────┘     └─────────────────┘
         │
         ▼
┌─────────────────┐
│  ComputedStyle  │ ← Final computed values
│                 │
└─────────────────┘
```

### Page Layout Flow

```
LayoutContext
    │
    ▼
┌─────────────────┐
│  Calculate      │
│  Page Area      │ ← page_size - margins
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│  Create BFC     │ ← Block Formatting Context
│  (root context) │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│  Layout Children│ ← For each child:
│                 │   - Calculate width
│                 │   - Calculate position
│                 │   - Layout children
│                 │   - Calculate height
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│  Fragmentation  │ ← Handle page breaks
│                 │   - Check break-before/after/inside
│                 │   - Handle orphans/widows
│                 │   - Create new pages as needed
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│  Output Pages   │ ← Vec<Page>
│                 │
└─────────────────┘
```

## Key Design Decisions

### 1. Zero External Dependencies for PDF

**Decision**: Implement PDF generation from scratch rather than using a library.

**Rationale**:
- Full control over output
- Smaller binary size
- No C dependencies
- Learning value
- Easier to add specialized features

**Trade-offs**:
- More code to maintain
- Must handle PDF spec edge cases
- Longer development time

### 2. Modular Parser Design

**Decision**: Separate tokenizers from parsers, following the HTML5/CSS specs.

**Rationale**:
- Matches specification structure
- Easier to test each phase
- Allows for different parsing strategies
- Clear separation of concerns

**Trade-offs**:
- More allocations (token stream)
- Slightly more complex than combined approach

### 3. Immutable DOM with Clone-on-Write

**Decision**: DOM nodes are immutable; modifications create new structures.

**Rationale**:
- Easier to reason about
- Thread-safe by design
- Enables certain optimizations

**Trade-offs**:
- Higher memory usage for modifications
- More complex mutation patterns

### 4. Layout as a Separate Phase

**Decision**: Layout is completely separate from parsing and rendering.

**Rationale**:
- Clean separation of concerns
- Easier to test layout independently
- Allows multiple layout strategies
- Enables pagination preview

**Trade-offs**:
- Requires storing intermediate structures
- More memory usage during conversion

### 5. PrintCSS First

**Decision**: Design layout engine with PrintCSS/pagination in mind from the start.

**Rationale**:
- Core use case is PDF generation
- Avoids retrofitting pagination later
- Cleaner page-related APIs

**Trade-offs**:
- Screen media support may differ from browsers
- Some web compatibility trade-offs

### 6. Error Handling with thiserror

**Decision**: Use `thiserror` for error types, single `Result<T>` alias.

**Rationale**:
- Consistent error handling
- Automatic `From` implementations
- Good error messages

**Example**:
```rust
pub type Result<T> = std::result::Result<T, PdfError>;

#[derive(Debug, thiserror::Error)]
pub enum PdfError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Parse error: {0}")]
    Parse(String),
    // ...
}
```

### 7. Type-Safe Units

**Decision**: Use separate types for different units, explicit conversion.

**Rationale**:
- Prevent unit confusion bugs
- Explicit conversion points
- Self-documenting code

**Example**:
```rust
pub enum Length {
    Px(f32),
    Pt(f32),
    Mm(f32),
    // ...
}

impl Length {
    pub fn to_pt(&self, base_font_size: f32) -> f32 {
        match *self {
            Length::Px(v) => v * 0.75,
            Length::Pt(v) => v,
            // ...
        }
    }
}
```

## Extension Points

### Adding New CSS Properties

1. Add to `STANDARD_PROPERTIES` in `src/css/mod.rs`
2. Add field to `ComputedStyle` in `src/layout/style.rs`
3. Add parsing logic in `src/css/values.rs`
4. Add layout handling in appropriate module
5. Add PDF rendering support

### Adding New Input Formats

1. Add variant to `Input` enum
2. Implement `load()` method
3. Add CLI support in `src/cli.rs`

Example:
```rust
pub enum Input {
    File(String),
    Html(String),
    Url(String),
    // Add new variant:
    Markdown(String),
}

impl Input {
    pub fn load(&self) -> Result<String> {
        match self {
            // ... existing cases
            Input::Markdown(md) => {
                // Convert markdown to HTML
                Ok(convert_markdown_to_html(md))
            }
        }
    }
}
```

### Adding New Layout Algorithms

The layout system is designed to support different formatting contexts:

1. **Block formatting**: Already implemented in `flow.rs`
2. **Flexbox**: Add `src/layout/flex.rs`
3. **Grid**: Add `src/layout/grid.rs`
4. **Table**: Add `src/layout/table.rs`

Each should:
- Implement `FormattingContext` trait
- Handle child layout
- Manage positioning
- Support fragmentation

Example structure:
```rust
// src/layout/flex.rs
pub struct FlexFormattingContext {
    container: Rect,
    flex_direction: FlexDirection,
    justify_content: JustifyContent,
    align_items: AlignItems,
}

impl FormattingContext for FlexFormattingContext {
    fn layout_children(&mut self, parent: &mut LayoutBox, children: &[LayoutBox]) {
        // Flex layout algorithm
    }
}
```

### Adding New PDF Features

1. **Annotations**: Extend `PdfWriter` in `writer.rs`
2. **Forms**: Add form field types to `object.rs`
3. **JavaScript**: Add document-level JS actions
4. **Encryption**: Add security handler

Example - adding hyperlink support:
```rust
// In src/pdf/writer.rs
impl PdfWriter {
    pub fn add_link_annotation(
        &mut self,
        page_ref: PdfReference,
        rect: Rect,
        uri: &str,
    ) -> PdfReference {
        let mut annot = PdfDictionary::new();
        annot.insert("Type", PdfObject::Name("Annot".to_string()));
        annot.insert("Subtype", PdfObject::Name("Link".to_string()));
        // ... annotation properties
        self.add_object(PdfObject::Dictionary(annot))
    }
}
```

### Custom Renderers

The layout output (`PdfBox`) can be rendered to different backends:

```rust
pub trait Renderer {
    fn render_box(&mut self, box_: &PdfBox);
    fn render_text(&mut self, text: &str, style: &TextStyle);
    fn render_image(&mut self, image: &ImageData);
    fn finish(self) -> Vec<u8>;
}

// Existing: PdfRenderer
// Possible: SvgRenderer, PngRenderer, etc.
```

### Plugin Architecture (Future)

Planned extension points:

1. **Preprocessors**: Transform input before parsing
   ```rust
   pub trait Preprocessor {
       fn process(&self, input: &str) -> String;
   }
   ```

2. **Postprocessors**: Transform PDF after generation
   ```rust
   pub trait Postprocessor {
       fn process(&self, pdf: &mut PdfWriter);
   }
   ```

3. **Custom Functions**: CSS-like extension functions
   ```rust
   pub trait CssFunction {
       fn name(&self) -> &str;
       fn evaluate(&self, args: &[CssValue]) -> Result<CssValue>;
   }
   ```

---

## Performance Considerations

### Memory Usage

- **Streaming parsing**: HTML is tokenized and parsed in a single pass
- **Box tree reuse**: Layout can be recomputed without re-parsing
- **PDF object pooling**: Reuse object allocations where possible

### CPU Optimization

- **Selector matching**: Use bloom filters for fast rejection
- **Style caching**: Cache computed styles for identical elements
- **Text shaping**: Cache glyph metrics

### Parallelization Opportunities

1. **Independent page layout**: Pages can be laid out in parallel
2. **Image encoding**: Image compression can be parallelized
3. **Font subsetting**: Font processing can be parallel

---

## Testing Strategy

### Unit Tests

Each module has comprehensive unit tests in `#[cfg(test)]` blocks:

- **Tokenizer**: Token patterns, edge cases
- **Parser**: Structure validation, error handling
- **Layout**: Position calculations, constraint solving
- **PDF**: Output validation, object structure

### Integration Tests

Located in `tests/` directory:

- **End-to-end**: HTML input → PDF output
- **Reference tests**: Compare against expected output
- **Round-trip**: Parse → Serialize → Parse

### Property-Based Tests

Using `proptest` for:

- **HTML parsing**: Arbitrary HTML strings
- **CSS parsing**: Arbitrary CSS declarations
- **Layout**: Random box constraints

### Snapshot Testing

Using `insta` for:

- **PDF output**: Binary comparison of generated PDFs
- **Layout trees**: Structured comparison

---

## Security Considerations

### Input Validation

- **HTML**: Entity expansion limits, nesting depth limits
- **CSS**: Property value limits, @import restrictions
- **Images**: Maximum dimensions, format validation

### Output Safety

- **PDF**: Object reference limits, stream size limits
- **Fonts**: Glyph count limits, table validation

### Resource Limits

- Maximum document size
- Maximum parsing time
- Maximum memory usage
