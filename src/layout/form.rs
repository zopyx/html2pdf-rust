//! HTML Form Element Layout and Rendering
//!
//! This module handles the layout and rendering of HTML form elements including:
//! - Input elements (text, password, email, number, checkbox, radio, date, time, file, hidden)
//! - Textarea elements
//! - Select/Option elements
//! - Button elements
//! - Label elements

use crate::html::Element;
use crate::layout::box_model::{LayoutBox, Dimensions, EdgeSizes};
use crate::layout::style::ComputedStyle;
use crate::types::{Rect, Color};

/// Type of form control
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FormControlType {
    /// Text input
    Text,
    /// Password input
    Password,
    /// Email input
    Email,
    /// Number input
    Number,
    /// Checkbox
    Checkbox,
    /// Radio button
    Radio,
    /// Date input
    Date,
    /// Time input
    Time,
    /// File input
    File,
    /// Hidden input (not rendered)
    Hidden,
    /// Textarea
    Textarea,
    /// Select dropdown
    Select,
    /// Button
    Button,
    /// Submit button
    Submit,
    /// Reset button
    Reset,
    /// Label
    Label,
}

impl FormControlType {
    /// Get the form control type from an input element's type attribute
    pub fn from_input_type(input_type: &str) -> Self {
        match input_type.to_ascii_lowercase().as_str() {
            "password" => FormControlType::Password,
            "email" => FormControlType::Email,
            "number" => FormControlType::Number,
            "checkbox" => FormControlType::Checkbox,
            "radio" => FormControlType::Radio,
            "date" => FormControlType::Date,
            "time" => FormControlType::Time,
            "file" => FormControlType::File,
            "hidden" => FormControlType::Hidden,
            _ => FormControlType::Text,
        }
    }

    /// Check if this form control should be rendered
    pub fn is_visible(&self) -> bool {
        !matches!(self, FormControlType::Hidden)
    }

    /// Check if this is a text-based input
    pub fn is_text_input(&self) -> bool {
        matches!(self, FormControlType::Text | FormControlType::Password | FormControlType::Email | FormControlType::Number)
    }

    /// Check if this is a selection control (checkbox/radio)
    pub fn is_selection_control(&self) -> bool {
        matches!(self, FormControlType::Checkbox | FormControlType::Radio)
    }

    /// Check if this is a button type
    pub fn is_button(&self) -> bool {
        matches!(self, FormControlType::Button | FormControlType::Submit | FormControlType::Reset)
    }
}

/// Data for form control boxes
#[derive(Debug, Clone, PartialEq)]
pub struct FormBox {
    /// Type of form control
    pub control_type: FormControlType,
    /// Current value/text content
    pub value: String,
    /// Placeholder text
    pub placeholder: String,
    /// Whether the control is disabled
    pub disabled: bool,
    /// Whether the control is readonly
    pub readonly: bool,
    /// Whether checkbox/radio is checked
    pub checked: bool,
    /// For select: selected option index
    pub selected_index: Option<usize>,
    /// For select: options list
    pub options: Vec<SelectOption>,
    /// Name attribute
    pub name: String,
    /// ID attribute
    pub id: String,
    /// For file input: filename
    pub filename: Option<String>,
    /// Associated label text (for labels)
    pub label_for: Option<String>,
}

/// Option for select elements
#[derive(Debug, Clone, PartialEq)]
pub struct SelectOption {
    pub value: String,
    pub text: String,
    pub selected: bool,
    pub disabled: bool,
}

impl FormBox {
    /// Create a new form box from an input element
    pub fn from_input(element: &Element) -> Self {
        let input_type = element.attr("type").unwrap_or("text");
        let control_type = FormControlType::from_input_type(input_type);
        
        Self {
            control_type,
            value: element.attr("value").unwrap_or("").to_string(),
            placeholder: element.attr("placeholder").unwrap_or("").to_string(),
            disabled: element.has_attr("disabled"),
            readonly: element.has_attr("readonly"),
            checked: element.has_attr("checked"),
            selected_index: None,
            options: Vec::new(),
            name: element.attr("name").unwrap_or("").to_string(),
            id: element.attr("id").unwrap_or("").to_string(),
            filename: None,
            label_for: None,
        }
    }

