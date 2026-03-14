//! DOM types for HTML5

use std::collections::HashMap;

/// Document quirks mode
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum QuirksMode {
    #[default]
    NoQuirks,
    Quirks,
    LimitedQuirks,
}

/// Element namespace
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum Namespace {
    #[default]
    Html,
    Svg,
    MathMl,
}

impl Namespace {
    pub fn url(&self) -> &'static str {
        match self {
            Namespace::Html => "http://www.w3.org/1999/xhtml",
            Namespace::Svg => "http://www.w3.org/2000/svg",
            Namespace::MathMl => "http://www.w3.org/1998/Math/MathML",
        }
    }
}

/// HTML attribute
#[derive(Debug, Clone, PartialEq)]
pub struct Attribute {
    pub name: String,
    pub value: String,
    pub namespace: Option<Namespace>,
}

impl Attribute {
    pub fn new(name: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            value: value.into(),
            namespace: None,
        }
    }
}

/// Node types
#[derive(Debug, Clone, PartialEq)]
pub enum Node {
    Document(Document),
    DocumentType(DocumentType),
    Element(Element),
    Text(TextNode),
    Comment(Comment),
    ProcessingInstruction { target: String, data: String },
}

impl Node {
    /// Get node type as string
    pub fn node_name(&self) -> String {
        match self {
            Node::Document(_) => "#document".to_string(),
            Node::DocumentType(dt) => dt.name.clone(),
            Node::Element(el) => el.tag_name.clone(),
            Node::Text(_) => "#text".to_string(),
            Node::Comment(_) => "#comment".to_string(),
            Node::ProcessingInstruction { target, .. } => target.clone(),
        }
    }

    /// Check if this is an element node
    pub fn is_element(&self) -> bool {
        matches!(self, Node::Element(_))
    }

    /// Check if this is a text node
    pub fn is_text(&self) -> bool {
        matches!(self, Node::Text(_))
    }

    /// Get element if this is an element node
    pub fn as_element(&self) -> Option<&Element> {
        match self {
            Node::Element(el) => Some(el),
            _ => None,
        }
    }

    /// Get mutable element reference
    pub fn as_element_mut(&mut self) -> Option<&mut Element> {
        match self {
            Node::Element(el) => Some(el),
            _ => None,
        }
    }

    /// Get text content if this is a text node
    pub fn as_text(&self) -> Option<&str> {
        match self {
            Node::Text(t) => Some(&t.data),
            _ => None,
        }
    }
}

/// HTML Document
#[derive(Debug, Clone, PartialEq)]
pub struct Document {
    pub doctype: Option<DocumentType>,
    pub document_element: Option<Element>,
    pub head: Option<Element>,
    pub body: Option<Element>,
    pub quirks_mode: QuirksMode,
    pub title: String,
}

impl Document {
    pub fn new() -> Self {
        Self {
            doctype: None,
            document_element: None,
            head: None,
            body: None,
            quirks_mode: QuirksMode::NoQuirks,
            title: String::new(),
        }
    }

    /// Get root HTML element
    pub fn root_element(&self) -> &Element {
        self.document_element.as_ref().expect("Document has no root element")
    }

    /// Get body element
    pub fn body_element(&self) -> &Element {
        self.body.as_ref().expect("Document has no body")
    }

    /// Get head element
    pub fn head_element(&self) -> Option<&Element> {
        self.head.as_ref()
    }

    /// Get element by ID
    pub fn get_element_by_id(&self, id: &str) -> Option<&Element> {
        self.document_element.as_ref()?.find_by_id(id)
    }

    /// Get all elements with given tag name
    pub fn get_elements_by_tag_name(&self, tag_name: &str) -> Vec<&Element> {
        let mut result = Vec::new();
        if let Some(root) = &self.document_element {
            root.find_by_tag_name(tag_name, &mut result);
        }
        result
    }

    /// Get all elements with given class name
    pub fn get_elements_by_class_name(&self, class_name: &str) -> Vec<&Element> {
        let mut result = Vec::new();
        if let Some(root) = &self.document_element {
            root.find_by_class_name(class_name, &mut result);
        }
        result
    }
}

impl Default for Document {
    fn default() -> Self {
        Self::new()
    }
}

/// Document type declaration
#[derive(Debug, Clone, PartialEq)]
pub struct DocumentType {
    pub name: String,
    pub public_id: Option<String>,
    pub system_id: Option<String>,
}

impl DocumentType {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            public_id: None,
            system_id: None,
        }
    }

    pub fn with_public_id(mut self, public_id: impl Into<String>) -> Self {
        self.public_id = Some(public_id.into());
        self
    }

    pub fn with_system_id(mut self, system_id: impl Into<String>) -> Self {
        self.system_id = Some(system_id.into());
        self
    }
}

/// HTML Element
#[derive(Debug, Clone, PartialEq)]
pub struct Element {
    pub tag_name: String,
    pub attributes: HashMap<String, Attribute>,
    pub children: Vec<Node>,
    pub namespace: Namespace,
    pub parent: Option<Box<Element>>,
}

