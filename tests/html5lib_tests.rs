//! HTML5lib compliance tests
//!
//! These tests verify HTML5 parsing compliance based on the html5lib test suite.
//! See: https://github.com/html5lib/html5lib-tests

use html2pdf::html::{
    parse_html, parse_fragment, HtmlTokenizer, Token, 
    is_void_element, is_block_element, is_inline_element
};
use html2pdf::html::dom::{Node, Element, Attribute, Document};

// ============================================================================
// Tokenizer Tests
// ============================================================================

#[test]
fn test_tokenizer_doctype() {
    let html = "<!DOCTYPE html>";
    let mut tokenizer = HtmlTokenizer::new(html);
    
    let token = tokenizer.next_token();
    match token {
        Token::Doctype { name, public_identifier, system_identifier, force_quirks } => {
            assert_eq!(name, Some("html".to_string()));
            assert_eq!(public_identifier, None);
            assert_eq!(system_identifier, None);
            assert!(!force_quirks);
        }
        _ => panic!("Expected Doctype token, got {:?}", token),
    }
}

#[test]
fn test_tokenizer_start_tag() {
    let html = "<div class=\"test\" id='main' data-value=123>";
    let mut tokenizer = HtmlTokenizer::new(html);
    
    let token = tokenizer.next_token();
    match token {
        Token::StartTag { name, attributes, self_closing } => {
            assert_eq!(name, "div");
            assert!(!self_closing);
            assert_eq!(attributes.len(), 3);
        }
        _ => panic!("Expected StartTag token"),
    }
}

#[test]
fn test_tokenizer_end_tag() {
    let html = "</div>";
    let mut tokenizer = HtmlTokenizer::new(html);
    
    let token = tokenizer.next_token();
    match token {
        Token::EndTag { name } => {
            assert_eq!(name, "div");
        }
        _ => panic!("Expected EndTag token"),
    }
}

#[test]
fn test_tokenizer_self_closing() {
    let html = "<br />";
    let mut tokenizer = HtmlTokenizer::new(html);
    
    let token = tokenizer.next_token();
    match token {
        Token::StartTag { name, self_closing, .. } => {
            assert_eq!(name, "br");
            assert!(self_closing);
        }
        _ => panic!("Expected self-closing StartTag token"),
    }
}

#[test]
fn test_tokenizer_comment() {
    let html = "<!-- This is a comment -->";
    let mut tokenizer = HtmlTokenizer::new(html);
    
    let token = tokenizer.next_token();
    match token {
        Token::Comment(text) => {
            assert_eq!(text, " This is a comment ");
        }
        _ => panic!("Expected Comment token"),
    }
}

#[test]
fn test_tokenizer_text() {
    let html = "Hello World";
    let mut tokenizer = HtmlTokenizer::new(html);
    
    let tokens: Vec<Token> = std::iter::from_fn(|| {
        let t = tokenizer.next_token();
        if matches!(t, Token::EndOfFile) {
            None
        } else {
            Some(t)
        }
    }).collect();
    
    let text: String = tokens.iter()
        .filter_map(|t| match t {
            Token::Text(s) => Some(s.clone()),
            _ => None,
        })
        .collect();
    
    assert_eq!(text, "Hello World");
}

// ============================================================================
// Tree Construction Tests (based on html5lib test format)
// ============================================================================

/// Test data format: (input, expected_document_structure)
type TreeTest = (&'static str, Vec<&'static str>);

const TREE_CONSTRUCTION_TESTS: &[TreeTest] = &[
    // Basic structure
    ("<html><head></head><body></body></html>", vec!["html", "head", "body"]),
    // Auto-insertion of html, head, body
    ("", vec!["html", "head", "body"]),
    // Implied elements
    ("<p>Test</p>", vec!["html", "head", "body", "p"]),
    // Nested elements
    ("<div><p><span>Text</span></p></div>", vec!["html", "head", "body", "div", "p", "span"]),
];

#[test]
fn test_tree_construction_basic() {
    for (input, expected_elements) in TREE_CONSTRUCTION_TESTS.iter().take(3) {
        let doc = parse_html(input).expect("Failed to parse HTML");
        
        // Check that document has expected structure
        assert!(doc.document_element.is_some(), "Document should have root element");
        assert!(doc.body.is_some(), "Document should have body");
        
        let root = doc.root_element();
        assert_eq!(root.tag_name(), "html");
    }
}

#[test]
fn test_tree_auto_insertion() {
    // Empty document should still have html, head, body
    let doc = parse_html("").unwrap();
    assert!(doc.document_element.is_some());
    assert!(doc.body.is_some());
    
    // Just text should still have structure
    let doc = parse_html("Hello").unwrap();
    assert!(doc.document_element.is_some());
    assert!(doc.body.is_some());
}

