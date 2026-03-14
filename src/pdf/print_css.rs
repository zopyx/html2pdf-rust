//! PrintCSS / CSS Paged Media support for PDF generation
//!
//! Implements CSS Paged Media Module Level 3 features including:
//! - @page rules with selectors and named pages
//! - Margin boxes (@top-left, @bottom-center, etc.)
//! - Page breaks and fragmentation
//! - Running headers/footers
//! - Generated content for paged media
//! - Page counters and cross-references

use crate::css::{PageRule, PageSelector, PageMarginBox, MarginBoxType};
use crate::css::parser::Declaration;
use crate::css::CssValue;
use crate::types::{PaperSize, Orientation, Margins, Rect};
use std::collections::HashMap;

/// Page context for a single page
#[derive(Debug, Clone)]
pub struct PageContext {
    /// Page number (1-indexed)
    pub page_number: u32,
    /// Total pages in document
    pub total_pages: u32,
    /// Whether this is the first page
    pub is_first: bool,
    /// Whether this is a left page (verso) or right page (recto)
    pub is_left: bool,
    /// Whether this page is blank
    pub is_blank: bool,
    /// Named page type (if specified)
    pub named_page: Option<String>,
    /// Page size
    pub size: PageSize,
    /// Page margins
    pub margins: Margins,
    /// Running strings (set by string-set property)
    pub running_strings: HashMap<String, String>,
}

impl PageContext {
    /// Create a new page context
    pub fn new(page_number: u32, total_pages: u32) -> Self {
        let is_first = page_number == 1;
        let is_left = page_number % 2 == 0; // Even pages are left (verso)
        
        Self {
            page_number,
            total_pages,
            is_first,
            is_left,
            is_blank: false,
            named_page: None,
            size: PageSize::default(),
            margins: Margins::all(72.0),
            running_strings: HashMap::new(),
        }
    }

    /// Get page selector types that apply to this page
    pub fn applicable_selectors(&self) -> Vec<PageSelector> {
        let mut selectors = vec![PageSelector::Named("".to_string())]; // Default page
        
        if self.is_first {
            selectors.push(PageSelector::First);
        }
        if self.is_left {
            selectors.push(PageSelector::Left);
        } else {
            selectors.push(PageSelector::Right);
        }
        if self.is_blank {
            selectors.push(PageSelector::Blank);
        }
        if let Some(ref name) = self.named_page {
            selectors.push(PageSelector::Named(name.clone()));
        }
        
        selectors
    }

    /// Check if this page matches the given selectors
    pub fn matches_selectors(&self, selectors: &[PageSelector]) -> bool {
        if selectors.is_empty() {
            return true; // No selectors = matches all pages
        }
        
        let applicable = self.applicable_selectors();
        selectors.iter().any(|s| applicable.contains(s))
    }

    /// Get the content area rectangle
    pub fn content_area(&self) -> Rect {
        Rect::new(
            self.margins.left,
            self.margins.top,
            self.size.width - self.margins.left - self.margins.right,
            self.size.height - self.margins.top - self.margins.bottom,
        )
    }

    /// Get a running string by name
    pub fn get_running_string(&self, name: &str) -> Option<&str> {
        self.running_strings.get(name).map(|s| s.as_str())
    }
}

/// Page size configuration
#[derive(Debug, Clone)]
pub struct PageSize {
    /// Page width in points
    pub width: f32,
    /// Page height in points
    pub height: f32,
}

impl Default for PageSize {
    fn default() -> Self {
        let (w, h) = PaperSize::A4.size();
        Self { width: w, height: h }
    }
}

impl PageSize {
    /// Create from paper size and orientation
    pub fn from_paper_size(paper_size: PaperSize, orientation: Orientation) -> Self {
        let (mut w, mut h) = paper_size.size();
        if orientation == Orientation::Landscape {
            std::mem::swap(&mut w, &mut h);
        }
        Self { width: w, height: h }
    }
}

