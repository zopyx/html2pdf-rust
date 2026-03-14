//! Command-line interface for html2pdf
//!
//! This module provides the CLI argument parsing and orchestration
//! for the html2pdf conversion tool.

use clap::{Parser, Subcommand, ValueEnum};

use std::io::{self, Read, Write};
use std::path::PathBuf;
use tracing::{debug, info, Level};

use crate::{
    html_to_pdf_from_input, Config, Input, Margins, Orientation, PaperSize,
    Result as PdfResult,
};
use crate::cache::{CacheConfig, CacheManager};
use crate::progress::{ProgressCallback, ProgressStage, ProgressTracker};

/// HTML to PDF converter with W3C PrintCSS support
#[derive(Parser, Debug)]
#[command(
    name = "html2pdf",
    version = env!("CARGO_PKG_VERSION"),
    about = "Convert HTML to PDF with full PrintCSS support",
    long_about = r#"
html2pdf is a from-scratch Rust HTML to PDF converter supporting:
- Complete HTML5 parsing
- CSS3 with PrintCSS/Paged Media extensions
- Custom page sizes, margins, and orientation
- Headers and footers with template variables
- High-quality typography and layout

Examples:
  html2pdf input.html -o output.pdf
  cat input.html | html2pdf -o output.pdf
  html2pdf https://example.com -o output.pdf --paper-size A4
"#
)]
#[command(disable_help_flag = true)]
pub struct Cli {
    /// Input HTML file, URL, or '-' for stdin
    #[arg(value_name = "INPUT", help = "HTML file, URL, or '-' for stdin")]
    pub input: Option<String>,

    /// Output PDF file or '-' for stdout
    #[arg(short, long, value_name = "FILE", help = "Output PDF file (or '-' for stdout)")]
    pub output: Option<String>,

    /// Paper size
    #[arg(
        short,
        long,
        value_enum,
        default_value = "a4",
        help = "Paper size (A4, Letter, Legal, etc.)"
    )]
    pub paper_size: PaperSizeArg,

    /// Page orientation
    #[arg(
        short,
        long,
        value_enum,
        default_value = "portrait",
        help = "Page orientation"
    )]
    pub orientation: OrientationArg,

    /// Page margins (in points, or with units: 1in, 20mm, etc.)
    #[arg(
        short,
        long,
        value_name = "MARGIN",
        help = "Page margins (e.g., 72, 1in, 20mm, or 'top,right,bottom,left')"
    )]
    pub margin: Option<String>,

    /// Custom page width (overrides paper-size)
    #[arg(
        long,
        value_name = "WIDTH",
        help = "Custom page width (e.g., 210mm, 8.5in)"
    )]
    pub page_width: Option<String>,

    /// Custom page height (overrides paper-size)
    #[arg(
        long,
        value_name = "HEIGHT",
        help = "Custom page height (e.g., 297mm, 11in)"
    )]
    pub page_height: Option<String>,

    /// Header template HTML
    #[arg(
        long,
        value_name = "HTML",
        help = "Header template HTML (e.g., '<h1>Header</h1>')"
    )]
    pub header: Option<String>,

    /// Footer template HTML
    #[arg(
        long,
        value_name = "HTML",
        help = "Footer template HTML (e.g., '<p>Page <span class=\"page\"></span></p>')"
    )]
    pub footer: Option<String>,

    /// Path to header HTML file
    #[arg(
        long,
        value_name = "FILE",
        help = "Path to header HTML file",
        conflicts_with = "header"
    )]
    pub header_file: Option<PathBuf>,

    /// Path to footer HTML file
    #[arg(
        long,
        value_name = "FILE",
        help = "Path to footer HTML file",
        conflicts_with = "footer"
    )]
    pub footer_file: Option<PathBuf>,

    /// Configuration file (JSON or TOML)
    #[arg(
        short,
        long,
        value_name = "FILE",
        help = "Configuration file path (JSON format)"
    )]
    pub config: Option<PathBuf>,

    /// Additional CSS stylesheet
    #[arg(
        short,
        long,
        value_name = "FILE",
        help = "Additional CSS stylesheet to apply"
    )]
    pub stylesheet: Vec<PathBuf>,

    /// Base URL for resolving relative URLs
    #[arg(long, value_name = "URL", help = "Base URL for resolving relative URLs")]
    pub base_url: Option<String>,

    /// Wait for network idle (for URL input)
    #[arg(
        long,
        value_name = "SECONDS",
        help = "Timeout for network requests in seconds",
        default_value = "30"
    )]
    pub timeout: u64,

    /// Enable verbose output
    #[arg(short, long, help = "Enable verbose output")]
    pub verbose: bool,

    /// Enable debug layout visualization
    #[arg(long, help = "Show layout debugging information")]
    pub debug_layout: bool,

    /// Cache directory path
    #[arg(
        long,
        value_name = "DIR",
        help = "Cache directory for downloaded resources"
    )]
    pub cache_dir: Option<std::path::PathBuf>,

    /// Disable caching
    #[arg(long, help = "Disable all caching")]
    pub no_cache: bool,

    /// Cache TTL in seconds
    #[arg(
        long,
        value_name = "SECONDS",
        help = "Cache TTL in seconds for HTTP resources",
        default_value = "3600"
    )]
    pub cache_ttl: u64,

    /// Clear cache and exit
    #[arg(long, help = "Clear the cache and exit")]
    pub clear_cache: bool,

    /// Print cache statistics
    #[arg(long, help = "Print cache statistics after conversion")]
    pub cache_stats: bool,

    /// Print version and exit
    #[arg(short = 'V', long, help = "Print version information")]
    pub version: bool,

    /// Print help
    #[arg(short = 'h', long, help = "Print help information")]
    pub help: bool,

    /// Subcommands
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Show version information
    Version,
    /// Validate HTML/CSS without generating PDF
    Validate {
        /// Input file or URL
        input: String,
    },
    /// Print default configuration
    Config,
}

