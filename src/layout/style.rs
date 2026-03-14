//! Style Computation
//!
//! Handles CSS cascade, specificity, inheritance, and computed values.
//! Implements style resolution from parsed CSS rules.

use crate::css::{
    Declaration, Rule, Selector, SelectorPart, Stylesheet,
    CssValue, Unit,
};
use crate::css::parser::StyleRule;
use crate::html::{Element, Node};
use crate::layout::box_model::BoxType;
use crate::types::{Color, Length};

/// Computed style for an element
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ComputedStyle {
    // Layout properties
    pub display: Display,
    pub position: Position,
    pub float: Float,
    pub clear: Clear,
    
    // Box model
    pub width: Length,
    pub height: Length,
    pub min_width: Length,
    pub min_height: Length,
    pub max_width: Length,
    pub max_height: Length,
    
    pub margin_top: Length,
    pub margin_right: Length,
    pub margin_bottom: Length,
    pub margin_left: Length,
    
    pub padding_top: Length,
    pub padding_right: Length,
    pub padding_bottom: Length,
    pub padding_left: Length,
    
    pub border_top_width: Length,
    pub border_right_width: Length,
    pub border_bottom_width: Length,
    pub border_left_width: Length,
    
    pub border_top_color: Color,
    pub border_right_color: Color,
    pub border_bottom_color: Color,
    pub border_left_color: Color,
    
    pub border_top_style: BorderStyle,
    pub border_right_style: BorderStyle,
    pub border_bottom_style: BorderStyle,
    pub border_left_style: BorderStyle,
    
    // Typography
    pub font_family: Vec<String>,
    pub font_size: Length,
    pub font_weight: FontWeight,
    pub font_style: FontStyle,
    pub line_height: LineHeight,
    
    pub color: Color,
    pub text_align: TextAlign,
    pub text_decoration: TextDecoration,
    pub text_transform: TextTransform,
    pub white_space: WhiteSpace,
    pub word_wrap: WordWrap,
    
    pub letter_spacing: Length,
    pub word_spacing: Length,
    pub text_indent: Length,
    
    // Visual
    pub background_color: Color,
    pub opacity: f32,
    pub visibility: Visibility,
    pub overflow: Overflow,
    
    // Positioning offsets
    pub top: Length,
    pub right: Length,
    pub bottom: Length,
    pub left: Length,
    pub z_index: ZIndex,
    
    // PrintCSS
    pub page_break_before: PageBreak,
    pub page_break_after: PageBreak,
    pub page_break_inside: PageBreakInside,
    pub orphans: u32,
    pub widows: u32,
}

/// Display property value
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum Display {
    #[default]
    Inline,
    Block,
    InlineBlock,
    Flex,
    InlineFlex,
    Grid,
    InlineGrid,
    None,
    ListItem,
    Table,
    TableRow,
    TableCell,
}

impl Display {
    pub fn to_box_type(&self) -> BoxType {
        match self {
            Display::Block | Display::Flex | Display::Grid | Display::Table | Display::ListItem => BoxType::Block,
            Display::Inline | Display::InlineFlex | Display::InlineGrid => BoxType::Inline,
            Display::InlineBlock => BoxType::InlineBlock,
            Display::None => BoxType::Anonymous, // Will be filtered out
            _ => BoxType::Block,
        }
    }

    pub fn is_none(&self) -> bool {
        matches!(self, Display::None)
    }
}

/// Position property value
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum Position {
    #[default]
    Static,
    Relative,
    Absolute,
    Fixed,
    Sticky,
}

/// Float property value
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum Float {
    #[default]
    None,
    Left,
    Right,
}

/// Clear property value
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum Clear {
    #[default]
    None,
    Left,
    Right,
    Both,
}

/// Border style
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum BorderStyle {
    #[default]
    None,
    Hidden,
    Solid,
    Dashed,
    Dotted,
    Double,
    Groove,
    Ridge,
    Inset,
    Outset,
}

/// Font weight
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FontWeight {
    Normal,
    Bold,
    Bolder,
    Lighter,
    Number(u16),
}

impl Default for FontWeight {
    fn default() -> Self {
        FontWeight::Normal
    }
}

/// Font style
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum FontStyle {
    #[default]
    Normal,
    Italic,
    Oblique,
}

/// Line height
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LineHeight {
    Normal,
    Number(f32),
    Length(Length),
}

