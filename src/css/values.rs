//! CSS Values parsing and representation
//!
//! Implements CSS Values and Units Module Level 4

use std::fmt;

/// CSS value types
#[derive(Debug, Clone, PartialEq)]
pub enum CssValue {
    /// Identifier (keyword)
    Ident(String),
    /// String value
    String(String),
    /// Number value
    Number(f32),
    /// Integer value
    Integer(i32),
    /// Percentage value
    Percentage(f32),
    /// Length value with unit
    Length(f32, Unit),
    /// Hex color value
    HexColor(u32),
    /// URL/URI value
    Url(String),
    /// Function call
    Function(CssFunction),
    /// List of values
    List(Vec<CssValue>),
    /// Literal character/string
    Literal(String),
    /// Parenthesized expression
    Parenthesized(Box<CssValue>),
    /// Keyword value (deprecated, use Ident)
    Keyword(String),
    /// Color value (deprecated, use HexColor)
    Color(ColorValue),
    /// Calc expression
    Calc(Box<CssValue>),
    /// Initial keyword
    Initial,
    /// Inherit keyword
    Inherit,
    /// Unset keyword
    Unset,
    /// Revert keyword
    Revert,
    /// Auto keyword
    Auto,
    /// None keyword
    None,
}

impl CssValue {
    /// Convert to points (for PDF output)
    pub fn to_pt(&self, base_font_size: f32) -> Option<f32> {
        match self {
            CssValue::Length(n, unit) => Some(unit.to_pt(*n, base_font_size)),
            CssValue::Number(n) => Some(*n),
            CssValue::Integer(n) => Some(*n as f32),
            _ => None,
        }
    }

    /// Check if the value is a length
    pub fn is_length(&self) -> bool {
        matches!(self, CssValue::Length(_, _))
    }

    /// Check if the value is a percentage
    pub fn is_percentage(&self) -> bool {
        matches!(self, CssValue::Percentage(_))
    }

    /// Check if the value is auto
    pub fn is_auto(&self) -> bool {
        matches!(self, CssValue::Auto) || 
        matches!(self, CssValue::Ident(s) if s == "auto")
    }
}

impl CssFunction {
    /// Create a new function (alias for new)
    pub fn with_name(name: impl Into<String>) -> Self {
        Self::new(name)
    }
}

impl fmt::Display for CssValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CssValue::Keyword(k) => write!(f, "{}", k),
            CssValue::Length(n, u) => write!(f, "{}{}", n, u),
            CssValue::Percentage(n) => write!(f, "{}%", n),
            CssValue::Color(c) => write!(f, "{}", c),
            CssValue::Url(u) => write!(f, "url('{}')", u),
            CssValue::String(s) => write!(f, "'{}'", s),
            CssValue::Number(n) => write!(f, "{}", n),
            CssValue::Integer(n) => write!(f, "{}", n),
            CssValue::Function(func) => write!(f, "{}", func),
            CssValue::List(list) => {
                for (i, v) in list.iter().enumerate() {
                    if i > 0 {
                        write!(f, " ")?;
                    }
                    write!(f, "{}", v)?;
                }
                Ok(())
            }
            CssValue::Calc(expr) => write!(f, "calc({})", expr),
            CssValue::Initial => write!(f, "initial"),
            CssValue::Inherit => write!(f, "inherit"),
            CssValue::Unset => write!(f, "unset"),
            CssValue::Revert => write!(f, "revert"),
            CssValue::Auto => write!(f, "auto"),
            CssValue::None => write!(f, "none"),
            CssValue::Ident(s) => write!(f, "{}", s),
            CssValue::HexColor(h) => write!(f, "#{:06x}", h),
            CssValue::Literal(s) => write!(f, "{}", s),
            CssValue::Parenthesized(v) => write!(f, "({})", v),
        }
    }
}