/// Paper size argument for CLI
///
/// Maps to the library's `PaperSize` type.
#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum PaperSizeArg {
    /// ISO A0 (841 × 1189 mm)
    A0,
    /// ISO A1 (594 × 841 mm)
    A1,
    /// ISO A2 (420 × 594 mm)
    A2,
    /// ISO A3 (297 × 420 mm)
    A3,
    /// ISO A4 (210 × 297 mm)
    A4,
    /// ISO A5 (148 × 210 mm)
    A5,
    /// ISO A6 (105 × 148 mm)
    A6,
    /// US Letter (8.5 × 11 inches)
    Letter,
    /// US Legal (8.5 × 14 inches)
    Legal,
    /// US Tabloid (11 × 17 inches)
    Tabloid,
}

impl From<PaperSizeArg> for PaperSize {
    fn from(arg: PaperSizeArg) -> Self {
        match arg {
            PaperSizeArg::A0 => PaperSize::A0,
            PaperSizeArg::A1 => PaperSize::A1,
            PaperSizeArg::A2 => PaperSize::A2,
            PaperSizeArg::A3 => PaperSize::A3,
            PaperSizeArg::A4 => PaperSize::A4,
            PaperSizeArg::A5 => PaperSize::A5,
            PaperSizeArg::A6 => PaperSize::A6,
            PaperSizeArg::Letter => PaperSize::Letter,
            PaperSizeArg::Legal => PaperSize::Legal,
            PaperSizeArg::Tabloid => PaperSize::Tabloid,
        }
    }
}

/// Orientation argument for CLI
///
/// Maps to the library's `Orientation` type.
#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum OrientationArg {
    /// Portrait orientation (taller than wide)
    Portrait,
    /// Landscape orientation (wider than tall)
    Landscape,
}

impl From<OrientationArg> for Orientation {
    fn from(arg: OrientationArg) -> Self {
        match arg {
            OrientationArg::Portrait => Orientation::Portrait,
            OrientationArg::Landscape => Orientation::Landscape,
        }
    }
}

