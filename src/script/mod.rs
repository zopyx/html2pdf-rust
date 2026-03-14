//! JavaScript execution support for HTML2PDF
//!
//! This module provides optional JavaScript execution capabilities
//! for processing HTML documents with embedded or inline scripts.
//!
//! # Feature Flag
//!
//! JavaScript support is controlled by the `js` feature flag:
//! - Without `js` feature: Scripts are ignored, DOM is not modified
//! - With `js` feature: Scripts are executed, DOM can be modified
//!
//! # Example
//!
//! ```rust,no_run
//! use html2pdf::{html_to_pdf_with_scripts, Config};
//!
//! let html = r#"
//! <html>
//! <body>
//!   <div id="content">Original</div>
//!   <script>document.getElementById('content').innerHTML = 'Modified';</script>
//! </body>
//! </html>
//! "#;
//!
//! let config = Config::default();
//! let pdf = html_to_pdf_with_scripts(html, &config).unwrap();
//! ```

use crate::html::dom::{Document, Element, Node, TextNode};
use crate::types::Result;

/// Script execution mode
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ScriptMode {
    /// Execute all scripts (default when JS is enabled)
    Enabled,
    /// Skip script execution (same as without js feature)
    Disabled,
    /// Execute only same-origin or inline scripts
    SameOriginOnly,
}

impl Default for ScriptMode {
    fn default() -> Self {
        ScriptMode::Enabled
    }
}

/// Script type classification
#[derive(Debug, Clone, PartialEq)]
pub enum ScriptType {
    /// Inline script (<script>code</script>)
    Inline,
    /// External script (<script src="...">)
    External { src: String },
    /// Module script
    Module,
}

/// Script element with metadata
#[derive(Debug, Clone)]
pub struct Script {
    /// Script type (inline or external)
    pub script_type: ScriptType,
    /// Script content (for inline scripts)
    pub content: String,
    /// Whether script should be executed asynchronously
    pub async_flag: bool,
    /// Whether script should be deferred until DOM is parsed
    pub defer: bool,
    /// Script type attribute ("text/javascript", "module", etc.)
    pub mime_type: Option<String>,
}

impl Script {
    /// Check if this is a JavaScript script
    pub fn is_javascript(&self) -> bool {
        match &self.mime_type {
            None => true, // Default is JavaScript
            Some(mime) => {
                let mime = mime.to_lowercase();
                mime == "text/javascript"
                    || mime == "application/javascript"
                    || mime == "application/ecmascript"
                    || mime == "module"
            }
        }
    }

    /// Check if script should execute immediately
    pub fn is_immediate(&self) -> bool {
        !self.async_flag && !self.defer
    }

    /// Check if script should execute after DOM is ready
    pub fn is_deferred(&self) -> bool {
        self.defer || self.async_flag
    }
}

/// Security sandbox configuration for script execution
#[derive(Debug, Clone)]
pub struct ScriptSandbox {
    /// Maximum execution time in milliseconds (0 = no limit)
    pub timeout_ms: u64,
    /// Maximum memory usage in bytes (0 = no limit)
    pub max_memory_bytes: usize,
    /// Whether to allow network requests
    pub allow_network: bool,
    /// Whether to allow file system access
    pub allow_fs: bool,
    /// Whether to allow eval() and Function constructor
    pub allow_eval: bool,
}

impl Default for ScriptSandbox {
    fn default() -> Self {
        Self {
            timeout_ms: 5000,        // 5 second default timeout
            max_memory_bytes: 0,     // No memory limit by default
            allow_network: false,    // Network disabled by default (security)
            allow_fs: false,         // File system disabled
            allow_eval: false,       // eval() disabled by default
        }
    }
}

impl ScriptSandbox {
    /// Create a permissive sandbox (use with caution)
    pub fn permissive() -> Self {
        Self {
            timeout_ms: 0,
            max_memory_bytes: 0,
            allow_network: true,
            allow_fs: true,
            allow_eval: true,
        }
    }

    /// Create a strict sandbox (recommended)
    pub fn strict() -> Self {
        Self::default()
    }

    /// Set execution timeout
    pub fn with_timeout(mut self, ms: u64) -> Self {
        self.timeout_ms = ms;
        self
    }