/// CSS units
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Unit {
    // Absolute units
    /// Points (1/72 inch)
    Pt,
    /// Pixels (1/96 inch)
    Px,
    /// Inches
    In,
    /// Centimeters
    Cm,
    /// Millimeters
    Mm,
    /// Picas (1/6 inch)
    Pc,
    
    // Relative font units
    /// Font size of the element
    Em,
    /// Font size of the root element
    Rem,
    /// x-height of the font
    Ex,
    /// Width of the "0" glyph
    Ch,
    
    // Viewport-relative units
    /// 1% of viewport width
    Vw,
    /// 1% of viewport height
    Vh,
    /// 1% of the smaller dimension
    Vmin,
    /// 1% of the larger dimension
    Vmax,
    /// Percentage (alias)
    Percent,
}

impl Unit {
    /// Convert a value in this unit to points
    pub fn to_pt(&self, value: f32, base_font_size: f32) -> f32 {
        match self {
            Unit::Pt => value,
            Unit::Px => value * 0.75, // 96 DPI
            Unit::In => value * 72.0,
            Unit::Cm => value * 28.346_46,
            Unit::Mm => value * 2.834_646,
            Unit::Pc => value * 12.0,
            Unit::Em => value * base_font_size,
            Unit::Rem => value * base_font_size, // Simplified
            Unit::Ex => value * base_font_size * 0.5, // Approximation
            Unit::Ch => value * base_font_size * 0.5, // Approximation
            Unit::Vw | Unit::Vh | Unit::Vmin | Unit::Vmax => {
                // Viewport units - would need viewport context
                value * 6.0 // Approximate for A4
            }
            Unit::Percent => value * base_font_size / 100.0, // Percentage of parent
        }
    }

    /// Parse unit from string
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "pt" => Some(Unit::Pt),
            "px" => Some(Unit::Px),
            "in" => Some(Unit::In),
            "cm" => Some(Unit::Cm),
            "mm" => Some(Unit::Mm),
            "pc" => Some(Unit::Pc),
            "em" => Some(Unit::Em),
            "rem" => Some(Unit::Rem),
            "ex" => Some(Unit::Ex),
            "ch" => Some(Unit::Ch),
            "vw" => Some(Unit::Vw),
            "vh" => Some(Unit::Vh),
            "vmin" => Some(Unit::Vmin),
            "vmax" => Some(Unit::Vmax),
            _ => None,
        }
    }

    /// Check if this is an absolute unit
    pub fn is_absolute(&self) -> bool {
        matches!(self, Unit::Pt | Unit::Px | Unit::In | Unit::Cm | Unit::Mm | Unit::Pc)
    }

    /// Check if this is a relative font unit
    pub fn is_font_relative(&self) -> bool {
        matches!(self, Unit::Em | Unit::Rem | Unit::Ex | Unit::Ch)
    }

    /// Check if this is a viewport-relative unit
    pub fn is_viewport_relative(&self) -> bool {
        matches!(self, Unit::Vw | Unit::Vh | Unit::Vmin | Unit::Vmax)
    }
}

impl fmt::Display for Unit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Unit::Pt => write!(f, "pt"),
            Unit::Px => write!(f, "px"),
            Unit::In => write!(f, "in"),
            Unit::Cm => write!(f, "cm"),
            Unit::Mm => write!(f, "mm"),
            Unit::Pc => write!(f, "pc"),
            Unit::Em => write!(f, "em"),
            Unit::Rem => write!(f, "rem"),
            Unit::Ex => write!(f, "ex"),
            Unit::Ch => write!(f, "ch"),
            Unit::Vw => write!(f, "vw"),
            Unit::Vh => write!(f, "vh"),
            Unit::Vmin => write!(f, "vmin"),
            Unit::Vmax => write!(f, "vmax"),
            Unit::Percent => write!(f, "%"),
        }
    }
}

