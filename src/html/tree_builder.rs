//! HTML5 Tree Builder
//!
//! Implements the tree construction phase of HTML5 parsing per WHATWG spec

use super::{
    dom::{Document, DocumentType, Element, Node, QuirksMode, TextNode, Comment},
    tokenizer::Token,
    is_void_element, Attribute,
};
use crate::error::{ErrorCollector, WarningCategory};

/// Insertion modes for the tree builder
#[derive(Debug, Clone, Copy, PartialEq)]
#[allow(dead_code)]
enum InsertionMode {
    Initial,
    BeforeHtml,
    BeforeHead,
    InHead,
    InHeadNoscript,
    AfterHead,
    InBody,
    Text,
    InTable,
    InTableText,
    InCaption,
    InColumnGroup,
    InTableBody,
    InRow,
    InCell,
    InSelect,
    InSelectInTable,
    InTemplate,
    AfterBody,
    InFrameset,
    AfterFrameset,
    AfterAfterBody,
    AfterAfterFrameset,
}

/// Tree builder state
pub struct TreeBuilder {
    document: Document,
    insertion_mode: InsertionMode,
    original_insertion_mode: Option<InsertionMode>,
    open_elements: Vec<Element>,
    active_formatting_elements: Vec<FormattingElement>,
    head_element: Option<Element>,
    #[allow(dead_code)]
    pending_table_character_tokens: Vec<char>,
    #[allow(dead_code)]
    foster_parenting: bool,
    scripting_enabled: bool,
    frameset_ok: bool,
    template_insertion_modes: Vec<InsertionMode>,
}

#[derive(Debug, Clone)]
enum FormattingElement {
    Element(Element),
    Marker,
}

impl TreeBuilder {
    pub fn new() -> Self {
        Self {
            document: Document::new(),
            insertion_mode: InsertionMode::Initial,
            original_insertion_mode: None,
            open_elements: Vec::new(),
            active_formatting_elements: Vec::new(),
            head_element: None,
            pending_table_character_tokens: Vec::new(),
            foster_parenting: false,
            scripting_enabled: false,
            frameset_ok: true,
            template_insertion_modes: Vec::new(),
        }
    }

    /// Get the final document
    pub fn document(mut self) -> Document {
        self.ensure_body_exists();
        self.document
    }
    
    /// Ensure body element exists (called before returning document)
    fn ensure_body_exists(&mut self) {
        if self.document.body.is_none() {
            // Create body element
            let body = Element::new("body", Vec::new());
            self.document.body = Some(body.clone());
            
            // If we have a document element but no body, add body to it
            if let Some(ref mut root) = self.document.document_element {
                root.append_child(Node::Element(body));
            }
        }
    }

    /// Get current node (last in open elements)
    fn current_node(&self) -> Option<&Element> {
        self.open_elements.last()
    }

    /// Get adjusted current node (accounting for template)
    #[allow(dead_code)]
    fn adjusted_current_node(&self) -> Option<&Element> {
        if self.insertion_mode == InsertionMode::InTemplate && 
           self.template_insertion_modes.last() == Some(&InsertionMode::InTemplate) {
            // Return template contents
            self.current_node()
        } else {
            self.current_node()
        }
    }

    /// Process a token
    pub fn process_token(&mut self, token: Token) {
        match self.insertion_mode {
            InsertionMode::Initial => self.handle_initial(token),
            InsertionMode::BeforeHtml => self.handle_before_html(token),
            InsertionMode::BeforeHead => self.handle_before_head(token),
            InsertionMode::InHead => self.handle_in_head(token),
            InsertionMode::AfterHead => self.handle_after_head(token),
            InsertionMode::InBody => self.handle_in_body(token),
            InsertionMode::Text => self.handle_text(token),
            _ => {
                // Simplified: default to in-body handling for unimplemented modes
                self.handle_in_body(token);
            }
        }
    }