impl Default for LineHeight {
    fn default() -> Self {
        LineHeight::Normal
    }
}

/// Text alignment
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum TextAlign {
    #[default]
    Left,
    Center,
    Right,
    Justify,
}

/// Text decoration
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum TextDecoration {
    #[default]
    None,
    Underline,
    Overline,
    LineThrough,
}

/// Text transform
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum TextTransform {
    #[default]
    None,
    Capitalize,
    Uppercase,
    Lowercase,
}

/// White space handling
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum WhiteSpace {
    #[default]
    Normal,
    Nowrap,
    Pre,
    PreWrap,
    PreLine,
}

impl WhiteSpace {
    pub fn preserves_newlines(&self) -> bool {
        matches!(self, WhiteSpace::Pre | WhiteSpace::PreWrap | WhiteSpace::PreLine)
    }

    pub fn preserves_spaces(&self) -> bool {
        matches!(self, WhiteSpace::Pre | WhiteSpace::PreWrap)
    }

    pub fn collapses_spaces(&self) -> bool {
        matches!(self, WhiteSpace::Normal | WhiteSpace::Nowrap | WhiteSpace::PreLine)
    }

    pub fn wraps(&self) -> bool {
        matches!(self, WhiteSpace::Normal | WhiteSpace::PreWrap | WhiteSpace::PreLine)
    }
}

/// Word wrap
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum WordWrap {
    #[default]
    Normal,
    BreakWord,
    Anywhere,
}

/// Visibility
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum Visibility {
    #[default]
    Visible,
    Hidden,
    Collapse,
}

/// Overflow
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum Overflow {
    #[default]
    Visible,
    Hidden,
    Scroll,
    Auto,
}

/// Z-index
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ZIndex {
    Auto,
    Number(i32),
}

impl Default for ZIndex {
    fn default() -> Self {
        ZIndex::Auto
    }
}

/// Page break
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum PageBreak {
    #[default]
    Auto,
    Always,
    Avoid,
    Left,
    Right,
}

/// Page break inside
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum PageBreakInside {
    #[default]
    Auto,
    Avoid,
}

/// A matching rule with its specificity
#[derive(Debug, Clone)]
struct MatchingRule<'a> {
    rule: &'a StyleRule,
    specificity: Specificity,
}

/// CSS specificity: (a, b, c) where a = ID count, b = class/attribute/pseudo count, c = type count
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct Specificity(u32, u32, u32);

impl Specificity {
    fn new() -> Self {
        Specificity(0, 0, 0)
    }

    fn from_selector(selector: &Selector) -> Self {
        let mut spec = Specificity::new();
        for part in &selector.parts {
            match part {
                SelectorPart::Id(_) => spec.0 += 1,
                SelectorPart::Class(_) |
                SelectorPart::Attribute { .. } |
                SelectorPart::PseudoClass(_) => spec.1 += 1,
                SelectorPart::Type(_) |
                SelectorPart::PseudoElement(_) => spec.2 += 1,
                _ => {}
            }
        }
        spec
    }
}

/// Style resolver that matches CSS rules to elements
#[derive(Debug)]
pub struct StyleResolver {
    stylesheets: Vec<Stylesheet>,
    default_styles: Vec<(Selector, Vec<Declaration>)>,
}

impl StyleResolver {
    pub fn new() -> Self {
        let mut resolver = Self {
            stylesheets: Vec::new(),
            default_styles: Vec::new(),
        };
        resolver.init_default_styles();
        resolver
    }

    /// Add a stylesheet
    pub fn add_stylesheet(&mut self, stylesheet: Stylesheet) {
        self.stylesheets.push(stylesheet);
    }

    /// Compute styles for an element
    pub fn compute_style(&self, element: &Element, parent_style: Option<&ComputedStyle>) -> ComputedStyle {
        let mut style = ComputedStyle::default();

        // 1. Apply initial values
        self.apply_initial_values(&mut style);

        // 2. Apply parent-inherited values
        if let Some(parent) = parent_style {
            self.apply_inheritance(&mut style, parent);
        }

        // 3. Apply default browser styles (user-agent stylesheet)
        self.apply_default_styles(element, &mut style);

        // 4. Apply author styles (from stylesheets)
        self.apply_author_styles(element, &mut style);

        // 5. Handle 'inherit' and 'initial' keywords
        self.resolve_keywords(&mut style, parent_style);

        style
    }