/// Page master configuration from @page rules
#[derive(Debug, Clone)]
pub struct PageMaster {
    /// Page selectors this master applies to
    pub selectors: Vec<PageSelector>,
    /// Named page identifier
    pub named_page: Option<String>,
    /// Page size (overrides default)
    pub size: Option<PageSize>,
    /// Page margins
    pub margins: Margins,
    /// Margin boxes
    pub margin_boxes: HashMap<MarginBoxType, MarginBoxContent>,
}

impl PageMaster {
    /// Create a new page master from a page rule
    pub fn from_page_rule(rule: &PageRule) -> Self {
        let mut master = Self {
            selectors: rule.selectors.clone(),
            named_page: None,
            size: None,
            margins: Margins::all(72.0),
            margin_boxes: HashMap::new(),
        };

        // Parse declarations
        for decl in &rule.declarations {
            match decl.name.as_str() {
                "size" => {
                    master.size = parse_size_declaration(&decl.value);
                }
                "margin" => {
                    master.margins = parse_margin_declaration(&decl.value);
                }
                "margin-top" => {
                    if let Some(val) = parse_length_value(&decl.value) {
                        master.margins.top = val;
                    }
                }
                "margin-right" => {
                    if let Some(val) = parse_length_value(&decl.value) {
                        master.margins.right = val;
                    }
                }
                "margin-bottom" => {
                    if let Some(val) = parse_length_value(&decl.value) {
                        master.margins.bottom = val;
                    }
                }
                "margin-left" => {
                    if let Some(val) = parse_length_value(&decl.value) {
                        master.margins.left = val;
                    }
                }
                _ => {}
            }
        }

        // Parse margin boxes
        for margin_box in &rule.margin_boxes {
            let content = MarginBoxContent::from_declarations(&margin_box.declarations);
            master.margin_boxes.insert(margin_box.box_type, content);
        }

        master
    }

    /// Apply this master to a page context
    pub fn apply_to(&self, context: &mut PageContext) {
        if let Some(ref size) = self.size {
            context.size = size.clone();
        }
        context.margins = self.margins;
    }
}

/// Content for a margin box
#[derive(Debug, Clone)]
pub struct MarginBoxContent {
    /// Content declarations
    pub content: Vec<MarginContentPart>,
    /// Font size for this margin box
    pub font_size: f32,
    /// Text color
    pub color: Option<crate::types::Color>,
    /// Text alignment
    pub text_align: TextAlign,
    /// Vertical alignment
    pub vertical_align: VerticalAlign,
}

impl MarginBoxContent {
    /// Create from margin box declarations
    pub fn from_declarations(declarations: &[Declaration]) -> Self {
        let mut content = Vec::new();
        let mut font_size = 10.0;
        let mut color = None;
        let mut text_align = TextAlign::Center;
        let mut vertical_align = VerticalAlign::Middle;

        for decl in declarations {
            match decl.name.as_str() {
                "content" => {
                    content = parse_content_value(&decl.value);
                }
                "font-size" => {
                    if let Some(size) = parse_length_value(&decl.value) {
                        font_size = size;
                    }
                }
                "color" => {
                    color = parse_color_value(&decl.value);
                }
                "text-align" => {
                    text_align = parse_text_align(&decl.value);
                }
                "vertical-align" => {
                    vertical_align = parse_vertical_align(&decl.value);
                }
                _ => {}
            }
        }

        Self {
            content,
            font_size,
            color,
            text_align,
            vertical_align,
        }
    }

    /// Generate the actual text content for this margin box given a page context
    pub fn generate_text(&self, context: &PageContext) -> String {
        let mut result = String::new();
        
        for part in &self.content {
            match part {
                MarginContentPart::Text(text) => result.push_str(text),
                MarginContentPart::StringRef(name) => {
                    if let Some(value) = context.get_running_string(name) {
                        result.push_str(value);
                    }
                }
                MarginContentPart::PageCounter => {
                    result.push_str(&context.page_number.to_string());
                }
                MarginContentPart::PagesCounter => {
                    result.push_str(&context.total_pages.to_string());
                }
                MarginContentPart::TargetCounter { .. } => {
                    // Target counters require cross-reference resolution
                    // Would need access to the document structure
                }
                MarginContentPart::Leader(char) => {
                    result.push(*char);
                }
            }
        }
        
        result
    }
}

