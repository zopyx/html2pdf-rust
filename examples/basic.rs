//! Basic HTML to PDF Conversion Example
//!
//! This example demonstrates the simplest way to convert HTML to PDF using
//! the html2pdf library. It covers:
//! - Converting an HTML string to PDF
//! - Using the Config struct to customize output
//! - Proper error handling
//! - Writing the PDF to a file
//!
//! Run with: cargo run --example basic

use html2pdf::{html_to_pdf, Config, Input, Margins, Orientation, PaperSize};
use std::fs;

fn main() {
    println!("=== HTML2PDF Basic Example ===\n");

    // Example 1: Simple HTML to PDF conversion with default settings
    match example_simple_conversion() {
        Ok(path) => println!("✓ Example 1 complete: PDF saved to {}\n", path),
        Err(e) => eprintln!("✗ Example 1 failed: {}\n", e),
    }

    // Example 2: Custom paper size and orientation
    match example_custom_page_settings() {
        Ok(path) => println!("✓ Example 2 complete: PDF saved to {}\n", path),
        Err(e) => eprintln!("✗ Example 2 failed: {}\n", e),
    }

    // Example 3: Custom margins and custom CSS
    match example_custom_styling() {
        Ok(path) => println!("✓ Example 3 complete: PDF saved to {}\n", path),
        Err(e) => eprintln!("✗ Example 3 failed: {}\n", e),
    }

    // Example 4: Using Input enum for different sources
    match example_input_enum() {
        Ok(path) => println!("✓ Example 4 complete: PDF saved to {}\n", path),
        Err(e) => eprintln!("✗ Example 4 failed: {}\n", e),
    }

    // Example 5: Error handling demonstration
    example_error_handling();

    println!("\n=== All Examples Complete ===");
}

/// Example 1: Simple HTML to PDF with default configuration
fn example_simple_conversion() -> Result<String, Box<dyn std::error::Error>> {
    println!("Example 1: Simple HTML to PDF Conversion");
    println!("------------------------------------------");

    // Simple HTML content
    let html = r#"<!DOCTYPE html>
<html>
<head>
    <title>Simple Example</title>
</head>
<body>
    <h1>Hello, PDF!</h1>
    <p>This is a simple HTML document converted to PDF.</p>
    <ul>
        <li>Easy to use</li>
        <li>Fast conversion</li>
        <li>High quality output</li>
    </ul>
</body>
</html>"#;

    // Use default configuration (A4, Portrait, 72pt margins)
    let config = Config::default();
    println!("  Using default config: A4 Portrait, 72pt margins");

    // Convert HTML to PDF
    let pdf_bytes = html_to_pdf(html, &config)?;
    println!("  Generated PDF: {} bytes", pdf_bytes.len());

    // Save to file
    let output_path = "examples/output_basic_simple.pdf";
    fs::write(output_path, pdf_bytes)?;

    Ok(output_path.to_string())
}

