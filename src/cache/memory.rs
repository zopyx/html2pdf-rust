//! In-memory cache with LRU eviction
//!
//! Provides a thread-safe, size-bounded in-memory cache using
//! LRU (Least Recently Used) eviction policy.

use super::types::{AnyCacheEntry, CacheEntry, CacheKey, CacheStats};
use std::any::Any;
use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock};
use std::time::Duration;

/// LRU cache entry with access tracking
struct LruEntry {
    /// The cached data
    pub entry: AnyCacheEntry,
    /// Previous entry in LRU list (key)
    pub prev: Option<String>,
    /// Next entry in LRU list (key)
    pub next: Option<String>,
}

/// In-memory LRU cache
pub struct MemoryCache {
    /// Map of key to cache entry
    entries: RwLock<HashMap<String, LruEntry>>,
    /// Maximum size in bytes
    max_size: usize,
    /// Current size in bytes
    current_size: Mutex<usize>,
    /// Head of LRU list (most recently used)
    lru_head: Mutex<Option<String>>,
    /// Tail of LRU list (least recently used)
    lru_tail: Mutex<Option<String>>,
    /// Cache statistics
    stats: Mutex<CacheStats>,
}

impl MemoryCache {
    /// Create a new memory cache with the given size limit
    pub fn with_capacity(max_size: usize) -> Self {
        Self {
            entries: RwLock::new(HashMap::new()),
            max_size,
            current_size: Mutex::new(0),
            lru_head: Mutex::new(None),
            lru_tail: Mutex::new(None),
            stats: Mutex::new(CacheStats::default()),
        }
    }

    /// Get a value from the cache
    pub fn get<T: Any + Clone>(&self, key: &CacheKey) -> Option<T> {
        let key_str = key.hash_key();
        
        // Check if entry exists and is not expired
        let entry_opt = {
            let entries = self.entries.read().ok()?;
            entries.get(&key_str).map(|e| e.entry.meta.clone())
        };

        if let Some(ref meta) = entry_opt {
            if meta.is_expired() {
                // Remove expired entry
                drop(entry_opt);
                self.remove(key);
                self.stats.lock().ok()?.record_expired();
                return None;
            }

            // Move to front of LRU list
            self.touch(&key_str);

            // Update access stats
            if let Ok(mut entries) = self.entries.write() {
                if let Some(lru_entry) = entries.get_mut(&key_str) {
                    lru_entry.entry.record_access();
                }
            }

            self.stats.lock().ok()?.record_hit();

            // Return cloned value
            let entries = self.entries.read().ok()?;
            entries.get(&key_str)?.entry.downcast_ref::<T>().cloned()
        } else {
            self.stats.lock().ok()?.record_miss();
            None
        }
    }

    /// Insert a value into the cache
    pub fn insert<T: Any + Send + Sync>(
        &self,
        key: &CacheKey,
        value: T,
        size: usize,
        ttl: Duration,
    ) -> bool {
        let key_str = key.hash_key();

        // Check size limit
        if size > self.max_size {
            return false;
        }

        // Remove old entry if exists
        self.remove(key);

        // Make room for new entry
        self.evict_for_size(size);

        // Create new entry
        let entry = AnyCacheEntry::new(value, size, ttl);
        let lru_entry = LruEntry {
            entry,
            prev: None,
            next: None,
        };

        // Insert into map
        if let Ok(mut entries) = self.entries.write() {
            entries.insert(key_str.clone(), lru_entry);
        } else {
            return false;
        }

        // Update size
        if let Ok(mut current_size) = self.current_size.lock() {
            *current_size += size;
        }

        // Add to front of LRU list
        self.add_to_front(&key_str);

        // Update stats
        if let Ok(mut stats) = self.stats.lock() {
            stats.entries += 1;
            stats.total_size += size;
        }

        true
    }

    /// Remove an entry from the cache
    pub fn remove(&self, key: &CacheKey) -> bool {
        let key_str = key.hash_key();

        let entry_opt = {
            let mut entries = match self.entries.write() {
                Ok(e) => e,
                Err(_) => return false,
            };
            entries.remove(&key_str)
        };

        if let Some(entry) = entry_opt {
            // Remove from LRU list
            self.remove_from_lru(&key_str, &entry);

            // Update size
            if let Ok(mut current_size) = self.current_size.lock() {
                *current_size = current_size.saturating_sub(entry.entry.meta.size);
            }

            // Update stats
            if let Ok(mut stats) = self.stats.lock() {
                stats.entries = stats.entries.saturating_sub(1);
                stats.total_size = stats.total_size.saturating_sub(entry.entry.meta.size);
            }

            true
        } else {
            false
        }
    }

    /// Check if key exists in cache
    pub fn contains(&self, key: &CacheKey) -> bool {
        let key_str = key.hash_key();
        let entries = match self.entries.read() {
            Ok(e) => e,
            Err(_) => return false,
        };
        entries.contains_key(&key_str)
    }