/// CSS color value
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ColorValue {
    /// Named color
    Named(&'static str),
    /// RGB color
    Rgb(u8, u8, u8),
    /// RGBA color
    Rgba(u8, u8, u8, f32),
    /// HSL color
    Hsl(f32, f32, f32),
    /// HSLA color
    Hsla(f32, f32, f32, f32),
    /// Hex color (stored as RGB)
    Hex(u32),
    /// Current color keyword
    CurrentColor,
    /// Transparent
    Transparent,
}

impl ColorValue {
    /// Convert to RGB tuple
    pub fn as_rgb(self) -> (u8, u8, u8) {
        match self {
            ColorValue::Named(name) => Self::named_color_to_rgb(name),
            ColorValue::Rgb(r, g, b) => (r, g, b),
            ColorValue::Rgba(r, g, b, _) => (r, g, b),
            ColorValue::Hex(hex) => {
                let r = ((hex >> 16) & 0xFF) as u8;
                let g = ((hex >> 8) & 0xFF) as u8;
                let b = (hex & 0xFF) as u8;
                (r, g, b)
            }
            ColorValue::Transparent => (0, 0, 0),
            _ => (0, 0, 0), // HSL conversion not implemented
        }
    }

    fn named_color_to_rgb(name: &str) -> (u8, u8, u8) {
        match name.to_lowercase().as_str() {
            "black" => (0, 0, 0),
            "white" => (255, 255, 255),
            "red" => (255, 0, 0),
            "green" => (0, 128, 0),
            "blue" => (0, 0, 255),
            "yellow" => (255, 255, 0),
            "cyan" => (0, 255, 255),
            "magenta" => (255, 0, 255),
            "silver" => (192, 192, 192),
            "gray" => (128, 128, 128),
            "maroon" => (128, 0, 0),
            "olive" => (128, 128, 0),
            "lime" => (0, 255, 0),
            "aqua" => (0, 255, 255),
            "teal" => (0, 128, 128),
            "navy" => (0, 0, 128),
            "fuchsia" => (255, 0, 255),
            "purple" => (128, 0, 128),
            _ => (0, 0, 0),
        }
    }
}

impl fmt::Display for ColorValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ColorValue::Named(name) => write!(f, "{}", name),
            ColorValue::Rgb(r, g, b) => write!(f, "rgb({}, {}, {})", r, g, b),
            ColorValue::Rgba(r, g, b, a) => write!(f, "rgba({}, {}, {}, {})", r, g, b, a),
            ColorValue::Hsl(h, s, l) => write!(f, "hsl({}, {}%, {}%)", h, s, l),
            ColorValue::Hsla(h, s, l, a) => write!(f, "hsla({}, {}%, {}%, {})", h, s, l, a),
            ColorValue::Hex(hex) => write!(f, "#{:06x}", hex),
            ColorValue::CurrentColor => write!(f, "currentColor"),
            ColorValue::Transparent => write!(f, "transparent"),
        }
    }
}

/// CSS function call
#[derive(Debug, Clone, PartialEq)]
pub struct CssFunction {
    pub name: String,
    pub arguments: Vec<CssValue>,
    /// Alias for arguments (for compatibility)
    pub args: Vec<CssValue>,
}

impl CssFunction {
    /// Create a new function
    pub fn new(name: impl Into<String>) -> Self {
        let name = name.into();
        Self {
            name: name.clone(),
            arguments: Vec::new(),
            args: Vec::new(),
        }
    }

    /// Add an argument
    pub fn add_argument(&mut self, value: CssValue) {
        self.arguments.push(value.clone());
        self.args.push(value);
    }
}

impl fmt::Display for CssFunction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}(", self.name)?;
        for (i, arg) in self.arguments.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{}", arg)?;
        }
        write!(f, ")")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unit_conversion() {
        assert_eq!(Unit::Pt.to_pt(72.0, 12.0), 72.0);
        assert!((Unit::Px.to_pt(96.0, 12.0) - 72.0).abs() < 0.01);
        assert!((Unit::In.to_pt(1.0, 12.0) - 72.0).abs() < 0.01);
    }

    #[test]
    fn test_color_to_rgb() {
        let color = ColorValue::Rgb(255, 128, 0);
        assert_eq!(color.as_rgb(), (255, 128, 0));

        let named = ColorValue::Named("red");
        assert_eq!(named.as_rgb(), (255, 0, 0));
    }

    #[test]
    fn test_css_value_display() {
        assert_eq!(CssValue::Keyword("auto".to_string()).to_string(), "auto");
        assert_eq!(CssValue::Length(12.0, Unit::Px).to_string(), "12px");
        assert_eq!(CssValue::Percentage(50.0).to_string(), "50%");
    }
}
