//! Comprehensive Error Handling for HTML2PDF
//!
//! This module provides structured error types with context, chaining,
//! and pretty formatting for the html2pdf library.

use std::fmt;
use std::path::PathBuf;

/// Position in source code (line, column)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct SourcePosition {
    pub line: usize,
    pub column: usize,
}

impl SourcePosition {
    pub fn new(line: usize, column: usize) -> Self {
        Self { line, column }
    }

    pub fn unknown() -> Self {
        Self { line: 0, column: 0 }
    }

    pub fn is_unknown(&self) -> bool {
        self.line == 0 && self.column == 0
    }
}

impl fmt::Display for SourcePosition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_unknown() {
            write!(f, "unknown position")
        } else {
            write!(f, "line {}, column {}", self.line, self.column)
        }
    }
}

/// Context information for errors
#[derive(Debug, Clone, Default)]
pub struct ErrorContext {
    /// Source file path
    pub file: Option<PathBuf>,
    /// Position in source
    pub position: SourcePosition,
    /// Snippet of source code around the error
    pub snippet: Option<String>,
    /// Additional context message
    pub context: Option<String>,
}

impl ErrorContext {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_file(mut self, file: impl Into<PathBuf>) -> Self {
        self.file = Some(file.into());
        self
    }

    pub fn with_position(mut self, line: usize, column: usize) -> Self {
        self.position = SourcePosition::new(line, column);
        self
    }

    pub fn with_snippet(mut self, snippet: impl Into<String>) -> Self {
        self.snippet = Some(snippet.into());
        self
    }

    pub fn with_context(mut self, context: impl Into<String>) -> Self {
        self.context = Some(context.into());
        self
    }
}

/// HTML parse error details
#[derive(Debug, Clone)]
pub struct ParseErrorDetails {
    pub message: String,
    pub position: SourcePosition,
    pub expected: Vec<String>,
    pub found: Option<String>,
}

impl ParseErrorDetails {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            position: SourcePosition::unknown(),
            expected: Vec::new(),
            found: None,
        }
    }

    pub fn at(mut self, line: usize, column: usize) -> Self {
        self.position = SourcePosition::new(line, column);
        self
    }

    pub fn expected(mut self, what: impl Into<String>) -> Self {
        self.expected.push(what.into());
        self
    }

    pub fn found(mut self, what: impl Into<String>) -> Self {
        self.found = Some(what.into());
        self
    }
}