    /// Process a token with error recovery and collection
    ///
    /// Non-fatal errors are collected in the error_collector.
    /// Fatal errors are returned as Err.
    pub fn process_token_with_recovery(
        &mut self,
        token: Token,
        error_collector: &mut ErrorCollector,
    ) -> Result<(), String> {
        // Try to process the token
        // In recovery mode, we try to continue even on errors
        match &token {
            Token::StartTag { name, .. } => {
                // Check for potentially problematic tags
                if name == "script" || name == "style" {
                    // These are handled specially
                }
            }
            Token::EndTag { name } => {
                // Check for mismatched end tags
                if !self.is_element_in_scope(name) {
                    // Non-fatal: unmatched end tag
                    error_collector.add_warning(
                        format!("Unmatched end tag: </{}>", name),
                        WarningCategory::UnsupportedFeature,
                    );
                }
            }
            _ => {}
        }

        // Process the token normally
        self.process_token(token);
        Ok(())
    }

    /// Handle token in Initial mode
    fn handle_initial(&mut self, token: Token) {
        match token {
            Token::Text(text) if text.chars().all(|c| c.is_ascii_whitespace()) => {
                // Ignore whitespace
            }
            Token::Comment(text) => {
                if let Some(root) = self.document.document_element.as_mut() {
                    root.append_child(Node::Comment(Comment::new(text)));
                }
            }
            Token::Doctype { name, public_identifier, system_identifier, force_quirks } => {
                let mut doctype = DocumentType::new(name.as_deref().unwrap_or(""));
                if let Some(ref public) = public_identifier {
                    doctype = doctype.with_public_id(public.clone());
                }
                if let Some(ref system) = system_identifier {
                    doctype = doctype.with_system_id(system.clone());
                }
                self.document.doctype = Some(doctype);
                
                // Determine quirks mode
                self.document.quirks_mode = if force_quirks {
                    QuirksMode::Quirks
                } else {
                    self.determine_quirks_mode(&name, &public_identifier, &system_identifier)
                };
                
                self.insertion_mode = InsertionMode::BeforeHtml;
            }
            _ => {
                // Missing doctype, quirks mode
                self.document.quirks_mode = QuirksMode::Quirks;
                self.insertion_mode = InsertionMode::BeforeHtml;
                self.process_token(token); // Reprocess
            }
        }
    }

    fn determine_quirks_mode(
        &self,
        name: &Option<String>,
        public_id: &Option<String>,
        _system_id: &Option<String>,
    ) -> QuirksMode {
        // Simplified quirks mode detection
        // Full implementation would check against the quirks mode doctype list
        
        if name.as_deref() != Some("html") {
            return QuirksMode::Quirks;
        }
        
        if let Some(ref pid) = public_id {
            let pid_upper = pid.to_ascii_uppercase();
            if pid_upper.starts_with("-//W3C//DTD HTML 4.0 TRANSITIONAL") ||
               pid_upper.starts_with("-//W3C//DTD HTML 4.01 TRANSITIONAL") ||
               pid_upper.starts_with("-//W3C//DTD XHTML 1.0 TRANSITIONAL") ||
               pid_upper.contains("TRANSITIONAL") {
                return QuirksMode::Quirks;
            }
        }
        
        QuirksMode::NoQuirks
    }

    /// Handle token in BeforeHtml mode
    fn handle_before_html(&mut self, token: Token) {
        match token {
            Token::Doctype { .. } => {
                // Parse error, ignore
            }
            Token::Text(text) if text.chars().all(|c| c.is_ascii_whitespace()) => {
                // Ignore whitespace
            }
            Token::Comment(text) => {
                if let Some(root) = self.document.document_element.as_mut() {
                    root.append_child(Node::Comment(Comment::new(text)));
                }
            }
            Token::StartTag { name, .. } if name == "html" => {
                let element = Element::new("html", Vec::new());
                self.document.document_element = Some(element.clone());
                self.open_elements.push(element);
                self.insertion_mode = InsertionMode::BeforeHead;
            }
            Token::EndTag { ref name } if name != "head" && name != "body" && name != "html" && name != "br" => {
                // Parse error, ignore
            }
            _ => {
                // Create html element implicitly
                let element = Element::new("html", Vec::new());
                self.document.document_element = Some(element.clone());
                self.open_elements.push(element);
                self.insertion_mode = InsertionMode::BeforeHead;
                self.process_token(token); // Reprocess
            }
        }
    }

