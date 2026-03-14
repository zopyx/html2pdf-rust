//! CSS Parser
//!
//! Parses token stream into Stylesheet, Rules, and Declarations

use super::{
    CssToken, CssTokenizer,
    values::{CssValue, CssFunction, Unit},
    selectors::Selector,
    at_rules::{AtRule, PageRule, PageSelector, PageMarginBox, MarginBoxType},
};
use crate::types::{Result, PdfError};

/// A CSS stylesheet
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Stylesheet {
    pub rules: Vec<Rule>,
    pub imports: Vec<ImportRule>,
}

impl Stylesheet {
    pub fn new() -> Self {
        Self {
            rules: Vec::new(),
            imports: Vec::new(),
        }
    }

    /// Add a rule
    pub fn add_rule(&mut self, rule: Rule) {
        self.rules.push(rule);
    }

    /// Get all style rules matching a selector
    pub fn get_matching_rules(&self, element: &crate::html::Element) -> Vec<&StyleRule> {
        self.rules
            .iter()
            .filter_map(|r| match r {
                Rule::Style(s) => {
                    if s.selector.matches(element) {
                        Some(s)
                    } else {
                        None
                    }
                }
                _ => None,
            })
            .collect()
    }

    /// Get @page rules
    pub fn get_page_rules(&self) -> Vec<&PageRule> {
        self.rules
            .iter()
            .filter_map(|r| match r {
                Rule::At(AtRule::Page(p)) => Some(p),
                _ => None,
            })
            .collect()
    }
}

/// Import rule (@import)
#[derive(Debug, Clone, PartialEq)]
pub struct ImportRule {
    pub url: String,
    pub media: Vec<String>,
}

/// CSS Rule types
#[derive(Debug, Clone, PartialEq)]
pub enum Rule {
    Style(StyleRule),
    At(AtRule),
}

impl Rule {
    /// Get the style rule variant
    pub fn StyleRule(rule: StyleRule) -> Self {
        Rule::Style(rule)
    }
}

/// Style rule (selector + declarations)
#[derive(Debug, Clone, PartialEq)]
pub struct StyleRule {
    pub selector: Selector,
    pub declarations: Vec<Declaration>,
    pub important_flags: Vec<bool>, // Parallel to declarations
}

impl Default for StyleRule {
    fn default() -> Self {
        Self {
            selector: Selector::new(),
            declarations: Vec::new(),
            important_flags: Vec::new(),
        }
    }
}

impl StyleRule {
    pub fn new(selector: Selector) -> Self {
        Self {
            selector,
            declarations: Vec::new(),
            important_flags: Vec::new(),
        }
    }

    pub fn add_declaration(&mut self, name: impl Into<String>, value: CssValue) {
        self.declarations.push(Declaration::new(name, value));
        self.important_flags.push(false);
    }

    pub fn add_declaration_important(&mut self, name: impl Into<String>, value: CssValue) {
        self.declarations.push(Declaration::new(name, value));
        self.important_flags.push(true);
    }

    /// Get property value
    pub fn get_property(&self, name: &str) -> Option<&CssValue> {
        self.declarations
            .iter()
            .find(|d| d.name.eq_ignore_ascii_case(name))
            .map(|d| &d.value)
    }

    /// Check if property is !important
    pub fn is_important(&self, name: &str) -> bool {
        self.declarations
            .iter()
            .position(|d| d.name.eq_ignore_ascii_case(name))
            .map(|idx| self.important_flags.get(idx).copied().unwrap_or(false))
            .unwrap_or(false)
    }
}

/// CSS Declaration (property: value)
#[derive(Debug, Clone, PartialEq)]
pub struct Declaration {
    pub name: String,
    pub value: CssValue,
}

impl Declaration {
    pub fn new(name: impl Into<String>, value: CssValue) -> Self {
        Self {
            name: name.into(),
            value,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn value(&self) -> &CssValue {
        &self.value
    }
}

/// CSS Parser
pub struct CssParser<'a> {
    tokens: Vec<CssToken>,
    position: usize,
    original_input: &'a str,
}

impl<'a> CssParser<'a> {
    pub fn new(input: &'a str) -> Self {
        let tokens = CssTokenizer::new(input).collect();
        Self {
            tokens,
            position: 0,
            original_input: input,
        }
    }

    /// Collect tokens from tokenizer
    fn collect_tokens(tokenizer: &mut CssTokenizer) -> Vec<CssToken> {
        let mut tokens = Vec::new();
        loop {
            let token = tokenizer.next_token();
            let is_eof = matches!(token, CssToken::EOF);
            tokens.push(token);
            if is_eof {
                break;
            }
        }
        tokens
    }