impl Default for MarginBoxContent {
    fn default() -> Self {
        Self {
            content: Vec::new(),
            font_size: 10.0,
            color: None,
            text_align: TextAlign::Center,
            vertical_align: VerticalAlign::Middle,
        }
    }
}

/// Parts of margin box content
#[derive(Debug, Clone)]
pub enum MarginContentPart {
    /// Plain text
    Text(String),
    /// Reference to a running string (string())
    StringRef(String),
    /// Current page number counter
    PageCounter,
    /// Total pages counter
    PagesCounter,
    /// Target counter for cross-references (target-counter())
    TargetCounter { selector: String, counter: String },
    /// Leader character for filling space
    Leader(char),
}

/// Text alignment for margin boxes
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TextAlign {
    Left,
    Center,
    Right,
    Justify,
}

/// Vertical alignment for margin boxes
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum VerticalAlign {
    Top,
    Middle,
    Bottom,
}

/// Page break types
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BreakType {
    Auto,
    Always,
    Avoid,
    Page,
    Column,
    Region,
    Left,
    Right,
    Recto,
    Verso,
}

impl Default for BreakType {
    fn default() -> Self {
        BreakType::Auto
    }
}

/// Page break inside values
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BreakInside {
    Auto,
    Avoid,
    AvoidPage,
    AvoidColumn,
    AvoidRegion,
}

impl Default for BreakInside {
    fn default() -> Self {
        BreakInside::Auto
    }
}

/// Running element (position: running())
#[derive(Debug, Clone)]
pub struct RunningElement {
    /// Name of the running element
    pub name: String,
    /// The content as HTML/XML string
    pub content: String,
}

/// String set declaration from CSS
#[derive(Debug, Clone)]
pub struct StringSet {
    /// Name of the string
    pub name: String,
    /// Value expression
    pub value: StringSetValue,
}

/// Value for string-set
#[derive(Debug, Clone)]
pub enum StringSetValue {
    /// Static text
    Text(String),
    /// Content from element (first-letter, first-line, before, after)
    Content(ContentPart),
    /// Element attribute
    Attr(String),
}

/// Content part selector
#[derive(Debug, Clone)]
pub enum ContentPart {
    Element,
    FirstLetter,
    FirstLine,
    Before,
    After,
}

/// Page counter manager
#[derive(Debug, Clone)]
pub struct PageCounter {
    /// Current page number
    pub current: u32,
    /// Counter resets at specific elements
    pub resets: HashMap<String, u32>,
    /// Named counters
    pub named_counters: HashMap<String, Vec<u32>>,
}

impl PageCounter {
    /// Create a new page counter starting at 1
    pub fn new() -> Self {
        Self {
            current: 1,
            resets: HashMap::new(),
            named_counters: HashMap::new(),
        }
    }

    /// Reset the page counter
    pub fn reset(&mut self, value: u32) {
        self.current = value;
    }

    /// Increment the page counter
    pub fn increment(&mut self) {
        self.current += 1;
    }

    /// Set a named counter
    pub fn set_named_counter(&mut self, name: &str, value: u32) {
        self.named_counters
            .entry(name.to_string())
            .or_insert_with(Vec::new)
            .push(value);
    }

    /// Get a named counter value
    pub fn get_named_counter(&self, name: &str) -> Option<u32> {
        self.named_counters.get(name).and_then(|v| v.last().copied())
    }

    /// Reset a named counter
    pub fn reset_named_counter(&mut self, name: &str, value: u32) {
        if let Some(counter) = self.named_counters.get_mut(name) {
            counter.clear();
            counter.push(value);
        } else {
            self.named_counters.insert(name.to_string(), vec![value]);
        }
    }
}

impl Default for PageCounter {
    fn default() -> Self {
        Self::new()
    }
}

/// Bookmark/Outline entry for PDF
#[derive(Debug, Clone)]
pub struct Bookmark {
    /// Title text
    pub title: String,
    /// Page number (1-indexed)
    pub page: u32,
    /// Nesting level (0 = top level)
    pub level: u32,
    /// Children bookmarks
    pub children: Vec<Bookmark>,
}

