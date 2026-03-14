//! Progress reporting and callbacks for HTML2PDF
//!
//! This module provides progress tracking capabilities for the HTML to PDF
//! conversion process, including stage-based progress, statistics collection,
//! and callback support for both synchronous and asynchronous use cases.
//!
//! # Example
//!
//! ```rust,no_run
//! use html2pdf::progress::{ProgressStage, ProgressCallback, ProgressTracker, ConversionStats};
//!
//! struct MyProgressHandler;
//!
//! impl ProgressCallback for MyProgressHandler {
//!     fn on_progress(&self, stage: ProgressStage, percent: f32, message: &str) -> bool {
//!         println!("[{:?}] {}%: {}", stage, percent, message);
//!         true // Continue processing
//!     }
//!
//!     fn on_warning(&self, message: &str) {
//!         eprintln!("Warning: {}", message);
//!     }
//!
//!     fn on_error(&self, error: &str) {
//!         eprintln!("Error: {}", error);
//!     }
//! }
//! ```

use std::collections::HashMap;
use std::fmt;
use std::sync::{Arc, Mutex, RwLock};
use std::time::{Duration, Instant};

/// Represents a stage in the PDF conversion process
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ProgressStage {
    /// Loading input (file, URL, or string)
    Loading,
    /// Parsing HTML into DOM
    ParsingHtml,
    /// Executing JavaScript (optional)
    ExecutingScripts,
    /// Parsing CSS stylesheets
    ParsingCss,
    /// Computing styles (cascade, inheritance)
    ComputingStyles,
    /// Building the layout box tree
    BuildingLayout,
    /// Performing layout computation
    LayingOut,
    /// Rendering to PDF pages
    Rendering,
    /// Writing PDF to output
    WritingPdf,
    /// Conversion complete
    Complete,
}

impl ProgressStage {
    /// Get a human-readable name for this stage
    pub fn name(&self) -> &'static str {
        match self {
            ProgressStage::Loading => "Loading",
            ProgressStage::ParsingHtml => "Parsing HTML",
            ProgressStage::ExecutingScripts => "Executing Scripts",
            ProgressStage::ParsingCss => "Parsing CSS",
            ProgressStage::ComputingStyles => "Computing Styles",
            ProgressStage::BuildingLayout => "Building Layout Tree",
            ProgressStage::LayingOut => "Layout",
            ProgressStage::Rendering => "Rendering",
            ProgressStage::WritingPdf => "Writing PDF",
            ProgressStage::Complete => "Complete",
        }
    }

    /// Get a short (3-4 character) code for this stage
    pub fn code(&self) -> &'static str {
        match self {
            ProgressStage::Loading => "LOAD",
            ProgressStage::ParsingHtml => "HTML",
            ProgressStage::ExecutingScripts => "JS",
            ProgressStage::ParsingCss => "CSS",
            ProgressStage::ComputingStyles => "STYL",
            ProgressStage::BuildingLayout => "BLDT",
            ProgressStage::LayingOut => "LAYO",
            ProgressStage::Rendering => "REND",
            ProgressStage::WritingPdf => "WRIT",
            ProgressStage::Complete => "DONE",
        }
    }

    /// Get the typical weight (relative time) for this stage
    pub fn weight(&self) -> f32 {
        match self {
            ProgressStage::Loading => 0.05,
            ProgressStage::ParsingHtml => 0.15,
            ProgressStage::ExecutingScripts => 0.10,
            ProgressStage::ParsingCss => 0.10,
            ProgressStage::ComputingStyles => 0.10,
            ProgressStage::BuildingLayout => 0.10,
            ProgressStage::LayingOut => 0.20,
            ProgressStage::Rendering => 0.15,
            ProgressStage::WritingPdf => 0.05,
            ProgressStage::Complete => 0.0,
        }
    }

    /// Returns true if this is a terminal stage
    pub fn is_terminal(&self) -> bool {
        matches!(self, ProgressStage::Complete)
    }

    /// Get all stages in order
    pub fn all_stages() -> Vec<ProgressStage> {
        vec![
            ProgressStage::Loading,
            ProgressStage::ParsingHtml,
            ProgressStage::ExecutingScripts,
            ProgressStage::ParsingCss,
            ProgressStage::ComputingStyles,
            ProgressStage::BuildingLayout,
            ProgressStage::LayingOut,
            ProgressStage::Rendering,
            ProgressStage::WritingPdf,
            ProgressStage::Complete,
        ]
    }

    /// Get the next stage
    pub fn next(&self) -> Option<ProgressStage> {
        match self {
            ProgressStage::Loading => Some(ProgressStage::ParsingHtml),
            ProgressStage::ParsingHtml => Some(ProgressStage::ExecutingScripts),
            ProgressStage::ExecutingScripts => Some(ProgressStage::ParsingCss),
            ProgressStage::ParsingCss => Some(ProgressStage::ComputingStyles),
            ProgressStage::ComputingStyles => Some(ProgressStage::BuildingLayout),
            ProgressStage::BuildingLayout => Some(ProgressStage::LayingOut),
            ProgressStage::LayingOut => Some(ProgressStage::Rendering),
            ProgressStage::Rendering => Some(ProgressStage::WritingPdf),
            ProgressStage::WritingPdf => Some(ProgressStage::Complete),
            ProgressStage::Complete => None,
        }
    }

    /// Get the previous stage
    pub fn previous(&self) -> Option<ProgressStage> {
        match self {
            ProgressStage::Loading => None,
            ProgressStage::ParsingHtml => Some(ProgressStage::Loading),
            ProgressStage::ExecutingScripts => Some(ProgressStage::ParsingHtml),
            ProgressStage::ParsingCss => Some(ProgressStage::ExecutingScripts),
            ProgressStage::ComputingStyles => Some(ProgressStage::ParsingCss),
            ProgressStage::BuildingLayout => Some(ProgressStage::ComputingStyles),
            ProgressStage::LayingOut => Some(ProgressStage::BuildingLayout),
            ProgressStage::Rendering => Some(ProgressStage::LayingOut),
            ProgressStage::WritingPdf => Some(ProgressStage::Rendering),
            ProgressStage::Complete => Some(ProgressStage::WritingPdf),
        }
    }
}

