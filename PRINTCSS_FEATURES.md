# PrintCSS Features Added to html2pdf-rs

This document summarizes the enhanced PrintCSS (CSS Paged Media) support added to the html2pdf-rs library.

## 1. @page Rules Support

### Page Size (Named Sizes)
- **Location**: `src/pdf/print_css.rs` - `PageSize` struct
- **Features**:
  - Named paper sizes: A4, A3, A5, Letter, Legal, Tabloid
  - Custom dimensions: `width` and `height` in points
  - Orientation: Portrait and Landscape
  - Parsing from CSS: `size: A4`, `size: letter landscape`, `size: 210mm 297mm`

### Page Selectors
- **Location**: `src/css/at_rules.rs` - `PageSelector` enum
- **Features**:
  - `:first` - First page
  - `:left` - Left pages (verso)
  - `:right` - Right pages (recto)
  - `:blank` - Blank pages
  - Named pages: `@page chapter { ... }`

### Page Margins
- **Location**: `src/pdf/print_css.rs` - `Margins` in `PageContext`
- **Features**:
  - Individual margins: `margin-top`, `margin-right`, `margin-bottom`, `margin-left`
  - Shorthand: `margin: 1in` or `margin: 1in 0.75in`
  - Corner boxes for margin areas

### Page Master System
- **Location**: `src/pdf/print_css.rs` - `PageMaster` struct
- **Features**:
  - Multiple page masters with different selectors
  - Automatic application based on page context
  - Named page support for chapter differentiation

## 2. Margin Boxes

### Margin Box Types
- **Location**: `src/css/at_rules.rs` - `MarginBoxType` enum
- **Implemented Boxes**:
  - Top: `@top-left-corner`, `@top-left`, `@top-center`, `@top-right`, `@top-right-corner`
  - Bottom: `@bottom-left-corner`, `@bottom-left`, `@bottom-center`, `@bottom-right`, `@bottom-right-corner`
  - Left: `@left-top`, `@left-middle`, `@left-bottom`
  - Right: `@right-top`, `@right-middle`, `@right-bottom`

### Margin Box Content
- **Location**: `src/pdf/print_css.rs` - `MarginBoxContent` struct
- **Features**:
  - Text content with font styling
  - Color support
  - Text alignment: left, center, right, justify
  - Vertical alignment: top, middle, bottom

### Content Functions
- **Location**: `src/pdf/print_css.rs` - `MarginContentPart` enum
- **Implemented Functions**:
  - `string(name)` - Reference to running strings
  - `counter(page)` - Current page number
  - `counter(pages)` - Total pages
  - `leader(char)` - Filler characters for TOC entries
  - `target-counter(url, counter)` - Cross-reference counters (structure ready)

## 3. Page Breaks

### Legacy Properties
- **Location**: `src/layout/style.rs` - `PageBreak` enum
- **Properties**:
  - `page-break-before: auto | always | avoid | left | right`
  - `page-break-after: auto | always | avoid | left | right`
  - `page-break-inside: auto | avoid`

### Modern Break Properties (CSS Fragmentation Module Level 4)
- **Location**: `src/pdf/print_css.rs` - `BreakType` and `BreakInside` enums
- **Properties**:
  - `break-before: auto | always | avoid | page | left | right | recto | verso | column | region`
  - `break-after: auto | always | avoid | page | left | right | recto | verso | column | region`
  - `break-inside: auto | avoid | avoid-page | avoid-column | avoid-region`

### Widows and Orphans Control
- **Location**: `src/layout/style.rs` - ComputedStyle fields
- **Properties**:
  - `orphans: <integer>` (default: 2)
  - `widows: <integer>` (default: 2)

## 4. Running Headers/Footers

### String-Set Property
- **Location**: `src/layout/style.rs` - `string_set` field in ComputedStyle
- **Features**:
  - Define named strings: `string-set: header "Chapter 1"`
  - Attribute extraction: `string-set: title attr(data-title)`
  - Counter integration: `string-set: chapter counter(chapter)`

