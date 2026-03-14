//! Style Computation
//!
//! Handles CSS cascade, specificity, inheritance, and computed values.
//! Implements style resolution from parsed CSS rules.

use crate::css::{
    Declaration, Rule, Selector, SelectorPart, Stylesheet,
    CssValue, Unit,
};
use crate::css::parser::StyleRule;
use crate::html::{Element};
use crate::layout::box_model::BoxType;
use crate::layout::grid::{GridLine, GridAutoFlow, parse_grid_auto_flow as parse_grid_auto_flow_fn};
use crate::types::{Color, Length};
use crate::pdf::print_css::{BreakType, BreakInside, StringSetValue, parse_break_value, parse_break_inside_value};

// Re-export StringSetValue for public API
pub use crate::pdf::print_css::StringSetValue;

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
    pub font_stretch: FontStretch,
    pub font_variant: FontVariant,
    pub font_variant_caps: FontVariantCaps,
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
    pub background_image: Option<String>,
    pub background_size: BackgroundSize,
    pub background_position: BackgroundPosition,
    pub background_repeat: BackgroundRepeat,
    pub opacity: f32,
    pub visibility: Visibility,
    pub overflow: Overflow,
    
    // Image properties
    pub object_fit: ObjectFit,
    pub object_position: ObjectPosition,
    
    // List properties
    pub list_style_type: ListStyleType,
    pub list_style_image: Option<String>,
    pub list_style_position: ListStylePosition,
    
    // Positioning offsets
    pub top: Length,
    pub right: Length,
    pub bottom: Length,
    pub left: Length,
    pub z_index: ZIndex,
    
    // PrintCSS - Legacy properties
    pub page_break_before: PageBreak,
    pub page_break_after: PageBreak,
    pub page_break_inside: PageBreakInside,
    // PrintCSS - Modern break properties (CSS Fragmentation Module Level 4)
    pub break_before: crate::pdf::print_css::BreakType,
    pub break_after: crate::pdf::print_css::BreakType,
    pub break_inside: crate::pdf::print_css::BreakInside,
    // PrintCSS - Widows and orphans control
    pub orphans: u32,
    pub widows: u32,
    // PrintCSS - Named page
    pub page: Option<String>,
    // PrintCSS - String set for running headers/footers
    pub string_set: Vec<(String, StringSetValue)>,
    
    // Grid container properties
    pub grid_template_columns: String,
    pub grid_template_rows: String,
    pub grid_template_areas: String,
    pub grid_auto_columns: String,
    pub grid_auto_rows: String,
    pub grid_auto_flow: GridAutoFlow,
    pub column_gap: Length,
    pub row_gap: Length,
    
    // Grid item properties
    pub grid_column_start: GridLine,
    pub grid_column_end: GridLine,
    pub grid_row_start: GridLine,
    pub grid_row_end: GridLine,
    
    // Table properties
    pub border_collapse: BorderCollapse,
    pub border_spacing_h: Length,
    pub border_spacing_v: Length,
    pub caption_side: CaptionSide,
    pub empty_cells: EmptyCells,
    pub table_layout: TableLayout,
}

/// Border collapse property for tables
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum BorderCollapse {
    #[default]
    Separate,
    Collapse,
}

/// Caption side property for tables
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum CaptionSide {
    #[default]
    Top,
    Bottom,
    Left,
    Right,
}

/// Empty cells property for tables
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum EmptyCells {
    #[default]
    Show,
    Hide,
}

/// Table layout property
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum TableLayout {
    #[default]
    Auto,
    Fixed
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
    InlineTable,
    TableRow,
    TableRowGroup,
    TableHeaderGroup,
    TableFooterGroup,
    TableColumn,
    TableColumnGroup,
    TableCell,
    TableCaption,
}

impl Display {
    pub fn to_box_type(&self) -> BoxType {
        match self {
            Display::Block | Display::Flex | Display::Grid | Display::Table | Display::InlineTable | Display::ListItem => BoxType::Block,
            Display::Inline | Display::InlineFlex | Display::InlineGrid => BoxType::Inline,
            Display::InlineBlock => BoxType::InlineBlock,
            Display::None => BoxType::Anonymous, // Will be filtered out
            Display::TableRow | Display::TableRowGroup | Display::TableHeaderGroup | Display::TableFooterGroup => BoxType::TableRow,
            Display::TableCell => BoxType::TableCell,
            Display::TableColumn | Display::TableColumnGroup => BoxType::Table,
            Display::TableCaption => BoxType::Block,
        }
    }

    pub fn is_none(&self) -> bool {
        matches!(self, Display::None)
    }
    
