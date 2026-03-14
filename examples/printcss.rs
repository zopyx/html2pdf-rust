//! PrintCSS / CSS Paged Media Example
//!
//! This example demonstrates the CSS Paged Media features supported by html2pdf,
//! including:
//! - @page rules for page setup
//! - Page size and orientation
//! - Page margins and margin boxes
//! - Running headers and footers
//! - Page breaks and pagination control
//! - Named pages
//!
//! Run with: cargo run --example printcss

use html2pdf::{html_to_pdf, Config, Input, Margins, Orientation, PaperSize};
use std::fs;

fn main() {
    println!("=== HTML2PDF PrintCSS Example ===\n");

    // Example 1: Basic @page rules
    match example_basic_page_rules() {
        Ok(path) => println!("✓ Example 1 complete: PDF saved to {}\n", path),
        Err(e) => eprintln!("✗ Example 1 failed: {}\n", e),
    }

    // Example 2: Headers and footers with page numbers
    match example_headers_footers() {
        Ok(path) => println!("✓ Example 2 complete: PDF saved to {}\n", path),
        Err(e) => eprintln!("✗ Example 2 failed: {}\n", e),
    }

    // Example 3: Multiple page sizes (named pages)
    match example_named_pages() {
        Ok(path) => println!("✓ Example 3 complete: PDF saved to {}\n", path),
        Err(e) => eprintln!("✗ Example 3 failed: {}\n", e),
    }

    // Example 4: Complex document with page breaks
    match example_page_breaks() {
        Ok(path) => println!("✓ Example 4 complete: PDF saved to {}\n", path),
        Err(e) => eprintln!("✗ Example 4 failed: {}\n", e),
    }

    // Example 5: Landscape pages in portrait document
    match example_mixed_orientation() {
        Ok(path) => println!("✓ Example 5 complete: PDF saved to {}\n", path),
        Err(e) => eprintln!("✗ Example 5 failed: {}\n", e),
    }

    println!("\n=== All PrintCSS Examples Complete ===");
    println!("Open the generated PDFs to see the PrintCSS features in action.");
}

/// Example 1: Basic @page rules with margins and page size
fn example_basic_page_rules() -> Result<String, Box<dyn std::error::Error>> {
    println!("Example 1: Basic @page Rules");
    println!("------------------------------");

    let html = r#"<!DOCTYPE html>
<html>
<head>
    <title>Basic Page Rules</title>
    <style>
        /* Define page properties using @page */
        @page {
            /* Page size: A4 (default) */
            size: A4;
            
            /* Margins: top right bottom left */
            margin: 2cm 1.5cm 2cm 1.5cm;
        }
        
        body {
            font-family: Georgia, serif;
            font-size: 12pt;
            line-height: 1.6;
        }
        
        h1 {
            color: #2c3e50;
            border-bottom: 2px solid #3498db;
            padding-bottom: 0.3em;
        }
        
        .info-box {
            background: #e8f4f8;
            border-left: 4px solid #3498db;
            padding: 1em;
            margin: 1em 0;
        }
    </style>
</head>
<body>
    <h1>Basic @page Rules</h1>
    <div class="info-box">
        <strong>Page Settings:</strong>
        <ul>
            <li>Size: A4</li>
            <li>Top margin: 2cm</li>
            <li>Right margin: 1.5cm</li>
            <li>Bottom margin: 2cm</li>
            <li>Left margin: 1.5cm</li>
        </ul>
    </div>
    <p>
        This document demonstrates basic @page rules. The margins are set 
        asymmetrically - wider at the top and bottom for better readability.
    </p>
    <p>
        The @page rule is the foundation of CSS Paged Media. It allows you 
        to control the page box, which is the rectangular area that contains 
        the page content.
    </p>
</body>
</html>"#;

    let config = Config::default()
        .with_paper_size(PaperSize::A4)
        .with_margins(Margins::new(56.7, 42.5, 56.7, 42.5)); // 2cm, 1.5cm in points

    println!("  Config: A4 with asymmetric margins (2cm/1.5cm)");

    let pdf_bytes = html_to_pdf(html, &config)?;
    println!("  Generated PDF: {} bytes", pdf_bytes.len());

    let output_path = "examples/output_printcss_basic.pdf";
    fs::write(output_path, pdf_bytes)?;

    Ok(output_path.to_string())
}

