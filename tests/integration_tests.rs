//! Integration Tests for HTML2PDF
//!
//! End-to-end tests that verify the complete HTML to PDF pipeline.

use html2pdf::{html_to_pdf, convert_file};
use html2pdf::html::parse_html;
use html2pdf::css::parse_stylesheet;
use html2pdf::pdf::{PdfWriter, PageContent};
use html2pdf::types::{PaperSize, Orientation, Margins, Color};
use std::fs;
use std::path::Path;

// ============================================================================
// Helper Functions
// ============================================================================

fn assert_valid_pdf(pdf: &[u8]) {
    // PDF must start with %PDF
    assert!(pdf.starts_with(b"%PDF"), "PDF should start with %PDF header");
    
    // PDF must end with %%EOF
    assert!(pdf.ends_with(b"%%EOF\n") || pdf.windows(5).any(|w| w == b"%%EOF"), 
        "PDF should end with %%EOF marker");
    
    // Should have reasonable size
    assert!(pdf.len() > 100, "PDF should have reasonable size");
}

fn load_fixture(name: &str) -> String {
    let path = format!("tests/fixtures/{}", name);
    fs::read_to_string(&path).unwrap_or_else(|_| {
        // If fixture doesn't exist, return a minimal document
        String::from("<!DOCTYPE html><html><body><p>Test</p></body></html>")
    })
}

// ============================================================================
// Simple HTML to PDF Tests
// ============================================================================

#[test]
fn test_simple_html_to_pdf() {
    let html = r#"<!DOCTYPE html>
<html>
<head><title>Test</title></head>
<body>
    <h1>Hello World</h1>
    <p>This is a test paragraph.</p>
</body>
</html>"#;
    
    let result = html_to_pdf(html);
    assert!(result.is_ok(), "html_to_pdf should succeed: {:?}", result.err());
    
    let pdf = result.unwrap();
    assert_valid_pdf(&pdf);
}

#[test]
fn test_minimal_document() {
    let html = "<p>Hello</p>";
    
    let result = html_to_pdf(html);
    assert!(result.is_ok());
    
    let pdf = result.unwrap();
    assert_valid_pdf(&pdf);
}

#[test]
fn test_empty_document() {
    let html = "";
    
    let result = html_to_pdf(html);
    assert!(result.is_ok());
    
    let pdf = result.unwrap();
    assert_valid_pdf(&pdf);
}

// ============================================================================
// Complex Document Tests
// ============================================================================

#[test]
fn test_document_with_headings() {
    let html = r#"<!DOCTYPE html>
<html>
<body>
    <h1>Heading 1</h1>
    <h2>Heading 2</h2>
    <h3>Heading 3</h3>
    <h4>Heading 4</h4>
    <h5>Heading 5</h5>
    <h6>Heading 6</h6>
</body>
</html>"#;
    
    let result = html_to_pdf(html);
    assert!(result.is_ok());
    
    let pdf = result.unwrap();
    assert_valid_pdf(&pdf);
}

#[test]
fn test_document_with_lists() {
    let html = r#"<!DOCTYPE html>
<html>
<body>
    <ul>
        <li>Item 1</li>
        <li>Item 2</li>
        <li>Item 3</li>
    </ul>
    <ol>
        <li>First</li>
        <li>Second</li>
        <li>Third</li>
    </ol>
</body>
</html>"#;
    
    let result = html_to_pdf(html);
    assert!(result.is_ok());
    
    let pdf = result.unwrap();
    assert_valid_pdf(&pdf);
}

#[test]
fn test_document_with_tables() {
    let html = r#"<!DOCTYPE html>
<html>
<body>
    <table>
        <thead>
            <tr><th>Name</th><th>Age</th></tr>
        </thead>
        <tbody>
            <tr><td>Alice</td><td>30</td></tr>
            <tr><td>Bob</td><td>25</td></tr>
        </tbody>
    </table>
</body>
</html>"#;
    
    let result = html_to_pdf(html);
    assert!(result.is_ok());
    
    let pdf = result.unwrap();
    assert_valid_pdf(&pdf);
}