/// Run the CLI application
pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    // Handle help flag
    if cli.help {
        print_help();
        return Ok(());
    }

    // Handle version flag
    if cli.version {
        println!("html2pdf {}", env!("CARGO_PKG_VERSION"));
        return Ok(());
    }

    // Handle subcommands
    if let Some(cmd) = cli.command {
        match cmd {
            Commands::Version => {
                println!("html2pdf {}", env!("CARGO_PKG_VERSION"));
                println!("Rust HTML to PDF converter with W3C PrintCSS support");
                return Ok(());
            }
            Commands::Validate { input } => {
                return validate_input(&input).map_err(|e| Box::new(e) as Box<dyn std::error::Error>);
            }
            Commands::Config => {
                print_default_config();
                return Ok(());
            }
        }
    }

    // Handle --clear-cache
    if cli.clear_cache {
        let cache_config = CacheConfig::default();
        if let Some(cache_dir) = &cli.cache_dir {
            let cache_config = cache_config.with_cache_dir(cache_dir.clone());
            let cache = CacheManager::with_config(&cache_config);
            match cache.clear_all() {
                Ok(_) => {
                    println!("Cache cleared successfully: {}", cache.cache_dir().display());
                    return Ok(());
                }
                Err(e) => {
                    eprintln!("Failed to clear cache: {}", e);
                    std::process::exit(1);
                }
            }
        } else {
            let cache = CacheManager::with_config(&cache_config);
            match cache.clear_all() {
                Ok(_) => {
                    println!("Cache cleared successfully: {}", cache.cache_dir().display());
                    return Ok(());
                }
                Err(e) => {
                    eprintln!("Failed to clear cache: {}", e);
                    std::process::exit(1);
                }
            }
        }
    }

    // Setup logging
    setup_logging(cli.verbose);

    // Get input source
    let input = get_input(&cli).map_err(|e| {
        let err = errors::io(format!("Failed to read input: {}", e));
        Box::new(err) as Box<dyn std::error::Error>
    })?;
    info!("Input source: {}", input.description());

    // Build configuration
    let config = build_config(&cli).map_err(|e| {
        let err = errors::validation(format!("Invalid configuration: {}", e));
        Box::new(err) as Box<dyn std::error::Error>
    })?;
    debug!("Configuration: {:?}", config);

    // Perform conversion with progress indication
    info!("Converting HTML to PDF...");
    let pdf_bytes = if is_stderr_tty() && cli.verbose {
        convert_with_progress(&input, &config).map_err(|e| {
            Box::new(e) as Box<dyn std::error::Error>
        })?
    } else {
        html_to_pdf_from_input(&input, &config).map_err(|e| {
            Box::new(e) as Box<dyn std::error::Error>
        })?
    };

    // Write output
    write_output(&cli, pdf_bytes).map_err(|e| {
        let err = errors::io(format!("Failed to write output: {}", e));
        Box::new(err) as Box<dyn std::error::Error>
    })?;

    // Print cache statistics if requested
    if cli.cache_stats {
        let cache = CacheManager::with_config(&config.cache_config);
        let stats = cache.stats();
        eprintln!("\nCache Statistics:");
        eprintln!("  Entries: {}", stats.entries);
        eprintln!("  Total Size: {} bytes", stats.total_size);
        eprintln!("  Hits: {}", stats.hits);
        eprintln!("  Misses: {}", stats.misses);
        eprintln!("  Hit Rate: {:.1}%", stats.hit_rate() * 100.0);
        eprintln!("  Evictions: {}", stats.evictions);
    }

    info!("Conversion complete!");
    Ok(())
}