    /// Clear all entries
    pub fn clear(&self) {
        if let Ok(mut entries) = self.entries.write() {
            entries.clear();
        }

        if let Ok(mut current_size) = self.current_size.lock() {
            *current_size = 0;
        }

        if let Ok(mut head) = self.lru_head.lock() {
            *head = None;
        }

        if let Ok(mut tail) = self.lru_tail.lock() {
            *tail = None;
        }

        if let Ok(mut stats) = self.stats.lock() {
            stats.entries = 0;
            stats.total_size = 0;
        }
    }

    /// Get current size in bytes
    pub fn size(&self) -> usize {
        self.current_size.lock().map(|s| *s).unwrap_or(0)
    }

    /// Get entry count
    pub fn len(&self) -> usize {
        self.entries.read().map(|e| e.len()).unwrap_or(0)
    }

    /// Check if cache is empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Get cache statistics
    pub fn stats(&self) -> CacheStats {
        self.stats.lock().map(|s| s.clone()).unwrap_or_default()
    }

    /// Reset statistics counters
    pub fn reset_stats(&self) {
        if let Ok(mut stats) = self.stats.lock() {
            stats.reset_counters();
        }
    }

    /// Touch an entry (move to front of LRU list)
    fn touch(&self, key: &str) {
        self.remove_from_lru_by_key(key);
        self.add_to_front(key);
    }

    /// Add entry to front of LRU list
    fn add_to_front(&self, key: &str) {
        let _ = (|| -> Option<()> {
            let mut head = self.lru_head.lock().ok()?;
            let mut tail = self.lru_tail.lock().ok()?;
            let mut entries = self.entries.write().ok()?;

            // Update new head
            if let Some(entry) = entries.get_mut(key) {
                entry.next = head.clone();
                entry.prev = None;
            }

            // Update old head
            if let Some(ref old_head) = *head {
                if let Some(entry) = entries.get_mut(old_head) {
                    entry.prev = Some(key.to_string());
                }
            }

            // Set new head
            *head = Some(key.to_string());

            // Set tail if first entry
            if tail.is_none() {
                *tail = Some(key.to_string());
            }

            Some(())
        })();
    }

    /// Remove entry from LRU list
    fn remove_from_lru(&self, key: &str, entry: &LruEntry) {
        let _ = (|| -> Option<()> {
            let mut head = self.lru_head.lock().ok()?;
            let mut tail = self.lru_tail.lock().ok()?;
            let mut entries = self.entries.write().ok()?;

            // Update prev
            if let Some(ref prev_key) = entry.prev {
                if let Some(prev) = entries.get_mut(prev_key) {
                    prev.next = entry.next.clone();
                }
            } else {
                // This was the head
                *head = entry.next.clone();
            }

            // Update next
            if let Some(ref next_key) = entry.next {
                if let Some(next) = entries.get_mut(next_key) {
                    next.prev = entry.prev.clone();
                }
            } else {
                // This was the tail
                *tail = entry.prev.clone();
            }

            Some(())
        })();
    }

    /// Remove from LRU by key (lookup entry first)
    fn remove_from_lru_by_key(&self, key: &str) {
        let entry_opt = {
            let entries = match self.entries.read() {
                Ok(e) => e,
                Err(_) => return,
            };
            entries.get(key).map(|e| (e.prev.clone(), e.next.clone()))
        };

        if let Some((prev, next)) = entry_opt {
            let _ = (|| -> Option<()> {
                let mut head = self.lru_head.lock().ok()?;
                let mut tail = self.lru_tail.lock().ok()?;
                let mut entries = self.entries.write().ok()?;

                // Update prev
                if let Some(ref prev_key) = prev {
                    if let Some(p) = entries.get_mut(prev_key) {
                        p.next = next.clone();
                    }
                } else {
                    *head = next.clone();
                }

                // Update next
                if let Some(ref next_key) = next {
                    if let Some(n) = entries.get_mut(next_key) {
                        n.prev = prev.clone();
                    }
                } else {
                    *tail = prev.clone();
                }

                // Update entry
                if let Some(entry) = entries.get_mut(key) {
                    entry.prev = None;
                    entry.next = None;
                }

                Some(())
            })();
        }
    }

    /// Evict entries to make room for the given size
    fn evict_for_size(&self, needed_size: usize) {
        let available = self.max_size - self.size();
        if needed_size <= available {
            return;
        }

        let to_free = needed_size - available;
        let mut freed = 0usize;

        while freed < to_free {
            let tail_key = {
                let tail = match self.lru_tail.lock() {
                    Ok(t) => t,
                    Err(_) => break,
                };
                tail.clone()
            };

            if let Some(key) = tail_key {
                let entry_size = {
                    let entries = match self.entries.read() {
                        Ok(e) => e,
                        Err(_) => break,
                    };
                    entries.get(&key).map(|e| e.entry.meta.size).unwrap_or(0)
                };

                // Remove entry
                let key_obj = CacheKey(key.clone());
                self.remove(&key_obj);

                if let Ok(mut stats) = self.stats.lock() {
                    stats.record_eviction();
                }

                freed += entry_size;
            } else {
                break;
            }
        }
    }