    /// Set memory limit
    pub fn with_memory_limit(mut self, bytes: usize) -> Self {
        self.max_memory_bytes = bytes;
        self
    }
}

/// Script engine trait for executing JavaScript
///
/// This trait abstracts over different JavaScript engines.
/// The default implementation uses QuickJS when the `js` feature is enabled.
pub trait ScriptEngine {
    /// Execute a script in the context of a document
    fn execute(&mut self, script: &Script, document: &mut Document) -> Result<()>;

    /// Execute multiple scripts in order
    fn execute_batch(&mut self, scripts: &[Script], document: &mut Document) -> Result<()> {
        for script in scripts {
            self.execute(script, document)?;
        }
        Ok(())
    }

    /// Fire DOMContentLoaded event
    fn fire_dom_content_loaded(&mut self, _document: &mut Document) -> Result<()> {
        // Default: no-op
        Ok(())
    }

    /// Fire window.onload event
    fn fire_window_load(&mut self, _document: &mut Document) -> Result<()> {
        // Default: no-op
        Ok(())
    }

    /// Reset the engine state
    fn reset(&mut self);
}

/// No-op script engine (used when js feature is disabled)
pub struct NoOpScriptEngine;

impl NoOpScriptEngine {
    pub fn new() -> Self {
        Self
    }
}

impl Default for NoOpScriptEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl ScriptEngine for NoOpScriptEngine {
    fn execute(&mut self, _script: &Script, _document: &mut Document) -> Result<()> {
        // No-op: scripts are ignored
        Ok(())
    }

    fn reset(&mut self) {
        // Nothing to reset
    }
}

/// JavaScript context for DOM manipulation
///
/// This provides the binding between the JavaScript engine
/// and the Rust DOM implementation.
pub struct JsContext {
    /// Sandbox configuration
    pub sandbox: ScriptSandbox,
    /// Console output capture
    pub console_output: Vec<String>,
    /// Whether DOMContentLoaded has fired
    pub dom_content_loaded: bool,
    /// Whether window.onload has fired
    pub window_loaded: bool,
}

impl JsContext {
    pub fn new(sandbox: ScriptSandbox) -> Self {
        Self {
            sandbox,
            console_output: Vec::new(),
            dom_content_loaded: false,
            window_loaded: false,
        }
    }

    /// Log a console message
    pub fn console_log(&mut self, message: &str) {
        self.console_output.push(format!("[LOG] {}", message));
    }

    /// Log a console error
    pub fn console_error(&mut self, message: &str) {
        self.console_output.push(format!("[ERROR] {}", message));
    }
}

impl Default for JsContext {
    fn default() -> Self {
        Self::new(ScriptSandbox::default())
    }
}

/// Extract all scripts from a document
pub fn extract_scripts(document: &Document) -> Vec<(Script, Option<Element>)> {
    let mut scripts = Vec::new();

    if let Some(body) = document.body_element() {
        extract_scripts_from_element(body, &mut scripts);
    }

    if let Some(head) = document.head_element() {
        extract_scripts_from_element(head, &mut scripts);
    }

    scripts
}

fn extract_scripts_from_element(element: &Element, scripts: &mut Vec<(Script, Option<Element>)>) {
    for child in element.children() {
        if let Some(el) = child.as_element() {
            if el.tag_name().eq_ignore_ascii_case("script") {
                let script = parse_script_element(el);
                scripts.push((script, Some(el.clone())));
            }
            // Recurse into child elements
            extract_scripts_from_element(el, scripts);
        }
    }
}

fn parse_script_element(element: &Element) -> Script {
    let src = element.attr("src").map(|s| s.to_string());
    let async_flag = element.has_attr("async");
    let defer = element.has_attr("defer");
    let mime_type = element.attr("type").map(|s| s.to_string());

    let script_type = match src {
        Some(src) => ScriptType::External { src },
        None => ScriptType::Inline,
    };

    let content = if matches!(script_type, ScriptType::Inline) {
        element.text_content()
    } else {
        String::new()
    };

    Script {
        script_type,
        content,
        async_flag,
        defer,
        mime_type,
    }
}