impl Bookmark {
    /// Create a new bookmark
    pub fn new(title: impl Into<String>, page: u32, level: u32) -> Self {
        Self {
            title: title.into(),
            page,
            level,
            children: Vec::new(),
        }
    }

    /// Add a child bookmark
    pub fn add_child(&mut self, child: Bookmark) {
        self.children.push(child);
    }
}

/// Parse size declaration from @page rule
fn parse_size_declaration(value: &CssValue) -> Option<PageSize> {
    match value {
        CssValue::Ident(name) => {
            // Named paper sizes
            let paper_size = match name.as_str() {
                "a4" | "A4" => PaperSize::A4,
                "a3" | "A3" => PaperSize::A3,
                "a5" | "A5" => PaperSize::A5,
                "letter" | "Letter" => PaperSize::Letter,
                "legal" | "Legal" => PaperSize::Legal,
                _ => return None,
            };
            let (w, h) = paper_size.size();
            Some(PageSize { width: w, height: h })
        }
        CssValue::List(list) if list.len() >= 2 => {
            // size: width height
            if let (Some(w), Some(h)) = (parse_length_value(&list[0]), parse_length_value(&list[1])) {
                Some(PageSize { width: w, height: h })
            } else {
                None
            }
        }
        _ => None,
    }
}

/// Parse margin declaration
fn parse_margin_declaration(value: &CssValue) -> Margins {
    match value {
        CssValue::Length(n, unit) => {
            let val = unit.to_pt(*n, 12.0);
            Margins::all(val)
        }
        CssValue::List(list) => {
            let values: Vec<f32> = list.iter()
                .filter_map(|v| parse_length_value(v))
                .collect();
            
            match values.len() {
                1 => Margins::all(values[0]),
                2 => Margins::symmetric(values[0], values[1]),
                4 => Margins::new(values[0], values[1], values[2], values[3]),
                _ => Margins::all(72.0),
            }
        }
        _ => Margins::all(72.0),
    }
}

/// Parse a CSS length value to points
fn parse_length_value(value: &CssValue) -> Option<f32> {
    match value {
        CssValue::Length(n, unit) => Some(unit.to_pt(*n, 12.0)),
        CssValue::Number(n) => Some(*n),
        CssValue::Integer(n) => Some(*n as f32),
        _ => None,
    }
}

/// Parse content value for margin boxes
fn parse_content_value(value: &CssValue) -> Vec<MarginContentPart> {
    let mut result = Vec::new();
    
    match value {
        CssValue::String(s) => {
            result.push(MarginContentPart::Text(s.clone()));
        }
        CssValue::Ident(s) if s == "none" => {}
        CssValue::Function(func) => {
            match func.name.as_str() {
                "string" => {
                    if let Some(arg) = func.arguments.first() {
                        if let CssValue::Ident(name) | CssValue::String(name) = arg {
                            result.push(MarginContentPart::StringRef(name.clone()));
                        }
                    }
                }
                "counter" => {
                    if let Some(arg) = func.arguments.first() {
                        match arg {
                            CssValue::Ident(name) | CssValue::String(name) => {
                                if name == "page" {
                                    result.push(MarginContentPart::PageCounter);
                                } else if name == "pages" {
                                    result.push(MarginContentPart::PagesCounter);
                                }
                            }
                            _ => {}
                        }
                    }
                }
                "leader" => {
                    // Leader pattern for filling space
                    let leader_char = func.arguments.first()
                        .and_then(|v| match v {
                            CssValue::String(s) => s.chars().next(),
                            CssValue::Literal(s) => s.chars().next(),
                            _ => Some('.'),
                        })
                        .unwrap_or('.');
                    result.push(MarginContentPart::Leader(leader_char));
                }
                "target-counter" => {
                    // target-counter(url, counter-name)
                    if func.arguments.len() >= 2 {
                        if let (CssValue::String(url), CssValue::Ident(counter)) = 
                            (&func.arguments[0], &func.arguments[1]) {
                            result.push(MarginContentPart::TargetCounter {
                                selector: url.clone(),
                                counter: counter.clone(),
                            });
                        }
                    }
                }
                _ => {}
            }
        }
        CssValue::List(list) => {
            // Concatenate multiple content parts
            for item in list {
                result.extend(parse_content_value(item));
            }
        }
        _ => {}
    }
    
    result
}

