//! Resource-specific cache implementations
//!
//! Provides specialized caches for different resource types:
//! - ImageCache: Decoded images ready for PDF embedding
//! - FontCache: Loaded font data
//! - StyleCache: Parsed stylesheets
//! - LayoutCache: Computed layouts (optional)

use super::config::CacheConfig;
use super::disk::SharedDiskCache;
use super::memory::SharedMemoryCache;
use super::types::{CacheKey, CacheStats, ResourceType};
// Note: We use function-based access to avoid complex type dependencies
// The actual types are accessed through the crate root when needed
use std::io;
use std::sync::Arc;
use std::time::Duration;

/// Image cache for decoded images
#[derive(Clone)]
pub struct ImageCache {
    /// Memory cache for fast access
    memory: SharedMemoryCache,
    /// Disk cache for persistence
    disk: Option<SharedDiskCache>,
    /// Default TTL
    ttl: Duration,
    /// Maximum single image size
    max_image_size: usize,
}

impl ImageCache {
    /// Create a new image cache
    pub fn new(config: &CacheConfig) -> Self {
        let memory = SharedMemoryCache::with_capacity(config.memory_cache_size / 4); // 25% for images
        let disk = if config.enable_disk_cache {
            SharedDiskCache::from_config(config, ResourceType::Image)
        } else {
            None
        };

        Self {
            memory,
            disk,
            ttl: config.image_ttl,
            max_image_size: config.max_entry_size,
        }
    }

    /// Get an image from cache
    pub fn get(&self, key: &CacheKey) -> Option<Arc<crate::pdf::image::PdfImage>> {
        // Try memory first
        if let Some(img) = self.memory.get::<Arc<crate::pdf::image::PdfImage>>(key) {
            return Some(img);
        }

        // Try disk
        if let Some(ref disk) = self.disk {
            if let Some(data) = disk.get(key) {
                // Deserialize and add to memory cache
                if let Ok(img) = crate::pdf::image::PdfImage::from_bytes(&data) {
                    let arc_img = Arc::new(img);
                    let size = data.len();
                    self.memory.insert(key, arc_img.clone(), size, self.ttl);
                    return Some(arc_img);
                }
            }
        }

        None
    }

    /// Store an image in cache
    pub fn put(&self, key: &CacheKey, image: Arc<crate::pdf::image::PdfImage>) -> bool {
        // Calculate size (estimate)
        let size = image.width as usize * image.height as usize * 4 + 100;
        if size > self.max_image_size {
            return false;
        }

        // Store in memory only (images are complex to serialize)
        self.memory.insert(key, image, size, self.ttl);

        true
    }

    /// Store raw image data
    pub fn put_raw(&self, key: &CacheKey, data: &[u8]) -> bool {
        if data.len() > self.max_image_size {
            return false;
        }

        // Try to decode image
        if let Ok(img) = crate::pdf::image::PdfImage::from_bytes(data) {
            self.put(key, Arc::new(img))
        } else {
            false
        }
    }

    /// Remove from cache
    pub fn remove(&self, key: &CacheKey) {
        self.memory.remove(key);
        if let Some(ref disk) = self.disk {
            disk.remove(key);
        }
    }

    /// Clear all images
    pub fn clear(&self) -> io::Result<()> {
        self.memory.clear();
        if let Some(ref disk) = self.disk {
            disk.clear()?;
        }
        Ok(())
    }

    /// Get statistics
    pub fn stats(&self) -> CacheStats {
        let mut stats = self.memory.stats();
        if let Some(ref disk) = self.disk {
            let disk_stats = disk.stats();
            stats.entries += disk_stats.entries;
            stats.total_size += disk_stats.total_size;
            stats.hits += disk_stats.hits;
            stats.misses += disk_stats.misses;
        }
        stats
    }

    /// Clean up expired entries
    pub fn cleanup(&self) {
        self.memory.cleanup_expired();
        if let Some(ref disk) = self.disk {
            disk.cleanup_expired();
        }
    }
}

/// Font cache for loaded fonts
#[derive(Clone)]
pub struct FontCache {
    /// Memory cache
    memory: SharedMemoryCache,
    /// Disk cache
    disk: Option<SharedDiskCache>,
    /// Default TTL
    ttl: Duration,
    /// Maximum font size
    max_font_size: usize,
}

impl FontCache {
    /// Create a new font cache
    pub fn new(config: &CacheConfig) -> Self {
        let memory = SharedMemoryCache::with_capacity(config.memory_cache_size / 4);
        let disk = if config.enable_disk_cache {
            SharedDiskCache::from_config(config, ResourceType::Font)
        } else {
            None
        };

        Self {
            memory,
            disk,
            ttl: config.font_ttl,
            max_font_size: config.max_entry_size,
        }
    }

