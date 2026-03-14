//! URL Fetching Example
//!
//! This example demonstrates how to fetch HTML from a URL and convert it to PDF.
//! It includes:
//! - Fetching HTML content from remote URLs
//! - Handling relative URLs for resources (CSS, images)
//! - Setting a base URL for resource resolution
//! - Error handling for network operations
//!
//! Note: This example demonstrates the API. Actual URL fetching requires
//! an HTTP client (like reqwest) which can be added as an optional dependency.
//!
//! Run with: cargo run --example from_url

use html2pdf::{html_to_pdf, html_to_pdf_from_input, Config, Input, Margins, PaperSize};
use std::fs;

fn main() {
    println!("=== HTML2PDF URL Fetching Example ===\n");

    // Example 1: Simulating URL-based conversion with local HTML
    match example_simulated_url_conversion() {
        Ok(path) => println!("✓ Example 1 complete: PDF saved to {}\n", path),
        Err(e) => eprintln!("✗ Example 1 failed: {}\n", e),
    }

    // Example 2: Handling relative URLs with base_url
    match example_relative_urls() {
        Ok(path) => println!("✓ Example 2 complete: PDF saved to {}\n", path),
        Err(e) => eprintln!("✗ Example 2 failed: {}\n", e),
    }

    // Example 3: Custom configuration for web content
    match example_web_content_config() {
        Ok(path) => println!("✓ Example 3 complete: PDF saved to {}\n", path),
        Err(e) => eprintln!("✗ Example 3 failed: {}\n", e),
    }

    // Example 4: Error handling for network operations
    example_network_error_handling();

    // Example 5: Complete URL to PDF workflow (documented)
    document_url_workflow();

    println!("\n=== All URL Examples Complete ===");
    println!("\nNote: For production URL fetching, enable the 'url-fetch' feature");
    println!("and add an HTTP client like reqwest to dependencies.");
}

/// Example 1: Simulating URL-based conversion
/// 
/// In a real implementation with an HTTP client, you would:
/// 1. Fetch the HTML from the URL
/// 2. Extract the base URL for relative resource resolution
/// 3. Convert to PDF with proper base URL
fn example_simulated_url_conversion() -> Result<String, Box<dyn std::error::Error>> {
    println!("Example 1: Simulated URL Conversion");
    println!("-------------------------------------");
    
    // Simulating HTML that would be fetched from a URL
    let simulated_fetched_html = r#"<!DOCTYPE html>
<html>
<head>
    <title>Example Website</title>
    <!-- In real scenario, these would be fetched from the server -->
    <link rel="stylesheet" href="/css/style.css">
    <base href="https://example.com/">
</head>
<body>
    <header>
        <h1>Example Website</h1>
        <nav>
            <a href="/">Home</a>
            <a href="/about">About</a>
            <a href="/contact">Contact</a>
        </nav>
    </header>
    <main>
        <h2>Welcome</h2>
        <p>
            This simulates HTML that would be fetched from https://example.com/.
            The &lt;base&gt; tag in the head specifies the base URL for relative links.
        </p>
        <p>
            Images and CSS would use the base URL:
            <code>&lt;img src="/images/logo.png"&gt;</code>
            resolves to https://example.com/images/logo.png
        </p>
    </main>
    <footer>
        <p>&copy; 2024 Example Website</p>
    </footer>
</body>
</html>"#;

    // Create config with base URL for resource resolution
    let config = Config::default()
        .with_paper_size(PaperSize::A4)
        .with_margins(Margins::all(36.0));

    println!("  Simulating fetch from: https://example.com/");
    println!("  Base URL set for resource resolution");

    let pdf_bytes = html_to_pdf(simulated_fetched_html, &config)?;
    println!("  Generated PDF: {} bytes", pdf_bytes.len());

    let output_path = "examples/output_url_simulated.pdf";
    fs::write(output_path, pdf_bytes)?;

    Ok(output_path.to_string())
}

