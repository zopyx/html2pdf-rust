//! Disk cache for persistent storage
//!
//! Provides file-based caching with metadata tracking for
//! resources that should persist between sessions.

use super::config::CacheConfig;
use super::types::{CacheEntry, CacheKey, CacheStats, ResourceType};
use std::collections::HashMap;
use std::fs;
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, RwLock};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Disk cache entry metadata (stored alongside data)
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct DiskEntryMeta {
    /// Creation timestamp
    created_at: u64,
    /// TTL in seconds
    ttl_secs: u64,
    /// Original key
    key: String,
    /// Content type
    content_type: Option<String>,
    /// ETag
    etag: Option<String>,
    /// Size in bytes
    size: usize,
}

impl DiskEntryMeta {
    fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap_or_default()
    }

    fn from_json(json: &str) -> Option<Self> {
        serde_json::from_str(json).ok()
    }

    fn is_expired(&self) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        now > self.created_at + self.ttl_secs
    }
}

/// File-based disk cache
pub struct DiskCache {
    /// Base cache directory
    cache_dir: PathBuf,
    /// Maximum size in bytes
    max_size: usize,
    /// Current size tracking
    current_size: Mutex<usize>,
    /// Entry metadata cache
    entries: RwLock<HashMap<String, DiskEntryMeta>>,
    /// Statistics
    stats: Mutex<CacheStats>,
    /// Resource type for subdirectory
    resource_type: ResourceType,
}

impl DiskCache {
    /// Create a new disk cache
    pub fn new(cache_dir: impl AsRef<Path>, max_size: usize, resource_type: ResourceType) -> io::Result<Self> {
        let cache_dir = cache_dir.as_ref().join(resource_type.subdir());

        // Ensure directory exists
        fs::create_dir_all(&cache_dir)?;

        let cache = Self {
            cache_dir,
            max_size,
            current_size: Mutex::new(0),
            entries: RwLock::new(HashMap::new()),
            stats: Mutex::new(CacheStats::default()),
            resource_type,
        };

        // Load existing entries
        cache.load_existing_entries()?;

        Ok(cache)
    }

    /// Create from cache configuration
    pub fn from_config(config: &CacheConfig, resource_type: ResourceType) -> Option<Self> {
        if !config.enabled || !config.enable_disk_cache {
            return None;
        }

        Self::new(&config.cache_dir, config.disk_cache_size, resource_type).ok()
    }

    /// Get the path for a cache key
    fn entry_path(&self, key_hash: &str) -> PathBuf {
        self.cache_dir.join(format!("{}.bin", key_hash))
    }

    /// Get the metadata path
    fn meta_path(&self, key_hash: &str) -> PathBuf {
        self.cache_dir.join(format!("{}.json", key_hash))
    }

    /// Load existing entries from disk
    fn load_existing_entries(&self) -> io::Result<()> {
        let mut entries = self.entries.write().map_err(|_| {
            io::Error::new(io::ErrorKind::Other, "Lock poisoned")
        })?;

        let mut total_size = 0usize;

        if let Ok(dir_entries) = fs::read_dir(&self.cache_dir) {
            for entry in dir_entries.flatten() {
                let path = entry.path();
                if path.extension().map(|e| e == "json").unwrap_or(false) {
                    if let Ok(content) = fs::read_to_string(&path) {
                        if let Some(meta) = DiskEntryMeta::from_json(&content) {
                            // Check if data file exists
                            let data_path = path.with_extension("bin");
                            if data_path.exists() {
                                total_size += meta.size;
                                let key = path
                                    .file_stem()
                                    .and_then(|s| s.to_str())
                                    .unwrap_or("")
                                    .to_string();
                                entries.insert(key, meta);
                            }
                        }
                    }
                }
            }
        }

        if let Ok(mut size) = self.current_size.lock() {
            *size = total_size;
        }

        Ok(())
    }

    /// Get data from cache
    pub fn get(&self, key: &CacheKey) -> Option<Vec<u8>> {
        let key_hash = key.hash_key();

        // Check metadata
        let meta = {
            let entries = self.entries.read().ok()?;
            entries.get(&key_hash).cloned()
        }?;

        // Check expiration
        if meta.is_expired() {
            drop(meta);
            self.remove(key);
            self.stats.lock().ok()?.record_expired();
            return None;
        }

        // Read data
        let data_path = self.entry_path(&key_hash);
        match fs::read(&data_path) {
            Ok(data) => {
                self.stats.lock().ok()?.record_hit();
                Some(data)
            }
            Err(_) => {
                // File missing, clean up metadata
                self.remove(key);
                self.stats.lock().ok()?.record_miss();
                None
            }
        }
    }

    /// Get with metadata
    pub fn get_with_meta(&self, key: &CacheKey) -> Option<(Vec<u8>, DiskEntryMeta)> {
        let key_hash = key.hash_key();

        let meta = {
            let entries = self.entries.read().ok()?;
            entries.get(&key_hash).cloned()
        }?;

        if meta.is_expired() {
            drop(meta);
            self.remove(key);
            return None;
        }

        let data_path = self.entry_path(&key_hash);
        fs::read(&data_path).ok().map(|data| {
            self.stats.lock().ok().map(|mut s| s.record_hit());
            (data, meta)
        })
    }

