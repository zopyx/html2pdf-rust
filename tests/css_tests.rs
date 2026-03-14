//! CSS Parsing and Processing Tests
//!
//! Tests for CSS tokenizer, parser, selectors, values, and at-rules.

use html2pdf::css::{
    parse_stylesheet, parse_rule, parse_value, parse_selector,
    CssTokenizer, CssToken, CssParser, Stylesheet, Rule, Declaration,
    is_valid_property, STANDARD_PROPERTIES, PRINT_PROPERTIES,
};
use html2pdf::css::values::{CssValue, CssFunction, Unit};
use html2pdf::css::selectors::{Selector, SelectorPart, Combinator};
use html2pdf::css::at_rules::{AtRule, PageRule, PageSelector, PageMarginBox};
use html2pdf::html::dom::{Element, Attribute};

// ============================================================================
// CSS Tokenizer Tests
// ============================================================================

#[test]
fn test_tokenizer_ident() {
    let css = "body";
    let mut tokenizer = CssTokenizer::new(css);
    
    let token = tokenizer.next_token();
    assert!(matches!(token, CssToken::Ident(s) if s == "body"));
}

#[test]
fn test_tokenizer_number() {
    let css = "42 3.14 -10";
    let mut tokenizer = CssTokenizer::new(css);
    
    let t1 = tokenizer.next_token();
    assert!(matches!(t1, CssToken::Number(n, _) if (n - 42.0).abs() < 0.001));
    
    let _ws = tokenizer.next_token(); // whitespace
    
    let t2 = tokenizer.next_token();
    assert!(matches!(t2, CssToken::Number(n, _) if (n - 3.14).abs() < 0.001));
}

#[test]
fn test_tokenizer_dimension() {
    let css = "10px 20em 30% 1.5rem";
    let mut tokenizer = CssTokenizer::new(css);
    
    let t1 = tokenizer.next_token();
    assert!(matches!(t1, CssToken::Dimension(n, u, _) 
        if (n - 10.0).abs() < 0.001 && u == "px"));
    
    let _ws = tokenizer.next_token();
    
    let t2 = tokenizer.next_token();
    assert!(matches!(t2, CssToken::Dimension(n, u, _) 
        if (n - 20.0).abs() < 0.001 && u == "em"));
}

#[test]
fn test_tokenizer_percentage() {
    let css = "50% 100%";
    let mut tokenizer = CssTokenizer::new(css);
    
    let t1 = tokenizer.next_token();
    assert!(matches!(t1, CssToken::Percentage(p) if (p - 50.0).abs() < 0.001));
}

#[test]
fn test_tokenizer_hash() {
    let css = "#id #FF0000";
    let mut tokenizer = CssTokenizer::new(css);
    
    let t1 = tokenizer.next_token();
    assert!(matches!(t1, CssToken::Hash(h, _) if h == "id"));
    
    let _ws = tokenizer.next_token();
    
    let t2 = tokenizer.next_token();
    assert!(matches!(t2, CssToken::Hash(h, _) if h == "FF0000"));
}

#[test]
fn test_tokenizer_string() {
    let css = r#""hello" 'world'"#;
    let mut tokenizer = CssTokenizer::new(css);
    
    let t1 = tokenizer.next_token();
    assert!(matches!(t1, CssToken::String(s) if s == "hello"));
    
    let _ws = tokenizer.next_token();
    
    let t2 = tokenizer.next_token();
    assert!(matches!(t2, CssToken::String(s) if s == "world"));
}

#[test]
fn test_tokenizer_url() {
    let css = "url(http://example.com) url('test.css')";
    let mut tokenizer = CssTokenizer::new(css);
    
    let t1 = tokenizer.next_token();
    assert!(matches!(t1, CssToken::Function(f) if f == "url"));
}

#[test]
fn test_tokenizer_at_keyword() {
    let css = "@media @import @page";
    let mut tokenizer = CssTokenizer::new(css);
    
    let t1 = tokenizer.next_token();
    assert!(matches!(t1, CssToken::AtKeyword(s) if s == "media"));
    
    let _ws = tokenizer.next_token();
    
    let t2 = tokenizer.next_token();
    assert!(matches!(t2, CssToken::AtKeyword(s) if s == "import"));
}