    /// Resolve display type for an element
    pub fn resolve_display(&self, element: &Element) -> BoxType {
        let style = self.compute_style(element, None);
        style.display.to_box_type()
    }

    /// Apply initial values to all properties
    fn apply_initial_values(&self, style: &mut ComputedStyle) {
        // Layout
        style.display = Display::Inline;
        style.position = Position::Static;
        style.float = Float::None;
        style.clear = Clear::None;

        // Box model
        style.width = Length::Auto;
        style.height = Length::Auto;
        style.min_width = Length::Px(0.0);
        style.min_height = Length::Px(0.0);
        style.max_width = Length::Auto;
        style.max_height = Length::Auto;

        style.margin_top = Length::Px(0.0);
        style.margin_right = Length::Px(0.0);
        style.margin_bottom = Length::Px(0.0);
        style.margin_left = Length::Px(0.0);

        style.padding_top = Length::Px(0.0);
        style.padding_right = Length::Px(0.0);
        style.padding_bottom = Length::Px(0.0);
        style.padding_left = Length::Px(0.0);

        style.border_top_width = Length::Px(0.0);
        style.border_right_width = Length::Px(0.0);
        style.border_bottom_width = Length::Px(0.0);
        style.border_left_width = Length::Px(0.0);

        style.border_top_style = BorderStyle::None;
        style.border_right_style = BorderStyle::None;
        style.border_bottom_style = BorderStyle::None;
        style.border_left_style = BorderStyle::None;

        style.border_top_color = Color::BLACK;
        style.border_right_color = Color::BLACK;
        style.border_bottom_color = Color::BLACK;
        style.border_left_color = Color::BLACK;

        // Typography
        style.font_family = vec!["serif".to_string()];
        style.font_size = Length::Px(16.0); // Default 16px
        style.font_weight = FontWeight::Normal;
        style.font_style = FontStyle::Normal;
        style.line_height = LineHeight::Normal;

        style.color = Color::BLACK;
        style.text_align = TextAlign::Left;
        style.text_decoration = TextDecoration::None;
        style.text_transform = TextTransform::None;
        style.white_space = WhiteSpace::Normal;
        style.word_wrap = WordWrap::Normal;

        style.letter_spacing = Length::Px(0.0);
        style.word_spacing = Length::Px(0.0);
        style.text_indent = Length::Px(0.0);

        // Visual
        style.background_color = Color::TRANSPARENT;
        style.opacity = 1.0;
        style.visibility = Visibility::Visible;
        style.overflow = Overflow::Visible;

        // Positioning
        style.top = Length::Auto;
        style.right = Length::Auto;
        style.bottom = Length::Auto;
        style.left = Length::Auto;
        style.z_index = ZIndex::Auto;

        // PrintCSS
        style.page_break_before = PageBreak::Auto;
        style.page_break_after = PageBreak::Auto;
        style.page_break_inside = PageBreakInside::Auto;
        style.orphans = 2;
        style.widows = 2;
    }

    /// Apply inherited values from parent
    fn apply_inheritance(&self, style: &mut ComputedStyle, parent: &ComputedStyle) {
        // Inheritable properties
        style.color = parent.color;
        style.font_family = parent.font_family.clone();
        style.font_size = parent.font_size;
        style.font_weight = parent.font_weight;
        style.font_style = parent.font_style;
        style.line_height = parent.line_height;
        style.text_align = parent.text_align;
        style.text_decoration = parent.text_decoration;
        style.text_transform = parent.text_transform;
        style.white_space = parent.white_space;
        style.word_wrap = parent.word_wrap;
        style.letter_spacing = parent.letter_spacing;
        style.word_spacing = parent.word_spacing;
        style.visibility = parent.visibility;
        style.orphans = parent.orphans;
        style.widows = parent.widows;
    }