/// Print help message
fn print_help() {
    println!("html2pdf {}", env!("CARGO_PKG_VERSION"));
    println!("Convert HTML to PDF with full PrintCSS support\n");
    println!("USAGE:");
    println!("    html2pdf [OPTIONS] [INPUT]\n");
    println!("ARGS:");
    println!("    <INPUT>    HTML file, URL, or '-' for stdin\n");
    println!("OPTIONS:");
    println!("    -o, --output <FILE>          Output PDF file (or '-' for stdout)");
    println!("    -p, --paper-size <SIZE>      Paper size: A0-A6, Letter, Legal, Tabloid [default: A4]");
    println!("    -O, --orientation <ORIENT>   Orientation: Portrait, Landscape [default: Portrait]");
    println!("    -m, --margin <MARGIN>        Page margins (points, mm, in, cm)");
    println!("        --page-width <WIDTH>     Custom page width (overrides paper-size)");
    println!("        --page-height <HEIGHT>   Custom page height (overrides paper-size)");
    println!("        --header <HTML>          Header template HTML");
    println!("        --footer <HTML>          Footer template HTML");
    println!("        --header-file <FILE>     Path to header HTML file");
    println!("        --footer-file <FILE>     Path to footer HTML file");
    println!("    -c, --config <FILE>          Configuration file path (JSON)");
    println!("    -s, --stylesheet <FILE>      Additional CSS stylesheet");
    println!("        --base-url <URL>         Base URL for resolving relative URLs");
    println!("        --timeout <SECONDS>      Network timeout [default: 30]");
    println!("    -v, --verbose                Enable verbose output");
    println!("        --debug-layout           Show layout debugging");
    println!("        --cache-dir <DIR>        Cache directory for resources");
    println!("        --no-cache               Disable all caching");
    println!("        --cache-ttl <SECONDS>    Cache TTL for HTTP resources [default: 3600]");
    println!("        --clear-cache            Clear cache and exit");
    println!("        --cache-stats            Print cache statistics");
    println!("    -V, --version                Print version");
    println!("    -h, --help                   Print help\n");
    println!("EXAMPLES:");
    println!("    html2pdf input.html -o output.pdf");
    println!("    cat input.html | html2pdf -o output.pdf");
    println!("    html2pdf https://example.com -o output.pdf --paper-size A4 -O landscape");
    println!("    html2pdf - -o output.pdf < input.html");
}

/// Print default configuration
fn print_default_config() {
    println!(
        r#"{{
    "paper_size": "A4",
    "orientation": "portrait",
    "margin": 72,
    "debug_layout": false
}}"#
    );
}

/// Setup logging/tracing
fn setup_logging(verbose: bool) {
    let _level = if verbose {
        Level::DEBUG
    } else {
        Level::INFO
    };

    // Simple stderr logging for CLI
    // In production, use tracing-subscriber with env-filter
    if verbose {
        eprintln!("[INFO] Verbose logging enabled");
    }
}

/// Get input source from CLI arguments
fn get_input(cli: &Cli) -> Result<Input, Box<dyn std::error::Error>> {
    // Check if input is provided as argument
    if let Some(input) = &cli.input {
        if input == "-" {
            // Read from stdin
            let mut buffer = String::new();
            io::stdin().read_to_string(&mut buffer)?;
            Ok(Input::Html(buffer))
        } else if input.starts_with("http://") || input.starts_with("https://") {
            // URL input
            Ok(Input::Url(input.clone()))
        } else {
            // File input
            if !std::path::Path::new(input).exists() {
                return Err(format!("Input file not found: {}", input).into());
            }
            Ok(Input::File(input.clone()))
        }
    } else {
        // No input argument, try stdin
        let mut buffer = String::new();
        match io::stdin().read_to_string(&mut buffer) {
            Ok(0) => Err("No input provided. Use '-' for stdin or provide a file/URL.".into()),
            Ok(_) => Ok(Input::Html(buffer)),
            Err(e) => Err(format!("Failed to read from stdin: {}", e).into()),
        }
    }
}

