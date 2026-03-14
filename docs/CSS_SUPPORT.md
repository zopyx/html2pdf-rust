# CSS Support Reference

> Complete reference for CSS properties, selectors, and at-rules supported by html2pdf

---

## Table of Contents

1. [Overview](#overview)
2. [Supported CSS Properties](#supported-css-properties)
3. [Supported Selectors](#supported-selectors)
4. [Supported At-Rules](#supported-at-rules)
5. [CSS Units](#css-units)
6. [CSS Functions](#css-functions)
7. [Limitations](#limitations)
8. [Browser Compatibility Notes](#browser-compatibility-notes)

---

## Overview

html2pdf implements CSS Syntax Module Level 3 with extensive support for CSS Paged Media (PrintCSS). This document provides a comprehensive reference of all supported CSS features.

### Feature Status Legend

| Symbol | Meaning |
|--------|---------|
| ✅ | Fully supported |
| 🚧 | Partially supported / In development |
| ❌ | Not yet supported |

---

## Supported CSS Properties

### Layout Properties

| Property | Status | Notes |
|----------|--------|-------|
| `display` | ✅ | block, inline, inline-block, none, flex, grid, table |
| `position` | ✅ | static, relative, absolute, fixed |
| `top`, `right`, `bottom`, `left` | ✅ | Position offsets |
| `z-index` | ✅ | Layer ordering |
| `float` | ✅ | left, right, none |
| `clear` | ✅ | left, right, both, none |
| `visibility` | ✅ | visible, hidden |
| `overflow` | ✅ | visible, hidden, scroll, auto |
| `overflow-x`, `overflow-y` | ✅ | Axis-specific overflow |
| `box-sizing` | ✅ | content-box, border-box |
| `clip` | 🚧 | Limited support |
| `resize` | ❌ | Not applicable to PDF |
| `cursor` | ❌ | Not applicable to PDF |

### Flexbox Properties

| Property | Status | Notes |
|----------|--------|-------|
| `flex` | ✅ | Shorthand property |
| `flex-grow` | ✅ | Growth factor |
| `flex-shrink` | ✅ | Shrink factor |
| `flex-basis` | ✅ | Base size |
| `flex-direction` | ✅ | row, row-reverse, column, column-reverse |
| `flex-wrap` | ✅ | wrap, nowrap, wrap-reverse |
| `flex-flow` | ✅ | Shorthand |
| `justify-content` | ✅ | flex-start, center, flex-end, space-between, space-around |
| `align-items` | ✅ | Cross-axis alignment |
| `align-content` | ✅ | Multi-line alignment |
| `align-self` | ✅ | Individual item alignment |
| `order` | ✅ | Visual ordering |
| `gap` | ✅ | Grid/Flex gap |
| `row-gap` | ✅ | Row gap |
| `column-gap` | ✅ | Column gap |

### Grid Properties

| Property | Status | Notes |
|----------|--------|-------|
| `grid` | 🚧 | Basic support |
| `grid-template` | 🚧 | Basic support |
| `grid-template-columns` | 🚧 | Basic support |
| `grid-template-rows` | 🚧 | Basic support |
| `grid-template-areas` | ❌ | Planned |
| `grid-auto-columns` | ❌ | Planned |
| `grid-auto-rows` | ❌ | Planned |
| `grid-auto-flow` | ❌ | Planned |
| `grid-column`, `grid-row` | 🚧 | Basic support |
| `grid-area` | ❌ | Planned |
| `justify-items` | 🚧 | Basic support |
| `justify-self` | 🚧 | Basic support |

### Box Model Properties

| Property | Status | Notes |
|----------|--------|-------|
| `width`, `height` | ✅ | All length units |
| `min-width`, `min-height` | ✅ | Minimum dimensions |
| `max-width`, `max-height` | ✅ | Maximum dimensions |
| `margin` | ✅ | All sides shorthand |
| `margin-top`, `margin-right`, `margin-bottom`, `margin-left` | ✅ | Individual margins |
| `padding` | ✅ | All sides shorthand |
| `padding-top`, `padding-right`, `padding-bottom`, `padding-left` | ✅ | Individual padding |
| `border` | ✅ | Shorthand property |
| `border-width` | ✅ | All sides |
| `border-style` | ✅ | solid, dashed, dotted, none |
| `border-color` | ✅ | Color values |
| `border-radius` | 🚧 | Basic support |
| `border-collapse` | ✅ | Table property |
| `border-spacing` | ✅ | Table property |
| `outline` | 🚧 | Limited support |
| `outline-offset` | ❌ | Not yet supported |

### Background Properties

| Property | Status | Notes |
|----------|--------|-------|
| `background` | ✅ | Shorthand |
| `background-color` | ✅ | Color values |
| `background-image` | 🚧 | Basic support |
| `background-position` | 🚧 | Basic support |
| `background-size` | 🚧 | Basic support |
| `background-repeat` | ✅ | repeat, no-repeat, repeat-x, repeat-y |
| `background-origin` | ❌ | Not yet supported |
| `background-clip` | ❌ | Not yet supported |
| `background-attachment` | ❌ | Not applicable to PDF |
| `background-blend-mode` | ❌ | Not yet supported |

### Color Properties

| Property | Status | Notes |
|----------|--------|-------|
| `color` | ✅ | Text color |
| `opacity` | ✅ | Element opacity (0-1) |
| `mix-blend-mode` | ❌ | Not yet supported |

### Typography Properties

| Property | Status | Notes |
|----------|--------|-------|
| `font` | ✅ | Shorthand property |
| `font-family` | ✅ | Font family names |
| `font-size` | ✅ | All length units |
| `font-weight` | ✅ | 100-900, normal, bold |
| `font-style` | ✅ | normal, italic, oblique |
| `font-variant` | ✅ | Small caps |
| `font-stretch` | 🚧 | Limited font support |
| `line-height` | ✅ | Number or length |
| `text-align` | ✅ | left, center, right, justify |
| `text-align-last` | 🚧 | Last line alignment |
| `text-indent` | ✅ | Paragraph indentation |
| `text-justify` | 🚧 | Justification method |
| `text-transform` | ✅ | uppercase, lowercase, capitalize |
| `text-decoration` | ✅ | underline, overline, line-through |
| `text-decoration-line` | ✅ | Decoration type |
| `text-decoration-color` | ✅ | Decoration color |
| `text-decoration-style` | ✅ | solid, dashed, dotted, wavy |
| `text-decoration-thickness` | 🚧 | Line thickness |
| `text-shadow` | ❌ | Not yet supported |
| `letter-spacing` | ✅ | Character spacing |
| `word-spacing` | ✅ | Word spacing |
| `white-space` | ✅ | normal, nowrap, pre, pre-wrap, pre-line |
| `word-wrap` / `overflow-wrap` | ✅ | Break long words |
| `word-break` | ✅ | Line breaking rules |
| `line-break` | 🚧 | Basic support |
| `hyphens` | ❌ | Not yet supported |
| `text-overflow` | ❌ | Not applicable to PDF |
| `vertical-align` | ✅ | inline elements |
| `direction` | ✅ | ltr, rtl |
| `unicode-bidi` | 🚧 | Basic support |
| `writing-mode` | 🚧 | Basic support |

### List Properties

| Property | Status | Notes |
|----------|--------|-------|
| `list-style` | ✅ | Shorthand |
| `list-style-type` | ✅ | disc, circle, square, decimal, etc. |
| `list-style-position` | ✅ | inside, outside |
| `list-style-image` | 🚧 | Basic support |

### Table Properties

| Property | Status | Notes |
|----------|--------|-------|
| `table-layout` | ✅ | auto, fixed |
| `caption-side` | ✅ | top, bottom |
| `empty-cells` | ✅ | show, hide |
| `border-collapse` | ✅ | collapse, separate |
| `border-spacing` | ✅ | Spacing between cells |

### Transform Properties

| Property | Status | Notes |
|----------|--------|-------|
| `transform` | ❌ | Planned for future |
| `transform-origin` | ❌ | Planned |
| `transform-style` | ❌ | Planned |

### Transition & Animation Properties

| Property | Status | Notes |
|----------|--------|-------|
| `transition` | ❌ | Not applicable to PDF |
| `animation` | ❌ | Not applicable to PDF |

### PrintCSS / Paged Media Properties

| Property | Status | Notes |
|----------|--------|-------|
| `page` | ✅ | Named page assignment |
| `page-break-before` | ✅ | Legacy: always, avoid, auto |
| `page-break-after` | ✅ | Legacy: always, avoid, auto |
| `page-break-inside` | ✅ | Legacy: avoid, auto |
| `break-before` | ✅ | page, column, avoid-page, avoid-column, auto |
| `break-after` | ✅ | page, column, avoid-page, avoid-column, auto |
| `break-inside` | ✅ | avoid, auto |
| `orphans` | ✅ | Minimum lines at bottom |
| `widows` | ✅ | Minimum lines at top |
| `box-decoration-break` | 🚧 | Basic support |
| `marks` | 🚧 | Crop marks |
| `bleed` | 🚧 | Bleed area |
| `string-set` | 🚧 | Running headers |
| `running` | 🚧 | Running elements |

### Generated Content Properties

| Property | Status | Notes |
|----------|--------|-------|
| `content` | ✅ | For pseudo-elements |
| `quotes` | ✅ | Quote characters |
| `counter-increment` | ✅ | Counter increment |
| `counter-reset` | ✅ | Counter initialization |
| `counter-set` | 🚧 | Counter setting |

---

## Supported Selectors

### Basic Selectors

| Selector | Status | Example |
|----------|--------|---------|
| Universal | ✅ | `*` |
| Type/Element | ✅ | `div`, `p`, `h1` |
| Class | ✅ | `.container`, `.btn-primary` |
| ID | ✅ | `#header`, `#main-content` |
| Attribute presence | ✅ | `[disabled]`, `[required]` |
| Attribute equals | ✅ | `[type="text"]`, `[lang="en"]` |
| Attribute contains word | ✅ | `[class~="active"]` |
| Attribute starts with | ✅ | `[href^="https"]` |
| Attribute ends with | ✅ | `[src$=".png"]` |
| Attribute contains | ✅ | `[href*="example"]` |
| Attribute dash-separated | ✅ | `[lang|="en"]` |

### Combinators

| Combinator | Status | Example |
|------------|--------|---------|
| Descendant | ✅ | `div p` |
| Child | ✅ | `ul > li` |
| Adjacent sibling | ✅ | `h1 + p` |
| General sibling | ✅ | `h1 ~ p` |

### Pseudo-classes

| Pseudo-class | Status | Notes |
|--------------|--------|-------|
| `:first-child` | ✅ | First child element |
| `:last-child` | ✅ | Last child element |
| `:only-child` | ✅ | Only child element |
| `:nth-child()` | 🚧 | Basic support (an+b) |
| `:nth-last-child()` | ❌ | Planned |
| `:first-of-type` | 🚧 | Basic support |
| `:last-of-type` | 🚧 | Basic support |
| `:only-of-type` | ❌ | Planned |
| `:nth-of-type()` | ❌ | Planned |
| `:nth-last-of-type()` | ❌ | Planned |
| `:not()` | 🚧 | Simple selectors only |
| `:is()` | ❌ | Planned |
| `:where()` | ❌ | Planned |
| `:has()` | ❌ | Planned |
| `:empty` | ✅ | Empty element |
| `:root` | ✅ | Document root |
| `:link` | ❌ | Not applicable |
| `:visited` | ❌ | Not applicable |
| `:hover` | ❌ | Not applicable to PDF |
| `:active` | ❌ | Not applicable to PDF |
| `:focus` | ❌ | Not applicable to PDF |
| `:target` | ❌ | Not applicable to PDF |
| `:checked` | ✅ | Form elements |
| `:disabled` | ✅ | Form elements |
| `:enabled` | ✅ | Form elements |

### Pseudo-elements

| Pseudo-element | Status | Notes |
|----------------|--------|-------|
| `::before` | ✅ | Generated before content |
| `::after` | ✅ | Generated after content |
| `::first-line` | 🚧 | Basic support |
| `::first-letter` | 🚧 | Basic support |
| `::selection` | ❌ | Not applicable to PDF |
| `::marker` | 🚧 | List markers |
| `::placeholder` | ✅ | Form placeholders |

### Page Selectors (PrintCSS)

| Selector | Status | Notes |
|----------|--------|-------|
| `:first` | ✅ | First page |
| `:left` | ✅ | Left pages (verso) |
| `:right` | ✅ | Right pages (recto) |
| `:blank` | ✅ | Blank pages |
| Named pages | ✅ | `@page chapter` |

---

## Supported At-Rules

### Core At-Rules

| At-Rule | Status | Notes |
|---------|--------|-------|
| `@page` | ✅ | Page layout rules |
| `@media` | ✅ | Media queries |
| `@import` | 🚧 | URL imports |
| `@font-face` | 🚧 | Font embedding |
| `@keyframes` | ❌ | Not applicable to PDF |
| `@supports` | 🚧 | Feature queries |
| `@charset` | ✅ | Character encoding |
| `@namespace` | 🚧 | XML namespaces |

### @page Margin Boxes

All 16 page margin boxes are supported:

```css
@page {
  @top-left-corner { }
  @top-left { }
  @top-center { }
  @top-right { }
  @top-right-corner { }
  @bottom-left-corner { }
  @bottom-left { }
  @bottom-center { }
  @bottom-right { }
  @bottom-right-corner { }
  @left-top { }
  @left-middle { }
  @left-bottom { }
  @right-top { }
  @right-middle { }
  @right-bottom { }
}
```

---

## CSS Units

### Length Units

| Unit | Status | Description |
|------|--------|-------------|
| `px` | ✅ | Pixels (96 DPI) |
| `pt` | ✅ | Points (1/72 inch) |
| `pc` | ✅ | Picas (12 points) |
| `in` | ✅ | Inches |
| `cm` | ✅ | Centimeters |
| `mm` | ✅ | Millimeters |
| `em` | ✅ | Relative to font size |
| `rem` | ✅ | Relative to root font size |
| `ex` | 🚧 | Relative to x-height |
| `ch` | 🚧 | Relative to "0" width |
| `vw` | ✅ | Viewport width percentage |
| `vh` | ✅ | Viewport height percentage |
| `vmin` | ✅ | Minimum of vw/vh |
| `vmax` | ✅ | Maximum of vw/vh |

### Other Units

| Unit Type | Status | Examples |
|-----------|--------|----------|
| Percentage | ✅ | `50%`, `100%` |
| Angles | 🚧 | `deg`, `rad`, `turn` |
| Time | ❌ | `s`, `ms` (not applicable) |
| Frequency | ❌ | `Hz`, `kHz` (not applicable) |
| Resolution | 🚧 | `dpi`, `dpcm` |

---

## CSS Functions

| Function | Status | Notes |
|----------|--------|-------|
| `rgb()` / `rgba()` | ✅ | RGB color |
| `hsl()` / `hsla()` | ✅ | HSL color |
| `calc()` | 🚧 | Calculations |
| `var()` | 🚧 | CSS custom properties |
| `url()` | ✅ | URL references |
| `counter()` | ✅ | Counter values |
| `counters()` | 🚧 | Nested counters |
| `attr()` | 🚧 | Attribute values |
| `linear-gradient()` | ❌ | Planned |
| `radial-gradient()` | ❌ | Planned |

---

## Limitations

### Current Limitations

1. **JavaScript**: JavaScript execution is not supported. Documents must be static HTML/CSS.

2. **External Resources**: URL fetching for external resources is limited. Local files are recommended.

3. **Complex Layouts**: Some advanced CSS Grid and Flexbox features are still being implemented.

4. **Web Fonts**: Web font loading from CDNs is not yet fully supported. System fonts work best.

5. **Advanced CSS**: CSS features like `transform`, `filter`, and complex gradients are planned but not yet implemented.

6. **Interactive Elements**: Form fields, interactive PDF features are not yet supported.

### Workarounds

For features not yet supported, consider:

1. **Pre-processing**: Use tools like Sass/Less to generate compatible CSS
2. **Inlining**: Inline all CSS and use data URIs for images
3. **Simplification**: Use simpler layout techniques that are fully supported
4. **Post-processing**: Use PDF manipulation tools for advanced features

---

## Browser Compatibility Notes

### Comparison with Browsers

html2pdf aims to follow web standards but is specifically designed for print/PDF output:

| Feature | html2pdf | Chrome | Firefox | Safari |
|---------|----------|--------|---------|--------|
| `@page` | ✅ Native | Partial | Partial | Partial |
| Page margin boxes | ✅ Full | None | None | None |
| Named pages | ✅ Full | None | None | None |
| Running headers | ✅ Native | None | None | None |
| `break-*` | ✅ Full | Partial | Partial | Partial |

### Print-Specific Behavior

html2pdf behaves more like a dedicated print formatter than a browser:

1. **Viewport**: Uses page size as viewport instead of screen size
2. **Media Queries**: `@media print` is always active
3. **Interactive**: No hover, focus, or active states
4. **Animation**: No transitions or animations
5. **Fonts**: System fonts and embedded fonts only (no web fonts yet)

### Testing Compatibility

To test how your document will render:

```bash
# Use debug mode to see layout information
html2pdf input.html -o output.pdf --debug-layout -v

# Validate CSS
html2pdf validate input.html
```

### Known Differences from Browsers

1. **Box Model**: Strict standards mode always used
2. **Default Styles**: Print-optimized default stylesheet
3. **Pagination**: Native support for page-based layouts
4. **Margin Collapsing**: Follows CSS2.1 specification strictly

---

## Related Documentation

- [User Guide](USER_GUIDE.md) - Complete user guide and tutorials
- [PrintCSS Guide](PRINTCSS_GUIDE.md) - In-depth PrintCSS tutorial
- [API Guide](API_GUIDE.md) - Library usage for Rust developers
- [README.md](../README.md) - Project overview

---

## References

- [CSS Syntax Module Level 3](https://www.w3.org/TR/css-syntax-3/)
- [CSS Paged Media Module Level 3](https://www.w3.org/TR/css-page-3/)
- [CSS Fragmentation Module Level 3](https://www.w3.org/TR/css-break-3/)
- [CSS Generated Content Module Level 3](https://www.w3.org/TR/css-content-3/)
