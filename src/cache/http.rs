//! HTTP cache with Cache-Control and ETag support
//!
//! Implements HTTP caching semantics including:
//! - Cache-Control header parsing
//! - ETag-based validation
//! - Conditional requests
//! - Freshness lifetime calculation

use super::config::CacheConfig;
use super::disk::{DiskCache, SharedDiskCache};
use super::types::{CacheKey, CacheState, HttpCacheHeaders, ResourceType};
use std::collections::HashMap;
use std::io;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// HTTP cache entry
#[derive(Debug, Clone)]
pub struct HttpCacheEntry {
    /// Cached response body
    pub data: Vec<u8>,
    /// HTTP headers
    pub headers: HttpCacheHeaders,
    /// When the entry was stored
    pub stored_at: SystemTime,
    /// Original URL
    pub url: String,
}

impl HttpCacheEntry {
    /// Calculate freshness lifetime based on Cache-Control
    pub fn freshness_lifetime(&self) -> Duration {
        // Check for immutable
        if self.headers.immutable() {
            return Duration::from_secs(365 * 86400); // 1 year
        }

        // Check max-age
        if let Some(max_age) = self.headers.max_age() {
            return max_age;
        }

        // Default TTL
        Duration::from_secs(3600) // 1 hour default
    }

    /// Check if the entry is fresh
    pub fn is_fresh(&self) -> bool {
        let age = SystemTime::now()
            .duration_since(self.stored_at)
            .unwrap_or_default();
        age < self.freshness_lifetime()
    }

    /// Check if the entry needs revalidation
    pub fn needs_revalidation(&self) -> bool {
        if self.headers.no_cache() || self.headers.must_revalidate() {
            return true;
        }
        !self.is_fresh()
    }

    /// Get the current cache state
    pub fn state(&self) -> CacheState {
        if self.is_fresh() {
            CacheState::Fresh
        } else if self.needs_revalidation() {
            CacheState::Stale
        } else {
            CacheState::Expired
        }
    }

    /// Create conditional request headers for validation
    pub fn conditional_headers(&self) -> Vec<(String, String)> {
        let mut headers = Vec::new();

        if let Some(ref etag) = self.headers.etag {
            headers.push(("If-None-Match".to_string(), etag.clone()));
        }

        if let Some(ref last_modified) = self.headers.last_modified {
            headers.push(("If-Modified-Since".to_string(), last_modified.clone()));
        }

        headers
    }
}

/// HTTP response for caching
#[derive(Debug, Clone)]
pub struct HttpResponse {
    /// Status code
    pub status: u16,
    /// Response headers
    pub headers: HashMap<String, String>,
    /// Response body
    pub body: Vec<u8>,
    /// Final URL after redirects
    pub url: String,
}

impl HttpResponse {
    /// Check if response is cacheable
    pub fn is_cacheable(&self) -> bool {
        // Only cache successful responses
        if self.status != 200 {
            return false;
        }

        // Check for no-store directive
        if let Some(cc) = self.headers.get("cache-control") {
            if cc.contains("no-store") {
                return false;
            }
        }

        // Check for private directive (we cache as private)
        if let Some(cc) = self.headers.get("cache-control") {
            if cc.contains("private") || cc.contains("public") || cc.contains("max-age") {
                return true;
            }
        }

        // Check for Expires header
        if self.headers.contains_key("expires") {
            return true;
        }

        // Check for ETag or Last-Modified
        if self.headers.contains_key("etag") || self.headers.contains_key("last-modified") {
            return true;
        }

        // Default: don't cache without explicit cache headers
        false
    }

    /// Extract cache headers
    pub fn cache_headers(&self) -> HttpCacheHeaders {
        HttpCacheHeaders {
            etag: self.headers.get("etag").cloned(),
            last_modified: self.headers.get("last-modified").cloned(),
            cache_control: self.headers.get("cache-control").cloned(),
            expires: self.headers.get("expires").cloned(),
            content_length: self.headers.get("content-length").and_then(|v| v.parse().ok()),
            content_type: self.headers.get("content-type").cloned(),
        }
    }

