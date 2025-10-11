//! Caching system for code block processing results
//! 
//! This module provides a comprehensive caching system to improve performance
//! by avoiding redundant processing of identical code blocks.

use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

use crate::markdown::code_block::{ProcessedCodeBlock, ProcessingConfig};

/// A cache key that uniquely identifies a code block processing request
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CacheKey {
    /// Hash of the source code
    code_hash: u64,
    /// Language identifier
    language: Option<String>,
    /// Hash of the processing configuration
    config_hash: u64,
    /// Strategy version for cache invalidation
    strategy_version: String,
}

impl CacheKey {
    /// Create a new cache key
    pub fn new(
        code: &str,
        language: Option<&str>,
        config: &ProcessingConfig,
        strategy_version: &str,
    ) -> Self {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        code.hash(&mut hasher);
        let code_hash = hasher.finish();

        let mut config_hasher = std::collections::hash_map::DefaultHasher::new();
        config.hash(&mut config_hasher);
        let config_hash = config_hasher.finish();

        Self {
            code_hash,
            language: language.map(|s| s.to_string()),
            config_hash,
            strategy_version: strategy_version.to_string(),
        }
    }

    /// Get a string representation for debugging
    pub fn to_debug_string(&self) -> String {
        format!(
            "CacheKey(code:{:x}, lang:{:?}, config:{:x}, version:{})",
            self.code_hash,
            self.language,
            self.config_hash,
            self.strategy_version
        )
    }
}

/// A cached entry with metadata
#[derive(Debug, Clone)]
pub struct CacheEntry {
    /// The cached processed code block
    pub processed_block: ProcessedCodeBlock,
    /// When this entry was created
    pub created_at: Instant,
    /// When this entry was last accessed
    pub last_accessed: Instant,
    /// Number of times this entry has been accessed
    pub access_count: u64,
    /// Size estimate in bytes
    pub size_estimate: usize,
}

impl CacheEntry {
    /// Create a new cache entry
    pub fn new(processed_block: ProcessedCodeBlock) -> Self {
        let now = Instant::now();
        let size_estimate = Self::estimate_size(&processed_block);
        
        Self {
            processed_block,
            created_at: now,
            last_accessed: now,
            access_count: 1,
            size_estimate,
        }
    }

    /// Update access information
    pub fn mark_accessed(&mut self) {
        self.last_accessed = Instant::now();
        self.access_count += 1;
    }

    /// Check if this entry has expired
    pub fn is_expired(&self, max_age: Duration) -> bool {
        self.created_at.elapsed() > max_age
    }

    /// Check if this entry is stale (not accessed recently)
    pub fn is_stale(&self, stale_threshold: Duration) -> bool {
        self.last_accessed.elapsed() > stale_threshold
    }

    /// Get the age of this entry
    pub fn age(&self) -> Duration {
        self.created_at.elapsed()
    }

    /// Estimate the memory size of a processed code block
    fn estimate_size(block: &ProcessedCodeBlock) -> usize {
        let mut size = 0;
        
        // Original code
        size += block.original_code.len();
        
        // Processed code
        if let Some(ref processed) = block.processed_code {
            size += processed.len();
        }
        
        // Language string
        if let Some(ref lang) = block.language {
            size += lang.len();
        }
        
        // Metadata
        size += block.metadata.processor_version.len();
        size += block.metadata.custom_attributes.iter()
            .map(|(k, v)| k.len() + v.len())
            .sum::<usize>();
        
        // Errors and warnings (rough estimate)
        size += block.errors.len() * 100; // Rough estimate per error
        size += block.warnings.len() * 80; // Rough estimate per warning
        
        // Base struct overhead
        size += 200;
        
        size
    }
}

/// Configuration for the cache system
#[derive(Debug, Clone)]
pub struct CacheConfig {
    /// Maximum number of entries in the cache
    pub max_entries: usize,
    /// Maximum total memory usage in bytes
    pub max_memory_bytes: usize,
    /// Maximum age for cache entries
    pub max_entry_age: Duration,
    /// Threshold for considering entries stale
    pub stale_threshold: Duration,
    /// Whether to enable cache statistics
    pub enable_statistics: bool,
    /// Cleanup interval for expired entries
    pub cleanup_interval: Duration,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            max_entries: 1000,
            max_memory_bytes: 50 * 1024 * 1024, // 50MB
            max_entry_age: Duration::from_secs(3600), // 1 hour
            stale_threshold: Duration::from_secs(300), // 5 minutes
            enable_statistics: true,
            cleanup_interval: Duration::from_secs(60), // 1 minute
        }
    }
}