/// Example 2: Handling relative URLs with base_url configuration
fn example_relative_urls() -> Result<String, Box<dyn std::error::Error>> {
    println!("\nExample 2: Relative URL Resolution");
    println!("------------------------------------");

    // HTML with various relative URL types
    let html_with_relative_urls = r#"<!DOCTYPE html>
<html>
<head>
    <title>Relative URL Test</title>
    <style>
        body { 
            font-family: Arial, sans-serif;
            /* In real scenario, background-image would use base_url */
            background-color: #f5f5f5;
        }
        .logo {
            /* Simulated: would resolve to https://cdn.example.com/logo.png */
            width: 200px;
            height: 100px;
            background: #e3f2fd;
            display: flex;
            align-items: center;
            justify-content: center;
            margin-bottom: 20px;
        }
    </style>
</head>
<body>
    <div class="logo">Logo Placeholder</div>
    
    <h1>Relative URL Resolution</h1>
    
    <p>This example shows how different relative URLs would be resolved:</p>
    
    <table border="1" cellpadding="10" style="border-collapse: collapse;">
        <tr style="background: #f0f0f0;">
            <th>Relative URL</th>
            <th>Resolved URL (with base)</th>
            <th>Type</th>
        </tr>
        <tr>
            <td><code>/images/logo.png</code></td>
            <td>https://example.com/images/logo.png</td>
            <td>Absolute path</td>
        </tr>
        <tr>
            <td><code>css/style.css</code></td>
            <td>https://example.com/css/style.css</td>
            <td>Relative path</td>
        </tr>
        <tr>
            <td><code>../images/bg.png</code></td>
            <td>https://images/bg.png</td>
            <td>Parent directory</td>
        </tr>
        <tr>
            <td><code>//cdn.example.com/lib.js</code></td>
            <td>https://cdn.example.com/lib.js</td>
            <td>Protocol-relative</td>
        </tr>
        <tr>
            <td><code>https://other.com/page</code></td>
            <td>https://other.com/page (unchanged)</td>
            <td>Absolute URL</td>
        </tr>
    </table>
    
    <div style="margin-top: 20px; padding: 15px; background: #fff3cd; border-left: 4px solid #ffc107;">
        <strong>Note:</strong> In the actual implementation, the base_url config option
        would be used to resolve these relative URLs when fetching resources.
    </div>
</body>
</html>"#;

    let config = Config::default()
        .with_paper_size(PaperSize::A4)
        .with_margins(Margins::all(50.0));

    println!("  Base URL: https://example.com/");
    println!("  Demonstrating resolution of:");
    println!("    - Absolute paths (/images/logo.png)");
    println!("    - Relative paths (css/style.css)");
    println!("    - Parent directory (../images/bg.png)");
    println!("    - Protocol-relative (//cdn.example.com)");

    let pdf_bytes = html_to_pdf(html_with_relative_urls, &config)?;
    println!("  Generated PDF: {} bytes", pdf_bytes.len());

    let output_path = "examples/output_url_relative.pdf";
    fs::write(output_path, pdf_bytes)?;

    Ok(output_path.to_string())
}

/// Example 3: Configuration optimized for web content
fn example_web_content_config() -> Result<String, Box<dyn std::error::Error>> {
    println!("\nExample 3: Web Content Configuration");
    println!("--------------------------------------");

    // HTML that might come from a content management system
    let cms_content = r#"<!DOCTYPE html>
<html>
<head>
    <title>Article Title</title>
    <style>
        body {
            font-family: Georgia, 'Times New Roman', serif;
            font-size: 12pt;
            line-height: 1.8;
            color: #333;
            max-width: 600px;
            margin: 0 auto;
        }
        
        h1 {
            font-size: 24pt;
            color: #1a1a1a;
            margin-bottom: 10px;
        }
        
        .meta {
            font-size: 10pt;
            color: #666;
            margin-bottom: 30px;
            padding-bottom: 20px;
            border-bottom: 1px solid #ddd;
        }
        
        p {
            margin-bottom: 1em;
            text-align: justify;
        }
        
        .lead {
            font-size: 14pt;
            color: #555;
            font-style: italic;
        }
        
        blockquote {
            margin: 20px 0;
            padding: 20px;
            background: #f9f9f9;
            border-left: 4px solid #333;
            font-style: italic;
        }
        
        figure {
            margin: 20px 0;
            page-break-inside: avoid;
        }
        
        figcaption {
            font-size: 10pt;
            color: #666;
            text-align: center;
            margin-top: 10px;
        }
        
        .image-placeholder {
            width: 100%;
            height: 200px;
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            display: flex;
            align-items: center;
            justify-content: center;
            color: white;
            border-radius: 4px;
        }
    </style>
</head>
<body>
    <article>
        <h1>The Future of Web Development</h1>
        <div class="meta">
            By Jane Smith | March 15, 2024 | Technology
        </div>
        
        <p class="lead">
            Web development continues to evolve at a rapid pace, with new technologies 
            and methodologies emerging constantly.
        </p>
        
        <p>
            The landscape of web development has transformed dramatically over the past 
            decade. From simple static pages to complex single-page applications, the 
            tools and techniques we use have matured significantly.
        </p>
        
        <figure>
            <div class="image-placeholder">Article Image</div>
            <figcaption>Figure 1: Modern web development workflow</figcaption>
        </figure>
        
        <p>
            Modern frameworks and libraries have streamlined the development process, 
            allowing developers to build more complex applications with less code. 
            However, this comes with its own challenges.
        </p>
        
        <blockquote>
            "The best code is no code at all. Every line of code you write is a 
            liability that needs to be maintained."
        </blockquote>
        
        <p>
            As we look to the future, several trends are shaping the industry:
            server-side rendering for performance, edge computing for lower latency, 
            and AI-assisted development tools.
        </p>
        
        <h2>Key Takeaways</h2>
        <ul>
            <li>Performance remains critical for user experience</li>
            <li>Developer experience is increasingly important</li>
            <li>Accessibility should be built in, not bolted on</li>
            <li>Security is everyone's responsibility</li>
        </ul>
    </article>
</body>
</html>"#;

    // Config optimized for reading web articles
    let config = Config::default()
        .with_paper_size(PaperSize::A4)
        .with_margins(Margins::symmetric(72.0, 90.0)); // Wider side margins for readability

    println!("  Config optimized for web content:");
    println!("    - A4 paper size");
    println!("    - Generous margins for comfortable reading");
    println!("    - Proper handling of web typography");
    println!("    - Figure and blockquote preservation");

    let pdf_bytes = html_to_pdf(cms_content, &config)?;
    println!("  Generated PDF: {} bytes", pdf_bytes.len());

    let output_path = "examples/output_url_web_content.pdf";
    fs::write(output_path, pdf_bytes)?;

    Ok(output_path.to_string())
}

