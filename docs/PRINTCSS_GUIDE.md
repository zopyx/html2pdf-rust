# PrintCSS Guide

> Complete tutorial for CSS Paged Media with html2pdf

---

## Table of Contents

1. [Introduction to PrintCSS](#introduction-to-printcss)
2. [@page Rules](#page-rules)
3. [Page Size and Orientation](#page-size-and-orientation)
4. [Page Selectors](#page-selectors)
5. [Margin Boxes](#margin-boxes)
6. [Running Headers and Footers](#running-headers-and-footers)
7. [Page Breaks](#page-breaks)
8. [Orphans and Widows](#orphans-and-widows)
9. [Named Pages](#named-pages)
10. [Complete Example](#complete-example)
11. [Best Practices](#best-practices)

---

## Introduction to PrintCSS

PrintCSS (also known as CSS Paged Media) is a set of CSS specifications designed specifically for paginated output like PDF documents. While standard CSS is designed for continuous scrollable media (websites), PrintCSS adds features for:

- **Page-based layouts**: Define different layouts for different pages
- **Headers and footers**: Running headers and footers that repeat on each page
- **Page breaks**: Control where content breaks across pages
- **Page margins**: Complex margin configurations with content areas
- **Cross-references**: Page numbers, counters, and generated content

### Why PrintCSS?

Traditional HTML-to-PDF converters often struggle with pagination because they:
1. First render as a web page
2. Then try to "slice" the page into PDF pages

html2pdf takes a different approach:
1. Parses HTML and CSS
2. Understands page boundaries from the start
3. Lays out content specifically for the target page size

This results in professional-quality PDF documents with proper typography and pagination.

---

## @page Rules

The `@page` rule is the foundation of PrintCSS. It allows you to define page properties that apply to the printed output.

### Basic Syntax

```css
@page {
  /* Page properties */
  size: A4;
  margin: 2cm;
}
```

### Common Page Properties

| Property | Values | Description |
|----------|--------|-------------|
| `size` | paper name or dimensions | Page size |
| `margin` | length | Page margins |
| `marks` | `crop`, `cross`, `none` | Printer marks |
| `bleed` | length | Bleed area for trimming |

### Setting Page Size

```css
/* Standard paper sizes */
@page {
  size: A4;           /* ISO A4 */
  size: Letter;       /* US Letter */
  size: Legal;        /* US Legal */
}

/* Custom dimensions */
@page {
  size: 210mm 297mm;  /* Width Height */
  size: 8.5in 11in;   /* US Letter in inches */
}

/* Landscape orientation */
@page {
  size: A4 landscape;
  /* or */
  size: landscape;
}
```

### Setting Margins

```css
/* Uniform margins */
@page {
  margin: 2cm;
}

/* Vertical and horizontal margins */
@page {
  margin: 2cm 1.5cm;  /* vertical horizontal */
}

/* Individual margins (TRBL) */
@page {
  margin: 2cm 1.5cm 2cm 1.5cm;  /* top right bottom left */
}

/* Individual properties */
@page {
  margin-top: 2cm;
  margin-right: 1.5cm;
  margin-bottom: 2cm;
  margin-left: 1.5cm;
}
```

---

## Page Size and Orientation

### Standard Paper Sizes

html2pdf supports standard paper sizes by name:

```css
@page {
  /* ISO Sizes */
  size: A0;  /* 841 × 1189 mm */
  size: A1;  /* 594 × 841 mm */
  size: A2;  /* 420 × 594 mm */
  size: A3;  /* 297 × 420 mm */
  size: A4;  /* 210 × 297 mm */
  size: A5;  /* 148 × 210 mm */
  size: A6;  /* 105 × 148 mm */
  
  /* US Sizes */
  size: Letter;   /* 8.5 × 11 inches */
  size: Legal;    /* 8.5 × 14 inches */
  size: Tabloid;  /* 11 × 17 inches */
}
```

### Orientation

```css
/* Portrait (default) */
@page {
  size: A4 portrait;
}

/* Landscape */
@page {
  size: A4 landscape;
}

/* Orientation only (uses default size) */
@page {
  size: landscape;
}
```

### Custom Page Sizes

```css
/* Custom dimensions */
@page {
  size: 200mm 250mm;  /* Custom book size */
}

/* With orientation */
@page {
  size: 297mm 210mm;  /* A4 landscape by dimensions */
}
```

---

## Page Selectors

Page selectors allow you to apply different styles to different types of pages.

### :first Selector

Styles for the first page of the document:

```css
@page :first {
  margin-top: 5cm;  /* Extra top margin for title page */
  
  @top-center {
    content: none;  /* No header on first page */
  }
}
```

### :left and :right Selectors

For facing pages (like in a book):

```css
/* Left pages (verso) */
@page :left {
  margin-left: 3cm;   /* Extra margin for binding */
  margin-right: 2cm;
}

/* Right pages (recto) */
@page :right {
  margin-left: 2cm;
  margin-right: 3cm;  /* Extra margin for binding */
}

/* Page numbers on outside */
@page :left {
  @bottom-left {
    content: counter(page);
  }
}

@page :right {
  @bottom-right {
    content: counter(page);
  }
}
```

### :blank Selector

For intentionally blank pages:

```css
@page :blank {
  @top-center { content: none; }
  @bottom-center { content: none; }
}
```

### Named Page Selectors

See [Named Pages](#named-pages) section below.

---

## Margin Boxes

Page margin boxes are rectangular areas in the page margins that can contain generated content like headers, footers, and page numbers.

### The 16 Margin Boxes

```
+--------------------------------------------------+
| top-left-corner   top-left   top-center   top-right   top-right-corner |
|               +--------------------------------+                           |
| left-top      |                                |      right-top          |
|               |                                |                           |
| left-middle   |         CONTENT AREA           |      right-middle       |
|               |                                |                           |
| left-bottom   |                                |      right-bottom       |
|               +--------------------------------+                           |
| bottom-left-corner bottom-left bottom-center bottom-right bottom-right-corner |
+--------------------------------------------------+
```

### Common Margin Box Properties

```css
@page {
  @top-center {
    content: "Document Title";
    font-size: 9pt;
    color: #666;
  }
  
  @bottom-center {
    content: counter(page);
    font-size: 9pt;
  }
}
```

### Margin Box Properties

| Property | Description |
|----------|-------------|
| `content` | Generated content (text, counters, strings) |
| `font-*` | Font properties (family, size, weight, etc.) |
| `color` | Text color |
| `text-align` | Horizontal alignment |
| `vertical-align` | Vertical alignment |
| `border-*` | Borders |
| `padding` | Internal spacing |
| `background-*` | Background color/image |
| `width` / `height` | Explicit dimensions |

### Examples by Position

**Top Header:**
```css
@page {
  @top-center {
    content: "Annual Report 2024";
    font-family: "Helvetica", sans-serif;
    font-size: 10pt;
    color: #333;
    border-bottom: 1pt solid #ccc;
    padding-bottom: 5pt;
  }
}
```

**Bottom Footer with Page Numbers:**
```css
@page {
  @bottom-center {
    content: "Page " counter(page) " of " counter(pages);
    font-size: 9pt;
    color: #666;
  }
}
```

**Outside Page Numbers (Book Style):**
```css
@page :left {
  @bottom-left {
    content: counter(page);
  }
}

@page :right {
  @bottom-right {
    content: counter(page);
  }
}
```

**Side Margins:**
```css
@page {
  @left-middle {
    content: "Confidential";
    writing-mode: vertical-rl;
    transform: rotate(180deg);
    font-size: 8pt;
    color: #999;
  }
}
```

---

## Running Headers and Footers

Running headers and footers can display content from the document, such as chapter titles.

### Using string-set

The `string-set` property captures text content to use in headers/footers:

```css
/* Capture chapter titles */
h1.chapter-title {
  string-set: chapter-title content();
}

/* Use in header */
@page {
  @top-left {
    content: string(chapter-title);
    font-size: 9pt;
  }
}
```

### String Options

```css
/* First occurrence on page (default) */
content: string(chapter-title);
content: string(chapter-title, first);

/* Last occurrence on page */
content: string(chapter-title, last);

/* Start of page */
content: string(chapter-title, start);

/* First matching element */
content: string(chapter-title, first-except);
```

### Multiple Strings

```css
h1 {
  string-set: document-title content();
}

h2 {
  string-set: section-title content();
}

@page {
  @top-left {
    content: string(document-title);
  }
  
  @top-right {
    content: string(section-title);
  }
}
```

### Running Elements

Use `position: running()` to move elements to margin boxes:

```html
<div class="running-header">
  <span class="chapter">Chapter 1</span>
  <span class="title">Introduction</span>
</div>
```

```css
.running-header {
  position: running(header);
  display: none; /* Hide from normal flow */
}

@page {
  @top-center {
    content: element(header);
  }
}
```

---

## Page Breaks

Control where content breaks across pages.

### Break Properties

| Property | Values | Description |
|----------|--------|-------------|
| `break-before` | `page`, `column`, `avoid-page`, `avoid-column`, `auto` | Force or avoid break before element |
| `break-after` | `page`, `column`, `avoid-page`, `avoid-column`, `auto` | Force or avoid break after element |
| `break-inside` | `avoid`, `auto` | Avoid break inside element |

### Legacy Properties (also supported)

| Property | Values |
|----------|--------|
| `page-break-before` | `always`, `avoid`, `auto`, `left`, `right` |
| `page-break-after` | `always`, `avoid`, `auto`, `left`, `right` |
| `page-break-inside` | `avoid`, `auto` |

### Common Patterns

**Chapters on New Pages:**
```css
.chapter {
  break-before: page;
}

/* Or using legacy syntax */
.chapter {
  page-break-before: always;
}
```

**Keep Elements Together:**
```css
.keep-together {
  break-inside: avoid;
}

/* Useful for tables, figures, code blocks */
table, figure, pre {
  break-inside: avoid;
}
```

**Keep Headings with Content:**
```css
h1, h2, h3 {
  break-after: avoid;
}

h2 + p, h3 + p {
  break-before: avoid;
}
```

**Force Break After Specific Elements:**
```css
.cover-page {
  break-after: page;
}

.toc {
  break-after: page;
}
```

### Column Breaks

```css
/* Force column break */
.column-break {
  break-before: column;
}

/* Avoid column break inside */
.no-column-break {
  break-inside: avoid-column;
}
```

---

## Orphans and Widows

Orphans and widows control the minimum number of lines that must be left at the bottom (orphans) or top (widows) of a page.

### Definitions

- **Orphan**: A paragraph line that appears alone at the bottom of a page
- **Widow**: A paragraph line that appears alone at the top of a page

### Setting Minimum Lines

```css
/* Prevent single-line orphans and widows */
p {
  orphans: 2;
  widows: 2;
}

/* Require at least 3 lines */
p {
  orphans: 3;
  widows: 3;
}
```

### Best Practice

```css
/* Apply to all paragraphs and list items */
p, li {
  orphans: 2;
  widows: 2;
}

/* Headings should never be alone */
h1, h2, h3, h4 {
  orphans: 3;
  widows: 3;
}
```

---

## Named Pages

Named pages allow you to define different page layouts for different sections of your document.

### Basic Usage

```css
/* Define named page */
@page cover {
  margin: 0;
  background: #2c3e50;
  
  @top-center { content: none; }
  @bottom-center { content: none; }
}

@page chapter {
  margin: 2.5cm 2cm;
  
  @top-left {
    content: "Chapter " counter(chapter);
  }
}

/* Apply to elements */
.cover {
  page: cover;
}

.chapter {
  page: chapter;
  break-before: page;
}
```

### Combining with Selectors

```css
@page cover {
  margin: 0;
}

@page cover:first {
  /* Styles for first page of cover sequence */
}

@page chapter {
  margin: 2cm;
}

@page chapter:left {
  /* Left pages in chapter sequence */
  margin-left: 3cm;
}

@page chapter:right {
  /* Right pages in chapter sequence */
  margin-right: 3cm;
}
```

### Example: Book Layout

```css
/* Cover page - no margins, full bleed */
@page cover {
  margin: 0;
  @top-center { content: none; }
  @bottom-center { content: none; }
}

/* Table of contents */
@page toc {
  @top-center {
    content: "Table of Contents";
  }
}

/* Chapter pages */
@page chapter {
  @top-left {
    content: string(chapter-title);
  }
  @bottom-center {
    content: counter(page);
  }
}

@page chapter:first {
  @top-left { content: none; }
}

/* Index */
@page index {
  @top-center {
    content: "Index";
  }
}

/* Apply named pages */
.cover { page: cover; }
.toc { page: toc; }
.chapter { page: chapter; }
.index { page: index; }
```

---

## Complete Example

Here's a complete example demonstrating PrintCSS features:

```html
<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <title>Annual Report 2024</title>
  <style>
    /* ========== PAGE DEFINITIONS ========== */
    
    /* Default page */
    @page {
      size: A4;
      margin: 2.5cm 2cm;
      
      @top-center {
        content: "Annual Report 2024";
        font-family: "Helvetica", sans-serif;
        font-size: 9pt;
        color: #666;
      }
      
      @bottom-center {
        content: counter(page);
        font-size: 9pt;
      }
    }
    
    /* Cover page - full bleed */
    @page cover {
      margin: 0;
      background: linear-gradient(135deg, #1e3c72 0%, #2a5298 100%);
      
      @top-center { content: none; }
      @bottom-center { content: none; }
    }
    
    /* First page of chapters - no header */
    @page :first {
      margin-top: 4cm;
      @top-center { content: none; }
    }
    
    /* Left and right pages (binding margins) */
    @page :left {
      margin-left: 2.5cm;
      margin-right: 1.5cm;
      
      @bottom-left {
        content: counter(page);
      }
      @bottom-center { content: none; }
    }
    
    @page :right {
      margin-left: 1.5cm;
      margin-right: 2.5cm;
      
      @bottom-right {
        content: counter(page);
      }
      @bottom-center { content: none; }
    }
    
    /* ========== TYPOGRAPHY ========== */
    
    * {
      box-sizing: border-box;
    }
    
    body {
      font-family: "Georgia", serif;
      font-size: 11pt;
      line-height: 1.6;
      color: #333;
    }
    
    h1 {
      font-family: "Helvetica", sans-serif;
      font-size: 28pt;
      font-weight: 300;
      color: #2c3e50;
      margin-top: 0;
      break-after: avoid;
    }
    
    h2 {
      font-family: "Helvetica", sans-serif;
      font-size: 18pt;
      color: #34495e;
      margin-top: 1.5em;
      break-after: avoid;
    }
    
    h3 {
      font-size: 14pt;
      break-after: avoid;
    }
    
    p {
      orphans: 2;
      widows: 2;
      text-align: justify;
    }
    
    /* ========== PAGE ASSIGNMENTS ========== */
    
    .cover {
      page: cover;
      height: 100vh;
      display: flex;
      flex-direction: column;
      justify-content: center;
      align-items: center;
      color: white;
      text-align: center;
    }
    
    .cover h1 {
      color: white;
      font-size: 42pt;
      margin-bottom: 0.5em;
    }
    
    .cover .subtitle {
      font-size: 18pt;
      opacity: 0.9;
    }
    
    .toc {
      break-before: page;
    }
    
    .chapter {
      break-before: page;
    }
    
    /* ========== RUNNING HEADERS ========== */
    
    h2 {
      string-set: section-title content();
    }
    
    @page :left {
      @top-left {
        content: string(section-title);
        font-size: 9pt;
        color: #666;
      }
      @top-center { content: none; }
    }
    
    @page :right {
      @top-right {
        content: string(section-title);
        font-size: 9pt;
        color: #666;
      }
      @top-center { content: none; }
    }
    
    /* ========== FIGURES AND TABLES ========== */
    
    figure {
      break-inside: avoid;
      margin: 1.5em 0;
    }
    
    figure img {
      max-width: 100%;
      display: block;
    }
    
    figcaption {
      font-size: 10pt;
      color: #666;
      margin-top: 0.5em;
      text-align: center;
    }
    
    table {
      width: 100%;
      border-collapse: collapse;
      break-inside: avoid;
      margin: 1em 0;
    }
    
    th, td {
      padding: 0.5em;
      text-align: left;
      border-bottom: 1pt solid #ddd;
    }
    
    th {
      font-weight: bold;
      border-bottom: 2pt solid #333;
    }
    
    /* ========== LISTS ========== */
    
    ul, ol {
      orphans: 2;
      widows: 2;
    }
    
    li {
      orphans: 2;
      widows: 2;
    }
    
    /* ========== PRINT OPTIMIZATIONS ========== */
    
    a {
      color: #2a5298;
      text-decoration: none;
    }
    
    a[href]::after {
      content: " (" attr(href) ")";
      font-size: 9pt;
      color: #666;
    }
    
    /* Don't show URLs for internal links */
    a[href^="#"]::after {
      content: none;
    }
    
    /* Page break utilities */
    .page-break {
      break-before: page;
    }
    
    .no-break {
      break-inside: avoid;
    }
  </style>
</head>
<body>
  <!-- Cover Page -->
  <div class="cover">
    <h1>Annual Report 2024</h1>
    <div class="subtitle">Company Performance and Future Outlook</div>
  </div>
  
  <!-- Table of Contents -->
  <div class="toc">
    <h1>Table of Contents</h1>
    <ul>
      <li>Executive Summary</li>
      <li>Financial Overview</li>
      <li>Market Analysis</li>
      <li>Future Outlook</li>
    </ul>
  </div>
  
  <!-- Chapter 1 -->
  <div class="chapter">
    <h1>Executive Summary</h1>
    <p>This report provides a comprehensive overview...</p>
    
    <h2>Key Highlights</h2>
    <ul>
      <li>Revenue increased by 25%</li>
      <li>Expanded to 3 new markets</li>
      <li>Launched 5 new products</li>
    </ul>
  </div>
  
  <!-- Chapter 2 -->
  <div class="chapter">
    <h1>Financial Overview</h1>
    <p>Our financial performance this year...</p>
    
    <figure>
      <table>
        <thead>
          <tr>
            <th>Quarter</th>
            <th>Revenue</th>
            <th>Expenses</th>
            <th>Profit</th>
          </tr>
        </thead>
        <tbody>
          <tr>
            <td>Q1</td>
            <td>$1.2M</td>
            <td>$0.9M</td>
            <td>$0.3M</td>
          </tr>
          <tr>
            <td>Q2</td>
            <td>$1.4M</td>
            <td>$1.0M</td>
            <td>$0.4M</td>
          </tr>
        </tbody>
      </table>
      <figcaption>Quarterly Financial Summary</figcaption>
    </figure>
  </div>
</body>
</html>
```

---

## Best Practices

### 1. Always Set Orphans and Widows

```css
p, li {
  orphans: 2;
  widows: 2;
}
```

### 2. Keep Headings with Content

```css
h1, h2, h3 {
  break-after: avoid;
}

h1 + *, h2 + *, h3 + * {
  break-before: avoid;
}
```

### 3. Keep Related Elements Together

```css
figure, table, pre, .keep-together {
  break-inside: avoid;
}
```

### 4. Use Named Pages for Different Layouts

```css
@page cover { margin: 0; }
@page chapter { margin: 2.5cm 2cm; }
@page appendix { margin: 2cm; }
```

### 5. Design for Binding

```css
@page :left {
  margin-left: 2.5cm;  /* Extra for binding */
  margin-right: 1.5cm;
}

@page :right {
  margin-left: 1.5cm;
  margin-right: 2.5cm;  /* Extra for binding */
}
```

### 6. Use Relative Units

```css
/* Good */
@page {
  margin: 2cm;
  font-size: 11pt;
}

/* Avoid */
@page {
  margin: 56.6929pt;  /* Hard to understand */
}
```

### 7. Test with Real Content

Always test your PrintCSS with realistic content amounts to catch pagination issues.

### 8. Progressive Enhancement

Start with a solid base stylesheet, then add PrintCSS enhancements:

```css
/* Base styles */
body {
  font-family: Georgia, serif;
  line-height: 1.6;
}

/* PrintCSS enhancements */
@page {
  margin: 2cm;
}

p {
  orphans: 2;
  widows: 2;
}
```

---

## Related Documentation

- [User Guide](USER_GUIDE.md) - Complete user guide
- [CSS Support Reference](CSS_SUPPORT.md) - CSS properties and selectors
- [API Guide](API_GUIDE.md) - Library usage for Rust developers
- [README.md](../README.md) - Project overview

---

## References

- [CSS Paged Media Module Level 3](https://www.w3.org/TR/css-page-3/)
- [CSS Generated Content for Paged Media](https://www.w3.org/TR/css-gcpm-3/)
- [CSS Fragmentation Module Level 3](https://www.w3.org/TR/css-break-3/)
- [CSS Page Floats](https://www.w3.org/TR/css-page-floats-3/)
