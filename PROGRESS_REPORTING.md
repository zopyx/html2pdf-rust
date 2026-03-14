# Progress Reporting and Callbacks

This document describes the progress reporting and callback system added to html2pdf-rs.

## Overview

The progress reporting system provides:

- **ProgressStage enum**: Tracks the current stage of PDF conversion
- **ProgressCallback trait**: Receives progress updates, warnings, and errors
- **ProgressTracker**: Collects statistics and manages progress state
- **CLI Progress Bar**: Visual progress indication using indicatif
- **Cancellation Support**: Request conversion cancellation via callback

## Progress Stages

The conversion process is divided into these stages:

1. **Loading** - Loading input (file, URL, or string)
2. **ParsingHtml** - Parsing HTML into DOM
3. **ExecutingScripts** - Executing JavaScript (optional)
4. **ParsingCss** - Parsing CSS stylesheets
5. **ComputingStyles** - Computing styles (cascade, inheritance)
6. **BuildingLayout** - Building the layout box tree
7. **LayingOut** - Performing layout computation
8. **Rendering** - Rendering to PDF pages
9. **WritingPdf** - Writing PDF to output
10. **Complete** - Conversion complete

## Usage

### Basic Progress Callback

```rust
use html2pdf::{
    html_to_pdf, Config,
    progress::{ProgressCallback, ProgressStage, ProgressTracker},
};

struct MyProgressHandler;

impl ProgressCallback for MyProgressHandler {
    fn on_progress(&self, stage: ProgressStage, percent: f32, message: &str) -> bool {
        println!("[{}] {:.0}%: {}", stage.code(), percent, message);
        true // Return false to cancel
    }

    fn on_warning(&self, message: &str) {
        eprintln!("Warning: {}", message);
    }

    fn on_error(&self, error: &str) {
        eprintln!("Error: {}", error);
    }
}

fn main() -> html2pdf::Result<()> {
    let handler = MyProgressHandler;
    let tracker = ProgressTracker::new(handler);
    
    let config = Config::default()
        .with_progress(tracker);
    
    let pdf = html_to_pdf("<h1>Hello</h1>", &config)?;
    Ok(())
}
```

### Using Closure Callback

```rust
use html2pdf::{
    html_to_pdf, Config,
    progress::{ClosureProgressCallback, ProgressTracker},
};

let callback = ClosureProgressCallback::new(|stage, percent, message| {
    println!("{:?}: {}% - {}", stage, percent, message);
    true
});

let tracker = ProgressTracker::new(callback);
let config = Config::default().with_progress(tracker);
```

### CLI with Progress Bar

When using the CLI with the `progress` feature enabled:

```bash
# Show progress bar
html2pdf input.html -o output.pdf --verbose

# With detailed stage information
html2pdf input.html -o output.pdf -v
```

## Statistics

The `ProgressTracker` collects detailed statistics:

```rust
let stats = tracker.stats();

println!("Elements processed: {}", stats.elements_processed);
println!("CSS rules parsed: {}", stats.css_rules_parsed);
println!("Pages generated: {}", stats.pages_generated);
println!("Total time: {}", stats.format_total_time());
```

## Features

### Cargo.toml

```toml
[features]
default = ["progress"]
progress = ["indicatif", "sysinfo"]
```

### Without Progress Feature

```bash
cargo build --no-default-features
```

The library works without the progress feature - progress tracking becomes a no-op.

## API Reference

### ProgressStage

- `name()` - Human-readable stage name
- `code()` - Short 3-4 character code
- `weight()` - Relative time weight for this stage
- `next()` / `previous()` - Navigate between stages
- `all_stages()` - Get all stages in order

### ProgressCallback

- `on_progress(stage, percent, message) -> bool` - Called on progress updates
- `on_warning(message)` - Called when a warning occurs
- `on_error(error)` - Called when an error occurs
- `on_stage_begin(stage)` - Called when a stage starts
- `on_stage_end(stage)` - Called when a stage ends

### ProgressTracker

- `new(callback)` - Create with a callback
- `noop()` - Create with no-op callback
- `report_progress(percent, message)` - Report progress
- `report_items(current, total, item_name)` - Report item progress
- `begin_stage(stage)` - Start a new stage
- `warning(message)` / `error(message)` - Report issues
- `is_cancelled()` - Check if cancellation requested
- `update_stats(f)` - Update statistics
- `stats()` - Get current statistics
- `item_counter(total, item_name)` - Create an item counter

## Examples

See `examples/progress_reporting.rs` for a complete working example.

## Thread Safety

All progress callbacks must implement `Send + Sync` for safe use across threads.

## Cancellation

Return `false` from `on_progress` to request cancellation:

```rust
impl ProgressCallback for MyCallback {
    fn on_progress(&self, stage: ProgressStage, percent: f32, message: &str) -> bool {
        if should_cancel() {
            return false; // Cancel conversion
        }
        true
    }
}
```