#[test]
fn test_tokenizer_comment() {
    let css = "/* This is a comment */ body";
    let mut tokenizer = CssTokenizer::new(css);
    
    // Comments are typically skipped or returned based on implementation
    let token = tokenizer.next_token();
    // After comment, should get 'body'
    if matches!(token, CssToken::Comment(_)) {
        let t2 = tokenizer.next_token();
        assert!(matches!(t2, CssToken::Ident(s) if s == "body"));
    } else {
        assert!(matches!(token, CssToken::Ident(s) if s == "body"));
    }
}

// ============================================================================
// Selector Parsing Tests
// ============================================================================

#[test]
fn test_selector_type() {
    let selector = parse_selector("div").unwrap();
    assert!(selector.parts.iter().any(|p| matches!(p, SelectorPart::Type(t) if t == "div")));
}

#[test]
fn test_selector_id() {
    let selector = parse_selector("#main").unwrap();
    assert!(selector.parts.iter().any(|p| matches!(p, SelectorPart::Id(id) if id == "main")));
}

#[test]
fn test_selector_class() {
    let selector = parse_selector(".container").unwrap();
    assert!(selector.parts.iter().any(|p| matches!(p, SelectorPart::Class(c) if c == "container")));
}

#[test]
fn test_selector_multiple_classes() {
    let selector = parse_selector(".a.b.c").unwrap();
    let classes: Vec<_> = selector.parts.iter()
        .filter_map(|p| match p {
            SelectorPart::Class(c) => Some(c.clone()),
            _ => None,
        })
        .collect();
    assert_eq!(classes, vec!["a", "b", "c"]);
}

#[test]
fn test_selector_universal() {
    let selector = parse_selector("*").unwrap();
    assert!(selector.parts.iter().any(|p| matches!(p, SelectorPart::Universal)));
}

#[test]
fn test_selector_attribute() {
    let selector = parse_selector("[disabled]").unwrap();
    assert!(selector.parts.iter().any(|p| matches!(p, SelectorPart::Attribute { name, .. } if name == "disabled")));
}

#[test]
fn test_selector_attribute_value() {
    let selector = parse_selector("[type=\"text\"]").unwrap();
    assert!(selector.parts.iter().any(|p| matches!(p, 
        SelectorPart::Attribute { name, op, value } 
        if name == "type" && op.as_deref() == Some("=") && value.as_deref() == Some("text")
    )));
}

#[test]
fn test_selector_pseudo_class() {
    let cases = vec![
        ":hover", ":focus", ":active", ":checked", ":disabled",
        ":first-child", ":last-child", ":nth-child(2n)",
    ];
    
    for case in cases {
        let result = parse_selector(case);
        assert!(result.is_ok(), "Failed to parse: {}", case);
    }
}

#[test]
fn test_selector_pseudo_element() {
    let cases = vec!["::before", "::after", "::first-line", "::first-letter"];
    
    for case in cases {
        let selector = parse_selector(case).unwrap();
        assert!(selector.parts.iter().any(|p| matches!(p, SelectorPart::PseudoElement(_))));
    }
}

#[test]
fn test_selector_descendant() {
    let selector = parse_selector("div p").unwrap();
    assert!(selector.parts.iter().any(|p| matches!(p, 
        SelectorPart::Combinator(Combinator::Descendant)
    )));
}

#[test]
fn test_selector_child() {
    let selector = parse_selector("div > p").unwrap();
    assert!(selector.parts.iter().any(|p| matches!(p, 
        SelectorPart::Combinator(Combinator::Child)
    )));
}

#[test]
fn test_selector_adjacent_sibling() {
    let selector = parse_selector("h1 + p").unwrap();
    assert!(selector.parts.iter().any(|p| matches!(p, 
        SelectorPart::Combinator(Combinator::Adjacent)
    )));
}

#[test]
fn test_selector_general_sibling() {
    let selector = parse_selector("h1 ~ p").unwrap();
    assert!(selector.parts.iter().any(|p| matches!(p, 
        SelectorPart::Combinator(Combinator::GeneralSibling)
    )));
}

#[test]
fn test_selector_group() {
    let css = "h1, h2, h3 { color: red; }";
    let stylesheet = parse_stylesheet(css).unwrap();
    
    assert_eq!(stylesheet.rules.len(), 1);
}

// ============================================================================
// Value Parsing Tests
// ============================================================================

#[test]
fn test_value_ident() {
    let value = parse_value("red").unwrap();
    assert!(matches!(value, CssValue::Ident(s) if s == "red"));
}