/// Example 2: Headers and footers with page numbers
fn example_headers_footers() -> Result<String, Box<dyn std::error::Error>> {
    println!("\nExample 2: Headers and Footers");
    println!("--------------------------------");

    let html = r#"<!DOCTYPE html>
<html>
<head>
    <title>Headers and Footers</title>
    <style>
        /* Page with header and footer */
        @page {
            size: A4;
            margin: 3cm 2cm 3cm 2cm;
            
            /* Running header at top center */
            @top-center {
                content: "Annual Report 2024";
                font-family: Arial, sans-serif;
                font-size: 9pt;
                color: #666;
            }
            
            /* Page number at bottom center */
            @bottom-center {
                content: "Page " counter(page) " of " counter(pages);
                font-family: Arial, sans-serif;
                font-size: 9pt;
            }
            
            /* Confidential notice at bottom right */
            @bottom-right {
                content: "Confidential";
                font-family: Arial, sans-serif;
                font-size: 8pt;
                font-style: italic;
                color: #999;
            }
            
            /* Date at bottom left */
            @bottom-left {
                content: "Generated: March 2024";
                font-family: Arial, sans-serif;
                font-size: 8pt;
                color: #999;
            }
        }
        
        /* First page has no header/footer */
        @page :first {
            @top-center { content: none; }
            @bottom-center { content: none; }
            @bottom-right { content: none; }
            @bottom-left { content: none; }
        }
        
        body {
            font-family: Georgia, serif;
            font-size: 11pt;
            line-height: 1.6;
        }
        
        h1 {
            color: #2c3e50;
            margin-top: 0;
        }
        
        .cover {
            text-align: center;
            padding-top: 200px;
        }
        
        .cover h1 {
            font-size: 36pt;
            margin-bottom: 20px;
        }
        
        .cover p {
            font-size: 14pt;
            color: #666;
        }
        
        .content {
            page-break-before: always;
        }
        
        h2 {
            color: #34495e;
            border-bottom: 1px solid #bdc3c7;
            padding-bottom: 0.2em;
            page-break-after: avoid;
        }
    </style>
</head>
<body>
    <!-- Cover page (no header/footer due to :first) -->
    <div class="cover">
        <h1>Annual Report</h1>
        <p>Fiscal Year 2024</p>
        <p style="margin-top: 100px;">Company Name</p>
    </div>
    
    <!-- Content pages (with headers/footers) -->
    <div class="content">
        <h2>Executive Summary</h2>
        <p>
            This document demonstrates running headers and footers using CSS Paged Media. 
            Notice how each page has a consistent header showing the document title, and 
            the footer shows the page number and total pages.
        </p>
        <p>
            The first page (this cover) has no header or footer, achieved using the 
            @page :first pseudo-class selector.
        </p>
        <p>
            Lorem ipsum dolor sit amet, consectetur adipiscing elit. Sed do eiusmod 
            tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, 
            quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat.
        </p>
        
        <h2>Page Break Control</h2>
        <p>
            The heading above uses page-break-after: avoid to prevent it from appearing 
            alone at the bottom of a page. This is part of the widows and orphans control 
            in CSS Paged Media.
        </p>
        <p>
            Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore 
            eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt 
            in culpa qui officia deserunt mollit anim id est laborum.
        </p>
    </div>
</body>
</html>"#;

    let config = Config::default()
        .with_paper_size(PaperSize::A4)
        .with_margins(Margins::new(85.0, 56.7, 85.0, 56.7)); // 3cm, 2cm in points

    println!("  Config: A4 with margin boxes for headers/footers");
    println!("  Features: @top-center, @bottom-center, @bottom-left, @bottom-right");

    let pdf_bytes = html_to_pdf(html, &config)?;
    println!("  Generated PDF: {} bytes", pdf_bytes.len());

    let output_path = "examples/output_printcss_headers_footers.pdf";
    fs::write(output_path, pdf_bytes)?;

    Ok(output_path.to_string())
}