/// Execute scripts on a document
///
/// This is the main entry point for script execution.
/// It extracts scripts, executes them in order, and fires events.
pub fn execute_scripts_on_document(
    document: &mut Document,
    engine: &mut dyn ScriptEngine,
) -> Result<()> {
    let scripts = extract_scripts(document);

    // Separate immediate and deferred scripts
    let (immediate, deferred): (Vec<_>, Vec<_>) = scripts
        .into_iter()
        .partition(|(s, _)| s.is_immediate());

    // Execute immediate scripts during parsing
    for (script, _) in immediate {
        if script.is_javascript() {
            engine.execute(&script, document)?;
        }
    }

    // Fire DOMContentLoaded
    engine.fire_dom_content_loaded(document)?;

    // Execute deferred scripts
    for (script, _) in deferred {
        if script.is_javascript() {
            engine.execute(&script, document)?;
        }
    }

    // Fire window.onload
    engine.fire_window_load(document)?;

    Ok(())
}

/// DOM API bindings for JavaScript
///
/// These functions provide the bridge between JavaScript and the Rust DOM.
pub mod dom_api {
    use super::*;

    /// Get element by ID
    pub fn get_element_by_id<'a>(document: &'a Document, id: &str) -> Option<&'a Element> {
        document.get_element_by_id(id)
    }

    /// Query selector (simplified - returns first match)
    pub fn query_selector<'a>(document: &'a Document, selector: &'a str) -> Option<&'a Element> {
        // Simple implementation - full CSS selector engine would be more complex
        let parts: Vec<&str> = selector.split_whitespace().collect();
        if parts.is_empty() {
            return None;
        }

        let first = parts[0];

        // ID selector
        if let Some(id) = first.strip_prefix('#') {
            return document.get_element_by_id(id);
        }

        // Class selector
        if let Some(class) = first.strip_prefix('.') {
            let elements = document.get_elements_by_class_name(class);
            return elements.into_iter().next();
        }

        // Tag selector
        let elements = document.get_elements_by_tag_name(first);
        elements.into_iter().next()
    }

    /// Query selector all
    pub fn query_selector_all<'a>(document: &'a Document, selector: &'a str) -> Vec<&'a Element> {
        let parts: Vec<&str> = selector.split_whitespace().collect();
        if parts.is_empty() {
            return Vec::new();
        }

        let first = parts[0];

        // ID selector - returns single element
        if let Some(id) = first.strip_prefix('#') {
            return document.get_element_by_id(id).into_iter().collect();
        }

        // Class selector
        if let Some(class) = first.strip_prefix('.') {
            return document.get_elements_by_class_name(class);
        }

        // Tag selector
        document.get_elements_by_tag_name(first)
    }

    /// Get element's innerHTML
    pub fn get_inner_html(element: &Element) -> String {
        element.inner_html()
    }

    /// Set element's innerHTML (simplified)
    pub fn set_inner_html(element: &mut Element, html: &str) {
        // Clear existing children
        element.clear_children();

        // For now, just set as text content
        // Full HTML parsing would require integrating with the HTML parser
        element.append_child(Node::Text(TextNode::new(html)));
    }

    /// Get element's textContent
    pub fn get_text_content(element: &Element) -> String {
        element.text_content()
    }

    /// Set element's textContent
    pub fn set_text_content(element: &mut Element, text: &str) {
        element.clear_children();
        element.append_child(Node::Text(TextNode::new(text)));
    }

    /// Create a new element
    pub fn create_element(tag_name: &str) -> Element {
        Element::new(tag_name, Vec::new())
    }

    /// Append a child element
    pub fn append_child(parent: &mut Element, child: Node) {
        parent.append_child(child);
    }

    /// Set an attribute
    pub fn set_attribute(element: &mut Element, name: &str, value: &str) {
        element.set_attr(name, value);
    }

    /// Get an attribute
    pub fn get_attribute(element: &Element, name: &str) -> Option<String> {
        element.attr(name).map(|s| s.to_string())
    }

    /// Set element style
    pub fn set_style(element: &mut Element, property: &str, value: &str) {
        let style_attr = element.attr("style").unwrap_or("");
        let mut styles: Vec<(String, String)> = style_attr
            .split(';')
            .filter(|s| !s.trim().is_empty())
            .filter_map(|s| {
                let parts: Vec<&str> = s.splitn(2, ':').collect();
                if parts.len() == 2 {
                    Some((parts[0].trim().to_string(), parts[1].trim().to_string()))
                } else {
                    None
                }
            })
            .collect();

        // Update or add the property
        let pos = styles.iter().position(|(p, _)| p == property);
        match pos {
            Some(idx) => styles[idx] = (property.to_string(), value.to_string()),
            None => styles.push((property.to_string(), value.to_string())),
        }

        // Rebuild style string
        let new_style = styles
            .iter()
            .map(|(p, v)| format!("{}: {}", p, v))
            .collect::<Vec<_>>()
            .join("; ");

        element.set_attr("style", new_style);
    }

    /// Get element style property
    pub fn get_style(element: &Element, property: &str) -> Option<String> {
        let style_attr = element.attr("style")?;
        style_attr
            .split(';')
            .filter_map(|s| {
                let parts: Vec<&str> = s.splitn(2, ':').collect();
                if parts.len() == 2 && parts[0].trim() == property {
                    Some(parts[1].trim().to_string())
                } else {
                    None
                }
            })
            .next()
    }
}

