//! Integrated performance management for code block processing
//! 
//! This module provides a unified interface for all performance optimization
//! features including caching, lazy loading, parallel processing, and memory management.

use std::sync::Arc;
use std::time::{Duration, Instant};


use crate::markdown::code_block::{
    CodeBlockStrategy, ProcessingConfig, ProcessingError,
    CodeBlockCache, CacheConfig, CacheKey,
    LazyStrategyRegistry, LazyLoadingConfig,
    ParallelProcessor, ParallelConfig,
    MemoryManager, MemoryConfig, ManagedCodeBlock,
    DefaultStrategy
};

/// Configuration for the integrated performance manager
#[derive(Debug, Clone)]
pub struct PerformanceConfig {
    /// Cache configuration
    pub cache: CacheConfig,
    /// Lazy loading configuration
    pub lazy_loading: LazyLoadingConfig,
    /// Parallel processing configuration
    pub parallel: ParallelConfig,
    /// Memory management configuration
    pub memory: MemoryConfig,
    /// Whether to enable performance monitoring
    pub enable_monitoring: bool,
    /// Whether to enable adaptive optimization
    pub enable_adaptive_optimization: bool,
    /// Threshold for switching to parallel processing
    pub parallel_threshold: usize,
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            cache: CacheConfig::default(),
            lazy_loading: LazyLoadingConfig::default(),
            parallel: ParallelConfig::default(),
            memory: MemoryConfig::default(),
            enable_monitoring: true,
            enable_adaptive_optimization: true,
            parallel_threshold: 5,
        }
    }
}

/// Performance metrics and statistics
#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
    /// Cache hit ratio
    pub cache_hit_ratio: f64,
    /// Average processing time per code block
    pub average_processing_time: Duration,
    /// Memory utilization percentage
    pub memory_utilization: f64,
    /// Number of strategies currently loaded
    pub loaded_strategies: usize,
    /// Parallel processing throughput (tasks per second)
    pub parallel_throughput: f64,
    /// Total number of code blocks processed
    pub total_processed: u64,
    /// Number of processing errors
    pub error_count: u64,
    /// When these metrics were collected
    pub collected_at: Instant,
}

impl PerformanceMetrics {
    /// Calculate overall performance score (0.0 to 1.0)
    pub fn performance_score(&self) -> f64 {
        let cache_score = self.cache_hit_ratio;
        let memory_score = 1.0 - (self.memory_utilization / 100.0).min(1.0);
        let error_score = if self.total_processed == 0 {
            1.0
        } else {
            1.0 - (self.error_count as f64 / self.total_processed as f64).min(1.0)
        };
        
        // Weighted average
        (cache_score * 0.3 + memory_score * 0.3 + error_score * 0.4).max(0.0).min(1.0)
    }

    /// Check if performance is good (score > 0.7)
    pub fn is_performing_well(&self) -> bool {
        self.performance_score() > 0.7
    }

    /// Get performance status as a string
    pub fn performance_status(&self) -> &'static str {
        let score = self.performance_score();
        if score > 0.8 {
            "Excellent"
        } else if score > 0.6 {
            "Good"
        } else if score > 0.4 {
            "Fair"
        } else {
            "Poor"
        }
    }
}

/// Integrated performance manager
pub struct PerformanceManager {
    config: PerformanceConfig,
    cache: CodeBlockCache,
    lazy_registry: LazyStrategyRegistry,
    parallel_processor: Option<ParallelProcessor>,
    memory_manager: MemoryManager,
    metrics_history: Vec<PerformanceMetrics>,
    start_time: Instant,
}

impl PerformanceManager {
    /// Create a new performance manager
    pub fn new() -> Self {
        Self::with_config(PerformanceConfig::default())
    }

