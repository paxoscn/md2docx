//! Lazy loading system for code block strategies
//! 
//! This module provides lazy loading capabilities to improve startup performance
//! by only loading strategies when they are actually needed.

use std::collections::HashMap;
use std::sync::{Arc, RwLock, Mutex};
use std::time::Instant;

use crate::markdown::code_block::{CodeBlockStrategy, ProcessingConfig, ProcessedCodeBlock, ProcessingError};

/// A factory function that creates a strategy instance
pub type StrategyFactory = Box<dyn Fn() -> Result<Box<dyn CodeBlockStrategy>, Box<dyn std::error::Error + Send + Sync>> + Send + Sync>;

/// Information about a lazy-loaded strategy
#[derive(Debug, Clone)]
pub struct LazyStrategyInfo {
    /// The language this strategy supports
    pub language: String,
    /// Description of the strategy
    pub description: String,
    /// Priority of this strategy
    pub priority: u8,
    /// Whether the strategy has been loaded
    pub is_loaded: bool,
    /// When the strategy was last accessed
    pub last_accessed: Option<Instant>,
    /// Number of times this strategy has been accessed
    pub access_count: u64,
    /// Estimated memory usage when loaded (in bytes)
    pub estimated_memory_usage: usize,
}

/// A wrapper that holds either a loaded strategy or a factory to create one
enum StrategyHolder {
    /// Strategy is not yet loaded, contains factory
    Unloaded {
        factory: StrategyFactory,
        info: LazyStrategyInfo,
    },
    /// Strategy is loaded and ready to use
    Loaded {
        strategy: Arc<dyn CodeBlockStrategy>,
        info: LazyStrategyInfo,
    },
    /// Strategy failed to load
    Failed {
        error: String,
        info: LazyStrategyInfo,
    },
}

impl StrategyHolder {
    /// Get the strategy info
    fn get_info(&self) -> &LazyStrategyInfo {
        match self {
            StrategyHolder::Unloaded { info, .. } => info,
            StrategyHolder::Loaded { info, .. } => info,
            StrategyHolder::Failed { info, .. } => info,
        }
    }

    /// Get mutable strategy info
    fn get_info_mut(&mut self) -> &mut LazyStrategyInfo {
        match self {
            StrategyHolder::Unloaded { info, .. } => info,
            StrategyHolder::Loaded { info, .. } => info,
            StrategyHolder::Failed { info, .. } => info,
        }
    }

    /// Check if the strategy is loaded
    fn is_loaded(&self) -> bool {
        matches!(self, StrategyHolder::Loaded { .. })
    }

    /// Check if the strategy failed to load
    fn is_failed(&self) -> bool {
        matches!(self, StrategyHolder::Failed { .. })
    }
}

/// Configuration for lazy loading behavior
#[derive(Debug, Clone)]
pub struct LazyLoadingConfig {
    /// Maximum number of strategies to keep loaded simultaneously
    pub max_loaded_strategies: usize,
    /// Time after which unused strategies are unloaded
    pub unload_after: std::time::Duration,
    /// Whether to preload high-priority strategies
    pub preload_high_priority: bool,
    /// Priority threshold for preloading (strategies with priority >= this will be preloaded)
    pub preload_priority_threshold: u8,
    /// Whether to enable lazy loading statistics
    pub enable_statistics: bool,
}

impl Default for LazyLoadingConfig {
    fn default() -> Self {
        Self {
            max_loaded_strategies: 5,
            unload_after: std::time::Duration::from_secs(300), // 5 minutes
            preload_high_priority: true,
            preload_priority_threshold: 150,
            enable_statistics: true,
        }
    }
}

/// Statistics about lazy loading performance
#[derive(Debug, Clone, Default)]
pub struct LazyLoadingStatistics {
    /// Number of strategies loaded on demand
    pub strategies_loaded: u64,
    /// Number of strategies unloaded due to inactivity
    pub strategies_unloaded: u64,
    /// Number of load failures
    pub load_failures: u64,
    /// Total time spent loading strategies
    pub total_load_time: std::time::Duration,
    /// Number of cache hits (strategy already loaded)
    pub cache_hits: u64,
    /// Number of cache misses (strategy needed loading)
    pub cache_misses: u64,
}