/// Statistics about cache performance
#[derive(Debug, Clone, Default)]
pub struct CacheStatistics {
    /// Total number of cache hits
    pub hits: u64,
    /// Total number of cache misses
    pub misses: u64,
    /// Total number of entries evicted
    pub evictions: u64,
    /// Total number of expired entries cleaned up
    pub expirations: u64,
    /// Current number of entries
    pub current_entries: usize,
    /// Current estimated memory usage
    pub current_memory_bytes: usize,
    /// Last cleanup time
    pub last_cleanup: Option<Instant>,
}

impl CacheStatistics {
    /// Calculate hit ratio
    pub fn hit_ratio(&self) -> f64 {
        let total = self.hits + self.misses;
        if total == 0 {
            0.0
        } else {
            self.hits as f64 / total as f64
        }
    }

    /// Get total requests
    pub fn total_requests(&self) -> u64 {
        self.hits + self.misses
    }

    /// Check if cache is effective (hit ratio > 50%)
    pub fn is_effective(&self) -> bool {
        self.hit_ratio() > 0.5
    }
}

/// Thread-safe LRU cache for processed code blocks
pub struct CodeBlockCache {
    /// The actual cache storage
    cache: Arc<RwLock<HashMap<CacheKey, CacheEntry>>>,
    /// Cache configuration
    config: CacheConfig,
    /// Cache statistics
    statistics: Arc<RwLock<CacheStatistics>>,
    /// Last cleanup time
    last_cleanup: Arc<RwLock<Instant>>,
}

impl CodeBlockCache {
    /// Create a new cache with default configuration
    pub fn new() -> Self {
        Self::with_config(CacheConfig::default())
    }