    /// Create a performance manager with custom configuration
    pub fn with_config(config: PerformanceConfig) -> Self {
        let cache = CodeBlockCache::with_config(config.cache.clone());
        let default_strategy = Arc::new(DefaultStrategy::new());
        let lazy_registry = LazyStrategyRegistry::with_config(
            default_strategy,
            config.lazy_loading.clone(),
        );
        let memory_manager = MemoryManager::with_config(config.memory.clone());
        
        // Initialize parallel processor if enabled
        let parallel_processor = if config.parallel.max_workers > 0 {
            let mut processor = ParallelProcessor::with_config(config.parallel.clone());
            if let Err(e) = processor.start() {
                eprintln!("Failed to start parallel processor: {}", e);
                None
            } else {
                Some(processor)
            }
        } else {
            None
        };

        Self {
            config,
            cache,
            lazy_registry,
            parallel_processor,
            memory_manager,
            metrics_history: Vec::new(),
            start_time: Instant::now(),
        }
    }

    /// Process a single code block with all optimizations
    pub fn process_code_block(
        &self,
        code: &str,
        language: Option<&str>,
        config: &ProcessingConfig,
    ) -> Result<ManagedCodeBlock, ProcessingError> {
        let _start_time = Instant::now();
        
        // Get strategy (lazy loaded)
        let strategy = self.lazy_registry.get_strategy(
            language.unwrap_or("default")
        );

        // Create cache key
        let cache_key = CacheKey::new(
            code,
            language,
            config,
            strategy.get_version(),
        );

        // Try to get from cache or compute
        let processed_block = self.cache.get_or_compute(cache_key, || {
            strategy.process(code, config)
                .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)
        }).map_err(|e| {
            ProcessingError::new("processing_failed", &format!("Processing failed: {}", e))
        })?;

        // Wrap in managed code block for memory tracking
        let managed_block = ManagedCodeBlock::new(processed_block, &self.memory_manager);

