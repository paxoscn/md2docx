//! Memory optimization utilities for code block processing
//!
//! This module provides memory management and optimization features to ensure
//! efficient memory usage during code block processing operations.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use crate::markdown::code_block::ProcessedCodeBlock;

/// Configuration for memory optimization
#[derive(Debug, Clone)]
pub struct MemoryConfig {
    /// Maximum memory usage in bytes before triggering cleanup
    pub max_memory_usage: usize,
    /// Target memory usage after cleanup (as percentage of max)
    pub target_memory_ratio: f64,
    /// Interval between memory usage checks
    pub check_interval: Duration,
    /// Whether to enable memory usage tracking
    pub enable_tracking: bool,
    /// Whether to enable automatic cleanup
    pub enable_auto_cleanup: bool,
    /// Threshold for considering objects as "large" (in bytes)
    pub large_object_threshold: usize,
    /// Maximum age for cached objects before they become eligible for cleanup
    pub max_object_age: Duration,
}

impl Default for MemoryConfig {
    fn default() -> Self {
        Self {
            max_memory_usage: 100 * 1024 * 1024, // 100MB
            target_memory_ratio: 0.7,            // Clean up to 70% of max
            check_interval: Duration::from_secs(30),
            enable_tracking: true,
            enable_auto_cleanup: true,
            large_object_threshold: 1024 * 1024,      // 1MB
            max_object_age: Duration::from_secs(600), // 10 minutes
        }
    }
}

/// Information about memory usage
#[derive(Debug, Clone)]
pub struct MemoryUsage {
    /// Total allocated memory in bytes
    pub total_allocated: usize,
    /// Memory used by processed code blocks
    pub code_blocks_memory: usize,
    /// Memory used by strategies
    pub strategies_memory: usize,
    /// Memory used by caches
    pub cache_memory: usize,
    /// Number of tracked objects
    pub object_count: usize,
    /// Number of large objects
    pub large_object_count: usize,
    /// Peak memory usage since last reset
    pub peak_usage: usize,
    /// When this measurement was taken
    pub measured_at: Instant,
}

impl MemoryUsage {
    /// Calculate memory utilization as a percentage
    pub fn utilization_percentage(&self, max_memory: usize) -> f64 {
        if max_memory == 0 {
            0.0
        } else {
            self.total_allocated as f64 / max_memory as f64 * 100.0
        }
    }

    /// Check if memory usage is critical
    pub fn is_critical(&self, max_memory: usize, threshold: f64) -> bool {
        self.utilization_percentage(max_memory) > threshold
    }

    /// Get average object size
    pub fn average_object_size(&self) -> usize {
        if self.object_count == 0 {
            0
        } else {
            self.total_allocated / self.object_count
        }
    }
}

/// A tracked memory object with metadata
#[derive(Debug)]
struct TrackedObject {
    /// Estimated size in bytes
    size: usize,
    /// When this object was created
    created_at: Instant,
    /// When this object was last accessed
    last_accessed: Instant,
    /// Number of times this object has been accessed
    access_count: u64,
    /// Whether this object is considered "large"
    is_large: bool,
    /// Category of this object (for statistics)
    category: String,
}

impl TrackedObject {
    fn new(size: usize, category: String, large_threshold: usize) -> Self {
        let now = Instant::now();
        Self {
            size,
            created_at: now,
            last_accessed: now,
            access_count: 1,
            is_large: size >= large_threshold,
            category,
        }
    }

    fn mark_accessed(&mut self) {
        self.last_accessed = Instant::now();
        self.access_count += 1;
    }

    fn age(&self) -> Duration {
        self.created_at.elapsed()
    }

    fn time_since_last_access(&self) -> Duration {
        self.last_accessed.elapsed()
    }

    fn is_stale(&self, max_age: Duration) -> bool {
        self.age() > max_age || self.time_since_last_access() > max_age
    }
}

/// Memory manager for tracking and optimizing memory usage
pub struct MemoryManager {
    config: MemoryConfig,
    tracked_objects: Mutex<HashMap<usize, TrackedObject>>,
    next_object_id: Mutex<usize>,
    peak_usage: Mutex<usize>,
    last_cleanup: Mutex<Instant>,
    cleanup_callbacks: Mutex<Vec<Box<dyn Fn() + Send + Sync>>>,
}