    /// Get effective TTL based on cache headers
    pub fn effective_ttl(&self, default_ttl: Duration) -> Duration {
        let headers = self.cache_headers();

        // Check for immutable
        if headers.immutable() {
            return Duration::from_secs(365 * 86400);
        }

        // Check max-age
        if let Some(max_age) = headers.max_age() {
            return max_age;
        }

        // Parse Expires header
        if let Some(ref expires) = headers.expires {
            if let Some(expires_time) = parse_http_date(expires) {
                if let Ok(now) = SystemTime::now().duration_since(UNIX_EPOCH) {
                    let expires_secs = expires_time
                        .duration_since(UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs();
                    let now_secs = now.as_secs();
                    if expires_secs > now_secs {
                        return Duration::from_secs(expires_secs - now_secs);
                    }
                }
            }
        }

        default_ttl
    }
}

/// HTTP cache implementation
pub struct HttpCache {
    /// Underlying disk storage
    disk_cache: Option<SharedDiskCache>,
    /// Pending requests (for request coalescing)
    pending: Mutex<HashMap<String, Vec<std::sync::mpsc::Sender<Option<HttpCacheEntry>>>>>,
    /// Default TTL
    default_ttl: Duration,
    /// Respect Cache-Control headers
    respect_cache_control: bool,
    /// Enable ETag validation
    enable_etag: bool,
}

impl HttpCache {
    /// Create a new HTTP cache
    pub fn new(cache_dir: impl AsRef<Path>, max_size: usize, default_ttl: Duration) -> io::Result<Self> {
        let disk_cache = DiskCache::new(&cache_dir, max_size, ResourceType::Http)
            .ok()
            .map(|c| SharedDiskCache::new(&cache_dir, max_size, ResourceType::Http).ok())
            .flatten();

        Ok(Self {
            disk_cache,
            pending: Mutex::new(HashMap::new()),
            default_ttl,
            respect_cache_control: true,
            enable_etag: true,
        })
    }

    /// Create from cache configuration
    pub fn from_config(config: &CacheConfig) -> Option<Self> {
        if !config.enabled || !config.enable_http_cache {
            return None;
        }

        Self::new(
            &config.cache_dir,
            config.http_cache_size,
            config.http_ttl,
        )
        .ok()
    }

    /// Get a cached response
    pub fn get(&self, url: &str) -> Option<HttpCacheEntry> {
        let disk = self.disk_cache.as_ref()?;
        let key = CacheKey::new(url);

        // Try to get from disk
        let (data, meta) = disk.get_with_meta(&key)?;

        // Parse metadata
        let headers = HttpCacheHeaders {
            etag: meta.etag,
            last_modified: None,
            cache_control: None,
            content_type: meta.content_type,
            expires: None,
            content_length: Some(meta.size),
        };

        Some(HttpCacheEntry {
            data,
            headers,
            stored_at: SystemTime::UNIX_EPOCH + Duration::from_secs(meta.created_at),
            url: url.to_string(),
        })
    }

    /// Store a response in the cache
    pub fn store(&self, response: &HttpResponse) -> bool {
        if !response.is_cacheable() {
            return false;
        }

        let disk = match self.disk_cache.as_ref() {
            Some(d) => d,
            None => return false,
        };

        let key = CacheKey::new(&response.url);
        let ttl = if self.respect_cache_control {
            response.effective_ttl(self.default_ttl)
        } else {
            self.default_ttl
        };

        let headers = response.cache_headers();

        disk.put(
            &key,
            &response.body,
            ttl,
            headers.content_type,
            headers.etag,
        )
    }

    /// Store raw data with headers
    pub fn store_raw(
        &self,
        url: &str,
        data: &[u8],
        headers: &HttpCacheHeaders,
    ) -> bool {
        let disk = match self.disk_cache.as_ref() {
            Some(d) => d,
            None => return false,
        };

        let key = CacheKey::new(url);

        let ttl = if self.respect_cache_control {
            if let Some(max_age) = headers.max_age() {
                max_age
            } else if headers.immutable() {
                Duration::from_secs(365 * 86400)
            } else {
                self.default_ttl
            }
        } else {
            self.default_ttl
        };

        disk.put(&key, data, ttl, headers.content_type.clone(), headers.etag.clone())
    }

    /// Invalidate an entry
    pub fn invalidate(&self, url: &str) -> bool {
        let disk = match self.disk_cache.as_ref() {
            Some(d) => d,
            None => return false,
        };

        let key = CacheKey::new(url);
        disk.remove(&key)
    }