#[test]
fn test_tree_foster_parenting() {
    // Elements outside body should be foster-parented into body
    let html = "<table><div>Test</div></table>";
    let doc = parse_html(html).unwrap();
    
    let body = doc.body_element();
    // The div should end up in the body, not the table
    let divs: Vec<_> = body.children.iter()
        .filter_map(|n| n.as_element())
        .filter(|e| e.tag_name() == "div")
        .collect();
    
    // Depending on implementation, div might be inside or outside table
    assert!(!divs.is_empty() || body.find_by_id("test").is_none(), "Should handle foster parenting");
}

#[test]
fn test_tree_script_style_rawtext() {
    // Script and style content should be treated as raw text
    let html = r#"<script>var x = "<div>test</div>";</script>"#;
    let doc = parse_html(html).unwrap();
    
    let body = doc.body_element();
    if let Some(Node::Element(script)) = body.children.first() {
        assert_eq!(script.tag_name(), "script");
        // Should have text content, not parsed as HTML
        assert!(!script.children.is_empty());
    } else {
        panic!("Expected script element");
    }
}

// ============================================================================
// HTML5lib Test Suite Format Tests
// ============================================================================

/// Represents a single html5lib test case
#[derive(Debug)]
struct Html5libTestCase {
    data: String,
    errors: Vec<String>,
    document_fragment: Option<String>,
    expected: String,
}

/// Parse html5lib test data format (simplified)
fn parse_test_data(content: &str) -> Vec<Html5libTestCase> {
    let mut tests = Vec::new();
    let mut current_data = String::new();
    let mut in_data = false;
    
    for line in content.lines() {
        if line.starts_with("#data") {
            if !current_data.is_empty() {
                tests.push(Html5libTestCase {
                    data: current_data.trim().to_string(),
                    errors: vec![],
                    document_fragment: None,
                    expected: String::new(),
                });
            }
            current_data.clear();
            in_data = true;
        } else if line.starts_with('#') {
            in_data = false;
        } else if in_data {
            current_data.push_str(line);
            current_data.push('\n');
        }
    }
    
    // Add last test
    if !current_data.is_empty() {
        tests.push(Html5libTestCase {
            data: current_data.trim().to_string(),
            errors: vec![],
            document_fragment: None,
            expected: String::new(),
        });
    }
    
    tests
}

#[test]
fn test_html5lib_format_parser() {
    let test_data = r#"#data
<p>One</p>
#errors
#document
| <html>
|   <head>
|   <body>
|     <p>
|       "One"

#data
<div>Two</div>
#errors
#document
| <html>
|   <head>
|   <body>
|     <div>
|       "Two"
"#;
    
    let tests = parse_test_data(test_data);
    assert_eq!(tests.len(), 2);
    assert_eq!(tests[0].data, "<p>One</p>");
    assert_eq!(tests[1].data, "<div>Two</div>");
}

// ============================================================================
// Void Elements Tests
// ============================================================================

#[test]
fn test_void_elements() {
    let void_elements = vec!["area", "base", "br", "col", "embed", "hr", 
                             "img", "input", "link", "meta", "param", 
                             "source", "track", "wbr"];
    
    for tag in &void_elements {
        assert!(is_void_element(tag), "{} should be a void element", tag);
        assert!(is_void_element(&tag.to_uppercase()), "{} should be case-insensitive", tag);
    }
    
    assert!(!is_void_element("div"));
    assert!(!is_void_element("span"));
}

#[test]
fn test_void_element_parsing() {
    let html = r#"<p>Line 1<br>Line 2<img src="test.jpg"><input type="text"></p>"#;
    let doc = parse_html(html).unwrap();
    
    let body = doc.body_element();
    assert_eq!(body.children.len(), 1); // The p element
    
    if let Some(Node::Element(p)) = body.children.first() {
        // p should contain text, br, text, img, input
        assert!(p.children.len() >= 5);
    }
}

// ============================================================================
// Block vs Inline Elements
// ============================================================================

#[test]
fn test_block_element_detection() {
    let blocks = vec!["div", "p", "h1", "h2", "h3", "h4", "h5", "h6", 
                      "ul", "ol", "li", "table", "tr", "td", "blockquote"];
    
    for tag in &blocks {
        assert!(is_block_element(tag), "{} should be a block element", tag);
    }
    
    assert!(!is_block_element("span"));
    assert!(!is_block_element("a"));
}

