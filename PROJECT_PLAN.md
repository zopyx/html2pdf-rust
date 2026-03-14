# HTML2PDF-rs Project Plan

## Overview
Building a complete HTML to PDF converter from scratch in Rust with full HTML5, CSS3, and W3C PrintCSS support.

## Architecture

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                           HTML2PDF Converter                                 │
├─────────────────────────────────────────────────────────────────────────────┤
│  Layer 5: CLI & Public API                                                   │
│  ├── Command-line interface                                                  │
│  ├── Configuration system                                                    │
│  └── Library public API                                                      │
├─────────────────────────────────────────────────────────────────────────────┤
│  Layer 4: PrintCSS Engine                                                    │
│  ├── @page rule processor                                                    │
│  ├── Margin boxes (@top-left, @top-center, etc.)                             │
│  ├── Running headers/footers                                                 │
│  ├── Page breaks and orphans/widows                                          │
│  ├── Named pages                                                             │
│  └── Page counters and generated content                                     │
├─────────────────────────────────────────────────────────────────────────────┤
│  Layer 3: Layout Engine                                                      │
│  ├── Box model (margin, border, padding, content)                            │
│  ├── Normal flow (block & inline)                                            │
│  ├── Floats and clear                                                        │
│  ├── Positioning (static, relative, absolute, fixed, sticky)                 │
│  ├── Flexbox layout                                                          │
│  ├── CSS Grid layout                                                         │
│  ├── Table layout                                                            │
│  ├── CSS transforms                                                          │
│  ├── Pagination engine                                                       │
│  └── Line breaking & text shaping                                            │
├─────────────────────────────────────────────────────────────────────────────┤
│  Layer 2: CSS Engine                                                         │
│  ├── CSS parser (full CSS3)                                                  │
│  ├── Selector engine (all CSS3/4 selectors)                                  │
│  ├── Cascade & specificity                                                   │
│  ├── Inheritance                                                             │
│  ├── CSS variables (custom properties)                                       │
│  ├── Media queries                                                           │
│  ├── @supports rules                                                         │
│  ├── @keyframes animations (snapshot)                                        │
│  └── Property value computation                                              │
├─────────────────────────────────────────────────────────────────────────────┤
│  Layer 1: HTML & Resources                                                   │
│  ├── HTML5 tokenizer (WHATWG spec)                                           │
│  ├── Tree builder (fragment & full document)                                 │
│  ├── DOM implementation                                                      │
│  ├── HTTP client for resources                                               │
│  ├── Image loading (PNG, JPEG, GIF, SVG, WebP)                               │
│  ├── Font loading (TTF, OTF, WOFF, WOFF2)                                    │
│  └── CSS stylesheet loading                                                  │
├─────────────────────────────────────────────────────────────────────────────┤
│  Layer 0: PDF Generation (Foundation)                                        │
│  ├── PDF object model                                                        │
│  ├── PDF writer                                                              │
│  ├── Text streams & encoding                                                 │
│  ├── Font embedding (Type1, TrueType, CID)                                   │
│  ├── Image embedding (raw, DCT/JPEG, flate)                                  │
│  ├── Vector graphics (paths, fills, strokes)                                 │
│  ├── Transparency & blending                                                 │
│  └── PDF/A compliance                                                        │
└─────────────────────────────────────────────────────────────────────────────┘
```

## Milestones

### Milestone 0: Foundation (Week 1-2)
**Goal**: Basic PDF generation working

#### Tasks:
- [x] Core types (Point, Rect, Color, Length, etc.)
- [x] PDF object model (Dictionary, Array, Reference)
- [x] PDF stream handling with Flate compression
- [x] PDF writer with cross-reference table
- [ ] Font embedding (Type1, basic TrueType)
- [ ] Image embedding (raw, DCT)
- [ ] Page content operators

**Deliverable**: Can generate simple PDF with text and basic shapes

---

### Milestone 1: HTML Parsing (Week 3-4)
**Goal**: Full HTML5 parsing per WHATWG spec

#### Tasks:
- [x] Tokenizer state machine
- [ ] Tree builder with insertion modes
- [ ] Foster parenting
- [ ] Adoption agency algorithm
- [ ] Template support
- [ ] MathML and SVG integration points
- [ ] Document fragments
- [ ] HTML entity decoding
- [ ] Character encoding detection

**Deliverable**: Can parse any valid HTML5 document into DOM

**Test Suite**:
- html5lib tokenizer tests
- html5lib tree construction tests
- Custom edge cases

---

### Milestone 2: CSS Parsing (Week 5-6)
**Goal**: Full CSS3 parsing

#### Tasks:
- [ ] Tokenizer (CSS Syntax Module Level 3)
- [ ] Parser for all CSS rules
  - [ ] Style rules
  - [ ] @import
  - [ ] @media
  - [ ] @supports
  - [ ] @keyframes
  - [ ] @font-face
  - [ ] @namespace
  - [ ] @page (PrintCSS)
- [ ] Value parsing for all property types
- [ ] CSS custom properties (--*)
- [ ] CSS nesting

**Deliverable**: Can parse any valid CSS3 stylesheet

**Test Suite**:
- CSSWG parsing tests
- Custom property tests
- PrintCSS @page tests

---

### Milestone 3: Selector Engine (Week 7)
**Goal**: Full CSS3/4 selector matching

#### Tasks:
- [ ] Simple selectors (type, class, ID, attribute)
- [ ] Combinators (descendant, child, adjacent, general sibling)
- [ ] Pseudo-classes
  - [ ] Structural (:first-child, :nth-child, etc.)
  - [ ] Input (:checked, :disabled, etc.)
  - [ ] Link (:link, :visited)
  - [ ] Linguistic (:lang)
  - [ ] Logical (:is, :where, :not, :has)
- [ ] Pseudo-elements (::before, ::after, etc.)
- [ ] Selector specificity calculation
- [ ] Selector list matching

**Deliverable**: Can match any CSS selector against DOM

**Test Suite**:
- Selectors Level 4 tests
- Web Platform Tests (selectors)

---

### Milestone 4: CSS Cascade & Computed Styles (Week 8)
**Goal**: Full CSS cascade and computed value resolution

#### Tasks:
- [ ] Cascade origin sorting (UA, user, author)
- [ ] Specificity-based tie-breaking
- [ ] Importance (!important)
- [ ] Inheritance
- [ ] Initial values
- [ ] Computed value calculation
- [ ] CSS variable resolution
- [ ] Media query evaluation
- [ ] @supports evaluation

**Deliverable**: Can compute final styles for any element

**Test Suite**:
- Cascade tests
- Inheritance tests
- CSS variables tests

---

### Milestone 5: Basic Layout - Box Model & Normal Flow (Week 9-10)
**Goal**: Block and inline layout working

#### Tasks:
- [ ] Box model (content, padding, border, margin)
- [ ] Box sizing (content-box, border-box)
- [ ] Block formatting context
- [ ] Inline formatting context
- [ ] Line box generation
- [ ] Inline-block
- [ ] Inline elements
- [ ] Display property (block, inline, inline-block, none)
- [ ] Writing modes (LTR/RTL)
- [ ] Vertical writing mode support

**Deliverable**: Can render basic documents with paragraphs, headings, spans

**Test Suite**:
- Box model tests
- Inline layout tests
- Writing mode tests

---

### Milestone 6: Floats & Positioning (Week 11)
**Goal**: Floats and all positioning schemes

#### Tasks:
- [ ] Floats (left, right)
- [ ] Clear property
- [ ] Block formatting context creation
- [ ] Static positioning
- [ ] Relative positioning
- [ ] Absolute positioning
- [ ] Fixed positioning
- [ ] Sticky positioning (simplified)
- [ ] Z-index and stacking contexts
- [ ] Overflow handling

**Deliverable**: Can render complex layouts with floats and positioned elements

**Test Suite**:
- Float tests
- Positioning tests
- Stacking context tests

---

### Milestone 7: Flexbox Layout (Week 12-13)
**Goal**: Full CSS Flexbox Level 1

#### Tasks:
- [ ] Flex container (display: flex/inline-flex)
- [ ] Flex direction
- [ ] Flex wrap
- [ ] Flex flow shorthand
- [ ] Justify content
- [ ] Align items
- [ ] Align content
- [ ] Gap property
- [ ] Flex item properties
  - [ ] Order
  - [ ] Flex grow
  - [ ] Flex shrink
  - [ ] Flex basis
  - [ ] Flex shorthand
  - [ ] Align self

**Deliverable**: Full Flexbox support

**Test Suite**:
- CSS Flexbox tests
- Custom flex scenarios

---

### Milestone 8: CSS Grid Layout (Week 14-15)
**Goal**: Full CSS Grid Level 1

#### Tasks:
- [ ] Grid container (display: grid/inline-grid)
- [ ] Grid template columns/rows
  - [ ] Track sizes (length, percentage, fr, min-content, max-content)
  - [ ] repeat()
  - [ ] minmax()
  - [ ] fit-content()
  - [ ] auto-fill, auto-fit
- [ ] Grid template areas
- [ ] Grid template shorthand
- [ ] Grid auto columns/rows
- [ ] Grid auto flow
- [ ] Gap properties
- [ ] Grid item placement
  - [ ] Grid column/row start/end
  - [ ] Grid column/row shorthand
  - [ ] Grid area
  - [ ] Named grid areas
  - [ ] Implicit grid

**Deliverable**: Full CSS Grid support

**Test Suite**:
- CSS Grid tests
- Complex grid layouts

---

### Milestone 9: Table Layout (Week 16)
**Goal**: CSS Table layout

#### Tasks:
- [ ] Table formatting
- [ ] Table wrapper box
- [ ] Table cells, rows, row groups
- [ ] Column groups
- [ ] Table captions
- [ ] Border collapse/separate
- [ ] Empty cells
- [ ] Table layout algorithms (fixed, auto)

**Deliverable**: Full table support

**Test Suite**:
- Table layout tests
- Border collapse tests

---

### Milestone 10: Text & Fonts (Week 17-18)
**Goal**: Advanced text handling and font support

#### Tasks:
- [ ] Font loading (system fonts, web fonts)
- [ ] @font-face support
- [ ] Font matching algorithm
- [ ] Font synthesis
- [ ] Text shaping (HarfBuzz integration or native)
- [ ] Line breaking (Unicode Line Breaking Algorithm)
- [ ] Bidirectional text (Unicode Bidi Algorithm)
- [ ] Text decoration
- [ ] Text transform
- [ ] Letter spacing
- [ ] Word spacing
- [ ] Text indentation
- [ ] Text alignment (left, right, center, justify)
- [ ] White space handling

**Deliverable**: Professional typography support

**Test Suite**:
- Font loading tests
- Text shaping tests
- BiDi tests

---

### Milestone 11: Graphics & Images (Week 19)
**Goal**: Full graphics and image support

#### Tasks:
- [ ] Image formats: PNG, JPEG, GIF, BMP, TIFF
- [ ] SVG support (basic to full)
- [ ] Gradients (linear, radial)
- [ ] Multiple backgrounds
- [ ] Background positioning
- [ ] CSS filters
- [ ] Clip paths
- [ ] Masks
- [ ] Blend modes
- [ ] Opacity
- [ ] Border images
- [ ] Box shadows

**Deliverable**: Rich visual content support

**Test Suite**:
- Image rendering tests
- Gradient tests
- Visual effect tests

---

### Milestone 12: PrintCSS - @page Rules (Week 20)
**Goal**: W3C CSS Paged Media Module

#### Tasks:
- [ ] @page rule parsing
- [ ] Page size (auto, length, named sizes)
- [ ] Page orientation
- [ ] Page margins
- [ ] Margin boxes:
  - [ ] @top-left, @top-center, @top-right
  - [ ] @left-top, @left-middle, @left-bottom
  - [ ] @right-top, @right-middle, @right-bottom
  - [ ] @bottom-left, @bottom-center, @bottom-right
- [ ] Page selectors (:first, :left, :right, :blank)
- [ ] Named pages

**Deliverable**: Professional page setup

**Test Suite**:
- @page rule tests
- Margin box tests

---

### Milestone 13: PrintCSS - Pagination (Week 21)
**Goal**: Content fragmentation

#### Tasks:
- [ ] Page breaks
  - [ ] break-before
  - [ ] break-after
  - [ ] break-inside
  - [ ] page-break-* (legacy)
- [ ] Orphans and widows control
- [ ] Box decoration break
- [ ] Running headers/footers
  - [ ] running() value
  - [ ] element() value
- [ ] Page counters
  - [ ] counter-reset: page
  - [ ] counter-increment: page
  - [ ] target-counter() for cross-references
- [ ] Footnotes

**Deliverable**: Professional pagination

**Test Suite**:
- Page break tests
- Running header/footer tests

---

### Milestone 14: PrintCSS - Generated Content (Week 22)
**Goal**: CSS Generated Content for Paged Media

#### Tasks:
- [ ] Leader()
- [ ] target-counter()
- [ ] target-text()
- [ ] string-set
- [ ] string()
- [ ] Cross-references
- [ ] Table of contents generation
- [ ] Bookmarks (PDF outlines)
- [ ] Named destinations

**Deliverable**: Professional publishing features

**Test Suite**:
- Generated content tests
- TOC generation tests

---

### Milestone 15: CLI & Configuration (Week 23)
**Goal**: Production-ready CLI tool

#### Tasks:
- [ ] Command-line argument parsing
- [ ] Configuration file support (TOML, YAML, JSON)
- [ ] Input from file or URL
- [ ] Output to file or stdout
- [ ] Page size options
- [ ] Margin options
- [ ] Header/footer templates
- [ ] Debug mode (show layout boxes)
- [ ] Verbose logging
- [ ] Progress indication
- [ ] Error handling and reporting

**Deliverable**: Usable CLI tool

---

### Milestone 16: Performance & Optimization (Week 24)
**Goal**: Production performance

#### Tasks:
- [ ] Incremental layout
- [ ] Parallel selector matching
- [ ] Font subsetting
- [ ] Image optimization
- [ ] Object reuse and pooling
- [ ] Streaming PDF output
- [ ] Memory optimization
- [ ] Profiling and benchmarks

**Deliverable**: Fast conversion

---

### Milestone 17: Testing & Compliance (Week 25-26)
**Goal**: Comprehensive test coverage

#### Tasks:
- [ ] Unit tests (>80% coverage)
- [ ] Integration tests
- [ ] CSSWG test suite integration
- [ ] html5lib test suite
- [ ] Visual regression tests
- [ ] Performance benchmarks
- [ ] Memory leak detection
- [ ] Fuzzing tests

**Deliverable**: Reliable, well-tested software

---

### Milestone 18: Documentation & Release (Week 27)
**Goal**: Production release

#### Tasks:
- [ ] API documentation (rustdoc)
- [ ] User manual
- [ ] Examples gallery
- [ ] Migration guide from other tools
- [ ] Changelog
- [ ] crates.io publication
- [ ] Binary releases (GitHub)
- [ ] Docker image

**Deliverable**: v1.0 Release

---

## Testing Strategy

### Unit Tests
- Every module has comprehensive unit tests
- Property-based testing for parsers
- Mock objects for external dependencies

### Integration Tests
- End-to-end HTML to PDF conversion
- Visual comparison with reference images
- Cross-browser compatibility checks

### Compliance Tests
- html5lib tokenizer tests (17,000+ tests)
- html5lib tree builder tests (1500+ tests)
- CSSWG CSS2.1 tests (10,000+ tests)
- CSSWG Selectors tests
- CSSWG Flexbox tests
- CSSWG Grid tests

### Performance Tests
- Large document benchmarks
- Memory usage profiling
- Conversion speed targets

### Test Organization
```
tests/
├── unit/                    # Unit tests (in source files)
├── integration/
│   ├── html5lib/           # HTML5 parsing compliance
│   ├── csswg/              # CSS compliance
│   ├── printcss/           # PrintCSS features
│   ├── visual/             # Visual regression
│   └── performance/        # Performance benchmarks
├── fixtures/               # Test HTML/CSS files
└── expected/               # Expected PDF outputs
```

## Dependencies Strategy

### Zero External Dependencies for Core
- Pure Rust implementation
- No browser engines
- No system dependencies except fonts

### Minimal External Dependencies for Extended Features
- `image`: Image format decoding (optional)
- `ttf-parser`: Font parsing (optional, can use subset)
- `miniz_oxide`: Compression (already using)
- `reqwest`: HTTP client (optional, for URL loading)

## Current Status

### Completed
- [x] Core types and PDF foundation
- [x] Basic PDF writer
- [x] HTML tokenizer (partial)
- [x] DOM types

### In Progress
- [ ] HTML tree builder
- [ ] CSS parser
- [ ] Layout engine

### Not Started
- [ ] PrintCSS features
- [ ] CLI
- [ ] Comprehensive test suite

## Estimated Timeline
- **Total Duration**: 27 weeks (6-7 months full-time)
- **Team Size**: 2-3 experienced Rust developers
- **Single Developer**: 12-18 months

## Risk Factors
1. **Text shaping complexity** - May need HarfBuzz integration
2. **JavaScript support** - Not in scope, but may be requested
3. **Performance** - Large documents may require optimization
4. **Memory usage** - Streaming may be needed for large files

## Success Criteria
- [ ] Passes 95%+ of html5lib tests
- [ ] Passes 90%+ of relevant CSSWG tests
- [ ] Converts complex real-world documents
- [ ] Performance: <5s for 100-page document
- [ ] Memory: <500MB for 100-page document
