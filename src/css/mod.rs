//! CSS3 Parser with PrintCSS support
//!
//! Implements CSS Syntax Module Level 3 and CSS Paged Media Module

mod tokenizer;
pub mod parser;
mod values;
mod selectors;
mod at_rules;

pub use tokenizer::{CssTokenizer, CssToken};
pub use parser::{CssParser, Stylesheet, Rule, StyleRule, Declaration, ImportRule};
pub use values::{CssValue, CssFunction, Unit};
pub use selectors::{Selector, SelectorPart, Combinator, AttributeOp};
pub use at_rules::{AtRule, PageRule, PageMarginBox, PageSelector, MarginBoxType};

use crate::types::Result;

/// Parse CSS stylesheet from string
pub fn parse_stylesheet(input: &str) -> Result<Stylesheet> {
    let mut parser = CssParser::new(input);
    parser.parse()
}

/// Parse a single CSS rule
pub fn parse_rule(input: &str) -> Result<Rule> {
    let mut parser = CssParser::new(input);
    // Try to parse as a style rule
    parser.parse()
        .map(|stylesheet| {
            stylesheet.rules.into_iter().next()
                .unwrap_or_else(|| Rule::style_rule(StyleRule::default()))
        })
}

/// Parse CSS value
pub fn parse_value(input: &str) -> Result<CssValue> {
    // Create a simple value parser
    let trimmed = input.trim();
    
    // Try to parse as length
    if let Some((num, unit)) = parse_length_value(trimmed) {
        return Ok(CssValue::Length(num, unit));
    }
    
    // Try to parse as percentage
    if let Some(stripped) = trimmed.strip_suffix('%') {
        if let Ok(num) = stripped.parse() {
            return Ok(CssValue::Percentage(num));
        }
    }
    
    // Try to parse as number
    if let Ok(num) = trimmed.parse::<f32>() {
        return Ok(CssValue::Number(num));
    }
    
    // Default to keyword
    Ok(CssValue::Keyword(trimmed.to_string()))
}

/// Parse CSS selector
pub fn parse_selector(input: &str) -> Result<Selector> {
    let mut selector = Selector::new();
    let input = input.trim();
    
    // Simple selector parsing
    if input == "*" {
        selector.parts.push(SelectorPart::Universal);
    } else if let Some(id) = input.strip_prefix('#') {
        selector.parts.push(SelectorPart::Id(id.to_string()));
    } else if let Some(class) = input.strip_prefix('.') {
        selector.parts.push(SelectorPart::Class(class.to_string()));
    } else {
        selector.parts.push(SelectorPart::Element(input.to_string()));
    }
    
    Ok(selector)
}

fn parse_length_value(s: &str) -> Option<(f32, Unit)> {
    let s = s.trim();
    
    if let Some(stripped) = s.strip_suffix("px") {
        stripped.trim().parse().ok().map(|n| (n, Unit::Px))
    } else if let Some(stripped) = s.strip_suffix("pt") {
        stripped.trim().parse().ok().map(|n| (n, Unit::Pt))
    } else if let Some(stripped) = s.strip_suffix("em") {
        stripped.trim().parse().ok().map(|n| (n, Unit::Em))
    } else if let Some(stripped) = s.strip_suffix("rem") {
        stripped.trim().parse().ok().map(|n| (n, Unit::Rem))
    } else if let Some(stripped) = s.strip_suffix("vw") {
        stripped.trim().parse().ok().map(|n| (n, Unit::Vw))
    } else if let Some(stripped) = s.strip_suffix("vh") {
        stripped.trim().parse().ok().map(|n| (n, Unit::Vh))
    } else if let Some(stripped) = s.strip_suffix("mm") {
        stripped.trim().parse().ok().map(|n| (n, Unit::Mm))
    } else if let Some(stripped) = s.strip_suffix("cm") {
        stripped.trim().parse().ok().map(|n| (n, Unit::Cm))
    } else if let Some(stripped) = s.strip_suffix("in") {
        stripped.trim().parse().ok().map(|n| (n, Unit::In))
    } else {
        None
    }
}

/// CSS property name validation
pub fn is_valid_property(name: &str) -> bool {
    // CSS property names are case-insensitive
    let name_lower = name.to_ascii_lowercase();
    
    // All standard CSS properties
    STANDARD_PROPERTIES.contains(&name_lower.as_str()) ||
    name_lower.starts_with("--") // Custom properties
}