    /// Store data in cache
    pub fn put(
        &self,
        key: &CacheKey,
        data: &[u8],
        ttl: Duration,
        content_type: Option<String>,
        etag: Option<String>,
    ) -> bool {
        if data.len() > self.max_size {
            return false;
        }

        let key_hash = key.hash_key();

        // Make room for new entry
        self.evict_for_size(data.len());

        // Remove old entry if exists
        self.remove(key);

        // Create metadata
        let meta = DiskEntryMeta {
            created_at: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            ttl_secs: ttl.as_secs(),
            key: key.0.clone(),
            content_type,
            etag,
            size: data.len(),
        };

        // Write metadata
        let meta_path = self.meta_path(&key_hash);
        if let Err(_) = fs::write(&meta_path, meta.to_json()) {
            return false;
        }

        // Write data
        let data_path = self.entry_path(&key_hash);
        if let Err(_) = fs::write(&data_path, data) {
            let _ = fs::remove_file(&meta_path);
            return false;
        }

        // Update tracking
        if let Ok(mut entries) = self.entries.write() {
            entries.insert(key_hash, meta);
        }

        if let Ok(mut size) = self.current_size.lock() {
            *size += data.len();
        }

        if let Ok(mut stats) = self.stats.lock() {
            stats.entries += 1;
            stats.total_size += data.len();
        }

        true
    }

    /// Remove an entry
    pub fn remove(&self, key: &CacheKey) -> bool {
        let key_hash = key.hash_key();

        let size = {
            let entries = self.entries.read();
            match entries {
                Ok(e) => e.get(&key_hash).map(|m| m.size).unwrap_or(0),
                Err(_) => return false,
            }
        };

        // Remove files
        let data_path = self.entry_path(&key_hash);
        let meta_path = self.meta_path(&key_hash);

        let _ = fs::remove_file(&data_path);
        let _ = fs::remove_file(&meta_path);

        // Update tracking
        if let Ok(mut entries) = self.entries.write() {
            entries.remove(&key_hash);
        }

        if let Ok(mut current_size) = self.current_size.lock() {
            *current_size = current_size.saturating_sub(size);
        }

        if let Ok(mut stats) = self.stats.lock() {
            stats.entries = stats.entries.saturating_sub(1);
            stats.total_size = stats.total_size.saturating_sub(size);
        }

        true
    }

    /// Check if entry exists
    pub fn contains(&self, key: &CacheKey) -> bool {
        let key_hash = key.hash_key();
        let entries = match self.entries.read() {
            Ok(e) => e,
            Err(_) => return false,
        };

        if let Some(meta) = entries.get(&key_hash) {
            if meta.is_expired() {
                drop(entries);
                self.remove(key);
                return false;
            }
            return true;
        }
        false
    }

    /// Clear all entries
    pub fn clear(&self) -> io::Result<()> {
        // Remove all files
        if let Ok(entries) = fs::read_dir(&self.cache_dir) {
            for entry in entries.flatten() {
                let _ = fs::remove_file(entry.path());
            }
        }

        // Reset tracking
        if let Ok(mut entries) = self.entries.write() {
            entries.clear();
        }

        if let Ok(mut size) = self.current_size.lock() {
            *size = 0;
        }

        if let Ok(mut stats) = self.stats.lock() {
            stats.entries = 0;
            stats.total_size = 0;
        }

        Ok(())
    }

    /// Clean up expired entries
    pub fn cleanup_expired(&self) -> usize {
        let expired: Vec<String> = {
            let entries = match self.entries.read() {
                Ok(e) => e,
                Err(_) => return 0,
            };
            entries
                .iter()
                .filter(|(_, meta)| meta.is_expired())
                .map(|(k, _)| k.clone())
                .collect()
        };

        let count = expired.len();
        for key_hash in expired {
            let key = CacheKey(key_hash);
            self.remove(&key);
        }

        if let Ok(mut stats) = self.stats.lock() {
            stats.expired += count as u64;
        }

        count
    }

    /// Get current size
    pub fn size(&self) -> usize {
        self.current_size.lock().map(|s| *s).unwrap_or(0)
    }