impl Element {
    pub fn new(tag_name: impl Into<String>, attrs: Vec<Attribute>) -> Self {
        let tag_name = tag_name.into();
        let mut attributes = HashMap::new();
        
        for attr in attrs {
            attributes.insert(attr.name.clone(), attr);
        }

        Self {
            tag_name,
            attributes,
            children: Vec::new(),
            namespace: Namespace::Html,
            parent: None,
        }
    }

    /// Get tag name
    pub fn tag_name(&self) -> &str {
        &self.tag_name
    }

    /// Get lowercase tag name for comparison
    pub fn tag_name_lower(&self) -> String {
        self.tag_name.to_ascii_lowercase()
    }

    /// Get attribute value
    pub fn attr(&self, name: &str) -> Option<&str> {
        self.attributes.get(name).map(|a| a.value.as_str())
    }

    /// Set attribute
    pub fn set_attr(&mut self, name: impl Into<String>, value: impl Into<String>) {
        let name = name.into();
        let value = value.into();
        self.attributes.insert(name.clone(), Attribute::new(name, value));
    }

    /// Remove attribute
    pub fn remove_attr(&mut self, name: &str) -> Option<Attribute> {
        self.attributes.remove(name)
    }

    /// Check if attribute exists
    pub fn has_attr(&self, name: &str) -> bool {
        self.attributes.contains_key(name)
    }

    /// Get element ID
    pub fn id(&self) -> Option<&str> {
        self.attr("id")
    }

    /// Get class list
    pub fn class_list(&self) -> Vec<&str> {
        self.attr("class")
            .map(|c| c.split_whitespace().collect())
            .unwrap_or_default()
    }

    /// Check if has class
    pub fn has_class(&self, class_name: &str) -> bool {
        self.class_list().contains(&class_name)
    }

    /// Get children
    pub fn children(&self) -> &[Node] {
        &self.children
    }

    /// Get mutable children
    pub fn children_mut(&mut self) -> &mut Vec<Node> {
        &mut self.children
    }

    /// Append a child node
    pub fn append_child(&mut self, child: Node) {
        self.children.push(child);
    }

    /// Remove all children
    pub fn clear_children(&mut self) {
        self.children.clear();
    }

    /// Get text content (concatenation of all text descendants)
    pub fn text_content(&self) -> String {
        let mut result = String::new();
        self.collect_text(&mut result);
        result
    }

    fn collect_text(&self, result: &mut String) {
        for child in &self.children {
            match child {
                Node::Text(t) => result.push_str(&t.data),
                Node::Element(e) => e.collect_text(result),
                _ => {}
            }
        }
    }

    /// Get inner HTML
    pub fn inner_html(&self) -> String {
        // Serialize children back to HTML
        let mut result = String::new();
        for child in &self.children {
            result.push_str(&serialize_node(child));
        }
        result
    }

    /// Find element by ID (recursive)
    pub fn find_by_id(&self, id: &str) -> Option<&Element> {
        if self.id() == Some(id) {
            return Some(self);
        }
        
        for child in &self.children {
            if let Node::Element(el) = child {
                if let Some(found) = el.find_by_id(id) {
                    return Some(found);
                }
            }
        }
        
        None
    }

    /// Find elements by tag name
    pub fn find_by_tag_name<'a>(&'a self, tag_name: &str, result: &mut Vec<&'a Element>) {
        if self.tag_name().eq_ignore_ascii_case(tag_name) {
            result.push(self);
        }
        
        for child in &self.children {
            if let Node::Element(el) = child {
                el.find_by_tag_name(tag_name, result);
            }
        }
    }

    /// Find elements by class name
    pub fn find_by_class_name<'a>(&'a self, class_name: &str, result: &mut Vec<&'a Element>) {
        if self.has_class(class_name) {
            result.push(self);
        }
        
        for child in &self.children {
            if let Node::Element(el) = child {
                el.find_by_class_name(class_name, result);
            }
        }
    }

    /// Get first child element
    pub fn first_element_child(&self) -> Option<&Element> {
        self.children.iter().find_map(|c| c.as_element())
    }

    /// Get last child element
    pub fn last_element_child(&self) -> Option<&Element> {
        self.children.iter().rev().find_map(|c| c.as_element())
    }

    /// Get next sibling element
    pub fn next_element_sibling(&self) -> Option<&Element> {
        // This would need parent reference to work properly
        None
    }

    /// Get previous sibling element
    pub fn previous_element_sibling(&self) -> Option<&Element> {
        // This would need parent reference to work properly
        None
    }

    /// Check if element matches a simple selector
    pub fn matches(&self, selector: &str) -> bool {
        // Simple selector matching - full CSS selector engine would be separate
        let parts: Vec<&str> = selector.split_whitespace().collect();
        
        if parts.is_empty() {
            return false;
        }
        
        let simple_selector = parts[0];
        
        // Tag name match
        if simple_selector.eq_ignore_ascii_case(&self.tag_name) {
            return true;
        }
        
        // ID selector
        if simple_selector.starts_with('#') {
            return self.id() == Some(&simple_selector[1..]);
        }
        
        // Class selector
        if simple_selector.starts_with('.') {
            return self.has_class(&simple_selector[1..]);
        }
        
        // Attribute selector [attr]
        if simple_selector.starts_with('[') && simple_selector.ends_with(']') {
            let attr_name = &simple_selector[1..simple_selector.len()-1];
            return self.has_attr(attr_name);
        }
        
        false
    }
}