/// Main error type for HTML2PDF operations
#[derive(Debug, thiserror::Error)]
pub enum Html2PdfError {
    /// HTML parsing error with position information
    #[error("HTML parse error at {position}: {message}")]
    HtmlParseError {
        message: String,
        position: SourcePosition,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// CSS parsing error with position information
    #[error("CSS parse error at {position}: {message}")]
    CssParseError {
        message: String,
        position: SourcePosition,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// I/O error (file operations)
    #[error("I/O error{file}: {message}")]
    IoError {
        message: String,
        file: Option<PathBuf>,
        #[source]
        source: Option<std::io::Error>,
    },

    /// Network error (URL fetching)
    #[error("Network error for {url}: {message}")]
    NetworkError {
        url: String,
        message: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// Layout computation error
    #[error("Layout error: {message}")]
    LayoutError {
        message: String,
        element: Option<String>,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// PDF generation/rendering error
    #[error("Render error: {message}")]
    RenderError {
        message: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// Font loading error
    #[error("Font error for '{family}': {message}")]
    FontError {
        family: String,
        message: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// Image loading/processing error
    #[error("Image error for '{path}': {message}")]
    ImageError {
        path: String,
        message: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// Input validation error
    #[error("Validation error: {message}")]
    ValidationError {
        message: String,
        field: Option<String>,
        suggestion: Option<String>,
    },

    /// Multiple errors collected during processing
    #[error("Multiple errors occurred ({count}):
{errors}", count = errors.len(), errors = errors.iter().map(|e| format!("  - {}", e)).collect::<Vec<_>>().join("\n"))]
    MultipleErrors {
        errors: Vec<Html2PdfError>,
    },

    /// Warning (non-fatal, but worth noting)
    #[error("Warning: {message}")]
    Warning {
        message: String,
        category: WarningCategory,
    },
}

/// Warning categories for non-fatal issues
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WarningCategory {
    UnsupportedFeature,
    Deprecated,
    Performance,
    Accessibility,
    Security,
}

impl fmt::Display for WarningCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WarningCategory::UnsupportedFeature => write!(f, "unsupported feature"),
            WarningCategory::Deprecated => write!(f, "deprecated"),
            WarningCategory::Performance => write!(f, "performance"),
            WarningCategory::Accessibility => write!(f, "accessibility"),
            WarningCategory::Security => write!(f, "security"),
        }
    }
}

/// Result type alias for HTML2PDF operations
pub type Html2PdfResult<T> = std::result::Result<T, Html2PdfError>;

/// Error with context for enhanced error reporting
#[derive(Debug)]
pub struct ContextualError {
    pub error: Html2PdfError,
    pub context: ErrorContext,
}

impl ContextualError {
    pub fn new(error: Html2PdfError) -> Self {
        Self {
            error,
            context: ErrorContext::default(),
        }
    }

    pub fn with_context(mut self, context: ErrorContext) -> Self {
        self.context = context;
        self
    }

    /// Format the error with full context for display
    pub fn format_pretty(&self) -> String {
        let mut output = String::new();

        // Error header
        output.push_str(&format!("\n❌ Error: {}\n", self.error));

        // Context information
        if let Some(ref file) = self.context.file {
            output.push_str(&format!("   File: {}\n", file.display()));
        }

        if !self.context.position.is_unknown() {
            output.push_str(&format!("   Position: {}\n", self.context.position));
        }

        if let Some(ref context) = self.context.context {
            output.push_str(&format!("   Context: {}\n", context));
        }

        // Source snippet
        if let Some(ref snippet) = self.context.snippet {
            output.push_str("\n   Source:\n");
            for (i, line) in snippet.lines().enumerate() {
                let line_num = self.context.position.line.saturating_sub(1) + i;
                output.push_str(&format!("      {:>4} | {}\n", line_num, line));
            }
            // Add pointer to error position
            if !self.context.position.is_unknown() && self.context.position.column > 0 {
                let spaces = " ".repeat(self.context.position.column.saturating_sub(1) + 10);
                output.push_str(&format!("{}^\n", spaces));
            }
        }

        // Suggestions for common errors
        if let Some(suggestion) = self.suggest_fix() {
            output.push_str(&format!("\n💡 Suggestion: {}\n", suggestion));
        }

        output
    }

    /// Generate a fix suggestion based on the error type
    fn suggest_fix(&self) -> Option<String> {
        match &self.error {
            Html2PdfError::HtmlParseError { message, .. } => {
                if message.contains("unclosed tag") {
                    Some("Check for missing closing tags (e.g., </div>, </p>)".to_string())
                } else if message.contains("unexpected token") {
                    Some("Check for syntax errors in your HTML".to_string())
                } else {
                    None
                }
            }
            Html2PdfError::CssParseError { message, .. } => {
                if message.contains("unexpected token") {
                    Some("Check CSS syntax - ensure properties end with semicolons".to_string())
                } else if message.contains("invalid property") {
                    Some("Verify the CSS property name is correct".to_string())
                } else {
                    None
                }
            }
            Html2PdfError::IoError { file, .. } => {
                file.as_ref().map(|f| format!("Check that the file exists and is readable: {}", f.display()))
            }
            Html2PdfError::NetworkError { url, .. } => {
                Some(format!("Verify the URL is correct and accessible: {}", url))
            }
            Html2PdfError::FontError { family, .. } => {
                Some(format!("Consider using a web-safe font or providing a @font-face rule for '{}'", family))
            }
            Html2PdfError::ImageError { path, .. } => {
                Some(format!("Ensure the image file exists and is a supported format (PNG, JPEG, GIF, BMP): {}", path))
            }
            Html2PdfError::ValidationError { suggestion, .. } => suggestion.clone(),
            _ => None,
        }
    }
}

impl fmt::Display for ContextualError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.format_pretty())
    }
}

impl std::error::Error for ContextualError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.error.source()
    }
}