impl fmt::Display for ProgressStage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

/// Callback trait for progress reporting
///
/// Implement this trait to receive progress updates during PDF conversion.
/// All methods are `Send + Sync` for safe use across threads.
///
/// Return `false` from `on_progress` to request cancellation.
pub trait ProgressCallback: Send + Sync {
    /// Called when progress is updated
    ///
    /// # Arguments
    ///
    /// * `stage` - The current conversion stage
    /// * `percent` - Progress percentage within this stage (0.0 - 100.0)
    /// * `message` - Human-readable status message
    ///
    /// # Returns
    ///
    /// `true` to continue processing, `false` to cancel
    fn on_progress(&self, stage: ProgressStage, percent: f32, message: &str) -> bool;

    /// Called when a warning occurs
    fn on_warning(&self, message: &str);

    /// Called when an error occurs
    fn on_error(&self, error: &str);

    /// Called when a stage begins
    fn on_stage_begin(&self, stage: ProgressStage) {
        let _ = self.on_progress(stage, 0.0, &format!("Starting {}", stage.name()));
    }

    /// Called when a stage completes
    fn on_stage_end(&self, stage: ProgressStage) {
        let _ = self.on_progress(stage, 100.0, &format!("Completed {}", stage.name()));
    }
}

/// A no-op progress callback (default)
pub struct NoOpProgressCallback;

impl ProgressCallback for NoOpProgressCallback {
    fn on_progress(&self, _stage: ProgressStage, _percent: f32, _message: &str) -> bool {
        true
    }

    fn on_warning(&self, _message: &str) {}

    fn on_error(&self, _error: &str) {}
}

/// Statistics collected during conversion
#[derive(Debug, Clone, Default)]
pub struct ConversionStats {
    /// Number of HTML elements processed
    pub elements_processed: usize,
    /// Number of CSS rules parsed
    pub css_rules_parsed: usize,
    /// Number of layout boxes created
    pub layout_boxes_created: usize,
    /// Number of pages generated
    pub pages_generated: usize,
    /// Number of images embedded
    pub images_embedded: usize,
    /// Number of fonts used
    pub fonts_used: usize,
    /// Input size in bytes
    pub input_bytes: usize,
    /// Output size in bytes
    pub output_bytes: usize,
    /// Time spent in each stage
    pub stage_times: HashMap<ProgressStage, Duration>,
    /// Peak memory usage in bytes (if available)
    pub peak_memory_bytes: Option<usize>,
    /// Current memory usage in bytes (if available)
    pub current_memory_bytes: Option<usize>,
}

impl ConversionStats {
    /// Create new empty stats
    pub fn new() -> Self {
        Self::default()
    }

    /// Get total conversion time
    pub fn total_time(&self) -> Duration {
        self.stage_times.values().sum()
    }