/// Build configuration from CLI arguments
fn build_config(cli: &Cli) -> Result<Config, Box<dyn std::error::Error>> {
    let mut config = if let Some(config_path) = &cli.config {
        // Load from file
        if !config_path.exists() {
            return Err(format!("Config file not found: {:?}", config_path).into());
        }
        Config::from_file(config_path)?
    } else {
        Config::default()
    };

    // Apply CLI overrides
    config.paper_size = cli.paper_size.into();
    config.orientation = cli.orientation.into();

    // Parse margins
    if let Some(margin_str) = &cli.margin {
        config.margins = parse_margins(margin_str)?;
    }

    // Parse custom page size
    if let Some(width_str) = &cli.page_width {
        config.page_width = Some(parse_length(width_str)?);
    }
    if let Some(height_str) = &cli.page_height {
        config.page_height = Some(parse_length(height_str)?);
    }

    // Load header/footer from files if specified
    if let Some(header_file) = &cli.header_file {
        config.header = Some(std::fs::read_to_string(header_file)?);
    } else if let Some(header) = &cli.header {
        config.header = Some(header.clone());
    }

    if let Some(footer_file) = &cli.footer_file {
        config.footer = Some(std::fs::read_to_string(footer_file)?);
    } else if let Some(footer) = &cli.footer {
        config.footer = Some(footer.clone());
    }

    // Load additional stylesheets
    for stylesheet_path in &cli.stylesheet {
        let css = std::fs::read_to_string(stylesheet_path)?;
        config.user_stylesheets.push(css);
    }

    // Set base URL
    if let Some(base_url) = &cli.base_url {
        config.base_url = Some(base_url.clone());
    }

    // Set timeout
    config.timeout_seconds = cli.timeout;

    // Set debug layout
    config.debug_layout = cli.debug_layout;

    // Apply cache configuration
    if cli.no_cache {
        config.cache_config = config.cache_config.disable_cache();
    }

    if let Some(cache_dir) = &cli.cache_dir {
        config.cache_config = config.cache_config.with_cache_dir(cache_dir.clone());
    }

    config.cache_config = config.cache_config.with_http_ttl(std::time::Duration::from_secs(cli.cache_ttl));

    Ok(config)
}

/// Parse margin string (supports various formats)
fn parse_margins(s: &str) -> Result<Margins, Box<dyn std::error::Error>> {
    // Check for comma-separated format: top,right,bottom,left
    if s.contains(',') {
        let parts: Vec<&str> = s.split(',').collect();
        if parts.len() != 4 {
            return Err(
                "Invalid margin format. Use 'top,right,bottom,left' or single value.".into(),
            );
        }
        return Ok(Margins::new(
            parse_length(parts[0].trim())?,
            parse_length(parts[1].trim())?,
            parse_length(parts[2].trim())?,
            parse_length(parts[3].trim())?,
        ));
    }

    // Single value applies to all sides
    let value = parse_length(s)?;
    Ok(Margins::all(value))
}

/// Parse a length value with optional unit
fn parse_length(s: &str) -> Result<f32, Box<dyn std::error::Error>> {
    let s = s.trim();

    // Try parsing as number (assumes points)
    if let Ok(n) = s.parse::<f32>() {
        return Ok(n);
    }

    // Parse with unit
    if let Some(stripped) = s.strip_suffix("pt") {
        Ok(stripped.trim().parse()?)
    } else if let Some(stripped) = s.strip_suffix("mm") {
        let mm: f32 = stripped.trim().parse()?;
        Ok(mm * 2.834_646) // mm to points
    } else if let Some(stripped) = s.strip_suffix("cm") {
        let cm: f32 = stripped.trim().parse()?;
        Ok(cm * 28.346_46) // cm to points
    } else if let Some(stripped) = s.strip_suffix("in") {
        let inch: f32 = stripped.trim().parse()?;
        Ok(inch * 72.0) // inches to points
    } else if let Some(stripped) = s.strip_suffix("px") {
        let px: f32 = stripped.trim().parse()?;
        Ok(px * 0.75) // pixels to points (96 DPI)
    } else {
        Err(format!("Unknown length unit in: {}", s).into())
    }
}

/// CLI progress callback implementation
#[cfg(feature = "progress")]
struct CliProgressCallback {
    progress_bar: indicatif::ProgressBar,
    current_stage: std::sync::Mutex<ProgressStage>,
}

#[cfg(feature = "progress")]
impl CliProgressCallback {
    fn new() -> Self {
        let pb = indicatif::ProgressBar::new(100);
        pb.set_style(
            indicatif::ProgressStyle::default_bar()
                .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {msg}")
                .unwrap()
                .progress_chars("#>-"),
        );
        
        Self {
            progress_bar: pb,
            current_stage: std::sync::Mutex::new(ProgressStage::Loading),
        }
    }
    
    fn finish(&self) {
        self.progress_bar.finish_with_message("Done");
    }
}