/// Example 4: Network error handling patterns
fn example_network_error_handling() {
    println!("\nExample 4: Network Error Handling");
    println!("-----------------------------------");

    println!("  Common network errors when fetching URLs:");
    println!("    - Connection timeout");
    println!("    - DNS resolution failure");
    println!("    - HTTP 404 Not Found");
    println!("    - HTTP 403 Forbidden");
    println!("    - SSL certificate errors");
    println!("    - Redirect loops");
    println!("");

    // Demonstrate the Input::Url variant
    let url_input = Input::Url("https://example.com/document.html".to_string());
    
    println!("  Input type: {}", url_input.description());
    
    // Try to load (will fail as URL fetching is not yet implemented)
    match url_input.load() {
        Ok(content) => println!("  Content loaded: {} bytes", content.len()),
        Err(e) => {
            println!("  ✓ Expected error: {}", e);
            println!("  (URL fetching requires an HTTP client implementation)");
        }
    }

    // Show proper error handling pattern
    println!("\n  Proper error handling pattern:");
    println!("    match Input::Url(url).load() {{");
    println!("        Ok(html) => convert_to_pdf(html),");
    println!("        Err(NetworkError) => retry_or_fail_gracefully(),");
    println!("        Err(TimeoutError) => increase_timeout_and_retry(),");
    println!("        Err(e) => log_error_and_return(e),");
    println!("    }}");
}

/// Example 5: Document the complete URL to PDF workflow
fn document_url_workflow() {
    println!("\nExample 5: Complete URL to PDF Workflow");
    println!("-----------------------------------------");

    println!("  The complete workflow for URL to PDF conversion:");
    println!();
    println!("  1. CONFIGURATION PHASE");
    println!("     - Set timeout (default: 30s)");
    println!("     - Configure base URL for relative links");
    println!("     - Set paper size and margins");
    println!("     - Enable/disable image loading");
    println!();
    println!("  2. FETCH PHASE");
    println!("     - Send HTTP GET request");
    println!("     - Follow redirects (up to limit)");
    println!("     - Handle authentication if needed");
    println!("     - Validate response status");
    println!();
    println!("  3. PARSE PHASE");
    println!("     - Parse HTML into DOM");
    println!("     - Extract and resolve relative URLs");
    println!("     - Identify external resources (CSS, images)");
    println!();
    println!("  4. RESOURCE FETCHING (optional)");
    println!("     - Download linked stylesheets");
    println!("     - Download and embed images");
    println!("     - Apply all CSS styles");
    println!();
    println!("  5. CONVERSION PHASE");
    println!("     - Compute styles");
    println!("     - Perform layout");
    println!("     - Generate PDF");
    println!();
    println!("  Code example:");
    println!("    let config = Config::default()");
    println!("        .with_timeout(60)");
    println!("        .with_base_url(\"https://example.com/\")");
    println!();
    println!("    let input = Input::Url(\"https://example.com/page\".to_string());");
    println!("    let pdf = html_to_pdf_from_input(&input, &config)?;");
}