impl MemoryManager {
    /// Create a new memory manager
    pub fn new() -> Self {
        Self::with_config(MemoryConfig::default())
    }

    /// Create a memory manager with custom configuration
    pub fn with_config(config: MemoryConfig) -> Self {
        Self {
            config,
            tracked_objects: Mutex::new(HashMap::new()),
            next_object_id: Mutex::new(0),
            peak_usage: Mutex::new(0),
            last_cleanup: Mutex::new(Instant::now()),
            cleanup_callbacks: Mutex::new(Vec::new()),
        }
    }

    /// Track a new object
    pub fn track_object(&self, size: usize, category: &str) -> Option<ObjectHandle> {
        if !self.config.enable_tracking {
            return None;
        }

        let object_id = self.get_next_object_id();
        let tracked_object = TrackedObject::new(
            size,
            category.to_string(),
            self.config.large_object_threshold,
        );

        if let Ok(mut objects) = self.tracked_objects.lock() {
            objects.insert(object_id, tracked_object);

            // Update peak usage
            let current_usage = self.calculate_total_usage(&objects);
            if let Ok(mut peak) = self.peak_usage.lock() {
                *peak = (*peak).max(current_usage);
            }

            // Check if we need cleanup
            if self.config.enable_auto_cleanup {
                self.maybe_trigger_cleanup(current_usage);
            }

            Some(ObjectHandle {
                id: object_id,
                manager: self as *const MemoryManager,
            })
        } else {
            None
        }
    }

    /// Untrack an object
    pub fn untrack_object(&self, object_id: usize) {
        if let Ok(mut objects) = self.tracked_objects.lock() {
            objects.remove(&object_id);
        }
    }

    /// Mark an object as accessed
    pub fn mark_object_accessed(&self, object_id: usize) {
        if let Ok(mut objects) = self.tracked_objects.lock() {
            if let Some(object) = objects.get_mut(&object_id) {
                object.mark_accessed();
            }
        }
    }

    /// Get current memory usage information
    pub fn get_memory_usage(&self) -> MemoryUsage {
        if let Ok(objects) = self.tracked_objects.lock() {
            let total_allocated = self.calculate_total_usage(&objects);
            let large_object_count = objects.values().filter(|obj| obj.is_large).count();

            // Calculate memory by category
            let mut code_blocks_memory = 0;
            let mut strategies_memory = 0;
            let mut cache_memory = 0;

            for object in objects.values() {
                match object.category.as_str() {
                    "code_block" => code_blocks_memory += object.size,
                    "strategy" => strategies_memory += object.size,
                    "cache" => cache_memory += object.size,
                    _ => {}
                }
            }

            let peak_usage = if let Ok(peak) = self.peak_usage.lock() {
                *peak
            } else {
                0
            };

            MemoryUsage {
                total_allocated,
                code_blocks_memory,
                strategies_memory,
                cache_memory,
                object_count: objects.len(),
                large_object_count,
                peak_usage,
                measured_at: Instant::now(),
            }
        } else {
            MemoryUsage {
                total_allocated: 0,
                code_blocks_memory: 0,
                strategies_memory: 0,
                cache_memory: 0,
                object_count: 0,
                large_object_count: 0,
                peak_usage: 0,
                measured_at: Instant::now(),
            }
        }
    }