    /// Handle token in BeforeHead mode
    fn handle_before_head(&mut self, token: Token) {
        match token {
            Token::Text(text) if text.chars().all(|c| c.is_ascii_whitespace()) => {
                // Ignore whitespace
            }
            Token::Comment(text) => {
                self.insert_comment(text);
            }
            Token::Doctype { .. } => {
                // Parse error, ignore
            }
            Token::StartTag { ref name, .. } if name == "html" => {
                // Process using rules for InBody
                self.handle_in_body(token);
            }
            Token::StartTag { ref name, .. } if name == "head" => {
                let element = Element::new("head", Vec::new());
                self.head_element = Some(element.clone());
                self.open_elements.push(element);
                self.insertion_mode = InsertionMode::InHead;
            }
            Token::EndTag { ref name } if name != "head" && name != "body" && name != "html" && name != "br" => {
                // Parse error, ignore
            }
            _ => {
                // Insert head element implicitly
                let element = Element::new("head", Vec::new());
                self.head_element = Some(element.clone());
                self.open_elements.push(element);
                self.insertion_mode = InsertionMode::InHead;
                self.process_token(token); // Reprocess
            }
        }
    }

    /// Handle token in InHead mode
    fn handle_in_head(&mut self, token: Token) {
        match token {
            Token::Text(text) if text.chars().all(|c| c.is_ascii_whitespace()) => {
                self.insert_text(&text);
            }
            Token::Comment(text) => {
                self.insert_comment(text);
            }
            Token::Doctype { .. } => {
                // Parse error, ignore
            }
            Token::StartTag { ref name, .. } if name == "html" => {
                self.handle_in_body(token);
            }
            Token::StartTag { name, .. } if ["base", "basefont", "bgsound", "link", "meta"].contains(&name.as_str()) => {
                self.insert_void_element(&name, Vec::new());
            }
            Token::StartTag { name, .. } if name == "title" => {
                self.parse_raw_text(&name);
            }
            Token::StartTag { name, .. } if name == "noscript" && !self.scripting_enabled => {
                self.insert_element(&name, Vec::new());
                self.insertion_mode = InsertionMode::InHeadNoscript;
            }
            Token::StartTag { name, .. } if name == "script" || name == "style" || name == "noscript" => {
                self.parse_raw_text(&name);
            }
            Token::StartTag { name, .. } if name == "template" => {
                self.insert_element(&name, Vec::new());
                self.active_formatting_elements.push(FormattingElement::Marker);
                self.frameset_ok = false;
                self.insertion_mode = InsertionMode::InTemplate;
                self.template_insertion_modes.push(InsertionMode::InTemplate);
            }
            Token::EndTag { name } if name == "head" => {
                self.open_elements.pop();
                self.insertion_mode = InsertionMode::AfterHead;
            }
            Token::EndTag { ref name } if ["body", "html", "br"].contains(&name.as_str()) => {
                self.open_elements.pop();
                self.insertion_mode = InsertionMode::AfterHead;
                self.process_token(token); // Reprocess
            }
            Token::StartTag { name, .. } if name == "head" => {
                // Parse error, ignore
            }
            Token::EndTag { .. } => {
                // Parse error, ignore
            }
            _ => {
                self.open_elements.pop();
                self.insertion_mode = InsertionMode::AfterHead;
                self.process_token(token); // Reprocess
            }
        }
    }

    /// Handle token in AfterHead mode
    fn handle_after_head(&mut self, token: Token) {
        match token {
            Token::Text(text) if text.chars().all(|c| c.is_ascii_whitespace()) => {
                self.insert_text(&text);
            }
            Token::Comment(text) => {
                self.insert_comment(text);
            }
            Token::Doctype { .. } => {
                // Parse error, ignore
            }
            Token::StartTag { ref name, .. } if name == "html" => {
                self.handle_in_body(token);
            }
            Token::StartTag { name, .. } if name == "body" => {
                let element = Element::new("body", Vec::new());
                self.document.body = Some(element.clone());
                self.open_elements.push(element);
                self.frameset_ok = false;
                self.insertion_mode = InsertionMode::InBody;
            }
            Token::StartTag { name, .. } if name == "frameset" => {
                let element = Element::new("frameset", Vec::new());
                self.document.body = Some(element.clone());
                self.open_elements.push(element);
                self.insertion_mode = InsertionMode::InFrameset;
            }
            Token::StartTag { ref name, .. } if ["base", "basefont", "bgsound", "link", "meta", "noframes", "script", "style", "template", "title"].contains(&name.as_str()) => {
                // Push head back onto stack temporarily
                let head_element = self.head_element.clone();
                if let Some(ref head) = head_element {
                    self.open_elements.push(head.clone());
                }
                self.handle_in_head(token);
                // Remove head from stack
                let len = self.open_elements.len();
                if len > 1 {
                    self.open_elements.retain(|e| e.tag_name != "head" || len == 1);
                }
            }
            Token::EndTag { ref name } if name == "template" => {
                self.handle_in_head(token);
            }
            Token::EndTag { ref name } if ["body", "html", "br"].contains(&name.as_str()) => {
                // Anything else case
                let element = Element::new("body", Vec::new());
                self.document.body = Some(element.clone());
                self.open_elements.push(element);
                self.insertion_mode = InsertionMode::InBody;
                self.process_token(token); // Reprocess
            }
            Token::StartTag { name, .. } if name == "head" => {
                // Parse error, ignore
            }
            _ => {
                // Anything else
                let element = Element::new("body", Vec::new());
                self.document.body = Some(element.clone());
                self.open_elements.push(element);
                self.insertion_mode = InsertionMode::InBody;
                self.process_token(token); // Reprocess
            }
        }
    }