    /// Parse entire stylesheet
    pub fn parse(&mut self) -> Result<Stylesheet> {
        let mut stylesheet = Stylesheet::new();

        self.consume_whitespace();

        while !self.is_at_end() {
            if let Some(rule) = self.consume_rule()? {
                match rule {
                    Rule::At(AtRule::Import(import)) => {
                        stylesheet.imports.push(import);
                    }
                    _ => {
                        stylesheet.add_rule(rule);
                    }
                }
            }
            self.consume_whitespace();
        }

        Ok(stylesheet)
    }

    /// Consume a single rule
    fn consume_rule(&mut self) -> Result<Option<Rule>> {
        match self.peek() {
            Some(CssToken::AtKeyword(_)) => {
                self.consume_at_rule().map(|r| Some(Rule::At(r)))
            }
            Some(CssToken::CDO) | Some(CssToken::CDC) => {
                self.advance(); // Skip HTML comment markers
                Ok(None)
            }
            Some(_) => {
                self.consume_qualified_rule().map(|r| r.map(Rule::Style))
            }
            None => Ok(None),
        }
    }

    /// Consume at-rule (@media, @page, etc.)
    fn consume_at_rule(&mut self) -> Result<AtRule> {
        let name = match self.peek() {
            Some(CssToken::AtKeyword(name)) => {
                let name = name.clone();
                self.advance();
                name
            }
            _ => return Err(PdfError::Parse("Expected at-keyword".to_string())),
        };

        match name.as_str() {
            "import" => self.parse_import_rule(),
            "media" => self.parse_media_rule(),
            "page" => self.parse_page_rule(),
            "font-face" => self.parse_font_face_rule(),
            "keyframes" | "-webkit-keyframes" | "-moz-keyframes" => {
                self.parse_keyframes_rule(&name)
            }
            "supports" => self.parse_supports_rule(),
            _ => {
                // Unknown at-rule, consume prelude and optional block
                self.consume_prelude();
                if self.peek() == Some(&CssToken::OpenBrace) {
                    self.consume_simple_block()?;
                }
                Ok(AtRule::Unknown(name))
            }
        }
    }

    /// Parse @import rule
    fn parse_import_rule(&mut self) -> Result<AtRule> {
        self.consume_whitespace();

        let url = match self.peek() {
            Some(CssToken::Url(url)) => {
                let url = url.clone();
                self.advance();
                url
            }
            Some(CssToken::String(url)) => {
                let url = url.clone();
                self.advance();
                url
            }
            _ => return Err(PdfError::Parse("Expected URL in @import".to_string())),
        };

        self.consume_whitespace();

        // Parse media query list (optional)
        let mut media = Vec::new();
        while !self.is_at_end() && self.peek() != Some(&CssToken::Semicolon) {
            if let Some(CssToken::Ident(m)) = self.peek() {
                media.push(m.clone());
                self.advance();
            } else if self.peek() == Some(&CssToken::Comma) {
                self.advance();
            } else {
                break;
            }
            self.consume_whitespace();
        }

        // Consume semicolon
        if self.peek() == Some(&CssToken::Semicolon) {
            self.advance();
        }

        Ok(AtRule::Import(ImportRule { url, media }))
    }

    /// Parse @media rule
    fn parse_media_rule(&mut self) -> Result<AtRule> {
        // Consume media query list (simplified)
        self.consume_whitespace();
        let media_query = self.consume_prelude_string();

        if self.peek() != Some(&CssToken::OpenBrace) {
            return Err(PdfError::Parse("Expected { in @media rule".to_string()));
        }

        self.advance(); // {

        let mut rules = Vec::new();
        self.consume_whitespace();

        while !self.is_at_end() && self.peek() != Some(&CssToken::CloseBrace) {
            if let Some(rule) = self.consume_qualified_rule()? {
                rules.push(Rule::Style(rule));
            }
            self.consume_whitespace();
        }

        if self.peek() == Some(&CssToken::CloseBrace) {
            self.advance();
        }

        Ok(AtRule::Media { query: media_query, rules })
    }

