//! Cache configuration
//!
//! Provides configuration options for the caching system including
//! size limits, TTL values, and directory paths.

use std::path::PathBuf;
use std::time::Duration;

/// Default cache size limits
pub const DEFAULT_MEMORY_CACHE_SIZE: usize = 100 * 1024 * 1024; // 100MB
pub const DEFAULT_DISK_CACHE_SIZE: usize = 500 * 1024 * 1024; // 500MB
pub const DEFAULT_HTTP_CACHE_SIZE: usize = 200 * 1024 * 1024; // 200MB

/// Default TTL values for different resource types
pub const DEFAULT_IMAGE_TTL: Duration = Duration::from_secs(86400 * 7); // 7 days
pub const DEFAULT_FONT_TTL: Duration = Duration::from_secs(86400 * 30); // 30 days
pub const DEFAULT_STYLE_TTL: Duration = Duration::from_secs(86400 * 1); // 1 day
pub const DEFAULT_HTTP_TTL: Duration = Duration::from_secs(3600); // 1 hour

/// Cache configuration
#[derive(Debug, Clone)]
pub struct CacheConfig {
    /// Enable/disable caching globally
    pub enabled: bool,
    /// Cache directory for disk storage
    pub cache_dir: PathBuf,
    /// Memory cache size limit in bytes
    pub memory_cache_size: usize,
    /// Disk cache size limit in bytes
    pub disk_cache_size: usize,
    /// HTTP cache size limit in bytes
    pub http_cache_size: usize,
    /// TTL for images
    pub image_ttl: Duration,
    /// TTL for fonts
    pub font_ttl: Duration,
    /// TTL for stylesheets
    pub style_ttl: Duration,
    /// Default TTL for HTTP resources
    pub http_ttl: Duration,
    /// Enable disk cache
    pub enable_disk_cache: bool,
    /// Enable HTTP cache
    pub enable_http_cache: bool,
    /// Respect Cache-Control headers
    pub respect_cache_control: bool,
    /// Enable ETag support
    pub enable_etag: bool,
    /// Maximum single entry size (10MB default)
    pub max_entry_size: usize,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            cache_dir: default_cache_dir(),
            memory_cache_size: DEFAULT_MEMORY_CACHE_SIZE,
            disk_cache_size: DEFAULT_DISK_CACHE_SIZE,
            http_cache_size: DEFAULT_HTTP_CACHE_SIZE,
            image_ttl: DEFAULT_IMAGE_TTL,
            font_ttl: DEFAULT_FONT_TTL,
            style_ttl: DEFAULT_STYLE_TTL,
            http_ttl: DEFAULT_HTTP_TTL,
            enable_disk_cache: true,
            enable_http_cache: true,
            respect_cache_control: true,
            enable_etag: true,
            max_entry_size: 10 * 1024 * 1024, // 10MB
        }
    }
}

impl CacheConfig {
    /// Create a new cache configuration with defaults
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a disabled configuration
    pub fn disabled() -> Self {
        Self {
            enabled: false,
            ..Default::default()
        }
    }

    /// Set cache directory
    pub fn with_cache_dir(mut self, dir: impl Into<PathBuf>) -> Self {
        self.cache_dir = dir.into();
        self
    }

    /// Set memory cache size
    pub fn with_memory_size(mut self, size: usize) -> Self {
        self.memory_cache_size = size;
        self
    }

    /// Set disk cache size
    pub fn with_disk_size(mut self, size: usize) -> Self {
        self.disk_cache_size = size;
        self
    }

    /// Set image TTL
    pub fn with_image_ttl(mut self, ttl: Duration) -> Self {
        self.image_ttl = ttl;
        self
    }

    /// Set font TTL
    pub fn with_font_ttl(mut self, ttl: Duration) -> Self {
        self.font_ttl = ttl;
        self
    }

    /// Set style TTL
    pub fn with_style_ttl(mut self, ttl: Duration) -> Self {
        self.style_ttl = ttl;
        self
    }

    /// Set HTTP TTL (CLI --cache-ttl)
    pub fn with_http_ttl(mut self, ttl: Duration) -> Self {
        self.http_ttl = ttl;
        self
    }

    /// Disable disk cache
    pub fn without_disk_cache(mut self) -> Self {
        self.enable_disk_cache = false;
        self
    }

    /// Disable HTTP cache
    pub fn without_http_cache(mut self) -> Self {
        self.enable_http_cache = false;
        self
    }

    /// Disable caching (CLI --no-cache)
    pub fn disable_cache(mut self) -> Self {
        self.enabled = false;
        self
    }

    /// Check if caching is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Get the subdirectory for a cache type
    pub fn subdir(&self, name: &str) -> PathBuf {
        self.cache_dir.join(name)
    }
}

/// Get the default cache directory
fn default_cache_dir() -> PathBuf {
    // Use platform-specific cache directory
    if let Some(cache_dir) = dirs::cache_dir() {
        cache_dir.join("html2pdf")
    } else {
        // Fallback to temp directory
        std::env::temp_dir().join("html2pdf-cache")
    }
}

/// Simple dirs module for cross-platform cache directory detection
mod dirs {
    use std::path::PathBuf;

    pub fn cache_dir() -> Option<PathBuf> {
        #[cfg(target_os = "macos")]
        {
            std::env::var_os("HOME").map(|home| {
                PathBuf::from(home).join("Library/Caches")
            })
        }

        #[cfg(target_os = "linux")]
        {
            std::env::var_os("XDG_CACHE_HOME")
                .map(PathBuf::from)
                .or_else(|| {
                    std::env::var_os("HOME").map(|home| {
                        PathBuf::from(home).join(".cache")
                    })
                })
        }

        #[cfg(target_os = "windows")]
        {
            std::env::var_os("LOCALAPPDATA").map(PathBuf::from)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = CacheConfig::default();
        assert!(config.enabled);
        assert!(config.enable_disk_cache);
        assert!(config.enable_http_cache);
        assert_eq!(config.memory_cache_size, DEFAULT_MEMORY_CACHE_SIZE);
    }

    #[test]
    fn test_disabled_config() {
        let config = CacheConfig::disabled();
        assert!(!config.enabled);
    }

    #[test]
    fn test_builder_methods() {
        let config = CacheConfig::new()
            .with_memory_size(50 * 1024 * 1024)
            .without_disk_cache()
            .with_image_ttl(Duration::from_secs(3600));

        assert_eq!(config.memory_cache_size, 50 * 1024 * 1024);
        assert!(!config.enable_disk_cache);
        assert_eq!(config.image_ttl, Duration::from_secs(3600));
    }
}