/// Text node
#[derive(Debug, Clone, PartialEq)]
pub struct TextNode {
    pub data: String,
}

impl TextNode {
    pub fn new(data: impl Into<String>) -> Self {
        Self { data: data.into() }
    }

    pub fn data(&self) -> &str {
        &self.data
    }

    pub fn set_data(&mut self, data: impl Into<String>) {
        self.data = data.into();
    }

    /// Get length of text
    pub fn length(&self) -> usize {
        self.data.len()
    }
}

/// Comment node
#[derive(Debug, Clone, PartialEq)]
pub struct Comment {
    pub data: String,
}

impl Comment {
    pub fn new(data: impl Into<String>) -> Self {
        Self { data: data.into() }
    }

    pub fn data(&self) -> &str {
        &self.data
    }
}

/// Serialize a node to HTML string
fn serialize_node(node: &Node) -> String {
    match node {
        Node::Element(el) => {
            let mut result = format!("<{}", el.tag_name);
            
            for (name, attr) in &el.attributes {
                if attr.value.is_empty() {
                    result.push_str(&format!(" {}", name));
                } else {
                    let escaped = attr.value
                        .replace('&', "&amp;")
                        .replace('"', "&quot;")
                        .replace('<', "&lt;")
                        .replace('>', "&gt;");
                    result.push_str(&format!(" {}=\"{}\"", name, escaped));
                }
            }
            
            if super::is_void_element(&el.tag_name) && el.children.is_empty() {
                result.push_str(" />");
            } else {
                result.push('>');
                for child in &el.children {
                    result.push_str(&serialize_node(child));
                }
                result.push_str(&format!("</{}>", el.tag_name));
            }
            
            result
        }
        Node::Text(t) => {
            t.data
                .replace('&', "&amp;")
                .replace('<', "&lt;")
                .replace('>', "&gt;")
        }
        Node::Comment(c) => format!("<!--{}-->", c.data),
        _ => String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_element_creation() {
        let attrs = vec![
            Attribute::new("id", "test"),
            Attribute::new("class", "foo bar"),
        ];
        let el = Element::new("div", attrs);
        
        assert_eq!(el.tag_name(), "div");
        assert_eq!(el.id(), Some("test"));
        assert!(el.has_class("foo"));
        assert!(el.has_class("bar"));
        assert!(!el.has_class("baz"));
    }

    #[test]
    fn test_element_attributes() {
        let mut el = Element::new("div", vec![]);
        
        el.set_attr("data-test", "value");
        assert_eq!(el.attr("data-test"), Some("value"));
        
        el.remove_attr("data-test");
        assert!(!el.has_attr("data-test"));
    }

    #[test]
    fn test_element_children() {
        let mut el = Element::new("div", vec![]);
        
        el.append_child(Node::Text(TextNode::new("Hello")));
        assert_eq!(el.children().len(), 1);
        
        el.clear_children();
        assert!(el.children().is_empty());
    }

    #[test]
    fn test_text_content() {
        let mut parent = Element::new("div", vec![]);
        parent.append_child(Node::Text(TextNode::new("Hello ")));
        
        let mut child = Element::new("span", vec![]);
        child.append_child(Node::Text(TextNode::new("world")));
        parent.append_child(Node::Element(child));
        
        assert_eq!(parent.text_content(), "Hello world");
    }

    #[test]
    fn test_element_matching() {
        let el = Element::new("div", vec![
            Attribute::new("id", "myid"),
            Attribute::new("class", "myclass"),
        ]);
        
        assert!(el.matches("div"));
        assert!(el.matches("DIV"));
        assert!(el.matches("#myid"));
        assert!(el.matches(".myclass"));
        assert!(el.matches("[id]"));
        assert!(!el.matches("span"));
        assert!(!el.matches("#otherid"));
    }

    #[test]
    fn test_serialize() {
        let mut el = Element::new("div", vec![
            Attribute::new("id", "test"),
        ]);
        el.append_child(Node::Text(TextNode::new("Hello <world>")));
        
        let html = serialize_node(&Node::Element(el));
        assert!(html.contains("<div id=\"test\">"));
        assert!(html.contains("Hello &lt;world&gt;"));
        assert!(html.contains("</div>"));
    }

    #[test]
    fn test_void_element_serialization() {
        let el = Element::new("br", vec![]);
        let html = serialize_node(&Node::Element(el));
        assert_eq!(html, "<br />");
    }
}
