//! Common test utilities for HTML2PDF
//!
//! This module provides shared testing functionality across all test files.

use std::fs;
use std::path::Path;

/// Assert that a byte slice is a valid PDF
pub fn assert_valid_pdf(pdf: &[u8]) {
    assert!(
        pdf.starts_with(b"%PDF"),
        "PDF should start with %PDF header, got: {:?}",
        &pdf[..pdf.len().min(10)]
    );
    
    let has_eof = pdf.windows(5).any(|w| w == b"%%EOF");
    assert!(has_eof, "PDF should contain %%EOF marker");
    
    assert!(
        pdf.len() > 100,
        "PDF should have reasonable size, got {} bytes",
        pdf.len()
    );
}

/// Load a fixture file from the fixtures directory
pub fn load_fixture(name: &str) -> String {
    let path = format!("tests/fixtures/{}", name);
    fs::read_to_string(&path).unwrap_or_else(|e| {
        panic!("Failed to load fixture '{}': {}", name, e)
    })
}

/// Load an expected output file
pub fn load_expected(name: &str) -> Vec<u8> {
    let path = format!("tests/expected/{}", name);
    fs::read(&path).unwrap_or_else(|e| {
        panic!("Failed to load expected output '{}': {}", name, e)
    })
}

/// Check if a test resource exists
pub fn resource_exists(path: &str) -> bool {
    Path::new(path).exists()
}

/// Create a temporary file for test output
pub fn temp_output_path() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis();
    
    format!("/tmp/html2pdf_test_{}.pdf", timestamp)
}

/// Compare two PDFs for similarity (not exact byte equality)
pub fn pdfs_similar(pdf1: &[u8], pdf2: &[u8]) -> bool {
    // Both should be valid PDFs
    if !pdf1.starts_with(b"%PDF") || !pdf2.starts_with(b"%PDF") {
        return false;
    }
    
    // Size should be similar (within 50% of each other)
    let size_diff = (pdf1.len() as f64 - pdf2.len() as f64).abs();
    let avg_size = (pdf1.len() + pdf2.len()) as f64 / 2.0;
    
    if size_diff / avg_size > 0.5 {
        return false;
    }
    
    // Check for key PDF structure elements
    let checks = vec![
        pdf1.windows(5).any(|w| w == b"/Type"),
        pdf1.windows(6).any(|w| w == b"/Pages"),
        pdf2.windows(5).any(|w| w == b"/Type"),
        pdf2.windows(6).any(|w| w == b"/Pages"),
    ];
    
    checks.iter().all(|&check| check)
}

/// Generate HTML document of specified complexity
pub fn generate_html_document(complexity: DocumentComplexity) -> String {
    match complexity {
        DocumentComplexity::Minimal => {
            "<p>Hello</p>".to_string()
        }
        DocumentComplexity::Simple => {
            r#"<!DOCTYPE html>
<html>
<head><title>Simple</title></head>
<body>
    <h1>Title</h1>
    <p>Content</p>
</body>
</html>"#.to_string()
        }
        DocumentComplexity::Medium => {
            let mut html = String::from(r#"<!DOCTYPE html>
<html>
<head><title>Medium</title></head>
<body>
    <h1>Document</h1>
"#);
            for i in 0..10 {
                html.push_str(&format!(
                    "<h2>Section {}</h2><p>Paragraph content.</p>\n",
                    i + 1
                ));
            }
            html.push_str("</body></html>");
            html
        }
        DocumentComplexity::Complex => {
            let mut html = String::from(r#"<!DOCTYPE html>
<html>
<head>
    <title>Complex</title>
    <style>
        body { font-family: Arial; }
        .header { background: #333; color: white; }
    </style>
</head>
<body>
    <header class="header"><h1>Complex Doc</h1></header>
    <main>
"#);
            for i in 0..50 {
                html.push_str(&format!(
                    r#"<section>
    <h2>Section {}</h2>
    <p>Content with <strong>formatting</strong> and <a href="#">links</a>.</p>
    <ul>
        <li>Item 1</li>
        <li>Item 2</li>
        <li>Item 3</li>
    </ul>
</section>
"#,
                    i + 1
                ));
            }
            html.push_str("</main></body></html>");
            html
        }
    }
}

/// Document complexity levels for test generation
pub enum DocumentComplexity {
    Minimal,
    Simple,
    Medium,
    Complex,
}

/// Timing helper for performance tests
pub struct Timer {
    start: std::time::Instant,
    name: String,
}

impl Timer {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            start: std::time::Instant::now(),
            name: name.into(),
        }
    }
    
    pub fn elapsed(&self) -> std::time::Duration {
        self.start.elapsed()
    }
    
    pub fn print_elapsed(&self) {
        println!("{} took: {:?}", self.name, self.elapsed());
    }
}

impl Drop for Timer {
    fn drop(&mut self) {
        // Optionally print timing information
    }
}

/// Approximate equality for floating point numbers
pub fn approx_eq(a: f32, b: f32, epsilon: f32) -> bool {
    (a - b).abs() < epsilon
}

/// Setup function for tests that need environment preparation
pub fn test_setup() {
    // Initialize tracing if needed
    let _ = tracing_subscriber::fmt()
        .with_env_filter("debug")
        .try_init();
}

/// Cleanup function for tests
pub fn test_cleanup() {
    // Cleanup temporary files, etc.
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_assert_valid_pdf_valid() {
        let valid_pdf = b"%PDF-1.4\n1 0 obj\n<<>>\nendobj\n%%EOF\n";
        assert_valid_pdf(valid_pdf);
    }
    
    #[test]
    #[should_panic(expected = "PDF should start with %PDF")]
    fn test_assert_valid_pdf_invalid() {
        let invalid_pdf = b"NOT A PDF";
        assert_valid_pdf(invalid_pdf);
    }
    
    #[test]
    fn test_pdfs_similar() {
        let pdf1 = b"%PDF-1.4\ncontent\n%%EOF";
        let pdf2 = b"%PDF-1.4\ndifferent content\n%%EOF";
        
        assert!(pdfs_similar(pdf1, pdf2));
    }
    
    #[test]
    fn test_generate_html() {
        let html = generate_html_document(DocumentComplexity::Simple);
        assert!(html.contains("<html>"));
        assert!(html.contains("<body>"));
        
        let complex = generate_html_document(DocumentComplexity::Complex);
        assert!(complex.len() > html.len());
    }
    
    #[test]
    fn test_approx_eq() {
        assert!(approx_eq(1.0, 1.0001, 0.001));
        assert!(!approx_eq(1.0, 1.1, 0.001));
    }
}