    /// Initialize default browser styles
    fn init_default_styles(&mut self) {
        use crate::css::SelectorPart;

        // Block elements
        let block_selector = Selector {
            parts: vec![SelectorPart::Type("body".to_string())],
        };
        self.default_styles.push((block_selector, vec![
            Declaration::new("display", CssValue::Ident("block".to_string())),
            Declaration::new("margin", CssValue::Length(8.0, Unit::Px)),
        ]));

        // Headings
        for i in 1..=6 {
            let selector = Selector {
                parts: vec![SelectorPart::Type(format!("h{}", i))],
            };
            let font_size = match i {
                1 => CssValue::Length(2.0, Unit::Em),
                2 => CssValue::Length(1.5, Unit::Em),
                3 => CssValue::Length(1.17, Unit::Em),
                4 => CssValue::Length(1.0, Unit::Em),
                5 => CssValue::Length(0.83, Unit::Em),
                _ => CssValue::Length(0.67, Unit::Em),
            };
            self.default_styles.push((selector, vec![
                Declaration::new("display", CssValue::Ident("block".to_string())),
                Declaration::new("font-weight", CssValue::Ident("bold".to_string())),
                Declaration::new("font-size", font_size),
                Declaration::new("margin-top", CssValue::Length(0.67, Unit::Em)),
                Declaration::new("margin-bottom", CssValue::Length(0.67, Unit::Em)),
            ]));
        }

        // Paragraph
        let p_selector = Selector {
            parts: vec![SelectorPart::Type("p".to_string())],
        };
        self.default_styles.push((p_selector, vec![
            Declaration::new("display", CssValue::Ident("block".to_string())),
            Declaration::new("margin-top", CssValue::Length(1.0, Unit::Em)),
            Declaration::new("margin-bottom", CssValue::Length(1.0, Unit::Em)),
        ]));

        // Div
        let div_selector = Selector {
            parts: vec![SelectorPart::Type("div".to_string())],
        };
        self.default_styles.push((div_selector, vec![
            Declaration::new("display", CssValue::Ident("block".to_string())),
        ]));

        // Span (inline)
        let span_selector = Selector {
            parts: vec![SelectorPart::Type("span".to_string())],
        };
        self.default_styles.push((span_selector, vec![
            Declaration::new("display", CssValue::Ident("inline".to_string())),
        ]));

        // Strong/Bold
        let strong_selector = Selector {
            parts: vec![SelectorPart::Type("strong".to_string())],
        };
        self.default_styles.push((strong_selector, vec![
            Declaration::new("font-weight", CssValue::Ident("bold".to_string())),
        ]));

        let b_selector = Selector {
            parts: vec![SelectorPart::Type("b".to_string())],
        };
        self.default_styles.push((b_selector, vec![
            Declaration::new("font-weight", CssValue::Ident("bold".to_string())),
        ]));

        // Em/Italic
        let em_selector = Selector {
            parts: vec![SelectorPart::Type("em".to_string())],
        };
        self.default_styles.push((em_selector, vec![
            Declaration::new("font-style", CssValue::Ident("italic".to_string())),
        ]));

        let i_selector = Selector {
            parts: vec![SelectorPart::Type("i".to_string())],
        };
        self.default_styles.push((i_selector, vec![
            Declaration::new("font-style", CssValue::Ident("italic".to_string())),
        ]));
    }

    /// Apply default styles matching the element
    fn apply_default_styles(&self, element: &Element, style: &mut ComputedStyle) {
        for (selector, declarations) in &self.default_styles {
            if selector.matches(element) {
                for decl in declarations {
                    self.apply_declaration(style, decl);
                }
            }
        }
    }

    /// Apply author styles from stylesheets
    fn apply_author_styles(&self, element: &Element, style: &mut ComputedStyle) {
        let mut matching_rules: Vec<MatchingRule> = Vec::new();

        // Collect all matching rules from all stylesheets
        for stylesheet in &self.stylesheets {
            for rule in &stylesheet.rules {
                if let Rule::Style(style_rule) = rule {
                    if style_rule.selector.matches(element) {
                        let specificity = Specificity::from_selector(&style_rule.selector);
                        matching_rules.push(MatchingRule {
                            rule: style_rule,
                            specificity,
                        });
                    }
                }
            }
        }

        // Sort by specificity (lowest to highest)
        matching_rules.sort_by(|a, b| a.specificity.cmp(&b.specificity));

        // Apply in order (later rules with higher specificity override)
        for matching in matching_rules {
            for decl in &matching.rule.declarations {
                self.apply_declaration(style, decl);
            }
        }
    }

