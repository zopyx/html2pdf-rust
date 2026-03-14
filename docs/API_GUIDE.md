# API Guide

> Complete guide to using html2pdf as a Rust library

---

## Table of Contents

1. [Getting Started](#getting-started)
2. [Basic Library Usage](#basic-library-usage)
3. [Configuration](#configuration)
4. [Input Sources](#input-sources)
5. [Error Handling](#error-handling)
6. [Working with HTML](#working-with-html)
7. [Working with CSS](#working-with-css)
8. [Layout and Rendering](#layout-and-rendering)
9. [PDF Generation](#pdf-generation)
10. [Advanced Configuration](#advanced-configuration)
11. [Custom Renderers](#custom-renderers)
12. [Examples](#examples)

---

## Getting Started

### Adding the Dependency

Add html2pdf to your `Cargo.toml`:

```toml
[dependencies]
html2pdf = "0.1.0"
```

### Basic Structure

```rust
use html2pdf::{html_to_pdf, Config};

fn main() -> html2pdf::Result<()> {
    // Your code here
    Ok(())
}
```

---

## Basic Library Usage

### Simple Conversion

Convert HTML string to PDF bytes:

```rust
use html2pdf::{html_to_pdf, Config};

fn main() -> html2pdf::Result<()> {
    let html = r#"
        <html>
            <body>
                <h1>Hello, PDF!</h1>
                <p>This is a test document.</p>
            </body>
        </html>
    "#;
    
    let config = Config::default();
    let pdf = html_to_pdf(html, &config)?;
    
    // Write to file
    std::fs::write("output.pdf", pdf)?;
    
    Ok(())
}
```

### Using Input Sources

Handle different input types uniformly:

```rust
use html2pdf::{html_to_pdf_from_input, Input, Config};

fn main() -> html2pdf::Result<()> {
    let config = Config::default();
    
    // From file
    let input = Input::File("document.html".to_string());
    let pdf = html_to_pdf_from_input(&input, &config)?;
    std::fs::write("from_file.pdf", pdf)?;
    
    // From string
    let input = Input::Html("<h1>Hello</h1>".to_string());
    let pdf = html_to_pdf_from_input(&input, &config)?;
    std::fs::write("from_string.pdf", pdf)?;
    
    // From URL (when implemented)
    let input = Input::Url("https://example.com".to_string());
    // let pdf = html_to_pdf_from_input(&input, &config)?;
    
    Ok(())
}
```

---

## Configuration

### Creating Configuration

```rust
use html2pdf::{Config, PaperSize, Orientation, Margins};

// Default configuration (A4, portrait, 72pt margins)
let config = Config::default();

// Custom configuration
let config = Config::default()
    .with_paper_size(PaperSize::Letter)
    .with_orientation(Orientation::Landscape)
    .with_margins(Margins::all(50.0));
```

### Configuration Options

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `paper_size` | `PaperSize` | `A4` | Page size |
| `orientation` | `Orientation` | `Portrait` | Page orientation |
| `margins` | `Margins` | 72pt all sides | Page margins |
| `page_width` | `Option<f32>` | `None` | Custom page width (overrides paper_size) |
| `page_height` | `Option<f32>` | `None` | Custom page height (overrides paper_size) |
| `header` | `Option<String>` | `None` | Header HTML template |
| `footer` | `Option<String>` | `None` | Footer HTML template |
| `user_stylesheets` | `Vec<String>` | `[]` | Additional CSS stylesheets |
| `base_url` | `Option<String>` | `None` | Base URL for resolving relative URLs |
| `timeout_seconds` | `u64` | `30` | Network timeout |
| `debug_layout` | `bool` | `false` | Enable layout debugging |

### Builder Pattern

```rust
use html2pdf::{Config, PaperSize, Orientation, Margins};

let config = Config::default()
    .with_paper_size(PaperSize::Letter)
    .with_orientation(Orientation::Landscape)
    .with_margins(Margins::new(72.0, 54.0, 72.0, 54.0))
    .with_header("<h2>Company Report</h2>")
    .with_footer("<p>Page {{page}} of {{pages}}</p>");
```

### Loading from JSON

```rust
use html2pdf::Config;

// From file
let config = Config::from_file("config.json")?;

// From string
let config = Config::from_json(r#"{
    "paper_size": "A4",
    "orientation": "landscape",
    "margin": 50
}"#)?;
```

### Paper Sizes

```rust
use html2pdf::PaperSize;

let sizes = [
    PaperSize::A0,
    PaperSize::A1,
    PaperSize::A2,
    PaperSize::A3,
    PaperSize::A4,
    PaperSize::A5,
    PaperSize::A6,
    PaperSize::Letter,
    PaperSize::Legal,
    PaperSize::Tabloid,
    PaperSize::Custom { width: 400.0, height: 600.0 },
];

// Get dimensions in points
let (width, height) = PaperSize::A4.size();
```

### Margins

```rust
use html2pdf::Margins;

// All sides equal
let margins = Margins::all(72.0);

// Symmetric margins (vertical, horizontal)
let margins = Margins::symmetric(72.0, 54.0);

// Individual sides (top, right, bottom, left)
let margins = Margins::new(72.0, 54.0, 72.0, 54.0);
```

---

## Input Sources

### Input Enum

```rust
use html2pdf::Input;

// File input
let input = Input::File("document.html".to_string());

// String input
let input = Input::Html("<h1>Hello</h1>".to_string());

// URL input
let input = Input::Url("https://example.com".to_string());
```

### Loading Input

```rust
use html2pdf::Input;

let input = Input::File("test.html".to_string());

// Get description
println!("Loading: {}", input.description());
// Output: "Loading: file: test.html"

// Load content
let html_content = input.load()?;
println!("Content length: {} bytes", html_content.len());
```

### Custom Input Handling

```rust
use html2pdf::Input;

fn process_input(input: &Input) -> html2pdf::Result<String> {
    match input {
        Input::File(path) => {
            println!("Reading file: {}", path);
            input.load()
        }
        Input::Html(content) => {
            println!("Using HTML string ({} bytes)", content.len());
            Ok(content.clone())
        }
        Input::Url(url) => {
            println!("Fetching URL: {}", url);
            // Implement URL fetching
            input.load()
        }
    }
}
```

---

## Error Handling

### Error Types

html2pdf uses a custom error type `PdfError`:

```rust
use html2pdf::PdfError;

fn handle_error(error: PdfError) {
    match error {
        PdfError::Io(e) => {
            eprintln!("IO Error: {}", e);
        }
        PdfError::Parse(msg) => {
            eprintln!("Parse Error: {}", msg);
        }
        PdfError::Layout(msg) => {
            eprintln!("Layout Error: {}", msg);
        }
        PdfError::Font(msg) => {
            eprintln!("Font Error: {}", msg);
        }
        PdfError::Image(msg) => {
            eprintln!("Image Error: {}", msg);
        }
        PdfError::InvalidColor(msg) => {
            eprintln!("Color Error: {}", msg);
        }
    }
}
```

### Using Result Type

```rust
use html2pdf::{Result, html_to_pdf, Config};

fn convert_document(html: &str) -> Result<Vec<u8>> {
    let config = Config::default();
    html_to_pdf(html, &config)
}

fn main() {
    match convert_document("<h1>Test</h1>") {
        Ok(pdf) => {
            std::fs::write("output.pdf", pdf).unwrap();
            println!("Success!");
        }
        Err(e) => {
            eprintln!("Conversion failed: {}", e);
        }
    }
}
```

### Propagating Errors

```rust
use html2pdf::Result;

fn process_documents(files: &[String]) -> Result<Vec<Vec<u8>>> {
    let mut results = Vec::new();
    
    for file in files {
        let input = html2pdf::Input::File(file.clone());
        let config = html2pdf::Config::default();
        let pdf = html2pdf::html_to_pdf_from_input(&input, &config)?;
        results.push(pdf);
    }
    
    Ok(results)
}
```

### Custom Error Handling

```rust
use html2pdf::{Result, PdfError};

fn convert_with_fallback(html: &str) -> Result<Vec<u8>> {
    let config = html2pdf::Config::default();
    
    match html2pdf::html_to_pdf(html, &config) {
        Ok(pdf) => Ok(pdf),
        Err(PdfError::Parse(msg)) => {
            eprintln!("Parse error, trying with simplified HTML: {}", msg);
            // Try with simplified HTML
            let simplified = format!("<html><body>{}</body></html>", html);
            html2pdf::html_to_pdf(&simplified, &config)
        }
        Err(e) => Err(e),
    }
}
```

---

## Working with HTML

### Parsing HTML

```rust
use html2pdf::html;

let html_content = r#"
    <html>
        <head><title>Test Document</title></head>
        <body>
            <h1>Hello World</h1>
            <p>This is a paragraph.</p>
        </body>
    </html>
"#;

let document = html::parse_html(html_content)?;
```

### Accessing Document Elements

```rust
use html2pdf::html;

let document = html::parse_html("<html><body><h1>Test</h1></body></html>")?;

// Get document title
println!("Title: {:?}", document.title);

// Get body element
let body = document.body_element();
println!("Body tag: {}", body.tag_name());

// Get head element
let head = document.head_element();
```

### Working with Elements

```rust
use html2pdf::html::{parse_html, Element};

let document = parse_html(r#"
    <div id="main" class="container">
        <p class="intro">Introduction</p>
    </div>
"#)?;

// Access elements
let body = document.body_element();

// Element methods (available on Element type)
// - tag_name(): Get tag name
// - id(): Get ID attribute
// - has_class(class): Check for class
// - get_attribute(name): Get attribute value
// - children(): Iterate over children
// - text_content(): Get text content
```

### Creating HTML Programmatically

```rust
fn create_document(title: &str, content: &str) -> String {
    format!(r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <title>{}</title>
    <style>
        body {{ font-family: Arial, sans-serif; margin: 2cm; }}
        h1 {{ color: #333; }}
    </style>
</head>
<body>
    <h1>{}</h1>
    {}
</body>
</html>"#, title, title, content)
}
```

---

## Working with CSS

### Parsing Stylesheets

```rust
use html2pdf::css;

let css_content = r#"
    body {
        font-family: Georgia, serif;
        font-size: 12pt;
        line-height: 1.6;
    }
    
    h1 {
        color: #333;
        font-size: 24pt;
    }
"#;

let stylesheet = css::parse_stylesheet(css_content)?;
```

### Using Multiple Stylesheets

```rust
use html2pdf::css;

let base_css = css::parse_stylesheet("body { margin: 0; }")?;
let theme_css = css::parse_stylesheet("body { background: white; }")?;
let print_css = css::parse_stylesheet("@page { margin: 2cm; }")?;

let stylesheets = vec![base_css, theme_css, print_css];
```

### Parsing Individual Selectors

```rust
use html2pdf::css;

let selector = css::parse_selector("div.container > p:first-child")?;
println!("Selector: {:?}", selector);
```

### Validating CSS Properties

```rust
use html2pdf::css;

// Check if a property is valid
assert!(css::is_valid_property("display"));
assert!(css::is_valid_property("font-size"));
assert!(css::is_valid_property("--custom-property"));
assert!(!css::is_valid_property("invalid-property"));
```

### Working with CSS Values

```rust
use html2pdf::css;

// Parse CSS values
let value = css::parse_value("24px")?;
let value = css::parse_value("#FF0000")?;
let value = css::parse_value("center")?;
```

---

## Layout and Rendering

### Creating Layout Context

```rust
use html2pdf::layout::{LayoutContext, layout_document};
use html2pdf::{PaperSize, Orientation, Margins};

// Default context
let context = LayoutContext::new();

// With page size
let context = LayoutContext::with_page_size(PaperSize::A4, Orientation::Portrait);

// With margins
let context = LayoutContext::with_page_size(PaperSize::Letter, Orientation::Portrait)
    .with_margins(Margins::all(72.0));
```

### Layout Engine

```rust
use html2pdf::layout::LayoutEngine;
use html2pdf::html;

let html_content = "<html><body><h1>Test</h1></body></html>";
let document = html::parse_html(html_content)?;

// Create engine
let mut engine = LayoutEngine::new();

// Layout document
let layout_tree = engine.layout_document(&document)?;
```

### Accessing Layout Information

```rust
use html2pdf::layout::{LayoutEngine, LayoutBox, BoxType};

// After layout...
fn analyze_layout(layout_tree: &LayoutBox) {
    // Check if laid out
    if layout_tree.is_laid_out {
        println!("Layout complete!");
    }
    
    // Get box type
    match layout_tree.box_type {
        BoxType::Block => println!("Block box"),
        BoxType::Inline => println!("Inline box"),
        BoxType::Anonymous => println!("Anonymous box"),
        BoxType::Text => println!("Text box"),
    }
    
    // Get dimensions
    let dims = &layout_tree.dimensions;
    println!("Content: {}x{} at ({}, {})",
        dims.content.width,
        dims.content.height,
        dims.content.x,
        dims.content.y
    );
    
    // Recurse into children
    for child in &layout_tree.children {
        analyze_layout(child);
    }
}
```

### Printing Layout Tree

```rust
use html2pdf::layout::print_layout_tree;

// Print the layout tree for debugging
print_layout_tree(&layout_tree, 0);
```

### Collecting Positioned Boxes

```rust
use html2pdf::layout::collect_positioned_boxes;

// Get all boxes that have been positioned
let boxes = collect_positioned_boxes(&layout_tree);
println!("Total boxes: {}", boxes.len());

for box_ in boxes {
    println!("Box at: ({}, {})", 
        box_.dimensions.content.x,
        box_.dimensions.content.y
    );
}
```

---

## PDF Generation

### Using PdfWriter

```rust
use html2pdf::pdf::{PdfWriter, PageContent};
use html2pdf::{PaperSize, Orientation, Margins};

// Create writer
let mut writer = PdfWriter::new();
writer.init_document();

// Set page properties
writer.set_paper_size(PaperSize::A4, Orientation::Portrait);
writer.set_margins(Margins::all(72.0));

// Add font
writer.add_standard_font("F1", "Helvetica");

// Create content
let mut content = PageContent::new();
content.begin_text();
content.set_font("F1", 12.0);
content.text_position(100.0, 700.0);
content.show_text("Hello, PDF!");
content.end_text();

// Add page
writer.add_page(content);

// Write output
let mut output = Vec::new();
writer.write(&mut output)?;

std::fs::write("output.pdf", output)?;
```

### Page Content Operations

```rust
use html2pdf::pdf::PageContent;

let mut content = PageContent::new();

// Text operations
content.begin_text();
content.set_font("F1", 12.0);
content.text_position(100.0, 700.0);
content.show_text("Hello World");
content.end_text();

// Graphics operations
content.set_stroke_color(1.0, 0.0, 0.0);  // Red
content.set_fill_color(0.0, 0.0, 1.0);    // Blue
content.draw_rect(100.0, 600.0, 200.0, 100.0);
content.fill();
content.stroke();

// Lines
content.move_to(50.0, 500.0);
content.line_to(300.0, 500.0);
content.stroke();
```

### Setting PDF Metadata

```rust
use html2pdf::pdf::PdfWriter;

let mut writer = PdfWriter::new();
writer.init_document();
writer.set_info(
    "Document Title",      // Title
    "PDF Subject",         // Subject
    "html2pdf-rs"          // Creator
);
```

---

## Advanced Configuration

### Custom Page Size

```rust
use html2pdf::{Config, PaperSize};

// Using custom paper size
let config = Config::default()
    .with_paper_size(PaperSize::Custom {
        width: 400.0,   // points
        height: 600.0,
    });

// Or using explicit dimensions (override paper size)
let mut config = Config::default();
config.page_width = Some(400.0);
config.page_height = Some(600.0);
```

### Adding User Stylesheets

```rust
use html2pdf::Config;

let mut config = Config::default();

// Add multiple stylesheets
config.user_stylesheets.push(r#"
    @page {
        margin: 2cm;
    }
    body {
        font-family: Georgia, serif;
    }
"#.to_string());

config.user_stylesheets.push(r#"
    h1 {
        color: #333;
    }
"#.to_string());
```

### Debug Mode

```rust
use html2pdf::Config;

let mut config = Config::default();
config.debug_layout = true;

// This will output layout debugging information
let pdf = html2pdf::html_to_pdf(html, &config)?;
```

---

## Custom Renderers

### Creating a Custom Renderer

```rust
use html2pdf::layout::{LayoutBox, BoxType, PdfBox};

fn custom_render(layout_tree: &LayoutBox) -> Vec<u8> {
    // Collect positioned boxes
    let boxes = html2pdf::layout::collect_positioned_boxes(layout_tree);
    
    // Convert to PDF boxes
    let pdf_boxes: Vec<PdfBox> = boxes.iter()
        .map(|b| PdfBox::from_layout_box(b))
        .collect();
    
    // Custom rendering logic
    let mut writer = html2pdf::pdf::PdfWriter::new();
    writer.init_document();
    
    // ... custom rendering code ...
    
    let mut output = Vec::new();
    writer.write(&mut output).unwrap();
    output
}
```

### Processing Layout Boxes

```rust
use html2pdf::layout::{LayoutBox, BoxType};

fn process_boxes(box_: &LayoutBox, depth: usize) {
    let indent = "  ".repeat(depth);
    
    match box_.box_type {
        BoxType::Block => {
            println!("{}Block box", indent);
        }
        BoxType::Inline => {
            println!("{}Inline box", indent);
        }
        BoxType::Text => {
            if let Some(ref text) = box_.text_content {
                println!("{}Text: '{}'", indent, text);
            }
        }
        BoxType::Anonymous => {
            println!("{}Anonymous box", indent);
        }
    }
    
    // Process children
    for child in &box_.children {
        process_boxes(child, depth + 1);
    }
}
```

### Custom Box Processing

```rust
use html2pdf::layout::{LayoutBox, collect_positioned_boxes};

fn find_text_boxes(layout_tree: &LayoutBox) -> Vec<&LayoutBox> {
    let mut text_boxes = Vec::new();
    
    fn collect_text_boxes<'a>(box_: &'a LayoutBox, result: &mut Vec<&'a LayoutBox>) {
        if box_.box_type == html2pdf::layout::BoxType::Text {
            result.push(box_);
        }
        
        for child in &box_.children {
            collect_text_boxes(child, result);
        }
    }
    
    collect_text_boxes(layout_tree, &mut text_boxes);
    text_boxes
}
```

---

## Examples

### Example 1: Batch Conversion

```rust
use html2pdf::{html_to_pdf_from_input, Input, Config, Result};
use std::path::Path;

fn batch_convert(input_dir: &str, output_dir: &str) -> Result<Vec<String>> {
    std::fs::create_dir_all(output_dir)?;
    
    let mut output_files = Vec::new();
    
    for entry in std::fs::read_dir(input_dir)? {
        let entry = entry?;
        let path = entry.path();
        
        if path.extension().map(|e| e == "html").unwrap_or(false) {
            let input = Input::File(path.to_string_lossy().to_string());
            let config = Config::default();
            
            let pdf = html_to_pdf_from_input(&input, &config)?;
            
            let output_name = path.file_stem().unwrap().to_string_lossy() + ".pdf";
            let output_path = Path::new(output_dir).join(&*output_name);
            
            std::fs::write(&output_path, pdf)?;
            output_files.push(output_path.to_string_lossy().to_string());
        }
    }
    
    Ok(output_files)
}
```

### Example 2: Report Generator

```rust
use html2pdf::{html_to_pdf, Config, PaperSize, Orientation, Margins, Result};

fn generate_report(title: &str, sections: &[(&str, &str)]) -> Result<Vec<u8>> {
    let mut html = String::new();
    
    html.push_str("<!DOCTYPE html><html><head><style>");
    html.push_str(r#"
        @page { margin: 2.5cm 2cm; }
        h1 { color: #333; break-after: avoid; }
        h2 { color: #666; break-after: avoid; }
        p { orphans: 2; widows: 2; }
    "#);
    html.push_str("</style></head><body>");
    
    html.push_str(&format!("<h1>{}</h1>", title));
    
    for (section_title, content) in sections {
        html.push_str(&format!("<h2>{}</h2>", section_title));
        html.push_str(&format!("<p>{}</p>", content));
    }
    
    html.push_str("</body></html>");
    
    let config = Config::default()
        .with_paper_size(PaperSize::A4)
        .with_orientation(Orientation::Portrait)
        .with_margins(Margins::all(72.0));
    
    html_to_pdf(&html, &config)
}
```

### Example 3: Template Engine Integration

```rust
use html2pdf::{html_to_pdf, Config, Result};

struct InvoiceData {
    invoice_number: String,
    date: String,
    customer: String,
    items: Vec<(String, f32)>,
}

fn generate_invoice(data: &InvoiceData) -> Result<Vec<u8>> {
    let html = format!(r#"<!DOCTYPE html>
<html>
<head>
    <style>
        @page {{ margin: 1.5cm; }}
        body {{ font-family: Arial, sans-serif; }}
        .header {{ text-align: right; }}
        table {{ width: 100%; border-collapse: collapse; }}
        th, td {{ padding: 0.5em; text-align: left; border-bottom: 1pt solid #ccc; }}
        .total {{ font-weight: bold; }}
    </style>
</head>
<body>
    <div class="header">
        <h1>Invoice {}</h1>
        <p>Date: {}</p>
    </div>
    <p>Bill To: {}</p>
    <table>
        <tr><th>Item</th><th>Amount</th></tr>
        {}
    </table>
</body>
</html>"#,
        data.invoice_number,
        data.date,
        data.customer,
        data.items.iter()
            .map(|(item, amount)| format!("<tr><td>{}</td><td>${:.2}</td></tr>", item, amount))
            .collect::<String>()
    );
    
    let config = Config::default();
    html_to_pdf(&html, &config)
}
```

### Example 4: Error Handling Wrapper

```rust
use html2pdf::{html_to_pdf, Config, Result, PdfError};
use std::fmt;

#[derive(Debug)]
enum ConversionError {
    Pdf(html2pdf::PdfError),
    Template(String),
    Validation(Vec<String>),
}

impl fmt::Display for ConversionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConversionError::Pdf(e) => write!(f, "PDF error: {}", e),
            ConversionError::Template(msg) => write!(f, "Template error: {}", msg),
            ConversionError::Validation(errors) => {
                write!(f, "Validation errors: {:?}", errors)
            }
        }
    }
}

impl std::error::Error for ConversionError {}

impl From<PdfError> for ConversionError {
    fn from(e: PdfError) -> Self {
        ConversionError::Pdf(e)
    }
}

fn safe_convert(html: &str, config: &Config) -> Result<Vec<u8>, ConversionError> {
    // Validate HTML
    if !html.contains("<html") {
        return Err(ConversionError::Validation(vec![
            "Missing <html> tag".to_string()
        ]));
    }
    
    // Convert
    let pdf = html_to_pdf(html, config)?;
    
    // Validate output
    if pdf.len() < 100 {
        return Err(ConversionError::Template(
            "Generated PDF is too small".to_string()
        ));
    }
    
    Ok(pdf)
}
```

### Example 5: Async Usage Pattern

```rust
use html2pdf::{html_to_pdf, Config};
use std::sync::Arc;

// Even though html2pdf is synchronous, you can use it in async contexts
async fn convert_async(html: String, config: Arc<Config>) -> Result<Vec<u8>, html2pdf::PdfError> {
    // Run CPU-intensive conversion in a blocking task
    tokio::task::spawn_blocking(move || {
        html2pdf::html_to_pdf(&html, &config)
    }).await.expect("Task panicked")
}

// Usage
// let config = Arc::new(Config::default());
// let pdf = convert_async(html, config).await?;
```

---

## Related Documentation

- [User Guide](USER_GUIDE.md) - Complete user guide and CLI usage
- [CSS Support Reference](CSS_SUPPORT.md) - CSS properties and selectors
- [PrintCSS Guide](PRINTCSS_GUIDE.md) - PrintCSS tutorial
- [README.md](../README.md) - Project overview

---

## API Stability

The html2pdf library follows semantic versioning. The current API is considered stable but may evolve as the library matures.

### Stability Levels

| Component | Stability | Notes |
|-----------|-----------|-------|
| `html_to_pdf` | Stable | Main conversion function |
| `Config` | Stable | Configuration struct |
| `Input` | Stable | Input source enum |
| `PdfError` | Stable | Error types |
| Layout modules | Evolving | May have breaking changes |
| PDF modules | Evolving | May have breaking changes |

---

## Contributing

Contributions to the library API are welcome! Please:

1. Follow the existing code style
2. Add documentation for new public APIs
3. Include examples in documentation
4. Write tests for new functionality

See the [README.md](../README.md) for contribution guidelines.