/// Example 2: Custom paper size and orientation
fn example_custom_page_settings() -> Result<String, Box<dyn std::error::Error>> {
    println!("\nExample 2: Custom Page Settings");
    println!("---------------------------------");

    let html = r#"<!DOCTYPE html>
<html>
<head>
    <title>Landscape Document</title>
    <style>
        body { font-family: Arial, sans-serif; padding: 20px; }
        h1 { color: #2c3e50; }
        .landscape-note { 
            background: #e3f2fd; 
            padding: 15px; 
            border-left: 4px solid #2196f3;
            margin: 20px 0;
        }
    </style>
</head>
<body>
    <h1>Letter Landscape Document</h1>
    <div class="landscape-note">
        <strong>Note:</strong> This PDF uses US Letter size in landscape orientation.
    </div>
    <p>Landscape orientation is perfect for wide tables, charts, and presentations.</p>
</body>
</html>"#;

    // Create custom configuration with Letter size and Landscape orientation
    let config = Config::default()
        .with_paper_size(PaperSize::Letter)
        .with_orientation(Orientation::Landscape);

    println!("  Using: Letter size, Landscape orientation");
    println!("  Letter size in landscape: 792pt x 612pt");

    let pdf_bytes = html_to_pdf(html, &config)?;
    println!("  Generated PDF: {} bytes", pdf_bytes.len());

    let output_path = "examples/output_basic_landscape.pdf";
    fs::write(output_path, pdf_bytes)?;

    Ok(output_path.to_string())
}

/// Example 3: Custom margins and user stylesheets
fn example_custom_styling() -> Result<String, Box<dyn std::error::Error>> {
    println!("\nExample 3: Custom Margins and Styling");
    println!("---------------------------------------");

    let html = r#"<!DOCTYPE html>
<html>
<head>
    <title>Custom Styled Document</title>
</head>
<body>
    <h1>Custom Margins & Styling</h1>
    <p>This document has narrow margins (36pt = 0.5 inch on each side).</p>
    <p class="highlight">The custom CSS adds special styling to elements.</p>
    <table>
        <tr>
            <th>Setting</th>
            <th>Value</th>
        </tr>
        <tr>
            <td>Top Margin</td>
            <td>36pt</td>
        </tr>
        <tr>
            <td>Right Margin</td>
            <td>36pt</td>
        </tr>
        <tr>
            <td>Bottom Margin</td>
            <td>36pt</td>
        </tr>
        <tr>
            <td>Left Margin</td>
            <td>36pt</td>
        </tr>
    </table>
</body>
</html>"#;

    // Create configuration with custom margins
    // 36pt = 0.5 inch on all sides
    let config = Config::default()
        .with_paper_size(PaperSize::A4)
        .with_margins(Margins::all(36.0));

    println!("  Using: A4 size, 36pt margins on all sides");

    let pdf_bytes = html_to_pdf(html, &config)?;
    println!("  Generated PDF: {} bytes", pdf_bytes.len());

    let output_path = "examples/output_basic_narrow_margins.pdf";
    fs::write(output_path, pdf_bytes)?;

    Ok(output_path.to_string())
}

/// Example 4: Using the Input enum for different sources
fn example_input_enum() -> Result<String, Box<dyn std::error::Error>> {
    println!("\nExample 4: Using Input Enum");
    println!("-----------------------------");

    // The Input enum allows you to specify HTML from different sources
    
    // Input::Html - from a string
    let input = Input::Html(r#"<h1>From HTML String</h1><p>Created using Input::Html</p>"#.to_string());
    println!("  Input type: {}", input.description());

    // Convert using the high-level API with Input
    let config = Config::default();
    let pdf_bytes = html2pdf::html_to_pdf_from_input(&input, &config)?;
    println!("  Generated PDF: {} bytes", pdf_bytes.len());

    let output_path = "examples/output_basic_input_enum.pdf";
    fs::write(output_path, pdf_bytes)?;

    // You can also use Input::File for reading from files:
    // let input = Input::File("document.html".to_string());
    
    // Or Input::Url for fetching from URLs (requires network):
    // let input = Input::Url("https://example.com/page.html".to_string());

    Ok(output_path.to_string())
}

/// Example 5: Demonstrating proper error handling
fn example_error_handling() {
    println!("\nExample 5: Error Handling");
    println!("---------------------------");

    // Example of handling errors properly
    match demonstrate_error_handling() {
        Ok(_) => println!("  Unexpected: Should have failed"),
        Err(e) => {
            println!("  ✓ Caught expected error: {}", e);
            println!("  Error type: {:?}", e);
        }
    }
}

fn demonstrate_error_handling() -> html2pdf::Result<()> {
    // Valid HTML conversion - should succeed
    let valid_html = "<p>Valid HTML</p>";
    let config = Config::default();
    let _pdf = html2pdf::html_to_pdf(valid_html, &config)?;
    println!("  ✓ Valid HTML converted successfully");

    // The library handles various error cases:
    // - PdfError::Io - File or network errors
    // - PdfError::Parse - HTML/CSS parsing errors
    // - PdfError::Layout - Layout computation errors
    // - PdfError::Font - Font-related errors
    // - PdfError::Image - Image processing errors

    // This is just a demonstration - all our examples use valid input
    Ok(())
}
