//! Tests for PrintCSS features

#[cfg(test)]
mod tests {
    use super::super::*;
    use crate::css::at_rules::{PageRule, PageSelector, PageMarginBox, MarginBoxType};
    use crate::css::parser::Declaration;
    use crate::css::values::CssValue;
    use crate::types::Margins;

    #[test]
    fn test_page_context_creation() {
        let ctx = PageContext::new(1, 10);
        assert_eq!(ctx.page_number, 1);
        assert_eq!(ctx.total_pages, 10);
        assert!(ctx.is_first);
        assert!(!ctx.is_left); // Page 1 is right (recto)
        assert!(!ctx.is_blank);

        let ctx2 = PageContext::new(2, 10);
        assert!(!ctx2.is_first);
        assert!(ctx2.is_left); // Page 2 is left (verso)
    }

    #[test]
    fn test_page_context_selectors() {
        let ctx = PageContext::new(1, 10);
        let selectors = ctx.applicable_selectors();
        
        // Should include: default (""), First, Right
        assert!(selectors.contains(&PageSelector::First));
        assert!(selectors.contains(&PageSelector::Right));
        assert!(!selectors.contains(&PageSelector::Left));
    }

    #[test]
    fn test_page_master_from_rule() {
        let mut rule = PageRule::new();
        rule.add_selector(PageSelector::First);
        rule.add_declaration(Declaration::new("margin", CssValue::Length(50.0, crate::css::Unit::Pt)));
        
        let master = PageMaster::from_page_rule(&rule);
        
        assert_eq!(master.selectors.len(), 1);
        assert!(master.selectors.contains(&PageSelector::First));
        assert_eq!(master.margins.top, 50.0);
    }

    #[test]
    fn test_page_size_default() {
        let size = PageSize::default();
        assert!(size.width > 0.0);
        assert!(size.height > 0.0);
    }

    #[test]
    fn test_page_size_from_paper_size() {
        use crate::types::{PaperSize, Orientation};
        
        let size = PageSize::from_paper_size(PaperSize::Letter, Orientation::Portrait);
        assert_eq!(size.width, 612.0);
        assert_eq!(size.height, 792.0);
        
        let size_landscape = PageSize::from_paper_size(PaperSize::Letter, Orientation::Landscape);
        assert_eq!(size_landscape.width, 792.0);
        assert_eq!(size_landscape.height, 612.0);
    }

    #[test]
    fn test_margin_box_content_parsing() {
        let value = CssValue::String("Page ".to_string());
        let parts = parse_content_value(&value);
        
        assert_eq!(parts.len(), 1);
        matches!(parts[0], MarginContentPart::Text(ref s) if s == "Page ");
    }

    #[test]
    fn test_margin_box_content_counter() {
        use crate::css::values::CssFunction;
        
        let mut func = CssFunction::new("counter");
        func.add_argument(CssValue::Ident("page".to_string()));
        let value = CssValue::Function(func);
        
        let parts = parse_content_value(&value);
        
        assert_eq!(parts.len(), 1);
        matches!(parts[0], MarginContentPart::PageCounter);
    }

    #[test]
    fn test_margin_box_content_string_ref() {
        use crate::css::values::CssFunction;
        
        let mut func = CssFunction::new("string");
        func.add_argument(CssValue::Ident("header".to_string()));
        let value = CssValue::Function(func);
        
        let parts = parse_content_value(&value);
        
        assert_eq!(parts.len(), 1);
        matches!(parts[0], MarginContentPart::StringRef(ref s) if s == "header");
    }

    #[test]
    fn test_page_counter() {
        let mut counter = PageCounter::new();
        assert_eq!(counter.current, 1);
        
        counter.increment();
        assert_eq!(counter.current, 2);
        
        counter.increment();
        assert_eq!(counter.current, 3);
        
        counter.reset(10);
        assert_eq!(counter.current, 10);
    }

    #[test]
    fn test_page_counter_named() {
        let mut counter = PageCounter::new();
        
        counter.set_named_counter("chapter", 1);
        assert_eq!(counter.get_named_counter("chapter"), Some(1));
        
        counter.set_named_counter("chapter", 2);
        assert_eq!(counter.get_named_counter("chapter"), Some(2));
        
        counter.reset_named_counter("chapter", 5);
        assert_eq!(counter.get_named_counter("chapter"), Some(5));
    }

    #[test]
    fn test_margin_box_rect_top_center() {
        let page_size = PageSize { width: 595.0, height: 842.0 };
        let margins = Margins::all(72.0);
        
        let rect = get_margin_box_rect(MarginBoxType::TopCenter, &page_size, &margins);
        
        assert!(rect.width > 0.0);
        assert!(rect.height > 0.0);
        assert_eq!(rect.y, page_size.height - margins.top);
    }

    #[test]
    fn test_bookmark_creation() {
        let bookmark = Bookmark::new("Chapter 1", 1, 0);
        assert_eq!(bookmark.title, "Chapter 1");
        assert_eq!(bookmark.page, 1);
        assert_eq!(bookmark.level, 0);
        assert!(bookmark.children.is_empty());
    }

    #[test]
    fn test_bookmark_with_children() {
        let mut parent = Bookmark::new("Part 1", 1, 0);
        let child = Bookmark::new("Chapter 1", 2, 1);
        
        parent.add_child(child);
        
        assert_eq!(parent.children.len(), 1);
        assert_eq!(parent.children[0].title, "Chapter 1");
    }

    #[test]
    fn test_parse_break_value() {
        assert!(matches!(
            parse_break_value(&CssValue::Ident("auto".to_string())),
            BreakType::Auto
        ));
        assert!(matches!(
            parse_break_value(&CssValue::Ident("always".to_string())),
            BreakType::Always
        ));
        assert!(matches!(
            parse_break_value(&CssValue::Ident("avoid".to_string())),
            BreakType::Avoid
        ));
        assert!(matches!(
            parse_break_value(&CssValue::Ident("page".to_string())),
            BreakType::Page
        ));
        assert!(matches!(
            parse_break_value(&CssValue::Ident("left".to_string())),
            BreakType::Left
        ));
        assert!(matches!(
            parse_break_value(&CssValue::Ident("right".to_string())),
            BreakType::Right
        ));
    }

    #[test]
    fn test_parse_break_inside_value() {
        assert!(matches!(
            parse_break_inside_value(&CssValue::Ident("auto".to_string())),
            BreakInside::Auto
        ));
        assert!(matches!(
            parse_break_inside_value(&CssValue::Ident("avoid".to_string())),
            BreakInside::Avoid
        ));
        assert!(matches!(
            parse_break_inside_value(&CssValue::Ident("avoid-page".to_string())),
            BreakInside::AvoidPage
        ));
    }

    #[test]
    fn test_running_strings() {
        let mut ctx = PageContext::new(1, 10);
        ctx.running_strings.insert("header".to_string(), "Document Title".to_string());
        
        assert_eq!(ctx.get_running_string("header"), Some("Document Title"));
        assert_eq!(ctx.get_running_string("footer"), None);
    }
}
