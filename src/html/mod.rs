//! HTML5 parser and DOM implementation
//!
//! Implements a complete HTML5 parser following the WHATWG specification

mod tokenizer;
mod tree_builder;
mod dom;

pub use tokenizer::{HtmlTokenizer, Token};
pub use tree_builder::TreeBuilder;
pub use dom::{
    Document, Element, Node, Attribute, 
    Comment, TextNode, DocumentType,
    QuirksMode, Namespace,
};

/// Node type enum
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NodeType {
    Document,
    DocumentType,
    Element,
    Text,
    Comment,
    ProcessingInstruction,
}

use crate::types::Result;

/// Parse HTML string into a Document
pub fn parse_html(input: &str) -> Result<Document> {
    let mut tokenizer = HtmlTokenizer::new(input);
    let mut tree_builder = TreeBuilder::new();
    
    loop {
        let token = tokenizer.next_token();
        let done = matches!(token, Token::EndOfFile);
        
        tree_builder.process_token(token);
        
        if done {
            break;
        }
    }
    
    Ok(tree_builder.document())
}

/// Parse HTML fragment with a given context element
pub fn parse_fragment(input: &str, context_element: &str) -> Result<Vec<Node>> {
    let mut tokenizer = HtmlTokenizer::new(input);
    let mut nodes = Vec::new();
    
    loop {
        let token = tokenizer.next_token();
        
        match token {
            Token::EndOfFile => break,
            Token::Text(text) => {
                nodes.push(Node::Text(TextNode::new(&text)));
            }
            Token::StartTag { name, attributes, .. } => {
                let element = Element::new(&name, attributes);
                nodes.push(Node::Element(element));
            }
            Token::Comment(text) => {
                nodes.push(Node::Comment(Comment::new(&text)));
            }
            _ => {}
        }
    }
    
    Ok(nodes)
}

/// HTML5 void elements (self-closing)
pub const VOID_ELEMENTS: &[&str] = &[
    "area", "base", "br", "col", "embed", "hr", "img", "input",
    "link", "meta", "param", "source", "track", "wbr",
];

/// Raw text elements (contents parsed as raw text)
pub const RAW_TEXT_ELEMENTS: &[&str] = &["script", "style"];

/// Check if an element is a void element
pub fn is_void_element(tag_name: &str) -> bool {
    VOID_ELEMENTS.contains(&tag_name.to_ascii_lowercase().as_str())
}

/// Check if an element is a raw text element
pub fn is_raw_text_element(tag_name: &str) -> bool {
    RAW_TEXT_ELEMENTS.contains(&tag_name.to_ascii_lowercase().as_str())
}

/// HTML5 block-level elements (for default styling)
pub const BLOCK_ELEMENTS: &[&str] = &[
    "address", "article", "aside", "blockquote", "body", "br", "button",
    "canvas", "caption", "col", "colgroup", "dd", "div", "dl", "dt",
    "embed", "fieldset", "figcaption", "figure", "footer", "form",
    "h1", "h2", "h3", "h4", "h5", "h6", "header", "hgroup", "hr",
    "html", "iframe", "li", "main", "map", "nav", "noscript", "object",
    "ol", "output", "p", "picture", "pre", "progress", "ruby", "section",
    "table", "tbody", "td", "tfoot", "th", "thead", "tr", "ul", "video",
];

/// Check if element is block-level by default
pub fn is_block_element(tag_name: &str) -> bool {
    BLOCK_ELEMENTS.contains(&tag_name.to_ascii_lowercase().as_str())
}

/// Inline elements (for default styling)
pub const INLINE_ELEMENTS: &[&str] = &[
    "a", "abbr", "acronym", "b", "bdi", "bdo", "big", "cite", "code",
    "dfn", "em", "i", "img", "input", "kbd", "label", "mark", "q",
    "samp", "select", "small", "span", "strong", "sub", "sup", "textarea",
    "time", "tt", "var",
];

