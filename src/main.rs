//! HTML2PDF - A from-scratch Rust HTML to PDF converter
//!
//! This is the CLI entry point. The main library is in lib.rs.

use std::process;

fn main() {
    // Run the CLI application
    if let Err(e) = html2pdf::cli::run() {
        // Check if it's our error type with pretty formatting
        if let Some(html2pdf_err) = e.downcast_ref::<html2pdf::Html2PdfError>() {
            eprintln!("{}", html2pdf_err.format_pretty());
            process::exit(html2pdf_err.exit_code());
        }
        
        // Fallback for other error types
        eprintln!("\n❌ Error: {}", e);
        
        // Try to get the source chain for verbose output
        let mut source = e.source();
        while let Some(s) = source {
            eprintln!("   Caused by: {}", s);
            source = s.source();
        }
        
        process::exit(1);
    }
}