    /// Parse @page rule
    fn parse_page_rule(&mut self) -> Result<AtRule> {
        self.consume_whitespace();

        // Parse page selectors (e.g., :first, :left, :right)
        let mut selectors = Vec::new();

        while !self.is_at_end() && self.peek() != Some(&CssToken::OpenBrace) {
            match self.peek() {
                Some(CssToken::Ident(name)) => {
                    selectors.push(PageSelector::Named(name.clone()));
                    self.advance();
                }
                Some(CssToken::Colon) => {
                    self.advance();
                    if let Some(CssToken::Ident(pseudo)) = self.peek() {
                        let selector = match pseudo.as_str() {
                            "first" => PageSelector::First,
                            "left" => PageSelector::Left,
                            "right" => PageSelector::Right,
                            "blank" => PageSelector::Blank,
                            _ => PageSelector::Named(pseudo.clone()),
                        };
                        selectors.push(selector);
                        self.advance();
                    }
                }
                _ => break,
            }
            self.consume_whitespace();
        }

        if self.peek() != Some(&CssToken::OpenBrace) {
            return Err(PdfError::Parse("Expected { in @page rule".to_string()));
        }

        self.advance(); // {

        let mut declarations = Vec::new();
        let mut margin_boxes = Vec::new();

        self.consume_whitespace();

        while !self.is_at_end() && self.peek() != Some(&CssToken::CloseBrace) {
            // Check for margin box (@top-left, @bottom-center, etc.)
            if let Some(CssToken::AtKeyword(box_name)) = self.peek() {
                if Self::is_margin_box_name(box_name) {
                    let margin_box = self.parse_margin_box()?;
                    margin_boxes.push(margin_box);
                    continue;
                }
            }

            // Regular declaration
            if let Some(decl) = self.consume_declaration()? {
                declarations.push(decl);
            }

            self.consume_whitespace();
        }

        if self.peek() == Some(&CssToken::CloseBrace) {
            self.advance();
        }

        Ok(AtRule::Page(PageRule {
            selectors,
            declarations,
            margin_boxes,
        }))
    }

    /// Parse margin box (@top-left, @bottom-center, etc.)
    fn parse_margin_box(&mut self) -> Result<PageMarginBox> {
        let name = match self.peek() {
            Some(CssToken::AtKeyword(name)) => {
                let name = name.clone();
                self.advance();
                name
            }
            _ => return Err(PdfError::Parse("Expected margin box name".to_string())),
        };

        let box_type = match name.as_str() {
            "top-left-corner" => MarginBoxType::TopLeftCorner,
            "top-left" => MarginBoxType::TopLeft,
            "top-center" => MarginBoxType::TopCenter,
            "top-right" => MarginBoxType::TopRight,
            "top-right-corner" => MarginBoxType::TopRightCorner,
            "bottom-left-corner" => MarginBoxType::BottomLeftCorner,
            "bottom-left" => MarginBoxType::BottomLeft,
            "bottom-center" => MarginBoxType::BottomCenter,
            "bottom-right" => MarginBoxType::BottomRight,
            "bottom-right-corner" => MarginBoxType::BottomRightCorner,
            "left-top" => MarginBoxType::LeftTop,
            "left-middle" => MarginBoxType::LeftMiddle,
            "left-bottom" => MarginBoxType::LeftBottom,
            "right-top" => MarginBoxType::RightTop,
            "right-middle" => MarginBoxType::RightMiddle,
            "right-bottom" => MarginBoxType::RightBottom,
            _ => return Err(PdfError::Parse(format!("Unknown margin box: {}", name))),
        };

        if self.peek() != Some(&CssToken::OpenBrace) {
            return Err(PdfError::Parse("Expected { in margin box".to_string()));
        }

        self.advance(); // {

        let mut declarations = Vec::new();
        self.consume_whitespace();

        while !self.is_at_end() && self.peek() != Some(&CssToken::CloseBrace) {
            if let Some(decl) = self.consume_declaration()? {
                declarations.push(decl);
            }
            self.consume_whitespace();
        }

        if self.peek() == Some(&CssToken::CloseBrace) {
            self.advance();
        }

        Ok(PageMarginBox {
            box_type,
            declarations,
        })
    }

    fn is_margin_box_name(name: &str) -> bool {
        matches!(name,
            "top-left-corner" | "top-left" | "top-center" | "top-right" |
            "top-right-corner" | "bottom-left-corner" | "bottom-left" |
            "bottom-center" | "bottom-right" | "bottom-right-corner" |
            "left-top" | "left-middle" | "left-bottom" |
            "right-top" | "right-middle" | "right-bottom"
        )
    }

    /// Parse @font-face rule
    fn parse_font_face_rule(&mut self) -> Result<AtRule> {
        self.consume_whitespace();

        if self.peek() != Some(&CssToken::OpenBrace) {
            return Err(PdfError::Parse("Expected { in @font-face".to_string()));
        }

        self.advance(); // {

        let mut declarations = Vec::new();
        self.consume_whitespace();

        while !self.is_at_end() && self.peek() != Some(&CssToken::CloseBrace) {
            if let Some(decl) = self.consume_declaration()? {
                declarations.push(decl);
            }
            self.consume_whitespace();
        }

        if self.peek() == Some(&CssToken::CloseBrace) {
            self.advance();
        }

        Ok(AtRule::FontFace(declarations))
    }