    /// Check if URL is cached
    pub fn is_cached(&self, url: &str) -> bool {
        let disk = match self.disk_cache.as_ref() {
            Some(d) => d,
            None => return false,
        };

        let key = CacheKey::new(url);
        disk.contains(&key)
    }

    /// Get cache state for URL
    pub fn state(&self, url: &str) -> CacheState {
        match self.get(url) {
            Some(entry) => entry.state(),
            None => CacheState::Missing,
        }
    }

    /// Clear all cached responses
    pub fn clear(&self) -> io::Result<()> {
        if let Some(disk) = self.disk_cache.as_ref() {
            disk.clear()?;
        }
        Ok(())
    }

    /// Get cache statistics
    pub fn stats(&self) -> super::types::CacheStats {
        self.disk_cache
            .as_ref()
            .map(|d| d.stats())
            .unwrap_or_default()
    }

    /// Clean up expired entries
    pub fn cleanup_expired(&self) -> usize {
        self.disk_cache
            .as_ref()
            .map(|d| d.cleanup_expired())
            .unwrap_or(0)
    }
}

/// Thread-safe HTTP cache wrapper
#[derive(Clone)]
pub struct SharedHttpCache {
    inner: Arc<HttpCache>,
}

impl SharedHttpCache {
    /// Create a new shared HTTP cache
    pub fn new(cache_dir: impl AsRef<Path>, max_size: usize, default_ttl: Duration) -> io::Result<Self> {
        Ok(Self {
            inner: Arc::new(HttpCache::new(cache_dir, max_size, default_ttl)?),
        })
    }

    /// Create from config
    pub fn from_config(config: &CacheConfig) -> Option<Self> {
        HttpCache::from_config(config).map(|c| Self {
            inner: Arc::new(c),
        })
    }

    /// Get cached response
    pub fn get(&self, url: &str) -> Option<HttpCacheEntry> {
        self.inner.get(url)
    }

    /// Store response
    pub fn store(&self, response: &HttpResponse) -> bool {
        self.inner.store(response)
    }

    /// Store raw data
    pub fn store_raw(&self, url: &str, data: &[u8], headers: &HttpCacheHeaders) -> bool {
        self.inner.store_raw(url, data, headers)
    }

    /// Invalidate entry
    pub fn invalidate(&self, url: &str) -> bool {
        self.inner.invalidate(url)
    }

    /// Check if cached
    pub fn is_cached(&self, url: &str) -> bool {
        self.inner.is_cached(url)
    }

    /// Get cache state
    pub fn state(&self, url: &str) -> CacheState {
        self.inner.state(url)
    }

    /// Clear cache
    pub fn clear(&self) -> io::Result<()> {
        self.inner.clear()
    }

    /// Get stats
    pub fn stats(&self) -> super::types::CacheStats {
        self.inner.stats()
    }