    /// Create a new form box from a textarea element
    pub fn from_textarea(element: &Element) -> Self {
        let value = element.text_content();
        
        Self {
            control_type: FormControlType::Textarea,
            value,
            placeholder: element.attr("placeholder").unwrap_or("").to_string(),
            disabled: element.has_attr("disabled"),
            readonly: element.has_attr("readonly"),
            checked: false,
            selected_index: None,
            options: Vec::new(),
            name: element.attr("name").unwrap_or("").to_string(),
            id: element.attr("id").unwrap_or("").to_string(),
            filename: None,
            label_for: None,
        }
    }

    /// Create a new form box from a select element
    pub fn from_select(element: &Element) -> Self {
        let mut options = Vec::new();
        let mut selected_index = None;
        let mut index = 0;

        // Parse option elements
        for child in element.children() {
            if let Some(el) = child.as_element() {
                if el.tag_name().eq_ignore_ascii_case("option") {
                    let opt = SelectOption {
                        value: el.attr("value").unwrap_or("").to_string(),
                        text: el.text_content(),
                        selected: el.has_attr("selected"),
                        disabled: el.has_attr("disabled"),
                    };
                    if opt.selected && selected_index.is_none() {
                        selected_index = Some(index);
                    }
                    options.push(opt);
                    index += 1;
                }
            }
        }

        // Default to first option if none selected
        if selected_index.is_none() && !options.is_empty() {
            selected_index = Some(0);
        }

        Self {
            control_type: FormControlType::Select,
            value: selected_index.map(|i| options.get(i).map(|o| o.text.clone()).unwrap_or_default()).unwrap_or_default(),
            placeholder: "",
            disabled: element.has_attr("disabled"),
            readonly: false,
            checked: false,
            selected_index,
            options,
            name: element.attr("name").unwrap_or("").to_string(),
            id: element.attr("id").unwrap_or("").to_string(),
            filename: None,
            label_for: None,
        }
    }

    /// Create a new form box from a button element
    pub fn from_button(element: &Element) -> Self {
        let button_type = element.attr("type").unwrap_or("submit");
        let control_type = match button_type.to_ascii_lowercase().as_str() {
            "button" => FormControlType::Button,
            "reset" => FormControlType::Reset,
            _ => FormControlType::Submit,
        };

        let value = if element.text_content().is_empty() {
            match control_type {
                FormControlType::Submit => "Submit".to_string(),
                FormControlType::Reset => "Reset".to_string(),
                _ => element.attr("value").unwrap_or("Button").to_string(),
            }
        } else {
            element.text_content()
        };

        Self {
            control_type,
            value,
            placeholder: "",
            disabled: element.has_attr("disabled"),
            readonly: false,
            checked: false,
            selected_index: None,
            options: Vec::new(),
            name: element.attr("name").unwrap_or("").to_string(),
            id: element.attr("id").unwrap_or("").to_string(),
            filename: None,
            label_for: None,
        }
    }

    /// Create a new form box from a label element
    pub fn from_label(element: &Element) -> Self {
        Self {
            control_type: FormControlType::Label,
            value: element.text_content(),
            placeholder: "",
            disabled: false,
            readonly: false,
            checked: false,
            selected_index: None,
            options: Vec::new(),
            name: "",
            id: element.attr("id").unwrap_or("").to_string(),
            filename: None,
            label_for: element.attr("for").map(|s| s.to_string()),
        }
    }

    /// Get the intrinsic width for this form control
    pub fn intrinsic_width(&self) -> f32 {
        match self.control_type {
            FormControlType::Text | FormControlType::Password | FormControlType::Email => 200.0,
            FormControlType::Number => 100.0,
            FormControlType::Date | FormControlType::Time => 120.0,
            FormControlType::File => 250.0,
            FormControlType::Textarea => 300.0,
            FormControlType::Select => 200.0,
            FormControlType::Button | FormControlType::Submit | FormControlType::Reset => {
                // Width based on text content + padding
                (self.value.len() as f32) * 8.0 + 32.0
            }
            FormControlType::Checkbox | FormControlType::Radio => 16.0,
            FormControlType::Label => {
                (self.value.len() as f32) * 7.0
            }
            FormControlType::Hidden => 0.0,
        }
    }