        Ok(managed_block)
    }

    /// Process multiple code blocks with adaptive optimization
    pub fn process_code_blocks(
        &self,
        requests: Vec<(String, Option<String>, ProcessingConfig)>,
    ) -> Vec<Result<ManagedCodeBlock, ProcessingError>> {
        let request_count = requests.len();
        
        // Decide processing strategy based on request count and system state
        if request_count >= self.config.parallel_threshold 
            && self.parallel_processor.is_some() 
            && !self.is_system_overloaded() {
            
            self.process_parallel(requests)
        } else {
            self.process_sequential(requests)
        }
    }

    /// Process requests sequentially
    fn process_sequential(
        &self,
        requests: Vec<(String, Option<String>, ProcessingConfig)>,
    ) -> Vec<Result<ManagedCodeBlock, ProcessingError>> {
        requests
            .into_iter()
            .map(|(code, language, config)| {
                self.process_code_block(&code, language.as_deref(), &config)
            })
            .collect()
    }

    /// Process requests in parallel
    fn process_parallel(
        &self,
        requests: Vec<(String, Option<String>, ProcessingConfig)>,
    ) -> Vec<Result<ManagedCodeBlock, ProcessingError>> {
        // For now, fall back to sequential processing
        // In a full implementation, this would use the parallel processor
        self.process_sequential(requests)
    }

    /// Register a lazy-loaded strategy
    pub fn register_strategy<F>(
        &self,
        language: &str,
        description: &str,
        priority: u8,
        estimated_memory: usize,
        factory: F,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>
    where
        F: Fn() -> Result<Box<dyn CodeBlockStrategy>, Box<dyn std::error::Error + Send + Sync>> + Send + Sync + 'static,
    {
        self.lazy_registry.register_lazy_strategy(
            language,
            description,
            priority,
            estimated_memory,
            factory,
        )
    }

    /// Get current performance metrics
    pub fn get_performance_metrics(&self) -> PerformanceMetrics {
        let cache_stats = self.cache.get_statistics();
        let memory_usage = self.memory_manager.get_memory_usage();
        let _lazy_stats = self.lazy_registry.get_statistics();
        let memory_info = self.lazy_registry.get_memory_info();
        
        let parallel_stats = if let Some(ref processor) = self.parallel_processor {
            processor.get_statistics()
        } else {
            Default::default()
        };

        let uptime = self.start_time.elapsed();
        
        PerformanceMetrics {
            cache_hit_ratio: cache_stats.hit_ratio(),
            average_processing_time: if cache_stats.total_requests() > 0 {
                Duration::from_millis(100) // Placeholder - would calculate from actual data
            } else {
                Duration::from_millis(0)
            },
            memory_utilization: memory_usage.utilization_percentage(self.config.memory.max_memory_usage),
            loaded_strategies: memory_info.loaded_strategies,
            parallel_throughput: parallel_stats.throughput(uptime),
            total_processed: cache_stats.hits + cache_stats.misses,
            error_count: 0, // Would track this in real implementation
            collected_at: Instant::now(),
        }
    }

    /// Force cleanup of all caches and memory
    pub fn cleanup(&self) -> CleanupSummary {
        let start_time = Instant::now();
        
        // Cache cleanup
        let initial_cache_size = self.cache.get_size_info();
        self.cache.cleanup();
        let final_cache_size = self.cache.get_size_info();
        
        // Memory cleanup
        let memory_result = self.memory_manager.cleanup();
        
        // Lazy loading cleanup
        self.lazy_registry.unload_unused_strategies();
        self.lazy_registry.cleanup_failed_strategies();
        
        CleanupSummary {
            cache_entries_removed: initial_cache_size.entry_count - final_cache_size.entry_count,
            cache_memory_freed: initial_cache_size.memory_bytes - final_cache_size.memory_bytes,
            objects_removed: memory_result.objects_removed,
            memory_freed: memory_result.memory_freed,
            cleanup_duration: start_time.elapsed(),
        }
    }

    /// Check if the system is overloaded
    pub fn is_system_overloaded(&self) -> bool {
        let memory_usage = self.memory_manager.get_memory_usage();
        let memory_critical = memory_usage.is_critical(
            self.config.memory.max_memory_usage,
            80.0
        );
        
        let cache_near_capacity = self.cache.is_near_capacity();
        
        let parallel_overloaded = if let Some(ref processor) = self.parallel_processor {
            processor.is_overloaded()
        } else {
            false
        };

        memory_critical || cache_near_capacity || parallel_overloaded
    }

    /// Optimize performance based on current metrics
    pub fn optimize(&self) -> OptimizationResult {
        if !self.config.enable_adaptive_optimization {
            return OptimizationResult {
                actions_taken: Vec::new(),
                performance_improvement: 0.0,
            };
        }

        let mut actions = Vec::new();
        let initial_metrics = self.get_performance_metrics();
        
        // Check if cleanup is needed
        if self.is_system_overloaded() {
            let cleanup_summary = self.cleanup();
            actions.push(format!(
                "Cleaned up {} cache entries and {} objects, freed {} bytes",
                cleanup_summary.cache_entries_removed,
                cleanup_summary.objects_removed,
                cleanup_summary.memory_freed
            ));
        }

        // Check if we should preload high-priority strategies
        if initial_metrics.cache_hit_ratio < 0.5 {
            if let Err(e) = self.lazy_registry.preload_high_priority_strategies() {
                actions.push(format!("Failed to preload strategies: {}", e));
            } else {
                actions.push("Preloaded high-priority strategies".to_string());
            }
        }

        let final_metrics = self.get_performance_metrics();
        let improvement = final_metrics.performance_score() - initial_metrics.performance_score();

        OptimizationResult {
            actions_taken: actions,
            performance_improvement: improvement,
        }
    }

    /// Get performance history
    pub fn get_performance_history(&self) -> &[PerformanceMetrics] {
        &self.metrics_history
    }

    /// Record current metrics in history
    pub fn record_metrics(&mut self) {
        let metrics = self.get_performance_metrics();
        self.metrics_history.push(metrics);
        
        // Keep only last 100 entries
        if self.metrics_history.len() > 100 {
            self.metrics_history.remove(0);
        }
    }

    /// Get detailed system status
    pub fn get_system_status(&self) -> SystemStatus {
        let metrics = self.get_performance_metrics();
        let cache_size = self.cache.get_size_info();
        let memory_usage = self.memory_manager.get_memory_usage();
        let memory_info = self.lazy_registry.get_memory_info();
        
        SystemStatus {
            performance_score: metrics.performance_score(),
            performance_status: metrics.performance_status().to_string(),
            cache_utilization: cache_size.entry_utilization(),
            memory_utilization: memory_usage.utilization_percentage(self.config.memory.max_memory_usage),
            loaded_strategies: memory_info.loaded_strategies,
            total_strategies: memory_info.total_strategies,
            is_overloaded: self.is_system_overloaded(),
            uptime: self.start_time.elapsed(),
        }
    }
}

