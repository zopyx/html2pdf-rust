# HTML2PDF Test Suite - Coverage Plan

## Overview

This document outlines the comprehensive test suite for the html2pdf-rs project, designed to achieve >80% code coverage and ensure reliability across all components.

## Test Structure

```
tests/
├── html5lib_tests.rs      # HTML5 parsing compliance tests
├── css_tests.rs           # CSS parsing and processing tests
├── layout_tests.rs        # Layout engine tests
├── integration_tests.rs   # End-to-end integration tests
├── common/
│   └── mod.rs            # Shared test utilities
└── fixtures/              # Test HTML/CSS files
    ├── simple.html
    ├── css_test.html
    ├── printcss_test.html
    └── complex_layout.html

benches/
├── html_parsing.rs        # HTML parser benchmarks
├── css_parsing.rs         # CSS parser benchmarks
├── pdf_generation.rs      # PDF generation benchmarks
└── end_to_end.rs          # Full pipeline benchmarks

scripts/
├── run_tests.sh          # Test runner with options
└── benchmark.sh          # Performance benchmarks

.github/workflows/
└── test.yml              # GitHub Actions CI/CD
```

## Test Coverage Areas

### 1. HTML5 Parsing Tests (`tests/html5lib_tests.rs`)

#### Tokenizer Tests
- Doctype token parsing
- Start/End tag parsing
- Self-closing tags
- Attribute parsing (quoted, unquoted)
- Comments
- Text content
- Character references (entities)
- Raw text elements (script, style)

#### Tree Construction Tests
- Basic document structure
- Auto-insertion of html/head/body
- Void elements handling
- Foster parenting
- Adoption agency algorithm
- Misnested tag recovery
- Unclosed tag handling

#### Compliance Tests
- html5lib test suite integration
- Quirks mode detection
- Namespace handling (HTML, SVG, MathML)
- Template element support
- Form elements

**Estimated Coverage: 25+ test cases**

### 2. CSS Parsing Tests (`tests/css_tests.rs`)

#### Tokenizer Tests
- Identifiers and keywords
- Numbers and dimensions
- Percentages
- Hash values (IDs, colors)
- Strings
- URLs
- At-keywords
- Comments

#### Selector Tests
- Type selectors
- ID selectors
- Class selectors
- Universal selectors
- Attribute selectors
- Pseudo-classes
- Pseudo-elements
- Combinators (descendant, child, sibling)
- Specificity calculation

#### Value Tests
- Identifiers and keywords
- Numbers and lengths
- Percentages
- Colors (hex, rgb, hsl, named)
- Functions (calc, var, etc.)
- Lists

#### At-Rule Tests
- @page rules (PrintCSS)
- @media queries
- @import
- @font-face
- @keyframes
- @supports

#### PrintCSS Tests
- Page margin boxes (@top-left, @bottom-center, etc.)
- Page selectors (:first, :left, :right, :blank)
- Page size definitions
- Running headers/footers

**Estimated Coverage: 40+ test cases**

### 3. Layout Engine Tests (`tests/layout_tests.rs`)

#### Box Model Tests
- Content rectangle calculation
- Padding rectangle calculation
- Border rectangle calculation
- Margin rectangle calculation
- Box sizing (content-box vs border-box)
- Total width/height calculations

#### Box Type Tests
- Block boxes
- Inline boxes
- Inline-block boxes
- Float boxes
- Positioned boxes

#### Float Tests
- Float left/right
- Clear left/right/both
- Float positioning

#### Positioning Tests
- Static positioning
- Relative positioning
- Absolute positioning
- Fixed positioning
- Z-index stacking

#### Flexbox Tests
- Flex direction (row, column)
- Flex grow/shrink
- Justify content
- Align items

#### Grid Tests
- Grid template columns/rows
- Grid gaps
- Grid placement

#### Page Break Tests (PrintCSS)
- Page break before/after
- Page break inside
- Widows and orphans control
- Page size definitions

**Estimated Coverage: 35+ test cases**

### 4. Integration Tests (`tests/integration_tests.rs`)

#### Simple HTML to PDF
- Minimal documents
- Basic typography
- Lists (ordered/unordered)
- Tables
- Links
- Images (including data URIs)

#### Complex Documents
- Multi-page documents
- Nested elements
- Mixed content types
- CSS styling

#### PrintCSS Features
- @page rules
- Page break controls
- Running headers/footers
- Named pages
- Page selectors

#### CSS Features
- Colors (named, hex, rgb, rgba, hsl)
- Fonts (families, sizes, weights, styles)
- Flexbox layouts
- Grid layouts
- Borders and backgrounds
- Shadows

#### Error Handling
- Malformed HTML
- Malformed CSS
- Invalid colors
- Missing resources

#### Unicode & Internationalization
- Multi-language text
- RTL text direction
- Emoji support

#### Performance Tests
- Large documents (1000+ paragraphs)
- Deeply nested structures
- Scaling tests

**Estimated Coverage: 30+ test cases**

## Benchmark Suite

### HTML Parsing Benchmarks
- Small documents (10 paragraphs)
- Medium documents (100 paragraphs)
- Large documents (1000 paragraphs)
- Complex documents
- Entity parsing
- Nested elements

