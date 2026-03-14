# Contributing to HTML2PDF

Thank you for your interest in contributing to html2pdf-rs! This document provides guidelines and instructions for contributing to the project.

## Table of Contents

1. [Development Setup](#development-setup)
2. [Project Structure](#project-structure)
3. [Coding Standards](#coding-standards)
4. [Testing Guidelines](#testing-guidelines)
5. [Pull Request Process](#pull-request-process)
6. [Release Process](#release-process)

## Development Setup

### Prerequisites

- **Rust**: Version 1.75 or higher
- **Git**: For version control
- **Cargo**: Comes with Rust

### Clone and Build

```bash
# Clone the repository
git clone https://github.com/yourusername/html2pdf-rs.git
cd html2pdf-rs

# Build in debug mode (faster compilation)
cargo build

# Build in release mode (optimized)
cargo build --release

# Run tests
cargo test

# Run with verbose output
cargo run -- -v input.html -o output.pdf
```

### Development Tools

Install recommended tools:

```bash
# Rust formatter (rustfmt)
rustup component add rustfmt

# Rust linter (clippy)
rustup component add clippy

# Cargo extensions
cargo install cargo-edit        # For cargo add/rm/upgrade
cargo install cargo-watch       # For auto-rebuild on changes
cargo install cargo-tarpaulin   # For code coverage
cargo install cargo-machete     # For finding unused dependencies
```

### IDE Setup

#### VS Code

Recommended extensions:
- rust-analyzer
- Even Better TOML
- CodeLLDB (for debugging)

#### IntelliJ / RustRover

The Rust plugin provides excellent support for:
- Auto-completion
- Refactoring
- Debugging

## Project Structure

```
html2pdf-rs/
├── src/
│   ├── main.rs          # CLI entry point
│   ├── lib.rs           # Library exports
│   ├── cli.rs           # CLI argument parsing
│   ├── types.rs         # Core types (Point, Size, Color, etc.)
│   ├── html/            # HTML5 parser
│   │   ├── mod.rs       # Module exports and parsing functions
│   │   ├── tokenizer.rs # HTML tokenizer
│   │   ├── tree_builder.rs # DOM tree construction
│   │   └── dom.rs       # DOM types (Document, Element, Node)
│   ├── css/             # CSS3 parser
│   │   ├── mod.rs       # Module exports
│   │   ├── tokenizer.rs # CSS tokenizer
│   │   ├── parser.rs    # CSS parser
│   │   ├── selectors.rs # CSS selectors
│   │   ├── at_rules.rs  # CSS at-rules (@page, etc.)
│   │   └── values.rs    # CSS values
│   ├── layout/          # Layout engine
│   │   ├── mod.rs       # Layout engine and context
│   │   ├── box_model.rs # Box model
│   │   ├── flow.rs      # Normal flow layout
│   │   ├── style.rs     # Style computation
│   │   └── text.rs      # Text layout
│   └── pdf/             # PDF generation
│       ├── mod.rs       # Page content builder
│       ├── writer.rs    # PDF document writer
│       ├── object.rs    # PDF objects
│       ├── stream.rs    # PDF streams
│       ├── font.rs      # Font handling
│       └── image.rs     # Image embedding
├── tests/               # Integration tests
├── benches/             # Benchmarks (future)
├── examples/            # Example HTML files
├── Cargo.toml
└── README.md
```

## Coding Standards

### Rust Style Guidelines

We follow the [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/) and [Rust Style Guide](https://doc.rust-lang.org/style-guide/).

#### Formatting

Run `rustfmt` before committing:

```bash
cargo fmt
```

Configuration is in `.rustfmt.toml` (if present) or uses defaults.

#### Linting

Address all clippy warnings:

```bash
cargo clippy --all-targets --all-features
```

Allow exceptions only with explicit justification:

```rust
#[allow(clippy::too_many_arguments)]
fn complex_function(...)
```

### Code Organization

#### Module Structure

```rust
//! Module-level documentation (required)
//!
//! Description of what this module does.

// Private modules
mod private_module;

// Public re-exports
pub use private_module::{PublicType, public_function};

// Public API
pub struct PublicStruct {
    pub public_field: Type,
    private_field: Type,
}

impl PublicStruct {
    /// Constructor
    pub fn new() -> Self { ... }
    
    /// Public method with documentation
    pub fn public_method(&self) -> Result<Type> { ... }
    
    // Private method
    fn private_helper(&self) { ... }
}
```

#### Documentation

All public items must have documentation:

```rust
/// Brief description (one line)
///
/// Longer description with details.
/// Can span multiple paragraphs.
///
/// # Examples
///
/// ```
/// let result = my_function(42);
/// assert_eq!(result, 42);
/// ```
///
/// # Errors
///
/// Returns an error if...
pub fn my_function(input: i32) -> Result<i32> {
    // Implementation
}
```

Documentation sections:
- **Description**: What the item does
- **Examples**: Usage examples (compile-checked)
- **Errors**: Error conditions and types
- **Panics**: When the function might panic
- **Safety**: For unsafe functions

#### Naming Conventions

| Item | Convention | Example |
|------|------------|---------|
| Modules | snake_case | `box_model`, `html_parser` |
| Types | PascalCase | `LayoutBox`, `PdfWriter` |
| Traits | PascalCase | `Renderer`, `Formatter` |
| Functions | snake_case | `parse_html()`, `add_page()` |
| Variables | snake_case | `layout_tree`, `page_count` |
| Constants | SCREAMING_SNAKE_CASE | `MAX_DEPTH`, `DEFAULT_MARGIN` |
| Enums | PascalCase | `BoxType`, `PaperSize` |
| Enum variants | PascalCase | `Block`, `Inline`, `A4` |
| Lifetimes | snake_case, prefixed with ' | `'a`, `'doc` |
| Generic types | PascalCase, single letter | `T`, `K`, `V` |

### Error Handling

Use `Result` for recoverable errors:

```rust
use crate::types::{Result, PdfError};

pub fn parse(input: &str) -> Result<Document> {
    if input.is_empty() {
        return Err(PdfError::Parse("Empty input".to_string()));
    }
    // ...
    Ok(document)
}
```

Use `?` for error propagation:

```rust
pub fn process(input: &str) -> Result<Output> {
    let doc = parse(input)?;
    let styled = apply_styles(doc)?;
    let layout = compute_layout(styled)?;
    Ok(layout)
}
```

### Unsafe Code

Avoid unsafe code. If absolutely necessary:

```rust
/// # Safety
/// 
/// Caller must ensure:
/// 1. Pointer is valid and aligned
/// 2. Memory is not aliased
unsafe fn dangerous_operation(ptr: *const u8) { ... }
```

### Testing

#### Unit Tests

Place in the same file, in a `#[cfg(test)]` module:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_feature() {
        let result = function_under_test();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_error_case() {
        let result = function_under_test("invalid");
        assert!(result.is_err());
    }
}
```

#### Documentation Tests

Examples in doc comments are tested:

```rust
/// Parses HTML from a string.
///
/// # Examples
///
/// ```
/// use html2pdf::html::parse_html;
///
/// let doc = parse_html("<h1>Hello</h1>").unwrap();
/// ```
pub fn parse_html(input: &str) -> Result<Document> { ... }
```

#### Property-Based Tests

Use `proptest` for fuzzing:

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn doesnt_crash(s in "\\PC*") {
        let _ = parse_html(&s);
    }
}
```

## Testing Guidelines

### Test Organization

```
tests/
├── integration_tests.rs    # End-to-end tests
├── html_tests.rs           # HTML parser tests
├── css_tests.rs            # CSS parser tests
└── fixtures/               # Test data
    ├── html/
    └── css/
```

### Writing Good Tests

#### Test Naming

```rust
// Use descriptive names
#[test]
fn parses_void_elements_correctly() { }

#[test]
fn rejects_malformed_html() { }

#[test]
fn layout_respects_margins() { }
```

#### Test Structure (AAA)

```rust
#[test]
fn calculates_width_correctly() {
    // Arrange
    let box_model = LayoutBox::new(BoxType::Block, None);
    let container_width = 500.0;
    
    // Act
    let width = calculate_width(&box_model, container_width);
    
    // Assert
    assert_eq!(width, 500.0);
}
```

#### Edge Cases

Test boundary conditions:

```rust
#[test]
fn handles_empty_input() { }

#[test]
fn handles_maximum_nesting() { }

#[test]
fn handles_very_long_lines() { }

#[test]
fn handles_unicode_characters() { }
```

### Test Coverage

Aim for high coverage, but prioritize meaningful tests over coverage percentage.

Run coverage:

```bash
cargo tarpaulin --out Html
```

### Snapshot Testing

Use `insta` for output comparison:

```rust
use insta::assert_yaml_snapshot;

#[test]
fn test_layout_output() {
    let layout = compute_layout(input);
    assert_yaml_snapshot!(layout);
}
```

Update snapshots:

```bash
cargo insta review
```

## Pull Request Process

### Before You Start

1. **Check existing issues**: Look for related issues or PRs
2. **Create an issue**: For significant changes, discuss first
3. **Fork the repository**: Create your own fork

### Making Changes

1. **Create a branch**: 
   ```bash
   git checkout -b feature/your-feature-name
   # or
   git checkout -b fix/issue-description
   ```

2. **Make focused commits**:
   - One logical change per commit
   - Clear commit messages
   - Reference issues: `Fixes #123`

3. **Commit message format**:
   ```
   Short (50 chars or less) summary
   
   More detailed explanatory text, if necessary. Wrap it to about 72
   characters. The blank line separating the summary from the body is
   critical.
   
   - Bullet points are okay
   - Reference issues: Fixes #123
   ```

### Code Quality Checklist

Before submitting:

- [ ] `cargo fmt` has been run
- [ ] `cargo clippy` produces no warnings
- [ ] `cargo test` passes all tests
- [ ] New tests added for new functionality
- [ ] Documentation updated (code comments and md files)
- [ ] CHANGELOG.md updated (for user-facing changes)

### Submitting the PR

1. **Push to your fork**:
   ```bash
   git push origin feature/your-feature-name
   ```

2. **Create a Pull Request**:
   - Use a clear title
   - Describe what changed and why
   - Reference related issues
   - Include screenshots for UI changes

3. **PR Description Template**:
   ```markdown
   ## Description
   Brief description of changes
   
   ## Motivation
   Why this change was needed
   
   ## Changes
   - List of specific changes
   - Breaking changes noted
   
   ## Testing
   How the changes were tested
   
   ## Checklist
   - [ ] Tests pass
   - [ ] Documentation updated
   - [ ] CHANGELOG.md updated
   ```

### Review Process

1. **Automated checks** must pass (CI)
2. **Code review** by maintainers
3. **Address feedback** with additional commits
4. **Squash commits** if requested
5. **Merge** by maintainer

### Types of Changes

#### Bug Fixes

- Include regression test
- Reference issue number
- Explain the root cause

#### Features

- Document new API
- Add examples
- Update relevant documentation

#### Refactoring

- Ensure no behavior changes
- Maintain or improve test coverage
- Explain benefits

#### Documentation

- Proofread for clarity
- Check code examples compile
- Update table of contents

## Release Process

### Versioning

We follow [Semantic Versioning](https://semver.org/):

- **MAJOR**: Incompatible API changes
- **MINOR**: Backward-compatible functionality
- **PATCH**: Backward-compatible bug fixes

### Release Checklist

1. **Update version** in `Cargo.toml`
2. **Update CHANGELOG.md** with release date
3. **Create git tag**:
   ```bash
   git tag -a v0.1.0 -m "Release version 0.1.0"
   git push origin v0.1.0
   ```
4. **Publish to crates.io** (maintainers only):
   ```bash
   cargo publish
   ```

## Getting Help

- **Discord**: [Invite link]
- **Discussions**: GitHub Discussions tab
- **Issues**: For bugs and feature requests

## Code of Conduct

### Our Standards

- Be respectful and inclusive
- Welcome newcomers
- Focus on constructive feedback
- Accept responsibility and apologize when needed

### Unacceptable Behavior

- Harassment or discrimination
- Trolling or insulting comments
- Personal or political attacks
- Publishing others' private information

### Enforcement

Violations may result in temporary or permanent ban from the project.

---

Thank you for contributing to html2pdf-rs!