    /// Parse @keyframes rule
    fn parse_keyframes_rule(&mut self, name: &str) -> Result<AtRule> {
        self.consume_whitespace();

        let animation_name = match self.peek() {
            Some(CssToken::Ident(n)) | Some(CssToken::String(n)) => {
                let n = n.clone();
                self.advance();
                n
            }
            _ => return Err(PdfError::Parse("Expected animation name".to_string())),
        };

        self.consume_whitespace();

        if self.peek() != Some(&CssToken::OpenBrace) {
            return Err(PdfError::Parse("Expected { in @keyframes".to_string()));
        }

        self.advance(); // {

        let mut keyframes = Vec::new();
        self.consume_whitespace();

        while !self.is_at_end() && self.peek() != Some(&CssToken::CloseBrace) {
            // Parse keyframe selector (0%, 100%, from, to)
            let selector = self.consume_keyframe_selector()?;
            self.consume_whitespace();

            if self.peek() != Some(&CssToken::OpenBrace) {
                return Err(PdfError::Parse("Expected { in keyframe".to_string()));
            }

            self.advance(); // {

            let mut declarations = Vec::new();
            self.consume_whitespace();

            while !self.is_at_end() && self.peek() != Some(&CssToken::CloseBrace) {
                if let Some(decl) = self.consume_declaration()? {
                    declarations.push(decl);
                }
                self.consume_whitespace();
            }

            if self.peek() == Some(&CssToken::CloseBrace) {
                self.advance();
            }

            keyframes.push((selector, declarations));
            self.consume_whitespace();
        }

        if self.peek() == Some(&CssToken::CloseBrace) {
            self.advance();
        }

        Ok(AtRule::Keyframes {
            name: animation_name,
            vendor_prefix: if name.starts_with('-') {
                Some(name.split('-').nth(1).unwrap_or("").to_string())
            } else {
                None
            },
            keyframes,
        })
    }

    fn consume_keyframe_selector(&mut self) -> Result<Vec<String>> {
        let mut selectors = Vec::new();

        loop {
            match self.peek() {
                Some(CssToken::Ident(s)) if s == "from" || s == "to" => {
                    selectors.push(s.clone());
                    self.advance();
                }
                Some(CssToken::Percentage(p)) => {
                    selectors.push(format!("{}%", p));
                    self.advance();
                }
                _ => break,
            }

            self.consume_whitespace();

            if self.peek() == Some(&CssToken::Comma) {
                self.advance();
                self.consume_whitespace();
            } else {
                break;
            }
        }

        Ok(selectors)
    }

    /// Parse @supports rule
    fn parse_supports_rule(&mut self) -> Result<AtRule> {
        self.consume_whitespace();

        // Consume supports condition (simplified)
        let condition = self.consume_prelude_string();

        if self.peek() != Some(&CssToken::OpenBrace) {
            return Err(PdfError::Parse("Expected { in @supports".to_string()));
        }

        self.advance(); // {

        let mut rules = Vec::new();
        self.consume_whitespace();

        while !self.is_at_end() && self.peek() != Some(&CssToken::CloseBrace) {
            if let Some(rule) = self.consume_qualified_rule()? {
                rules.push(Rule::Style(rule));
            }
            self.consume_whitespace();
        }

        if self.peek() == Some(&CssToken::CloseBrace) {
            self.advance();
        }

        Ok(AtRule::Supports { condition, rules })
    }

    /// Consume qualified rule (style rule)
    fn consume_qualified_rule(&mut self) -> Result<Option<StyleRule>> {
        // Try to parse as style rule
        let prelude_start = self.position;

        // Try to parse selector
        match self.consume_selector() {
            Ok(selector) => {
                self.consume_whitespace();

                if self.peek() != Some(&CssToken::OpenBrace) {
                    // Not a style rule, might be something else
                    self.position = prelude_start;
                    return Ok(None);
                }

                self.advance(); // {

                let mut rule = StyleRule::new(selector);
                self.consume_whitespace();

                while !self.is_at_end() && self.peek() != Some(&CssToken::CloseBrace) {
                    if let Some(decl) = self.consume_declaration()? {
                        rule.declarations.push(decl);
                    }
                    self.consume_whitespace();
                }

                if self.peek() == Some(&CssToken::CloseBrace) {
                    self.advance();
                }

                Ok(Some(rule))
            }
            Err(_) => {
                self.position = prelude_start;
                Ok(None)
            }
        }
    }