/// Error collector for non-fatal error handling
#[derive(Debug, Default)]
pub struct ErrorCollector {
    errors: Vec<ContextualError>,
    warnings: Vec<(String, WarningCategory)>,
}

impl ErrorCollector {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add an error to the collection
    pub fn add_error(&mut self, error: impl Into<Html2PdfError>) {
        self.errors.push(ContextualError::new(error.into()));
    }

    /// Add a contextual error
    pub fn add_contextual_error(&mut self, error: ContextualError) {
        self.errors.push(error);
    }

    /// Add a warning
    pub fn add_warning(&mut self, message: impl Into<String>, category: WarningCategory) {
        self.warnings.push((message.into(), category));
    }

    /// Add a warning for an unsupported feature
    pub fn add_unsupported_warning(&mut self, feature: impl Into<String>) {
        self.warnings.push((
            format!("Unsupported feature: {}", feature.into()),
            WarningCategory::UnsupportedFeature,
        ));
    }

    /// Check if any errors were collected
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    /// Check if any warnings were collected
    pub fn has_warnings(&self) -> bool {
        !self.warnings.is_empty()
    }

    /// Get the number of errors
    pub fn error_count(&self) -> usize {
        self.errors.len()
    }

    /// Get the number of warnings
    pub fn warning_count(&self) -> usize {
        self.warnings.len()
    }

    /// Get all errors
    pub fn errors(&self) -> &[ContextualError] {
        &self.errors
    }

    /// Get all warnings
    pub fn warnings(&self) -> &[(String, WarningCategory)] {
        &self.warnings
    }

    /// Convert to a single error if there are any errors
    pub fn to_error(&self) -> Option<Html2PdfError> {
        if self.errors.is_empty() {
            None
        } else if self.errors.len() == 1 {
            Some(self.errors[0].error.clone())
        } else {
            Some(Html2PdfError::MultipleErrors {
                errors: self.errors.iter().map(|e| e.error.clone()).collect(),
            })
        }
    }

    /// Print all warnings
    pub fn print_warnings(&self) {
        for (message, category) in &self.warnings {
            eprintln!("⚠️  Warning ({}): {}", category, message);
        }
    }

    /// Print all errors
    pub fn print_errors(&self) {
        for error in &self.errors {
            eprintln!("{}", error.format_pretty());
        }
    }
}

/// Extension trait for adding context to errors
pub trait ResultExt<T> {
    /// Add file context to an error
    fn with_file(self, file: impl Into<PathBuf>) -> Self;

    /// Add position context to an error
    fn with_position(self, line: usize, column: usize) -> Self;

    /// Add a context message to an error
    fn with_context_msg(self, context: impl Into<String>) -> Self;

    /// Convert to a contextual error
    fn into_contextual(self) -> Result<T, ContextualError>;
}

impl<T> ResultExt<T> for Html2PdfResult<T> {
    fn with_file(self, file: impl Into<PathBuf>) -> Self {
        self
    }

    fn with_position(self, _line: usize, _column: usize) -> Self {
        self
    }

    fn with_context_msg(self, _context: impl Into<String>) -> Self {
        self
    }

    fn into_contextual(self) -> Result<T, ContextualError> {
        self.map_err(|e| ContextualError::new(e))
    }
}

/// Builder for constructing errors with context
pub struct ErrorBuilder {
    error: Option<Html2PdfError>,
    context: ErrorContext,
}

impl ErrorBuilder {
    pub fn new() -> Self {
        Self {
            error: None,
            context: ErrorContext::default(),
        }
    }

    pub fn error(mut self, error: Html2PdfError) -> Self {
        self.error = Some(error);
        self
    }

    pub fn file(mut self, file: impl Into<PathBuf>) -> Self {
        self.context.file = Some(file.into());
        self
    }