    /// Apply a single declaration to the style
    fn apply_declaration(&self, style: &mut ComputedStyle, decl: &Declaration) {
        let name = decl.name.to_ascii_lowercase();

        match name.as_str() {
            "display" => style.display = parse_display(&decl.value),
            "position" => style.position = parse_position(&decl.value),
            "float" => style.float = parse_float(&decl.value),
            "clear" => style.clear = parse_clear(&decl.value),

            "width" => style.width = parse_length(&decl.value),
            "height" => style.height = parse_length(&decl.value),
            "min-width" => style.min_width = parse_length(&decl.value),
            "min-height" => style.min_height = parse_length(&decl.value),
            "max-width" => style.max_width = parse_length(&decl.value),
            "max-height" => style.max_height = parse_length(&decl.value),

            "margin" => {
                let (top, right, bottom, left) = parse_margin_or_padding(&decl.value);
                style.margin_top = top;
                style.margin_right = right;
                style.margin_bottom = bottom;
                style.margin_left = left;
            }
            "margin-top" => style.margin_top = parse_length(&decl.value),
            "margin-right" => style.margin_right = parse_length(&decl.value),
            "margin-bottom" => style.margin_bottom = parse_length(&decl.value),
            "margin-left" => style.margin_left = parse_length(&decl.value),

            "padding" => {
                let (top, right, bottom, left) = parse_margin_or_padding(&decl.value);
                style.padding_top = top;
                style.padding_right = right;
                style.padding_bottom = bottom;
                style.padding_left = left;
            }
            "padding-top" => style.padding_top = parse_length(&decl.value),
            "padding-right" => style.padding_right = parse_length(&decl.value),
            "padding-bottom" => style.padding_bottom = parse_length(&decl.value),
            "padding-left" => style.padding_left = parse_length(&decl.value),

            "border-width" => {
                let widths = parse_border_widths(&decl.value);
                style.border_top_width = widths.0;
                style.border_right_width = widths.1;
                style.border_bottom_width = widths.2;
                style.border_left_width = widths.3;
            }
            "border-top-width" => style.border_top_width = parse_length(&decl.value),
            "border-right-width" => style.border_right_width = parse_length(&decl.value),
            "border-bottom-width" => style.border_bottom_width = parse_length(&decl.value),
            "border-left-width" => style.border_left_width = parse_length(&decl.value),

            "border-color" => {
                let color = parse_color(&decl.value);
                style.border_top_color = color;
                style.border_right_color = color;
                style.border_bottom_color = color;
                style.border_left_color = color;
            }
            "border-top-color" => style.border_top_color = parse_color(&decl.value),
            "border-right-color" => style.border_right_color = parse_color(&decl.value),
            "border-bottom-color" => style.border_bottom_color = parse_color(&decl.value),
            "border-left-color" => style.border_left_color = parse_color(&decl.value),

            "border-style" => {
                let s = parse_border_style(&decl.value);
                style.border_top_style = s;
                style.border_right_style = s;
                style.border_bottom_style = s;
                style.border_left_style = s;
            }
            "border-top-style" => style.border_top_style = parse_border_style(&decl.value),
            "border-right-style" => style.border_right_style = parse_border_style(&decl.value),
            "border-bottom-style" => style.border_bottom_style = parse_border_style(&decl.value),
            "border-left-style" => style.border_left_style = parse_border_style(&decl.value),

            "font-family" => style.font_family = parse_font_family(&decl.value),
            "font-size" => style.font_size = parse_length(&decl.value),
            "font-weight" => style.font_weight = parse_font_weight(&decl.value),
            "font-style" => style.font_style = parse_font_style(&decl.value),
            "line-height" => style.line_height = parse_line_height(&decl.value),

            "color" => style.color = parse_color(&decl.value),
            "text-align" => style.text_align = parse_text_align(&decl.value),
            "text-decoration" => style.text_decoration = parse_text_decoration(&decl.value),
            "text-transform" => style.text_transform = parse_text_transform(&decl.value),
            "white-space" => style.white_space = parse_white_space(&decl.value),
            "word-wrap" => style.word_wrap = parse_word_wrap(&decl.value),

            "letter-spacing" => style.letter_spacing = parse_length(&decl.value),
            "word-spacing" => style.word_spacing = parse_length(&decl.value),
            "text-indent" => style.text_indent = parse_length(&decl.value),

            "background-color" => style.background_color = parse_color(&decl.value),
            "opacity" => style.opacity = parse_number(&decl.value, 1.0),
            "visibility" => style.visibility = parse_visibility(&decl.value),
            "overflow" => style.overflow = parse_overflow(&decl.value),

            "top" => style.top = parse_length(&decl.value),
            "right" => style.right = parse_length(&decl.value),
            "bottom" => style.bottom = parse_length(&decl.value),
            "left" => style.left = parse_length(&decl.value),
            "z-index" => style.z_index = parse_z_index(&decl.value),

            "page-break-before" => style.page_break_before = parse_page_break(&decl.value),
            "page-break-after" => style.page_break_after = parse_page_break(&decl.value),
            "page-break-inside" => style.page_break_inside = parse_page_break_inside(&decl.value),
            "orphans" => style.orphans = parse_integer(&decl.value, 2) as u32,
            "widows" => style.widows = parse_integer(&decl.value, 2) as u32,

            _ => {} // Unknown property
        }
    }