    /// Check if this is a table-related display type
    pub fn is_table_type(&self) -> bool {
        matches!(self, 
            Display::Table | Display::InlineTable | Display::TableRow | 
            Display::TableRowGroup | Display::TableHeaderGroup | Display::TableFooterGroup |
            Display::TableCell | Display::TableColumn | Display::TableColumnGroup | Display::TableCaption
        )
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
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum FontWeight {
    #[default]
    Normal,
    Bold,
    Bolder,
    Lighter,
    Number(u16),
}

/// Font style
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum FontStyle {
    #[default]
    Normal,
    Italic,
    Oblique,
}

/// Font variant
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum FontVariant {
    #[default]
    Normal,
    SmallCaps,
}

impl FontVariant {
    /// Parse from CSS value
    pub fn from_css(value: &CssValue) -> Self {
        match value {
            CssValue::Ident(s) => match s.as_str() {
                "small-caps" => FontVariant::SmallCaps,
                _ => FontVariant::Normal,
            },
            _ => FontVariant::Normal,
        }
    }
}

/// Font variant caps (more detailed control)
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum FontVariantCaps {
    #[default]
    Normal,
    SmallCaps,
    AllSmallCaps,
    PetiteCaps,
    AllPetiteCaps,
    Unicase,
    TitlingCaps,
}

impl FontVariantCaps {
    /// Parse from CSS value
    pub fn from_css(value: &CssValue) -> Self {
        match value {
            CssValue::Ident(s) => match s.as_str() {
                "small-caps" => FontVariantCaps::SmallCaps,
                "all-small-caps" => FontVariantCaps::AllSmallCaps,
                "petite-caps" => FontVariantCaps::PetiteCaps,
                "all-petite-caps" => FontVariantCaps::AllPetiteCaps,
                "unicase" => FontVariantCaps::Unicase,
                "titling-caps" => FontVariantCaps::TitlingCaps,
                _ => FontVariantCaps::Normal,
            },
            _ => FontVariantCaps::Normal,
        }
    }

    /// Check if this uses small caps
    pub fn is_small_caps(&self) -> bool {
        matches!(self, 
            FontVariantCaps::SmallCaps | 
            FontVariantCaps::AllSmallCaps |
            FontVariantCaps::PetiteCaps |
            FontVariantCaps::AllPetiteCaps
        )
    }
}

/// Font stretch (condensed/expanded)
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum FontStretch {
    #[default]
    Normal,
    UltraCondensed,
    ExtraCondensed,
    Condensed,
    SemiCondensed,
    SemiExpanded,
    Expanded,
    ExtraExpanded,
    UltraExpanded,
}

impl FontStretch {
    /// Parse from CSS value
    pub fn from_css(value: &CssValue) -> Self {
        match value {
            CssValue::Ident(s) => match s.as_str() {
                "ultra-condensed" => FontStretch::UltraCondensed,
                "extra-condensed" => FontStretch::ExtraCondensed,
                "condensed" => FontStretch::Condensed,
                "semi-condensed" => FontStretch::SemiCondensed,
                "semi-expanded" => FontStretch::SemiExpanded,
                "expanded" => FontStretch::Expanded,
                "extra-expanded" => FontStretch::ExtraExpanded,
                "ultra-expanded" => FontStretch::UltraExpanded,
                _ => FontStretch::Normal,
            },
            CssValue::Percentage(p) => Self::from_percentage(*p),
            _ => FontStretch::Normal,
        }
    }

    /// Convert from percentage
    pub fn from_percentage(p: f32) -> Self {
        match p as i32 {
            0..=49 => FontStretch::UltraCondensed,
            50..=62 => FontStretch::ExtraCondensed,
            63..=74 => FontStretch::Condensed,
            75..=87 => FontStretch::SemiCondensed,
            88..=112 => FontStretch::Normal,
            113..=124 => FontStretch::SemiExpanded,
            125..=149 => FontStretch::Expanded,
            150..=199 => FontStretch::ExtraExpanded,
            _ => FontStretch::UltraExpanded,
        }
    }

    /// Get percentage value
    pub fn to_percentage(&self) -> f32 {
        match self {
            FontStretch::UltraCondensed => 50.0,
            FontStretch::ExtraCondensed => 62.5,
            FontStretch::Condensed => 75.0,
            FontStretch::SemiCondensed => 87.5,
            FontStretch::Normal => 100.0,
            FontStretch::SemiExpanded => 112.5,
            FontStretch::Expanded => 125.0,
            FontStretch::ExtraExpanded => 150.0,
            FontStretch::UltraExpanded => 200.0,
        }
    }
}

/// Line height
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum LineHeight {
    #[default]
    Normal,
    Number(f32),
    Length(Length),
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

/// Object-fit property for images
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum ObjectFit {
    /// Fill the box, may distort aspect ratio
    Fill,
    /// Preserve aspect ratio, may be cropped
    #[default]
    Cover,
    /// Preserve aspect ratio, may have empty space
    Contain,
    /// No resizing
    None,
    /// Like 'contain' but if smaller than box, not scaled up
    ScaleDown,
}

/// Object-position property
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct ObjectPosition {
    pub horizontal: f32, // 0.0 = left, 1.0 = right
    pub vertical: f32,   // 0.0 = top, 1.0 = bottom
}

impl ObjectPosition {
    pub const CENTER: Self = Self { horizontal: 0.5, vertical: 0.5 };
    pub const TOP_LEFT: Self = Self { horizontal: 0.0, vertical: 0.0 };
    pub const TOP_RIGHT: Self = Self { horizontal: 1.0, vertical: 0.0 };
    pub const BOTTOM_LEFT: Self = Self { horizontal: 0.0, vertical: 1.0 };
    pub const BOTTOM_RIGHT: Self = Self { horizontal: 1.0, vertical: 1.0 };
}

/// Background size property
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum BackgroundSize {
    #[default]
    Auto,
    Cover,
    Contain,
    Dimensions(Length, Length),
}

/// Background position property
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct BackgroundPosition {
    pub horizontal: PositionValue,
    pub vertical: PositionValue,
}

impl BackgroundPosition {
    pub const CENTER: Self = Self {
        horizontal: PositionValue::Center,
        vertical: PositionValue::Center,
    };
}

/// Position value for background-position
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PositionValue {
    Left,
    Center,
    Right,
    Top,
    Bottom,
    Percentage(f32),
    Length(Length),
}

impl Default for PositionValue {
    fn default() -> Self {
        PositionValue::Percentage(0.0)
    }
}

/// Background repeat property
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum BackgroundRepeat {
    #[default]
    Repeat,
    RepeatX,
    RepeatY,
    NoRepeat,
    Space,
    Round,
}

/// List style type property
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum ListStyleType {
    #[default]
    Disc,
    Circle,
    Square,
    Decimal,
    DecimalLeadingZero,
    LowerRoman,
    UpperRoman,
    LowerAlpha,
    UpperAlpha,
    None,
}

/// List style position property
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum ListStylePosition {
    #[default]
    Outside,
    Inside,
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
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum ZIndex {
    #[default]
    Auto,
    Number(i32),
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
        style.font_stretch = FontStretch::Normal;
        style.font_variant = FontVariant::Normal;
        style.font_variant_caps = FontVariantCaps::Normal;
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
        style.background_image = None;
        style.background_size = BackgroundSize::Auto;
        style.background_position = BackgroundPosition::CENTER;
        style.background_repeat = BackgroundRepeat::Repeat;
        style.opacity = 1.0;
        style.visibility = Visibility::Visible;
        style.overflow = Overflow::Visible;

        // Image properties
        style.object_fit = ObjectFit::Cover;
        style.object_position = ObjectPosition::CENTER;

        // List properties
        style.list_style_type = ListStyleType::Disc;
        style.list_style_image = None;
        style.list_style_position = ListStylePosition::Outside;

        // Positioning
        style.top = Length::Auto;
        style.right = Length::Auto;
        style.bottom = Length::Auto;
        style.left = Length::Auto;
        style.z_index = ZIndex::Auto;

        // PrintCSS - Legacy
        style.page_break_before = PageBreak::Auto;
        style.page_break_after = PageBreak::Auto;
        style.page_break_inside = PageBreakInside::Auto;
        // PrintCSS - Modern break properties
        style.break_before = BreakType::Auto;
        style.break_after = BreakType::Auto;
        style.break_inside = BreakInside::Auto;
        // PrintCSS - Widows and orphans
        style.orphans = 2;
        style.widows = 2;
        // PrintCSS - Named page and string-set
        style.page = None;
        style.string_set = Vec::new();
        
        // Grid defaults
        style.grid_template_columns = String::new();
        style.grid_template_rows = String::new();
        style.grid_template_areas = String::new();
        style.grid_auto_columns = String::new();
        style.grid_auto_rows = String::new();
        style.grid_auto_flow = GridAutoFlow::Row;
        style.column_gap = Length::Px(0.0);
        style.row_gap = Length::Px(0.0);
        style.grid_column_start = GridLine::auto();
        style.grid_column_end = GridLine::auto();
        style.grid_row_start = GridLine::auto();
        style.grid_row_end = GridLine::auto();
        
        // Table defaults
        style.border_collapse = BorderCollapse::Separate;
        style.border_spacing_h = Length::Px(2.0);
        style.border_spacing_v = Length::Px(2.0);
        style.caption_side = CaptionSide::Top;
        style.empty_cells = EmptyCells::Show;
        style.table_layout = TableLayout::Auto;
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
        style.list_style_type = parent.list_style_type;
        style.list_style_image = parent.list_style_image.clone();
        style.list_style_position = parent.list_style_position;
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

        // Img (inline by default)
        let img_selector = Selector {
            parts: vec![SelectorPart::Type("img".to_string())],
        };
        self.default_styles.push((img_selector, vec![
            Declaration::new("display", CssValue::Ident("inline".to_string())),
        ]));

        // Lists
        let ul_selector = Selector {
            parts: vec![SelectorPart::Type("ul".to_string())],
        };
        self.default_styles.push((ul_selector, vec![
            Declaration::new("display", CssValue::Ident("block".to_string())),
            Declaration::new("list-style-type", CssValue::Ident("disc".to_string())),
            Declaration::new("padding-left", CssValue::Length(40.0, Unit::Px)),
        ]));

        let ol_selector = Selector {
            parts: vec![SelectorPart::Type("ol".to_string())],
        };
        self.default_styles.push((ol_selector, vec![
            Declaration::new("display", CssValue::Ident("block".to_string())),
            Declaration::new("list-style-type", CssValue::Ident("decimal".to_string())),
            Declaration::new("padding-left", CssValue::Length(40.0, Unit::Px)),
        ]));

        let li_selector = Selector {
            parts: vec![SelectorPart::Type("li".to_string())],
        };
        self.default_styles.push((li_selector, vec![
            Declaration::new("display", CssValue::Ident("list-item".to_string())),
        ]));

        // Form elements
        // Input elements (general)
        let input_selector = Selector {
            parts: vec![SelectorPart::Type("input".to_string())],
        };
        self.default_styles.push((input_selector, vec![
            Declaration::new("display", CssValue::Ident("inline-block".to_string())),
            Declaration::new("border-top-width", CssValue::Length(1.0, Unit::Px)),
            Declaration::new("border-right-width", CssValue::Length(1.0, Unit::Px)),
            Declaration::new("border-bottom-width", CssValue::Length(1.0, Unit::Px)),
            Declaration::new("border-left-width", CssValue::Length(1.0, Unit::Px)),
            Declaration::new("border-top-style", CssValue::Ident("solid".to_string())),
            Declaration::new("border-right-style", CssValue::Ident("solid".to_string())),
            Declaration::new("border-bottom-style", CssValue::Ident("solid".to_string())),
            Declaration::new("border-left-style", CssValue::Ident("solid".to_string())),
            Declaration::new("border-top-color", CssValue::HexColor(0x999999)),
            Declaration::new("border-right-color", CssValue::HexColor(0x999999)),
            Declaration::new("border-bottom-color", CssValue::HexColor(0x999999)),
            Declaration::new("border-left-color", CssValue::HexColor(0x999999)),
            Declaration::new("background-color", CssValue::HexColor(0xFFFFFF)),
            Declaration::new("padding-top", CssValue::Length(4.0, Unit::Px)),
            Declaration::new("padding-right", CssValue::Length(8.0, Unit::Px)),
            Declaration::new("padding-bottom", CssValue::Length(4.0, Unit::Px)),
            Declaration::new("padding-left", CssValue::Length(8.0, Unit::Px)),
            Declaration::new("margin-top", CssValue::Length(2.0, Unit::Px)),
            Declaration::new("margin-bottom", CssValue::Length(2.0, Unit::Px)),
        ]));

        // Textarea
        let textarea_selector = Selector {
            parts: vec![SelectorPart::Type("textarea".to_string())],
        };
        self.default_styles.push((textarea_selector, vec![
            Declaration::new("display", CssValue::Ident("inline-block".to_string())),
            Declaration::new("border-top-width", CssValue::Length(1.0, Unit::Px)),
            Declaration::new("border-right-width", CssValue::Length(1.0, Unit::Px)),
            Declaration::new("border-bottom-width", CssValue::Length(1.0, Unit::Px)),
            Declaration::new("border-left-width", CssValue::Length(1.0, Unit::Px)),
            Declaration::new("border-top-style", CssValue::Ident("solid".to_string())),
            Declaration::new("border-right-style", CssValue::Ident("solid".to_string())),
            Declaration::new("border-bottom-style", CssValue::Ident("solid".to_string())),
            Declaration::new("border-left-style", CssValue::Ident("solid".to_string())),
            Declaration::new("border-top-color", CssValue::HexColor(0x999999)),
            Declaration::new("border-right-color", CssValue::HexColor(0x999999)),
            Declaration::new("border-bottom-color", CssValue::HexColor(0x999999)),
            Declaration::new("border-left-color", CssValue::HexColor(0x999999)),
            Declaration::new("background-color", CssValue::HexColor(0xFFFFFF)),
            Declaration::new("padding-top", CssValue::Length(6.0, Unit::Px)),
            Declaration::new("padding-right", CssValue::Length(8.0, Unit::Px)),
            Declaration::new("padding-bottom", CssValue::Length(6.0, Unit::Px)),
            Declaration::new("padding-left", CssValue::Length(8.0, Unit::Px)),
            Declaration::new("margin-top", CssValue::Length(2.0, Unit::Px)),
            Declaration::new("margin-bottom", CssValue::Length(2.0, Unit::Px)),
        ]));

        // Select
        let select_selector = Selector {
            parts: vec![SelectorPart::Type("select".to_string())],
        };
        self.default_styles.push((select_selector, vec![
            Declaration::new("display", CssValue::Ident("inline-block".to_string())),
            Declaration::new("border-top-width", CssValue::Length(1.0, Unit::Px)),
            Declaration::new("border-right-width", CssValue::Length(1.0, Unit::Px)),
            Declaration::new("border-bottom-width", CssValue::Length(1.0, Unit::Px)),
            Declaration::new("border-left-width", CssValue::Length(1.0, Unit::Px)),
            Declaration::new("border-top-style", CssValue::Ident("solid".to_string())),
            Declaration::new("border-right-style", CssValue::Ident("solid".to_string())),
            Declaration::new("border-bottom-style", CssValue::Ident("solid".to_string())),
            Declaration::new("border-left-style", CssValue::Ident("solid".to_string())),
            Declaration::new("border-top-color", CssValue::HexColor(0x999999)),
            Declaration::new("border-right-color", CssValue::HexColor(0x999999)),
            Declaration::new("border-bottom-color", CssValue::HexColor(0x999999)),
            Declaration::new("border-left-color", CssValue::HexColor(0x999999)),
            Declaration::new("background-color", CssValue::HexColor(0xFFFFFF)),
            Declaration::new("padding-top", CssValue::Length(4.0, Unit::Px)),
            Declaration::new("padding-right", CssValue::Length(8.0, Unit::Px)),
            Declaration::new("padding-bottom", CssValue::Length(4.0, Unit::Px)),
            Declaration::new("padding-left", CssValue::Length(8.0, Unit::Px)),
            Declaration::new("margin-top", CssValue::Length(2.0, Unit::Px)),
            Declaration::new("margin-bottom", CssValue::Length(2.0, Unit::Px)),
        ]));

        // Button
        let button_selector = Selector {
            parts: vec![SelectorPart::Type("button".to_string())],
        };
        self.default_styles.push((button_selector, vec![
            Declaration::new("display", CssValue::Ident("inline-block".to_string())),
            Declaration::new("border-top-width", CssValue::Length(1.0, Unit::Px)),
            Declaration::new("border-right-width", CssValue::Length(1.0, Unit::Px)),
            Declaration::new("border-bottom-width", CssValue::Length(1.0, Unit::Px)),
            Declaration::new("border-left-width", CssValue::Length(1.0, Unit::Px)),
            Declaration::new("border-top-style", CssValue::Ident("solid".to_string())),
            Declaration::new("border-right-style", CssValue::Ident("solid".to_string())),
            Declaration::new("border-bottom-style", CssValue::Ident("solid".to_string())),
            Declaration::new("border-left-style", CssValue::Ident("solid".to_string())),
            Declaration::new("border-top-color", CssValue::HexColor(0x999999)),
            Declaration::new("border-right-color", CssValue::HexColor(0x999999)),
            Declaration::new("border-bottom-color", CssValue::HexColor(0x999999)),
            Declaration::new("border-left-color", CssValue::HexColor(0x999999)),
            Declaration::new("background-color", CssValue::HexColor(0xF0F0F0)),
            Declaration::new("padding-top", CssValue::Length(6.0, Unit::Px)),
            Declaration::new("padding-right", CssValue::Length(16.0, Unit::Px)),
            Declaration::new("padding-bottom", CssValue::Length(6.0, Unit::Px)),
            Declaration::new("padding-left", CssValue::Length(16.0, Unit::Px)),
            Declaration::new("margin-top", CssValue::Length(2.0, Unit::Px)),
            Declaration::new("margin-bottom", CssValue::Length(2.0, Unit::Px)),
        ]));

        // Label
        let label_selector = Selector {
            parts: vec![SelectorPart::Type("label".to_string())],
        };
        self.default_styles.push((label_selector, vec![
            Declaration::new("display", CssValue::Ident("inline".to_string())),
            Declaration::new("margin-right", CssValue::Length(4.0, Unit::Px)),
        ]));

        // ===== Table Elements =====
        
        // Table
        let table_selector = Selector {
            parts: vec![SelectorPart::Type("table".to_string())],
        };
        self.default_styles.push((table_selector, vec![
            Declaration::new("display", CssValue::Ident("table".to_string())),
            Declaration::new("border-spacing", CssValue::Length(2.0, Unit::Px)),
            Declaration::new("border-collapse", CssValue::Ident("separate".to_string())),
        ]));
        
        // Table caption
        let caption_selector = Selector {
            parts: vec![SelectorPart::Type("caption".to_string())],
        };
        self.default_styles.push((caption_selector, vec![
            Declaration::new("display", CssValue::Ident("table-caption".to_string())),
            Declaration::new("text-align", CssValue::Ident("center".to_string())),
        ]));
        
        // Table column group
        let colgroup_selector = Selector {
            parts: vec![SelectorPart::Type("colgroup".to_string())],
        };
        self.default_styles.push((colgroup_selector, vec![
            Declaration::new("display", CssValue::Ident("table-column-group".to_string())),
        ]));
        
        // Table column
        let col_selector = Selector {
            parts: vec![SelectorPart::Type("col".to_string())],
        };
        self.default_styles.push((col_selector, vec![
            Declaration::new("display", CssValue::Ident("table-column".to_string())),
        ]));
        
        // Table head
        let thead_selector = Selector {
            parts: vec![SelectorPart::Type("thead".to_string())],
        };
        self.default_styles.push((thead_selector, vec![
            Declaration::new("display", CssValue::Ident("table-header-group".to_string())),
        ]));
        
        // Table body
        let tbody_selector = Selector {
            parts: vec![SelectorPart::Type("tbody".to_string())],
        };
        self.default_styles.push((tbody_selector, vec![
            Declaration::new("display", CssValue::Ident("table-row-group".to_string())),
        ]));
        
        // Table foot
        let tfoot_selector = Selector {
            parts: vec![SelectorPart::Type("tfoot".to_string())],
        };
        self.default_styles.push((tfoot_selector, vec![
            Declaration::new("display", CssValue::Ident("table-footer-group".to_string())),
        ]));
        
        // Table row
        let tr_selector = Selector {
            parts: vec![SelectorPart::Type("tr".to_string())],
        };
        self.default_styles.push((tr_selector, vec![
            Declaration::new("display", CssValue::Ident("table-row".to_string())),
        ]));
        
        // Table cell (td)
        let td_selector = Selector {
            parts: vec![SelectorPart::Type("td".to_string())],
        };
        self.default_styles.push((td_selector, vec![
            Declaration::new("display", CssValue::Ident("table-cell".to_string())),
            Declaration::new("padding", CssValue::Length(1.0, Unit::Px)),
        ]));
        
        // Table header cell (th)
        let th_selector = Selector {
            parts: vec![SelectorPart::Type("th".to_string())],
        };
        self.default_styles.push((th_selector, vec![
            Declaration::new("display", CssValue::Ident("table-cell".to_string())),
            Declaration::new("font-weight", CssValue::Ident("bold".to_string())),
            Declaration::new("text-align", CssValue::Ident("center".to_string())),
            Declaration::new("padding", CssValue::Length(1.0, Unit::Px)),
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
        let mut matching_rules: Vec<MatchingRule<'_>> = Vec::new();

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
            "font-stretch" => style.font_stretch = FontStretch::from_css(&decl.value),
            "font-variant" => style.font_variant = FontVariant::from_css(&decl.value),
            "font-variant-caps" => style.font_variant_caps = FontVariantCaps::from_css(&decl.value),
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
            "background-image" => style.background_image = parse_background_image(&decl.value),
            "background-size" => style.background_size = parse_background_size(&decl.value),
            "background-position" => style.background_position = parse_background_position(&decl.value),
            "background-repeat" => style.background_repeat = parse_background_repeat(&decl.value),
            
            "opacity" => style.opacity = parse_number(&decl.value, 1.0),
            "visibility" => style.visibility = parse_visibility(&decl.value),
            "overflow" => style.overflow = parse_overflow(&decl.value),

            // Image properties
            "object-fit" => style.object_fit = parse_object_fit(&decl.value),
            "object-position" => style.object_position = parse_object_position(&decl.value),

            // List properties
            "list-style-type" => style.list_style_type = parse_list_style_type(&decl.value),
            "list-style-image" => style.list_style_image = parse_list_style_image(&decl.value),
            "list-style-position" => style.list_style_position = parse_list_style_position(&decl.value),

            "top" => style.top = parse_length(&decl.value),
            "right" => style.right = parse_length(&decl.value),
            "bottom" => style.bottom = parse_length(&decl.value),
            "left" => style.left = parse_length(&decl.value),
            "z-index" => style.z_index = parse_z_index(&decl.value),

            // Legacy page break properties
            "page-break-before" => style.page_break_before = parse_page_break(&decl.value),
            "page-break-after" => style.page_break_after = parse_page_break(&decl.value),
            "page-break-inside" => style.page_break_inside = parse_page_break_inside(&decl.value),
            // Modern break properties (CSS Fragmentation Module Level 4)
            "break-before" => style.break_before = parse_break_value(&decl.value),
            "break-after" => style.break_after = parse_break_value(&decl.value),
            "break-inside" => style.break_inside = parse_break_inside_value(&decl.value),
            // Widows and orphans control
            "orphans" => style.orphans = parse_integer(&decl.value, 2) as u32,
            "widows" => style.widows = parse_integer(&decl.value, 2) as u32,
            // Named page
            "page" => style.page = parse_page_name(&decl.value),
            // String set for running headers/footers
            "string-set" => style.string_set = parse_string_set(&decl.value),

            // Grid container properties
            "grid-template-columns" => style.grid_template_columns = parse_grid_template(&decl.value),
            "grid-template-rows" => style.grid_template_rows = parse_grid_template(&decl.value),
            "grid-template-areas" => style.grid_template_areas = parse_grid_template_areas(&decl.value),
            "grid-auto-columns" => style.grid_auto_columns = parse_grid_auto_size(&decl.value),
            "grid-auto-rows" => style.grid_auto_rows = parse_grid_auto_size(&decl.value),
            "grid-auto-flow" => style.grid_auto_flow = parse_grid_auto_flow_value(&decl.value),
            "column-gap" => style.column_gap = parse_length(&decl.value),
            "row-gap" => style.row_gap = parse_length(&decl.value),
            "gap" => {
                let (row, col) = parse_gap(&decl.value);
                style.row_gap = row;
                style.column_gap = col;
            }

            // Grid item properties
            "grid-column-start" => style.grid_column_start = parse_grid_line(&decl.value),
            "grid-column-end" => style.grid_column_end = parse_grid_line(&decl.value),
            "grid-row-start" => style.grid_row_start = parse_grid_line(&decl.value),
            "grid-row-end" => style.grid_row_end = parse_grid_line(&decl.value),
            "grid-column" => {
                let (start, end) = parse_grid_line_shorthand(&decl.value);
                style.grid_column_start = start;
                style.grid_column_end = end;
            }
            "grid-row" => {
                let (start, end) = parse_grid_line_shorthand(&decl.value);
                style.grid_row_start = start;
                style.grid_row_end = end;
            }
            "grid-area" => {
                let (row_start, col_start, row_end, col_end) = parse_grid_area(&decl.value);
                style.grid_row_start = row_start;
                style.grid_column_start = col_start;
                style.grid_row_end = row_end;
                style.grid_column_end = col_end;
            }

            // Table properties
            "border-collapse" => style.border_collapse = parse_border_collapse(&decl.value),
            "border-spacing" => {
                let (h, v) = parse_border_spacing(&decl.value);
                style.border_spacing_h = Length::Px(h);
                style.border_spacing_v = Length::Px(v);
            }
            "caption-side" => style.caption_side = parse_caption_side(&decl.value),
            "empty-cells" => style.empty_cells = parse_empty_cells(&decl.value),
            "table-layout" => style.table_layout = parse_table_layout(&decl.value),

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
        CssValue::Number(n) => Length::Px(*n),
        CssValue::Ident(s) if s == "auto" => Length::Auto,
        CssValue::Percentage(p) => Length::Percent(*p),
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

/// Parse font-family with proper generic family handling
fn parse_font_family(value: &CssValue) -> Vec<String> {
    let families = match value {
        CssValue::List(values) => values.iter().filter_map(|v| {
            // Clean up family names (remove quotes if present)
            let name = match v {
                CssValue::String(s) => s.clone(),
                CssValue::Ident(s) => s.clone(),
                _ => return None,
            };
            Some(name)
        }).collect(),
        CssValue::String(s) => vec![s.clone()],
        CssValue::Ident(s) => vec![s.clone()],
        _ => vec!["serif".to_string()],
    };
    
    // Ensure there's a fallback
    if families.is_empty() {
        vec!["serif".to_string()]
    } else {
        families
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
        CssValue::Number(n) => LineHeight::Number(*n),
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
        CssValue::Function(f) if f.name == "rgba" && f.args.len() >= 4 => {
            let r = parse_color_component(&f.args[0]);
            let g = parse_color_component(&f.args[1]);
            let b = parse_color_component(&f.args[2]);
            let a = parse_color_component(&f.args[3]);
            Color::new_rgba(r, g, b, a)
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

fn parse_object_fit(value: &CssValue) -> ObjectFit {
    match value {
        CssValue::Ident(s) => match s.as_str() {
            "fill" => ObjectFit::Fill,
            "cover" => ObjectFit::Cover,
            "contain" => ObjectFit::Contain,
            "none" => ObjectFit::None,
            "scale-down" => ObjectFit::ScaleDown,
            _ => ObjectFit::Cover,
        },
        _ => ObjectFit::Cover,
    }
}

fn parse_object_position(value: &CssValue) -> ObjectPosition {
    match value {
        CssValue::Ident(s) => match s.as_str() {
            "center" => ObjectPosition::CENTER,
            "top" => ObjectPosition::TOP_LEFT, // vertical only
            "bottom" => ObjectPosition::BOTTOM_LEFT, // vertical only
            "left" => ObjectPosition::TOP_LEFT, // horizontal only
            "right" => ObjectPosition::TOP_RIGHT, // horizontal only
            _ => ObjectPosition::CENTER,
        },
        CssValue::List(values) if values.len() >= 2 => {
            let h = parse_position_value(&values[0]);
            let v = parse_position_value(&values[1]);
            ObjectPosition { horizontal: h, vertical: v }
        }
        _ => ObjectPosition::CENTER,
    }
}

fn parse_position_value(value: &CssValue) -> f32 {
    match value {
        CssValue::Ident(s) => match s.as_str() {
            "left" | "top" => 0.0,
            "center" => 0.5,
            "right" | "bottom" => 1.0,
            _ => 0.5,
        },
        CssValue::Percentage(p) => *p / 100.0,
        _ => 0.5,
    }
}

fn parse_background_image(value: &CssValue) -> Option<String> {
    match value {
        CssValue::Function(f) if f.name == "url" && !f.args.is_empty() => {
            // Extract URL from url("...") or url('...')
            let url = f.args[0].to_string();
            let url = url.trim_matches('"').trim_matches('\'').to_string();
            Some(url)
        }
        CssValue::Ident(s) if s == "none" => None,
        _ => None,
    }
}

fn parse_background_size(value: &CssValue) -> BackgroundSize {
    match value {
        CssValue::Ident(s) => match s.as_str() {
            "cover" => BackgroundSize::Cover,
            "contain" => BackgroundSize::Contain,
            "auto" => BackgroundSize::Auto,
            _ => BackgroundSize::Auto,
        },
        CssValue::List(values) if values.len() == 2 => {
            let w = parse_length(&values[0]);
            let h = parse_length(&values[1]);
            BackgroundSize::Dimensions(w, h)
        }
        _ => BackgroundSize::Auto,
    }
}

fn parse_background_position(value: &CssValue) -> BackgroundPosition {
    // Simplified - only handles single keywords
    match value {
        CssValue::Ident(s) => match s.as_str() {
            "center" => BackgroundPosition::CENTER,
            "top" => BackgroundPosition {
                horizontal: PositionValue::Center,
                vertical: PositionValue::Top,
            },
            "bottom" => BackgroundPosition {
                horizontal: PositionValue::Center,
                vertical: PositionValue::Bottom,
            },
            "left" => BackgroundPosition {
                horizontal: PositionValue::Left,
                vertical: PositionValue::Center,
            },
            "right" => BackgroundPosition {
                horizontal: PositionValue::Right,
                vertical: PositionValue::Center,
            },
            _ => BackgroundPosition::CENTER,
        },
        _ => BackgroundPosition::CENTER,
    }
}

fn parse_background_repeat(value: &CssValue) -> BackgroundRepeat {
    match value {
        CssValue::Ident(s) => match s.as_str() {
            "repeat" => BackgroundRepeat::Repeat,
            "repeat-x" => BackgroundRepeat::RepeatX,
            "repeat-y" => BackgroundRepeat::RepeatY,
            "no-repeat" => BackgroundRepeat::NoRepeat,
            "space" => BackgroundRepeat::Space,
            "round" => BackgroundRepeat::Round,
            _ => BackgroundRepeat::Repeat,
        },
        _ => BackgroundRepeat::Repeat,
    }
}

fn parse_list_style_type(value: &CssValue) -> ListStyleType {
    match value {
        CssValue::Ident(s) => match s.as_str() {
            "disc" => ListStyleType::Disc,
            "circle" => ListStyleType::Circle,
            "square" => ListStyleType::Square,
            "decimal" => ListStyleType::Decimal,
            "decimal-leading-zero" => ListStyleType::DecimalLeadingZero,
            "lower-roman" => ListStyleType::LowerRoman,
            "upper-roman" => ListStyleType::UpperRoman,
            "lower-alpha" => ListStyleType::LowerAlpha,
            "upper-alpha" => ListStyleType::UpperAlpha,
            "none" => ListStyleType::None,
            _ => ListStyleType::Disc,
        },
        _ => ListStyleType::Disc,
    }
}

fn parse_list_style_image(value: &CssValue) -> Option<String> {
    match value {
        CssValue::Function(f) if f.name == "url" && !f.args.is_empty() => {
            let url = f.args[0].to_string();
            let url = url.trim_matches('"').trim_matches('\'').to_string();
            Some(url)
        }
        CssValue::Ident(s) if s == "none" => None,
        _ => None,
    }
}

fn parse_list_style_position(value: &CssValue) -> ListStylePosition {
    match value {
        CssValue::Ident(s) => match s.as_str() {
            "inside" => ListStylePosition::Inside,
            "outside" => ListStylePosition::Outside,
            _ => ListStylePosition::Outside,
        },
        _ => ListStylePosition::Outside,
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

/// Parse named page value
fn parse_page_name(value: &CssValue) -> Option<String> {
    match value {
        CssValue::Ident(name) if name != "auto" => Some(name.clone()),
        CssValue::String(name) => Some(name.clone()),
        _ => None,
    }
}

/// Parse string-set declaration
/// Format: string-set: header "Chapter 1", footer "Page " counter(page);
fn parse_string_set(value: &CssValue) -> Vec<(String, StringSetValue)> {
    let mut result = Vec::new();
    
    let values = match value {
        CssValue::List(list) => list.clone(),
        _ => vec![value.clone()],
    };
    
    // Parse in pairs: name value
    let mut i = 0;
    while i < values.len() {
        if let CssValue::Ident(name) = &values[i] {
            i += 1;
            if i < values.len() {
                let set_value = parse_string_set_value(&values[i]);
                result.push((name.clone(), set_value));
                i += 1;
            }
        } else {
            i += 1;
        }
    }
    
    result
}

/// Parse individual string-set value
fn parse_string_set_value(value: &CssValue) -> StringSetValue {
    match value {
        CssValue::String(s) => StringSetValue::Text(s.clone()),
        CssValue::Ident(s) if s == "none" => StringSetValue::Text(String::new()),
        CssValue::Function(f) => {
            match f.name.as_str() {
                "attr" => {
                    if let Some(CssValue::Ident(attr_name)) = f.arguments.first() {
                        StringSetValue::Attr(attr_name.clone())
                    } else {
                        StringSetValue::Text(String::new())
                    }
                }
                "counter" => {
                    // Simplified - just use the counter name as text
                    if let Some(CssValue::Ident(counter_name)) = f.arguments.first() {
                        StringSetValue::Text(format!("[{}]", counter_name))
                    } else {
                        StringSetValue::Text(String::new())
                    }
                }
                _ => StringSetValue::Text(String::new()),
            }
        }
        _ => StringSetValue::Text(String::new()),
    }
}

fn parse_number(value: &CssValue, default: f32) -> f32 {
    match value {
        CssValue::Number(n) => *n,
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

// Grid property parsing functions

fn parse_grid_template(value: &CssValue) -> String {
    match value {
        CssValue::Ident(s) if s == "none" => String::new(),
        CssValue::List(_) | CssValue::Function(_) => {
            // Return the string representation for later parsing
            value.to_string()
        }
        CssValue::String(s) => s.clone(),
        _ => String::new(),
    }
}

fn parse_grid_template_areas(value: &CssValue) -> String {
    match value {
        CssValue::Ident(s) if s == "none" => String::new(),
        CssValue::String(s) => s.clone(),
        _ => value.to_string(),
    }
}

fn parse_grid_auto_size(value: &CssValue) -> String {
    value.to_string()
}

fn parse_grid_auto_flow_value(value: &CssValue) -> GridAutoFlow {
    match value {
        CssValue::Ident(s) => parse_grid_auto_flow(s),
        _ => GridAutoFlow::Row,
    }
}

fn parse_gap(value: &CssValue) -> (Length, Length) {
    match value {
        CssValue::List(values) => {
            let lengths: Vec<Length> = values.iter().map(parse_length).collect();
            match lengths.len() {
                1 => (lengths[0], lengths[0]),
                2 => (lengths[0], lengths[1]),
                _ => (Length::Px(0.0), Length::Px(0.0)),
            }
        }
        _ => {
            let len = parse_length(value);
            (len, len)
        }
    }
}

fn parse_grid_line(value: &CssValue) -> GridLine {
    match value {
        CssValue::Ident(s) => {
            if s == "auto" {
                GridLine::auto()
            } else if s.starts_with("span ") {
                if let Ok(n) = s[5..].trim().parse::<i32>() {
                    GridLine::span(n)
                } else {
                    GridLine::span(1)
                }
            } else {
                // Named area or line name
                GridLine {
                    line_number: 0,
                    area_name: Some(s.clone()),
                    span: None,
                }
            }
        }
        CssValue::Number(n) => {
            if *n > 0.0 {
                GridLine::numbered(*n as i32)
            } else {
                GridLine::auto()
            }
        }
        CssValue::Integer(n) => {
            if *n > 0 {
                GridLine::numbered(*n)
            } else {
                GridLine::auto()
            }
        }
        _ => GridLine::auto(),
    }
}

fn parse_grid_line_shorthand(value: &CssValue) -> (GridLine, GridLine) {
    let s = match value {
        CssValue::Ident(s) => s.clone(),
        CssValue::String(s) => s.clone(),
        CssValue::List(values) => {
            // Handle "start / end" syntax
            if values.len() == 2 {
                return (parse_grid_line(&values[0]), parse_grid_line(&values[1]));
            }
            return (parse_grid_line(value), GridLine::auto());
        }
        _ => return (parse_grid_line(value), GridLine::auto()),
    };

    // Parse "start / end" syntax from string
    if s.contains('/') {
        let parts: Vec<&str> = s.split('/').map(|p| p.trim()).collect();
        if parts.len() == 2 {
            let start = if let Ok(n) = parts[0].parse::<i32>() {
                GridLine::numbered(n)
            } else if parts[0].starts_with("span ") {
                if let Ok(n) = parts[0][5..].trim().parse::<i32>() {
                    GridLine::span(n)
                } else {
                    GridLine::span(1)
                }
            } else if parts[0] == "auto" {
                GridLine::auto()
            } else {
                GridLine {
                    line_number: 0,
                    area_name: Some(parts[0].to_string()),
                    span: None,
                }
            };

            let end = if let Ok(n) = parts[1].parse::<i32>() {
                GridLine::numbered(n)
            } else if parts[1].starts_with("span ") {
                if let Ok(n) = parts[1][5..].trim().parse::<i32>() {
                    GridLine::span(n)
                } else {
                    GridLine::span(1)
                }
            } else if parts[1] == "auto" {
                GridLine::auto()
            } else {
                GridLine {
                    line_number: 0,
                    area_name: Some(parts[1].to_string()),
                    span: None,
                }
            };

            return (start, end);
        }
    }

    // Single value
    (parse_grid_line(value), GridLine::auto())
}

fn parse_grid_area(value: &CssValue) -> (GridLine, GridLine, GridLine, GridLine) {
    let s = match value {
        CssValue::Ident(s) => s.clone(),
        CssValue::String(s) => s.clone(),
        CssValue::List(values) => {
            // Handle list syntax
            match values.len() {
                1 => {
                    let line = parse_grid_line(&values[0]);
                    return (line.clone(), line.clone(), GridLine::auto(), GridLine::auto());
                }
                2 => {
                    let row = parse_grid_line(&values[0]);
                    let col = parse_grid_line(&values[1]);
                    return (row.clone(), col.clone(), GridLine::auto(), GridLine::auto());
                }
                4 => {
                    let row_start = parse_grid_line(&values[0]);
                    let col_start = parse_grid_line(&values[1]);
                    let row_end = parse_grid_line(&values[2]);
                    let col_end = parse_grid_line(&values[3]);
                    return (row_start, col_start, row_end, col_end);
                }
                _ => return (GridLine::auto(), GridLine::auto(), GridLine::auto(), GridLine::auto()),
            }
        }
        _ => return (GridLine::auto(), GridLine::auto(), GridLine::auto(), GridLine::auto()),
    };

    // Parse "row-start / column-start / row-end / column-end" syntax from string
    if s.contains('/') {
        let parts: Vec<&str> = s.split('/').map(|p| p.trim()).collect();
        match parts.len() {
            1 => {
                let line = parse_grid_line(&CssValue::Ident(parts[0].to_string()));
                return (line.clone(), line.clone(), GridLine::auto(), GridLine::auto());
            }
            2 => {
                let row = parse_grid_line(&CssValue::Ident(parts[0].to_string()));
                let col = parse_grid_line(&CssValue::Ident(parts[1].to_string()));
                return (row, col, GridLine::auto(), GridLine::auto());
            }
            4 => {
                let row_start = parse_grid_line(&CssValue::Ident(parts[0].to_string()));
                let col_start = parse_grid_line(&CssValue::Ident(parts[1].to_string()));
                let row_end = parse_grid_line(&CssValue::Ident(parts[2].to_string()));
                let col_end = parse_grid_line(&CssValue::Ident(parts[3].to_string()));
                return (row_start, col_start, row_end, col_end);
            }
            _ => {}
        }
    }

    // Single named area
    let line = GridLine {
        line_number: 0,
        area_name: Some(s),
        span: None,
    };
    (line.clone(), GridLine::auto(), GridLine::auto(), GridLine::auto())
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

/// Font properties from shorthand
#[derive(Debug, Clone)]
pub struct FontShorthand {
    pub font_family: Vec<String>,
    pub font_size: Length,
    pub font_weight: FontWeight,
    pub font_style: FontStyle,
    pub font_variant: FontVariant,
    pub line_height: LineHeight,
}

impl Default for FontShorthand {
    fn default() -> Self {
        Self {
            font_family: vec!["serif".to_string()],
            font_size: Length::Px(16.0),
            font_weight: FontWeight::Normal,
            font_style: FontStyle::Normal,
            font_variant: FontVariant::Normal,
            line_height: LineHeight::Normal,
        }
    }
}

/// Parse font shorthand: [style] [variant] [weight] [stretch]? size/line-height family
fn parse_font_shorthand(value: &CssValue) -> Option<FontShorthand> {
    let values = match value {
        CssValue::List(v) => v.clone(),
        _ => vec![value.clone()],
    };
    
    let mut result = FontShorthand::default();
    let mut found_size = false;
    let mut found_family = false;
    let mut i = 0;
    
    while i < values.len() {
        let value = &values[i];
        
        if let CssValue::Ident(s) = value {
            match s.as_str() {
                // Font style
                "normal" | "italic" | "oblique" if !found_size => {
                    result.font_style = parse_font_style(value);
                    i += 1;
                    continue;
                }
                // Font variant
                "small-caps" if !found_size => {
                    result.font_variant = FontVariant::SmallCaps;
                    i += 1;
                    continue;
                }
                // Font weight
                "bold" | "bolder" | "lighter" if !found_size => {
                    result.font_weight = parse_font_weight(value);
                    i += 1;
                    continue;
                }
                _ => {}
            }
        }
        
        // Font weight as number
        if matches!(value, CssValue::Number(_)) && !found_size {
            result.font_weight = parse_font_weight(value);
            i += 1;
            continue;
        }
        
        // Font size (required) - look for length or number followed by /line-height
        if !found_size {
            result.font_size = parse_length(value);
            found_size = true;
            i += 1;
            continue;
        }
        
        // Everything else is font family
        if found_size && !found_family {
            let family_values: Vec<CssValue> = values[i..].to_vec();
            result.font_family = parse_font_family(&CssValue::List(family_values));
            found_family = true;
            break;
        }
        
        i += 1;
    }
    
    // Must have size and family
    if found_size && found_family {
        Some(result)
    } else {
        None
    }
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

    #[test]
    fn test_parse_object_fit() {
        assert_eq!(parse_object_fit(&CssValue::Ident("fill".to_string())), ObjectFit::Fill);
        assert_eq!(parse_object_fit(&CssValue::Ident("cover".to_string())), ObjectFit::Cover);
        assert_eq!(parse_object_fit(&CssValue::Ident("contain".to_string())), ObjectFit::Contain);
        assert_eq!(parse_object_fit(&CssValue::Ident("none".to_string())), ObjectFit::None);
        assert_eq!(parse_object_fit(&CssValue::Ident("scale-down".to_string())), ObjectFit::ScaleDown);
    }

    #[test]
    fn test_parse_background_image() {
        // Test url() parsing
        let mut func = crate::css::CssFunction::new("url");
        func.add_argument(CssValue::String("image.png".to_string()));
        let url_value = CssValue::Function(func);
        assert_eq!(parse_background_image(&url_value), Some("image.png".to_string()));

        // Test none
        assert_eq!(parse_background_image(&CssValue::Ident("none".to_string())), None);
    }

    #[test]
    fn test_parse_list_style() {
        assert_eq!(parse_list_style_type(&CssValue::Ident("disc".to_string())), ListStyleType::Disc);
        assert_eq!(parse_list_style_type(&CssValue::Ident("decimal".to_string())), ListStyleType::Decimal);
        assert_eq!(parse_list_style_type(&CssValue::Ident("none".to_string())), ListStyleType::None);

        assert_eq!(parse_list_style_position(&CssValue::Ident("inside".to_string())), ListStylePosition::Inside);
        assert_eq!(parse_list_style_position(&CssValue::Ident("outside".to_string())), ListStylePosition::Outside);
    }

    #[test]
    fn test_img_default_display() {
        let resolver = StyleResolver::new();
        let img = Element::new("img", vec![]);
        let style = resolver.compute_style(&img, None);
        
        // img should be inline by default
        assert_eq!(style.display, Display::Inline);
    }

    // ===== Additional tests for new CSS properties =====

    #[test]
    fn test_margin_padding_shorthand() {
        // 1 value - all sides same
        let value = CssValue::Length(10.0, Unit::Px);
        let (top, right, bottom, left) = parse_margin_or_padding(&value);
        assert_eq!(top, Length::Px(10.0));
        assert_eq!(right, Length::Px(10.0));
        assert_eq!(bottom, Length::Px(10.0));
        assert_eq!(left, Length::Px(10.0));

        // 2 values - vertical | horizontal
        let values = CssValue::List(vec![
            CssValue::Length(10.0, Unit::Px),
            CssValue::Length(20.0, Unit::Px),
        ]);
        let (top, right, bottom, left) = parse_margin_or_padding(&values);
        assert_eq!(top, Length::Px(10.0));
        assert_eq!(right, Length::Px(20.0));
        assert_eq!(bottom, Length::Px(10.0));
        assert_eq!(left, Length::Px(20.0));

        // 4 values - top | right | bottom | left
        let values = CssValue::List(vec![
            CssValue::Length(1.0, Unit::Px),
            CssValue::Length(2.0, Unit::Px),
            CssValue::Length(3.0, Unit::Px),
            CssValue::Length(4.0, Unit::Px),
        ]);
        let (top, right, bottom, left) = parse_margin_or_padding(&values);
        assert_eq!(top, Length::Px(1.0));
        assert_eq!(right, Length::Px(2.0));
        assert_eq!(bottom, Length::Px(3.0));
        assert_eq!(left, Length::Px(4.0));
    }

    #[test]
    fn test_css_variable_resolver() {
        use crate::css::CssVariableResolver;
        
        let mut resolver = CssVariableResolver::new();
        resolver.set_variable("--main-color", CssValue::HexColor(0xFF0000));
        resolver.set_variable("--spacing", CssValue::Length(10.0, Unit::Px));
        
        // Test existing variable
        let var_ref = CssValue::Var("--main-color".to_string(), None);
        let resolved = resolver.resolve(&var_ref);
        assert_eq!(resolved, CssValue::HexColor(0xFF0000));
        
        // Test fallback for undefined variable
        let var_undefined = CssValue::Var("--undefined".to_string(), 
            Some(Box::new(CssValue::Ident("blue".to_string()))));
        let resolved_fallback = resolver.resolve(&var_undefined);
        assert_eq!(resolved_fallback, CssValue::Ident("blue".to_string()));
    }

    #[test]
    fn test_font_properties() {
        let resolver = StyleResolver::new();
        let mut style = ComputedStyle::default();
        resolver.apply_initial_values(&mut style);

        // Font defaults
        assert_eq!(style.font_family, vec!["serif".to_string()]);
        assert_eq!(style.font_size, Length::Px(16.0));
        assert_eq!(style.font_weight, FontWeight::Normal);
        assert_eq!(style.font_style, FontStyle::Normal);
        assert_eq!(style.line_height, LineHeight::Normal);
    }
}