    /// Force a memory cleanup
    pub fn cleanup(&self) -> CleanupResult {
        let start_time = Instant::now();
        let initial_usage = self.get_memory_usage();

        let objects_removed = if let Ok(mut objects) = self.tracked_objects.lock() {
            let initial_count = objects.len();

            // Remove stale objects
            objects.retain(|_, obj| !obj.is_stale(self.config.max_object_age));

            // If still over target, remove least recently used objects
            let current_usage = self.calculate_total_usage(&objects);
            let target_usage =
                (self.config.max_memory_usage as f64 * self.config.target_memory_ratio) as usize;

            if current_usage > target_usage {
                self.cleanup_lru_objects(&mut objects, current_usage - target_usage);
            }

            initial_count - objects.len()
        } else {
            0
        };

        // Execute cleanup callbacks
        if let Ok(callbacks) = self.cleanup_callbacks.lock() {
            for callback in callbacks.iter() {
                callback();
            }
        }

        // Update last cleanup time
        if let Ok(mut last_cleanup) = self.last_cleanup.lock() {
            *last_cleanup = Instant::now();
        }

        let final_usage = self.get_memory_usage();

        CleanupResult {
            objects_removed,
            memory_freed: initial_usage
                .total_allocated
                .saturating_sub(final_usage.total_allocated),
            cleanup_duration: start_time.elapsed(),
            initial_usage: initial_usage.total_allocated,
            final_usage: final_usage.total_allocated,
        }
    }

    /// Register a cleanup callback
    pub fn register_cleanup_callback<F>(&self, callback: F)
    where
        F: Fn() + Send + Sync + 'static,
    {
        if let Ok(mut callbacks) = self.cleanup_callbacks.lock() {
            callbacks.push(Box::new(callback));
        }
    }

    /// Check if memory usage is critical
    pub fn is_memory_critical(&self) -> bool {
        let usage = self.get_memory_usage();
        usage.is_critical(self.config.max_memory_usage, 90.0)
    }

    /// Get memory statistics by category
    pub fn get_memory_stats_by_category(&self) -> HashMap<String, CategoryStats> {
        if let Ok(objects) = self.tracked_objects.lock() {
            let mut stats: HashMap<String, CategoryStats> = HashMap::new();

            for object in objects.values() {
                let category_stats =
                    stats
                        .entry(object.category.clone())
                        .or_insert_with(|| CategoryStats {
                            total_size: 0,
                            object_count: 0,
                            large_object_count: 0,
                            average_age: Duration::from_millis(0),
                            total_access_count: 0,
                        });

                category_stats.total_size += object.size;
                category_stats.object_count += 1;
                category_stats.total_access_count += object.access_count;

                if object.is_large {
                    category_stats.large_object_count += 1;
                }
            }

            // Calculate average ages
            for (category, stats) in stats.iter_mut() {
                let total_age: Duration = objects
                    .values()
                    .filter(|obj| &obj.category == category)
                    .map(|obj| obj.age())
                    .sum();

                if stats.object_count > 0 {
                    stats.average_age = total_age / stats.object_count as u32;
                }
            }

            stats
        } else {
            HashMap::new()
        }
    }

    /// Reset peak usage tracking
    pub fn reset_peak_usage(&self) {
        if let Ok(mut peak) = self.peak_usage.lock() {
            *peak = self.get_memory_usage().total_allocated;
        }
    }

    /// Get the next object ID
    fn get_next_object_id(&self) -> usize {
        if let Ok(mut id) = self.next_object_id.lock() {
            *id += 1;
            *id
        } else {
            0
        }
    }

    /// Calculate total memory usage
    fn calculate_total_usage(&self, objects: &HashMap<usize, TrackedObject>) -> usize {
        objects.values().map(|obj| obj.size).sum()
    }

    /// Maybe trigger cleanup if memory usage is high
    fn maybe_trigger_cleanup(&self, current_usage: usize) {
        if current_usage > self.config.max_memory_usage {
            // Check if enough time has passed since last cleanup
            if let Ok(last_cleanup) = self.last_cleanup.lock() {
                if last_cleanup.elapsed() > self.config.check_interval {
                    drop(last_cleanup); // Release lock before cleanup
                    self.cleanup();
                }
            }
        }
    }

    /// Clean up least recently used objects to free specified amount of memory
    fn cleanup_lru_objects(
        &self,
        objects: &mut HashMap<usize, TrackedObject>,
        target_bytes: usize,
    ) {
        // Sort objects by last access time (oldest first)
        let mut sorted_objects: Vec<_> = objects.iter().collect();
        sorted_objects.sort_by_key(|(_, obj)| obj.last_accessed);

        let mut freed_bytes = 0;
        let mut to_remove = Vec::new();

        for (id, object) in sorted_objects {
            if freed_bytes >= target_bytes {
                break;
            }

            // Don't remove recently accessed objects
            if object.time_since_last_access() < Duration::from_secs(60) {
                continue;
            }

            freed_bytes += object.size;
            to_remove.push(*id);
        }

        for id in to_remove {
            objects.remove(&id);
        }
    }
}