    /// Get entry count
    pub fn len(&self) -> usize {
        self.entries.read().map(|e| e.len()).unwrap_or(0)
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Get statistics
    pub fn stats(&self) -> CacheStats {
        self.stats.lock().map(|s| s.clone()).unwrap_or_default()
    }

    /// Get cache directory
    pub fn cache_dir(&self) -> &Path {
        &self.cache_dir
    }

    /// Evict entries to make room
    fn evict_for_size(&self, needed_size: usize) {
        let available = self.max_size.saturating_sub(self.size());
        if needed_size <= available {
            return;
        }

        let to_free = needed_size - available;
        let mut freed = 0usize;

        // Sort by age and evict oldest
        let mut entries_vec: Vec<(String, u64, usize)> = {
            let entries = match self.entries.read() {
                Ok(e) => e,
                Err(_) => return,
            };
            entries
                .iter()
                .map(|(k, m)| (k.clone(), m.created_at, m.size))
                .collect()
        };

        entries_vec.sort_by_key(|e| e.1);

        for (key_hash, _, size) in entries_vec {
            if freed >= to_free {
                break;
            }

            let key = CacheKey(key_hash);
            self.remove(&key);

            if let Ok(mut stats) = self.stats.lock() {
                stats.record_eviction();
            }

            freed += size;
        }
    }
}

/// Thread-safe disk cache wrapper
#[derive(Clone)]
pub struct SharedDiskCache {
    inner: Arc<DiskCache>,
}

impl SharedDiskCache {
    /// Create a new shared disk cache
    pub fn new(cache_dir: impl AsRef<Path>, max_size: usize, resource_type: ResourceType) -> io::Result<Self> {
        Ok(Self {
            inner: Arc::new(DiskCache::new(cache_dir, max_size, resource_type)?),
        })
    }

    /// Create from config
    pub fn from_config(config: &CacheConfig, resource_type: ResourceType) -> Option<Self> {
        DiskCache::from_config(config, resource_type).map(|c| Self {
            inner: Arc::new(c),
        })
    }

    /// Get data
    pub fn get(&self, key: &CacheKey) -> Option<Vec<u8>> {
        self.inner.get(key)
    }

    /// Store data
    pub fn put(
        &self,
        key: &CacheKey,
        data: &[u8],
        ttl: Duration,
        content_type: Option<String>,
        etag: Option<String>,
    ) -> bool {
        self.inner.put(key, data, ttl, content_type, etag)
    }

    /// Remove entry
    pub fn remove(&self, key: &CacheKey) -> bool {
        self.inner.remove(key)
    }

    /// Clear all
    pub fn clear(&self) -> io::Result<()> {
        self.inner.clear()
    }

    /// Check if contains key
    pub fn contains(&self, key: &CacheKey) -> bool {
        self.inner.contains(key)
    }

    /// Get stats
    pub fn stats(&self) -> CacheStats {
        self.inner.stats()
    }

    /// Get size
    pub fn size(&self) -> usize {
        self.inner.size()
    }

    /// Get directory
    pub fn cache_dir(&self) -> &Path {
        self.inner.cache_dir()
    }

    /// Cleanup expired
    pub fn cleanup_expired(&self) -> usize {
        self.inner.cleanup_expired()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_disk_cache_basic() {
        let temp_dir = TempDir::new().unwrap();
        let cache = DiskCache::new(temp_dir.path(), 1024 * 1024, ResourceType::Other).unwrap();
        let key = CacheKey::new("test");

        let data = b"hello world";
        assert!(cache.put(&key, data, Duration::from_secs(60), None, None));

        let retrieved = cache.get(&key).unwrap();
        assert_eq!(retrieved, data);
    }

    #[test]
    fn test_disk_cache_expiration() {
        let temp_dir = TempDir::new().unwrap();
        let cache = DiskCache::new(temp_dir.path(), 1024 * 1024, ResourceType::Other).unwrap();
        let key = CacheKey::new("test");

        cache.put(&key, b"data", Duration::from_millis(50), None, None);
        assert!(cache.contains(&key));

        std::thread::sleep(Duration::from_millis(60));
        assert!(!cache.contains(&key));
        assert!(cache.get(&key).is_none());
    }

    #[test]
    fn test_disk_cache_clear() {
        let temp_dir = TempDir::new().unwrap();
        let cache = DiskCache::new(temp_dir.path(), 1024 * 1024, ResourceType::Other).unwrap();

        for i in 0..5 {
            let key = CacheKey::new(format!("key{}", i));
            cache.put(&key, b"data", Duration::from_secs(60), None, None);
        }

        assert_eq!(cache.len(), 5);
        cache.clear().unwrap();
        assert!(cache.is_empty());
    }

    #[test]
    fn test_disk_cache_eviction() {
        let temp_dir = TempDir::new().unwrap();
        let cache = DiskCache::new(temp_dir.path(), 50, ResourceType::Other).unwrap();

        // Add entries that exceed limit
        for i in 0..10 {
            let key = CacheKey::new(format!("key{}", i));
            cache.put(&key, &[0u8; 10], Duration::from_secs(60), None, None);
        }

        // Should have evicted some
        assert!(cache.len() < 10);
        assert!(cache.size() <= 50);
    }

    #[test]
    fn test_disk_cache_persistence() {
        let temp_dir = TempDir::new().unwrap();
        let key = CacheKey::new("persistent");

        {
            let cache = DiskCache::new(temp_dir.path(), 1024 * 1024, ResourceType::Other).unwrap();
            cache.put(&key, b"persisted data", Duration::from_secs(60), None, None);
        }

        // Re-open cache
        let cache = DiskCache::new(temp_dir.path(), 1024 * 1024, ResourceType::Other).unwrap();
        let data = cache.get(&key).unwrap();
        assert_eq!(data, b"persisted data");
    }
}