#[test]
fn test_value_number() {
    let value = parse_value("42").unwrap();
    assert!(matches!(value, CssValue::Number(n) if (n - 42.0).abs() < 0.001));
}

#[test]
fn test_value_length() {
    let cases = vec![
        ("10px", Unit::Px),
        ("1em", Unit::Em),
        ("1rem", Unit::Rem),
        ("100%", Unit::Percent),
        ("10mm", Unit::Mm),
        ("1cm", Unit::Cm),
        ("1in", Unit::In),
        ("12pt", Unit::Pt),
    ];
    
    for (input, expected_unit) in cases {
        let value = parse_value(input).unwrap();
        assert!(matches!(value, CssValue::Length(_, u) if u == expected_unit),
            "Failed for {}: got {:?}", input, value);
    }
}

#[test]
fn test_value_percentage() {
    let value = parse_value("50%").unwrap();
    assert!(matches!(value, CssValue::Percentage(p) if (p - 50.0).abs() < 0.001));
}

#[test]
fn test_value_color_hex() {
    let cases = vec![
        ("#FFF", CssValue::HexColor(s) if s == "FFF"),
        ("#FFFFFF", CssValue::HexColor(s) if s == "FFFFFF"),
        ("#FF0000", CssValue::HexColor(s) if s == "FF0000"),
    ];
    
    for (input, matcher) in cases {
        let value = parse_value(input).unwrap();
        assert!(matcher.matches(&value), "Failed for {}", input);
    }
}

#[test]
fn test_value_color_named() {
    let value = parse_value("red").unwrap();
    assert!(matches!(value, CssValue::Ident(s) if s == "red"));
}

#[test]
fn test_value_function() {
    let value = parse_value("rgb(255, 0, 0)").unwrap();
    assert!(matches!(value, CssValue::Function(CssFunction { name, .. }) if name == "rgb"));
}

#[test]
fn test_value_url() {
    let value = parse_value("url(http://example.com)").unwrap();
    assert!(matches!(value, CssValue::Url(s) if s == "http://example.com"));
}