    /// Handle token in InBody mode
    fn handle_in_body(&mut self, token: Token) {
        match token {
            Token::Text(text) => {
                self.insert_text(&text);
            }
            Token::Comment(text) => {
                self.insert_comment(text);
            }
            Token::Doctype { .. } => {
                // Parse error, ignore
            }
            Token::StartTag { name, .. } if name == "html" => {
                // Add attributes to html element
                // Simplified
            }
            Token::StartTag { ref name, .. } if ["base", "basefont", "bgsound", "link", "meta", "noframes", "script", "style", "template", "title"].contains(&name.as_str()) => {
                self.handle_in_head(token);
            }
            Token::StartTag { name, .. } if name == "body" => {
                // Parse error unless second body is an error
                // Simplified: ignore
            }
            Token::StartTag { name, .. } if name == "frameset" => {
                if !self.frameset_ok || self.open_elements.len() != 1 {
                    // Parse error, ignore
                } else {
                    // Replace body with frameset
                    self.open_elements.pop(); // Remove body
                    let element = Element::new("frameset", Vec::new());
                    self.open_elements.push(element);
                    self.insertion_mode = InsertionMode::InFrameset;
                }
            }
            Token::EndTag { ref name } if name == "template" => {
                self.handle_in_head(token);
            }
            Token::StartTag { name, attributes, self_closing } 
                if ["address", "article", "aside", "blockquote", "center", "details", "dialog", "dir", "div", "dl", "fieldset", "figcaption", "figure", "footer", "header", "hgroup", "main", "menu", "nav", "ol", "p", "section", "summary", "ul"].contains(&name.as_str()) => {
                // Close p if in scope
                self.close_pelement_if_in_scope();
                self.insert_element(&name, attributes);
                if self_closing {
                    self.open_elements.pop();
                }
            }
            Token::StartTag { name, attributes, .. } if ["h1", "h2", "h3", "h4", "h5", "h6"].contains(&name.as_str()) => {
                self.close_pelement_if_in_scope();
                // Close any open heading
                if let Some(current) = self.current_node() {
                    if ["h1", "h2", "h3", "h4", "h5", "h6"].contains(&current.tag_name.as_str()) {
                        // Parse error
                        self.open_elements.pop();
                    }
                }
                self.insert_element(&name, attributes);
            }
            Token::StartTag { name, attributes, self_closing } if ["pre", "listing"].contains(&name.as_str()) => {
                self.close_pelement_if_in_scope();
                self.insert_element(&name, attributes);
                // Skip initial newline
                if self_closing {
                    self.open_elements.pop();
                }
            }
            Token::StartTag { name, attributes, .. } if name == "form" => {
                // Simplified
                self.close_pelement_if_in_scope();
                self.insert_element(&name, attributes);
            }
            Token::StartTag { name, attributes, self_closing } if name == "li" => {
                self.frameset_ok = false;
                // Close li elements
                self.close_element_in_scope("li");
                self.close_pelement_if_in_scope();
                self.insert_element(&name, attributes);
                if self_closing {
                    self.open_elements.pop();
                }
            }
            Token::StartTag { name, attributes, self_closing } if ["dd", "dt"].contains(&name.as_str()) => {
                self.frameset_ok = false;
                // Close dd/dt elements
                self.close_element_in_scope("dd");
                self.close_element_in_scope("dt");
                self.close_pelement_if_in_scope();
                self.insert_element(&name, attributes);
                if self_closing {
                    self.open_elements.pop();
                }
            }
            Token::StartTag { name, attributes, self_closing } if ["plaintext"].contains(&name.as_str()) => {
                self.close_pelement_if_in_scope();
                self.insert_element(&name, attributes);
                // Everything following is plain text
                if self_closing {
                    self.open_elements.pop();
                }
            }
            Token::StartTag { name, attributes, self_closing } if ["button"].contains(&name.as_str()) => {
                // Close button if in scope
                if self.is_element_in_scope("button") {
                    self.close_element_in_scope("button");
                }
                self.frameset_ok = false;
                self.insert_element(&name, attributes);
                if self_closing {
                    self.open_elements.pop();
                }
            }
            Token::EndTag { name } if ["address", "article", "aside", "blockquote", "button", "center", "details", "dialog", "dir", "div", "dl", "fieldset", "figcaption", "figure", "footer", "header", "hgroup", "listing", "main", "menu", "nav", "ol", "pre", "section", "summary", "ul"].contains(&name.as_str()) => {
                if self.is_element_in_scope(&name) {
                    self.close_element_in_scope(&name);
                }
            }
            Token::EndTag { name } if ["h1", "h2", "h3", "h4", "h5", "h6"].contains(&name.as_str()) => {
                if self.has_heading_in_scope() {
                    // Close the matching heading
                    for i in (0..self.open_elements.len()).rev() {
                        let tag = self.open_elements[i].tag_name.clone();
                        if ["h1", "h2", "h3", "h4", "h5", "h6"].contains(&tag.as_str()) {
                            self.open_elements.truncate(i);
                            break;
                        }
                    }
                }
            }
            Token::EndTag { name } if name == "p" => {
                self.close_pelement_if_in_scope();
            }
            Token::EndTag { name } if name == "li" => {
                if self.is_element_in_scope("li") {
                    self.close_element_in_scope("li");
                }
            }
            Token::EndTag { name } if ["dd", "dt"].contains(&name.as_str()) => {
                if self.is_element_in_scope(&name) {
                    self.close_element_in_scope(&name);
                }
            }
            Token::StartTag { name, attributes, .. } if name == "a" => {
                // Active formatting reconstruction
                self.reconstruct_active_formatting();
                self.insert_element(&name, attributes);
                // Add to active formatting
                if let Some(current) = self.current_node() {
                    self.active_formatting_elements.push(FormattingElement::Element(current.clone()));
                }
            }
            Token::StartTag { name, attributes, self_closing } if ["b", "big", "code", "em", "font", "i", "s", "small", "strike", "strong", "tt", "u"].contains(&name.as_str()) => {
                self.reconstruct_active_formatting();
                self.insert_element(&name, attributes);
                if let Some(current) = self.current_node() {
                    self.active_formatting_elements.push(FormattingElement::Element(current.clone()));
                }
                if self_closing {
                    self.open_elements.pop();
                }
            }
            Token::StartTag { name, attributes, self_closing } if ["nobr"].contains(&name.as_str()) => {
                self.reconstruct_active_formatting();
                if self.is_element_in_scope("nobr") {
                    // Adoption agency
                    self.adoption_agency_algorithm("nobr");
                    self.reconstruct_active_formatting();
                }
                self.insert_element(&name, attributes);
                if let Some(current) = self.current_node() {
                    self.active_formatting_elements.push(FormattingElement::Element(current.clone()));
                }
                if self_closing {
                    self.open_elements.pop();
                }
            }
            Token::EndTag { name } if ["a", "b", "big", "code", "em", "font", "i", "nobr", "s", "small", "strike", "strong", "tt", "u"].contains(&name.as_str()) => {
                // Adoption agency algorithm
                self.adoption_agency_algorithm(&name);
            }
            Token::StartTag { name, attributes, self_closing } if ["applet", "marquee", "object"].contains(&name.as_str()) => {
                self.reconstruct_active_formatting();
                self.insert_element(&name, attributes);
                self.active_formatting_elements.push(FormattingElement::Marker);
                self.frameset_ok = false;
                if self_closing {
                    self.open_elements.pop();
                }
            }
            Token::EndTag { name } if ["applet", "marquee", "object"].contains(&name.as_str()) => {
                if self.is_element_in_scope(&name) {
                    self.close_element_in_scope(&name);
                    self.clear_active_formatting_to_marker();
                }
            }
            Token::StartTag { name, attributes, self_closing } if ["table"].contains(&name.as_str()) => {
                if self.document.quirks_mode != QuirksMode::Quirks {
                    self.close_pelement_if_in_scope();
                }
                self.insert_element(&name, attributes);
                self.frameset_ok = false;
                self.insertion_mode = InsertionMode::InTable;
                if self_closing {
                    self.open_elements.pop();
                }
            }
            Token::StartTag { name, .. } if ["area", "br", "embed", "img", "keygen", "wbr"].contains(&name.as_str()) => {
                self.reconstruct_active_formatting();
                self.insert_void_element(&name, Vec::new());
                self.frameset_ok = false;
            }
            Token::StartTag { name, attributes, self_closing } if name == "input" => {
                self.reconstruct_active_formatting();
                self.insert_void_element(&name, attributes);
                // Check type attribute
                self.frameset_ok = false;
                if self_closing {
                    // Already handled by insert_void_element
                }
            }
            Token::StartTag { name, attributes, self_closing } if ["param", "source", "track"].contains(&name.as_str()) => {
                self.insert_void_element(&name, attributes);
                if self_closing {
                    // Already handled
                }
            }
            Token::StartTag { name, attributes, self_closing } if name == "hr" => {
                self.close_pelement_if_in_scope();
                self.insert_void_element(&name, attributes);
                self.frameset_ok = false;
                if self_closing {
                    // Already handled
                }
            }
            Token::StartTag { name, attributes, self_closing } if name == "image" => {
                // Parse error, treat as img
                self.process_token(Token::StartTag { 
                    name: "img".to_string(), 
                    attributes, 
                    self_closing 
                });
            }
            Token::StartTag { name, attributes, self_closing } if name == "textarea" => {
                self.insert_element(&name, attributes);
                // Skip initial newline
                self.frameset_ok = false;
                self.parse_raw_text(&name);
                if self_closing {
                    // Will be handled by raw text parsing
                }
            }
            Token::StartTag { name, attributes: _, self_closing } if name == "xmp" => {
                self.close_pelement_if_in_scope();
                self.reconstruct_active_formatting();
                self.frameset_ok = false;
                self.parse_raw_text(&name);
                if self_closing {
                    // Will be handled
                }
            }
            Token::StartTag { name, attributes: _, self_closing } if ["iframe", "noembed", "noframes"].contains(&name.as_str()) => {
                self.frameset_ok = false;
                self.parse_raw_text(&name);
                if self_closing {
                    // Will be handled
                }
            }
            Token::StartTag { name, attributes, self_closing } if name == "select" => {
                self.reconstruct_active_formatting();
                self.insert_element(&name, attributes);
                self.frameset_ok = false;
                // Check current mode
                self.insertion_mode = InsertionMode::InSelect;
                if self_closing {
                    self.open_elements.pop();
                }
            }
            Token::StartTag { name, attributes, self_closing } if ["optgroup", "option"].contains(&name.as_str()) => {
                if self.is_element_in_scope("option") {
                    self.open_elements.pop(); // Close option
                }
                self.reconstruct_active_formatting();
                self.insert_element(&name, attributes);
                if self_closing {
                    self.open_elements.pop();
                }
            }
            Token::StartTag { name, attributes, self_closing } if ["rb", "rtc"].contains(&name.as_str()) => {
                if self.is_element_in_scope("ruby") {
                    self.close_element_in_scope("ruby");
                }
                self.insert_element(&name, attributes);
                if self_closing {
                    self.open_elements.pop();
                }
            }
            Token::StartTag { name, attributes, self_closing } if ["rp", "rt"].contains(&name.as_str()) => {
                if self.is_element_in_scope("ruby") {
                    self.close_element_in_scope("ruby");
                }
                self.insert_element(&name, attributes);
                if self_closing {
                    self.open_elements.pop();
                }
            }
            Token::EndTag { name } if ["body"].contains(&name.as_str()) => {
                if self.is_element_in_scope("body") {
                    self.insertion_mode = InsertionMode::AfterBody;
                }
            }
            Token::EndTag { ref name } if ["html"].contains(&name.as_str()) => {
                if self.is_element_in_scope("body") {
                    self.insertion_mode = InsertionMode::AfterBody;
                    self.process_token(token); // Reprocess
                }
            }
            Token::EndTag { name } if ["address", "article", "aside", "blockquote", "button", "center", "details", "dialog", "dir", "div", "dl", "fieldset", "figcaption", "figure", "footer", "header", "hgroup", "listing", "main", "menu", "nav", "ol", "pre", "section", "summary", "ul", "form", "plaintext", "table"].contains(&name.as_str()) => {
                if self.is_element_in_scope(&name) {
                    self.close_element_in_scope(&name);
                }
            }
            Token::EndTag { name } if ["sarcasm"].contains(&name.as_str()) => {
                // Any other end tag
                self.handle_any_other_end_tag(&name);
            }
            Token::StartTag { name, attributes, self_closing } => {
                // Any other start tag
                self.reconstruct_active_formatting();
                self.insert_element(&name, attributes);
                if self_closing && is_void_element(&name) {
                    self.open_elements.pop();
                }
            }
            Token::EndTag { ref name } => {
                // Any other end tag
                self.handle_any_other_end_tag(name);
            }
            Token::EndOfFile => {
                // Ensure body exists before finishing
                self.ensure_body_exists();
            }
        }
    }