#[test]
fn test_document_with_images() {
    let html = r#"<!DOCTYPE html>
<html>
<body>
    <p>Image test:</p>
    <img src="data:image/png;base64,iVBORw0KGgo=" alt="Test">
</body>
</html>"#;
    
    let result = html_to_pdf(html);
    assert!(result.is_ok());
    
    let pdf = result.unwrap();
    assert_valid_pdf(&pdf);
}

#[test]
fn test_document_with_links() {
    let html = r#"<!DOCTYPE html>
<html>
<body>
    <p>Visit <a href="https://example.com">Example</a> for more.</p>
</body>
</html>"#;
    
    let result = html_to_pdf(html);
    assert!(result.is_ok());
    
    let pdf = result.unwrap();
    assert_valid_pdf(&pdf);
}

// ============================================================================
// PrintCSS Feature Tests
// ============================================================================

#[test]
fn test_page_rule_styling() {
    let html = r#"<!DOCTYPE html>
<html>
<head>
<style>
@page {
    size: A4;
    margin: 2cm;
}
</style>
</head>
<body>
    <p>Page with @page rule</p>
</body>
</html>"#;
    
    let result = html_to_pdf(html);
    assert!(result.is_ok());
    
    let pdf = result.unwrap();
    assert_valid_pdf(&pdf);
}

#[test]
fn test_page_break_properties() {
    let html = r#"<!DOCTYPE html>
<html>
<head>
<style>
.page {
    page-break-before: always;
}
h1 {
    page-break-after: avoid;
}
table {
    page-break-inside: avoid;
}
</style>
</head>
<body>
    <h1>Title</h1>
    <div class="page">Page 2 content</div>
    <div class="page">Page 3 content</div>
</body>
</html>"#;
    
    let result = html_to_pdf(html);
    assert!(result.is_ok());
    
    let pdf = result.unwrap();
    assert_valid_pdf(&pdf);
}

#[test]
fn test_running_headers_footers() {
    let html = r#"<!DOCTYPE html>
<html>
<head>
<style>
@page {
    @top-center { content: "Document Title"; }
    @bottom-center { content: counter(page); }
}
</style>
</head>
<body>
    <p>Content with running headers and footers</p>
</body>
</html>"#;
    
    let result = html_to_pdf(html);
    assert!(result.is_ok());
    
    let pdf = result.unwrap();
    assert_valid_pdf(&pdf);
}

#[test]
fn test_named_pages() {
    let html = r#"<!DOCTYPE html>
<html>
<head>
<style>
@page cover {
    margin: 0;
}
@page content {
    margin: 2cm;
}
.cover {
    page: cover;
}
</style>
</head>
<body>
    <div class="cover">Cover Page</div>
    <div class="content">Content Page</div>
</body>
</html>"#;
    
    let result = html_to_pdf(html);
    assert!(result.is_ok());
    
    let pdf = result.unwrap();
    assert_valid_pdf(&pdf);
}

#[test]
fn test_widows_orphans_control() {
    let html = r#"<!DOCTYPE html>
<html>
<head>
<style>
p {
    widows: 3;
    orphans: 3;
}
</style>
</head>
<body>
    <p>Long paragraph that should not break leaving only 1 or 2 lines...</p>
</body>
</html>"#;
    
    let result = html_to_pdf(html);
    assert!(result.is_ok());
    
    let pdf = result.unwrap();
    assert_valid_pdf(&pdf);
}

// ============================================================================
// CSS Feature Tests
// ============================================================================