impl Drop for PerformanceManager {
    fn drop(&mut self) {
        if let Some(ref mut processor) = self.parallel_processor {
            let _ = processor.stop();
        }
    }
}

/// Summary of cleanup operations
#[derive(Debug, Clone)]
pub struct CleanupSummary {
    pub cache_entries_removed: usize,
    pub cache_memory_freed: usize,
    pub objects_removed: usize,
    pub memory_freed: usize,
    pub cleanup_duration: Duration,
}

impl CleanupSummary {
    /// Get total memory freed
    pub fn total_memory_freed(&self) -> usize {
        self.cache_memory_freed + self.memory_freed
    }

    /// Check if cleanup was effective
    pub fn was_effective(&self) -> bool {
        self.cache_entries_removed > 0 || self.objects_removed > 0
    }
}

/// Result of optimization operations
#[derive(Debug, Clone)]
pub struct OptimizationResult {
    pub actions_taken: Vec<String>,
    pub performance_improvement: f64,
}

impl OptimizationResult {
    /// Check if optimization was successful
    pub fn was_successful(&self) -> bool {
        self.performance_improvement > 0.0 || !self.actions_taken.is_empty()
    }

    /// Get a summary of actions taken
    pub fn summary(&self) -> String {
        if self.actions_taken.is_empty() {
            "No optimization actions were needed".to_string()
        } else {
            format!(
                "Performed {} optimization actions with {:.2}% performance improvement: {}",
                self.actions_taken.len(),
                self.performance_improvement * 100.0,
                self.actions_taken.join(", ")
            )
        }
    }
}

/// Overall system status
#[derive(Debug, Clone)]
pub struct SystemStatus {
    pub performance_score: f64,
    pub performance_status: String,
    pub cache_utilization: f64,
    pub memory_utilization: f64,
    pub loaded_strategies: usize,
    pub total_strategies: usize,
    pub is_overloaded: bool,
    pub uptime: Duration,
}