    /// Handle text mode
    fn handle_text(&mut self, token: Token) {
        match token {
            Token::Text(text) => {
                self.insert_text(&text);
            }
            Token::EndTag { name: _ } => {
                self.open_elements.pop();
                self.insertion_mode = self.original_insertion_mode.unwrap_or(InsertionMode::InBody);
            }
            Token::EndOfFile => {
                // Parse error
                self.open_elements.pop();
                self.insertion_mode = self.original_insertion_mode.unwrap_or(InsertionMode::InBody);
            }
            _ => {}
        }
    }

    /// Handle any other end tag
    fn handle_any_other_end_tag(&mut self, name: &str) {
        // Adoption agency or simple close
        let mut found = false;
        for i in (0..self.open_elements.len()).rev() {
            if self.open_elements[i].tag_name == name {
                found = true;
                // Generate implied end tags
                // Close up to this element
                self.open_elements.truncate(i);
                break;
            }
        }
        
        if !found {
            // Adoption agency
            self.adoption_agency_algorithm(name);
        }
    }

    /// Close p element if in button scope
    fn close_pelement_if_in_scope(&mut self) {
        if self.is_element_in_scope("p") {
            self.close_element_in_scope("p");
        }
    }

    /// Check if element is in scope
    fn is_element_in_scope(&self, name: &str) -> bool {
        for el in self.open_elements.iter().rev() {
            if el.tag_name == name {
                return true;
            }
            // Check if it's a scope marker
            if ["applet", "caption", "html", "marquee", "object", "table", "td", "th"].contains(&el.tag_name.as_str()) {
                return false;
            }
        }
        false
    }