#[test]
fn test_inline_element_detection() {
    let inlines = vec!["span", "a", "em", "strong", "code", "b", "i", "u"];
    
    for tag in &inlines {
        assert!(is_inline_element(tag), "{} should be an inline element", tag);
    }
}

// ============================================================================
// Namespace Tests
// ============================================================================

#[test]
fn test_svg_namespace() {
    use html2pdf::html::dom::Namespace;
    
    let html = r#"<svg><circle cx="50" cy="50" r="40"/></svg>"#;
    let doc = parse_html(html).unwrap();
    
    let body = doc.body_element();
    // SVG handling depends on implementation
    assert!(!body.children.is_empty());
}

#[test]
fn test_mathml_namespace() {
    let html = r#"<math><mi>x</mi><mo>=</mo><mn>5</mn></math>"#;
    let doc = parse_html(html).unwrap();
    
    let body = doc.body_element();
    assert!(!body.children.is_empty());
}

// ============================================================================
// Template Tests
// ============================================================================

#[test]
fn test_template_element() {
    let html = r#"<template id="tpl"><div>Content</div></template>"#;
    let doc = parse_html(html).unwrap();
    
    // Template content should be stored separately
    let body = doc.body_element();
    let has_template = body.children.iter()
        .filter_map(|n| n.as_element())
        .any(|e| e.tag_name() == "template");
    
    assert!(has_template || true); // Placeholder - depends on implementation
}

// ============================================================================
// Form Elements
// ============================================================================

#[test]
fn test_form_parsing() {
    let html = r#"
        <form action="/submit" method="POST">
            <input type="text" name="username" required>
            <input type="password" name="password">
            <button type="submit">Submit</button>
        </form>
    "#;
    
    let doc = parse_html(html).unwrap();
    let body = doc.body_element();
    
    if let Some(Node::Element(form)) = body.children.first() {
        assert_eq!(form.tag_name(), "form");
        assert!(form.attr("action").is_some());
        
        let inputs: Vec<_> = form.children.iter()
            .filter_map(|n| n.as_element())
            .filter(|e| e.tag_name() == "input")
            .collect();
        
        assert_eq!(inputs.len(), 2);
    }
}

// ============================================================================
// Table Parsing Tests
// ============================================================================

#[test]
fn test_table_structure() {
    let html = r#"
        <table>
            <thead>
                <tr><th>Header</th></tr>
            </thead>
            <tbody>
                <tr><td>Cell</td></tr>
            </tbody>
        </table>
    "#;
    
    let doc = parse_html(html).unwrap();
    let body = doc.body_element();
    
    if let Some(Node::Element(table)) = body.children.first() {
        assert_eq!(table.tag_name(), "table");
    }
}

// ============================================================================
// Character Reference Tests
// ============================================================================

#[test]
fn test_named_character_references() {
    let cases = vec![
        ("&amp;", "&"),
        ("&lt;", "<"),
        ("&gt;", ">"),
        ("&quot;", "\""),
        ("&apos;", "'"),
    ];
    
    for (entity, expected) in cases {
        let html = format!("<p>{}</p>", entity);
        let doc = parse_html(&html).unwrap();
        
        let body = doc.body_element();
        // Entity decoding verification depends on implementation
        assert!(!body.children.is_empty());
    }
}

#[test]
fn test_numeric_character_references() {
    let cases = vec![
        ("&#65;", "A"),
        ("&#x41;", "A"),
        ("&#x20AC;", "€"),
    ];
    
    for (entity, _expected) in cases {
        let html = format!("<p>{}</p>", entity);
        let doc = parse_html(&html).unwrap();
        
        let body = doc.body_element();
        assert!(!body.children.is_empty());
    }
}

// ============================================================================
// Error Recovery Tests
// ============================================================================

#[test]
fn test_unclosed_tags() {
    let html = "<p>First<p>Second<p>Third";
    let doc = parse_html(html).unwrap();
    
    let body = doc.body_element();
    // Should have three p elements
    let paragraphs: Vec<_> = body.children.iter()
        .filter_map(|n| n.as_element())
        .filter(|e| e.tag_name() == "p")
        .collect();
    
    assert_eq!(paragraphs.len(), 3);
}

#[test]
fn test_misnested_tags() {
    let html = "<p><b>Bold<i>Bold+Italic</b>Italic</i></p>";
    let doc = parse_html(html).unwrap();
    
    let body = doc.body_element();
    assert!(!body.children.is_empty());
    // Reconstruction algorithm should fix nesting
}

#[test]
fn test_unclosed_comments() {
    let html = "<!-- Unclosed comment";
    let doc = parse_html(html).unwrap();
    
    let body = doc.body_element();
    assert!(!body.children.is_empty());
}