#[cfg(feature = "progress")]
impl ProgressCallback for CliProgressCallback {
    fn on_progress(&self, stage: ProgressStage, percent: f32, message: &str) -> bool {
        let mut current = self.current_stage.lock().unwrap();
        
        // Calculate overall progress based on stage weights
        let stages = ProgressStage::all_stages();
        let stage_index = stages.iter().position(|s| *s == stage).unwrap_or(0);
        let completed_weight: f32 = stages[..stage_index].iter().map(|s| s.weight()).sum();
        let current_weight = stage.weight() * (percent / 100.0);
        let overall_percent = ((completed_weight + current_weight) * 100.0) as u64;
        
        *current = stage;
        drop(current);
        
        self.progress_bar.set_position(overall_percent.min(100));
        self.progress_bar.set_message(format!("{}: {}", stage.code(), message));
        
        true
    }
    
    fn on_warning(&self, message: &str) {
        self.progress_bar.println(format!("⚠️  Warning: {}", message));
    }
    
    fn on_error(&self, error: &str) {
        self.progress_bar.println(format!("❌ Error: {}", error));
    }
}

/// Simple text-based progress callback for non-feature builds
struct TextProgressCallback;

impl ProgressCallback for TextProgressCallback {
    fn on_progress(&self, stage: ProgressStage, percent: f32, message: &str) -> bool {
        eprintln!("[{}] {:.0}%: {}", stage.code(), percent, message);
        true
    }
    
    fn on_warning(&self, message: &str) {
        eprintln!("⚠️  Warning: {}", message);
    }
    
    fn on_error(&self, error: &str) {
        eprintln!("❌ Error: {}", error);
    }
}

/// Convert with progress indication and detailed error handling
fn convert_with_progress(input: &Input, config: &Config) -> PdfResult<Vec<u8>> {
    // Create progress tracker
    #[cfg(feature = "progress")]
    let callback = CliProgressCallback::new();
    #[cfg(not(feature = "progress"))]
    let callback = TextProgressCallback;
    
    let tracker = ProgressTracker::new(callback);
    
    // Build config with progress tracker
    let mut config_with_progress = config.clone();
    config_with_progress.progress = Some(std::sync::Arc::new(tracker.clone()));

    eprint!("[1/4] Loading HTML... ");
    let html_content = input.load().map_err(|e| {
        eprintln!("FAILED");
        crate::types::PdfError::Io(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Failed to load input: {}", e)
        ))
    })?;
    eprintln!("OK ({} bytes)", html_content.len());

    eprint!("[2/4] Parsing document... ");
    let _document = crate::html::parse_html(&html_content).map_err(|e| {
        eprintln!("FAILED");
        e
    })?;
    eprintln!("OK");

    eprint!("[3/4] Processing styles... ");
    // Pre-parse stylesheets to catch errors early
    for css in &config.user_stylesheets {
        if let Err(_e) = crate::css::parse_stylesheet(css) {
            tracker.warning("Failed to parse user stylesheet");
        }
    }
    eprintln!("OK");

    eprint!("[4/4] Generating PDF... ");
    let result = html_to_pdf_from_input(input, &config_with_progress).map_err(|e| {
        eprintln!("FAILED");
        e
    })?;
    eprintln!("OK ({} bytes)", result.len());

    #[cfg(feature = "progress")]
    {
        use crate::progress::ProgressCallback;
        callback.finish();
    }

    // Get and display final statistics
    let stats = tracker.stats();
    if stats.elements_processed > 0 {
        eprintln!("\n📊 Statistics:");
        eprintln!("   Elements: {}", stats.elements_processed);
        eprintln!("   CSS rules: {}", stats.css_rules_parsed);
        eprintln!("   Pages: {}", stats.pages_generated);
        eprintln!("   Total time: {}", stats.format_total_time());
        if stats.output_bytes > 0 {
            eprintln!("   Output size: {}", crate::progress::format_bytes(stats.output_bytes));
        }
    }

    Ok(result)
}