    /// Check if heading is in scope
    fn has_heading_in_scope(&self) -> bool {
        for el in self.open_elements.iter().rev() {
            if ["h1", "h2", "h3", "h4", "h5", "h6"].contains(&el.tag_name.as_str()) {
                return true;
            }
            if ["applet", "caption", "html", "marquee", "object", "table", "td", "th"].contains(&el.tag_name.as_str()) {
                return false;
            }
        }
        false
    }

    /// Close element in scope
    fn close_element_in_scope(&mut self, name: &str) {
        for i in (0..self.open_elements.len()).rev() {
            if self.open_elements[i].tag_name == name {
                self.open_elements.truncate(i);
                break;
            }
        }
    }

    /// Insert an element
    fn insert_element(&mut self, name: &str, attributes: Vec<Attribute>) {
        let element = Element::new(name, attributes);
        
        // Append to current node
        if let Some(parent) = self.open_elements.last_mut() {
            parent.append_child(Node::Element(element.clone()));
        }
        
        self.open_elements.push(element);
    }

    /// Insert a void element
    fn insert_void_element(&mut self, name: &str, attributes: Vec<Attribute>) {
        let element = Element::new(name, attributes);
        
        if let Some(parent) = self.open_elements.last_mut() {
            parent.append_child(Node::Element(element));
        }
    }