    /// Consume a declaration
    fn consume_declaration(&mut self) -> Result<Option<Declaration>> {
        let start_pos = self.position;

        let name = match self.peek() {
            Some(CssToken::Ident(name)) => {
                let name = name.clone();
                self.advance();
                name
            }
            _ => return Ok(None),
        };

        self.consume_whitespace();

        if self.peek() != Some(&CssToken::Colon) {
            // Not a declaration, revert
            self.position = start_pos;
            return Ok(None);
        }

        self.advance(); // :
        self.consume_whitespace();

        // Consume value
        let value = self.consume_value()?;

        // Check for !important
        let is_important = self.consume_important();

        // Consume semicolon (optional for last declaration)
        if self.peek() == Some(&CssToken::Semicolon) {
            self.advance();
        }

        Ok(Some(Declaration { name, value }))
    }

    /// Consume a value
    fn consume_value(&mut self) -> Result<CssValue> {
        let mut components = Vec::new();

        while !self.is_at_end() {
            match self.peek() {
                Some(CssToken::Semicolon) | Some(CssToken::CloseBrace) |
                Some(CssToken::CloseParen) | Some(CssToken::CloseBracket) => break,
                Some(CssToken::Delim('!')) => {
                    // Might be !important, stop here
                    if self.peek_important() {
                        break;
                    }
                    self.advance();
                    components.push(CssValue::Literal("!".to_string()));
                }
                Some(CssToken::Ident(s)) if s == "important" => {
                    // Check if preceded by !
                    if components.last() == Some(&CssValue::Literal("!".to_string())) {
                        break;
                    }
                    let s = s.clone();
                    self.advance();
                    components.push(CssValue::Ident(s));
                }
                Some(_) => {
                    if let Some(value) = self.consume_value_component()? {
                        components.push(value);
                    } else {
                        break;
                    }
                }
                None => break,
            }
        }

        if components.len() == 1 {
            Ok(components.into_iter().next().unwrap())
        } else {
            Ok(CssValue::List(components))
        }
    }

    /// Consume a single value component
    fn consume_value_component(&mut self) -> Result<Option<CssValue>> {
        match self.peek() {
            Some(CssToken::Ident(s)) => {
                let s = s.clone();
                self.advance();
                Ok(Some(CssValue::Ident(s)))
            }
            Some(CssToken::String(s)) => {
                let s = s.clone();
                self.advance();
                Ok(Some(CssValue::String(s)))
            }
            Some(CssToken::Number(n, _)) => {
                let n = *n as f32;
                self.advance();
                Ok(Some(CssValue::Number(n)))
            }
            Some(CssToken::Percentage(p)) => {
                let p = *p as f32;
                self.advance();
                Ok(Some(CssValue::Percentage(p)))
            }
            Some(CssToken::Dimension(n, unit, _)) => {
                let n = *n as f32;
                let unit = unit.clone();
                self.advance();
                if let Some(u) = Unit::from_str(&unit) {
                    Ok(Some(CssValue::Length(n, u)))
                } else {
                    Ok(Some(CssValue::Number(n)))
                }
            }
            Some(CssToken::Hash(h, _)) => {
                let h = u32::from_str_radix(h, 16).unwrap_or(0);
                self.advance();
                Ok(Some(CssValue::HexColor(h)))
            }
            Some(CssToken::Url(url)) => {
                let url = url.clone();
                self.advance();
                Ok(Some(CssValue::Url(url)))
            }
            Some(CssToken::Function(name)) => {
                let name = name.clone();
                self.advance();
                self.consume_function(name).map(Some)
            }
            Some(CssToken::Delim(c)) => {
                let c = *c;
                self.advance();
                Ok(Some(CssValue::Literal(c.to_string())))
            }
            Some(CssToken::OpenParen) => {
                self.advance();
                let inner = self.consume_value()?;
                if self.peek() == Some(&CssToken::CloseParen) {
                    self.advance();
                }
                Ok(Some(CssValue::Parenthesized(Box::new(inner))))
            }
            _ => Ok(None),
        }
    }