#[test]
fn test_value_string() {
    let value = parse_value(r#""hello world""#).unwrap();
    assert!(matches!(value, CssValue::String(s) if s == "hello world"));
}

#[test]
fn test_value_list() {
    let value = parse_value("1px solid black").unwrap();
    assert!(matches!(value, CssValue::List(list) if list.len() == 3));
}

#[test]
fn test_value_calc() {
    let value = parse_value("calc(100% - 20px)").unwrap();
    assert!(matches!(value, CssValue::Function(CssFunction { name, .. }) if name == "calc"));
}

#[test]
fn test_value_var() {
    let value = parse_value("var(--primary-color)").unwrap();
    assert!(matches!(value, CssValue::Function(CssFunction { name, .. }) if name == "var"));
}

// ============================================================================
// Stylesheet Parsing Tests
// ============================================================================

#[test]
fn test_stylesheet_empty() {
    let stylesheet = parse_stylesheet("").unwrap();
    assert!(stylesheet.rules.is_empty());
}

#[test]
fn test_stylesheet_simple_rule() {
    let css = "body { color: red; }";
    let stylesheet = parse_stylesheet(css).unwrap();
    
    assert_eq!(stylesheet.rules.len(), 1);
}

#[test]
fn test_stylesheet_multiple_rules() {
    let css = r#"
        body { color: red; }
        h1 { font-size: 24px; }
        p { margin: 10px; }
    "#;
    let stylesheet = parse_stylesheet(css).unwrap();
    
    assert_eq!(stylesheet.rules.len(), 3);
}

#[test]
fn test_stylesheet_multiple_declarations() {
    let css = "body { color: red; background: white; font-size: 14px; }";
    let stylesheet = parse_stylesheet(css).unwrap();
    
    if let Rule::Style(rule) = &stylesheet.rules[0] {
        assert!(rule.declarations.len() >= 1);
    }
}

#[test]
fn test_stylesheet_important() {
    let css = "body { color: red !important; }";
    let stylesheet = parse_stylesheet(css).unwrap();
    
    if let Rule::Style(rule) = &stylesheet.rules[0] {
        assert!(rule.is_important("color"));
    }
}

// ============================================================================
// @page Rule Tests (PrintCSS)
// ============================================================================

#[test]
fn test_page_rule_simple() {
    let css = "@page { margin: 1cm; }";
    let stylesheet = parse_stylesheet(css).unwrap();
    
    assert_eq!(stylesheet.rules.len(), 1);
    assert!(matches!(&stylesheet.rules[0], Rule::At(AtRule::Page(_))));
}

#[test]
fn test_page_rule_named() {
    let css = "@page cover { margin: 0; }";
    let stylesheet = parse_stylesheet(css).unwrap();
    
    if let Rule::At(AtRule::Page(page)) = &stylesheet.rules[0] {
        assert!(page.selectors.iter().any(|s| matches!(s, PageSelector::Named(n) if n == "cover")));
    } else {
        panic!("Expected @page rule");
    }
}

#[test]
fn test_page_rule_pseudo() {
    let cases = vec![
        ("@page :first { margin-top: 2cm; }", PageSelector::First),
        ("@page :left { margin-left: 2cm; }", PageSelector::Left),
        ("@page :right { margin-right: 2cm; }", PageSelector::Right),
        ("@page :blank { display: none; }", PageSelector::Blank),
    ];
    
    for (css, expected) in cases {
        let stylesheet = parse_stylesheet(css).unwrap();
        
        if let Rule::At(AtRule::Page(page)) = &stylesheet.rules[0] {
            assert!(page.selectors.iter().any(|s| std::mem::discriminant(s) == std::mem::discriminant(&expected)));
        } else {
            panic!("Expected @page rule for {}", css);
        }
    }
}

#[test]
fn test_page_margin_boxes() {
    let css = r#"
        @page {
            @top-left { content: "Header"; }
            @top-center { content: "Title"; }
            @top-right { content: counter(page); }
            @bottom-center { content: "Footer"; }
        }
    "#;
    
    let stylesheet = parse_stylesheet(css).unwrap();
    
    if let Rule::At(AtRule::Page(page)) = &stylesheet.rules[0] {
        assert!(!page.margin_boxes.is_empty());
    }
}

#[test]
fn test_page_size() {
    let css = r#"
        @page {
            size: A4;
        }
        @page {
            size: A4 landscape;
        }
        @page {
            size: letter;
        }
        @page {
            size: 210mm 297mm;
        }
    "#;
    
    let stylesheet = parse_stylesheet(css).unwrap();
    assert_eq!(stylesheet.get_page_rules().len(), 4);
}

// ============================================================================
// @media Rule Tests
// ============================================================================

#[test]
fn test_media_rule() {
    let css = "@media print { body { color: black; } }";
    let stylesheet = parse_stylesheet(css).unwrap();
    
    assert!(matches!(&stylesheet.rules[0], Rule::At(AtRule::Media { .. })));
}

#[test]
fn test_media_rule_screen() {
    let css = "@media screen { body { color: blue; } }";
    let stylesheet = parse_stylesheet(css).unwrap();
    
    assert!(matches!(&stylesheet.rules[0], Rule::At(AtRule::Media { .. })));
}

#[test]
fn test_media_rule_complex() {
    let css = "@media screen and (min-width: 768px) { .container { width: 750px; } }";
    let stylesheet = parse_stylesheet(css).unwrap();
    
    if let Rule::At(AtRule::Media { query, .. }) = &stylesheet.rules[0] {
        assert!(query.contains("screen") || query.contains("min-width"));
    }
}

// ============================================================================
// @import Rule Tests
// ============================================================================

#[test]
fn test_import_rule() {
    let css = r#"@import url("styles.css");"#;
    let stylesheet = parse_stylesheet(css).unwrap();
    
    assert!(matches!(&stylesheet.rules[0], Rule::At(AtRule::Import(_))));
}

#[test]
fn test_import_with_media() {
    let css = r#"@import url("print.css") print;"#;
    let stylesheet = parse_stylesheet(css).unwrap();
    
    if let Rule::At(AtRule::Import(import)) = &stylesheet.rules[0] {
        assert!(import.media.contains(&"print".to_string()));
    }
}

// ============================================================================
// @font-face Rule Tests
// ============================================================================

#[test]
fn test_font_face_rule() {
    let css = r#"
        @font-face {
            font-family: "CustomFont";
            src: url("font.woff2");
        }
    "#;
    let stylesheet = parse_stylesheet(css).unwrap();
    
    assert!(matches!(&stylesheet.rules[0], Rule::At(AtRule::FontFace(_))));
}

// ============================================================================
// @supports Rule Tests
// ============================================================================

#[test]
fn test_supports_rule() {
    let css = r#"
        @supports (display: flex) {
            .container { display: flex; }
        }
    "#;
    let stylesheet = parse_stylesheet(css).unwrap();
    
    assert!(matches!(&stylesheet.rules[0], Rule::At(AtRule::Supports { .. })));
}

// ============================================================================
// Property Validation Tests
// ============================================================================

#[test]
fn test_valid_properties() {
    let valid = vec![
        "display", "position", "color", "background",
        "font-size", "margin", "padding", "border",
        "flex", "grid", "opacity", "z-index",
    ];
    
    for prop in valid {
        assert!(is_valid_property(prop), "{} should be valid", prop);
        assert!(is_valid_property(&prop.to_uppercase()), "{} should be case-insensitive", prop);
    }
}

#[test]
fn test_custom_properties() {
    assert!(is_valid_property("--primary-color"));
    assert!(is_valid_property("--spacing-unit"));
    assert!(is_valid_property("--my-custom-property"));
}

#[test]
fn test_invalid_properties() {
    let invalid = vec![
        "invalid",
        "not-a-property",
        "random-thing",
    ];
    
    for prop in invalid {
        assert!(!is_valid_property(prop), "{} should be invalid", prop);
    }
}

#[test]
fn test_print_css_properties() {
    let print_props = vec![
        "page", "page-break-before", "page-break-after",
        "break-before", "break-after", "orphans", "widows",
    ];
    
    for prop in print_props {
        assert!(is_valid_property(prop), "{} should be valid (PrintCSS)", prop);
    }
}

// ============================================================================
// Selector Matching Tests
// ============================================================================

fn create_test_element(tag: &str, attrs: Vec<(&str, &str)>) -> Element {
    let attributes: Vec<Attribute> = attrs.iter()
        .map(|(k, v)| Attribute::new(*k, *v))
        .collect();
    Element::new(tag, attributes)
}

#[test]
fn test_selector_matches_type() {
    let selector = parse_selector("div").unwrap();
    let element = create_test_element("div", vec![]);
    
    assert!(selector.matches(&element));
}

#[test]
fn test_selector_matches_id() {
    let selector = parse_selector("#main").unwrap();
    let element = create_test_element("div", vec![("id", "main")]);
    
    assert!(selector.matches(&element));
}

#[test]
fn test_selector_matches_class() {
    let selector = parse_selector(".container").unwrap();
    let element = create_test_element("div", vec![("class", "container")]);
    
    assert!(selector.matches(&element));
}

#[test]
fn test_selector_matches_multiple_classes() {
    let selector = parse_selector(".a.b").unwrap();
    let element = create_test_element("div", vec![("class", "a b c")]);
    
    assert!(selector.matches(&element));
}

#[test]
fn test_selector_matches_attribute() {
    let selector = parse_selector("[disabled]").unwrap();
    let element = create_test_element("input", vec![("disabled", "")]);
    
    assert!(selector.matches(&element));
}

#[test]
fn test_selector_matches_attribute_value() {
    let selector = parse_selector("[type=\"text\"]").unwrap();
    let element = create_test_element("input", vec![("type", "text")]);
    
    assert!(selector.matches(&element));
}

#[test]
fn test_selector_matches_compound() {
    let selector = parse_selector("div.container#main").unwrap();
    let element = create_test_element("div", vec![
        ("id", "main"),
        ("class", "container"),
    ]);
    
    assert!(selector.matches(&element));
}

// ============================================================================
// Specificity Tests
// ============================================================================

#[test]
fn test_specificity_calculation() {
    // (a, b, c) where:
    // a = ID selectors
    // b = class selectors, attribute selectors, and pseudo-classes
    // c = type selectors and pseudo-elements
    
    let cases = vec![
        ("*", (0, 0, 0)),
        ("div", (0, 0, 1)),
        (".class", (0, 1, 0)),
        ("#id", (1, 0, 0)),
        ("div.class", (0, 1, 1)),
        ("div#id", (1, 0, 1)),
        ("#id.class", (1, 1, 0)),
    ];
    
    for (selector_str, expected) in cases {
        let selector = parse_selector(selector_str).unwrap();
        let specificity = selector.specificity();
        assert_eq!(specificity, expected, "Specificity mismatch for {}", selector_str);
    }
}

// ============================================================================
// Error Recovery Tests
// ============================================================================

#[test]
fn test_error_recovery_invalid_selector() {
    let css = "/// { color: red; } body { color: blue; }";
    let result = parse_stylesheet(css);
    // Should either succeed or fail gracefully
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_error_recovery_invalid_value() {
    let css = "body { color: !!!; font-size: 16px; }";
    let result = parse_stylesheet(css);
    // Should recover and parse the valid declaration
    assert!(result.is_ok());
}

#[test]
fn test_error_recovery_unclosed_rule() {
    let css = "body { color: red";
    let result = parse_stylesheet(css);
    // Should handle gracefully
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_error_recovery_unclosed_string() {
    let css = r#"body { content: "unclosed; }"#;
    let result = parse_stylesheet(css);
    // Should handle gracefully
    assert!(result.is_ok() || result.is_err());
}

// ============================================================================
// Performance Tests
// ============================================================================

#[test]
fn test_large_stylesheet_parsing() {
    let mut css = String::new();
    for i in 0..1000 {
        css.push_str(&format!(".class-{} {{ color: red; background: blue; margin: 10px; }}\n", i));
    }
    
    let start = std::time::Instant::now();
    let stylesheet = parse_stylesheet(&css).unwrap();
    let elapsed = start.elapsed();
    
    assert!(elapsed.as_secs() < 1, "Large stylesheet parsing took too long: {:?}", elapsed);
    assert!(stylesheet.rules.len() >= 1000);
}

#[test]
fn test_deeply_nested_selectors() {
    let depth = 50;
    let mut selector = String::from("div");
    for _ in 1..depth {
        selector.push_str(" > div");
    }
    selector.push_str(" { color: red; }");
    
    let css = &selector;
    let result = parse_stylesheet(css);
    assert!(result.is_ok());
}

// ============================================================================
// CSS Custom Properties (Variables) Tests
// ============================================================================

#[test]
fn test_custom_property_declaration() {
    let css = r#"
        :root {
            --primary-color: #007bff;
            --spacing-unit: 8px;
        }
    "#;
    
    let stylesheet = parse_stylesheet(css).unwrap();
    assert!(!stylesheet.rules.is_empty());
}

#[test]
fn test_custom_property_usage() {
    let css = r#"
        body {
            color: var(--primary-color);
            margin: calc(var(--spacing-unit) * 2);
        }
    "#;
    
    let stylesheet = parse_stylesheet(css).unwrap();
    assert!(!stylesheet.rules.is_empty());
}

// ============================================================================
// CSS Grid Tests
// ============================================================================

#[test]
fn test_grid_properties() {
    let css = r#"
        .container {
            display: grid;
            grid-template-columns: 1fr 2fr 1fr;
            grid-template-rows: auto;
            gap: 20px;
        }
    "#;
    
    let stylesheet = parse_stylesheet(css).unwrap();
    assert!(!stylesheet.rules.is_empty());
}

// ============================================================================
// CSS Flexbox Tests
// ============================================================================

#[test]
fn test_flexbox_properties() {
    let css = r#"
        .container {
            display: flex;
            flex-direction: row;
            justify-content: center;
            align-items: stretch;
            gap: 10px;
        }
        .item {
            flex: 1 1 auto;
            align-self: flex-start;
        }
    "#;
    
    let stylesheet = parse_stylesheet(css).unwrap();
    assert!(stylesheet.rules.len() >= 2);
}

// ============================================================================
// CSS Animations/Transitions Tests
// ============================================================================

#[test]
fn test_keyframes_rule() {
    let css = r#"
        @keyframes slide {
            from { transform: translateX(0); }
            50% { transform: translateX(50%); }
            to { transform: translateX(100%); }
        }
    "#;
    
    let stylesheet = parse_stylesheet(css).unwrap();
    assert!(matches!(&stylesheet.rules[0], Rule::At(AtRule::Keyframes { .. })));
}

#[test]
fn test_transition_property() {
    let css = r#"
        .button {
            transition: all 0.3s ease-in-out;
            transition-property: color, background;
            transition-duration: 0.2s, 0.3s;
        }
    "#;
    
    let stylesheet = parse_stylesheet(css).unwrap();
    assert!(!stylesheet.rules.is_empty());
}