    /// Get a font from cache
    pub fn get(&self, key: &CacheKey) -> Option<Arc<crate::font::LoadedFont>> {
        // Try memory
        if let Some(font) = self.memory.get::<Arc<crate::font::LoadedFont>>(key) {
            return Some(font);
        }

        // Try disk - fonts are stored as raw data
        if let Some(ref disk) = self.disk {
            if let Some(data) = disk.get(key) {
                // Font data is stored raw, not as LoadedFont
                // The caller needs to parse it
                // For now, just return None for disk-loaded fonts
                // In a full implementation, we'd deserialize the LoadedFont
                let _ = data; // suppress unused warning
            }
        }

        None
    }

    /// Store a font in cache
    pub fn put(&self, key: &CacheKey, font: Arc<crate::font::LoadedFont>) -> bool {
        // Calculate size (approximate)
        let size = font.data.as_ref().map(|d: &Vec<u8>| d.len()).unwrap_or(1024);

        if size > self.max_font_size {
            return false;
        }

        // Store in memory
        self.memory.insert(key, font, size, self.ttl);

        // Note: disk storage would require access to raw font data
        // which is stored in font.data: Option<Vec<u8>>

        true
    }

    /// Store raw font data
    pub fn put_raw(&self, key: &CacheKey, data: &[u8]) -> bool {
        if data.len() > self.max_font_size {
            return false;
        }

        if let Some(ref disk) = self.disk {
            disk.put(key, data, self.ttl, Some("font/ttf".to_string()), None);
        }

        true
    }

    /// Remove from cache
    pub fn remove(&self, key: &CacheKey) {
        self.memory.remove(key);
        if let Some(ref disk) = self.disk {
            disk.remove(key);
        }
    }

    /// Clear all fonts
    pub fn clear(&self) -> io::Result<()> {
        self.memory.clear();
        if let Some(ref disk) = self.disk {
            disk.clear()?;
        }
        Ok(())
    }

    /// Get statistics
    pub fn stats(&self) -> CacheStats {
        let mut stats = self.memory.stats();
        if let Some(ref disk) = self.disk {
            let disk_stats = disk.stats();
            stats.entries += disk_stats.entries;
            stats.total_size += disk_stats.total_size;
        }
        stats
    }

    /// Clean up expired
    pub fn cleanup(&self) {
        self.memory.cleanup_expired();
        if let Some(ref disk) = self.disk {
            disk.cleanup_expired();
        }
    }
}

/// Style cache for parsed stylesheets
#[derive(Clone)]
pub struct StyleCache {
    /// Memory cache (stylesheets are small, keep in memory)
    memory: SharedMemoryCache,
    /// Disk cache for parsed CSS
    disk: Option<SharedDiskCache>,
    /// Default TTL
    ttl: Duration,
    /// Maximum stylesheet size
    max_size: usize,
}

impl StyleCache {
    /// Create a new style cache
    pub fn new(config: &CacheConfig) -> Self {
        let memory = SharedMemoryCache::with_capacity(config.memory_cache_size / 8);
        let disk = if config.enable_disk_cache {
            SharedDiskCache::from_config(config, ResourceType::Stylesheet)
        } else {
            None
        };

        Self {
            memory,
            disk,
            ttl: config.style_ttl,
            max_size: config.max_entry_size,
        }
    }

    /// Get a stylesheet from cache
    pub fn get(&self, key: &CacheKey) -> Option<Arc<crate::css::Stylesheet>> {
        // Try memory
        if let Some(stylesheet) = self.memory.get::<Arc<crate::css::Stylesheet>>(key) {
            return Some(stylesheet);
        }

        // Try disk
        if let Some(ref disk) = self.disk {
            if let Some(data) = disk.get(key) {
                if let Ok(css) = String::from_utf8(data) {
                    if let Ok(stylesheet) = crate::css::parse_stylesheet(&css) {
                        let arc_sheet: Arc<crate::css::Stylesheet> = Arc::new(stylesheet);
                        let size = data.len();
                        self.memory.insert(key, arc_sheet.clone(), size, self.ttl);
                        return Some(arc_sheet);
                    }
                }
            }
        }

        None
    }

    /// Store a stylesheet in cache
    pub fn put(&self, key: &CacheKey, stylesheet: Arc<crate::css::Stylesheet>) -> bool {
        // Estimate size
        let size = stylesheet.rules.len() * 100;

        if size > self.max_size {
            return false;
        }

        // Store in memory only (stylesheets are small)
        self.memory.insert(key, stylesheet, size, self.ttl);

        true
    }