    /// Consume a function
    fn consume_function(&mut self, name: String) -> Result<CssValue> {
        let mut args = Vec::new();

        while !self.is_at_end() && self.peek() != Some(&CssToken::CloseParen) {
            if let Some(arg) = self.consume_value_component()? {
                args.push(arg);
            }

            // Handle commas and whitespace
            if self.peek() == Some(&CssToken::Comma) {
                self.advance();
                self.consume_whitespace();
            } else if !matches!(self.peek(), Some(CssToken::CloseParen) | None) {
                // Continue consuming
            } else {
                break;
            }
        }

        if self.peek() == Some(&CssToken::CloseParen) {
            self.advance();
        }

        Ok(CssValue::Function(CssFunction { name, arguments: args.clone(), args }))
    }

    /// Consume !important
    fn consume_important(&mut self) -> bool {
        if self.peek() == Some(&CssToken::Delim('!')) {
            self.advance();
            self.consume_whitespace();

            if let Some(CssToken::Ident(s)) = self.peek() {
                if s.eq_ignore_ascii_case("important") {
                    self.advance();
                    return true;
                }
            }
        }
        false
    }

    /// Peek if next tokens are !important
    fn peek_important(&self) -> bool {
        let saved_pos = self.position;
        let mut pos = saved_pos;

        if matches!(self.tokens.get(pos), Some(CssToken::Delim('!'))) {
            pos += 1;
            // Skip whitespace
            while matches!(self.tokens.get(pos), Some(CssToken::Whitespace)) {
                pos += 1;
            }
            if matches!(self.tokens.get(pos), Some(CssToken::Ident(s)) if s.eq_ignore_ascii_case("important")) {
                return true;
            }
        }

        false
    }

    /// Consume selector
    fn consume_selector(&mut self) -> Result<Selector> {
        // Simplified selector parsing
        // Full implementation would be in selectors.rs
        use super::selectors::SelectorPart;

        let mut parts = Vec::new();
        let mut combinator = None;

        loop {
            self.consume_whitespace();

            match self.peek() {
                Some(CssToken::Ident(name)) => {
                    let name = name.clone();
                    self.advance();
                    parts.push(SelectorPart::Type(name));
                }
                Some(CssToken::Hash(id, _)) => {
                    let id = id.clone();
                    self.advance();
                    parts.push(SelectorPart::Id(id));
                }
                Some(CssToken::Delim('.')) => {
                    self.advance();
                    if let Some(CssToken::Ident(class)) = self.peek() {
                        let class = class.clone();
                        self.advance();
                        parts.push(SelectorPart::Class(class));
                    }
                }
                Some(CssToken::Delim('*')) => {
                    self.advance();
                    parts.push(SelectorPart::Universal);
                }
                Some(CssToken::OpenBracket) => {
                    self.advance();
                    let attr = self.consume_attribute_selector()?;
                    parts.push(attr);
                }
                Some(CssToken::Colon) => {
                    self.advance();
                    if self.peek() == Some(&CssToken::Colon) {
                        self.advance(); // :: pseudo-element
                        if let Some(CssToken::Ident(name)) = self.peek() {
                            let name = name.clone();
                            self.advance();
                            parts.push(SelectorPart::PseudoElement(name));
                        }
                    } else {
                        // : pseudo-class
                        if let Some(CssToken::Ident(name)) = self.peek() {
                            let name = name.clone();
                            self.advance();
                            parts.push(SelectorPart::PseudoClass(name));
                        }
                    }
                }
                Some(CssToken::Comma) | Some(CssToken::OpenBrace) | None => break,
                Some(CssToken::Delim(c)) if matches!(c, '>' | '+' | '~') => {
                    combinator = Some(match c {
                        '>' => super::selectors::Combinator::Child,
                        '+' => super::selectors::Combinator::Adjacent,
                        '~' => super::selectors::Combinator::GeneralSibling,
                        _ => unreachable!(),
                    });
                    self.advance();
                    parts.push(SelectorPart::Combinator(combinator.unwrap()));
                }
                Some(CssToken::Whitespace) => {
                    self.consume_whitespace();
                    if !self.is_at_end() && !matches!(self.peek(), Some(CssToken::Comma) | Some(CssToken::OpenBrace)) {
                        parts.push(SelectorPart::Combinator(super::selectors::Combinator::Descendant));
                    }
                }
                _ => {
                    self.advance(); // Skip unknown
                }
            }
        }

        if parts.is_empty() {
            return Err(PdfError::Parse("Empty selector".to_string()));
        }

        Ok(Selector { parts })
    }