    pub fn position(mut self, line: usize, column: usize) -> Self {
        self.context.position = SourcePosition::new(line, column);
        self
    }

    pub fn snippet(mut self, snippet: impl Into<String>) -> Self {
        self.context.snippet = Some(snippet.into());
        self
    }

    pub fn context(mut self, context: impl Into<String>) -> Self {
        self.context.context = Some(context.into());
        self
    }

    pub fn build(self) -> ContextualError {
        ContextualError {
            error: self.error.expect("Error must be set"),
            context: self.context,
        }
    }
}

/// Helper functions for creating common error types
pub mod errors {
    use super::*;

    /// Create an HTML parse error
    pub fn html_parse(message: impl Into<String>) -> Html2PdfError {
        Html2PdfError::HtmlParseError {
            message: message.into(),
            position: SourcePosition::unknown(),
            source: None,
        }
    }

    /// Create an HTML parse error at a specific position
    pub fn html_parse_at(
        message: impl Into<String>,
        line: usize,
        column: usize,
    ) -> Html2PdfError {
        Html2PdfError::HtmlParseError {
            message: message.into(),
            position: SourcePosition::new(line, column),
            source: None,
        }
    }

    /// Create a CSS parse error
    pub fn css_parse(message: impl Into<String>) -> Html2PdfError {
        Html2PdfError::CssParseError {
            message: message.into(),
            position: SourcePosition::unknown(),
            source: None,
        }
    }

    /// Create a CSS parse error at a specific position
    pub fn css_parse_at(
        message: impl Into<String>,
        line: usize,
        column: usize,
    ) -> Html2PdfError {
        Html2PdfError::CssParseError {
            message: message.into(),
            position: SourcePosition::new(line, column),
            source: None,
        }
    }

    /// Create an I/O error
    pub fn io(message: impl Into<String>) -> Html2PdfError {
        Html2PdfError::IoError {
            message: message.into(),
            file: None,
            source: None,
        }
    }

    /// Create an I/O error with file path
    pub fn io_file(message: impl Into<String>, file: impl Into<PathBuf>) -> Html2PdfError {
        Html2PdfError::IoError {
            message: message.into(),
            file: Some(file.into()),
            source: None,
        }
    }

    /// Create a network error
    pub fn network(url: impl Into<String>, message: impl Into<String>) -> Html2PdfError {
        Html2PdfError::NetworkError {
            url: url.into(),
            message: message.into(),
            source: None,
        }
    }

    /// Create a layout error
    pub fn layout(message: impl Into<String>) -> Html2PdfError {
        Html2PdfError::LayoutError {
            message: message.into(),
            element: None,
            source: None,
        }
    }

    /// Create a layout error for a specific element
    pub fn layout_element(
        message: impl Into<String>,
        element: impl Into<String>,
    ) -> Html2PdfError {
        Html2PdfError::LayoutError {
            message: message.into(),
            element: Some(element.into()),
            source: None,
        }
    }

    /// Create a render error
    pub fn render(message: impl Into<String>) -> Html2PdfError {
        Html2PdfError::RenderError {
            message: message.into(),
            source: None,
        }
    }

    /// Create a font error
    pub fn font(family: impl Into<String>, message: impl Into<String>) -> Html2PdfError {
        Html2PdfError::FontError {
            family: family.into(),
            message: message.into(),
            source: None,
        }
    }

    /// Create an image error
    pub fn image(path: impl Into<String>, message: impl Into<String>) -> Html2PdfError {
        Html2PdfError::ImageError {
            path: path.into(),
            message: message.into(),
            source: None,
        }
    }

    /// Create a validation error
    pub fn validation(message: impl Into<String>) -> Html2PdfError {
        Html2PdfError::ValidationError {
            message: message.into(),
            field: None,
            suggestion: None,
        }
    }

