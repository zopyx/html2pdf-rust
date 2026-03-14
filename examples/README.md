# HTML2PDF Examples

This directory contains comprehensive examples demonstrating the features and capabilities of the html2pdf library.

## Overview

These examples showcase:
- Basic HTML to PDF conversion
- PrintCSS / CSS Paged Media features
- URL-based conversion
- Custom page settings and configurations
- Error handling patterns

## Running the Examples

### Basic Example
```bash
cargo run --example basic
```

### PrintCSS Example
```bash
cargo run --example printcss
```

### URL Fetching Example
```bash
cargo run --example from_url
```

### Complete Example (existing)
```bash
cargo run --example complete_example
```

## Example Files

### Rust Examples

#### `basic.rs` - Basic Conversion Example
A simple example demonstrating:
- Converting HTML strings to PDF
- Using `Config` to customize output
- Different paper sizes and orientations
- Custom margins
- Using the `Input` enum
- Error handling patterns

**Output files:**
- `output_basic_simple.pdf` - Simple conversion with defaults
- `output_basic_landscape.pdf` - Letter landscape orientation
- `output_basic_narrow_margins.pdf` - Custom narrow margins
- `output_basic_input_enum.pdf` - Using Input enum

#### `printcss.rs` - PrintCSS / Paged Media Example
Demonstrates CSS Paged Media features:
- `@page` rules for page setup
- Page margins and margin boxes
- Running headers and footers
- Page numbering with `counter(page)`
- Named pages for different layouts
- Page breaks and pagination control
- Mixed portrait/landscape sections

**Output files:**
- `output_printcss_basic.pdf` - Basic @page rules
- `output_printcss_headers_footers.pdf` - Headers and footers
- `output_printcss_named_pages.pdf` - Named pages
- `output_printcss_page_breaks.pdf` - Page break control
- `output_printcss_mixed_orientation.pdf` - Mixed orientations

#### `from_url.rs` - URL Fetching Example
Shows how to work with URL-based content:
- Fetching HTML from URLs
- Handling relative URLs with base_url
- Resolving different URL types
- Error handling for network operations
- Web content optimized configuration

**Note:** URL fetching requires an HTTP client (like reqwest) to be added as a dependency.

**Output files:**
- `output_url_simulated.pdf` - Simulated URL content
- `output_url_relative.pdf` - Relative URL resolution
- `output_url_web_content.pdf` - Web-optimized config

#### `complete_example.rs` - Full Pipeline Example
Existing comprehensive example showing:
- Complete conversion pipeline
- HTML parsing
- CSS parsing
- Layout engine usage
- Direct PDF generation
- Both low-level and high-level APIs

**Output files:**
- `output_complete_example.pdf` - Low-level API output
- `output_high_level_api.pdf` - High-level API output

### HTML Examples

#### `basic.html` - Complete Feature Showcase
A comprehensive HTML document demonstrating all supported features:

**Typography:**
- Headings (h1-h4) with styling
- Paragraphs with justified alignment
- Text formatting (bold, italic, code)
- Lead paragraphs with accent borders

**Lists:**
- Unordered lists with nesting
- Ordered lists with nesting
- Definition lists

**Tables:**
- Styled tables with headers
- Zebra striping
- Table captions
- Multi-column layouts

**Code:**
- Inline code formatting
- Code blocks with background
- Preformatted text

**Components:**
- Blockquotes with citations
- Info boxes (info, warning, success)
- Figures and captions
- Horizontal rules
- Special containers

**PrintCSS:**
- @page rules
- Running headers/footers
- Page numbers
- First page handling
- Page break controls

#### `printcss.html` - Advanced PrintCSS Example
A professional business report showcasing advanced PrintCSS:

**Page Setup:**
- Cover page with full-bleed background
- Different margins for different page types
- Named pages (@page cover, @page toc, etc.)
- Landscape sections for wide tables

**Margin Boxes:**
- @top-left / @top-right for headers
- @bottom-center for page numbers
- @bottom-left for dates
- @bottom-right for confidential notices

**Advanced Features:**
- String-set for running headers
- Chapter-based headers
- Table of contents with leaders
- KPI cards and grids
- Professional typography

#### `invoice.html` - Invoice Template
A professional invoice design:
- Header with company info
- Two-column layout for parties
- Detailed itemized table
- Summary calculations
- Payment information
- Status badges

#### `complex-layout.html` - Dashboard Layout
A complex dashboard-style layout:
- Grid-based KPI cards
- Two-column layouts
- Charts and visualizations
- Data tables
- Progress bars
- Timelines
- Team member listings

## Building All Examples

To verify all examples compile:

```bash
cargo build --examples
```

To run all examples and generate PDFs:

```bash
# Build and run each example
cargo run --example basic
cargo run --example printcss
cargo run --example from_url
cargo run --example complete_example
```

## Example Output

All generated PDFs are saved in the `examples/` directory with the prefix `output_`.

## Configuration Options

The examples demonstrate various `Config` options:

```rust
use html2pdf::{Config, PaperSize, Orientation, Margins};

let config = Config::default()
    .with_paper_size(PaperSize::A4)        // A4, Letter, Legal, etc.
    .with_orientation(Orientation::Portrait) // Portrait or Landscape
    .with_margins(Margins::all(72.0));      // 72pt = 1 inch
```

### Paper Sizes
- `A0` through `A6` - ISO A series
- `Letter` - US Letter (8.5" x 11")
- `Legal` - US Legal (8.5" x 14")
- `Tabloid` - 11" x 17"
- `Custom { width, height }` - Custom dimensions in points

### Margins
```rust
Margins::all(72.0)                          // All sides equal
Margins::new(top, right, bottom, left)     // Individual values
Margins::symmetric(vertical, horizontal)   // Vertical/horizontal pairs
```

## CSS Features Demonstrated

### PrintCSS / Paged Media
```css
@page {
    size: A4;
    margin: 2cm;
    
    @top-center { content: "Header"; }
    @bottom-center { content: counter(page); }
}

@page :first { /* First page styles */ }
@page landscape { size: A4 landscape; }
```

### Page Break Control
```css
page-break-before: always;   /* Force page break */
page-break-after: avoid;     /* Avoid break after */
page-break-inside: avoid;    /* Keep element together */
orphans: 3;                  /* Min lines at top */
widows: 3;                   /* Min lines at bottom */
```

### CSS Features
- Box model (margin, border, padding)
- Colors and backgrounds
- Typography (fonts, sizes, line-height)
- Flexbox and Grid layouts
- Tables with styling
- Lists and nesting

## Tips for Creating PDFs

1. **Use PrintCSS**: Leverage @page rules for professional documents
2. **Page Breaks**: Use `page-break-inside: avoid` to keep related content together
3. **Margins**: Set appropriate margins for your content type
4. **Images**: Include images with proper sizing and alt text
5. **Fonts**: Use web-safe fonts or embed custom fonts
6. **Testing**: Always test with actual content length for pagination

## Additional Resources

- [CSS Paged Media Module](https://www.w3.org/TR/css-page-3/)
- [CSS Generated Content](https://www.w3.org/TR/css-content-3/)
- [PDF/A Standard](https://www.iso.org/standard/51502.html)

## License

These examples are provided under the same license as the html2pdf library (MIT OR Apache-2.0).