impl Default for MemoryManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Handle for a tracked object
pub struct ObjectHandle {
    id: usize,
    manager: *const MemoryManager,
}

impl ObjectHandle {
    /// Mark this object as accessed
    pub fn mark_accessed(&self) {
        unsafe {
            if let Some(manager) = self.manager.as_ref() {
                manager.mark_object_accessed(self.id);
            }
        }
    }

    /// Get the object ID
    pub fn id(&self) -> usize {
        self.id
    }
}

impl Drop for ObjectHandle {
    fn drop(&mut self) {
        unsafe {
            if let Some(manager) = self.manager.as_ref() {
                manager.untrack_object(self.id);
            }
        }
    }
}

// Safety: ObjectHandle is safe to send between threads as long as the MemoryManager lives
unsafe impl Send for ObjectHandle {}
unsafe impl Sync for ObjectHandle {}

/// Result of a cleanup operation
#[derive(Debug, Clone)]
pub struct CleanupResult {
    /// Number of objects removed
    pub objects_removed: usize,
    /// Amount of memory freed in bytes
    pub memory_freed: usize,
    /// Time taken for cleanup
    pub cleanup_duration: Duration,
    /// Memory usage before cleanup
    pub initial_usage: usize,
    /// Memory usage after cleanup
    pub final_usage: usize,
}

impl CleanupResult {
    /// Calculate the percentage of memory freed
    pub fn memory_freed_percentage(&self) -> f64 {
        if self.initial_usage == 0 {
            0.0
        } else {
            self.memory_freed as f64 / self.initial_usage as f64 * 100.0
        }
    }

    /// Check if cleanup was effective (freed significant memory)
    pub fn was_effective(&self) -> bool {
        self.memory_freed_percentage() > 10.0 // Freed more than 10%
    }
}

/// Statistics for a memory category
#[derive(Debug, Clone)]
pub struct CategoryStats {
    pub total_size: usize,
    pub object_count: usize,
    pub large_object_count: usize,
    pub average_age: Duration,
    pub total_access_count: u64,
}

impl CategoryStats {
    /// Get average object size for this category
    pub fn average_object_size(&self) -> usize {
        if self.object_count == 0 {
            0
        } else {
            self.total_size / self.object_count
        }
    }

    /// Get average access count per object
    pub fn average_access_count(&self) -> f64 {
        if self.object_count == 0 {
            0.0
        } else {
            self.total_access_count as f64 / self.object_count as f64
        }
    }

    /// Get percentage of large objects
    pub fn large_object_percentage(&self) -> f64 {
        if self.object_count == 0 {
            0.0
        } else {
            self.large_object_count as f64 / self.object_count as f64 * 100.0
        }
    }
}

/// Memory-aware wrapper for processed code blocks
pub struct ManagedCodeBlock {
    block: ProcessedCodeBlock,
    _handle: Option<ObjectHandle>,
}

impl ManagedCodeBlock {
    /// Create a new managed code block
    pub fn new(block: ProcessedCodeBlock, memory_manager: &MemoryManager) -> Self {
        let size = Self::estimate_size(&block);
        let handle = memory_manager.track_object(size, "code_block");

        Self {
            block,
            _handle: handle,
        }
    }

    /// Get the underlying processed code block
    pub fn get_block(&self) -> &ProcessedCodeBlock {
        if let Some(ref handle) = self._handle {
            handle.mark_accessed();
        }
        &self.block
    }