    /// Resolve 'inherit' and 'initial' keywords
    fn resolve_keywords(&self, _style: &mut ComputedStyle, _parent: Option<&ComputedStyle>) {
        // TODO: Implement inherit/initial resolution
        // For now, inheritance is handled during initial computation
    }
}

impl Default for StyleResolver {
    fn default() -> Self {
        Self::new()
    }
}

// Property parsing functions

fn parse_display(value: &CssValue) -> Display {
    match value {
        CssValue::Ident(s) => match s.as_str() {
            "block" => Display::Block,
            "inline" => Display::Inline,
            "inline-block" => Display::InlineBlock,
            "flex" => Display::Flex,
            "inline-flex" => Display::InlineFlex,
            "grid" => Display::Grid,
            "inline-grid" => Display::InlineGrid,
            "none" => Display::None,
            "list-item" => Display::ListItem,
            "table" => Display::Table,
            _ => Display::Inline,
        },
        _ => Display::Inline,
    }
}

fn parse_position(value: &CssValue) -> Position {
    match value {
        CssValue::Ident(s) => match s.as_str() {
            "static" => Position::Static,
            "relative" => Position::Relative,
            "absolute" => Position::Absolute,
            "fixed" => Position::Fixed,
            "sticky" => Position::Sticky,
            _ => Position::Static,
        },
        _ => Position::Static,
    }
}

fn parse_float(value: &CssValue) -> Float {
    match value {
        CssValue::Ident(s) => match s.as_str() {
            "none" => Float::None,
            "left" => Float::Left,
            "right" => Float::Right,
            _ => Float::None,
        },
        _ => Float::None,
    }
}

fn parse_clear(value: &CssValue) -> Clear {
    match value {
        CssValue::Ident(s) => match s.as_str() {
            "none" => Clear::None,
            "left" => Clear::Left,
            "right" => Clear::Right,
            "both" => Clear::Both,
            _ => Clear::None,
        },
        _ => Clear::None,
    }
}

fn parse_length(value: &CssValue) -> Length {
    match value {
        CssValue::Length(n, unit) => Length::from_css_value(*n as f64, *unit),
        CssValue::Number(n) => Length::Px(*n as f32),
        CssValue::Ident(s) if s == "auto" => Length::Auto,
        CssValue::Percentage(p) => Length::Percent(*p as f32),
        _ => Length::Auto,
    }
}

fn parse_margin_or_padding(value: &CssValue) -> (Length, Length, Length, Length) {
    // Handle 1-4 value syntax
    match value {
        CssValue::List(values) => {
            let lengths: Vec<Length> = values.iter().map(parse_length).collect();
            match lengths.len() {
                1 => (lengths[0], lengths[0], lengths[0], lengths[0]),
                2 => (lengths[0], lengths[1], lengths[0], lengths[1]),
                3 => (lengths[0], lengths[1], lengths[2], lengths[1]),
                4 => (lengths[0], lengths[1], lengths[2], lengths[3]),
                _ => (Length::Px(0.0), Length::Px(0.0), Length::Px(0.0), Length::Px(0.0)),
            }
        }
        _ => {
            let len = parse_length(value);
            (len, len, len, len)
        }
    }
}

fn parse_border_widths(value: &CssValue) -> (Length, Length, Length, Length) {
    parse_margin_or_padding(value)
}

fn parse_border_style(value: &CssValue) -> BorderStyle {
    match value {
        CssValue::Ident(s) => match s.as_str() {
            "none" => BorderStyle::None,
            "hidden" => BorderStyle::Hidden,
            "solid" => BorderStyle::Solid,
            "dashed" => BorderStyle::Dashed,
            "dotted" => BorderStyle::Dotted,
            "double" => BorderStyle::Double,
            _ => BorderStyle::None,
        },
        _ => BorderStyle::None,
    }
}