/// Parse color value
fn parse_color_value(value: &CssValue) -> Option<crate::types::Color> {
    match value {
        CssValue::HexColor(hex) => {
            let r = ((*hex >> 16) & 0xFF) as u8;
            let g = ((*hex >> 8) & 0xFF) as u8;
            let b = (*hex & 0xFF) as u8;
            Some(crate::types::Color::new(r, g, b))
        }
        CssValue::Ident(name) => {
            match name.as_str() {
                "black" => Some(crate::types::Color::BLACK),
                "white" => Some(crate::types::Color::WHITE),
                "red" => Some(crate::types::Color::RED),
                "green" => Some(crate::types::Color::GREEN),
                "blue" => Some(crate::types::Color::BLUE),
                _ => None,
            }
        }
        _ => None,
    }
}

/// Parse text-align value
fn parse_text_align(value: &CssValue) -> TextAlign {
    match value {
        CssValue::Ident(name) => match name.as_str() {
            "left" => TextAlign::Left,
            "center" => TextAlign::Center,
            "right" => TextAlign::Right,
            "justify" => TextAlign::Justify,
            _ => TextAlign::Center,
        },
        _ => TextAlign::Center,
    }
}

/// Parse vertical-align value
fn parse_vertical_align(value: &CssValue) -> VerticalAlign {
    match value {
        CssValue::Ident(name) => match name.as_str() {
            "top" => VerticalAlign::Top,
            "middle" => VerticalAlign::Middle,
            "bottom" => VerticalAlign::Bottom,
            _ => VerticalAlign::Middle,
        },
        _ => VerticalAlign::Middle,
    }
}

/// Parse break-before/after value
pub fn parse_break_value(value: &CssValue) -> BreakType {
    match value {
        CssValue::Ident(name) => match name.as_str() {
            "auto" => BreakType::Auto,
            "always" => BreakType::Always,
            "avoid" => BreakType::Avoid,
            "page" => BreakType::Page,
            "left" => BreakType::Left,
            "right" => BreakType::Right,
            "recto" => BreakType::Recto,
            "verso" => BreakType::Verso,
            "column" => BreakType::Column,
            "region" => BreakType::Region,
            _ => BreakType::Auto,
        },
        _ => BreakType::Auto,
    }
}

/// Parse break-inside value
pub fn parse_break_inside_value(value: &CssValue) -> BreakInside {
    match value {
        CssValue::Ident(name) => match name.as_str() {
            "auto" => BreakInside::Auto,
            "avoid" => BreakInside::Avoid,
            "avoid-page" => BreakInside::AvoidPage,
            "avoid-column" => BreakInside::AvoidColumn,
            "avoid-region" => BreakInside::AvoidRegion,
            _ => BreakInside::Auto,
        },
        _ => BreakInside::Auto,
    }
}

/// Parse orphans/widows value
pub fn parse_orphans_widows_value(value: &CssValue) -> u32 {
    match value {
        CssValue::Integer(n) => *n as u32,
        CssValue::Number(n) => *n as u32,
        _ => 2, // Default value
    }
}