    /// Get a mutable reference to the underlying block
    pub fn get_block_mut(&mut self) -> &mut ProcessedCodeBlock {
        if let Some(ref handle) = self._handle {
            handle.mark_accessed();
        }
        &mut self.block
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
        size += block
            .metadata
            .custom_attributes
            .iter()
            .map(|(k, v)| k.len() + v.len())
            .sum::<usize>();

        // Errors and warnings (rough estimate)
        size += block.errors.len() * 100;
        size += block.warnings.len() * 80;

        // Base struct overhead
        size += 200;

        size
    }
}

impl std::ops::Deref for ManagedCodeBlock {
    type Target = ProcessedCodeBlock;

    fn deref(&self) -> &Self::Target {
        self.get_block()
    }
}

impl std::ops::DerefMut for ManagedCodeBlock {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.get_block_mut()
    }
}

/// Global memory manager instance
static GLOBAL_MEMORY_MANAGER: std::sync::OnceLock<MemoryManager> = std::sync::OnceLock::new();

/// Get the global memory manager instance
pub fn global_memory_manager() -> &'static MemoryManager {
    GLOBAL_MEMORY_MANAGER.get_or_init(|| MemoryManager::new())
}

/// Initialize the global memory manager with custom configuration
pub fn init_global_memory_manager(config: MemoryConfig) -> Result<(), MemoryManager> {
    GLOBAL_MEMORY_MANAGER.set(MemoryManager::with_config(config))
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
    fn test_memory_config_default() {
        let config = MemoryConfig::default();
        assert!(config.max_memory_usage > 0);
        assert!(config.target_memory_ratio > 0.0 && config.target_memory_ratio < 1.0);
        assert!(config.enable_tracking);
        assert!(config.enable_auto_cleanup);
    }

    #[test]
    fn test_memory_manager_creation() {
        let manager = MemoryManager::new();
        let usage = manager.get_memory_usage();

        assert_eq!(usage.total_allocated, 0);
        assert_eq!(usage.object_count, 0);
        assert!(!manager.is_memory_critical());
    }

    #[test]
    fn test_object_tracking() {
        let manager = MemoryManager::new();

        let handle = manager.track_object(1024, "test");
        assert!(handle.is_some());

        let usage = manager.get_memory_usage();
        assert_eq!(usage.total_allocated, 1024);
        assert_eq!(usage.object_count, 1);

        // Drop the handle to untrack
        drop(handle);

        let usage = manager.get_memory_usage();
        assert_eq!(usage.total_allocated, 0);
        assert_eq!(usage.object_count, 0);
    }

    #[test]
    fn test_object_access_tracking() {
        let manager = MemoryManager::new();
        let handle = manager.track_object(1024, "test").unwrap();

        // Mark as accessed
        handle.mark_accessed();

        // The access should be tracked internally
        assert_eq!(handle.id(), 1);
    }

    #[test]
    fn test_memory_usage_calculations() {
        let usage = MemoryUsage {
            total_allocated: 50 * 1024 * 1024, // 50MB
            code_blocks_memory: 0,
            strategies_memory: 0,
            cache_memory: 0,
            object_count: 100,
            large_object_count: 5,
            peak_usage: 60 * 1024 * 1024, // 60MB
            measured_at: Instant::now(),
        };

        let max_memory = 100 * 1024 * 1024; // 100MB
        assert_eq!(usage.utilization_percentage(max_memory), 50.0);
        assert!(!usage.is_critical(max_memory, 80.0));
        assert!(usage.is_critical(max_memory, 40.0));
        assert_eq!(usage.average_object_size(), 512 * 1024); // 512KB
    }

    #[test]
    fn test_cleanup_operation() {
        let config = MemoryConfig {
            max_object_age: Duration::from_millis(1), // Very short age for testing
            ..Default::default()
        };
        let manager = MemoryManager::with_config(config);

        // Track some objects
        let _handle1 = manager.track_object(1024, "test");
        let _handle2 = manager.track_object(2048, "test");

        let initial_usage = manager.get_memory_usage();
        assert_eq!(initial_usage.object_count, 2);

        // Wait for objects to become stale
        std::thread::sleep(Duration::from_millis(2));

        // Cleanup should remove stale objects
        let result = manager.cleanup();

        assert!(result.objects_removed > 0);
        assert!(result.memory_freed > 0);

        let final_usage = manager.get_memory_usage();
        assert!(final_usage.object_count < initial_usage.object_count);
    }

    #[test]
    fn test_managed_code_block() {
        let manager = MemoryManager::new();
        let block = create_test_processed_block("fn main() {}", Some("rust"));

        let managed_block = ManagedCodeBlock::new(block, &manager);

        // Should be able to access the block
        assert_eq!(managed_block.original_code, "fn main() {}");
        assert_eq!(managed_block.language, Some("rust".to_string()));

        // Memory should be tracked
        let usage = manager.get_memory_usage();
        assert!(usage.total_allocated > 0);
        assert_eq!(usage.object_count, 1);
    }

    #[test]
    fn test_memory_stats_by_category() {
        let manager = MemoryManager::new();

        let _handle1 = manager.track_object(1024, "code_block");
        let _handle2 = manager.track_object(2048, "code_block");
        let _handle3 = manager.track_object(512, "cache");

        let stats = manager.get_memory_stats_by_category();

        assert_eq!(stats.len(), 2);

        let code_block_stats = stats.get("code_block").unwrap();
        assert_eq!(code_block_stats.total_size, 3072);
        assert_eq!(code_block_stats.object_count, 2);

        let cache_stats = stats.get("cache").unwrap();
        assert_eq!(cache_stats.total_size, 512);
        assert_eq!(cache_stats.object_count, 1);
    }

    #[test]
    fn test_cleanup_result() {
        let result = CleanupResult {
            objects_removed: 5,
            memory_freed: 10 * 1024, // 10KB
            cleanup_duration: Duration::from_millis(50),
            initial_usage: 100 * 1024, // 100KB
            final_usage: 90 * 1024,    // 90KB
        };

        assert_eq!(result.memory_freed_percentage(), 10.0);
        assert!(!result.was_effective()); // 10% is not > 10%

        let effective_result = CleanupResult {
            memory_freed: 20 * 1024,   // 20KB
            initial_usage: 100 * 1024, // 100KB
            ..result
        };

        assert_eq!(effective_result.memory_freed_percentage(), 20.0);
        assert!(effective_result.was_effective()); // 20% is > 10%
    }

    #[test]
    fn test_category_stats() {
        let stats = CategoryStats {
            total_size: 10 * 1024, // 10KB
            object_count: 5,
            large_object_count: 2,
            average_age: Duration::from_secs(60),
            total_access_count: 25,
        };

        assert_eq!(stats.average_object_size(), 2 * 1024); // 2KB
        assert_eq!(stats.average_access_count(), 5.0);
        assert_eq!(stats.large_object_percentage(), 40.0);
    }

    #[test]
    fn test_global_memory_manager() {
        let manager = global_memory_manager();

        let handle = manager.track_object(1024, "global_test");
        assert!(handle.is_some());

        let usage = manager.get_memory_usage();
        assert!(usage.total_allocated >= 1024);
    }

    #[test]
    fn test_cleanup_callbacks() {
        let manager = MemoryManager::new();
        let callback_executed = Arc::new(Mutex::new(false));
        let callback_flag = callback_executed.clone();

        manager.register_cleanup_callback(move || {
            if let Ok(mut flag) = callback_flag.lock() {
                *flag = true;
            }
        });

        // Trigger cleanup
        manager.cleanup();

        // Check if callback was executed
        if let Ok(flag) = callback_executed.lock() {
            assert!(*flag);
        };
    }

    #[test]
    fn test_peak_usage_tracking() {
        let manager = MemoryManager::new();

        // Track an object
        let _handle1 = manager.track_object(1024, "test");
        let usage1 = manager.get_memory_usage();

        // Track a larger object
        let _handle2 = manager.track_object(2048, "test");
        let usage2 = manager.get_memory_usage();

        // Peak should be at least the current usage
        assert!(usage2.peak_usage >= usage2.total_allocated);
        assert!(usage2.peak_usage >= usage1.total_allocated);

        // Reset peak usage
        manager.reset_peak_usage();
        let usage3 = manager.get_memory_usage();
        assert_eq!(usage3.peak_usage, usage3.total_allocated);
    }
}