    /// Get time spent in a specific stage
    pub fn stage_time(&self, stage: ProgressStage) -> Duration {
        self.stage_times.get(&stage).copied().unwrap_or_default()
    }

    /// Add time to a stage
    pub fn add_stage_time(&mut self, stage: ProgressStage, duration: Duration) {
        *self.stage_times.entry(stage).or_insert_with(Duration::default) += duration;
    }

    /// Get average time per element
    pub fn avg_time_per_element(&self) -> Option<Duration> {
        if self.elements_processed == 0 {
            None
        } else {
            Some(self.total_time() / self.elements_processed as u32)
        }
    }

    /// Get average time per page
    pub fn avg_time_per_page(&self) -> Option<Duration> {
        if self.pages_generated == 0 {
            None
        } else {
            Some(self.total_time() / self.pages_generated as u32)
        }
    }

    /// Format total time as human-readable string
    pub fn format_total_time(&self) -> String {
        format_duration(self.total_time())
    }

    /// Format memory usage
    pub fn format_memory(&self) -> String {
        match self.peak_memory_bytes {
            Some(bytes) => format_bytes(bytes),
            None => "unknown".to_string(),
        }
    }

    /// Reset all counters
    pub fn reset(&mut self) {
        *self = Self::new();
    }
}

impl fmt::Display for ConversionStats {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Conversion Statistics:")?;
        writeln!(f, "  Elements processed: {}", self.elements_processed)?;
        writeln!(f, "  CSS rules parsed: {}", self.css_rules_parsed)?;
        writeln!(f, "  Layout boxes created: {}", self.layout_boxes_created)?;
        writeln!(f, "  Pages generated: {}", self.pages_generated)?;
        writeln!(f, "  Images embedded: {}", self.images_embedded)?;
        writeln!(f, "  Fonts used: {}", self.fonts_used)?;
        writeln!(f, "  Input size: {}", format_bytes(self.input_bytes))?;
        writeln!(f, "  Output size: {}", format_bytes(self.output_bytes))?;
        writeln!(f, "  Total time: {}", self.format_total_time())?;
        
        for stage in ProgressStage::all_stages() {
            if let Some(&time) = self.stage_times.get(&stage) {
                writeln!(f, "  {}: {}", stage.name(), format_duration(time))?;
            }
        }
        
        if let Some(mem) = self.peak_memory_bytes {
            writeln!(f, "  Peak memory: {}", format_bytes(mem))?;
        }
        
        Ok(())
    }
}

/// Tracks progress and statistics for a conversion operation
pub struct ProgressTracker {
    callback: Arc<dyn ProgressCallback>,
    stats: Arc<RwLock<ConversionStats>>,
    current_stage: Arc<RwLock<ProgressStage>>,
    stage_start_time: Arc<Mutex<Option<Instant>>>,
    cancelled: Arc<RwLock<bool>>,
    verbose: bool,
}

