# html2pdf User Guide

> Complete guide to converting HTML documents to PDF with html2pdf-rs

---

## Table of Contents

1. [Introduction](#introduction)
2. [Installation](#installation)
3. [Quick Start](#quick-start)
4. [Command-Line Usage](#command-line-usage)
5. [Configuration](#configuration)
6. [PrintCSS Guide](#printcss-guide)
7. [Examples](#examples)
8. [Troubleshooting](#troubleshooting)
9. [Related Documentation](#related-documentation)

---

## Introduction

**html2pdf** is a command-line tool and Rust library that converts HTML documents to PDF format. Unlike many other HTML-to-PDF converters, html2pdf is built from scratch in Rust and provides native support for W3C PrintCSS (CSS Paged Media) specifications.

### Key Features

- **Complete HTML5 Parser**: Standards-compliant HTML5 parsing following the WHATWG specification
- **CSS3 Support**: Full CSS3 parsing including selectors, box model, flexbox, and grid
- **PrintCSS / Paged Media**: Native support for `@page` rules, page breaks, headers/footers, and print-specific layouts
- **Multiple Input Sources**: Convert HTML files, URLs, or stdin input
- **Flexible Output**: Write to PDF files or stdout for piping
- **Customizable Page Setup**: Paper sizes (A0-A6, Letter, Legal, etc.), orientation, margins
- **Zero External PDF Dependencies**: Native PDF 1.4 implementation written in Rust

---

## Installation

### From Source (Recommended)

If you have Rust installed (1.75 or later):

```bash
# Clone the repository
git clone https://github.com/yourusername/html2pdf-rs.git
cd html2pdf-rs

# Build in release mode
cargo build --release

# Install to ~/.cargo/bin
cargo install --path .
```

### Using Cargo

```bash
cargo install html2pdf
```

### Pre-built Binaries

Download pre-built binaries from the [Releases page](https://github.com/yourusername/html2pdf-rs/releases):

```bash
# Linux/macOS
curl -L https://github.com/yourusername/html2pdf-rs/releases/latest/download/html2pdf-linux-x64.tar.gz | tar xz
sudo mv html2pdf /usr/local/bin/

# Windows (PowerShell)
Invoke-WebRequest -Uri https://github.com/yourusername/html2pdf-rs/releases/latest/download/html2pdf-windows-x64.zip -OutFile html2pdf.zip
Expand-Archive html2pdf.zip -DestinationPath C:\Tools
```

### Verify Installation

```bash
html2pdf --version
```

Expected output:
```
html2pdf 0.1.0
```

---

## Quick Start

### Convert an HTML file to PDF

```bash
html2pdf input.html -o output.pdf
```

### Convert from stdin

```bash
cat input.html | html2pdf -o output.pdf
# or
html2pdf - -o output.pdf < input.html
```

### Convert a URL (when implemented)

```bash
html2pdf https://example.com -o output.pdf
```

### Simple inline HTML

```bash
echo "<h1>Hello World</h1>" | html2pdf - -o hello.pdf
```

---

## Command-Line Usage

### Basic Syntax

```bash
html2pdf [OPTIONS] [INPUT]
```

### Options Reference

| Option | Short | Description | Default |
|--------|-------|-------------|---------|
| `--output <FILE>` | `-o` | Output PDF file (`-` for stdout) | Derived from input |
| `--paper-size <SIZE>` | `-p` | Paper size: A0-A6, Letter, Legal, Tabloid | A4 |
| `--orientation <ORIENT>` | `-O` | Orientation: Portrait or Landscape | Portrait |
| `--margin <MARGIN>` | `-m` | Page margins (points or with units) | 72pt (1 inch) |
| `--page-width <WIDTH>` | | Custom page width (e.g., `210mm`, `8.5in`) | - |
| `--page-height <HEIGHT>` | | Custom page height (e.g., `297mm`, `11in`) | - |
| `--header <HTML>` | | Header template HTML | - |
| `--footer <HTML>` | | Footer template HTML | - |
| `--header-file <FILE>` | | Path to header HTML file | - |
| `--footer-file <FILE>` | | Path to footer HTML file | - |
| `--config <FILE>` | `-c` | Configuration file path (JSON) | - |
| `--stylesheet <FILE>` | `-s` | Additional CSS stylesheet (can be used multiple times) | - |
| `--base-url <URL>` | | Base URL for resolving relative URLs | - |
| `--timeout <SECONDS>` | | Network timeout for URL fetching | 30 |
| `--verbose` | `-v` | Enable verbose output | - |
| `--debug-layout` | | Show layout debugging information | - |
| `--version` | `-V` | Print version information | - |
| `--help` | `-h` | Print help information | - |

### Subcommands

```bash
# Show version
html2pdf version

# Validate HTML/CSS without generating PDF
html2pdf validate input.html

# Print default configuration
html2pdf config
```

### Paper Sizes

Available paper sizes:

| Size | Dimensions (Portrait) |
|------|----------------------|
| A0 | 841 × 1189 mm |
| A1 | 594 × 841 mm |
| A2 | 420 × 594 mm |
| A3 | 297 × 420 mm |
| A4 | 210 × 297 mm |
| A5 | 148 × 210 mm |
| A6 | 105 × 148 mm |
| Letter | 8.5 × 11 inches |
| Legal | 8.5 × 14 inches |
| Tabloid | 11 × 17 inches |

### Margin Formats

Margins can be specified in several ways:

```bash
# Single value (all sides)
html2pdf input.html -m 72

# With units
html2pdf input.html -m 1in
html2pdf input.html -m 25.4mm
html2pdf input.html -m 2.54cm

# Per-side (top,right,bottom,left)
html2pdf input.html -m "72,54,72,54"
```

**Supported units:**
- `pt` - Points (1/72 inch) - default
- `in` - Inches
- `mm` - Millimeters
- `cm` - Centimeters
- `px` - Pixels (converted at 96 DPI)

### Header and Footer Templates

Headers and footers can include special template variables:

| Variable | Description |
|----------|-------------|
| `{{page}}` | Current page number |
| `{{pages}}` | Total number of pages |
| `{{title}}` | Document title |
| `{{date}}` | Current date |
| `{{url}}` | Source URL (if applicable) |

**Example:**

```bash
# Inline HTML
html2pdf input.html -o output.pdf \
  --header "<h1>Document Header</h1>" \
  --footer "<p>Page {{page}} of {{pages}}</p>"

# From files
html2pdf input.html -o output.pdf \
  --header-file header.html \
  --footer-file footer.html
```

### Additional Stylesheets

Apply custom CSS to override document styles:

```bash
# Single stylesheet
html2pdf input.html -o output.pdf -s custom.css

# Multiple stylesheets (applied in order)
html2pdf input.html -o output.pdf -s reset.css -s print.css -s overrides.css
```

---

## Configuration

### Configuration File Format

Create a JSON configuration file:

```json
{
  "paper_size": "A4",
  "orientation": "portrait",
  "margin": 72,
  "debug_layout": false,
  "header": "<h1>Default Header</h1>",
  "footer": "<p>Page {{page}}</p>",
  "base_url": "https://example.com",
  "timeout_seconds": 30
}
```

### Using Configuration Files

```bash
# Use a configuration file
html2pdf input.html -c config.json -o output.pdf

# CLI options override config file settings
html2pdf input.html -c config.json -p Letter -o output.pdf
```

### Generating Default Config

```bash
html2pdf config > myconfig.json
```

Output:
```json
{
  "paper_size": "A4",
  "orientation": "portrait",
  "margin": 72,
  "debug_layout": false
}
```

---

## PrintCSS Guide

html2pdf fully supports the CSS Paged Media specification for professional document layouts.

### @page Rules

Define page properties using `@page` rules in your CSS:

```css
/* Default page settings */
@page {
  size: A4;
  margin: 2cm;
}

/* First page (title page) */
@page :first {
  margin-top: 5cm;
  @top-center { content: none; }
}

/* Left pages (verso) */
@page :left {
  margin-left: 3cm;
  margin-right: 2cm;
}

/* Right pages (recto) */
@page :right {
  margin-left: 2cm;
  margin-right: 3cm;
}
```

### Page Margin Boxes

Add content to page margins using margin boxes:

```css
@page {
  @top-center {
    content: "Document Title";
    font-size: 10pt;
    color: #666;
  }
  
  @bottom-center {
    content: counter(page);
    font-size: 10pt;
  }
  
  @bottom-right {
    content: "Page " counter(page) " of " counter(pages);
    font-size: 9pt;
  }
}
```

Available margin boxes:
- `@top-left-corner`, `@top-left`, `@top-center`, `@top-right`, `@top-right-corner`
- `@bottom-left-corner`, `@bottom-left`, `@bottom-center`, `@bottom-right`, `@bottom-right-corner`
- `@left-top`, `@left-middle`, `@left-bottom`
- `@right-top`, `@right-middle`, `@right-bottom`

### Page Break Control

```css
/* Force page break before */
.chapter {
  break-before: page;
}

/* Prevent page breaks inside */
.keep-together {
  break-inside: avoid;
}

/* Keep headings with following content */
h1, h2, h3 {
  break-after: avoid;
}

/* Control orphans and widows */
p {
  orphans: 3;   /* Minimum lines at bottom of page */
  widows: 3;    /* Minimum lines at top of page */
}
```

### Named Pages

Create different page layouts:

```css
.cover {
  page: cover;
}

@page cover {
  margin: 0;
  background: #2c3e50;
}

.content {
  page: content;
}

@page content {
  margin: 2.5cm 2cm;
}
```

For a comprehensive guide to PrintCSS, see [PRINTCSS_GUIDE.md](PRINTCSS_GUIDE.md).

---

## Examples

### Example 1: Basic Conversion

```bash
# Simple file conversion
html2pdf examples/basic.html -o output.pdf

# With custom paper size and landscape
html2pdf examples/basic.html -o output.pdf -p Letter -O landscape
```

### Example 2: Business Report with PrintCSS

Create `report.html`:

```html
<!DOCTYPE html>
<html>
<head>
  <style>
    @page {
      size: A4;
      margin: 2.5cm 2cm;
      
      @top-center {
        content: "Annual Report 2024";
        font-size: 9pt;
        color: #666;
      }
      
      @bottom-center {
        content: counter(page);
        font-size: 9pt;
      }
    }
    
    @page :first {
      margin-top: 5cm;
      @top-center { content: none; }
    }
    
    h1 { 
      color: #2c3e50;
      font-size: 28pt;
      break-after: avoid;
    }
    
    h2 {
      color: #34495e;
      font-size: 18pt;
      break-after: avoid;
    }
    
    .chapter {
      break-before: page;
    }
  </style>
</head>
<body>
  <h1>Annual Report 2024</h1>
  <p>Executive summary...</p>
  
  <div class="chapter">
    <h2>Financial Overview</h2>
    <p>Financial details...</p>
  </div>
  
  <div class="chapter">
    <h2>Market Analysis</h2>
    <p>Market details...</p>
  </div>
</body>
</html>
```

Convert:
```bash
html2pdf report.html -o report.pdf
```

### Example 3: Invoice with Custom Margins

```bash
html2pdf examples/invoice.html -o invoice.pdf -m 15mm
```

### Example 4: Using Pipes

```bash
# Generate HTML and convert to PDF in one pipeline
echo "<h1>Hello World</h1>" | html2pdf - -o output.pdf

# Process remote HTML through a filter
curl -s https://example.com | html2pdf - -o output.pdf

# Combine with other tools
pandoc document.md -t html | html2pdf - -o document.pdf
```

### Example 5: Batch Conversion

```bash
# Convert multiple HTML files
for file in *.html; do
  html2pdf "$file" -o "${file%.html}.pdf"
done
```

### Example 6: Complex Layout (Landscape)

```bash
# Dashboard-style layout in landscape
html2pdf examples/complex-layout.html -o dashboard.pdf -p A4 -O landscape
```

---

## Troubleshooting

### Common Issues

#### "Input file not found"

**Problem:** The specified HTML file doesn't exist.

**Solution:** Check the file path and ensure the file exists.

```bash
# Verify file exists
ls -la input.html

# Use absolute path if needed
html2pdf /full/path/to/input.html -o output.pdf
```

#### "PDF output is blank"

**Problem:** The HTML might not have proper styling or content.

**Solutions:**
1. Check that the HTML has visible content
2. Ensure CSS is properly linked or inline
3. Use `--debug-layout` to see layout information

```bash
html2pdf input.html -o output.pdf --debug-layout -v
```

#### "Margins appear incorrect"

**Problem:** Margin values might be misinterpreted.

**Solutions:**
1. Always specify units for clarity
2. Use comma-separated format for per-side margins

```bash
# Clear margin specification
html2pdf input.html -m "20mm,15mm,20mm,15mm"
```

#### "Images not appearing in PDF"

**Problem:** Image support is still in development.

**Solution:** Check the [Roadmap](../README.md#roadmap) for current status.

#### "Web fonts not working"

**Problem:** Web font support is still in development.

**Solution:** Use system fonts or embed fonts directly (when supported).

#### "Page breaks not working"

**Problem:** CSS properties might be incorrect.

**Solutions:**
1. Use `break-before`/`break-after` instead of legacy `page-break-*`
2. Ensure elements have `display: block` or similar

```css
/* Correct */
.chapter {
  break-before: page;
}

/* Legacy (also supported) */
.chapter {
  page-break-before: always;
}
```

### Debug Mode

Enable verbose output and layout debugging:

```bash
html2pdf input.html -o output.pdf -v --debug-layout
```

### Validation

Validate HTML/CSS without generating PDF:

```bash
html2pdf validate input.html
```

### Getting Help

```bash
# Show help
html2pdf --help

# Show version
html2pdf --version
```

### Reporting Issues

If you encounter a bug:

1. Run with verbose mode: `html2pdf -v input.html -o output.pdf`
2. Note your operating system and Rust version
3. Include a minimal example that reproduces the issue
4. Report at [GitHub Issues](https://github.com/yourusername/html2pdf-rs/issues)

---

## Related Documentation

- [CSS Support Reference](CSS_SUPPORT.md) - Complete CSS property and selector reference
- [PrintCSS Guide](PRINTCSS_GUIDE.md) - In-depth PrintCSS tutorial
- [API Guide](API_GUIDE.md) - Library usage for Rust developers
- [README.md](../README.md) - Project overview and quick reference

---

## License

This project is licensed under either MIT or Apache-2.0 license, at your option.