/// Check if element is inline by default
pub fn is_inline_element(tag_name: &str) -> bool {
    INLINE_ELEMENTS.contains(&tag_name.to_ascii_lowercase().as_str())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_html() {
        let html = r#"<!DOCTYPE html>
<html>
<head><title>Test</title></head>
<body>
<h1>Hello</h1>
<p>World</p>
</body>
</html>"#;

        let doc = parse_html(html).unwrap();
        let root = doc.root_element();
        assert_eq!(root.tag_name(), "html");
    }

    #[test]
    fn test_parse_void_elements() {
        let html = r#"<p>Line 1<br>Line 2<img src="test.jpg"><input type="text"></p>"#;
        
        let doc = parse_html(html).unwrap();
        // Should parse without errors
        let body = doc.body_element();
        assert!(body.children().len() > 0);
    }

    #[test]
    fn test_element_checks() {
        assert!(is_void_element("br"));
        assert!(is_void_element("BR"));
        assert!(!is_void_element("div"));
        
        assert!(is_block_element("div"));
        assert!(is_block_element("p"));
        assert!(is_inline_element("span"));
        assert!(is_inline_element("a"));
    }

    #[test]
    fn test_parse_attributes() {
        let html = r#"<div id="test" class="foo bar" data-value="123">content</div>"#;
        
        let doc = parse_html(html).unwrap();
        if let Some(Node::Element(element)) = doc.body_element().children().first() {
            assert_eq!(element.id(), Some("test"));
            assert!(element.has_class("foo"));
            assert!(element.has_class("bar"));
            assert_eq!(element.attr("data-value"), Some("123"));
        } else {
            panic!("Expected element node");
        }
    }

    #[test]
    fn test_parse_nested_elements() {
        let html = r#"<div><p><strong>Bold</strong> and <em>italic</em></p></div>"#;
        
        let doc = parse_html(html).unwrap();
        let body = doc.body_element();
        assert_eq!(body.children().len(), 1);
        
        if let Some(Node::Element(div)) = body.children().first() {
            assert_eq!(div.tag_name(), "div");
            assert_eq!(div.children().len(), 1);
        }
    }

    #[test]
    fn test_parse_special_characters() {
        let html = r#"<p>&lt;test&gt; &amp; &quot;quote&quot;</p>"#;
        
        let doc = parse_html(html).unwrap();
        // HTML entities should be decoded during parsing
        // This test verifies the structure is correct
        let body = doc.body_element();
        assert_eq!(body.children().len(), 1);
    }

    #[test]
    fn test_parse_comments() {
        let html = r#"<!-- This is a comment --><p>After comment</p>"#;
        
        let doc = parse_html(html).unwrap();
        // Comments should be preserved in the DOM
        let body = doc.body_element();
        // First child might be comment or p depending on implementation
        assert!(body.children().len() >= 1);
    }

    #[test]
    fn test_parse_unclosed_tags() {
        let html = r#"<p>Paragraph 1<p>Paragraph 2"#;
        
        let doc = parse_html(html).unwrap();
        // Unclosed p tags should auto-close before new p tag
        let body = doc.body_element();
        assert_eq!(body.children().len(), 2);
    }

    #[test]
    fn test_parse_self_closing() {
        let html = r#"<br /><hr /><img src="test.jpg" />"#;
        
        let doc = parse_html(html).unwrap();
        // Self-closing syntax should be parsed correctly
        let body = doc.body_element();
        assert_eq!(body.children().len(), 3);
    }

    #[test]
    fn test_parse_script_content() {
        let html = r#"<script>var x = "<div>test</div>";</script>"#;
        
        let doc = parse_html(html).unwrap();
        // Script content should be parsed as raw text
        let body = doc.body_element();
        assert_eq!(body.children().len(), 1);
        
        if let Some(Node::Element(script)) = body.children().first() {
            assert_eq!(script.tag_name(), "script");
            assert!(script.children().len() > 0);
        }
    }

    #[test]
    fn test_parse_fragment() {
        let html = r#"<span>text</span> more text"#;
        let nodes = parse_fragment(html, "div").unwrap();
        
        assert_eq!(nodes.len(), 2); // span element + text node
    }
}