    /// Create a new cache with custom configuration
    pub fn with_config(config: CacheConfig) -> Self {
        Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
            config,
            statistics: Arc::new(RwLock::new(CacheStatistics::default())),
            last_cleanup: Arc::new(RwLock::new(Instant::now())),
        }
    }

    /// Get a cached result or compute it using the provided function
    pub fn get_or_compute<F>(
        &self,
        key: CacheKey,
        compute_fn: F,
    ) -> Result<ProcessedCodeBlock, Box<dyn std::error::Error + Send + Sync>>
    where
        F: FnOnce() -> Result<ProcessedCodeBlock, Box<dyn std::error::Error + Send + Sync>>,
    {
        // Try to get from cache first
        if let Some(mut entry) = self.get_entry(&key) {
            entry.mark_accessed();
            let processed_block = entry.processed_block.clone();
            self.update_entry(&key, entry);
            
            if self.config.enable_statistics {
                if let Ok(mut stats) = self.statistics.write() {
                    stats.hits += 1;
                }
            }
            
            return Ok(processed_block);
        }

        // Cache miss - compute the result
        let result = compute_fn()?;
        
        // Store in cache
        self.put(key, result.clone());
        
        if self.config.enable_statistics {
            if let Ok(mut stats) = self.statistics.write() {
                stats.misses += 1;
            }
        }

        // Perform cleanup if needed
        self.maybe_cleanup();

        Ok(result)
    }

    /// Put a result in the cache
    pub fn put(&self, key: CacheKey, processed_block: ProcessedCodeBlock) {
        let entry = CacheEntry::new(processed_block);
        
        if let Ok(mut cache) = self.cache.write() {
            // Check if we need to evict entries
            if cache.len() >= self.config.max_entries {
                self.evict_lru_entries(&mut cache, 1);
            }
            
            // Check memory usage
            let current_memory = self.calculate_memory_usage(&cache);
            if current_memory + entry.size_estimate > self.config.max_memory_bytes {
                let bytes_to_free = (current_memory + entry.size_estimate) - self.config.max_memory_bytes;
                self.evict_by_memory(&mut cache, bytes_to_free);
            }
            
            cache.insert(key, entry);
            
            // Update statistics
            if self.config.enable_statistics {
                if let Ok(mut stats) = self.statistics.write() {
                    stats.current_entries = cache.len();
                    stats.current_memory_bytes = self.calculate_memory_usage(&cache);
                }
            }
        }
    }

    /// Get an entry from the cache
    fn get_entry(&self, key: &CacheKey) -> Option<CacheEntry> {
        if let Ok(cache) = self.cache.read() {
            cache.get(key).cloned()
        } else {
            None
        }
    }

    /// Update an entry in the cache
    fn update_entry(&self, key: &CacheKey, entry: CacheEntry) {
        if let Ok(mut cache) = self.cache.write() {
            cache.insert(key.clone(), entry);
        }
    }

    /// Clear all entries from the cache
    pub fn clear(&self) {
        if let Ok(mut cache) = self.cache.write() {
            cache.clear();
            
            if self.config.enable_statistics {
                if let Ok(mut stats) = self.statistics.write() {
                    stats.current_entries = 0;
                    stats.current_memory_bytes = 0;
                }
            }
        }
    }

    /// Get current cache statistics
    pub fn get_statistics(&self) -> CacheStatistics {
        if let Ok(stats) = self.statistics.read() {
            stats.clone()
        } else {
            CacheStatistics::default()
        }
    }

    /// Force cleanup of expired entries
    pub fn cleanup(&self) {
        if let Ok(mut cache) = self.cache.write() {
            let initial_count = cache.len();
            
            cache.retain(|_, entry| {
                !entry.is_expired(self.config.max_entry_age)
            });
            
            let removed_count = initial_count - cache.len();
            
            if self.config.enable_statistics && removed_count > 0 {
                if let Ok(mut stats) = self.statistics.write() {
                    stats.expirations += removed_count as u64;
                    stats.current_entries = cache.len();
                    stats.current_memory_bytes = self.calculate_memory_usage(&cache);
                    stats.last_cleanup = Some(Instant::now());
                }
            }
        }
        
        if let Ok(mut last_cleanup) = self.last_cleanup.write() {
            *last_cleanup = Instant::now();
        }
    }

    /// Maybe perform cleanup if enough time has passed
    fn maybe_cleanup(&self) {
        if let Ok(last_cleanup) = self.last_cleanup.read() {
            if last_cleanup.elapsed() > self.config.cleanup_interval {
                drop(last_cleanup); // Release read lock
                self.cleanup();
            }
        }
    }

    /// Evict LRU entries
    fn evict_lru_entries(&self, cache: &mut HashMap<CacheKey, CacheEntry>, count: usize) {
        if cache.is_empty() || count == 0 {
            return;
        }

        // Find the oldest entries by last access time
        let mut entries: Vec<_> = cache.iter().collect();
        entries.sort_by_key(|(_, entry)| entry.last_accessed);
        
        let to_remove: Vec<_> = entries.iter()
            .take(count)
            .map(|(key, _)| (*key).clone())
            .collect();
        
        for key in to_remove {
            cache.remove(&key);
        }
        
        if self.config.enable_statistics {
            if let Ok(mut stats) = self.statistics.write() {
                stats.evictions += count as u64;
            }
        }
    }

    /// Evict entries to free up memory
    fn evict_by_memory(&self, cache: &mut HashMap<CacheKey, CacheEntry>, bytes_to_free: usize) {
        if cache.is_empty() || bytes_to_free == 0 {
            return;
        }

        // Sort by access time (LRU first)
        let mut entries: Vec<_> = cache.iter().collect();
        entries.sort_by_key(|(_, entry)| entry.last_accessed);
        
        let mut freed_bytes = 0;
        let mut evicted_count = 0;
        let mut to_remove = Vec::new();
        
        for (key, entry) in entries {
            to_remove.push(key.clone());
            freed_bytes += entry.size_estimate;
            evicted_count += 1;
            
            if freed_bytes >= bytes_to_free {
                break;
            }
        }
        
        for key in to_remove {
            cache.remove(&key);
        }
        
        if self.config.enable_statistics {
            if let Ok(mut stats) = self.statistics.write() {
                stats.evictions += evicted_count;
            }
        }
    }

    /// Calculate total memory usage
    fn calculate_memory_usage(&self, cache: &HashMap<CacheKey, CacheEntry>) -> usize {
        cache.values().map(|entry| entry.size_estimate).sum()
    }

    /// Get cache size information
    pub fn get_size_info(&self) -> CacheSizeInfo {
        if let Ok(cache) = self.cache.read() {
            CacheSizeInfo {
                entry_count: cache.len(),
                memory_bytes: self.calculate_memory_usage(&cache),
                max_entries: self.config.max_entries,
                max_memory_bytes: self.config.max_memory_bytes,
            }
        } else {
            CacheSizeInfo {
                entry_count: 0,
                memory_bytes: 0,
                max_entries: self.config.max_entries,
                max_memory_bytes: self.config.max_memory_bytes,
            }
        }
    }

    /// Check if cache is near capacity
    pub fn is_near_capacity(&self) -> bool {
        let size_info = self.get_size_info();
        let entry_ratio = size_info.entry_count as f64 / size_info.max_entries as f64;
        let memory_ratio = size_info.memory_bytes as f64 / size_info.max_memory_bytes as f64;
        
        entry_ratio > 0.8 || memory_ratio > 0.8
    }
}