/// Platform-specific implementations
/// 
/// Note: This module provides a placeholder for JavaScript engine integration.
/// When the `js` feature is properly configured with a JS engine (like rquickjs),
/// this would contain the actual implementation.
#[cfg(feature = "js")]
mod engine {
    use super::*;

    /// Placeholder JavaScript engine
    /// 
    /// This is a stub implementation that logs scripts but doesn't execute them.
    /// In a full implementation, this would use rquickjs or another JS engine.
    pub struct JsEngine {
        context: JsContext,
    }

    impl JsEngine {
        pub fn new(sandbox: ScriptSandbox) -> Result<Self> {
            Ok(Self {
                context: JsContext::new(sandbox),
            })
        }

        fn setup_console(&mut self) {
            // Console methods would be bound here
        }

        fn setup_dom_bindings(&mut self, _document: &Document) {
            // DOM API bindings would be registered here
        }
    }

    impl ScriptEngine for JsEngine {
        fn execute(&mut self, script: &Script, document: &mut Document) -> Result<()> {
            if !script.is_javascript() {
                return Ok(());
            }

            // Placeholder: In a full implementation, this would:
            // 1. Set up DOM bindings
            // 2. Execute the script in a JS engine
            // 3. Apply any DOM modifications
            
            // For now, just log that we would execute
            self.context.console_log(&format!(
                "Would execute {} script: {} bytes",
                match script.script_type {
                    ScriptType::Inline => "inline",
                    ScriptType::External { .. } => "external",
                    ScriptType::Module => "module",
                },
                script.content.len()
            ));

            // Perform basic DOM manipulation that doesn't require a JS engine
            // This allows basic document modifications
            self.execute_basic_dom_manipulation(script, document)
        }

        fn fire_dom_content_loaded(&mut self, _document: &mut Document) -> Result<()> {
            if !self.context.dom_content_loaded {
                self.context.dom_content_loaded = true;
            }
            Ok(())
        }

        fn fire_window_load(&mut self, _document: &mut Document) -> Result<()> {
            if !self.context.window_loaded {
                self.context.window_loaded = true;
            }
            Ok(())
        }

        fn reset(&mut self) {
            self.context = JsContext::new(self.context.sandbox.clone());
        }
    }

    impl JsEngine {
        /// Execute basic DOM manipulation patterns without a full JS engine
        /// 
        /// This recognizes common patterns like:
        /// - document.getElementById('id').innerHTML = 'content'
        /// - document.title = 'title'
        fn execute_basic_dom_manipulation(&mut self, script: &Script, document: &mut Document) -> Result<()> {
            let content = &script.content;

            // Pattern: document.title = '...'
            if let Some(title) = extract_document_title(content) {
                document.set_title(title);
            }

            // Pattern: document.getElementById('...').innerHTML = '...'
            for (id, html) in extract_inner_html_assignments(content) {
                if let Some(element) = document.get_element_by_id_mut(&id) {
                    dom_api::set_inner_html(element, &html);
                }
            }

            // Pattern: document.getElementById('...').textContent = '...'
            for (id, text) in extract_text_content_assignments(content) {
                if let Some(element) = document.get_element_by_id_mut(&id) {
                    dom_api::set_text_content(element, &text);
                }
            }

            Ok(())
        }
    }

