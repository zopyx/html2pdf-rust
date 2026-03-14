//! Integration tests for SVG module

#[cfg(test)]
mod tests {
    use super::super::*;

    #[test]
    fn test_parse_simple_svg() {
        let svg = r#"<svg width="100" height="200"></svg>"#;
        let doc = parser::parse_svg(svg).unwrap();
        
        assert_eq!(doc.width, 75.0); // 100px * 0.75 = 75pt
        assert_eq!(doc.height, 150.0); // 200px * 0.75 = 150pt
    }

    #[test]
    fn test_parse_rect_element() {
        let svg = r#"<svg width="100" height="100">
            <rect x="10" y="20" width="30" height="40" fill="red"/>
        </svg>"#;
        let doc = parser::parse_svg(svg).unwrap();
        
        let rect = &doc.root.children[0];
        if let parser::SvgNode::Element(el) = rect {
            assert_eq!(el.tag_name, "rect");
            assert_eq!(el.get_attr("x"), Some("10"));
            assert_eq!(el.get_attr("fill"), Some("red"));
        } else {
            panic!("Expected element");
        }
    }

    #[test]
    fn test_path_parsing() {
        let path_data = "M 10 10 L 90 90";
        let commands = path::parse_path_data(path_data).unwrap();
        
        assert_eq!(commands.len(), 2);
        assert!(matches!(commands[0], path::PathCommand::MoveTo(_)));
        assert!(matches!(commands[1], path::PathCommand::LineTo(_)));
    }

    #[test]
    fn test_transform_parsing() {
        let transform = parse_transform("translate(10, 20)").unwrap();
        let point = types::Point::new(5.0, 5.0);
        let result = transform.apply(point);
        
        assert_eq!(result.x, 15.0);
        assert_eq!(result.y, 25.0);
    }

    #[test]
    fn test_style_parsing() {
        let style = style::SvgStyle::from_inline_style("fill: red; stroke: blue; stroke-width: 2");
        
        assert_eq!(style.fill, Some(style::Fill::Color(types::Color::new(255, 0, 0))));
        assert_eq!(style.stroke, Some(style::Stroke::Color(types::Color::new(0, 0, 255))));
        assert_eq!(style.stroke_width, 2.0);
    }

    #[test]
    fn test_render_svg_dimensions() {
        let svg_data = br#"<svg width="300" height="150" viewBox="0 0 300 150">
            <rect x="10" y="10" width="100" height="50" fill="blue"/>
        </svg>"#;
        
        let (width, height) = get_svg_dimensions(svg_data).unwrap();
        assert_eq!(width, 225.0); // 300px * 0.75
        assert_eq!(height, 112.5); // 150px * 0.75
    }
}