/// Get the rectangle for a margin box
pub fn get_margin_box_rect(box_type: MarginBoxType, page_size: &PageSize, margins: &Margins) -> Rect {
    // Corner boxes are 1em square (approximately 12pt)
    let corner_size = 12.0;
    
    match box_type {
        MarginBoxType::TopLeftCorner => Rect::new(
            0.0,
            page_size.height - margins.top,
            margins.left.min(corner_size),
            margins.top.min(corner_size),
        ),
        MarginBoxType::TopLeft => Rect::new(
            margins.left,
            page_size.height - margins.top,
            (page_size.width - margins.left - margins.right) / 3.0,
            margins.top,
        ),
        MarginBoxType::TopCenter => Rect::new(
            margins.left + (page_size.width - margins.left - margins.right) / 3.0,
            page_size.height - margins.top,
            (page_size.width - margins.left - margins.right) / 3.0,
            margins.top,
        ),
        MarginBoxType::TopRight => Rect::new(
            margins.left + 2.0 * (page_size.width - margins.left - margins.right) / 3.0,
            page_size.height - margins.top,
            (page_size.width - margins.left - margins.right) / 3.0,
            margins.top,
        ),
        MarginBoxType::TopRightCorner => Rect::new(
            page_size.width - margins.right,
            page_size.height - margins.top,
            margins.right.min(corner_size),
            margins.top.min(corner_size),
        ),
        MarginBoxType::BottomLeftCorner => Rect::new(
            0.0,
            0.0,
            margins.left.min(corner_size),
            margins.bottom.min(corner_size),
        ),
        MarginBoxType::BottomLeft => Rect::new(
            margins.left,
            0.0,
            (page_size.width - margins.left - margins.right) / 3.0,
            margins.bottom,
        ),
        MarginBoxType::BottomCenter => Rect::new(
            margins.left + (page_size.width - margins.left - margins.right) / 3.0,
            0.0,
            (page_size.width - margins.left - margins.right) / 3.0,
            margins.bottom,
        ),
        MarginBoxType::BottomRight => Rect::new(
            margins.left + 2.0 * (page_size.width - margins.left - margins.right) / 3.0,
            0.0,
            (page_size.width - margins.left - margins.right) / 3.0,
            margins.bottom,
        ),
        MarginBoxType::BottomRightCorner => Rect::new(
            page_size.width - margins.right,
            0.0,
            margins.right.min(corner_size),
            margins.bottom.min(corner_size),
        ),
        MarginBoxType::LeftTop => Rect::new(
            0.0,
            page_size.height - margins.top - corner_size,
            margins.left,
            corner_size,
        ),
        MarginBoxType::LeftMiddle => Rect::new(
            0.0,
            margins.bottom + (page_size.height - margins.top - margins.bottom) / 2.0 - corner_size / 2.0,
            margins.left,
            corner_size,
        ),
        MarginBoxType::LeftBottom => Rect::new(
            0.0,
            margins.bottom,
            margins.left,
            corner_size,
        ),
        MarginBoxType::RightTop => Rect::new(
            page_size.width - margins.right,
            page_size.height - margins.top - corner_size,
            margins.right,
            corner_size,
        ),
        MarginBoxType::RightMiddle => Rect::new(
            page_size.width - margins.right,
            margins.bottom + (page_size.height - margins.top - margins.bottom) / 2.0 - corner_size / 2.0,
            margins.right,
            corner_size,
        ),
        MarginBoxType::RightBottom => Rect::new(
            page_size.width - margins.right,
            margins.bottom,
            margins.right,
            corner_size,
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::css::PageRule;

    #[test]
    fn test_page_context() {
        let ctx = PageContext::new(1, 10);
        assert_eq!(ctx.page_number, 1);
        assert!(ctx.is_first);
        assert!(!ctx.is_left); // First page is right (recto)
        
        let ctx2 = PageContext::new(2, 10);
        assert!(ctx2.is_left); // Second page is left (verso)
    }

    #[test]
    fn test_page_master_from_rule() {
        let mut rule = PageRule::new();
        rule.add_selector(PageSelector::First);
        
        let master = PageMaster::from_page_rule(&rule);
        assert_eq!(master.selectors.len(), 1);
    }

    #[test]
    fn test_margin_box_rect() {
        let page_size = PageSize { width: 595.0, height: 842.0 };
        let margins = Margins::all(72.0);
        
        let top_center = get_margin_box_rect(MarginBoxType::TopCenter, &page_size, &margins);
        assert!(top_center.width > 0.0);
        assert!(top_center.height > 0.0);
    }

    #[test]
    fn test_parse_content_value() {
        let value = CssValue::String("Page ".to_string());
        let parts = parse_content_value(&value);
        assert_eq!(parts.len(), 1);
    }

    #[test]
    fn test_page_counter() {
        let mut counter = PageCounter::new();
        assert_eq!(counter.current, 1);
        
        counter.increment();
        assert_eq!(counter.current, 2);
        
        counter.reset(10);
        assert_eq!(counter.current, 10);
    }
}