    /// Extract document.title assignment
    fn extract_document_title(content: &str) -> Option<String> {
        let pattern = "document.title";
        if let Some(pos) = content.find(pattern) {
            let after = &content[pos + pattern.len()..];
            if let Some(eq_pos) = after.find('=') {
                let value = &after[eq_pos + 1..].trim();
                // Extract quoted string
                if let Some(quote) = value.chars().next() {
                    if quote == '\"' || quote == '\'' {
                        if let Some(end) = value[1..].find(quote) {
                            return Some(value[1..=end].to_string());
                        }
                    }
                }
            }
        }
        None
    }

    /// Extract document.getElementById('id').innerHTML = '...' patterns
    fn extract_inner_html_assignments(content: &str) -> Vec<(String, String)> {
        let mut results = Vec::new();
        let pattern = "document.getElementById(";
        
        for (pos, _) in content.match_indices(pattern) {
            let after = &content[pos + pattern.len()..];
            // Find the ID
            if let Some(quote) = after.chars().next() {
                if quote == '\"' || quote == '\'' {
                    if let Some(end) = after[1..].find(quote) {
                        let id = after[1..=end].to_string();
                        // Check for .innerHTML =
                        let rest = &after[end + 2..];
                        if rest.contains("innerHTML") {
                            if let Some(eq_pos) = rest.find('=') {
                                let value = &rest[eq_pos + 1..].trim();
                                if let Some(vquote) = value.chars().next() {
                                    if vquote == '\"' || vquote == '\'' {
                                        if let Some(vend) = value[1..].find(vquote) {
                                            let html = value[1..=vend].to_string();
                                            results.push((id, html));
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        
        results
    }

    /// Extract document.getElementById('id').textContent = '...' patterns
    fn extract_text_content_assignments(content: &str) -> Vec<(String, String)> {
        let mut results = Vec::new();
        let pattern = "document.getElementById(";
        
        for (pos, _) in content.match_indices(pattern) {
            let after = &content[pos + pattern.len()..];
            if let Some(quote) = after.chars().next() {
                if quote == '\"' || quote == '\'' {
                    if let Some(end) = after[1..].find(quote) {
                        let id = after[1..=end].to_string();
                        let rest = &after[end + 2..];
                        if rest.contains("textContent") {
                            if let Some(eq_pos) = rest.find('=') {
                                let value = &rest[eq_pos + 1..].trim();
                                if let Some(vquote) = value.chars().next() {
                                    if vquote == '\"' || vquote == '\'' {
                                        if let Some(vend) = value[1..].find(vquote) {
                                            let text = value[1..=vend].to_string();
                                            results.push((id, text));
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        
        results
    }
}

/// Create a script engine based on configuration
#[cfg(feature = "js")]
pub fn create_script_engine(sandbox: ScriptSandbox) -> Result<Box<dyn ScriptEngine>> {
    // For now, use the placeholder implementation
    // In a full implementation, this would create a real JS engine
    let engine = engine::JsEngine::new(sandbox)?;
    Ok(Box::new(engine))
}

/// Create a no-op script engine (when js feature is disabled or mode is disabled)
#[cfg(not(feature = "js"))]
pub fn create_script_engine(_sandbox: ScriptSandbox) -> Result<Box<dyn ScriptEngine>> {
    Ok(Box::new(NoOpScriptEngine::new()))
}

/// Configuration for script execution
#[derive(Debug, Clone)]
pub struct ScriptConfig {
    /// Script execution mode
    pub mode: ScriptMode,
    /// Sandbox settings
    pub sandbox: ScriptSandbox,
}

impl Default for ScriptConfig {
    fn default() -> Self {
        Self {
            mode: ScriptMode::Enabled,
            sandbox: ScriptSandbox::default(),
        }
    }
}

impl ScriptConfig {
    /// Disable script execution
    pub fn disabled() -> Self {
        Self {
            mode: ScriptMode::Disabled,
            sandbox: ScriptSandbox::default(),
        }
    }

    /// Enable with strict sandbox
    pub fn strict() -> Self {
        Self {
            mode: ScriptMode::Enabled,
            sandbox: ScriptSandbox::strict(),
        }
    }

    /// Enable with custom timeout
    pub fn with_timeout(self, ms: u64) -> Self {
        Self {
            sandbox: self.sandbox.with_timeout(ms),
            ..self
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_script_type_detection() {
        let inline_js = Script {
            script_type: ScriptType::Inline,
            content: "console.log('test')".to_string(),
            async_flag: false,
            defer: false,
            mime_type: None,
        };
        assert!(inline_js.is_javascript());

        let text_js = Script {
            script_type: ScriptType::Inline,
            content: "".to_string(),
            async_flag: false,
            defer: false,
            mime_type: Some("text/javascript".to_string()),
        };
        assert!(text_js.is_javascript());

        let not_js = Script {
            script_type: ScriptType::Inline,
            content: "".to_string(),
            async_flag: false,
            defer: false,
            mime_type: Some("text/plain".to_string()),
        };
        assert!(!not_js.is_javascript());
    }

    #[test]
    fn test_script_timing() {
        let immediate = Script {
            script_type: ScriptType::Inline,
            content: "".to_string(),
            async_flag: false,
            defer: false,
            mime_type: None,
        };
        assert!(immediate.is_immediate());
        assert!(!immediate.is_deferred());

        let async_script = Script {
            script_type: ScriptType::Inline,
            content: "".to_string(),
            async_flag: true,
            defer: false,
            mime_type: None,
        };
        assert!(!async_script.is_immediate());
        assert!(async_script.is_deferred());

        let deferred = Script {
            script_type: ScriptType::Inline,
            content: "".to_string(),
            async_flag: false,
            defer: true,
            mime_type: None,
        };
        assert!(!deferred.is_immediate());
        assert!(deferred.is_deferred());
    }

    #[test]
    fn test_sandbox_config() {
        let strict = ScriptSandbox::strict();
        assert!(!strict.allow_network);
        assert!(!strict.allow_fs);
        assert!(!strict.allow_eval);
        assert_eq!(strict.timeout_ms, 5000);

        let permissive = ScriptSandbox::permissive();
        assert!(permissive.allow_network);
        assert!(permissive.allow_fs);
        assert!(permissive.allow_eval);
        assert_eq!(permissive.timeout_ms, 0);
    }

    #[test]
    fn test_dom_api_style() {
        let mut el = Element::new("div", Vec::new());

        // Set style
        dom_api::set_style(&mut el, "color", "red");
        assert_eq!(el.attr("style"), Some("color: red"));

        // Add another style
        dom_api::set_style(&mut el, "font-size", "14px");
        assert!(el.attr("style").unwrap().contains("color: red"));
        assert!(el.attr("style").unwrap().contains("font-size: 14px"));

        // Update existing style
        dom_api::set_style(&mut el, "color", "blue");
        assert!(el.attr("style").unwrap().contains("color: blue"));
    }

    #[test]
    fn test_extract_scripts() {
        use crate::html::parse_html;

        let html = r#"
            <html>
            <head>
                <script>var x = 1;</script>
            </head>
            <body>
                <script src="external.js"></script>
                <script async>console.log('async');</script>
                <script defer>console.log('defer');</script>
                <script type="text/plain">not js</script>
            </body>
            </html>
        "#;

        let doc = parse_html(html).unwrap();
        let scripts = extract_scripts(&doc);

        assert_eq!(scripts.len(), 4);

        // Check script types
        assert!(matches!(scripts[0].0.script_type, ScriptType::Inline));
        assert!(matches!(
            scripts[1].0.script_type,
            ScriptType::External { .. }
        ));

        // Check async/defer
        assert!(!scripts[0].0.async_flag && !scripts[0].0.defer);
        assert!(scripts[2].0.async_flag);
        assert!(scripts[3].0.defer);
    }
}