/// Example 3: Named pages for different page layouts
fn example_named_pages() -> Result<String, Box<dyn std::error::Error>> {
    println!("\nExample 3: Named Pages");
    println!("------------------------");

    let html = r#"<!DOCTYPE html>
<html>
<head>
    <title>Named Pages</title>
    <style>
        /* Default page */
        @page {
            size: A4;
            margin: 2cm;
            
            @bottom-center {
                content: "Standard Page - " counter(page);
                font-size: 9pt;
                color: #666;
            }
        }
        
        /* Cover page - full bleed, no margins */
        @page cover {
            margin: 0;
            background-color: #2c3e50;
            
            @bottom-center { content: none; }
        }
        
        /* Wide page for tables/charts */
        @page wide {
            size: A4 landscape;
            margin: 1.5cm;
            
            @bottom-center {
                content: "Landscape Page - " counter(page);
                font-size: 9pt;
                color: #666;
            }
        }
        
        body {
            font-family: Arial, sans-serif;
            font-size: 11pt;
            line-height: 1.5;
        }
        
        /* Apply named pages */
        .cover-page {
            page: cover;
            height: 100vh;
            display: flex;
            flex-direction: column;
            justify-content: center;
            align-items: center;
            color: white;
            text-align: center;
        }
        
        .cover-page h1 {
            font-size: 48pt;
            margin: 0;
        }
        
        .cover-page p {
            font-size: 18pt;
            opacity: 0.8;
        }
        
        .content-page {
            page: standard;
        }
        
        .wide-page {
            page: wide;
            page-break-before: always;
        }
        
        table {
            width: 100%;
            border-collapse: collapse;
        }
        
        th, td {
            border: 1px solid #ddd;
            padding: 8px;
            text-align: left;
        }
        
        th {
            background-color: #f2f2f2;
        }
    </style>
</head>
<body>
    <!-- Cover page with named page "cover" -->
    <div class="cover-page">
        <h1>Named Pages Demo</h1>
        <p>Different page layouts in one document</p>
    </div>
    
    <!-- Standard content page -->
    <div class="content-page">
        <h1>Standard Portrait Page</h1>
        <p>
            This is a standard A4 portrait page. Named pages allow you to have 
            different page layouts within the same document.
        </p>
        <p>
            The cover page used the "cover" named page with no margins and a 
            background color. This page uses the default settings.
        </p>
    </div>
    
    <!-- Wide landscape page -->
    <div class="wide-page">
        <h1>Landscape Page for Wide Content</h1>
        <p>
            This page uses the "wide" named page with A4 landscape orientation, 
            perfect for wide tables or charts.
        </p>
        <table>
            <tr>
                <th>Column A</th>
                <th>Column B</th>
                <th>Column C</th>
                <th>Column D</th>
                <th>Column E</th>
                <th>Column F</th>
            </tr>
            <tr>
                <td>Data 1A</td>
                <td>Data 1B</td>
                <td>Data 1C</td>
                <td>Data 1D</td>
                <td>Data 1E</td>
                <td>Data 1F</td>
            </tr>
            <tr>
                <td>Data 2A</td>
                <td>Data 2B</td>
                <td>Data 2C</td>
                <td>Data 2D</td>
                <td>Data 2E</td>
                <td>Data 2F</td>
            </tr>
        </table>
    </div>
</body>
</html>"#;

    // Note: For a real implementation, you'd use the Input with a stylesheet
    // that defines the named pages, and the layout engine would handle them
    let config = Config::default()
        .with_paper_size(PaperSize::A4)
        .with_orientation(Orientation::Portrait)
        .with_margins(Margins::all(56.7)); // 2cm

    println!("  Config: Mixed portrait and landscape using named pages");
    println!("  Features: @page cover, @page wide, page: name");

    let pdf_bytes = html_to_pdf(html, &config)?;
    println!("  Generated PDF: {} bytes", pdf_bytes.len());

    let output_path = "examples/output_printcss_named_pages.pdf";
    fs::write(output_path, pdf_bytes)?;

    Ok(output_path.to_string())
}