impl LazyLoadingStatistics {
    /// Calculate the cache hit ratio
    pub fn hit_ratio(&self) -> f64 {
        let total = self.cache_hits + self.cache_misses;
        if total == 0 {
            0.0
        } else {
            self.cache_hits as f64 / total as f64
        }
    }

    /// Get average load time per strategy
    pub fn average_load_time(&self) -> std::time::Duration {
        if self.strategies_loaded == 0 {
            std::time::Duration::from_millis(0)
        } else {
            self.total_load_time / self.strategies_loaded as u32
        }
    }
}

/// Lazy loading registry for code block strategies
pub struct LazyStrategyRegistry {
    /// Map of language names to strategy holders
    strategies: RwLock<HashMap<String, StrategyHolder>>,
    /// Default strategy (always loaded)
    default_strategy: Arc<dyn CodeBlockStrategy>,
    /// Configuration for lazy loading
    config: LazyLoadingConfig,
    /// Statistics about lazy loading
    statistics: Mutex<LazyLoadingStatistics>,
    /// Language aliases
    aliases: RwLock<HashMap<String, String>>,
}

impl LazyStrategyRegistry {
    /// Create a new lazy strategy registry
    pub fn new(default_strategy: Arc<dyn CodeBlockStrategy>) -> Self {
        Self::with_config(default_strategy, LazyLoadingConfig::default())
    }

    /// Create a new lazy strategy registry with custom configuration
    pub fn with_config(
        default_strategy: Arc<dyn CodeBlockStrategy>,
        config: LazyLoadingConfig,
    ) -> Self {
        Self {
            strategies: RwLock::new(HashMap::new()),
            default_strategy,
            config,
            statistics: Mutex::new(LazyLoadingStatistics::default()),
            aliases: RwLock::new(HashMap::new()),
        }
    }