    /// Get the intrinsic height for this form control
    pub fn intrinsic_height(&self) -> f32 {
        match self.control_type {
            FormControlType::Text | FormControlType::Password | FormControlType::Email |
            FormControlType::Number | FormControlType::Date | FormControlType::Time |
            FormControlType::File | FormControlType::Select => 28.0,
            FormControlType::Textarea => 80.0,
            FormControlType::Button | FormControlType::Submit | FormControlType::Reset => 32.0,
            FormControlType::Checkbox | FormControlType::Radio => 16.0,
            FormControlType::Label => 20.0,
            FormControlType::Hidden => 0.0,
        }
    }

    /// Get default background color
    pub fn default_background_color(&self) -> Color {
        match self.control_type {
            FormControlType::Button | FormControlType::Submit | FormControlType::Reset => {
                Color::new(240, 240, 240)
            }
            FormControlType::Checkbox | FormControlType::Radio => {
                Color::WHITE
            }
            _ => Color::WHITE,
        }
    }

    /// Get default border color
    pub fn default_border_color(&self) -> Color {
        if self.disabled {
            Color::new(200, 200, 200)
        } else {
            Color::new(150, 150, 150)
        }
    }

    /// Get default text color
    pub fn default_text_color(&self) -> Color {
        if self.disabled {
            Color::new(128, 128, 128)
        } else {
            Color::BLACK
        }
    }

    /// Get the display text for this control
    pub fn display_text(&self) -> String {
        match self.control_type {
            FormControlType::Password => {
                // Show asterisks for password
                "•".repeat(self.value.len())
            }
            FormControlType::File => {
                // Show filename or "Choose file"
                self.filename.as_ref()
                    .map(|f| format!("📎 {}", f))
                    .unwrap_or_else(|| "Choose file...".to_string())
            }
            FormControlType::Select => {
                // Show selected option
                self.selected_index
                    .and_then(|i| self.options.get(i))
                    .map(|o| o.text.clone())
                    .unwrap_or_else(|| self.value.clone())
            }
            FormControlType::Checkbox => {
                if self.checked { "☑" } else { "☐" }.to_string()
            }
            FormControlType::Radio => {
                if self.checked { "◉" } else { "○" }.to_string()
            }
            _ => self.value.clone(),
        }
    }

    /// Check if this control needs a border
    pub fn needs_border(&self) -> bool {
        matches!(self.control_type,
            FormControlType::Text | FormControlType::Password | FormControlType::Email |
            FormControlType::Number | FormControlType::Date | FormControlType::Time |
            FormControlType::File | FormControlType::Textarea | FormControlType::Select |
            FormControlType::Button | FormControlType::Submit | FormControlType::Reset |
            FormControlType::Checkbox | FormControlType::Radio
        )
    }

    /// Get border width
    pub fn border_width(&self) -> f32 {
        match self.control_type {
            FormControlType::Checkbox | FormControlType::Radio => 1.0,
            FormControlType::Button | FormControlType::Submit | FormControlType::Reset => 1.0,
            _ => 1.0,
        }
    }

    /// Get padding
    pub fn padding(&self) -> EdgeSizes {
        match self.control_type {
            FormControlType::Text | FormControlType::Password | FormControlType::Email |
            FormControlType::Number | FormControlType::Date | FormControlType::Time |
            FormControlType::File | FormControlType::Select => {
                EdgeSizes::symmetric(4.0, 8.0)
            }
            FormControlType::Textarea => {
                EdgeSizes::symmetric(6.0, 8.0)
            }
            FormControlType::Button | FormControlType::Submit | FormControlType::Reset => {
                EdgeSizes::symmetric(6.0, 16.0)
            }
            _ => EdgeSizes::all(0.0),
        }
    }
}

/// Check if an element is a form element
pub fn is_form_element(tag_name: &str) -> bool {
    matches!(tag_name.to_ascii_lowercase().as_str(),
        "input" | "textarea" | "select" | "button" | "label"
    )
}

/// Create a form box from an element if it's a form element
pub fn create_form_box(element: &Element) -> Option<FormBox> {
    match element.tag_name().to_ascii_lowercase().as_str() {
        "input" => Some(FormBox::from_input(element)),
        "textarea" => Some(FormBox::from_textarea(element)),
        "select" => Some(FormBox::from_select(element)),
        "button" => Some(FormBox::from_button(element)),
        "label" => Some(FormBox::from_label(element)),
        _ => None,
    }
}

