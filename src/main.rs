//! HTML2PDF - A from-scratch Rust HTML to PDF converter
//!
//! This is the CLI entry point. The main library is in lib.rs.

use std::process;

fn main() {
    // Run the CLI application
    if let Err(e) = html2pdf::cli::run() {
        eprintln!("Error: {}", e);
        process::exit(1);
    }
}
