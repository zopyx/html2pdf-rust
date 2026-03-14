//! CSS Selector parsing and matching
//!
//! Implements CSS Selectors Level 4

use std::fmt;

/// A CSS selector
#[derive(Debug, Clone, PartialEq)]
pub struct Selector {
    pub parts: Vec<SelectorPart>,
}

impl Selector {
    /// Create a new empty selector
    pub fn new() -> Self {
        Self { parts: Vec::new() }
    }

    /// Check if this is a universal selector (*)
    pub fn is_universal(&self) -> bool {
        self.parts.len() == 1 && matches!(self.parts[0], SelectorPart::Universal)
    }

    /// Get the specificity of this selector (a, b, c) tuple
    /// where a = ID count, b = class/attribute count, c = element count
    pub fn specificity(&self) -> (u32, u32, u32) {
        let mut a = 0;
        let mut b = 0;
        let mut c = 0;

        for part in &self.parts {
            match part {
                SelectorPart::Id(_) => a += 1,
                SelectorPart::Class(_) | SelectorPart::Attribute { .. } => b += 1,
                SelectorPart::Element(_) => c += 1,
                _ => {}
            }
        }

        (a, b, c)
    }

    /// Check if this selector matches an element
    pub fn matches(&self, element: &crate::html::Element) -> bool {
        // Simple matching - check if any part matches
        for part in &self.parts {
            match part {
                SelectorPart::Universal => return true,
                SelectorPart::Element(name) => {
                    if element.tag_name().eq_ignore_ascii_case(name) {
                        return true;
                    }
                }
                SelectorPart::Id(id) => {
                    if element.id() == Some(id) {
                        return true;
                    }
                }
                SelectorPart::Class(class) => {
                    if element.has_class(class) {
                        return true;
                    }
                }
                _ => {}
            }
        }
        false
    }
}

impl Default for Selector {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for Selector {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (i, part) in self.parts.iter().enumerate() {
            if i > 0 {
                write!(f, " ")?;
            }
            write!(f, "{}", part)?;
        }
        Ok(())
    }
}

/// Part of a CSS selector
#[derive(Debug, Clone, PartialEq)]
pub enum SelectorPart {
    /// Universal selector (*)
    Universal,
    /// Element type selector (e.g., "div")
    /// Also known as Type selector
    Type(String),
    /// Element type selector (alias for Type)
    Element(String),
    /// ID selector (e.g., "#header")
    Id(String),
    /// Class selector (e.g., ".menu")
    Class(String),
    /// Attribute selector (e.g., "[type='text']")
    Attribute {
        name: String,
        op: AttributeOp,
        value: Option<String>,
    },
    /// Pseudo-class (e.g., ":hover", ":nth-child(2n)")
    PseudoClass(String),
    /// Pseudo-class with argument
    PseudoClassWithArg(String, String),
    /// Pseudo-element (e.g., "::before", "::after")
    PseudoElement(String),
    /// Combinator
    Combinator(Combinator),
}

impl fmt::Display for SelectorPart {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SelectorPart::Universal => write!(f, "*"),
            SelectorPart::Element(name) => write!(f, "{}", name),
            SelectorPart::Id(id) => write!(f, "#{}", id),
            SelectorPart::Class(class) => write!(f, ".{}", class),
            SelectorPart::Attribute { name, op, value } => {
                write!(f, "[{}{}", name, op)?;
                if let Some(v) = value {
                    write!(f, "'{}'", v)?;
                }
                write!(f, "]")
            }
            SelectorPart::Type(name) => write!(f, "{}", name),
            SelectorPart::PseudoClass(name) => write!(f, ":{}", name),
            SelectorPart::PseudoClassWithArg(name, arg) => write!(f, ":{}({})", name, arg),
            SelectorPart::PseudoElement(name) => write!(f, "::{}", name),
            SelectorPart::Combinator(c) => write!(f, "{}", c),
        }
    }
}

/// Attribute selector operators
#[derive(Debug, Clone, PartialEq)]
pub enum AttributeOp {
    /// Just presence ([attr])
    Present,
    /// Exact equality ([attr="value"])
    Equals,
    /// Contains word ([attr~="value"])
    Contains,
    /// Starts with ([attr^="value"])
    StartsWith,
    /// Ends with ([attr$="value"])
    EndsWith,
    /// Contains substring ([attr*="value"])
    Substring,
    /// Dash-separated ([attr|="value"])
    Dash,
}

impl fmt::Display for AttributeOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AttributeOp::Present => Ok(()),
            AttributeOp::Equals => write!(f, "="),
            AttributeOp::Contains => write!(f, "~="),
            AttributeOp::StartsWith => write!(f, "^="),
            AttributeOp::EndsWith => write!(f, "$="),
            AttributeOp::Substring => write!(f, "*="),
            AttributeOp::Dash => write!(f, "|="),
        }
    }
}

/// Combinator between selector parts
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Combinator {
    /// Descendant (space)
    Descendant,
    /// Child (>)
    Child,
    /// Adjacent sibling (+)
    Adjacent,
    /// General sibling (~)
    Sibling,
    /// General sibling (alias for Sibling)
    GeneralSibling,
}

impl fmt::Display for Combinator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Combinator::Descendant => Ok(()),
            Combinator::Child => write!(f, ">"),
            Combinator::Adjacent => write!(f, "+"),
            Combinator::Sibling | Combinator::GeneralSibling => write!(f, "~"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_selector_specificity() {
        let mut selector = Selector::new();
        selector.parts.push(SelectorPart::Id("header".to_string()));
        selector.parts.push(SelectorPart::Class("menu".to_string()));
        selector.parts.push(SelectorPart::Element("div".to_string()));

        assert_eq!(selector.specificity(), (1, 1, 1));
    }

    #[test]
    fn test_universal_selector() {
        let mut selector = Selector::new();
        selector.parts.push(SelectorPart::Universal);

        assert!(selector.is_universal());
    }

    #[test]
    fn test_selector_display() {
        let mut selector = Selector::new();
        selector.parts.push(SelectorPart::Element("div".to_string()));
        selector.parts.push(SelectorPart::Class("container".to_string()));

        assert_eq!(selector.to_string(), "div .container");
    }
}
