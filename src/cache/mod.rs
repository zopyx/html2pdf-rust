//! Caching system for html2pdf
//!
//! Provides comprehensive caching support for better performance:
//! - HTTP cache for fetched URLs with Cache-Control/ETag support
//! - Disk cache for persistent storage
//! - Memory cache with LRU eviction
//! - Resource-specific caches (images, fonts, stylesheets, layouts)
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │                    Cache Manager                             │
//! ├─────────────┬─────────────┬─────────────┬───────────────────┤
//! │ ImageCache  │  FontCache  │  StyleCache │   LayoutCache     │
//! │  (memory+   │  (memory+   │  (memory+   │    (memory)       │
//! │   disk)     │   disk)     │   disk)     │                   │
//! ├─────────────┴─────────────┴─────────────┴───────────────────┤
//! │                   HTTP Cache                                 │
//! │              (disk-backed with ETag)                         │
//! ├─────────────────────────────────────────────────────────────┤
//! │                   Disk Cache                                 │
//! │         (persistent file storage)                            │
//! ├─────────────────────────────────────────────────────────────┤
//! │                  Memory Cache                                │
//! │         (LRU eviction, thread-safe)                          │
//! └─────────────────────────────────────────────────────────────┘
//! ```
//!
//! # Usage
//!
//! ```rust,no_run
//! use html2pdf::cache::{CacheManager, CacheConfig};
//!
//! // Create with default configuration
//! let cache = CacheManager::default();
//!
//! // Or with custom configuration
//! let config = CacheConfig::new()
//!     .with_cache_dir("/tmp/html2pdf-cache")
//!     .with_memory_size(200 * 1024 * 1024)  // 200MB
//!     .with_image_ttl(std::time::Duration::from_secs(86400 * 7)); // 7 days
//!
//! let cache = CacheManager::with_config(&config);
//! ```

pub mod config;
pub mod disk;
pub mod http;
pub mod memory;
pub mod resource;
pub mod types;

pub use config::CacheConfig;
pub use disk::{DiskCache, SharedDiskCache};
pub use http::{HttpCache, HttpCacheEntry, HttpResponse, SharedHttpCache};
pub use types::HttpCacheHeaders;
pub use memory::{MemoryCache, SharedMemoryCache};
pub use resource::{FontCache, ImageCache, LayoutCache, StyleCache};
pub use types::{CacheEntry, CacheKey, CacheStats, CacheState, ResourceType};

use std::io;
use std::sync::Arc;

/// Central cache manager coordinating all cache types
#[derive(Clone)]
pub struct CacheManager {
    /// Configuration
    config: CacheConfig,
    /// Image cache
    pub images: ImageCache,
    /// Font cache
    pub fonts: FontCache,
    /// Stylesheet cache
    pub styles: StyleCache,
    /// Layout cache
    pub layouts: LayoutCache,
    /// HTTP cache
    pub http: Option<SharedHttpCache>,
    /// General disk cache
    pub disk: Option<SharedDiskCache>,
}

impl CacheManager {
    /// Create a new cache manager with the given configuration
    pub fn with_config(config: &CacheConfig) -> Self {
        let images = ImageCache::new(config);
        let fonts = FontCache::new(config);
        let styles = StyleCache::new(config);
        let layouts = LayoutCache::new(config);
        let http = SharedHttpCache::from_config(config);
        let disk = if config.enable_disk_cache {
            SharedDiskCache::from_config(config, ResourceType::Other)
        } else {
            None
        };

        Self {
            config: config.clone(),
            images,
            fonts,
            styles,
            layouts,
            http,
            disk,
        }
    }

    /// Create a disabled cache manager
    pub fn disabled() -> Self {
        Self::with_config(&CacheConfig::disabled())
    }

    /// Check if caching is enabled
    pub fn is_enabled(&self) -> bool {
        self.config.enabled
    }

    /// Get the cache configuration
    pub fn config(&self) -> &CacheConfig {
        &self.config
    }

    /// Get combined statistics from all caches
    pub fn stats(&self) -> CacheStats {
        let mut total = CacheStats::default();

        let image_stats = self.images.stats();
        total.entries += image_stats.entries;
        total.total_size += image_stats.total_size;
        total.hits += image_stats.hits;
        total.misses += image_stats.misses;
        total.evictions += image_stats.evictions;
        total.expired += image_stats.expired;

        let font_stats = self.fonts.stats();
        total.entries += font_stats.entries;
        total.total_size += font_stats.total_size;
        total.hits += font_stats.hits;
        total.misses += font_stats.misses;

        let style_stats = self.styles.stats();
        total.entries += style_stats.entries;
        total.total_size += style_stats.total_size;

        let layout_stats = self.layouts.stats();
        total.entries += layout_stats.entries;
        total.total_size += layout_stats.total_size;

        if let Some(ref http) = self.http {
            let http_stats = http.stats();
            total.entries += http_stats.entries;
            total.total_size += http_stats.total_size;
            total.hits += http_stats.hits;
            total.misses += http_stats.misses;
        }

        total
    }