fn parse_font_family(value: &CssValue) -> Vec<String> {
    match value {
        CssValue::List(values) => values.iter().map(|v| v.to_string()).collect(),
        CssValue::String(s) => vec![s.clone()],
        CssValue::Ident(s) => vec![s.clone()],
        _ => vec!["serif".to_string()],
    }
}

fn parse_font_weight(value: &CssValue) -> FontWeight {
    match value {
        CssValue::Ident(s) => match s.as_str() {
            "normal" => FontWeight::Normal,
            "bold" => FontWeight::Bold,
            "bolder" => FontWeight::Bolder,
            "lighter" => FontWeight::Lighter,
            _ => FontWeight::Normal,
        },
        CssValue::Number(n) => FontWeight::Number(*n as u16),
        _ => FontWeight::Normal,
    }
}

fn parse_font_style(value: &CssValue) -> FontStyle {
    match value {
        CssValue::Ident(s) => match s.as_str() {
            "normal" => FontStyle::Normal,
            "italic" => FontStyle::Italic,
            "oblique" => FontStyle::Oblique,
            _ => FontStyle::Normal,
        },
        _ => FontStyle::Normal,
    }
}

fn parse_line_height(value: &CssValue) -> LineHeight {
    match value {
        CssValue::Number(n) => LineHeight::Number(*n as f32),
        CssValue::Ident(s) if s == "normal" => LineHeight::Normal,
        _ => LineHeight::Length(parse_length(value)),
    }
}

fn parse_color(value: &CssValue) -> Color {
    match value {
        CssValue::HexColor(hex) => {
            Color::from_hex(&format!("#{}", hex)).unwrap_or(Color::BLACK)
        }
        CssValue::Ident(s) => match s.as_str() {
            "black" => Color::BLACK,
            "white" => Color::WHITE,
            "red" => Color::RED,
            "green" => Color::GREEN,
            "blue" => Color::BLUE,
            "transparent" => Color::TRANSPARENT,
            _ => Color::BLACK,
        },
        CssValue::Function(f) if f.name == "rgb" && f.args.len() >= 3 => {
            let r = parse_color_component(&f.args[0]);
            let g = parse_color_component(&f.args[1]);
            let b = parse_color_component(&f.args[2]);
            Color::new(r, g, b)
        }
        _ => Color::BLACK,
    }
}

fn parse_color_component(value: &CssValue) -> u8 {
    match value {
        CssValue::Number(n) => (*n as u8).clamp(0, 255),
        CssValue::Percentage(p) => ((*p * 2.55) as u8).clamp(0, 255),
        _ => 0,
    }
}

fn parse_text_align(value: &CssValue) -> TextAlign {
    match value {
        CssValue::Ident(s) => match s.as_str() {
            "left" => TextAlign::Left,
            "center" => TextAlign::Center,
            "right" => TextAlign::Right,
            "justify" => TextAlign::Justify,
            _ => TextAlign::Left,
        },
        _ => TextAlign::Left,
    }
}

fn parse_text_decoration(value: &CssValue) -> TextDecoration {
    match value {
        CssValue::Ident(s) => match s.as_str() {
            "none" => TextDecoration::None,
            "underline" => TextDecoration::Underline,
            "overline" => TextDecoration::Overline,
            "line-through" => TextDecoration::LineThrough,
            _ => TextDecoration::None,
        },
        _ => TextDecoration::None,
    }
}

fn parse_text_transform(value: &CssValue) -> TextTransform {
    match value {
        CssValue::Ident(s) => match s.as_str() {
            "none" => TextTransform::None,
            "capitalize" => TextTransform::Capitalize,
            "uppercase" => TextTransform::Uppercase,
            "lowercase" => TextTransform::Lowercase,
            _ => TextTransform::None,
        },
        _ => TextTransform::None,
    }
}

fn parse_white_space(value: &CssValue) -> WhiteSpace {
    match value {
        CssValue::Ident(s) => match s.as_str() {
            "normal" => WhiteSpace::Normal,
            "nowrap" => WhiteSpace::Nowrap,
            "pre" => WhiteSpace::Pre,
            "pre-wrap" => WhiteSpace::PreWrap,
            "pre-line" => WhiteSpace::PreLine,
            _ => WhiteSpace::Normal,
        },
        _ => WhiteSpace::Normal,
    }
}