### CSS Parsing Benchmarks
- Small stylesheets (10 rules)
- Medium stylesheets (100 rules)
- Large stylesheets (1000 rules)
- Complex selectors
- At-rules
- PrintCSS rules

### PDF Generation Benchmarks
- Single page
- Multiple pages (1-50)
- Drawing operations
- Text rendering
- Different paper sizes
- Font embedding

### End-to-End Benchmarks
- Simple HTML conversion
- Medium complexity
- Complex documents
- CSS-heavy documents
- Fixture file conversions

## Test Fixtures

### `tests/fixtures/simple.html`
Basic HTML document demonstrating:
- Semantic structure
- Typography
- Lists
- Links
- Basic CSS styling

### `tests/fixtures/css_test.html`
CSS feature showcase:
- Color formats (named, hex, rgb, rgba, hsl)
- Border styles
- Box model demonstrations
- Typography variations
- Box shadows
- Gradients

### `tests/fixtures/printcss_test.html`
PrintCSS features:
- @page rules
- Page margin boxes
- Running headers/footers
- Page break controls
- Named pages
- Table of contents
- Chapter-based structure

### `tests/fixtures/complex_layout.html`
Modern layout patterns:
- Flexbox navigation
- CSS Grid main layout
- Card components
- Stats display
- Progress indicators
- Pricing tables
- Responsive footer

## CI/CD Pipeline (`.github/workflows/test.yml`)

### Jobs

1. **Test Suite**
   - Run on: Ubuntu, Windows, macOS
   - Rust versions: stable, beta
   - Steps:
     - Check formatting
     - Run clippy
     - Build
     - Run tests
     - Run doc tests

2. **Coverage**
   - Generate LCOV report
   - Upload to Codecov

3. **Integration Tests**
   - Run integration test suite
   - Test fixture conversions

4. **HTML5lib Compliance**
   - Download html5lib tests
   - Run tokenizer tests
   - Run tree construction tests

5. **Benchmarks**
   - Run performance benchmarks
   - Upload results

6. **Miri Tests**
   - Check for undefined behavior

7. **Documentation**
   - Build docs
   - Deploy to GitHub Pages

## Running Tests

### Basic Usage

```bash
# Run all tests
cargo test

# Run specific test file
cargo test --test html5lib_tests
cargo test --test css_tests
cargo test --test layout_tests
cargo test --test integration_tests

# Run with all features
cargo test --all-features

# Run benchmarks
cargo bench
```

### Using Test Scripts

```bash
# Run all tests
./scripts/run_tests.sh --all

# Run specific test categories
./scripts/run_tests.sh --unit
./scripts/run_tests.sh --integration
./scripts/run_tests.sh --html5lib

# Run with coverage
./scripts/run_tests.sh --coverage

# Run benchmarks
./scripts/benchmark.sh --all
./scripts/benchmark.sh --e2e
```

## Coverage Goals

| Component | Target Coverage | Current Status |
|-----------|----------------|----------------|
| HTML Parser (Tokenizer) | 90% | 🟡 In Progress |
| HTML Parser (Tree Builder) | 85% | 🟡 In Progress |
| CSS Parser (Tokenizer) | 90% | 🟡 In Progress |
| CSS Parser (Parser) | 85% | 🟡 In Progress |
| Layout Engine | 80% | 🟡 In Progress |
| PDF Generation | 85% | 🟡 In Progress |
| Integration | 75% | 🟡 In Progress |
| **Overall** | **>80%** | 🟡 In Progress |

## Property-Based Testing

The test suite includes property-based tests using proptest for:
- HTML structure invariants
- CSS value parsing round-trips
- PDF output validity
- Layout calculation properties

## Visual Regression Testing

Visual comparison tests are planned for:
- PDF output against expected renderings
- Cross-platform consistency
- Font rendering accuracy

## Known Limitations

1. **Source Code Compilation**: Some source files have compilation errors that need to be resolved before all tests can run.

2. **External Test Data**: HTML5lib test data should be downloaded separately for full compliance testing.

3. **Visual Regression**: Requires additional tooling (e.g., ImageMagick) for PDF image comparison.

4. **Font Testing**: Limited to standard PDF fonts; custom font embedding tests need font files.

## Future Enhancements

1. **Fuzz Testing**: Add cargo-fuzz for automated fuzz testing
2. **Snapshot Testing**: Expand insta snapshot tests for PDF output
3. **Cross-Browser**: Test HTML rendering consistency
4. **Performance Profiling**: Add flamegraph generation
5. **Memory Testing**: Expand Miri tests and add valgrind profiling
6. **Accessibility**: Add tests for PDF/UA compliance

## Summary

This comprehensive test suite provides:

- **130+ unit and integration test cases**
- **Property-based testing** for robustness
- **Performance benchmarks** for all major components
- **CI/CD integration** for automated testing
- **Test fixtures** covering various HTML/CSS patterns
- **Code coverage tracking** with Codecov integration

The test suite is designed to ensure reliability, catch regressions, and maintain code quality as the project evolves.