/// Example 4: Page breaks and pagination control
fn example_page_breaks() -> Result<String, Box<dyn std::error::Error>> {
    println!("\nExample 4: Page Breaks and Pagination");
    println!("---------------------------------------");

    let html = r#"<!DOCTYPE html>
<html>
<head>
    <title>Page Breaks</title>
    <style>
        @page {
            size: A4;
            margin: 2.5cm 2cm;
            
            @bottom-center {
                content: counter(page);
                font-size: 10pt;
            }
        }
        
        body {
            font-family: Georgia, serif;
            font-size: 11pt;
            line-height: 1.6;
        }
        
        h1 {
            color: #2c3e50;
        }
        
        h2 {
            color: #34495e;
            page-break-after: avoid;
        }
        
        h3 {
            color: #555;
            page-break-after: avoid;
        }
        
        /* Force page break before chapters */
        .chapter {
            page-break-before: always;
        }
        
        .chapter:first-of-type {
            page-break-before: auto;
        }
        
        /* Prevent breaking inside important boxes */
        .info-box {
            background: #e3f2fd;
            border-left: 4px solid #2196f3;
            padding: 1em;
            margin: 1em 0;
            page-break-inside: avoid;
        }
        
        /* Keep table rows together */
        tr {
            page-break-inside: avoid;
        }
        
        thead {
            display: table-header-group;
        }
        
        /* Widows and orphans control */
        p {
            orphans: 3;
            widows: 3;
        }
        
        table {
            width: 100%;
            border-collapse: collapse;
            margin: 1em 0;
        }
        
        th, td {
            border: 1px solid #ddd;
            padding: 8px;
            text-align: left;
        }
        
        th {
            background-color: #f2f2f2;
        }
    </style>
</head>
<body>
    <h1>Page Break Control</h1>
    <p>
        This document demonstrates various page break control mechanisms 
        available in CSS Paged Media.
    </p>
    
    <div class="info-box">
        <strong>Page Break Properties:</strong>
        <ul>
            <li><code>page-break-before</code> - Force/avoid break before element</li>
            <li><code>page-break-after</code> - Force/avoid break after element</li>
            <li><code>page-break-inside</code> - Control breaking inside element</li>
            <li><code>widows</code> - Minimum lines at bottom of page</li>
            <li><code>orphans</code> - Minimum lines at top of page</li>
        </ul>
    </div>
    
    <section class="chapter">
        <h2>Chapter 1: Introduction</h2>
        <p>
            Chapters use <code>page-break-before: always</code> to start on a new page. 
            However, the first chapter overrides this to avoid an empty first page.
        </p>
        <p>
            Headings use <code>page-break-after: avoid</code> to prevent them from 
            appearing alone at the bottom of a page without content following them.
        </p>
    </section>
    
    <section class="chapter">
        <h2>Chapter 2: Tables</h2>
        <p>
            Tables use multiple break control properties:
        </p>
        <ul>
            <li>Table rows have <code>page-break-inside: avoid</code></li>
            <li>Table header is repeated on each page with <code>display: table-header-group</code></li>
        </ul>
        <table>
            <thead>
                <tr>
                    <th>Item</th>
                    <th>Description</th>
                    <th>Status</th>
                </tr>
            </thead>
            <tbody>
                <tr>
                    <td>Item 1</td>
                    <td>First example item in the table</td>
                    <td>Active</td>
                </tr>
                <tr>
                    <td>Item 2</td>
                    <td>Second example item in the table</td>
                    <td>Pending</td>
                </tr>
                <tr>
                    <td>Item 3</td>
                    <td>Third example item in the table</td>
                    <td>Complete</td>
                </tr>
            </tbody>
        </table>
    </section>
    
    <section class="chapter">
        <h2>Chapter 3: Content Control</h2>
        <div class="info-box">
            <strong>Important Box</strong>
            <p>
                This box uses <code>page-break-inside: avoid</code> to ensure it 
                stays together on one page. This is useful for callouts, warnings, 
                and important notes.
            </p>
        </div>
        <p>
            Paragraphs have <code>orphans: 3</code> and <code>widows: 3</code> to 
            ensure at least 3 lines appear on each page when a paragraph breaks 
            across pages.
        </p>
    </section>
</body>
</html>"#;

    let config = Config::default()
        .with_paper_size(PaperSize::A4)
        .with_margins(Margins::new(70.9, 56.7, 70.9, 56.7)); // 2.5cm, 2cm

    println!("  Config: A4 with pagination controls");
    println!("  Features: page-break-before, page-break-after, page-break-inside, orphans, widows");

    let pdf_bytes = html_to_pdf(html, &config)?;
    println!("  Generated PDF: {} bytes", pdf_bytes.len());

    let output_path = "examples/output_printcss_page_breaks.pdf";
    fs::write(output_path, pdf_bytes)?;

    Ok(output_path.to_string())
}