    /// Insert text
    fn insert_text(&mut self, text: &str) {
        if let Some(parent) = self.open_elements.last_mut() {
            // Check if last child is text node
            if let Some(Node::Text(last)) = parent.children().last() {
                // Append to existing
                let mut new_text = last.data().to_string();
                new_text.push_str(text);
                parent.children_mut().pop();
                parent.append_child(Node::Text(TextNode::new(new_text)));
            } else {
                parent.append_child(Node::Text(TextNode::new(text)));
            }
        }
    }

    /// Insert comment
    fn insert_comment(&mut self, text: String) {
        if let Some(parent) = self.open_elements.last_mut() {
            parent.append_child(Node::Comment(Comment::new(text)));
        }
    }

    /// Parse raw text
    fn parse_raw_text(&mut self, _name: &str) {
        self.original_insertion_mode = Some(self.insertion_mode);
        self.insertion_mode = InsertionMode::Text;
    }

    /// Reconstruct active formatting elements
    fn reconstruct_active_formatting(&mut self) {
        // Simplified
    }

    /// Clear active formatting to marker
    fn clear_active_formatting_to_marker(&mut self) {
        while let Some(el) = self.active_formatting_elements.pop() {
            if matches!(el, FormattingElement::Marker) {
                break;
            }
        }
    }