    /// Consume attribute selector [attr=value]
    fn consume_attribute_selector(&mut self) -> Result<super::selectors::SelectorPart> {
        self.consume_whitespace();

        let name = match self.peek() {
            Some(CssToken::Ident(n)) => {
                let n = n.clone();
                self.advance();
                n
            }
            _ => return Err(PdfError::Parse("Expected attribute name".to_string())),
        };

        self.consume_whitespace();

        let mut op = None;
        let mut value = None;

        match self.peek() {
            Some(CssToken::Delim('=')) => {
                op = Some("=");
                self.advance();
            }
            Some(CssToken::Delim('~')) |
            Some(CssToken::Delim('|')) |
            Some(CssToken::Delim('^')) |
            Some(CssToken::Delim('$')) |
            Some(CssToken::Delim('*')) => {
                if let Some(CssToken::Delim(c)) = self.peek() {
                    let c = *c;
                    self.advance();
                    if self.peek() == Some(&CssToken::Delim('=')) {
                        self.advance();
                        op = Some(match c {
                            '~' => "~=",
                            '|' => "|=",
                            '^' => "^=",
                            '$' => "$=",
                            '*' => "*=",
                            _ => "=",
                        });
                    }
                }
            }
            _ => {}
        }

        if op.is_some() {
            self.consume_whitespace();

            value = match self.peek() {
                Some(CssToken::Ident(v)) => {
                    let v = v.clone();
                    self.advance();
                    Some(v)
                }
                Some(CssToken::String(v)) => {
                    let v = v.clone();
                    self.advance();
                    Some(v)
                }
                _ => None,
            };
        }

        self.consume_whitespace();

        if self.peek() == Some(&CssToken::CloseBracket) {
            self.advance();
        }

        let attr_op = op.map(|s| match s {
            "=" => super::selectors::AttributeOp::Equals,
            "~=" => super::selectors::AttributeOp::Contains,
            "|=" => super::selectors::AttributeOp::Dash,
            "^=" => super::selectors::AttributeOp::StartsWith,
            "$=" => super::selectors::AttributeOp::EndsWith,
            "*=" => super::selectors::AttributeOp::Substring,
            _ => super::selectors::AttributeOp::Present,
        });
        
        Ok(super::selectors::SelectorPart::Attribute {
            name,
            op: attr_op.unwrap_or(super::selectors::AttributeOp::Present),
            value,
        })
    }

    /// Consume prelude until block or semicolon
    fn consume_prelude(&mut self) {
        while !self.is_at_end() {
            match self.peek() {
                Some(CssToken::OpenBrace) | Some(CssToken::Semicolon) => break,
                _ => self.advance(),
            }
        }
    }

    /// Consume prelude as string
    fn consume_prelude_string(&mut self) -> String {
        let start = self.position;
        self.consume_prelude();
        // Convert tokens back to string (simplified)
        format!("[{} tokens]", self.position - start)
    }

    /// Consume simple block { ... }
    fn consume_simple_block(&mut self) -> Result<()> {
        if self.peek() != Some(&CssToken::OpenBrace) {
            return Err(PdfError::Parse("Expected {".to_string()));
        }

        self.advance();
        let mut depth = 1;

        while !self.is_at_end() && depth > 0 {
            match self.peek() {
                Some(CssToken::OpenBrace) => depth += 1,
                Some(CssToken::CloseBrace) => depth -= 1,
                _ => {}
            }
            self.advance();
        }

        Ok(())
    }

    /// Consume whitespace tokens
    fn consume_whitespace(&mut self) {
        while self.peek() == Some(&CssToken::Whitespace) {
            self.advance();
        }
    }

    /// Check if at end
    fn is_at_end(&self) -> bool {
        self.position >= self.tokens.len() ||
        matches!(self.tokens.get(self.position), Some(CssToken::EOF))
    }

    /// Peek at current token
    fn peek(&self) -> Option<&CssToken> {
        self.tokens.get(self.position)
    }