### String Retrieval
- **Location**: `src/pdf/print_css.rs` - `PageContext::running_strings`
- **Features**:
  - Store and retrieve running strings per page
  - Automatic string updates based on content

### First Page Special Handling
- **Location**: `src/pdf/print_css.rs` - `PageContext::is_first`
- **Features**:
  - Special margin boxes for first page
  - Different header/footer on first page
  - Suppress content on first page

## 5. Generated Content for Paged Media

### Target-Counter
- **Location**: `src/pdf/print_css.rs` - `MarginContentPart::TargetCounter`
- **Features**:
  - Cross-reference page numbers: `target-counter(url, page)`
  - Structure ready for document-wide counter resolution

### Leader Function
- **Location**: `src/pdf/print_css.rs` - `MarginContentPart::Leader`
- **Features**:
  - Dotted leaders: `leader('.')`
  - For table of contents entries
  - Space filling in margin boxes

## 6. Page Counters

### Page Numbering
- **Location**: `src/pdf/print_css.rs` - `PageCounter` struct
- **Features**:
  - Current page tracking
  - Total pages calculation
  - Page counter reset
  - Named counters support

### Counter Reset on Chapters
- **Location**: `src/pdf/print_css.rs` - `PageCounter::reset_named_counter`
- **Features**:
  - Reset page counter at chapter boundaries
  - Named counter management
  - Multiple counter chains

### Counter Display
- **Features**:
  - `counter(page)` in margin boxes
  - `counter(pages)` for "Page X of Y"
  - Chapter-relative page numbers

## 7. PDF Bookmarks/Outlines

### Bookmark Structure
- **Location**: `src/pdf/print_css.rs` - `Bookmark` struct
- **Features**:
  - Hierarchical bookmarks
  - Page destinations
  - Title text
  - Nested levels

### PDF Integration
- **Location**: `src/pdf/writer.rs` - `PdfWriter::build_outline`
- **Features**:
  - Automatic outline generation from bookmarks
  - Parent-child relationships
  - PDF destination arrays

## File Structure

```
src/pdf/
├── print_css.rs         # Core PrintCSS types and logic (895 lines)
├── print_css_tests.rs   # Unit tests for PrintCSS features (219 lines)
├── writer.rs            # Enhanced PDF writer with margin box support (678 lines)
└── mod.rs               # Updated exports (589 lines)

src/layout/
├── style.rs             # Updated with break-* properties and string-set
└── mod.rs               # Updated exports

src/css/
├── at_rules.rs          # Page rules and margin box types
├── parser.rs            # @page rule parsing
└── values.rs            # CSS value types
```

## Usage Examples

### Basic @page Rule
```css
@page {
  size: A4;
  margin: 1in;
}
```

### Page Selectors
```css
@page :first {
  margin-top: 2in;
}

@page :left {
  margin-left: 1.5in;
  margin-right: 1in;
}

@page :right {
  margin-left: 1in;
  margin-right: 1.5in;
}
```

### Margin Boxes
```css
@page {
  @top-center {
    content: string(chapter-title);
    font-size: 10pt;
  }
  
  @bottom-center {
    content: counter(page);
  }
}
```

### Running Headers
```css
h1 {
  string-set: chapter-title content();
}
```

### Page Breaks
```css
.chapter {
  break-before: page;
  break-after: avoid;
}

.keep-together {
  break-inside: avoid;
}
```

### Named Pages
```css
@page chapter {
  @top-center {
    content: "Chapter Page";
  }
}

.chapter {
  page: chapter;
}
```

## Summary

The html2pdf-rs library now has comprehensive PrintCSS support including:

1. **@page rules** with full selector support and named pages
2. **16 margin box types** with content generation
3. **Page break control** via both legacy and modern CSS properties
4. **Running headers/footers** via string-set
5. **Page counters** with named counter support
6. **Generated content** functions (string(), counter(), leader())
7. **PDF bookmarks** for document navigation

Total new code: ~1,500 lines across multiple files, with comprehensive unit tests for all major features.