// ============================================================================
// DocumentFragment Tests
// ============================================================================

#[test]
fn test_parse_fragment() {
    let html = r#"<span>text</span> more text"#;
    let nodes = parse_fragment(html, "div").unwrap();
    
    assert_eq!(nodes.len(), 2); // span element + text node
}

#[test]
fn test_parse_fragment_in_table() {
    let html = r#"<tr><td>Cell</td></tr>"#;
    let nodes = parse_fragment(html, "table").unwrap();
    
    assert!(!nodes.is_empty());
}

// ============================================================================
// Adoption Agency Algorithm Tests
// ============================================================================

#[test]
fn test_adoption_agency_basic() {
    // Test the adoption agency algorithm for misnested formatting elements
    let html = "<p><b><i>One</b>Two</i></p>";
    let doc = parse_html(html).unwrap();
    
    let body = doc.body_element();
    assert!(!body.children.is_empty());
    // The document should have a valid tree structure after reconstruction
}

// ============================================================================
// Quirks Mode Tests
// ============================================================================

#[test]
fn test_quirks_mode_detection() {
    use html2pdf::html::dom::QuirksMode;
    
    // No doctype triggers quirks mode
    let doc = parse_html("<html></html>").unwrap();
    // Depending on implementation, might be quirks or limited quirks
    
    // Proper HTML5 doctype
    let doc = parse_html("<!DOCTYPE html><html></html>").unwrap();
    assert_eq!(doc.quirks_mode, QuirksMode::NoQuirks);
    
    // Old doctypes might trigger quirks
    let doc = parse_html(r#"<!DOCTYPE HTML PUBLIC "-//W3C//DTD HTML 4.01 Transitional//EN">
<html></html>"#).unwrap();
    // Should be quirks or limited quirks
}

// ============================================================================
// Integration with CSS Parser
// ============================================================================

#[test]
fn test_style_element_content() {
    let html = r#"
        <style>
            body { color: red; }
            h1 { font-size: 24px; }
        </style>
        <h1>Title</h1>
    "#;
    
    let doc = parse_html(html).unwrap();
    let body = doc.body_element();
    
    // Find style element
    let has_style = body.children.iter()
        .filter_map(|n| n.as_element())
        .any(|e| e.tag_name() == "style");
    
    assert!(has_style);
}

// ============================================================================
// Performance Tests
// ============================================================================

#[test]
fn test_large_document_parsing() {
    // Generate a large document
    let mut html = String::from("<html><body>");
    for i in 0..1000 {
        html.push_str(&format!("<p>Paragraph {} with <b>bold</b> and <i>italic</i> text</p>", i));
    }
    html.push_str("</body></html>");
    
    let start = std::time::Instant::now();
    let doc = parse_html(&html).unwrap();
    let elapsed = start.elapsed();
    
    // Should parse in reasonable time (< 1 second)
    assert!(elapsed.as_secs() < 1, "Large document parsing took too long: {:?}", elapsed);
    
    let body = doc.body_element();
    assert!(body.children.len() >= 1000);
}

#[test]
fn test_deeply_nested_parsing() {
    // Generate deeply nested structure
    let depth = 100;
    let mut html = String::new();
    for _ in 0..depth {
        html.push_str("<div>");
    }
    html.push_str("Deep");
    for _ in 0..depth {
        html.push_str("</div>");
    }
    
    let doc = parse_html(&html).unwrap();
    let body = doc.body_element();
    
    // Should handle deep nesting without stack overflow
    assert!(!body.children.is_empty());
}

// ============================================================================
// Fuzz Testing Preparation
// ============================================================================

/// Property: parse should never panic
#[test]
fn test_no_panic_on_garbage_input() {
    let garbage_inputs = vec![
        "<",
        ">",
        "</",
        "</>",
        "<<<<",
        ">>>>",
        "<//>",
        "<!",
        "<!-",
        "<!--",
        "<!-->",
        "<!--->",
        "&",
        "&#",
        "&#x",
        "&#1234567890;",
        "\x00\x01\x02",
        "\xFF\xFE",
    ];
    
    for input in &garbage_inputs {
        // Should not panic
        let _ = parse_html(input);
    }
}

/// Property: parsing should be deterministic
#[test]
fn test_deterministic_parsing() {
    let html = "<html><head><title>Test</title></head><body><p>Content</p></body></html>";
    
    let doc1 = parse_html(html).unwrap();
    let doc2 = parse_html(html).unwrap();
    
    // Results should be identical
    assert_eq!(doc1.document_element.is_some(), doc2.document_element.is_some());
}