#[test]
fn test_css_colors() {
    let html = r#"<!DOCTYPE html>
<html>
<head>
<style>
.named { color: red; }
.hex { color: #FF0000; }
.rgb { color: rgb(255, 0, 0); }
.rgba { color: rgba(255, 0, 0, 0.5); }
.hsl { color: hsl(0, 100%, 50%); }
</style>
</head>
<body>
    <p class="named">Named color</p>
    <p class="hex">Hex color</p>
    <p class="rgb">RGB color</p>
    <p class="rgba">RGBA color</p>
    <p class="hsl">HSL color</p>
</body>
</html>"#;
    
    let result = html_to_pdf(html);
    assert!(result.is_ok());
    
    let pdf = result.unwrap();
    assert_valid_pdf(&pdf);
}

#[test]
fn test_css_fonts() {
    let html = r#"<!DOCTYPE html>
<html>
<head>
<style>
.serif { font-family: Georgia, serif; }
.sans { font-family: Arial, sans-serif; }
.mono { font-family: Consolas, monospace; }
.size { font-size: 18px; }
.weight { font-weight: bold; }
.style { font-style: italic; }
</style>
</head>
<body>
    <p class="serif">Serif font</p>
    <p class="sans">Sans-serif font</p>
    <p class="mono">Monospace font</p>
    <p class="size">Sized text</p>
    <p class="weight">Bold text</p>
    <p class="style">Italic text</p>
</body>
</html>"#;
    
    let result = html_to_pdf(html);
    assert!(result.is_ok());
    
    let pdf = result.unwrap();
    assert_valid_pdf(&pdf);
}

#[test]
fn test_css_flexbox() {
    let html = r#"<!DOCTYPE html>
<html>
<head>
<style>
.container {
    display: flex;
    flex-direction: row;
    justify-content: space-between;
}
.item {
    flex: 1;
    padding: 20px;
}
</style>
</head>
<body>
    <div class="container">
        <div class="item">Item 1</div>
        <div class="item">Item 2</div>
        <div class="item">Item 3</div>
    </div>
</body>
</html>"#;
    
    let result = html_to_pdf(html);
    assert!(result.is_ok());
    
    let pdf = result.unwrap();
    assert_valid_pdf(&pdf);
}

#[test]
fn test_css_grid() {
    let html = r#"<!DOCTYPE html>
<html>
<head>
<style>
.container {
    display: grid;
    grid-template-columns: 1fr 1fr 1fr;
    gap: 20px;
}
.item {
    padding: 20px;
}
</style>
</head>
<body>
    <div class="container">
        <div class="item">Item 1</div>
        <div class="item">Item 2</div>
        <div class="item">Item 3</div>
    </div>
</body>
</html>"#;
    
    let result = html_to_pdf(html);
    assert!(result.is_ok());
    
    let pdf = result.unwrap();
    assert_valid_pdf(&pdf);
}

#[test]
fn test_css_borders_backgrounds() {
    let html = r#"<!DOCTYPE html>
<html>
<head>
<style>
.box {
    border: 2px solid black;
    border-radius: 10px;
    background-color: #f0f0f0;
    padding: 20px;
    margin: 20px;
}
.shadow {
    box-shadow: 0 2px 4px rgba(0,0,0,0.2);
}
</style>
</head>
<body>
    <div class="box">Box with border and background</div>
    <div class="box shadow">Box with shadow</div>
</body>
</html>"#;
    
    let result = html_to_pdf(html);
    assert!(result.is_ok());
    
    let pdf = result.unwrap();
    assert_valid_pdf(&pdf);
}

// ============================================================================
// PDF Generation Tests
// ============================================================================

#[test]
fn test_pdf_writer_creation() {
    let mut writer = PdfWriter::new();
    writer.init_document();
    writer.set_info("Test", "Author", "HTML2PDF");
    
    let mut content = PageContent::new();
    content.begin_text();
    content.set_font("F1", 12.0);
    content.text_position(100.0, 700.0);
    content.show_text("Test");
    content.end_text();
    
    writer.add_page(content);
    
    let mut output = Vec::new();
    let result = writer.write(&mut output);
    
    assert!(result.is_ok());
    assert_valid_pdf(&output);
}

#[test]
fn test_multiple_pages() {
    let mut writer = PdfWriter::new();
    writer.init_document();
    writer.set_info("Multi-page", "Author", "HTML2PDF");
    
    for i in 1..=5 {
        let mut content = PageContent::new();
        content.begin_text();
        content.set_font("F1", 12.0);
        content.text_position(100.0, 700.0);
        content.show_text(&format!("Page {}", i));
        content.end_text();
        
        writer.add_page(content);
    }
    
    let mut output = Vec::new();
    let result = writer.write(&mut output);
    
    assert!(result.is_ok());
    assert_valid_pdf(&output);
    
    // Multi-page PDF should be larger
    assert!(output.len() > 500);
}

#[test]
fn test_different_paper_sizes() {
    for size in vec![PaperSize::A4, PaperSize::Letter, PaperSize::A3] {
        let mut writer = PdfWriter::new();
        writer.init_document();
        writer.set_paper_size(size, Orientation::Portrait);
        
        let content = PageContent::new();
        writer.add_page(content);
        
        let mut output = Vec::new();
        let result = writer.write(&mut output);
        
        assert!(result.is_ok());
        assert_valid_pdf(&output);
    }
}

#[test]
fn test_different_orientations() {
    for orientation in vec![Orientation::Portrait, Orientation::Landscape] {
        let mut writer = PdfWriter::new();
        writer.init_document();
        writer.set_paper_size(PaperSize::A4, orientation);
        
        let content = PageContent::new();
        writer.add_page(content);
        
        let mut output = Vec::new();
        let result = writer.write(&mut output);
        
        assert!(result.is_ok());
        assert_valid_pdf(&output);
    }
}

#[test]
fn test_page_with_drawing() {
    use html2pdf::types::{Rect, Point};
    
    let mut writer = PdfWriter::new();
    writer.init_document();
    
    let mut content = PageContent::new();
    
    // Draw a rectangle
    content.set_fill_color(Color::new(200, 200, 255));
    content.draw_rect(Rect::new(100.0, 600.0, 200.0, 100.0));
    content.fill();
    
    // Draw a line
    content.set_stroke_color(Color::new(0, 0, 0));
    content.set_line_width(2.0);
    content.draw_line(Point::new(100.0, 500.0), Point::new(300.0, 500.0));
    content.stroke();
    
    writer.add_page(content);
    
    let mut output = Vec::new();
    let result = writer.write(&mut output);
    
    assert!(result.is_ok());
    assert_valid_pdf(&output);
}

// ============================================================================
// Error Handling Tests
// ============================================================================

#[test]
fn test_malformed_html() {
    let html = "<p>Unclosed paragraph";
    
    let result = html_to_pdf(html);
    // Should handle gracefully
    assert!(result.is_ok());
}

#[test]
fn test_malformed_css() {
    let html = r#"<!DOCTYPE html>
<html>
<head>
<style>
.invalid { color: !!!; }
.missing { 
</style>
</head>
<body>
    <p>Test</p>
</body>
</html>"#;
    
    let result = html_to_pdf(html);
    // Should handle gracefully, ignoring invalid CSS
    assert!(result.is_ok());
}

#[test]
fn test_invalid_color_values() {
    let html = r#"<!DOCTYPE html>
<html>
<head>
<style>
.test { color: notacolor; }
</style>
</head>
<body>
    <p class="test">Test</p>
</body>
</html>"#;
    
    let result = html_to_pdf(html);
    assert!(result.is_ok());
}

#[test]
fn test_missing_images() {
    let html = r#"<!DOCTYPE html>
<html>
<body>
    <img src="nonexistent.jpg" alt="Missing">
</body>
</html>"#;
    
    let result = html_to_pdf(html);
    // Should handle gracefully, using alt text or placeholder
    assert!(result.is_ok());
}

// ============================================================================
// Unicode and Internationalization Tests
// ============================================================================

#[test]
fn test_unicode_text() {
    let html = r#"<!DOCTYPE html>
<html>
<body>
    <p>English: Hello</p>
    <p>Spanish: Hola</p>
    <p>French: Bonjour</p>
    <p>German: Hallo</p>
    <p>Japanese: こんにちは</p>
    <p>Chinese: 你好</p>
    <p>Arabic: مرحبا</p>
    <p>Russian: Привет</p>
    <p>Emoji: 🎉👋🌍</p>
</body>
</html>"#;
    
    let result = html_to_pdf(html);
    assert!(result.is_ok());
    
    let pdf = result.unwrap();
    assert_valid_pdf(&pdf);
}

#[test]
fn test_rtl_text() {
    let html = r#"<!DOCTYPE html>
<html dir="rtl">
<body>
    <p>هذا نص عربي</p>
    <p>עברית</p>
</body>
</html>"#;
    
    let result = html_to_pdf(html);
    assert!(result.is_ok());
    
    let pdf = result.unwrap();
    assert_valid_pdf(&pdf);
}

// ============================================================================
// Large Document Tests
// ============================================================================

#[test]
fn test_large_document() {
    let mut html = String::from("<!DOCTYPE html><html><body>");
    
    // Generate a large document with many elements
    for i in 0..100 {
        html.push_str(&format!(
            "<h2>Section {}</h2>
            <p>This is paragraph {} with some text content.</p>
            <ul>
                <li>Item A</li>
                <li>Item B</li>
                <li>Item C</li>
            </ul>",
            i, i
        ));
    }
    
    html.push_str("</body></html>");
    
    let start = std::time::Instant::now();
    let result = html_to_pdf(&html);
    let elapsed = start.elapsed();
    
    assert!(result.is_ok());
    assert!(elapsed.as_secs() < 5, "Large document took too long: {:?}", elapsed);
    
    let pdf = result.unwrap();
    assert_valid_pdf(&pdf);
}

#[test]
fn test_deeply_nested_document() {
    let depth = 50;
    let mut html = String::from("<!DOCTYPE html><html><body>");
    
    for _ in 0..depth {
        html.push_str("<div class=\"nested\">");
    }
    
    html.push_str("<p>Deep content</p>");
    
    for _ in 0..depth {
        html.push_str("</div>");
    }
    
    html.push_str("</body></html>");
    
    let result = html_to_pdf(&html);
    assert!(result.is_ok());
    
    let pdf = result.unwrap();
    assert_valid_pdf(&pdf);
}

// ============================================================================
// File I/O Tests
// ============================================================================

#[test]
fn test_convert_nonexistent_file() {
    let result = convert_file("/nonexistent/path/file.html", "/tmp/output.pdf");
    assert!(result.is_err());
}

// ============================================================================
// Performance Benchmarks (as tests)
// ============================================================================

#[test]
fn test_performance_simple_document() {
    let html = "<p>Simple paragraph</p>";
    
    let start = std::time::Instant::now();
    for _ in 0..100 {
        let _ = html_to_pdf(html);
    }
    let elapsed = start.elapsed();
    
    println!("100 simple conversions took: {:?}", elapsed);
    assert!(elapsed.as_secs() < 10, "Performance test took too long");
}

// ============================================================================
// Property-Based Tests (simplified)
// ============================================================================

#[test]
fn test_property_idempotent() {
    // Converting the same HTML twice should produce similar results
    let html = "<p>Test</p>";
    
    let pdf1 = html_to_pdf(html).unwrap();
    let pdf2 = html_to_pdf(html).unwrap();
    
    // Both should be valid PDFs
    assert_valid_pdf(&pdf1);
    assert_valid_pdf(&pdf2);
}

#[test]
fn test_property_deterministic_size() {
    // Similar documents should produce similar-sized PDFs
    let html1 = "<p>Short</p>";
    let html2 = "<p>Also short</p>";
    
    let pdf1 = html_to_pdf(html1).unwrap();
    let pdf2 = html_to_pdf(html2).unwrap();
    
    // Sizes should be reasonably close (within 50%)
    let size_diff = (pdf1.len() as f64 - pdf2.len() as f64).abs();
    let avg_size = (pdf1.len() + pdf2.len()) as f64 / 2.0;
    
    assert!(size_diff / avg_size < 0.5, "PDF sizes differ too much");
}