impl Default for CodeBlockCache {
    fn default() -> Self {
        Self::new()
    }
}

/// Information about cache size and capacity
#[derive(Debug, Clone)]
pub struct CacheSizeInfo {
    pub entry_count: usize,
    pub memory_bytes: usize,
    pub max_entries: usize,
    pub max_memory_bytes: usize,
}

impl CacheSizeInfo {
    /// Get entry utilization ratio (0.0 to 1.0)
    pub fn entry_utilization(&self) -> f64 {
        if self.max_entries == 0 {
            0.0
        } else {
            self.entry_count as f64 / self.max_entries as f64
        }
    }

    /// Get memory utilization ratio (0.0 to 1.0)
    pub fn memory_utilization(&self) -> f64 {
        if self.max_memory_bytes == 0 {
            0.0
        } else {
            self.memory_bytes as f64 / self.max_memory_bytes as f64
        }
    }

    /// Check if cache is full
    pub fn is_full(&self) -> bool {
        self.entry_count >= self.max_entries || self.memory_bytes >= self.max_memory_bytes
    }
}

// Make ProcessingConfig hashable for cache keys
impl Hash for ProcessingConfig {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.enable_syntax_validation.hash(state);
        self.enable_formatting.hash(state);
        self.enable_optimization.hash(state);
        self.timeout_ms.hash(state);
        