    /// Clean up all expired entries
    pub fn cleanup_expired(&self) {
        self.images.cleanup();
        self.fonts.cleanup();
        self.styles.cleanup();
        self.layouts.clear();
        if let Some(ref http) = self.http {
            http.cleanup_expired();
        }
    }

    /// Clear all caches
    pub fn clear_all(&self) -> io::Result<()> {
        self.images.clear()?;
        self.fonts.clear()?;
        self.styles.clear()?;
        self.layouts.clear();
        if let Some(ref http) = self.http {
            http.clear()?;
        }
        if let Some(ref disk) = self.disk {
            disk.clear()?;
        }
        Ok(())
    }

    /// Clear specific cache type
    pub fn clear_cache(&self, resource_type: ResourceType) -> io::Result<()> {
        match resource_type {
            ResourceType::Image => self.images.clear(),
            ResourceType::Font => self.fonts.clear(),
            ResourceType::Stylesheet => self.styles.clear(),
            ResourceType::Layout => {
                self.layouts.clear();
                Ok(())
            }
            ResourceType::Http => {
                if let Some(ref http) = self.http {
                    http.clear()
                } else {
                    Ok(())
                }
            }
            ResourceType::Other => {
                if let Some(ref disk) = self.disk {
                    disk.clear()
                } else {
                    Ok(())
                }
            }
        }
    }

    /// Initialize cache directories
    pub fn initialize(&self) -> io::Result<()> {
        use std::fs;

        // Create main cache directory
        fs::create_dir_all(&self.config.cache_dir)?;

        // Create subdirectories
        for resource_type in [
            ResourceType::Image,
            ResourceType::Font,
            ResourceType::Stylesheet,
            ResourceType::Layout,
            ResourceType::Http,
            ResourceType::Other,
        ] {
            let subdir = self.config.cache_dir.join(resource_type.subdir());
            fs::create_dir_all(&subdir)?;
        }

        Ok(())
    }

    /// Get cache directory path
    pub fn cache_dir(&self) -> &std::path::Path {
        &self.config.cache_dir
    }

    /// Invalidate a specific URL from HTTP cache
    pub fn invalidate_url(&self, url: &str) -> bool {
        if let Some(ref http) = self.http {
            http.invalidate(url)
        } else {
            false
        }
    }

    /// Check if a URL is cached
    pub fn is_url_cached(&self, url: &str) -> bool {
        if let Some(ref http) = self.http {
            http.is_cached(url)
        } else {
            false
        }
    }

    /// Get the cache state for a URL
    pub fn url_cache_state(&self, url: &str) -> CacheState {
        if let Some(ref http) = self.http {
            http.state(url)
        } else {
            CacheState::Missing
        }
    }
}

impl Default for CacheManager {
    fn default() -> Self {
        Self::with_config(&CacheConfig::default())
    }
}

/// Global cache instance (optional singleton pattern)
static mut GLOBAL_CACHE: Option<Arc<CacheManager>> = None;
static GLOBAL_CACHE_INIT: std::sync::Once = std::sync::Once::new();

/// Initialize the global cache
pub fn initialize_global(config: &CacheConfig) -> Arc<CacheManager> {
    unsafe {
        GLOBAL_CACHE_INIT.call_once(|| {
            let cache = CacheManager::with_config(config);
            GLOBAL_CACHE = Some(Arc::new(cache));
        });
        GLOBAL_CACHE.clone().unwrap()
    }
}

/// Get the global cache (must call initialize_global first)
pub fn global_cache() -> Option<Arc<CacheManager>> {
    unsafe { GLOBAL_CACHE.clone() }
}

/// Reset the global cache
pub fn reset_global_cache() {
    unsafe {
        GLOBAL_CACHE = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_cache_manager_default() {
        let cache = CacheManager::default();
        assert!(cache.is_enabled());
    }

    #[test]
    fn test_cache_manager_disabled() {
        let cache = CacheManager::disabled();
        assert!(!cache.is_enabled());
    }

    #[test]
    fn test_cache_manager_with_config() {
        let config = CacheConfig::new()
            .with_memory_size(50 * 1024 * 1024)
            .with_image_ttl(Duration::from_secs(3600))
            .without_disk_cache();

        let cache = CacheManager::with_config(&config);
        assert!(cache.is_enabled());
        assert!(cache.http.is_none() || cache.disk.is_none());
    }

    #[test]
    fn test_cache_manager_stats() {
        let cache = CacheManager::default();
        let stats = cache.stats();
        // Should have some default values
        assert_eq!(stats.entries, 0); // No entries yet
    }

    #[test]
    fn test_cache_key_hashing() {
        let key1 = CacheKey::new("https://example.com/image.png");
        let key2 = CacheKey::new("https://example.com/image.png");
        let key3 = CacheKey::new("https://example.com/other.png");

        assert_eq!(key1.hash_key(), key2.hash_key());
        assert_ne!(key1.hash_key(), key3.hash_key());
    }
}