    /// Register a strategy factory for lazy loading
    pub fn register_lazy_strategy<F>(
        &self,
        language: &str,
        description: &str,
        priority: u8,
        estimated_memory_usage: usize,
        factory: F,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>
    where
        F: Fn() -> Result<Box<dyn CodeBlockStrategy>, Box<dyn std::error::Error + Send + Sync>> + Send + Sync + 'static,
    {
        let info = LazyStrategyInfo {
            language: language.to_string(),
            description: description.to_string(),
            priority,
            is_loaded: false,
            last_accessed: None,
            access_count: 0,
            estimated_memory_usage,
        };

        let holder = StrategyHolder::Unloaded {
            factory: Box::new(factory),
            info,
        };

        if let Ok(mut strategies) = self.strategies.write() {
            strategies.insert(language.to_lowercase(), holder);
        }

        // Preload if configured and priority is high enough
        if self.config.preload_high_priority && priority >= self.config.preload_priority_threshold {
            self.preload_strategy(language)?;
        }

        Ok(())
    }

    /// Get a strategy, loading it if necessary
    pub fn get_strategy(&self, language: &str) -> Arc<dyn CodeBlockStrategy> {
        let normalized_lang = language.to_lowercase();
        
        // Check for alias
        let canonical_lang = if let Ok(aliases) = self.aliases.read() {
            aliases.get(&normalized_lang).cloned().unwrap_or(normalized_lang)
        } else {
            normalized_lang
        };

        // Try to get or load the strategy
        match self.get_or_load_strategy(&canonical_lang) {
            Ok(strategy) => strategy,
            Err(_) => {
                // Fallback to default strategy
                self.default_strategy.clone()
            }
        }
    }

    /// Get or load a strategy by language
    fn get_or_load_strategy(&self, language: &str) -> Result<Arc<dyn CodeBlockStrategy>, Box<dyn std::error::Error + Send + Sync>> {
        // First, try to get an already loaded strategy
        if let Ok(strategies) = self.strategies.read() {
            if let Some(holder) = strategies.get(language) {
                if let StrategyHolder::Loaded { strategy, .. } = holder {
                    // Update access statistics
                    self.update_access_stats(language);
                    
                    if self.config.enable_statistics {
                        if let Ok(mut stats) = self.statistics.lock() {
                            stats.cache_hits += 1;
                        }
                    }
                    
                    return Ok(strategy.clone());
                }
            }
        }

        // Strategy not loaded, need to load it
        self.load_strategy(language)
    }

    /// Load a strategy from its factory
    fn load_strategy(&self, language: &str) -> Result<Arc<dyn CodeBlockStrategy>, Box<dyn std::error::Error + Send + Sync>> {
        let load_start = Instant::now();
        
        // Load strategy with write lock directly
        self.load_strategy_with_write_lock(language, load_start)
    }

    /// Load strategy with write lock (internal method)
    fn load_strategy_with_write_lock(&self, language: &str, load_start: Instant) -> Result<Arc<dyn CodeBlockStrategy>, Box<dyn std::error::Error + Send + Sync>> {
        if let Ok(mut strategies) = self.strategies.write() {
            // Check if someone else loaded it while we were waiting for the lock
            if let Some(StrategyHolder::Loaded { strategy, .. }) = strategies.get(language) {
                return Ok(strategy.clone());
            }

            // Get the holder and extract the factory
            if let Some(holder) = strategies.remove(language) {
                match holder {
                    StrategyHolder::Unloaded { factory, mut info } => {
                        // Try to create the strategy
                        match factory() {
                            Ok(strategy_box) => {
                                let strategy: Arc<dyn CodeBlockStrategy> = Arc::from(strategy_box);
                                let load_time = load_start.elapsed();
                                
                                // Update info
                                info.is_loaded = true;
                                info.last_accessed = Some(Instant::now());
                                info.access_count += 1;
                                
                                // Create loaded holder
                                let loaded_holder = StrategyHolder::Loaded {
                                    strategy: strategy.clone(),
                                    info,
                                };
                                
                                // Put back the loaded strategy
                                strategies.insert(language.to_string(), loaded_holder);
                                
                                // Update statistics
                                if self.config.enable_statistics {
                                    if let Ok(mut stats) = self.statistics.lock() {
                                        stats.strategies_loaded += 1;
                                        stats.total_load_time += load_time;
                                        stats.cache_misses += 1;
                                    }
                                }
                                
                                // Check if we need to unload old strategies
                                self.maybe_unload_old_strategies(&mut strategies);
                                
                                Ok(strategy)
                            }
                            Err(e) => {
                                // Mark as failed
                                info.is_loaded = false;
                                let failed_holder = StrategyHolder::Failed {
                                    error: e.to_string(),
                                    info,
                                };
                                strategies.insert(language.to_string(), failed_holder);
                                
                                if self.config.enable_statistics {
                                    if let Ok(mut stats) = self.statistics.lock() {
                                        stats.load_failures += 1;
                                    }
                                }
                                
                                Err(e)
                            }
                        }
                    }
                    StrategyHolder::Loaded { strategy, .. } => {
                        // Someone else loaded it
                        strategies.insert(language.to_string(), StrategyHolder::Loaded { strategy: strategy.clone(), info: LazyStrategyInfo {
                            language: language.to_string(),
                            description: "".to_string(),
                            priority: 100,
                            is_loaded: true,
                            last_accessed: Some(Instant::now()),
                            access_count: 1,
                            estimated_memory_usage: 0,
                        }});
                        Ok(strategy)
                    }
                    StrategyHolder::Failed { error, info } => {
                        // Put back the failed holder
                        strategies.insert(language.to_string(), StrategyHolder::Failed { error: error.clone(), info });
                        Err(error.into())
                    }
                }
            } else {
                Err("Strategy not found".into())
            }
        } else {
            Err("Could not acquire write lock".into())
        }
    }

    /// Update access statistics for a strategy
    fn update_access_stats(&self, language: &str) {
        if let Ok(mut strategies) = self.strategies.write() {
            if let Some(holder) = strategies.get_mut(language) {
                let info = holder.get_info_mut();
                info.last_accessed = Some(Instant::now());
                info.access_count += 1;
            }
        }
    }

    /// Maybe unload old strategies if we have too many loaded
    fn maybe_unload_old_strategies(&self, strategies: &mut HashMap<String, StrategyHolder>) {
        let loaded_count = strategies.values().filter(|h| h.is_loaded()).count();
        
        if loaded_count <= self.config.max_loaded_strategies {
            return;
        }

        // Find strategies to unload (oldest access time, excluding high priority)
        let mut candidates: Vec<_> = strategies
            .iter()
            .filter_map(|(lang, holder)| {
                if let StrategyHolder::Loaded { info, .. } = holder {
                    if info.priority < self.config.preload_priority_threshold {
                        Some((lang.clone(), info.last_accessed.unwrap_or(Instant::now())))
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .collect();

        // Sort by access time (oldest first)
        candidates.sort_by_key(|(_, access_time)| *access_time);

        // Unload the oldest strategies
        let to_unload = loaded_count - self.config.max_loaded_strategies;
        for (language, _) in candidates.into_iter().take(to_unload) {
            self.unload_strategy_internal(strategies, &language);
        }
    }

    /// Unload a specific strategy (internal method)
    fn unload_strategy_internal(&self, strategies: &mut HashMap<String, StrategyHolder>, language: &str) {
        if let Some(holder) = strategies.remove(language) {
            if let StrategyHolder::Loaded { info, .. } = holder {
                // Create a new factory that will reload the strategy
                // For simplicity, we'll mark it as failed for now
                let failed_holder = StrategyHolder::Failed {
                    error: "Strategy was unloaded due to memory pressure".to_string(),
                    info,
                };
                strategies.insert(language.to_string(), failed_holder);
                
                if self.config.enable_statistics {
                    if let Ok(mut stats) = self.statistics.lock() {
                        stats.strategies_unloaded += 1;
                    }
                }
            }
        }
    }

    /// Preload a strategy
    pub fn preload_strategy(&self, language: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let _ = self.get_or_load_strategy(language)?;
        Ok(())
    }

    /// Preload all high-priority strategies
    pub fn preload_high_priority_strategies(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let languages_to_preload: Vec<String> = {
            if let Ok(strategies) = self.strategies.read() {
                strategies
                    .iter()
                    .filter_map(|(lang, holder)| {
                        let info = holder.get_info();
                        if info.priority >= self.config.preload_priority_threshold && !holder.is_loaded() {
                            Some(lang.clone())
                        } else {
                            None
                        }
                    })
                    .collect()
            } else {
                return Err("Could not read strategies".into());
            }
        };

        for language in languages_to_preload {
            if let Err(e) = self.preload_strategy(&language) {
                eprintln!("Failed to preload strategy for {}: {}", language, e);
            }
        }

        Ok(())
    }

    /// Unload unused strategies
    pub fn unload_unused_strategies(&self) {
        let now = Instant::now();
        let languages_to_unload: Vec<String> = {
            if let Ok(strategies) = self.strategies.read() {
                strategies
                    .iter()
                    .filter_map(|(lang, holder)| {
                        if let StrategyHolder::Loaded { info, .. } = holder {
                            if let Some(last_accessed) = info.last_accessed {
                                if now.duration_since(last_accessed) > self.config.unload_after
                                    && info.priority < self.config.preload_priority_threshold
                                {
                                    return Some(lang.clone());
                                }
                            }
                        }
                        None
                    })
                    .collect()
            } else {
                return;
            }
        };

        if !languages_to_unload.is_empty() {
            if let Ok(mut strategies) = self.strategies.write() {
                for language in languages_to_unload {
                    self.unload_strategy_internal(&mut strategies, &language);
                }
            }
        }
    }

    /// Get information about all registered strategies
    pub fn list_strategy_info(&self) -> Vec<LazyStrategyInfo> {
        if let Ok(strategies) = self.strategies.read() {
            strategies
                .values()
                .map(|holder| holder.get_info().clone())
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Get lazy loading statistics
    pub fn get_statistics(&self) -> LazyLoadingStatistics {
        if let Ok(stats) = self.statistics.lock() {
            stats.clone()
        } else {
            LazyLoadingStatistics::default()
        }
    }

    /// Register a language alias
    pub fn register_alias(&self, alias: &str, canonical_language: &str) {
        if let Ok(mut aliases) = self.aliases.write() {
            aliases.insert(alias.to_lowercase(), canonical_language.to_lowercase());
        }
    }

    /// Get memory usage information
    pub fn get_memory_info(&self) -> MemoryInfo {
        if let Ok(strategies) = self.strategies.read() {
            let mut loaded_memory = 0;
            let mut estimated_total_memory = 0;
            let mut loaded_count = 0;
            let total_count = strategies.len();

            for holder in strategies.values() {
                let info = holder.get_info();
                estimated_total_memory += info.estimated_memory_usage;
                
                if holder.is_loaded() {
                    loaded_memory += info.estimated_memory_usage;
                    loaded_count += 1;
                }
            }

            MemoryInfo {
                loaded_strategies: loaded_count,
                total_strategies: total_count,
                loaded_memory_bytes: loaded_memory,
                estimated_total_memory_bytes: estimated_total_memory,
            }
        } else {
            MemoryInfo {
                loaded_strategies: 0,
                total_strategies: 0,
                loaded_memory_bytes: 0,
                estimated_total_memory_bytes: 0,
            }
        }
    }

    /// Force cleanup of failed strategies
    pub fn cleanup_failed_strategies(&self) {
        if let Ok(mut strategies) = self.strategies.write() {
            strategies.retain(|_, holder| !holder.is_failed());
        }
    }
}

/// Information about memory usage
#[derive(Debug, Clone)]
pub struct MemoryInfo {
    pub loaded_strategies: usize,
    pub total_strategies: usize,
    pub loaded_memory_bytes: usize,
    pub estimated_total_memory_bytes: usize,
}

impl MemoryInfo {
    /// Get the percentage of strategies currently loaded
    pub fn loaded_percentage(&self) -> f64 {
        if self.total_strategies == 0 {
            0.0
        } else {
            self.loaded_strategies as f64 / self.total_strategies as f64
        }
    }

    /// Get the percentage of memory currently used
    pub fn memory_usage_percentage(&self) -> f64 {
        if self.estimated_total_memory_bytes == 0 {
            0.0
        } else {
            self.loaded_memory_bytes as f64 / self.estimated_total_memory_bytes as f64
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::markdown::code_block::strategy::DefaultStrategy;

    // Mock strategy for testing
    struct MockStrategy {
        language: String,
    }

    impl MockStrategy {
        fn new(language: &str) -> Self {
            Self {
                language: language.to_string(),
            }
        }
    }

    impl CodeBlockStrategy for MockStrategy {
        fn process(&self, code: &str, _config: &ProcessingConfig) -> Result<ProcessedCodeBlock, ProcessingError> {
            Ok(ProcessedCodeBlock::new(code.to_string(), Some(self.language.clone())))
        }

        fn supports_language(&self, language: &str) -> bool {
            language.to_lowercase() == self.language.to_lowercase()
        }

        fn get_language_name(&self) -> &'static str {
            // This is a limitation of the trait - we need a static str
            // In real implementation, this would be handled differently
            "mock"
        }

        fn get_priority(&self) -> u8 {
            100
        }
    }

    #[test]
    fn test_lazy_registry_creation() {
        let default_strategy = Arc::new(DefaultStrategy::new());
        let registry = LazyStrategyRegistry::new(default_strategy);
        
        let memory_info = registry.get_memory_info();
        assert_eq!(memory_info.total_strategies, 0);
        assert_eq!(memory_info.loaded_strategies, 0);
    }

    #[test]
    fn test_register_lazy_strategy() {
        let default_strategy = Arc::new(DefaultStrategy::new());
        let registry = LazyStrategyRegistry::new(default_strategy);
        
        let result = registry.register_lazy_strategy(
            "rust",
            "Rust code processor",
            100,
            1024,
            || Ok(Box::new(MockStrategy::new("rust"))),
        );
        
        assert!(result.is_ok());
        
        let memory_info = registry.get_memory_info();
        assert_eq!(memory_info.total_strategies, 1);
        assert_eq!(memory_info.loaded_strategies, 0); // Not loaded yet
    }

    #[test]
    fn test_get_strategy_loads_on_demand() {
        let default_strategy = Arc::new(DefaultStrategy::new());
        let registry = LazyStrategyRegistry::new(default_strategy);
        
        let _ = registry.register_lazy_strategy(
            "rust",
            "Rust code processor",
            100,
            1024,
            || Ok(Box::new(MockStrategy::new("rust"))),
        );
        
        // Getting the strategy should load it
        let strategy = registry.get_strategy("rust");
        assert_eq!(strategy.get_language_name(), "mock"); // Our mock returns "mock"
        
        let memory_info = registry.get_memory_info();
        assert_eq!(memory_info.loaded_strategies, 1);
    }

    #[test]
    fn test_fallback_to_default() {
        let default_strategy = Arc::new(DefaultStrategy::new());
        let registry = LazyStrategyRegistry::new(default_strategy.clone());
        
        // Request unknown language should return default
        let strategy = registry.get_strategy("unknown");
        assert_eq!(strategy.get_language_name(), default_strategy.get_language_name());
    }

    #[test]
    fn test_preload_high_priority() {
        let config = LazyLoadingConfig {
            preload_high_priority: true,
            preload_priority_threshold: 150,
            ..Default::default()
        };
        
        let default_strategy = Arc::new(DefaultStrategy::new());
        let registry = LazyStrategyRegistry::with_config(default_strategy, config);
        
        // Register high priority strategy - should be preloaded
        let _ = registry.register_lazy_strategy(
            "rust",
            "Rust code processor",
            200, // High priority
            1024,
            || Ok(Box::new(MockStrategy::new("rust"))),
        );
        
        let memory_info = registry.get_memory_info();
        assert_eq!(memory_info.loaded_strategies, 1); // Should be preloaded
    }

    #[test]
    fn test_statistics() {
        let default_strategy = Arc::new(DefaultStrategy::new());
        let registry = LazyStrategyRegistry::new(default_strategy);
        
        let _ = registry.register_lazy_strategy(
            "rust",
            "Rust code processor",
            100,
            1024,
            || Ok(Box::new(MockStrategy::new("rust"))),
        );
        
        // Initial statistics
        let stats = registry.get_statistics();
        assert_eq!(stats.strategies_loaded, 0);
        assert_eq!(stats.cache_hits, 0);
        assert_eq!(stats.cache_misses, 0);
        
        // Load strategy
        let _ = registry.get_strategy("rust");
        
        let stats = registry.get_statistics();
        assert_eq!(stats.strategies_loaded, 1);
        assert_eq!(stats.cache_misses, 1);
        
        // Access again - should be cache hit
        let _ = registry.get_strategy("rust");
        
        let stats = registry.get_statistics();
        assert_eq!(stats.cache_hits, 1);
    }

    #[test]
    fn test_language_aliases() {
        let default_strategy = Arc::new(DefaultStrategy::new());
        let registry = LazyStrategyRegistry::new(default_strategy);
        
        let _ = registry.register_lazy_strategy(
            "rust",
            "Rust code processor",
            100,
            1024,
            || Ok(Box::new(MockStrategy::new("rust"))),
        );
        
        registry.register_alias("rs", "rust");
        
        // Should resolve alias to rust strategy
        let strategy = registry.get_strategy("rs");
        assert_eq!(strategy.get_language_name(), "mock");
    }

    #[test]
    fn test_strategy_info_listing() {
        let default_strategy = Arc::new(DefaultStrategy::new());
        let registry = LazyStrategyRegistry::new(default_strategy);
        
        let _ = registry.register_lazy_strategy(
            "rust",
            "Rust code processor",
            100,
            1024,
            || Ok(Box::new(MockStrategy::new("rust"))),
        );
        
        let _ = registry.register_lazy_strategy(
            "javascript",
            "JavaScript code processor",
            90,
            512,
            || Ok(Box::new(MockStrategy::new("javascript"))),
        );
        
        let info_list = registry.list_strategy_info();
        assert_eq!(info_list.len(), 2);
        
        let rust_info = info_list.iter().find(|info| info.language == "rust").unwrap();
        assert_eq!(rust_info.description, "Rust code processor");
        assert_eq!(rust_info.priority, 100);
        assert!(!rust_info.is_loaded);
    }

    #[test]
    fn test_cleanup_failed_strategies() {
        let default_strategy = Arc::new(DefaultStrategy::new());
        let registry = LazyStrategyRegistry::new(default_strategy);
        
        // Register a strategy that will fail to load
        let _ = registry.register_lazy_strategy(
            "failing",
            "Failing strategy",
            100,
            1024,
            || Err("Intentional failure".into()),
        );
        
        // Try to load it (will fail)
        let _ = registry.get_strategy("failing");
        
        // Should have 1 strategy (failed)
        let memory_info = registry.get_memory_info();
        assert_eq!(memory_info.total_strategies, 1);
        
        // Cleanup failed strategies
        registry.cleanup_failed_strategies();
        
        // Should have 0 strategies now
        let memory_info = registry.get_memory_info();
        assert_eq!(memory_info.total_strategies, 0);
    }
}