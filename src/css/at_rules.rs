//! CSS At-Rules
//!
//! Implements CSS at-rules such as @import, @media, @page, @font-face, etc.

use crate::css::Declaration;

/// CSS At-Rule types
#[derive(Debug, Clone, PartialEq)]
pub enum AtRule {
    /// @import rule - uses ImportRule from parser module
    /// Note: ImportRule is defined in parser.rs to avoid duplication
    Import(super::parser::ImportRule),
    /// @media rule
    Media { query: String, rules: Vec<crate::css::Rule> },
    /// @page rule
    Page(PageRule),
    /// @font-face rule
    FontFace(Vec<Declaration>),
    /// @keyframes rule
    Keyframes { name: String, vendor_prefix: Option<String>, keyframes: Vec<(Vec<String>, Vec<Declaration>)> },
    /// @supports rule
    Supports { condition: String, rules: Vec<crate::css::Rule> },
    /// Unknown at-rule
    Unknown(String),
}

/// Page rule (@page)
#[derive(Debug, Clone, PartialEq, Default)]
pub struct PageRule {
    pub selectors: Vec<PageSelector>,
    pub declarations: Vec<Declaration>,
    pub margin_boxes: Vec<PageMarginBox>,
}

impl PageRule {
    /// Create a new empty page rule
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a selector to this page rule
    pub fn add_selector(&mut self, selector: PageSelector) {
        self.selectors.push(selector);
    }

    /// Add a declaration
    pub fn add_declaration(&mut self, declaration: Declaration) {
        self.declarations.push(declaration);
    }

    /// Add a margin box
    pub fn add_margin_box(&mut self, margin_box: PageMarginBox) {
        self.margin_boxes.push(margin_box);
    }
}

/// Page selector for @page rules
#[derive(Debug, Clone, PartialEq)]
pub enum PageSelector {
    /// First page
    First,
    /// Left pages (verso)
    Left,
    /// Right pages (recto)
    Right,
    /// Blank pages
    Blank,
    /// Named page
    Named(String),
    /// Pseudo-class selector
    Pseudo(String),
}

/// Page margin box type for @page margin boxes
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MarginBoxType {
    TopLeftCorner,
    TopLeft,
    TopCenter,
    TopRight,
    TopRightCorner,
    BottomLeftCorner,
    BottomLeft,
    BottomCenter,
    BottomRight,
    BottomRightCorner,
    LeftTop,
    LeftMiddle,
    LeftBottom,
    RightTop,
    RightMiddle,
    RightBottom,
}

/// Page margin box (@top-left, @bottom-center, etc.)
#[derive(Debug, Clone, PartialEq)]
pub struct PageMarginBox {
    /// Margin box type
    pub box_type: MarginBoxType,
    /// Content declarations for this margin box
    pub declarations: Vec<Declaration>,
}

impl PageMarginBox {
    /// Create a new margin box
    pub fn new(box_type: MarginBoxType) -> Self {
        Self {
            box_type,
            declarations: Vec::new(),
        }
    }

    /// Add a declaration
    pub fn add_declaration(&mut self, declaration: Declaration) {
        self.declarations.push(declaration);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_page_rule() {
        let mut rule = PageRule::new();
        rule.add_selector(PageSelector::First);
        
        assert_eq!(rule.selectors.len(), 1);
        assert!(matches!(rule.selectors[0], PageSelector::First));
    }

    #[test]
    fn test_page_margin_box() {
        let margin_box = PageMarginBox::new(MarginBoxType::TopCenter);
        
        assert!(matches!(margin_box.box_type, MarginBoxType::TopCenter));
        assert!(margin_box.declarations.is_empty());
    }
}
