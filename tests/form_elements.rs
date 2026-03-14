//! Tests for HTML form element rendering support

use html2pdf::html::parse_html;
use html2pdf::layout::{build_box_tree, is_form_element, create_form_box, FormControlType};
use html2pdf::layout::style::StyleResolver;

#[test]
fn test_is_form_element() {
    assert!(is_form_element("input"));
    assert!(is_form_element("textarea"));
    assert!(is_form_element("select"));
    assert!(is_form_element("button"));
    assert!(is_form_element("label"));
    assert!(!is_form_element("div"));
    assert!(!is_form_element("span"));
}

#[test]
fn test_form_control_type_from_input() {
    assert_eq!(FormControlType::from_input("text"), FormControlType::Text);
    assert_eq!(FormControlType::from_input("password"), FormControlType::Password);
    assert_eq!(FormControlType::from_input("email"), FormControlType::Email);
    assert_eq!(FormControlType::from_input("number"), FormControlType::Number);
    assert_eq!(FormControlType::from_input("checkbox"), FormControlType::Checkbox);
    assert_eq!(FormControlType::from_input("radio"), FormControlType::Radio);
    assert_eq!(FormControlType::from_input("date"), FormControlType::Date);
    assert_eq!(FormControlType::from_input("time"), FormControlType::Time);
    assert_eq!(FormControlType::from_input("file"), FormControlType::File);
    assert_eq!(FormControlType::from_input("hidden"), FormControlType::Hidden);
    assert_eq!(FormControlType::from_input("UNKNOWN"), FormControlType::Text);
}

#[test]
fn test_form_box_from_input_text() {
    let html = r#"<input type="text" name="username" value="john" placeholder="Enter username">"#;
    let doc = parse_html(html).unwrap();
    let input = doc.get_elements_by_tag_name("input").pop().unwrap();
    
    let form_box = create_form_box(input).unwrap();
    assert_eq!(form_box.control_type, FormControlType::Text);
    assert_eq!(form_box.name, "username");
    assert_eq!(form_box.value, "john");
    assert_eq!(form_box.placeholder, "Enter username");
}

#[test]
fn test_form_box_from_input_checkbox() {
    let html = r#"<input type="checkbox" name="agree" checked>"#;
    let doc = parse_html(html).unwrap();
    let input = doc.get_elements_by_tag_name("input").pop().unwrap();
    
    let form_box = create_form_box(input).unwrap();
    assert_eq!(form_box.control_type, FormControlType::Checkbox);
    assert!(form_box.checked);
}

#[test]
fn test_form_box_from_input_radio() {
    let html = r#"<input type="radio" name="gender" value="male">"#;
    let doc = parse_html(html).unwrap();
    let input = doc.get_elements_by_tag_name("input").pop().unwrap();
    
    let form_box = create_form_box(input).unwrap();
    assert_eq!(form_box.control_type, FormControlType::Radio);
    assert!(!form_box.checked);
}

#[test]
fn test_form_box_from_input_hidden() {
    let html = r#"<input type="hidden" name="token" value="secret123">"#;
    let doc = parse_html(html).unwrap();
    let input = doc.get_elements_by_tag_name("input").pop().unwrap();
    
    let form_box = create_form_box(input).unwrap();
    assert_eq!(form_box.control_type, FormControlType::Hidden);
    assert!(!form_box.control_type.is_visible());
}

#[test]
fn test_form_box_from_textarea() {
    let html = r#"<textarea name="description" placeholder="Enter description">Hello World</textarea>"#;
    let doc = parse_html(html).unwrap();
    let textarea = doc.get_elements_by_tag_name("textarea").pop().unwrap();
    
    let form_box = create_form_box(textarea).unwrap();
    assert_eq!(form_box.control_type, FormControlType::Textarea);
    assert_eq!(form_box.value, "Hello World");
    assert_eq!(form_box.placeholder, "Enter description");
}

#[test]
fn test_form_box_from_select() {
    let html = r#"
        <select name="country">
            <option value="us">United States</option>
            <option value="uk" selected>United Kingdom</option>
            <option value="ca">Canada</option>
        </select>
    "#;
    let doc = parse_html(html).unwrap();
    let select = doc.get_elements_by_tag_name("select").pop().unwrap();
    
    let form_box = create_form_box(select).unwrap();
    assert_eq!(form_box.control_type, FormControlType::Select);
    assert_eq!(form_box.options.len(), 3);
    assert_eq!(form_box.selected_index, Some(1));
}

#[test]
fn test_form_box_from_button() {
    let html = r#"<button type="submit" name="save">Save Changes</button>"#;
    let doc = parse_html(html).unwrap();
    let button = doc.get_elements_by_tag_name("button").pop().unwrap();
    
    let form_box = create_form_box(button).unwrap();
    assert_eq!(form_box.control_type, FormControlType::Submit);
    assert_eq!(form_box.value, "Save Changes");
}