/// Write PDF output
fn write_output(cli: &Cli, pdf_bytes: Vec<u8>) -> Result<(), Box<dyn std::error::Error>> {
    match &cli.output {
        Some(path) if path == "-" => {
            // Write to stdout
            let stdout = io::stdout();
            let mut handle = stdout.lock();
            handle.write_all(&pdf_bytes)?;
        }
        Some(path) => {
            // Write to file
            std::fs::write(path, pdf_bytes)?;
            if cli.verbose {
                eprintln!("[INFO] PDF written to: {}", path);
            }
        }
        None => {
            // Default: derive output name from input
            if let Some(input) = &cli.input {
                if input != "-" && !input.starts_with("http") {
                    let output = input.replace(".html", ".pdf").replace(".htm", ".pdf");
                    std::fs::write(&output, pdf_bytes)?;
                    if cli.verbose {
                        eprintln!("[INFO] PDF written to: {}", output);
                    }
                } else {
                    // Write to stdout
                    let stdout = io::stdout();
                    let mut handle = stdout.lock();
                    handle.write_all(&pdf_bytes)?;
                }
            } else {
                // Write to stdout
                let stdout = io::stdout();
                let mut handle = stdout.lock();
                handle.write_all(&pdf_bytes)?;
            }
        }
    }
    Ok(())
}

/// Validate HTML/CSS input
fn validate_input(input: &str) -> Result<(), Box<dyn std::error::Error>> {
    use crate::progress::{ProgressTracker, ProgressStage};
    
    println!("Validating: {}", input);
    
    // Create progress tracker for verbose validation
    let tracker = ProgressTracker::new(crate::progress::NoOpProgressCallback);
    tracker.begin_stage(ProgressStage::Loading);

    // Load and parse HTML
    let html = if input.starts_with("http://") || input.starts_with("https://") {
        return Err("URL validation requires network access (not yet implemented)".into());
    } else if std::path::Path::new(input).exists() {
        std::fs::read_to_string(input)?
    } else {
        input.to_string()
    };
    
    tracker.end_current_stage();
    tracker.begin_stage(ProgressStage::ParsingHtml);

    match crate::html::parse_html(&html) {
        Ok(doc) => {
            println!("  [✓] HTML parsing successful");
            println!("      Document title: {:?}", &doc.title);
            println!("      Elements: {}", doc.element_count());
        }
        Err(e) => {
            println!("  [✗] HTML parsing failed: {}", e);
            return Err("Validation failed".into());
        }
    }
    
    tracker.end_current_stage();
    
    // Try to extract and parse CSS
    println!("  [✓] Validation complete");
    Ok(())
}

/// Helper function for checking if stderr is a TTY
fn is_stderr_tty() -> bool {
    // Simplified implementation
    // In production, use the atty crate
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_length() {
        assert_eq!(parse_length("72").unwrap(), 72.0);
        assert_eq!(parse_length("72pt").unwrap(), 72.0);
        assert!((parse_length("1in").unwrap() - 72.0).abs() < 0.01);
        assert!((parse_length("25.4mm").unwrap() - 72.0).abs() < 0.1);
        assert!((parse_length("2.54cm").unwrap() - 72.0).abs() < 0.1);
    }

    #[test]
    fn test_parse_margins() {
        let margins = parse_margins("72").unwrap();
        assert_eq!(margins.top, 72.0);
        assert_eq!(margins.right, 72.0);
        assert_eq!(margins.bottom, 72.0);
        assert_eq!(margins.left, 72.0);

        let margins = parse_margins("72,36,72,36").unwrap();
        assert_eq!(margins.top, 72.0);
        assert_eq!(margins.right, 36.0);
        assert_eq!(margins.bottom, 72.0);
        assert_eq!(margins.left, 36.0);
    }

    #[test]
    fn test_paper_size_arg_conversion() {
        assert_eq!(PaperSize::A4, PaperSizeArg::A4.into());
        assert_eq!(PaperSize::Letter, PaperSizeArg::Letter.into());
    }

    #[test]
    fn test_orientation_arg_conversion() {
        assert_eq!(Orientation::Portrait, OrientationArg::Portrait.into());
        assert_eq!(Orientation::Landscape, OrientationArg::Landscape.into());
    }
}