    /// Create a validation error with field and suggestion
    pub fn validation_detailed(
        message: impl Into<String>,
        field: impl Into<String>,
        suggestion: impl Into<String>,
    ) -> Html2PdfError {
        Html2PdfError::ValidationError {
            message: message.into(),
            field: Some(field.into()),
            suggestion: Some(suggestion.into()),
        }
    }

    /// Create a warning
    pub fn warning(message: impl Into<String>, category: WarningCategory) -> Html2PdfError {
        Html2PdfError::Warning {
            message: message.into(),
            category,
        }
    }
}

/// Exit codes for CLI error handling
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum ExitCode {
    Success = 0,
    GeneralError = 1,
    ParseError = 2,
    IoError = 3,
    NetworkError = 4,
    LayoutError = 5,
    RenderError = 6,
    FontError = 7,
    ImageError = 8,
    ValidationError = 9,
    ConfigError = 10,
}

impl ExitCode {
    /// Get the exit code from an error
    pub fn from_error(error: &Html2PdfError) -> Self {
        match error {
            Html2PdfError::HtmlParseError { .. } | Html2PdfError::CssParseError { .. } => {
                ExitCode::ParseError
            }
            Html2PdfError::IoError { .. } => ExitCode::IoError,
            Html2PdfError::NetworkError { .. } => ExitCode::NetworkError,
            Html2PdfError::LayoutError { .. } => ExitCode::LayoutError,
            Html2PdfError::RenderError { .. } => ExitCode::RenderError,
            Html2PdfError::FontError { .. } => ExitCode::FontError,
            Html2PdfError::ImageError { .. } => ExitCode::ImageError,
            Html2PdfError::ValidationError { .. } => ExitCode::ValidationError,
            Html2PdfError::MultipleErrors { errors } => {
                // Use the first error's exit code
                errors.first().map(Self::from_error).unwrap_or(ExitCode::GeneralError)
            }
            _ => ExitCode::GeneralError,
        }
    }

    /// Get the numeric value
    pub fn as_i32(self) -> i32 {
        self as i32
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::errors::*;

    #[test]
    fn test_source_position() {
        let pos = SourcePosition::new(10, 5);
        assert_eq!(pos.line, 10);
        assert_eq!(pos.column, 5);
        assert!(!pos.is_unknown());

        let unknown = SourcePosition::unknown();
        assert!(unknown.is_unknown());
    }

    #[test]
    fn test_error_creation() {
        let err = html_parse("unexpected token");
        assert!(matches!(err, Html2PdfError::HtmlParseError { .. }));

        let err = html_parse_at("unclosed tag", 5, 10);
        match err {
            Html2PdfError::HtmlParseError { position, .. } => {
                assert_eq!(position.line, 5);
                assert_eq!(position.column, 10);
            }
            _ => panic!("Expected HtmlParseError"),
        }
    }

    #[test]
    fn test_error_collector() {
        let mut collector = ErrorCollector::new();
        
        collector.add_warning("test warning", WarningCategory::UnsupportedFeature);
        assert!(collector.has_warnings());
        assert_eq!(collector.warning_count(), 1);

        collector.add_error(html_parse("test error"));
        assert!(collector.has_errors());
        assert_eq!(collector.error_count(), 1);
    }

    #[test]
    fn test_contextual_error_formatting() {
        let error = html_parse_at("unexpected token", 10, 5);
        let contextual = ContextualError::new(error)
            .with_context(ErrorContext::new()
                .with_file("test.html")
                .with_position(10, 5)
                .with_context("parsing body element"));

        let formatted = contextual.format_pretty();
        assert!(formatted.contains("Error:"));
        assert!(formatted.contains("test.html"));
        assert!(formatted.contains("line 10"));
    }

    #[test]
    fn test_exit_codes() {
        let io_err = io("file not found");
        assert_eq!(ExitCode::from_error(&io_err), ExitCode::IoError);

        let parse_err = html_parse("syntax error");
        assert_eq!(ExitCode::from_error(&parse_err), ExitCode::ParseError);

        let layout_err = layout("overflow");
        assert_eq!(ExitCode::from_error(&layout_err), ExitCode::LayoutError);
    }
}