    /// Clean up expired entries
    pub fn cleanup_expired(&self) -> usize {
        self.inner.cleanup_expired()
    }
}

/// Parse HTTP date format (RFC 7231)
fn parse_http_date(date: &str) -> Option<SystemTime> {
    // Common formats:
    // Sun, 06 Nov 1994 08:49:37 GMT
    // Sunday, 06-Nov-94 08:49:37 GMT
    // Sun Nov  6 08:49:37 1994

    // Try RFC 2822 format first (most common)
    if let Ok(dt) = chrono::DateTime::parse_from_rfc2822(date) {
        return Some(SystemTime::UNIX_EPOCH + Duration::from_secs(dt.timestamp() as u64));
    }

    // Try RFC 3339 format
    if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(date) {
        return Some(SystemTime::UNIX_EPOCH + Duration::from_secs(dt.timestamp() as u64));
    }

    // Manual parsing for other formats
    parse_http_date_manual(date)
}

fn parse_http_date_manual(date: &str) -> Option<SystemTime> {
    let parts: Vec<&str> = date.split_whitespace().collect();

    // Need at least: day month year time
    if parts.len() < 4 {
        return None;
    }

    // Try to find day, month, year, time
    let mut day = 0u32;
    let mut month = 0u32;
    let mut year = 0i32;
    let mut time_str = "";

    for part in &parts {
        // Day
        if day == 0 {
            if let Ok(d) = part.parse::<u32>() {
                day = d;
                continue;
            }
        }

        // Month
        if month == 0 {
            month = parse_month(part);
            if month > 0 {
                continue;
            }
        }

        // Year
        if year == 0 {
            if let Ok(y) = part.parse::<i32>() {
                year = y;
                continue;
            }
        }

        // Time (contains colon)
        if part.contains(':') {
            time_str = part;
        }
    }

    if day == 0 || month == 0 || year == 0 || time_str.is_empty() {
        return None;
    }

    // Normalize year
    if year < 100 {
        year += if year >= 70 { 1900 } else { 2000 };
    }

    // Parse time
    let time_parts: Vec<&str> = time_str.split(':').collect();
    if time_parts.len() != 3 {
        return None;
    }

    let hour: u32 = time_parts[0].parse().ok()?;
    let minute: u32 = time_parts[1].parse().ok()?;
    let second: u32 = time_parts[2].parse().ok()?;

    // Create timestamp (simplified, assumes GMT)
    let days_since_epoch = days_since_1970(year, month, day);
    let seconds = days_since_epoch * 86400 + (hour * 3600 + minute * 60 + second) as i64;

    Some(SystemTime::UNIX_EPOCH + Duration::from_secs(seconds as u64))
}

fn parse_month(month: &str) -> u32 {
    match month.to_ascii_lowercase().as_str() {
        "jan" | "january" => 1,
        "feb" | "february" => 2,
        "mar" | "march" => 3,
        "apr" | "april" => 4,
        "may" => 5,
        "jun" | "june" => 6,
        "jul" | "july" => 7,
        "aug" | "august" => 8,
        "sep" | "september" => 9,
        "oct" | "october" => 10,
        "nov" | "november" => 11,
        "dec" | "december" => 12,
        _ => 0,
    }
}

fn is_leap_year(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}

fn days_since_1970(year: i32, month: u32, day: u32) -> i64 {
    const DAYS_IN_MONTH: [i32; 12] = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];

    let mut days = 0i64;

    // Years
    for y in 1970..year {
        days += if is_leap_year(y) { 366 } else { 365 };
    }

    // Months
    for m in 1..month {
        days += DAYS_IN_MONTH[(m - 1) as usize] as i64;
        if m == 2 && is_leap_year(year) {
            days += 1;
        }
    }

    // Days
    days += (day - 1) as i64;

    days
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_http_response_cacheable() {
        let response = HttpResponse {
            status: 200,
            headers: {
                let mut h = HashMap::new();
                h.insert("cache-control".to_string(), "max-age=3600".to_string());
                h
            },
            body: vec![1, 2, 3],
            url: "https://example.com/test".to_string(),
        };

        assert!(response.is_cacheable());
        assert_eq!(response.effective_ttl(Duration::from_secs(60)), Duration::from_secs(3600));
    }

    #[test]
    fn test_http_response_not_cacheable() {
        let response = HttpResponse {
            status: 404,
            headers: HashMap::new(),
            body: vec![],
            url: "https://example.com/notfound".to_string(),
        };

        assert!(!response.is_cacheable());
    }

    #[test]
    fn test_http_cache_headers_parsing() {
        let response = HttpResponse {
            status: 200,
            headers: {
                let mut h = HashMap::new();
                h.insert("cache-control".to_string(), "max-age=3600, must-revalidate".to_string());
                h.insert("etag".to_string(), "\"abc123\"".to_string());
                h.insert("content-type".to_string(), "image/png".to_string());
                h
            },
            body: vec![],
            url: "https://example.com/image.png".to_string(),
        };

        let headers = response.cache_headers();
        assert_eq!(headers.max_age(), Some(Duration::from_secs(3600)));
        assert!(headers.must_revalidate());
        assert_eq!(headers.etag, Some("\"abc123\"".to_string()));
    }

    #[test]
    fn test_http_date_parsing() {
        let date = "Sun, 06 Nov 1994 08:49:37 GMT";
        let parsed = parse_http_date(date);
        assert!(parsed.is_some());
    }

    #[test]
    fn test_cache_entry_state() {
        let entry = HttpCacheEntry {
            data: vec![1, 2, 3],
            headers: HttpCacheHeaders {
                cache_control: Some("max-age=0".to_string()),
                ..Default::default()
            },
            stored_at: SystemTime::now() - Duration::from_secs(10),
            url: "test".to_string(),
        };

        assert!(!entry.is_fresh());
        assert!(entry.needs_revalidation());
        assert_eq!(entry.state(), CacheState::Stale);
    }
}