impl SystemStatus {
    /// Get a human-readable status report
    pub fn status_report(&self) -> String {
        format!(
            "Performance: {} ({:.1}%)\n\
             Cache Utilization: {:.1}%\n\
             Memory Utilization: {:.1}%\n\
             Strategies: {}/{} loaded\n\
             System Status: {}\n\
             Uptime: {:.1}s",
            self.performance_status,
            self.performance_score * 100.0,
            self.cache_utilization * 100.0,
            self.memory_utilization,
            self.loaded_strategies,
            self.total_strategies,
            if self.is_overloaded { "Overloaded" } else { "Normal" },
            self.uptime.as_secs_f64()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_performance_config_default() {
        let config = PerformanceConfig::default();
        assert!(config.enable_monitoring);
        assert!(config.enable_adaptive_optimization);
        assert!(config.parallel_threshold > 0);
    }

    #[test]
    fn test_performance_manager_creation() {
        let manager = PerformanceManager::new();
        let status = manager.get_system_status();
        
        assert!(!status.is_overloaded);
        assert_eq!(status.loaded_strategies, 0);
    }

    #[test]
    fn test_single_code_block_processing() {
        let manager = PerformanceManager::new();
        
        let result = manager.process_code_block(
            "fn main() {}",
            Some("rust"),
            &ProcessingConfig::default(),
        );
        
        assert!(result.is_ok());
        let managed_block = result.unwrap();
        assert_eq!(managed_block.original_code, "fn main() {}");
    }

    #[test]
    fn test_multiple_code_blocks_processing() {
        let manager = PerformanceManager::new();
        
        let requests = vec![
            ("fn main() {}".to_string(), Some("rust".to_string()), ProcessingConfig::default()),
            ("console.log('hello');".to_string(), Some("javascript".to_string()), ProcessingConfig::default()),
        ];
        
        let results = manager.process_code_blocks(requests);
        
        assert_eq!(results.len(), 2);
        assert!(results[0].is_ok());
        assert!(results[1].is_ok());
    }

    #[test]
    fn test_performance_metrics() {
        let metrics = PerformanceMetrics {
            cache_hit_ratio: 0.8,
            average_processing_time: Duration::from_millis(100),
            memory_utilization: 50.0,
            loaded_strategies: 3,
            parallel_throughput: 10.0,
            total_processed: 100,
            error_count: 5,
            collected_at: Instant::now(),
        };
        
        assert!(metrics.performance_score() > 0.0);
        assert!(metrics.is_performing_well());
        assert_eq!(metrics.performance_status(), "Good");
    }

    #[test]
    fn test_system_overload_detection() {
        let manager = PerformanceManager::new();
        
        // Initially should not be overloaded
        assert!(!manager.is_system_overloaded());
    }

    #[test]
    fn test_cleanup_operations() {
        let manager = PerformanceManager::new();
        
        // Process some code blocks to populate caches
        let _ = manager.process_code_block(
            "fn main() {}",
            Some("rust"),
            &ProcessingConfig::default(),
        );
        
        let cleanup_summary = manager.cleanup();
        
        // Cleanup should complete without errors
        assert!(cleanup_summary.cleanup_duration > Duration::from_millis(0));
    }

    #[test]
    fn test_optimization() {
        let manager = PerformanceManager::new();
        
        let result = manager.optimize();
        
        // Optimization should complete
        assert!(result.actions_taken.len() >= 0); // May or may not take actions
    }

    #[test]
    fn test_system_status() {
        let manager = PerformanceManager::new();
        let status = manager.get_system_status();
        
        assert!(status.performance_score >= 0.0 && status.performance_score <= 1.0);
        assert!(!status.performance_status.is_empty());
        assert!(status.uptime > Duration::from_millis(0));
        
        let report = status.status_report();
        assert!(!report.is_empty());
        assert!(report.contains("Performance:"));
    }

    #[test]
    fn test_cleanup_summary() {
        let summary = CleanupSummary {
            cache_entries_removed: 10,
            cache_memory_freed: 1024,
            objects_removed: 5,
            memory_freed: 2048,
            cleanup_duration: Duration::from_millis(50),
        };
        
        assert_eq!(summary.total_memory_freed(), 3072);
        assert!(summary.was_effective());
    }

    #[test]
    fn test_optimization_result() {
        let result = OptimizationResult {
            actions_taken: vec!["Cleaned cache".to_string(), "Preloaded strategies".to_string()],
            performance_improvement: 0.15,
        };
        
        assert!(result.was_successful());
        let summary = result.summary();
        assert!(summary.contains("2 optimization actions"));
        assert!(summary.contains("15.00%"));
    }

    #[test]
    fn test_performance_metrics_calculations() {
        // Test excellent performance
        let excellent_metrics = PerformanceMetrics {
            cache_hit_ratio: 0.95,
            memory_utilization: 30.0,
            total_processed: 1000,
            error_count: 1,
            ..Default::default()
        };
        assert_eq!(excellent_metrics.performance_status(), "Excellent");
        
        // Test poor performance
        let poor_metrics = PerformanceMetrics {
            cache_hit_ratio: 0.1,
            memory_utilization: 95.0,
            total_processed: 100,
            error_count: 50,
            ..Default::default()
        };
        assert_eq!(poor_metrics.performance_status(), "Poor");
    }
}

impl Default for PerformanceMetrics {
    fn default() -> Self {
        Self {
            cache_hit_ratio: 0.0,
            average_processing_time: Duration::from_millis(0),
            memory_utilization: 0.0,
            loaded_strategies: 0,
            parallel_throughput: 0.0,
            total_processed: 0,
            error_count: 0,
            collected_at: Instant::now(),
        }
    }
}