/// Calculate dimensions for a form box
pub fn calculate_form_dimensions(
    form_box: &FormBox,
    style: &ComputedStyle,
    containing_block_width: f32,
    base_font_size: f32,
) -> Dimensions {
    let intrinsic_width = form_box.intrinsic_width();
    let intrinsic_height = form_box.intrinsic_height();

    // Apply CSS width if specified
    let width = if style.width.is_auto() {
        intrinsic_width
    } else {
        style.width.to_pt_with_container(base_font_size, containing_block_width)
    };

    // Apply CSS height if specified
    let height = if style.height.is_auto() {
        intrinsic_height
    } else {
        style.height.to_pt(base_font_size)
    };

    let padding = form_box.padding();
    let border_width = form_box.border_width();

    Dimensions {
        content: Rect::new(0.0, 0.0, width, height),
        padding,
        border: EdgeSizes::all(border_width),
        margin: EdgeSizes::new(
            style.margin_top.to_pt_with_container(base_font_size, containing_block_width),
            style.margin_right.to_pt_with_container(base_font_size, containing_block_width),
            style.margin_bottom.to_pt_with_container(base_font_size, containing_block_width),
            style.margin_left.to_pt_with_container(base_font_size, containing_block_width),
        ),
    }
}

/// Default form element styles to add to style resolver
pub fn get_form_default_styles() -> Vec<(String, Vec<(&'static str, &'static str)>)> {
    vec![
        ("input".to_string(), vec![
            ("display", "inline-block"),
            ("border", "1px solid #999"),
            ("background", "white"),
            ("padding", "4px 8px"),
            ("font-family", "inherit"),
            ("font-size", "inherit"),
        ]),
        ("textarea".to_string(), vec![
            ("display", "inline-block"),
            ("border", "1px solid #999"),
            ("background", "white"),
            ("padding", "6px 8px"),
            ("font-family", "inherit"),
            ("font-size", "inherit"),
            ("resize", "none"),
        ]),
        ("select".to_string(), vec![
            ("display", "inline-block"),
            ("border", "1px solid #999"),
            ("background", "white"),
            ("padding", "4px 8px"),
            ("font-family", "inherit"),
            ("font-size", "inherit"),
        ]),
        ("button".to_string(), vec![
            ("display", "inline-block"),
            ("border", "1px solid #999"),
            ("background", "#f0f0f0"),
            ("padding", "6px 16px"),
            ("font-family", "inherit"),
            ("font-size", "inherit"),
            ("cursor", "default"),
        ]),
        ("label".to_string(), vec![
            ("display", "inline"),
            ("font-family", "inherit"),
            ("font-size", "inherit"),
        ]),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_form_control_type_from_input() {
        assert_eq!(FormControlType::from_input("text"), FormControlType::Text);
        assert_eq!(FormControlType::from_input("password"), FormControlType::Password);
        assert_eq!(FormControlType::from_input("checkbox"), FormControlType::Checkbox);
        assert_eq!(FormControlType::from_input("hidden"), FormControlType::Hidden);
        assert_eq!(FormControlType::from_input("UNKNOWN"), FormControlType::Text);
    }

    #[test]
    fn test_form_box_from_input() {
        let mut el = Element::new("input", vec![]);
        el.set_attr("type", "text");
        el.set_attr("name", "username");
        el.set_attr("value", "john");
        
        let form_box = FormBox::from_input(&el);
        assert_eq!(form_box.control_type, FormControlType::Text);
        assert_eq!(form_box.name, "username");
        assert_eq!(form_box.value, "john");
    }

    #[test]
    fn test_form_box_checkbox() {
        let mut el = Element::new("input", vec![]);
        el.set_attr("type", "checkbox");
        el.set_attr("checked", "");
        
        let form_box = FormBox::from_input(&el);
        assert_eq!(form_box.control_type, FormControlType::Checkbox);
        assert!(form_box.checked);
    }

    #[test]
    fn test_form_box_password_display() {
        let mut el = Element::new("input", vec![]);
        el.set_attr("type", "password");
        el.set_attr("value", "secret");
        
        let form_box = FormBox::from_input(&el);
        assert_eq!(form_box.display_text(), "••••••");
    }

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
    fn test_hidden_visibility() {
        let hidden = FormControlType::Hidden;
        assert!(!hidden.is_visible());
        
        let text = FormControlType::Text;
        assert!(text.is_visible());
    }
}