impl ProgressTracker {
    /// Create a new progress tracker with the given callback
    pub fn new<C: ProgressCallback + 'static>(callback: C) -> Self {
        Self {
            callback: Arc::new(callback),
            stats: Arc::new(RwLock::new(ConversionStats::new())),
            current_stage: Arc::new(RwLock::new(ProgressStage::Loading)),
            stage_start_time: Arc::new(Mutex::new(Some(Instant::now()))),
            cancelled: Arc::new(RwLock::new(false)),
            verbose: false,
        }
    }

    /// Create a new progress tracker with no-op callback
    pub fn noop() -> Self {
        Self::new(NoOpProgressCallback)
    }

    /// Enable or disable verbose mode
    pub fn with_verbose(mut self, verbose: bool) -> Self {
        self.verbose = verbose;
        self
    }

    /// Report progress for the current stage
    ///
    /// Returns `false` if cancellation was requested
    pub fn report_progress(&self, percent: f32, message: impl Into<String>) -> bool {
        if *self.cancelled.read().unwrap() {
            return false;
        }

        let stage = *self.current_stage.read().unwrap();
        let message = message.into();

        if self.verbose {
            eprintln!("[{}] {:.1}%: {}", stage.code(), percent, message);
        }

        let should_continue = self.callback.on_progress(stage, percent.clamp(0.0, 100.0), &message);

        if !should_continue {
            *self.cancelled.write().unwrap() = true;
        }

        should_continue
    }

    /// Report progress with item count (e.g., "Processing element 5/100")
    ///
    /// Returns `false` if cancellation was requested
    pub fn report_items(&self, current: usize, total: usize, item_name: &str) -> bool {
        if total == 0 {
            return self.report_progress(0.0, format!("No {} to process", item_name));
        }
        let percent = (current as f32 / total as f32) * 100.0;
        let message = format!("Processing {} {} of {}", item_name, current, total);
        self.report_progress(percent, message)
    }

    /// Begin a new stage
    pub fn begin_stage(&self, stage: ProgressStage) {
        // End current stage if any
        self.end_current_stage();

        // Start new stage
        *self.current_stage.write().unwrap() = stage;
        *self.stage_start_time.lock().unwrap() = Some(Instant::now());

        self.callback.on_stage_begin(stage);
    }

    /// End the current stage and record timing
    pub fn end_current_stage(&self) {
        let stage = *self.current_stage.read().unwrap();
        let start_time = self.stage_start_time.lock().unwrap().take();

        if let Some(start) = start_time {
            let duration = start.elapsed();
            if let Ok(mut stats) = self.stats.write() {
                stats.add_stage_time(stage, duration);
            }
            self.callback.on_stage_end(stage);
        }
    }

    /// Report a warning
    pub fn warning(&self, message: impl Into<String>) {
        let message = message.into();
        if self.verbose {
            eprintln!("[WARN] {}", message);
        }
        self.callback.on_warning(&message);
    }

    /// Report an error
    pub fn error(&self, error: impl Into<String>) {
        let error = error.into();
        if self.verbose {
            eprintln!("[ERROR] {}", error);
        }
        self.callback.on_error(&error);
    }

    /// Check if cancellation was requested
    pub fn is_cancelled(&self) -> bool {
        *self.cancelled.read().unwrap()
    }

    /// Request cancellation
    pub fn cancel(&self) {
        *self.cancelled.write().unwrap() = true;
    }

    /// Update statistics
    pub fn update_stats<F>(&self, f: F)
    where
        F: FnOnce(&mut ConversionStats),
    {
        if let Ok(mut stats) = self.stats.write() {
            f(&mut stats);
        }
    }

    /// Get a copy of current statistics
    pub fn stats(&self) -> ConversionStats {
        self.stats.read().unwrap().clone()
    }

    /// Get the current stage
    pub fn current_stage(&self) -> ProgressStage {
        *self.current_stage.read().unwrap()
    }

    /// Get elapsed time for current stage
    pub fn current_stage_elapsed(&self) -> Option<Duration> {
        self.stage_start_time
            .lock()
            .unwrap()
            .map(|start| start.elapsed())
    }

    /// Create a counter for tracking item processing
    pub fn item_counter(&self, total: usize, item_name: &'static str) -> ItemCounter<'_> {
        ItemCounter {
            tracker: self,
            current: 0,
            total,
            item_name,
            report_interval: (total / 100).max(1), // Report ~100 times
        }
    }

    /// Finish tracking and return final statistics
    pub fn finish(self) -> ConversionStats {
        self.end_current_stage();
        self.stats()
    }
}

/// Helper for tracking item-by-item progress
pub struct ItemCounter<'a> {
    tracker: &'a ProgressTracker,
    current: usize,
    total: usize,
    item_name: &'static str,
    report_interval: usize,
}

impl<'a> ItemCounter<'a> {
    /// Increment the counter and report progress if needed
    ///
    /// Returns `false` if cancellation was requested
    pub fn increment(&mut self) -> bool {
        self.current += 1;
        if self.current % self.report_interval == 0 || self.current == self.total {
            self.tracker.report_items(self.current, self.total, self.item_name)
        } else {
            !self.tracker.is_cancelled()
        }
    }

    /// Get current count
    pub fn current(&self) -> usize {
        self.current
    }

    /// Get total count
    pub fn total(&self) -> usize {
        self.total
    }

    /// Set custom report interval
    pub fn with_interval(mut self, interval: usize) -> Self {
        self.report_interval = interval.max(1);
        self
    }
}

/// A progress callback that wraps a closure
pub struct ClosureProgressCallback<F>
where
    F: Fn(ProgressStage, f32, &str) -> bool + Send + Sync,
{
    handler: F,
}

impl<F> ClosureProgressCallback<F>
where
    F: Fn(ProgressStage, f32, &str) -> bool + Send + Sync,
{
    /// Create a new callback from a closure
    pub fn new(handler: F) -> Self {
        Self { handler }
    }
}

