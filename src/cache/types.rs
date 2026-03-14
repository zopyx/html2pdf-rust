//! Cache types for different resources
//!
//! Provides specialized cache types for images, fonts, stylesheets, and layouts.

// Type imports are done through the crate root when needed
use std::any::Any;
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Cache entry metadata
#[derive(Debug, Clone)]
pub struct CacheEntryMeta {
    /// When the entry was created
    pub created_at: Instant,
    /// Time-to-live duration
    pub ttl: Duration,
    /// Entry size in bytes
    pub size: usize,
    /// ETag for HTTP validation
    pub etag: Option<String>,
    /// Last-Modified timestamp
    pub last_modified: Option<String>,
    /// Cache-Control header value
    pub cache_control: Option<String>,
    /// Content-Type
    pub content_type: Option<String>,
    /// Access count for LRU tracking
    pub access_count: u64,
    /// Last access time
    pub last_accessed: Instant,
}

impl CacheEntryMeta {
    /// Create new metadata with current timestamp
    pub fn new(size: usize, ttl: Duration) -> Self {
        let now = Instant::now();
        Self {
            created_at: now,
            ttl,
            size,
            etag: None,
            last_modified: None,
            cache_control: None,
            content_type: None,
            access_count: 0,
            last_accessed: now,
        }
    }

    /// Check if entry has expired
    pub fn is_expired(&self) -> bool {
        self.created_at.elapsed() > self.ttl
    }

    /// Record an access
    pub fn record_access(&mut self) {
        self.access_count += 1;
        self.last_accessed = Instant::now();
    }

    /// Get remaining TTL
    pub fn remaining_ttl(&self) -> Option<Duration> {
        let elapsed = self.created_at.elapsed();
        if elapsed >= self.ttl {
            None
        } else {
            Some(self.ttl - elapsed)
        }
    }

    /// Update metadata from HTTP headers
    pub fn with_http_headers(
        mut self,
        etag: Option<String>,
        last_modified: Option<String>,
        cache_control: Option<String>,
    ) -> Self {
        self.etag = etag;
        self.last_modified = last_modified;
        self.cache_control = cache_control;
        self
    }
}

/// A generic cache entry that can hold any cached value
#[derive(Debug)]
pub struct CacheEntry<T> {
    /// The cached value
    pub value: T,
    /// Entry metadata
    pub meta: CacheEntryMeta,
}

impl<T> CacheEntry<T> {
    /// Create a new cache entry
    pub fn new(value: T, size: usize, ttl: Duration) -> Self {
        Self {
            value,
            meta: CacheEntryMeta::new(size, ttl),
        }
    }

    /// Check if entry has expired
    pub fn is_expired(&self) -> bool {
        self.meta.is_expired()
    }

    /// Record access
    pub fn record_access(&mut self) {
        self.meta.record_access();
    }
}

/// Type-erased cache entry for storage
#[derive(Debug)]
pub struct AnyCacheEntry {
    /// The cached value as Any
    pub value: Box<dyn Any + Send + Sync>,
    /// Entry metadata
    pub meta: CacheEntryMeta,
    /// Type name for debugging
    pub type_name: &'static str,
}

impl AnyCacheEntry {
    /// Create a new type-erased entry
    pub fn new<T: Any + Send + Sync>(value: T, size: usize, ttl: Duration) -> Self {
        Self {
            value: Box::new(value),
            meta: CacheEntryMeta::new(size, ttl),
            type_name: std::any::type_name::<T>(),
        }
    }

    /// Downcast to a concrete type
    pub fn downcast<T: Any + Clone>(self) -> Option<T> {
        self.value.downcast::<T>().ok().map(|b| *b)
    }

    /// Downcast reference to a concrete type
    pub fn downcast_ref<T: Any>(&self) -> Option<&T> {
        self.value.downcast_ref::<T>()
    }

    /// Check if expired
    pub fn is_expired(&self) -> bool {
        self.meta.is_expired()
    }

    /// Record access
    pub fn record_access(&mut self) {
        self.meta.record_access();
    }
}

/// Cache statistics
#[derive(Debug, Clone, Default)]
pub struct CacheStats {
    /// Total entries
    pub entries: usize,
    /// Total size in bytes
    pub total_size: usize,
    /// Number of hits
    pub hits: u64,
    /// Number of misses
    pub misses: u64,
    /// Number of evictions
    pub evictions: u64,
    /// Number of expired entries removed
    pub expired: u64,
}

impl CacheStats {
    /// Calculate hit rate (0.0 to 1.0)
    pub fn hit_rate(&self) -> f64 {
        let total = self.hits + self.misses;
        if total == 0 {
            0.0
        } else {
            self.hits as f64 / total as f64
        }
    }

    /// Record a hit
    pub fn record_hit(&mut self) {
        self.hits += 1;
    }

    /// Record a miss
    pub fn record_miss(&mut self) {
        self.misses += 1;
    }

    /// Record an eviction
    pub fn record_eviction(&mut self) {
        self.evictions += 1;
    }

    /// Record expired entry removal
    pub fn record_expired(&mut self) {
        self.expired += 1;
    }

    /// Reset counters (keep size/entries)
    pub fn reset_counters(&mut self) {
        self.hits = 0;
        self.misses = 0;
        self.evictions = 0;
        self.expired = 0;
    }
}