fn parse_word_wrap(value: &CssValue) -> WordWrap {
    match value {
        CssValue::Ident(s) => match s.as_str() {
            "normal" => WordWrap::Normal,
            "break-word" => WordWrap::BreakWord,
            "anywhere" => WordWrap::Anywhere,
            _ => WordWrap::Normal,
        },
        _ => WordWrap::Normal,
    }
}

fn parse_visibility(value: &CssValue) -> Visibility {
    match value {
        CssValue::Ident(s) => match s.as_str() {
            "visible" => Visibility::Visible,
            "hidden" => Visibility::Hidden,
            "collapse" => Visibility::Collapse,
            _ => Visibility::Visible,
        },
        _ => Visibility::Visible,
    }
}

fn parse_overflow(value: &CssValue) -> Overflow {
    match value {
        CssValue::Ident(s) => match s.as_str() {
            "visible" => Overflow::Visible,
            "hidden" => Overflow::Hidden,
            "scroll" => Overflow::Scroll,
            "auto" => Overflow::Auto,
            _ => Overflow::Visible,
        },
        _ => Overflow::Visible,
    }
}

fn parse_z_index(value: &CssValue) -> ZIndex {
    match value {
        CssValue::Number(n) => ZIndex::Number(*n as i32),
        CssValue::Ident(s) if s == "auto" => ZIndex::Auto,
        _ => ZIndex::Auto,
    }
}

fn parse_page_break(value: &CssValue) -> PageBreak {
    match value {
        CssValue::Ident(s) => match s.as_str() {
            "auto" => PageBreak::Auto,
            "always" => PageBreak::Always,
            "avoid" => PageBreak::Avoid,
            "left" => PageBreak::Left,
            "right" => PageBreak::Right,
            _ => PageBreak::Auto,
        },
        _ => PageBreak::Auto,
    }
}

fn parse_page_break_inside(value: &CssValue) -> PageBreakInside {
    match value {
        CssValue::Ident(s) => match s.as_str() {
            "auto" => PageBreakInside::Auto,
            "avoid" => PageBreakInside::Avoid,
            _ => PageBreakInside::Auto,
        },
        _ => PageBreakInside::Auto,
    }
}

fn parse_number(value: &CssValue, default: f32) -> f32 {
    match value {
        CssValue::Number(n) => *n as f32,
        _ => default,
    }
}

fn parse_integer(value: &CssValue, default: i32) -> i32 {
    match value {
        CssValue::Number(n) => *n as i32,
        _ => default,
    }
}

impl Length {
    fn from_css_value(value: f64, unit: Unit) -> Self {
        match unit {
            Unit::Px => Length::Px(value as f32),
            Unit::Pt => Length::Pt(value as f32),
            Unit::Mm => Length::Mm(value as f32),
            Unit::Cm => Length::Cm(value as f32),
            Unit::In => Length::In(value as f32),
            Unit::Em => Length::Em(value as f32),
            Unit::Rem => Length::Rem(value as f32),
            Unit::Percent => Length::Percent(value as f32),
            _ => Length::Auto,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_specificity() {
        use crate::css::SelectorPart;

        let selector = Selector {
            parts: vec![
                SelectorPart::Type("div".to_string()),
                SelectorPart::Class("foo".to_string()),
                SelectorPart::Id("bar".to_string()),
            ],
        };

        let spec = Specificity::from_selector(&selector);
        assert_eq!(spec.0, 1); // 1 ID
        assert_eq!(spec.1, 1); // 1 class
        assert_eq!(spec.2, 1); // 1 type
    }

    #[test]
    fn test_parse_display() {
        assert_eq!(parse_display(&CssValue::Ident("block".to_string())), Display::Block);
        assert_eq!(parse_display(&CssValue::Ident("inline".to_string())), Display::Inline);
        assert_eq!(parse_display(&CssValue::Ident("none".to_string())), Display::None);
    }

    #[test]
    fn test_parse_length() {
        assert_eq!(parse_length(&CssValue::Length(100.0, Unit::Px)), Length::Px(100.0));
        assert_eq!(parse_length(&CssValue::Ident("auto".to_string())), Length::Auto);
    }

    #[test]
    fn test_computed_style_defaults() {
        let resolver = StyleResolver::new();
        let mut style = ComputedStyle::default();
        resolver.apply_initial_values(&mut style);

        assert_eq!(style.display, Display::Inline);
        assert_eq!(style.position, Position::Static);
        assert_eq!(style.font_size, Length::Px(16.0));
    }
}