#[test]
fn test_form_box_from_label() {
    let html = r#"<label for="username">Username:</label>"#;
    let doc = parse_html(html).unwrap();
    let label = doc.get_elements_by_tag_name("label").pop().unwrap();
    
    let form_box = create_form_box(label).unwrap();
    assert_eq!(form_box.control_type, FormControlType::Label);
    assert_eq!(form_box.value, "Username:");
    assert_eq!(form_box.label_for, Some("username".to_string()));
}

#[test]
fn test_password_display_text() {
    let html = r#"<input type="password" value="secret">"#;
    let doc = parse_html(html).unwrap();
    let input = doc.get_elements_by_tag_name("input").pop().unwrap();
    
    let form_box = create_form_box(input).unwrap();
    assert_eq!(form_box.display_text(), "••••••");
}

#[test]
fn test_checkbox_display_text() {
    let checked_html = r#"<input type="checkbox" checked>"#;
    let doc1 = parse_html(checked_html).unwrap();
    let input1 = doc1.get_elements_by_tag_name("input").pop().unwrap();
    let form_box1 = create_form_box(input1).unwrap();
    assert_eq!(form_box1.display_text(), "☑");

    let unchecked_html = r#"<input type="checkbox">"#;
    let doc2 = parse_html(unchecked_html).unwrap();
    let input2 = doc2.get_elements_by_tag_name("input").pop().unwrap();
    let form_box2 = create_form_box(input2).unwrap();
    assert_eq!(form_box2.display_text(), "☐");
}

#[test]
fn test_radio_display_text() {
    let checked_html = r#"<input type="radio" checked>"#;
    let doc1 = parse_html(checked_html).unwrap();
    let input1 = doc1.get_elements_by_tag_name("input").pop().unwrap();
    let form_box1 = create_form_box(input1).unwrap();
    assert_eq!(form_box1.display_text(), "◉");

    let unchecked_html = r#"<input type="radio">"#;
    let doc2 = parse_html(unchecked_html).unwrap();
    let input2 = doc2.get_elements_by_tag_name("input").pop().unwrap();
    let form_box2 = create_form_box(input2).unwrap();
    assert_eq!(form_box2.display_text(), "○");
}

#[test]
fn test_form_intrinsic_dimensions() {
    use html2pdf::layout::form::FormBox;
    
    // Text input
    let text_form = FormBox {
        control_type: FormControlType::Text,
        value: "test".to_string(),
        placeholder: "".to_string(),
        disabled: false,
        readonly: false,
        checked: false,
        selected_index: None,
        options: vec![],
        name: "".to_string(),
        id: "".to_string(),
        filename: None,
        label_for: None,
    };
    assert_eq!(text_form.intrinsic_width(), 200.0);
    assert_eq!(text_form.intrinsic_height(), 28.0);

    // Button
    let button_form = FormBox {
        control_type: FormControlType::Button,
        value: "Click".to_string(),
        placeholder: "".to_string(),
        disabled: false,
        readonly: false,
        checked: false,
        selected_index: None,
        options: vec![],
        name: "".to_string(),
        id: "".to_string(),
        filename: None,
        label_for: None,
    };
    assert!(button_form.intrinsic_width() > 0.0);
    assert_eq!(button_form.intrinsic_height(), 32.0);
}

#[test]
fn test_build_box_tree_with_form_elements() {
    let html = r#"
        <form>
            <input type="text" name="username" value="john">
            <input type="checkbox" name="agree" checked>
            <button type="submit">Submit</button>
        </form>
    "#;
    let doc = parse_html(html).unwrap();
    let body = doc.body_element().unwrap();
    
    let resolver = StyleResolver::new();
    let root_box = build_box_tree(body, &|element| resolver.resolve_display(element));
    
    // The form should contain the form elements
    let form = root_box.children.iter()
        .find(|b| b.element().map(|e| e.tag_name() == "form").unwrap_or(false));
    assert!(form.is_some(), "Form element should be in box tree");
    
    // Count form control boxes
    let form_control_count = form.unwrap().children.iter()
        .filter(|b| b.form_data().is_some())
        .count();
    assert_eq!(form_control_count, 3, "Should have 3 form controls");
}

#[test]
fn test_box_type_form_methods() {
    use html2pdf::layout::BoxType;
    
    assert!(BoxType::Form.is_form());
    assert!(!BoxType::Block.is_form());
    assert!(!BoxType::Inline.is_form());
    
    // Form boxes should be inline-level
    assert!(BoxType::Form.is_inline_level());
    assert!(!BoxType::Form.is_block_level());
}
