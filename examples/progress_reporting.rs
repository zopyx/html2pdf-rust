//! Example: Progress Reporting
//!
//! This example demonstrates how to use the progress reporting
//! and callback system in html2pdf.

use html2pdf::{
    html_to_pdf, Config, PaperSize, Orientation, Margins,
    progress::{ProgressCallback, ProgressStage, ProgressTracker, ConversionStats},
};

/// A custom progress handler that prints to stderr
struct ConsoleProgressHandler {
    verbose: bool,
}

impl ConsoleProgressHandler {
    fn new(verbose: bool) -> Self {
        Self { verbose }
    }
}

impl ProgressCallback for ConsoleProgressHandler {
    fn on_progress(&self, stage: ProgressStage, percent: f32, message: &str) -> bool {
        if self.verbose {
            eprintln!(
                "[{:>4}] {:>20} {:>5.1}% | {}",
                stage.code(),
                stage.name(),
                percent,
                message
            );
        } else {
            // Simple progress bar
            let width = 40;
            let filled = ((percent / 100.0) * width as f32) as usize;
            let bar: String = std::iter::repeat('=').take(filled)
                .chain(std::iter::repeat(' ').take(width - filled))
                .collect();
            eprint!("\r[{}] {}: {:>3.0}%", bar, stage.code(), percent);
            if percent >= 100.0 {
                eprintln!();
            }
        }
        true // Continue processing
    }

    fn on_warning(&self, message: &str) {
        eprintln!("⚠️  Warning: {}", message);
    }

    fn on_error(&self, error: &str) {
        eprintln!("❌ Error: {}", error);
    }

    fn on_stage_begin(&self, stage: ProgressStage) {
        eprintln!("▶️  Starting: {}", stage.name());
    }

    fn on_stage_end(&self, stage: ProgressStage) {
        eprintln!("✅ Completed: {}", stage.name());
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Example HTML document
    let html = r#"
<!DOCTYPE html>
<html>
<head>
    <title>Progress Reporting Example</title>
    <style>
        body { font-family: Arial, sans-serif; margin: 50px; }
        h1 { color: #333; }
        p { line-height: 1.6; }
    </style>
</head>
<body>
    <h1>Hello, PDF!</h1>
    <p>This document was generated with progress reporting enabled.</p>
    <p>Lorem ipsum dolor sit amet, consectetur adipiscing elit.</p>
    <ul>
        <li>Item 1</li>
        <li>Item 2</li>
        <li>Item 3</li>
    </ul>
</body>
</html>
"#;

    println!("=== Progress Reporting Example ===\n");

    // Create custom progress handler
    let handler = ConsoleProgressHandler::new(true);
    let tracker = ProgressTracker::new(handler);

    // Create configuration with progress tracking
    let config = Config::default()
        .with_paper_size(PaperSize::A4)
        .with_orientation(Orientation::Portrait)
        .with_margins(Margins::all(72.0))
        .with_progress(tracker.clone());

    println!("Converting HTML to PDF...\n");

    // Convert with progress tracking
    let pdf = html_to_pdf(html, &config)?;

    // Get final statistics
    let stats = tracker.stats();

    // Save PDF
    let output_path = "progress_example.pdf";
    std::fs::write(output_path, pdf)?;

    // Print summary
    println!("\n=== Conversion Summary ===");
    println!("Output: {}", output_path);
    println!("Input size: {} bytes", stats.input_bytes);
    println!("Output size: {} bytes", stats.output_bytes);
    println!("Elements processed: {}", stats.elements_processed);
    println!("Pages generated: {}", stats.pages_generated);
    println!("CSS rules parsed: {}", stats.css_rules_parsed);
    println!("Total time: {}", stats.format_total_time());

    // Print stage timings
    println!("\nStage Timings:");
    for stage in [
        ProgressStage::Loading,
        ProgressStage::ParsingHtml,
        ProgressStage::ParsingCss,
        ProgressStage::ComputingStyles,
        ProgressStage::BuildingLayout,
        ProgressStage::LayingOut,
        ProgressStage::Rendering,
        ProgressStage::WritingPdf,
    ] {
        let time = stats.stage_time(stage);
        if time.as_millis() > 0 {
            println!("  {}: {}ms", stage.name(), time.as_millis());
        }
    }

    println!("\n✓ Done!");

    Ok(())
}