    /// Store raw CSS
    pub fn put_raw(&self, key: &CacheKey, css: &str) -> bool {
        let data = css.as_bytes();

        if data.len() > self.max_size {
            return false;
        }

        // Parse and store
        if let Ok(stylesheet) = crate::css::parse_stylesheet(css) {
            self.put(key, Arc::new(stylesheet))
        } else {
            // Cache raw CSS on disk even if parsing fails
            if let Some(ref disk) = self.disk {
                disk.put(key, data, self.ttl, Some("text/css".to_string()), None);
            }
            false
        }
    }

    /// Remove from cache
    pub fn remove(&self, key: &CacheKey) {
        self.memory.remove(key);
        if let Some(ref disk) = self.disk {
            disk.remove(key);
        }
    }

    /// Clear all styles
    pub fn clear(&self) -> io::Result<()> {
        self.memory.clear();
        if let Some(ref disk) = self.disk {
            disk.clear()?;
        }
        Ok(())
    }

    /// Get statistics
    pub fn stats(&self) -> CacheStats {
        let mut stats = self.memory.stats();
        if let Some(ref disk) = self.disk {
            let disk_stats = disk.stats();
            stats.entries += disk_stats.entries;
            stats.total_size += disk_stats.total_size;
        }
        stats
    }

    /// Clean up expired
    pub fn cleanup(&self) {
        self.memory.cleanup_expired();
        if let Some(ref disk) = self.disk {
            disk.cleanup_expired();
        }
    }
}

/// Layout cache for computed layouts (optional, typically memory-only)
#[derive(Clone)]
pub struct LayoutCache {
    /// Memory cache only (layouts are document-specific)
    memory: SharedMemoryCache,
    /// Maximum entries
    max_entries: usize,
    /// Current entry count
    entry_count: std::sync::Arc<std::sync::atomic::AtomicUsize>,
}

impl LayoutCache {
    /// Create a new layout cache
    pub fn new(config: &CacheConfig) -> Self {
        // Layout cache uses small portion of memory
        let memory = SharedMemoryCache::with_capacity(config.memory_cache_size / 8);

        Self {
            memory,
            max_entries: 10, // Keep only recent layouts
            entry_count: std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0)),
        }
    }

    /// Get a layout from cache
    pub fn get(&self, key: &CacheKey) -> Option<Arc<crate::layout::LayoutBox>> {
        self.memory.get::<Arc<crate::layout::LayoutBox>>(key)
    }

    /// Store a layout in cache
    pub fn put(&self, key: &CacheKey, layout: Arc<crate::layout::LayoutBox>) -> bool {
        // Check entry limit
        let count = self.entry_count.load(std::sync::atomic::Ordering::SeqCst);
        if count >= self.max_entries {
            // Don't cache more layouts
            return false;
        }

        // Estimate size
        let size = std::mem::size_of::<LayoutBox>() * 2; // Rough estimate

        // Use short TTL (1 minute) since layouts depend on viewport
        let ttl = Duration::from_secs(60);

        if self.memory.insert(key, layout, size, ttl) {
            self.entry_count.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            true
        } else {
            false
        }
    }

    /// Remove from cache
    pub fn remove(&self, key: &CacheKey) {
        if self.memory.remove(key) {
            self.entry_count.fetch_sub(1, std::sync::atomic::Ordering::SeqCst);
        }
    }

    /// Clear all layouts
    pub fn clear(&self) {
        self.memory.clear();
        self.entry_count.store(0, std::sync::atomic::Ordering::SeqCst);
    }

    /// Get statistics
    pub fn stats(&self) -> CacheStats {
        self.memory.stats()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_image_cache() {
        let config = CacheConfig::default();
        let cache = ImageCache::new(&config);

        let key = CacheKey::new("test.png");
        assert!(cache.get(&key).is_none());
    }

    #[test]
    fn test_font_cache() {
        let config = CacheConfig::default();
        let cache = FontCache::new(&config);

        let key = CacheKey::new("test.ttf");
        assert!(cache.get(&key).is_none());
    }

    #[test]
    fn test_style_cache() {
        let config = CacheConfig::default();
        let cache = StyleCache::new(&config);

        let key = CacheKey::new("test.css");
        assert!(cache.get(&key).is_none());
    }

    #[test]
    fn test_layout_cache() {
        let config = CacheConfig::default();
        let cache = LayoutCache::new(&config);

        let key = CacheKey::new("test-layout");
        assert!(cache.get(&key).is_none());
    }
}
