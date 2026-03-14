# HTML2PDF Public API Documentation

Complete reference for the html2pdf library API.

## Table of Contents

1. [Quick Start](#quick-start)
2. [Core Types](#core-types)
3. [HTML Module](#html-module)
4. [CSS Module](#css-module)
5. [Layout Module](#layout-module)
6. [PDF Module](#pdf-module)
7. [Configuration](#configuration)
8. [Error Handling](#error-handling)
9. [CLI Usage](#cli-usage)

## Quick Start

### Basic Conversion

```rust
use html2pdf::{html_to_pdf, Config};

fn main() -> html2pdf::Result<()> {
    let html = r#"<h1>Hello, PDF!</h1>"#;
    let config = Config::default();
    let pdf = html_to_pdf(html, &config)?;
    std::fs::write("output.pdf", pdf)?;
    Ok(())
}
```

### With Custom Configuration

```rust
use html2pdf::{Config, PaperSize, Orientation, Margins, html_to_pdf};

fn main() -> html2pdf::Result<()> {
    let html = r#"<h1>Landscape Document</h1>"#;
    
    let config = Config::default()
        .with_paper_size(PaperSize::A4)
        .with_orientation(Orientation::Landscape)
        .with_margins(Margins::all(72.0));
    
    let pdf = html_to_pdf(html, &config)?;
    std::fs::write("landscape.pdf", pdf)?;
    Ok(())
}
```

## Core Types

All core types are exported from the crate root and `types` module.

### Point

A 2D coordinate in PDF points (1/72 inch).

```rust
use html2pdf::Point;

let point = Point::new(100.0, 200.0);
assert_eq!(point.x, 100.0);
assert_eq!(point.y, 200.0);
```

**Fields**:
- `x: f32` - X coordinate
- `y: f32` - Y coordinate

**Methods**:
- `new(x: f32, y: f32) -> Self` - Create a new point

### Size

A 2D size in PDF points.

```rust
use html2pdf::Size;

let size = Size::new(612.0, 792.0); // Letter size
let zero = Size::zero();
```

**Fields**:
- `width: f32` - Width in points
- `height: f32` - Height in points

**Methods**:
- `new(width: f32, height: f32) -> Self`
- `zero() -> Self` - Zero size

### Rect

A rectangle defined by position and size.

```rust
use html2pdf::{Rect, Size, Point};

let rect = Rect::new(10.0, 20.0, 100.0, 200.0);
let from_origin = Rect::from_origin(Size::new(100.0, 100.0));

// Methods
let contains = rect.contains(Point::new(50.0, 50.0)); // true
let right = rect.right();   // 110.0
let bottom = rect.bottom(); // 220.0
```

**Fields**:
- `x: f32` - Left edge position
- `y: f32` - Top edge position (in layout coordinates)
- `width: f32` - Rectangle width
- `height: f32` - Rectangle height

**Methods**:
- `new(x: f32, y: f32, width: f32, height: f32) -> Self`
- `from_origin(size: Size) -> Self` - Rectangle at origin
- `contains(point: Point) -> bool` - Point containment test
- `right() -> f32` - Right edge position
- `bottom() -> f32` - Bottom edge position

### Length

CSS length value with various units.

```rust
use html2pdf::types::Length;

let px = Length::Px(100.0);
let pt = Length::Pt(72.0);
let mm = Length::Mm(25.4);
let em = Length::Em(1.5);
let percent = Length::Percent(50.0);
let auto = Length::Auto;

// Convert to points
let points = px.to_pt(12.0); // 75.0 (base font size for em)
let with_container = percent.to_pt_with_container(12.0, 500.0); // 250.0
```

**Variants**:
- `Px(f32)` - Pixels (96 DPI)
- `Pt(f32)` - Points (1/72 inch)
- `Mm(f32)` - Millimeters
- `Cm(f32)` - Centimeters
- `In(f32)` - Inches
- `Em(f32)` - Font-relative (current element)
- `Rem(f32)` - Font-relative (root element)
- `Percent(f32)` - Percentage of container
- `Auto` - Automatic sizing

**Methods**:
- `to_pt(base_font_size: f32) -> f32` - Convert to points
- `to_pt_with_container(base_font_size: f32, container_size: f32) -> f32` - Convert with percentage support
- `is_auto() -> bool` - Check if auto

### Color

RGBA color representation.

```rust
use html2pdf::Color;

// Construction
let red = Color::new(255, 0, 0);
let transparent = Color::new_rgba(255, 0, 0, 128);
let from_hex = Color::from_hex("#FF0000").unwrap();
let from_hex_short = Color::from_hex("#F00").unwrap();
let from_hex_alpha = Color::from_hex("#FF000080").unwrap();

// Constants
let black = Color::BLACK;
let white = Color::WHITE;

// PDF conversion
let (r, g, b) = red.to_pdf(); // (1.0, 0.0, 0.0)
```

**Fields**:
- `r: u8` - Red component (0-255)
- `g: u8` - Green component (0-255)
- `b: u8` - Blue component (0-255)
- `a: u8` - Alpha component (0-255)

**Methods**:
- `new(r: u8, g: u8, b: u8) -> Self` - Opaque color
- `new_rgba(r: u8, g: u8, b: u8, a: u8) -> Self` - With transparency
- `from_hex(hex: &str) -> Option<Self>` - Parse hex color (#RGB, #RRGGBB, #RRGGBBAA)
- `to_pdf() -> (f32, f32, f32)` - Convert to PDF color values (0.0-1.0)

**Constants**:
- `BLACK`, `WHITE`, `RED`, `GREEN`, `BLUE`, `TRANSPARENT`

### PaperSize

Standard paper sizes.

```rust
use html2pdf::PaperSize;

let a4 = PaperSize::A4;
let letter = PaperSize::Letter;
let custom = PaperSize::Custom { width: 500.0, height: 700.0 };

let (width, height) = a4.size(); // (595.28, 841.89) points
```

**Variants**:
- `A0`, `A1`, `A2`, `A3`, `A4`, `A5`, `A6` - ISO A series
- `Letter` - US Letter (8.5 × 11 in)
- `Legal` - US Legal (8.5 × 14 in)
- `Tabloid` - US Tabloid (11 × 17 in)
- `Custom { width: f32, height: f32 }` - Custom dimensions in points

**Methods**:
- `size() -> (f32, f32)` - Get dimensions in points

### Orientation

Page orientation.

```rust
use html2pdf::Orientation;

let portrait = Orientation::Portrait;
let landscape = Orientation::Landscape;
```

**Variants**:
- `Portrait` - Default orientation
- `Landscape` - Rotated 90 degrees

### Margins

Page margins in points.

```rust
use html2pdf::Margins;

let all_sides = Margins::all(72.0);
let per_side = Margins::new(72.0, 54.0, 72.0, 54.0); // top, right, bottom, left
let symmetric = Margins::symmetric(72.0, 54.0); // vertical, horizontal
```

**Fields**:
- `top: f32`
- `right: f32`
- `bottom: f32`
- `left: f32`

**Methods**:
- `new(top: f32, right: f32, bottom: f32, left: f32) -> Self`
- `all(value: f32) -> Self` - Same margin on all sides
- `symmetric(vertical: f32, horizontal: f32) -> Self` - Vertical/horizontal pairs

### PdfError

Error type for all PDF operations.

```rust
use html2pdf::{Result, PdfError};

fn may_fail() -> Result<()> {
    Err(PdfError::Parse("Invalid HTML".to_string()))
}

// Matching
match result {
    Err(PdfError::Io(e)) => eprintln!("IO error: {}", e),
    Err(PdfError::Parse(msg)) => eprintln!("Parse error: {}", msg),
    Err(PdfError::Layout(msg)) => eprintln!("Layout error: {}", msg),
    Err(PdfError::Font(msg)) => eprintln!("Font error: {}", msg),
    Err(PdfError::Image(msg)) => eprintln!("Image error: {}", msg),
    Err(PdfError::InvalidColor(msg)) => eprintln!("Color error: {}", msg),
    Ok(_) => {}
}
```

**Variants**:
- `Io(std::io::Error)` - I/O operations
- `Parse(String)` - HTML/CSS parsing errors
- `Layout(String)` - Layout computation errors
- `Font(String)` - Font loading/errors
- `Image(String)` - Image processing errors
- `InvalidColor(String)` - Color parsing errors

### Result

Type alias for PDF results.

```rust
pub type Result<T> = std::result::Result<T, PdfError>;
```

## HTML Module

The `html` module provides HTML5 parsing capabilities.

### Document

Represents a parsed HTML document.

```rust
use html2pdf::html::{Document, parse_html};

let html = r#"<!DOCTYPE html>
<html>
<head><title>Test</title></head>
<body><h1>Hello</h1></body>
</html>"#;

let doc = parse_html(html)?;

// Access elements
let root = doc.root_element();
let body = doc.body_element();
let title = doc.title();

// Query
let heading = doc.get_element_by_id("main-heading");
let paragraphs = doc.get_elements_by_tag_name("p");
let items = doc.get_elements_by_class_name("item");
```

**Methods**:
- `root_element() -> &Element` - Get root `<html>` element
- `body_element() -> &Element` - Get `<body>` element
- `head_element() -> Option<&Element>` - Get `<head>` element
- `get_element_by_id(id: &str) -> Option<&Element>` - Find by ID
- `get_elements_by_tag_name(tag_name: &str) -> Vec<&Element>` - Find by tag
- `get_elements_by_class_name(class_name: &str) -> Vec<&Element>` - Find by class

### Element

Represents an HTML element.

```rust
use html2pdf::html::Element;

// Access properties
let tag = element.tag_name();           // "div"
let tag_lower = element.tag_name_lower(); // "div"

// Attributes
let id = element.id();                  // Option<&str>
let class_list = element.class_list();  // Vec<&str>
let has_class = element.has_class("active");
let attr = element.attr("data-value");  // Option<&str>
let has_attr = element.has_attr("disabled");

// Content
let text = element.text_content();      // Concatenated text
let html = element.inner_html();        // Serialized HTML
let children = element.children();      // &[Node]

// Navigation
let first_child = element.first_element_child();
let last_child = element.last_element_child();

// Query
let matches = element.matches(".active"); // Simple selector matching
```

**Methods**:
- `tag_name() -> &str` - Tag name as-is
- `tag_name_lower() -> String` - Lowercase tag name
- `attr(name: &str) -> Option<&str>` - Get attribute
- `set_attr(name: impl Into<String>, value: impl Into<String>)` - Set attribute
- `remove_attr(name: &str) -> Option<Attribute>` - Remove attribute
- `has_attr(name: &str) -> bool` - Check attribute existence
- `id() -> Option<&str>` - Get ID attribute
- `class_list() -> Vec<&str>` - Get classes as list
- `has_class(class_name: &str) -> bool` - Check class membership
- `children() -> &[Node]` - Get child nodes
- `append_child(child: Node)` - Add child
- `text_content() -> String` - Get all text content
- `inner_html() -> String` - Serialize children
- `matches(selector: &str) -> bool` - Simple selector matching
- `find_by_id(id: &str) -> Option<&Element>` - Recursive find
- `find_by_tag_name(tag_name: &str, result: &mut Vec<&Element>)` - Recursive find
- `find_by_class_name(class_name: &str, result: &mut Vec<&Element>)` - Recursive find

### Node

Represents any DOM node.

```rust
use html2pdf::html::Node;

match node {
    Node::Document(doc) => { /* ... */ }
    Node::Element(el) => { /* ... */ }
    Node::Text(text) => { /* ... */ }
    Node::Comment(comment) => { /* ... */ }
    Node::DocumentType(dt) => { /* ... */ }
    Node::ProcessingInstruction { target, data } => { /* ... */ }
}

// Methods
let name = node.node_name();
let is_element = node.is_element();
let is_text = node.is_text();
let as_element = node.as_element(); // Option<&Element>
let as_text = node.as_text(); // Option<&str>
```

### Parsing Functions

```rust
use html2pdf::html::{parse_html, parse_fragment};

// Parse complete document
let doc = parse_html(r#"<html><body>Hello</body></html>"#)?;

// Parse fragment (for innerHTML-like scenarios)
let nodes = parse_fragment(r#"<span>text</span>"#, "div")?;
```

## CSS Module

The `css` module provides CSS3 parsing with PrintCSS support.

### parse_stylesheet

Parse a complete CSS stylesheet.

```rust
use html2pdf::css::parse_stylesheet;

let css = r#"
    body { color: black; }
    h1 { font-size: 24px; }
"#;

let stylesheet = parse_stylesheet(css)?;
```

### parse_rule

Parse a single CSS rule.

```rust
use html2pdf::css::parse_rule;

let rule = parse_rule("h1 { font-size: 24px; }")?;
```

### parse_value

Parse a CSS value.

```rust
use html2pdf::css::parse_value;

let px = parse_value("100px")?;
let em = parse_value("1.5em")?;
let percent = parse_value("50%")?;
let keyword = parse_value("red")?;
```

### parse_selector

Parse a CSS selector.

```rust
use html2pdf::css::parse_selector;

let universal = parse_selector("*")?;
let element = parse_selector("div")?;
let id = parse_selector("#myid")?;
let class = parse_selector(".myclass")?;
```

### Key Types

```rust
use html2pdf::css::{
    Stylesheet, Rule, StyleRule, Declaration,
    Selector, SelectorPart, Combinator,
    CssValue, Unit, CssFunction,
    AtRule, PageRule, PageMarginBox, MarginBoxType
};
```

## Layout Module

The `layout` module provides document layout computation.

### LayoutContext

Global layout settings and state.

```rust
use html2pdf::layout::LayoutContext;
use html2pdf::{PaperSize, Orientation, Margins};

// Default A4 context
let ctx = LayoutContext::new();

// Custom page size
let ctx = LayoutContext::with_page_size(PaperSize::Letter, Orientation::Portrait);

// With custom margins
let ctx = LayoutContext::new()
    .with_margins(Margins::all(50.0));

// Access
let page_width = ctx.page_width();
let page_height = ctx.page_height();
let content_width = ctx.content_width();
let content_height = ctx.content_height();
let content_area = ctx.content_area(); // Rect
```

### LayoutEngine

Main layout computation engine.

```rust
use html2pdf::layout::LayoutEngine;
use html2pdf::html::Document;

let mut engine = LayoutEngine::new();

// Or with custom context
let mut engine = LayoutEngine::with_context(ctx);

// Add stylesheets
engine.add_stylesheet(stylesheet);

// Set viewport for media queries
engine.set_viewport_width(800.0);

// Layout document
let layout_tree = engine.layout_document(&document)?;

// Compute style for element
let style = engine.compute_style(&element, None);
```

### LayoutBox

Represents a box in the layout tree.

```rust
use html2pdf::layout::{LayoutBox, BoxType, Dimensions};

// Access properties
let box_type = &layout_box.box_type; // Block, Inline, Anonymous, etc.
let dimensions = &layout_box.dimensions; // Position and size
let children = &layout_box.children; // Vec<LayoutBox>
let is_laid_out = layout_box.is_laid_out;
```

### BoxType

Type of layout box.

```rust
use html2pdf::layout::BoxType;

let block = BoxType::Block;
let inline = BoxType::Inline;
let anonymous = BoxType::Anonymous;
let text = BoxType::Text("Hello".to_string());
```

### Layout Functions

```rust
use html2pdf::layout::{layout_document, build_layout_tree, collect_positioned_boxes};

// Full layout
let layout_tree = layout_document(&document, &stylesheets, Some(ctx))?;

// Build tree without layout
let unpositioned = build_layout_tree(&document, &stylesheets)?;

// Collect all positioned boxes
let boxes = collect_positioned_boxes(&layout_tree);
```

### PdfBox

Simplified representation for PDF rendering.

```rust
use html2pdf::layout::PdfBox;

// Convert from layout box
let pdf_box = PdfBox::from_layout_box(&layout_box);

// Access
let x = pdf_box.x;
let y = pdf_box.y;
let width = pdf_box.width;
let height = pdf_box.height;
let text = &pdf_box.text; // Option<String>

// Coordinate conversion
let pdf_coords = pdf_box.to_pdf_coordinates(page_height);
```

## PDF Module

The `pdf` module provides PDF generation capabilities.

### PdfWriter

Main PDF document builder.

```rust
use html2pdf::pdf::PdfWriter;
use html2pdf::{PaperSize, Orientation, Margins};

let mut writer = PdfWriter::new();

// Configure
writer.set_paper_size(PaperSize::A4, Orientation::Portrait);
writer.set_margins(Margins::all(72.0));

// Initialize document structure
writer.init_document();

// Set document info
writer.set_info("Title", "Author", "HTML2PDF");

// Add font
writer.add_standard_font("F1", "Helvetica");

// Add page with content
let content = PageContent::new();
// ... build content
writer.add_page(content);

// Write to output
let mut output = Vec::new();
writer.write(&mut output)?;
```

### PageContent

Builds PDF page content streams.

```rust
use html2pdf::pdf::PageContent;
use html2pdf::{Rect, Point};

let mut content = PageContent::new();

// Text operations
content.begin_text();
content.set_font("F1", 12.0);
content.text_position(100.0, 700.0);
content.show_text("Hello, PDF!");
content.end_text();

// Graphics
content.set_fill_color(Color::new(255, 0, 0));
content.draw_rect(Rect::new(100.0, 100.0, 200.0, 50.0));
content.fill();

// Lines
content.set_line_width(2.0);
content.set_stroke_color(Color::BLACK);
content.draw_line(Point::new(0.0, 0.0), Point::new(100.0, 100.0));
content.stroke();

// Images
content.draw_image("Img1", 100.0, 100.0, 200.0, 150.0);

// Graphics state
content.save_state();
// ... transformations
content.restore_state();
```

### PDF Object Types

```rust
use html2pdf::pdf::{
    PdfObject, PdfReference, PdfDictionary, PdfArray,
    PdfStream, PdfFont, PdfImage
};

// Objects
let null = PdfObject::Null;
let boolean = PdfObject::Boolean(true);
let integer = PdfObject::Integer(42);
let real = PdfObject::Real(3.14);
let string = PdfObject::String(b"Hello".to_vec());
let name = PdfObject::Name("Type".to_string());

// Dictionary
let mut dict = PdfDictionary::new();
dict.insert("Type", PdfObject::Name("Page".to_string()));
dict.insert("MediaBox", PdfObject::Array(/* ... */));

// Array
let mut array = PdfArray::new();
array.push(0i32);
array.push(PdfObject::Real(612.0));

// Reference
let reference = PdfReference::new(1, 0);
```

## Configuration

### Config

Main configuration struct for PDF generation.

```rust
use html2pdf::{Config, Input, html_to_pdf_from_input};

// Default configuration
let config = Config::default();

// From file
let config = Config::from_file("config.json")?;

// From JSON string
let config = Config::from_json(r#"{"paper_size": "A4"}"#)?;

// Builder-style construction
let config = Config::default()
    .with_paper_size(PaperSize::Letter)
    .with_orientation(Orientation::Landscape)
    .with_margins(Margins::all(50.0))
    .with_header("<h1>Header</h1>")
    .with_footer("<p>Page <span class='page'></span></p>");
```

### Input

Input source abstraction.

```rust
use html2pdf::Input;

// From file
let input = Input::File("document.html".to_string());

// From string
let input = Input::Html("<h1>Hello</h1>".to_string());

// From URL
let input = Input::Url("https://example.com".to_string());

// Load content
let html = input.load()?;
```

### High-Level Functions

```rust
use html2pdf::{html_to_pdf, html_to_pdf_from_input, Config, Input};

// Convert HTML string to PDF bytes
let pdf = html_to_pdf("<h1>Hello</h1>", &Config::default())?;

// Convert from input source
let input = Input::File("input.html".to_string());
let pdf = html_to_pdf_from_input(&input, &Config::default())?;
```

## Error Handling

### The Result Type

All fallible operations return `html2pdf::Result<T>`:

```rust
use html2pdf::{Result, PdfError};

fn process() -> Result<()> {
    let doc = parse_html(input)?;           // Propagates PdfError
    let stylesheet = parse_stylesheet(css)?; // Propagates PdfError
    let layout = layout_document(&doc, &[stylesheet], None)?;
    Ok(())
}
```

### Converting from std::io::Error

```rust
use html2pdf::Result;
use std::fs;

fn read_file(path: &str) -> Result<String> {
    // Automatically converts io::Error to PdfError::Io
    let content = fs::read_to_string(path)?;
    Ok(content)
}
```

### Creating Custom Errors

```rust
use html2pdf::{Result, PdfError};

fn validate_input(input: &str) -> Result<()> {
    if input.is_empty() {
        return Err(PdfError::Parse("Empty input".to_string()));
    }
    Ok(())
}
```

### Error Messages

```rust
use html2pdf::PdfError;

let err = PdfError::Parse("Invalid HTML".to_string());
println!("{}", err); // "Parse error: Invalid HTML"

let io_err = PdfError::Io(std::io::Error::new(
    std::io::ErrorKind::NotFound,
    "file not found"
));
println!("{}", io_err); // "IO error: file not found"
```

## CLI Usage

### Library CLI

The `cli` module provides the command-line interface.

```rust
// In your main.rs or as a library consumer
fn main() {
    if let Err(e) = html2pdf::cli::run() {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
```

### Command Line Arguments

```bash
# Basic usage
html2pdf input.html -o output.pdf

# Paper size and orientation
html2pdf input.html -o output.pdf -p Letter -O landscape

# Margins
html2pdf input.html -o output.pdf -m 1in
html2pdf input.html -o output.pdf -m "72,54,72,54"

# Headers and footers
html2pdf input.html -o output.pdf \
    --header "<h1>Header</h1>" \
    --footer "<p>Page <span class='page'></span></p>"

# From stdin
cat input.html | html2pdf -o output.pdf
html2pdf - -o output.pdf < input.html

# Custom page size
html2pdf input.html -o output.pdf --page-width 210mm --page-height 297mm

# Configuration file
html2pdf input.html -c config.json -o output.pdf

# Additional stylesheets
html2pdf input.html -s print.css -o output.pdf

# Validation
html2pdf validate input.html

# Show configuration
html2pdf config
```

### Programmatic CLI

```rust
use html2pdf::cli::{Cli, run};
use clap::Parser;

// Parse arguments manually
let cli = Cli::parse_from(&["html2pdf", "input.html", "-o", "output.pdf"]);

// Or run full CLI
fn main() -> Result<(), Box<dyn std::error::Error>> {
    html2pdf::cli::run()
}
```

---

## Advanced Examples

### Custom Page Size with Margins

```rust
use html2pdf::{html_to_pdf, Config, PaperSize, Orientation, Margins};

let config = Config::default()
    .with_paper_size(PaperSize::Custom {
        width: 595.0,  // Custom width in points
        height: 842.0, // Custom height in points
    })
    .with_orientation(Orientation::Portrait)
    .with_margins(Margins::symmetric(72.0, 54.0)); // 1in vertical, 0.75in horizontal

let html = r#"<h1>Custom Page</h1>"#;
let pdf = html_to_pdf(html, &config)?;
```

### Multi-Stage Processing

```rust
use html2pdf::{html, css, layout, pdf};

// 1. Parse HTML
let document = html::parse_html(html_input)?;

// 2. Parse CSS
let mut stylesheets = vec![];
for css_source in css_sources {
    stylesheets.push(css::parse_stylesheet(css_source)?);
}

// 3. Layout
let layout_tree = layout::layout_document(&document, &stylesheets, None)?;

// 4. Collect boxes
let boxes = layout::collect_positioned_boxes(&layout_tree);

// 5. Generate PDF
let mut writer = pdf::PdfWriter::new();
writer.init_document();
// ... add content
let mut output = Vec::new();
writer.write(&mut output)?;
```

### Working with Layout Results

```rust
use html2pdf::layout::{print_layout_tree, collect_positioned_boxes, PdfBox};

// Debug: print tree structure
print_layout_tree(&layout_tree, 0);

// Convert to PDF boxes
let pdf_boxes: Vec<PdfBox> = boxes.iter()
    .map(|b| PdfBox::from_layout_box(b))
    .collect();

// Filter visible boxes
let visible: Vec<_> = pdf_boxes.iter()
    .filter(|b| b.is_visible)
    .collect();
```