    /// Advance to next token
    fn advance(&mut self) {
        if !self.is_at_end() {
            self.position += 1;
        }
    }
}

impl<'a> CssTokenizer<'a> {
    fn collect(&mut self) -> Vec<CssToken> {
        let mut tokens = Vec::new();
        loop {
            let token = self.next_token();
            let is_eof = matches!(token, CssToken::EOF);
            tokens.push(token);
            if is_eof {
                break;
            }
        }
        tokens
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_rule() {
        let css = "body { color: red; }";
        let mut parser = CssParser::new(css);
        let stylesheet = parser.parse().unwrap();

        assert_eq!(stylesheet.rules.len(), 1);
    }

    #[test]
    fn test_parse_multiple_declarations() {
        let css = r#"
            body {
                color: black;
                background: white;
                font-size: 16px;
            }
        "#;

        let mut parser = CssParser::new(css);
        let stylesheet = parser.parse().unwrap();

        if let Some(Rule::Style(rule)) = stylesheet.rules.first() {
            assert_eq!(rule.declarations.len(), 3);
            assert_eq!(rule.declarations[0].name, "color");
        } else {
            panic!("Expected style rule");
        }
    }

    #[test]
    fn test_parse_selector_types() {
        let cases = vec![
            ("body", "type selector"),
            ("#id", "ID selector"),
            (".class", "class selector"),
            ("*", "universal selector"),
            ("div.class", "compound"),
            ("div > span", "child combinator"),
            ("div span", "descendant combinator"),
            ("h1 + p", "adjacent sibling"),
            ("h1 ~ p", "general sibling"),
        ];

        for (selector, desc) in cases {
            let css = format!("{} {{ color: red; }}", selector);
            let mut parser = CssParser::new(&css);
            let result = parser.parse();
            assert!(result.is_ok(), "Failed to parse {}: {}", desc, selector);
        }
    }

    #[test]
    fn test_parse_values() {
        let cases = vec![
            "width: 100px",
            "color: #ff0000",
            "margin: 10px 20px",
            "background: url(image.jpg)",
            "font-family: Arial, sans-serif",
            "content: \"test\"",
        ];

        for case in cases {
            let css = format!("div {{ {}; }}", case);
            let mut parser = CssParser::new(&css);
            assert!(parser.parse().is_ok(), "Failed to parse: {}", case);
        }
    }

    #[test]
    fn test_parse_page_rule() {
        let css = r#"
            @page {
                size: A4;
                margin: 2cm;
                
                @top-center {
                    content: "Header";
                }
            }
        "#;

        let mut parser = CssParser::new(css);
        let stylesheet = parser.parse().unwrap();

        assert_eq!(stylesheet.rules.len(), 1);

        if let Some(Rule::At(AtRule::Page(page))) = stylesheet.rules.first() {
            assert!(page.declarations.iter().any(|d| d.name == "size"));
            assert_eq!(page.margin_boxes.len(), 1);
        } else {
            panic!("Expected @page rule");
        }
    }

    #[test]
    fn test_parse_media_rule() {
        let css = r#"
            @media print {
                body { color: black; }
            }
        "#;

        let mut parser = CssParser::new(css);
        let stylesheet = parser.parse().unwrap();

        assert_eq!(stylesheet.rules.len(), 1);
    }

    #[test]
    fn test_parse_important() {
        let css = "body { color: red !important; }";
        let mut parser = CssParser::new(css);
        let stylesheet = parser.parse().unwrap();

        if let Some(Rule::Style(rule)) = stylesheet.rules.first() {
            assert!(rule.is_important("color"));
        }
    }

    #[test]
    fn test_parse_pseudo_classes() {
        let cases = vec![
            "a:hover",
            "li:first-child",
            "input:checked",
            "div:not(.class)",
        ];

        for selector in cases {
            let css = format!("{} {{ color: red; }}", selector);
            let mut parser = CssParser::new(&css);
            assert!(parser.parse().is_ok(), "Failed to parse: {}", selector);
        }
    }

    #[test]
    fn test_parse_attribute_selectors() {
        let cases = vec![
            "[attr]",
            "[attr=value]",
            "[attr~=value]",
            "[attr|=value]",
            "[attr^=value]",
            "[attr$=value]",
            "[attr*=value]",
        ];

        for selector in cases {
            let css = format!("{} {{ color: red; }}", selector);
            let mut parser = CssParser::new(&css);
            assert!(parser.parse().is_ok(), "Failed to parse: {}", selector);
        }
    }

    #[test]
    fn test_parse_calc() {
        let css = "div { width: calc(100% - 20px); }";
        let mut parser = CssParser::new(css);
        assert!(parser.parse().is_ok());
    }

    #[test]
    fn test_parse_keyframes() {
        let css = r#"
            @keyframes slide {
                from { transform: translateX(0); }
                to { transform: translateX(100px); }
            }
        "#;

        let mut parser = CssParser::new(css);
        let stylesheet = parser.parse().unwrap();

        assert_eq!(stylesheet.rules.len(), 1);
    }

    #[test]
    fn test_parse_font_face() {
        let css = r#"
            @font-face {
                font-family: MyFont;
                src: url(font.woff);
            }
        "#;

        let mut parser = CssParser::new(css);
        let stylesheet = parser.parse().unwrap();

        assert_eq!(stylesheet.rules.len(), 1);
    }

    #[test]
    fn test_parse_custom_properties() {
        let css = r#"
            :root {
                --main-color: #06c;
                --secondary-color: #fff;
            }
        "#;

        let mut parser = CssParser::new(css);
        assert!(parser.parse().is_ok());
    }
}
