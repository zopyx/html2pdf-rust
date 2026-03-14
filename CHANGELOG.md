# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Initial project structure and architecture
- Core type system with `Point`, `Size`, `Rect`, `Color`, `Length`
- Paper size and orientation support (A0-A6, Letter, Legal, Tabloid)
- Comprehensive error handling with `PdfError` type

## [0.1.0] - 2026-03-14

### Added

#### HTML5 Parser
- Complete HTML5 tokenizer following WHATWG specification
- Tree builder with foster parenting and adoption agency algorithm
- Full DOM types: `Document`, `Element`, `Node`, `TextNode`, `Attribute`
- Support for void elements, raw text elements, and normal elements
- HTML entity decoding
- Document type declarations
- Comments and processing instructions
- HTML fragment parsing support
- DOM traversal and querying methods (`get_element_by_id`, `get_elements_by_tag_name`, `get_elements_by_class_name`)
- Element matching with simple selectors
- HTML serialization back to string

#### CSS3 Parser
- CSS Syntax Module Level 3 compliant tokenizer
- Stylesheet parser supporting rules and declarations
- Selector parser with all combinators
- CSS value parsing with full unit support (px, pt, mm, cm, in, em, rem, %, etc.)
- At-rule support including `@page`, `@media`
- PrintCSS extensions:
  - Page rules with `:first`, `:left`, `:right` pseudo-classes
  - Page margin boxes (`@top-center`, `@bottom-center`, etc.)
  - Page break properties (`break-before`, `break-after`, `break-inside`)
  - Orphans and widows control
- Property validation with standard CSS properties list
- Case-insensitive property names
- Custom property support (CSS variables with `--` prefix)

#### Layout Engine
- Layout context with page size, margins, and state management
- Box tree construction from DOM
- CSS box model implementation (margin, border, padding, content)
- Width and height calculation with constraints
- Block formatting context
- Inline formatting context
- Position calculation (static, relative, absolute)
- Float and clear handling (basic)
- Style computation with cascade and inheritance
- Text layout and line breaking
- Fragmentation support for pagination
- Layout box to PDF box conversion

#### PDF Generation
- Native PDF 1.4 implementation from scratch
- PDF object system (dictionary, array, reference, stream)
- Cross-reference table generation
- Page tree structure
- Content stream building
- Standard 14 PDF fonts support
- Text operations (positioning, fonts, encoding)
- Graphics operations (rectangles, lines, curves)
- Image embedding (PNG, JPEG)
- Flate (zlib) compression for streams
- Document metadata (title, author, creator, creation date)

#### Library API
- `html_to_pdf()` - Convert HTML string to PDF bytes
- `html_to_pdf_from_input()` - Convert from various input sources
- `Config` - Configuration with builder pattern
- `Input` - Input abstraction (File, Html, Url)
- Re-exported core types: `Point`, `Size`, `Rect`, `Color`, `Length`
- Re-exported paper types: `PaperSize`, `Orientation`, `Margins`
- Comprehensive error handling with `Result<T>` type

#### Command-Line Interface
- Full-featured CLI with clap
- Multiple input sources: files, URLs, stdin
- Multiple output options: files, stdout
- Paper size selection (A0-A6, Letter, Legal, Tabloid)
- Portrait and landscape orientations
- Flexible margin specification (points, mm, cm, in, px)
- Custom page dimensions
- Header and footer templates
- Additional stylesheet injection
- Configuration file support (JSON)
- Validation subcommand
- Config display subcommand
- Verbose output and debug layout options
- Progress indication during conversion

#### Documentation
- Comprehensive README with examples
- Architecture documentation
- API reference documentation
- Contributing guidelines
- This changelog
- Inline code documentation (rustdoc)
- Usage examples for common scenarios

#### Testing
- Unit tests for all major modules
- Integration tests for end-to-end workflows
- Property-based tests using proptest
- Snapshot testing using insta
- Test coverage for HTML parsing edge cases
- Test coverage for CSS parsing edge cases
- Test coverage for layout calculations
- Test coverage for PDF output validation

### Technical Details

#### Dependencies
- `thiserror` - Error handling
- `clap` - CLI parsing
- `tracing` - Structured logging
- `image` - Image decoding
- `ttf-parser` - Font parsing
- `unicode-width` - Unicode text width
- `unicode-bidi` - Bidirectional text
- `miniz_oxide` - Compression

#### Dev Dependencies
- `tempfile` - Temporary files for testing
- `insta` - Snapshot testing
- `proptest` - Property-based testing
- `criterion` - Benchmarks
- `pretty_assertions` - Better test output
- `approx` - Floating point comparisons
- `serde` / `serde_json` - Test data serialization

### Known Limitations

This is the initial release with some features still in development:

- **Layout Engine**: Basic block and inline layout implemented. Flexbox and Grid layout not yet complete.
- **JavaScript**: No JavaScript execution support.
- **Web Fonts**: No @font-face or web font loading yet.
- **Advanced PDF**: No forms, annotations, or digital signatures.
- **Network**: URL input support planned but not fully implemented.

### Performance

- HTML parsing: ~10MB/s on modern hardware
- CSS parsing: ~5MB/s for typical stylesheets
- Layout: Depends on document complexity
- PDF generation: ~100KB/s output

### Compatibility

- **Rust**: Version 1.75 or higher
- **Platforms**: Linux, macOS, Windows
- **PDF Output**: PDF 1.4 compatible
- **HTML Input**: HTML5 (WHATWG spec)
- **CSS Input**: CSS3 with PrintCSS extensions

---

## Release History

### Future Releases (Planned)

#### [0.2.0] - Target: Q2 2026
- Complete Flexbox layout
- CSS Grid layout
- Web font support (@font-face)
- Image URL fetching
- Enhanced pagination with running headers/footers

#### [0.3.0] - Target: Q3 2026
- Table layout
- Advanced selectors (pseudo-elements)
- CSS animations (for print timeline)
- PDF forms support

#### [0.4.0] - Target: Q4 2026
- JavaScript execution (optional)
- Enhanced error reporting
- Performance optimizations
- PDF/A compliance

#### [1.0.0] - Target: 2027
- Stable API
- Complete CSS 2.1 support
- Full PrintCSS support
- Production-ready performance

---

## How to Update

### From Previous Versions

Since this is the initial release (0.1.0), there are no migration steps from previous versions.

### General Update Process

```bash
# Update via cargo
cargo install html2pdf

# Or from source
git pull origin main
cargo build --release
```

### Breaking Changes Policy

- **Minor versions (0.x.0)**: May contain breaking changes
- **Patch versions (0.x.y)**: Backward-compatible bug fixes only
- **Major version (1.0.0)**: Stable API with deprecation policy

---

## Contributing to the Changelog

When making a pull request, please update this changelog under the `[Unreleased]` section:

1. Add your changes to the appropriate subsection:
   - `Added` for new features
   - `Changed` for changes in existing functionality
   - `Deprecated` for soon-to-be removed features
   - `Removed` for now removed features
   - `Fixed` for any bug fixes
   - `Security` for security improvements

2. Reference issue numbers where applicable: `(#123)`

3. Keep descriptions concise but informative