        // Hash custom options in a deterministic order
        let mut options: Vec<_> = self.custom_options.iter().collect();
        options.sort_by_key(|(k, _)| *k);
        for (key, value) in options {
            key.hash(state);
            value.hash(state);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::markdown::code_block::ProcessingMetadata;

    fn create_test_processed_block(code: &str, language: Option<&str>) -> ProcessedCodeBlock {
        ProcessedCodeBlock {
            original_code: code.to_string(),
            processed_code: None,
            language: language.map(|s| s.to_string()),
            metadata: ProcessingMetadata::default(),
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }

    #[test]
    fn test_cache_key_creation() {
        let config = ProcessingConfig::default();
        let key1 = CacheKey::new("fn main() {}", Some("rust"), &config, "1.0.0");
        let key2 = CacheKey::new("fn main() {}", Some("rust"), &config, "1.0.0");
        let key3 = CacheKey::new("fn main() {}", Some("javascript"), &config, "1.0.0");
        
        assert_eq!(key1, key2);
        assert_ne!(key1, key3);
    }

    #[test]
    fn test_cache_entry_creation() {
        let block = create_test_processed_block("fn main() {}", Some("rust"));
        let entry = CacheEntry::new(block);
        
        assert_eq!(entry.access_count, 1);
        assert!(entry.size_estimate > 0);
        assert!(!entry.is_expired(Duration::from_secs(3600)));
    }

    #[test]
    fn test_cache_basic_operations() {
        let cache = CodeBlockCache::new();
        let config = ProcessingConfig::default();
        let key = CacheKey::new("fn main() {}", Some("rust"), &config, "1.0.0");
        let block = create_test_processed_block("fn main() {}", Some("rust"));
        
        // Put and get
        cache.put(key.clone(), block.clone());
        
        let result = cache.get_or_compute(key, || {
            panic!("Should not compute - should hit cache");
        });
        
        assert!(result.is_ok());
        let retrieved = result.unwrap();
        assert_eq!(retrieved.original_code, block.original_code);
    }

    #[test]
    fn test_cache_miss_and_compute() {
        let cache = CodeBlockCache::new();
        let config = ProcessingConfig::default();
        let key = CacheKey::new("fn main() {}", Some("rust"), &config, "1.0.0");
        
        let mut computed = false;
        let result = cache.get_or_compute(key, || {
            computed = true;
            Ok(create_test_processed_block("fn main() {}", Some("rust")))
        });
        
        assert!(result.is_ok());
        assert!(computed);
    }

    #[test]
    fn test_cache_statistics() {
        let cache = CodeBlockCache::new();
        let config = ProcessingConfig::default();
        let key = CacheKey::new("fn main() {}", Some("rust"), &config, "1.0.0");
        
        // Initial statistics
        let stats = cache.get_statistics();
        assert_eq!(stats.hits, 0);
        assert_eq!(stats.misses, 0);
        
        // Cache miss
        let _ = cache.get_or_compute(key.clone(), || {
            Ok(create_test_processed_block("fn main() {}", Some("rust")))
        });
        
        let stats = cache.get_statistics();
        assert_eq!(stats.misses, 1);
        
        // Cache hit
        let _ = cache.get_or_compute(key, || {
            panic!("Should not compute");
        });
        
        let stats = cache.get_statistics();
        assert_eq!(stats.hits, 1);
        assert_eq!(stats.misses, 1);
    }

    #[test]
    fn test_cache_size_limits() {
        let config = CacheConfig {
            max_entries: 2,
            max_memory_bytes: 1024 * 1024, // 1MB
            ..Default::default()
        };
        let cache = CodeBlockCache::with_config(config);
        
        // Add entries up to limit
        for i in 0..3 {
            let key = CacheKey::new(
                &format!("fn main{i}() {{}}"),
                Some("rust"),
                &ProcessingConfig::default(),
                "1.0.0",
            );
            let block = create_test_processed_block(&format!("fn main{i}() {{}}"), Some("rust"));
            cache.put(key, block);
        }
        
        let size_info = cache.get_size_info();
        assert_eq!(size_info.entry_count, 2); // Should be limited to max_entries
    }

    #[test]
    fn test_cache_cleanup() {
        let config = CacheConfig {
            max_entry_age: Duration::from_millis(1), // Very short age
            ..Default::default()
        };
        let cache = CodeBlockCache::with_config(config);
        
        let key = CacheKey::new("fn main() {}", Some("rust"), &ProcessingConfig::default(), "1.0.0");
        let block = create_test_processed_block("fn main() {}", Some("rust"));
        cache.put(key, block);
        
        // Wait for expiration
        std::thread::sleep(Duration::from_millis(2));
        
        // Force cleanup
        cache.cleanup();
        
        let size_info = cache.get_size_info();
        assert_eq!(size_info.entry_count, 0);
    }

    #[test]
    fn test_cache_clear() {
        let cache = CodeBlockCache::new();
        let key = CacheKey::new("fn main() {}", Some("rust"), &ProcessingConfig::default(), "1.0.0");
        let block = create_test_processed_block("fn main() {}", Some("rust"));
        
        cache.put(key, block);
        assert_eq!(cache.get_size_info().entry_count, 1);
        
        cache.clear();
        assert_eq!(cache.get_size_info().entry_count, 0);
    }

    #[test]
    fn test_cache_near_capacity() {
        let config = CacheConfig {
            max_entries: 10,
            max_memory_bytes: 1000,
            ..Default::default()
        };
        let cache = CodeBlockCache::with_config(config);
        
        // Add entries to approach capacity
        for i in 0..9 {
            let key = CacheKey::new(
                &format!("fn main{i}() {{}}"),
                Some("rust"),
                &ProcessingConfig::default(),
                "1.0.0",
            );
            let block = create_test_processed_block(&format!("fn main{i}() {{}}"), Some("rust"));
            cache.put(key, block);
        }
        
        assert!(cache.is_near_capacity());
    }

    #[test]
    fn test_cache_size_info() {
        let cache = CodeBlockCache::new();
        let size_info = cache.get_size_info();
        
        assert_eq!(size_info.entry_count, 0);
        assert_eq!(size_info.memory_bytes, 0);
        assert!(!size_info.is_full());
        assert_eq!(size_info.entry_utilization(), 0.0);
        assert_eq!(size_info.memory_utilization(), 0.0);
    }

    #[test]
    fn test_processing_config_hash() {
        let config1 = ProcessingConfig {
            enable_syntax_validation: true,
            enable_formatting: false,
            enable_optimization: true,
            timeout_ms: 5000,
            custom_options: [("key1".to_string(), "value1".to_string())].iter().cloned().collect(),
        };
        
        let config2 = ProcessingConfig {
            enable_syntax_validation: true,
            enable_formatting: false,
            enable_optimization: true,
            timeout_ms: 5000,
            custom_options: [("key1".to_string(), "value1".to_string())].iter().cloned().collect(),
        };
        
        let key1 = CacheKey::new("code", Some("rust"), &config1, "1.0.0");
        let key2 = CacheKey::new("code", Some("rust"), &config2, "1.0.0");
        
        assert_eq!(key1, key2);
    }
}