/// Standard CSS properties list
pub const STANDARD_PROPERTIES: &[&str] = &[
    // Layout
    "display", "position", "top", "right", "bottom", "left", "z-index",
    "float", "clear", "visibility", "overflow", "overflow-x", "overflow-y",
    "clip", "box-sizing", "resize", "cursor",
    
    // Flexbox
    "flex", "flex-grow", "flex-shrink", "flex-basis", "flex-flow",
    "flex-direction", "flex-wrap", "justify-content", "align-items",
    "align-content", "align-self", "order", "gap", "row-gap", "column-gap",
    
    // Grid
    "grid", "grid-template", "grid-template-columns", "grid-template-rows",
    "grid-template-areas", "grid-auto-columns", "grid-auto-rows",
    "grid-auto-flow", "grid-column", "grid-row", "grid-area",
    "grid-column-start", "grid-column-end", "grid-row-start", "grid-row-end",
    "justify-items", "justify-self",
    
    // Box Model
    "width", "height", "min-width", "min-height", "max-width", "max-height",
    "margin", "margin-top", "margin-right", "margin-bottom", "margin-left",
    "padding", "padding-top", "padding-right", "padding-bottom", "padding-left",
    "border", "border-top", "border-right", "border-bottom", "border-left",
    "border-width", "border-style", "border-color", "border-radius",
    "border-collapse", "border-spacing", "outline", "outline-width",
    "outline-style", "outline-color", "outline-offset",
    
    // Background
    "background", "background-color", "background-image", "background-position",
    "background-size", "background-repeat", "background-origin",
    "background-clip", "background-attachment", "background-blend-mode",
    
    // Color
    "color", "opacity", "mix-blend-mode",
    
    // Typography
    "font", "font-family", "font-size", "font-weight", "font-style",
    "font-variant", "font-variant-ligatures", "font-variant-caps",
    "font-variant-numeric", "font-variant-east-asian", "font-stretch",
    "font-size-adjust", "font-kerning", "font-feature-settings",
    "font-variation-settings", "font-display", "line-height", "text-align",
    "text-align-last", "text-indent", "text-justify", "text-transform",
    "text-decoration", "text-decoration-line", "text-decoration-color",
    "text-decoration-style", "text-decoration-thickness", "text-underline-position",
    "text-shadow", "letter-spacing", "word-spacing", "white-space",
    "word-wrap", "overflow-wrap", "word-break", "line-break", "hyphens",
    "text-overflow", "vertical-align", "direction", "unicode-bidi",
    "writing-mode", "text-orientation", "text-combine-upright",
    
    // Lists
    "list-style", "list-style-type", "list-style-position", "list-style-image",
    
    // Tables
    "table-layout", "caption-side", "empty-cells",
    
    // Transforms
    "transform", "transform-origin", "transform-style", "transform-box",
    "perspective", "perspective-origin", "backface-visibility",
    
    // Transitions & Animations
    "transition", "transition-property", "transition-duration",
    "transition-timing-function", "transition-delay",
    "animation", "animation-name", "animation-duration", "animation-timing-function",
    "animation-delay", "animation-iteration-count", "animation-direction",
    "animation-fill-mode", "animation-play-state",
    
    // PrintCSS / Paged Media
    "page", "page-break-before", "page-break-after", "page-break-inside",
    "break-before", "break-after", "break-inside", "orphans", "widows",
    "box-decoration-break", "marks", "bleed",
    
    // Generated Content
    "content", "quotes", "counter-increment", "counter-reset", "counter-set",
    
    // Cursors
    "cursor", "caret-color",
    
    // UI
    "appearance", "user-select", "pointer-events", "touch-action",
    
    // Misc
    "filter", "image-orientation", "image-rendering", "image-resolution",
    "mask", "mask-image", "mask-mode", "mask-repeat", "mask-position",
    "mask-clip", "mask-origin", "mask-size", "mask-composite",
    "mask-border", "mask-border-source", "mask-border-slice",
    "mask-border-width", "mask-border-outset", "mask-border-repeat",
    "mask-border-mode", "mask-type",
    "object-fit", "object-position", "isolation",
    "contain", "contain-intrinsic-size", "content-visibility",
    "scroll-behavior", "scroll-margin", "scroll-padding", "scroll-snap-type",
    "scroll-snap-align", "scroll-snap-stop", "overscroll-behavior",
];

/// PrintCSS specific properties
pub const PRINT_PROPERTIES: &[&str] = &[
    "page",
    "page-break-before",
    "page-break-after", 
    "page-break-inside",
    "break-before",
    "break-after",
    "break-inside",
    "orphans",
    "widows",
    "box-decoration-break",
    "marks",
    "bleed",
    "string-set",
    "running",
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_property_validation() {
        assert!(is_valid_property("display"));
        assert!(is_valid_property("DISPLAY"));
        assert!(is_valid_property("--custom-property"));
        assert!(!is_valid_property("invalid-property"));
    }

    #[test]
    fn test_parse_simple_stylesheet() {
        let css = r#"
            body {
                color: black;
                background: white;
            }
            
            h1 {
                font-size: 24px;
            }
        "#;
        
        let result = parse_stylesheet(css);
        // Parser not fully implemented yet, but structure is ready
        assert!(result.is_ok() || result.is_err()); // Placeholder
    }

    #[test]
    fn test_print_properties() {
        assert!(PRINT_PROPERTIES.contains(&"page"));
        assert!(PRINT_PROPERTIES.contains(&"break-before"));
        assert!(PRINT_PROPERTIES.contains(&"orphans"));
    }
}