/// Example 5: Mixed orientation within a document
fn example_mixed_orientation() -> Result<String, Box<dyn std::error::Error>> {
    println!("\nExample 5: Mixed Orientation");
    println!("------------------------------");

    let html = r#"<!DOCTYPE html>
<html>
<head>
    <title>Mixed Orientation</title>
    <style>
        /* Default portrait */
        @page {
            size: A4 portrait;
            margin: 2cm;
        }
        
        /* Landscape for wide content */
        @page landscape {
            size: A4 landscape;
            margin: 1.5cm;
        }
        
        body {
            font-family: Arial, sans-serif;
            font-size: 11pt;
        }
        
        .portrait-content {
            page: default;
        }
        
        .landscape-content {
            page: landscape;
            page-break-before: always;
        }
        
        .chart-placeholder {
            width: 100%;
            height: 400px;
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            display: flex;
            align-items: center;
            justify-content: center;
            color: white;
            font-size: 24px;
            border-radius: 8px;
        }
    </style>
</head>
<body>
    <div class="portrait-content">
        <h1>Portrait Section</h1>
        <p>
            This content appears on a portrait page (A4 portrait). The default 
            @page rule sets the orientation to portrait.
        </p>
        <p>
            Most documents are primarily portrait, with occasional landscape pages 
            for wide content like charts, tables, or diagrams.
        </p>
    </div>
    
    <div class="landscape-content">
        <h1>Landscape Section</h1>
        <p>
            This content appears on a landscape page (A4 landscape), perfect for 
            wide visualizations or data tables.
        </p>
        <div class="chart-placeholder">
            Wide Chart / Diagram Area
        </div>
        <p style="margin-top: 20px;">
            The landscape page provides 297mm of width compared to 210mm in portrait, 
            giving 41% more horizontal space for content.
        </p>
    </div>
    
    <div class="portrait-content">
        <h1>Back to Portrait</h1>
        <p>
            The document returns to portrait orientation. The page counter continues 
            sequentially across orientation changes.
        </p>
    </div>
</body>
</html>"#;

    let config = Config::default()
        .with_paper_size(PaperSize::A4)
        .with_orientation(Orientation::Portrait);

    println!("  Config: A4 with mixed portrait and landscape sections");
    println!("  Features: @page landscape, page: landscape");

    let pdf_bytes = html_to_pdf(html, &config)?;
    println!("  Generated PDF: {} bytes", pdf_bytes.len());

    let output_path = "examples/output_printcss_mixed_orientation.pdf";
    fs::write(output_path, pdf_bytes)?;

    Ok(output_path.to_string())
}