    /// Adoption agency algorithm (simplified)
    fn adoption_agency_algorithm(&mut self, name: &str) {
        // Full adoption agency is complex - this is a simplified version
        // In production, implement full algorithm from spec
        
        // Find formatting element
        if let Some(idx) = self.active_formatting_elements.iter().rposition(|e| {
            matches!(e, FormattingElement::Element(el) if el.tag_name == name)
        }) {
            // Simplified: just remove from active formatting
            self.active_formatting_elements.remove(idx);
        }
        
        // Close element in open elements
        self.close_element_in_scope(name);
    }
}

impl Default for TreeBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::html::tokenizer::HtmlTokenizer;

    fn parse_html(input: &str) -> Document {
        let mut tokenizer = HtmlTokenizer::new(input);
        let mut builder = TreeBuilder::new();
        
        loop {
            let token = tokenizer.next_token();
            let done = matches!(token, Token::EndOfFile);
            builder.process_token(token);
            if done {
                break;
            }
        }
        
        builder.document()
    }

    #[test]
    fn test_basic_document() {
        let doc = parse_html("<!DOCTYPE html><html><head></head><body><p>Hello</p></body></html>");
        assert!(doc.doctype.is_some());
        assert_eq!(doc.root_element().tag_name(), "html");
        assert_eq!(doc.body_element().unwrap().tag_name(), "body");
    }

    #[test]
    fn test_implicit_elements() {
        let doc = parse_html("<p>Hello</p>");
        assert_eq!(doc.root_element().tag_name(), "html");
        assert_eq!(doc.body_element().unwrap().tag_name(), "body");
    }

    #[test]
    fn test_void_elements() {
        let doc = parse_html("<p>Line 1<br>Line 2<img src='test.jpg'></p>");
        let body = doc.body_element().unwrap();
        assert_eq!(body.children().len(), 1);
        
        if let Some(Node::Element(p)) = body.children().first() {
            assert_eq!(p.children().len(), 4); // text, br, text, img
        }
    }

    #[test]
    fn test_nested_elements() {
        let doc = parse_html("<div><p><strong>Bold</strong></p></div>");
        let body = doc.body_element().unwrap();
        
        if let Some(Node::Element(div)) = body.children().first() {
            if let Some(Node::Element(p)) = div.children().first() {
                if let Some(Node::Element(strong)) = p.children().first() {
                    assert_eq!(strong.tag_name(), "strong");
                } else {
                    panic!("Expected strong element");
                }
            } else {
                panic!("Expected p element");
            }
        } else {
            panic!("Expected div element");
        }
    }

    #[test]
    fn test_unclosed_tags() {
        let doc = parse_html("<p>First<p>Second");
        let body = doc.body_element().unwrap();
        assert_eq!(body.children().len(), 2);
    }

    #[test]
    fn test_comment() {
        let doc = parse_html("<!-- comment --><p>text</p>");
        let body = doc.body_element().unwrap();
        assert_eq!(body.children().len(), 2);
        assert!(matches!(body.children()[0], Node::Comment(_)));
    }

    #[test]
    fn test_script_content() {
        let doc = parse_html("<script>var x = '<div>';</script>");
        let body = doc.body_element().unwrap();
        assert_eq!(body.children().len(), 1);
    }
}