    /// Clean up expired entries
    pub fn cleanup_expired(&self) -> usize {
        let expired_keys: Vec<String> = {
            let entries = match self.entries.read() {
                Ok(e) => e,
                Err(_) => return 0,
            };
            entries
                .values()
                .filter(|e| e.entry.is_expired())
                .map(|e| {
                    // Find the key by looking up entries
                    entries
                        .iter()
                        .find(|(_, v)| std::ptr::eq(*v, e))
                        .map(|(k, _)| k.clone())
                })
                .flatten()
                .collect()
        };

        let count = expired_keys.len();
        for key in expired_keys {
            let key_obj = CacheKey(key);
            self.remove(&key_obj);
        }

        if let Ok(mut stats) = self.stats.lock() {
            stats.expired += count as u64;
        }

        count
    }
}

impl Default for MemoryCache {
    fn default() -> Self {
        Self::with_capacity(100 * 1024 * 1024) // 100MB default
    }
}

// Make MemoryCache thread-safe and cloneable
impl Clone for MemoryCache {
    fn clone(&self) -> Self {
        // Create new empty cache with same capacity
        Self::with_capacity(self.max_size)
    }
}

/// Thread-safe memory cache wrapper
#[derive(Clone)]
pub struct SharedMemoryCache {
    inner: Arc<MemoryCache>,
}

impl SharedMemoryCache {
    /// Create a new shared memory cache
    pub fn with_capacity(max_size: usize) -> Self {
        Self {
            inner: Arc::new(MemoryCache::with_capacity(max_size)),
        }
    }

    /// Get a value from the cache
    pub fn get<T: Any + Clone>(&self, key: &CacheKey) -> Option<T> {
        self.inner.get(key)
    }

    /// Insert a value into the cache
    pub fn insert<T: Any + Send + Sync>(
        &self,
        key: &CacheKey,
        value: T,
        size: usize,
        ttl: Duration,
    ) -> bool {
        self.inner.insert(key, value, size, ttl)
    }

    /// Remove an entry
    pub fn remove(&self, key: &CacheKey) -> bool {
        self.inner.remove(key)
    }

    /// Clear all entries
    pub fn clear(&self) {
        self.inner.clear();
    }

    /// Get cache statistics
    pub fn stats(&self) -> CacheStats {
        self.inner.stats()
    }

    /// Get current size
    pub fn size(&self) -> usize {
        self.inner.size()
    }

    /// Get entry count
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Clean up expired entries
    pub fn cleanup_expired(&self) -> usize {
        self.inner.cleanup_expired()
    }
}

impl Default for SharedMemoryCache {
    fn default() -> Self {
        Self::with_capacity(100 * 1024 * 1024)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_cache_basic() {
        let cache = MemoryCache::with_capacity(1024);
        let key = CacheKey::new("test");

        assert!(cache.insert(&key, "hello", 5, Duration::from_secs(60)));
        assert_eq!(cache.get::<String>(&key), Some("hello".to_string()));
    }

    #[test]
    fn test_memory_cache_expiration() {
        let cache = MemoryCache::with_capacity(1024);
        let key = CacheKey::new("test");

        cache.insert(&key, "value", 5, Duration::from_millis(50));
        assert!(cache.contains(&key));

        std::thread::sleep(Duration::from_millis(60));
        assert!(!cache.contains(&key));
        assert_eq!(cache.get::<String>(&key), None);
    }

    #[test]
    fn test_memory_cache_lru_eviction() {
        let cache = MemoryCache::with_capacity(100); // 100 bytes max

        // Insert entries that exceed capacity
        for i in 0..20 {
            let key = CacheKey::new(format!("key{}", i));
            cache.insert(&key, format!("value{}", i), 10, Duration::from_secs(60));
        }

        // Should have evicted oldest entries
        assert!(cache.len() < 20);
        assert!(cache.size() <= 100);
    }

    #[test]
    fn test_memory_cache_stats() {
        let cache = MemoryCache::with_capacity(1024);
        let key1 = CacheKey::new("key1");
        let key2 = CacheKey::new("key2");

        cache.insert(&key1, "value1", 10, Duration::from_secs(60));
        cache.get::<String>(&key1);
        cache.get::<String>(&key2); // miss

        let stats = cache.stats();
        assert_eq!(stats.hits, 1);
        assert_eq!(stats.misses, 1);
        assert_eq!(stats.hit_rate(), 0.5);
    }

    #[test]
    fn test_shared_memory_cache() {
        let cache = SharedMemoryCache::with_capacity(1024);
        let key = CacheKey::new("test");

        cache.insert(&key, 42i32, 4, Duration::from_secs(60));
        assert_eq!(cache.get::<i32>(&key), Some(42));

        let cache2 = cache.clone();
        assert_eq!(cache2.get::<i32>(&key), Some(42));
    }

    #[test]
    fn test_memory_cache_clear() {
        let cache = MemoryCache::with_capacity(1024);
        let key = CacheKey::new("test");

        cache.insert(&key, "value", 5, Duration::from_secs(60));
        assert!(!cache.is_empty());

        cache.clear();
        assert!(cache.is_empty());
        assert_eq!(cache.size(), 0);
    }
}