/// Cache key type (hashed string)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CacheKey(pub String);

impl CacheKey {
    /// Create a cache key from a string
    pub fn new(key: impl Into<String>) -> Self {
        Self(key.into())
    }

    /// Hash the key for storage
    pub fn hash_key(&self) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        self.0.hash(&mut hasher);
        format!("{:016x}", hasher.finish())
    }

    /// Get safe filename from key
    pub fn safe_filename(&self) -> String {
        // Use hash for safe filename
        self.hash_key()
    }
}

impl From<&str> for CacheKey {
    fn from(s: &str) -> Self {
        Self::new(s)
    }
}

impl From<String> for CacheKey {
    fn from(s: String) -> Self {
        Self::new(s)
    }
}

/// Specialized cache entry types
pub type ImageCacheEntry = CacheEntry<Arc<crate::pdf::image::PdfImage>>;
pub type FontCacheEntry = CacheEntry<Arc<crate::font::LoadedFont>>;
pub type StyleCacheEntry = CacheEntry<Arc<crate::css::Stylesheet>>;
pub type LayoutCacheEntry = CacheEntry<Arc<crate::layout::LayoutBox>>;

/// Resource type for cache organization
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ResourceType {
    Image,
    Font,
    Stylesheet,
    Layout,
    Http,
    Other,
}

impl ResourceType {
    /// Get subdirectory name for this resource type
    pub fn subdir(&self) -> &'static str {
        match self {
            ResourceType::Image => "images",
            ResourceType::Font => "fonts",
            ResourceType::Stylesheet => "styles",
            ResourceType::Layout => "layouts",
            ResourceType::Http => "http",
            ResourceType::Other => "other",
        }
    }

    /// Get default file extension
    pub fn extension(&self) -> &'static str {
        match self {
            ResourceType::Image => "img",
            ResourceType::Font => "font",
            ResourceType::Stylesheet => "css",
            ResourceType::Layout => "layout",
            ResourceType::Http => "http",
            ResourceType::Other => "bin",
        }
    }
}

/// HTTP cache headers
#[derive(Debug, Clone, Default)]
pub struct HttpCacheHeaders {
    pub etag: Option<String>,
    pub last_modified: Option<String>,
    pub cache_control: Option<String>,
    pub expires: Option<String>,
    pub content_length: Option<usize>,
    pub content_type: Option<String>,
}

impl HttpCacheHeaders {
    /// Parse max-age from Cache-Control header
    pub fn max_age(&self) -> Option<Duration> {
        let cc = self.cache_control.as_ref()?;
        for part in cc.split(',') {
            let part = part.trim();
            if let Some(value) = part.strip_prefix("max-age=") {
                return value.parse::<u64>().ok().map(Duration::from_secs);
            }
        }
        None
    }

    /// Check if no-cache directive is present
    pub fn no_cache(&self) -> bool {
        self.cache_control
            .as_ref()
            .map(|cc| cc.contains("no-cache") || cc.contains("no-store"))
            .unwrap_or(false)
    }

    /// Check if must-revalidate directive is present
    pub fn must_revalidate(&self) -> bool {
        self.cache_control
            .as_ref()
            .map(|cc| cc.contains("must-revalidate"))
            .unwrap_or(false)
    }

    /// Check if the response is immutable
    pub fn immutable(&self) -> bool {
        self.cache_control
            .as_ref()
            .map(|cc| cc.contains("immutable"))
            .unwrap_or(false)
    }
}

/// Cache entry state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CacheState {
    /// Entry is fresh and valid
    Fresh,
    /// Entry needs revalidation
    Stale,
    /// Entry is expired
    Expired,
    /// Entry not found
    Missing,
}

impl CacheState {
    /// Check if the entry is usable
    pub fn is_usable(&self) -> bool {
        matches!(self, CacheState::Fresh | CacheState::Stale)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_key() {
        let key = CacheKey::new("https://example.com/image.png");
        let hashed = key.hash_key();
        assert_eq!(hashed.len(), 16);
        assert!(hashed.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_cache_entry_expiration() {
        let entry = CacheEntry::new(42, 100, Duration::from_millis(50));
        assert!(!entry.is_expired());
        std::thread::sleep(Duration::from_millis(60));
        assert!(entry.is_expired());
    }

    #[test]
    fn test_cache_stats() {
        let mut stats = CacheStats::default();
        stats.record_hit();
        stats.record_hit();
        stats.record_miss();
        assert_eq!(stats.hits, 2);
        assert_eq!(stats.misses, 1);
        assert_eq!(stats.hit_rate(), 2.0 / 3.0);
    }

    #[test]
    fn test_http_cache_headers() {
        let headers = HttpCacheHeaders {
            cache_control: Some("max-age=3600, must-revalidate".to_string()),
            etag: Some("\"abc123\"".to_string()),
            ..Default::default()
        };

        assert_eq!(headers.max_age(), Some(Duration::from_secs(3600)));
        assert!(headers.must_revalidate());
        assert!(!headers.no_cache());
    }

    #[test]
    fn test_resource_type() {
        assert_eq!(ResourceType::Image.subdir(), "images");
        assert_eq!(ResourceType::Font.extension(), "font");
    }
}