impl<F> ProgressCallback for ClosureProgressCallback<F>
where
    F: Fn(ProgressStage, f32, &str) -> bool + Send + Sync,
{
    fn on_progress(&self, stage: ProgressStage, percent: f32, message: &str) -> bool {
        (self.handler)(stage, percent, message)
    }

    fn on_warning(&self, _message: &str) {}

    fn on_error(&self, _error: &str) {}
}

/// Format a duration as a human-readable string
pub fn format_duration(duration: Duration) -> String {
    let secs = duration.as_secs();
    let millis = duration.subsec_millis();

    if secs >= 60 {
        let mins = secs / 60;
        let secs = secs % 60;
        format!("{}m {}s", mins, secs)
    } else if secs > 0 {
        format!("{}.{:03}s", secs, millis)
    } else {
        format!("{}ms", millis)
    }
}

/// Format bytes as a human-readable string
pub fn format_bytes(bytes: usize) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB"];
    
    if bytes == 0 {
        return "0 B".to_string();
    }

    let exp = (bytes as f64).log(1024.0).min(UNITS.len() as f64 - 1.0) as usize;
    let value = bytes as f64 / 1024f64.powi(exp as i32);

    if exp == 0 {
        format!("{} B", bytes)
    } else {
        format!("{:.2} {}", value, UNITS[exp])
    }
}

/// Calculate estimated time remaining
pub fn estimate_remaining(elapsed: Duration, percent_complete: f32) -> Option<Duration> {
    if percent_complete <= 0.0 || percent_complete >= 100.0 {
        return None;
    }

    let elapsed_secs = elapsed.as_secs_f32();
    let total_estimate_secs = elapsed_secs / (percent_complete / 100.0);
    let remaining_secs = total_estimate_secs - elapsed_secs;

    Some(Duration::from_secs_f32(remaining_secs.max(0.0)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_progress_stage_order() {
        let stages = ProgressStage::all_stages();
        assert_eq!(stages[0], ProgressStage::Loading);
        assert_eq!(stages.last(), Some(&ProgressStage::Complete));

        // Check next/previous
        assert_eq!(ProgressStage::Loading.next(), Some(ProgressStage::ParsingHtml));
        assert_eq!(ProgressStage::ParsingHtml.previous(), Some(ProgressStage::Loading));
        assert_eq!(ProgressStage::Complete.next(), None);
    }

    #[test]
    fn test_progress_stage_weights() {
        let total: f32 = ProgressStage::all_stages()
            .iter()
            .map(|s| s.weight())
            .sum();
        assert!((total - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_conversion_stats() {
        let mut stats = ConversionStats::new();
        stats.elements_processed = 100;
        stats.pages_generated = 5;
        stats.add_stage_time(ProgressStage::ParsingHtml, Duration::from_secs(1));
        stats.add_stage_time(ProgressStage::Rendering, Duration::from_secs(2));

        assert_eq!(stats.total_time(), Duration::from_secs(3));
        assert_eq!(stats.stage_time(ProgressStage::ParsingHtml), Duration::from_secs(1));
        assert_eq!(stats.avg_time_per_page(), Some(Duration::from_millis(600)));
    }

    #[test]
    fn test_progress_tracker() {
        let tracker = ProgressTracker::noop();
        assert!(tracker.report_progress(50.0, "test"));
        assert!(!tracker.is_cancelled());
    }

    #[test]
    fn test_format_duration() {
        assert_eq!(format_duration(Duration::from_millis(500)), "500ms");
        assert_eq!(format_duration(Duration::from_secs(5)), "5.000s");
        assert_eq!(format_duration(Duration::from_secs(65)), "1m 5s");
    }

    #[test]
    fn test_format_bytes() {
        assert_eq!(format_bytes(0), "0 B");
        assert_eq!(format_bytes(512), "512 B");
        assert_eq!(format_bytes(1024), "1.00 KB");
        assert_eq!(format_bytes(1024 * 1024), "1.00 MB");
    }

    #[test]
    fn test_estimate_remaining() {
        let elapsed = Duration::from_secs(10);
        assert_eq!(estimate_remaining(elapsed, 0.0), None);
        assert_eq!(estimate_remaining(elapsed, 100.0), None);
        
        let remaining = estimate_remaining(elapsed, 25.0).unwrap();
        assert_eq!(remaining, Duration::from_secs(30)); // 10s = 25%, so 30s remaining
    }

    #[test]
    fn test_item_counter() {
        let tracker = ProgressTracker::noop();
        let mut counter = tracker.item_counter(100, "elements");
        
        for _ in 0..50 {
            assert!(counter.increment());
        }
        
        assert_eq!(counter.current(), 50);
    }
}
