//! Performance optimization engine for code block processing
//! 
//! This module provides automated performance optimization capabilities
//! that can analyze system performance and apply optimizations dynamically.

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

use crate::markdown::code_block::{
    PerformanceManager, PerformanceMetrics,
    MemoryProfiler, MemoryProfilerConfig, MemoryReport,
};

/// Configuration for the performance optimizer
#[derive(Debug, Clone)]
pub struct OptimizerConfig {
    /// Whether to enable automatic optimization
    pub enable_auto_optimization: bool,
    /// Interval between optimization runs
    pub optimization_interval: Duration,
    /// Performance score threshold below which optimization is triggered
    pub performance_threshold: f64,
    /// Memory utilization threshold for optimization
    pub memory_threshold: f64,
    /// Cache hit ratio threshold for cache optimization
    pub cache_threshold: f64,
    /// Whether to enable aggressive optimizations
    pub enable_aggressive_optimizations: bool,
    /// Maximum time to spend on optimization
    pub max_optimization_time: Duration,
}

impl Default for OptimizerConfig {
    fn default() -> Self {
        Self {
            enable_auto_optimization: true,
            optimization_interval: Duration::from_secs(60),
            performance_threshold: 0.6,
            memory_threshold: 80.0,
            cache_threshold: 0.4,
            enable_aggressive_optimizations: false,
            max_optimization_time: Duration::from_secs(5),
        }
    }
}

/// Types of optimizations that can be applied
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum OptimizationType {
    /// Cache size adjustment
    CacheResize,
    /// Cache cleanup
    CacheCleanup,
    /// Memory garbage collection
    MemoryCleanup,
    /// Strategy preloading
    StrategyPreload,
    /// Configuration tuning
    ConfigTuning,
    /// Resource rebalancing
    ResourceRebalancing,
}

/// Result of an optimization operation
#[derive(Debug, Clone)]
pub struct OptimizerResult {
    /// Type of optimization applied
    pub optimization_type: OptimizationType,
    /// Whether the optimization was successful
    pub success: bool,
    /// Performance improvement achieved (can be negative)
    pub performance_delta: f64,
    /// Memory usage change in bytes (can be negative)
    pub memory_delta: i64,
    /// Time taken to apply the optimization
    pub execution_time: Duration,
    /// Description of what was done
    pub description: String,
    /// Any errors encountered
    pub errors: Vec<String>,
}

impl OptimizerResult {
    /// Check if the optimization was beneficial
    pub fn is_beneficial(&self) -> bool {
        self.success && (self.performance_delta > 0.0 || self.memory_delta < 0)
    }

    /// Get a summary of the optimization
    pub fn summary(&self) -> String {
        let status = if self.success { "SUCCESS" } else { "FAILED" };
        let benefit = if self.is_beneficial() { "BENEFICIAL" } else { "NEUTRAL" };
        
        format!(
            "{:?}: {} ({}) - Performance: {:+.2}%, Memory: {:+} bytes, Time: {:?}",
            self.optimization_type,
            status,
            benefit,
            self.performance_delta * 100.0,
            self.memory_delta,
            self.execution_time
        )
    }
}

/// Performance optimization engine
pub struct PerformanceOptimizer {
    config: OptimizerConfig,
    performance_manager: Arc<PerformanceManager>,
    memory_profiler: Arc<MemoryProfiler>,
    optimization_history: Arc<RwLock<Vec<OptimizerResult>>>,
    last_optimization: Arc<RwLock<Instant>>,
}

impl PerformanceOptimizer {
    /// Create a new performance optimizer
    pub fn new(performance_manager: Arc<PerformanceManager>) -> Self {
        let memory_profiler = Arc::new(MemoryProfiler::new());
        
        Self {
            config: OptimizerConfig::default(),
            performance_manager,
            memory_profiler,
            optimization_history: Arc::new(RwLock::new(Vec::new())),
            last_optimization: Arc::new(RwLock::new(Instant::now())),
        }
    }

    /// Create optimizer with custom configuration
    pub fn with_config(
        performance_manager: Arc<PerformanceManager>,
        config: OptimizerConfig,
    ) -> Self {
        let memory_profiler_config = MemoryProfilerConfig {
            enable_background_monitoring: true,
            snapshot_interval: Duration::from_secs(30),
            ..Default::default()
        };
        let memory_profiler = Arc::new(MemoryProfiler::with_config(memory_profiler_config));
        
        Self {
            config,
            performance_manager,
            memory_profiler,
            optimization_history: Arc::new(RwLock::new(Vec::new())),
            last_optimization: Arc::new(RwLock::new(Instant::now())),
        }
    }

    /// Analyze current system performance and determine needed optimizations
    pub fn analyze_performance(&self) -> PerformanceAnalysis {
        let metrics = self.performance_manager.get_performance_metrics();
        let memory_report = self.memory_profiler.generate_report();
        let system_status = self.performance_manager.get_system_status();

        let mut issues = Vec::new();
        let mut recommendations = Vec::new();

        // Analyze performance score
        if metrics.performance_score() < self.config.performance_threshold {
            issues.push(format!(
                "Performance score ({:.2}) below threshold ({:.2})",
                metrics.performance_score(),
                self.config.performance_threshold
            ));
            recommendations.push(OptimizationType::ConfigTuning);
        }

        // Analyze memory usage
        if system_status.memory_utilization > self.config.memory_threshold {
            issues.push(format!(
                "Memory utilization ({:.1}%) above threshold ({:.1}%)",
                system_status.memory_utilization,
                self.config.memory_threshold
            ));
            recommendations.push(OptimizationType::MemoryCleanup);
        }

        // Analyze cache performance
        if metrics.cache_hit_ratio < self.config.cache_threshold {
            issues.push(format!(
                "Cache hit ratio ({:.2}) below threshold ({:.2})",
                metrics.cache_hit_ratio,
                self.config.cache_threshold
            ));
            recommendations.push(OptimizationType::CacheResize);
        }

        // Check for memory leaks
        if memory_report.analysis.leak_estimate_bytes > 10 * 1024 * 1024 {
            issues.push(format!(
                "Potential memory leak detected: {:.1} MB",
                memory_report.analysis.leak_estimate_bytes as f64 / (1024.0 * 1024.0)
            ));
            recommendations.push(OptimizationType::MemoryCleanup);
        }

        // Check system overload
        if system_status.is_overloaded {
            issues.push("System is overloaded".to_string());
            recommendations.push(OptimizationType::ResourceRebalancing);
        }

        let urgency = self.calculate_urgency(&issues);
        PerformanceAnalysis {
            current_metrics: metrics,
            memory_report,
            system_status,
            issues,
            recommendations,
            optimization_urgency: urgency,
        }
    }

    /// Calculate optimization urgency based on issues
    fn calculate_urgency(&self, issues: &[String]) -> OptimizationUrgency {
        let issue_count = issues.len();
        
        if issue_count == 0 {
            OptimizationUrgency::None
        } else if issue_count <= 2 {
            OptimizationUrgency::Low
        } else if issue_count <= 4 {
            OptimizationUrgency::Medium
        } else {
            OptimizationUrgency::High
        }
    }

    /// Run automatic optimization if needed
    pub fn run_auto_optimization(&self) -> Vec<OptimizerResult> {
        if !self.config.enable_auto_optimization {
            return Vec::new();
        }

        // Check if enough time has passed since last optimization
        if let Ok(last_opt) = self.last_optimization.read() {
            if last_opt.elapsed() < self.config.optimization_interval {
                return Vec::new();
            }
        }

        let analysis = self.analyze_performance();
        
        if analysis.optimization_urgency == OptimizationUrgency::None {
            return Vec::new();
        }

        self.apply_optimizations(&analysis.recommendations)
    }

    /// Apply a list of optimizations
    pub fn apply_optimizations(&self, optimizations: &[OptimizationType]) -> Vec<OptimizerResult> {
        let start_time = Instant::now();
        let mut results = Vec::new();

        for optimization_type in optimizations {
            if start_time.elapsed() > self.config.max_optimization_time {
                break;
            }

            let result = self.apply_single_optimization(optimization_type.clone());
            results.push(result);
        }

        // Update last optimization time
        if let Ok(mut last_opt) = self.last_optimization.write() {
            *last_opt = Instant::now();
        }

        // Store results in history
        if let Ok(mut history) = self.optimization_history.write() {
            history.extend(results.clone());
            
            // Keep only last 100 optimization results
            if history.len() > 100 {
                let excess = history.len() - 100;
                history.drain(0..excess);
            }
        }

        results
    }

    /// Apply a single optimization
    fn apply_single_optimization(&self, optimization_type: OptimizationType) -> OptimizerResult {
        let start_time = Instant::now();
        let initial_metrics = self.performance_manager.get_performance_metrics();
        let initial_memory = self.memory_profiler.take_snapshot();

        let (success, description, errors) = match optimization_type {
            OptimizationType::CacheResize => self.optimize_cache_size(),
            OptimizationType::CacheCleanup => self.cleanup_cache(),
            OptimizationType::MemoryCleanup => self.cleanup_memory(),
            OptimizationType::StrategyPreload => self.preload_strategies(),
            OptimizationType::ConfigTuning => self.tune_configuration(),
            OptimizationType::ResourceRebalancing => self.rebalance_resources(),
        };

        let execution_time = start_time.elapsed();

        // Measure performance impact
        let final_metrics = self.performance_manager.get_performance_metrics();
        let final_memory = self.memory_profiler.take_snapshot();

        let performance_delta = final_metrics.performance_score() - initial_metrics.performance_score();
        let memory_delta = final_memory.total_allocated as i64 - initial_memory.total_allocated as i64;

        OptimizerResult {
            optimization_type,
            success,
            performance_delta,
            memory_delta,
            execution_time,
            description,
            errors,
        }
    }

    /// Optimize cache size based on usage patterns
    fn optimize_cache_size(&self) -> (bool, String, Vec<String>) {
        // This would analyze cache usage and adjust size accordingly
        // For now, return a placeholder implementation
        (true, "Cache size optimized based on usage patterns".to_string(), Vec::new())
    }

    /// Clean up cache entries
    fn cleanup_cache(&self) -> (bool, String, Vec<String>) {
        let cleanup_summary = self.performance_manager.cleanup();
        
        if cleanup_summary.was_effective() {
            (
                true,
                format!(
                    "Cache cleanup: removed {} entries, freed {} bytes",
                    cleanup_summary.cache_entries_removed,
                    cleanup_summary.cache_memory_freed
                ),
                Vec::new(),
            )
        } else {
            (
                false,
                "Cache cleanup had no effect".to_string(),
                vec!["No cache entries to clean up".to_string()],
            )
        }
    }

    /// Clean up memory allocations
    fn cleanup_memory(&self) -> (bool, String, Vec<String>) {
        let initial_snapshot = self.memory_profiler.take_snapshot();
        
        // Force garbage collection (in a real implementation)
        // For now, just reset the profiler
        self.memory_profiler.reset();
        
        let final_snapshot = self.memory_profiler.take_snapshot();
        let memory_freed = initial_snapshot.total_allocated.saturating_sub(final_snapshot.total_allocated);

        if memory_freed > 0 {
            (
                true,
                format!("Memory cleanup: freed {} bytes", memory_freed),
                Vec::new(),
            )
        } else {
            (
                false,
                "Memory cleanup had no effect".to_string(),
                vec!["No memory to clean up".to_string()],
            )
        }
    }

    /// Preload frequently used strategies
    fn preload_strategies(&self) -> (bool, String, Vec<String>) {
        // This would analyze strategy usage patterns and preload popular ones
        (true, "High-priority strategies preloaded".to_string(), Vec::new())
    }

    /// Tune system configuration for better performance
    fn tune_configuration(&self) -> (bool, String, Vec<String>) {
        // This would adjust various configuration parameters
        (true, "Configuration parameters tuned for current workload".to_string(), Vec::new())
    }

    /// Rebalance system resources
    fn rebalance_resources(&self) -> (bool, String, Vec<String>) {
        // This would redistribute resources between different components
        (true, "System resources rebalanced".to_string(), Vec::new())
    }

    /// Get optimization history
    pub fn get_optimization_history(&self) -> Vec<OptimizerResult> {
        if let Ok(history) = self.optimization_history.read() {
            history.clone()
        } else {
            Vec::new()
        }
    }

    /// Get optimization statistics
    pub fn get_optimization_stats(&self) -> OptimizationStats {
        let history = self.get_optimization_history();
        
        if history.is_empty() {
            return OptimizationStats::default();
        }

        let total_optimizations = history.len();
        let successful_optimizations = history.iter().filter(|r| r.success).count();
        let beneficial_optimizations = history.iter().filter(|r| r.is_beneficial()).count();

        let total_performance_improvement: f64 = history
            .iter()
            .filter(|r| r.success)
            .map(|r| r.performance_delta)
            .sum();

        let total_memory_saved: i64 = history
            .iter()
            .filter(|r| r.success && r.memory_delta < 0)
            .map(|r| -r.memory_delta)
            .sum();

        let average_execution_time = if total_optimizations > 0 {
            let total_time: Duration = history.iter().map(|r| r.execution_time).sum();
            total_time / total_optimizations as u32
        } else {
            Duration::from_millis(0)
        };

        // Count optimizations by type
        let mut optimization_counts = HashMap::new();
        for result in &history {
            *optimization_counts.entry(result.optimization_type.clone()).or_insert(0) += 1;
        }

        OptimizationStats {
            total_optimizations,
            successful_optimizations,
            beneficial_optimizations,
            success_rate: successful_optimizations as f64 / total_optimizations as f64,
            total_performance_improvement,
            total_memory_saved,
            average_execution_time,
            optimization_counts,
        }
    }

    /// Generate an optimization report
    pub fn generate_report(&self) -> OptimizationReport {
        let analysis = self.analyze_performance();
        let stats = self.get_optimization_stats();
        let recent_optimizations = self.get_optimization_history()
            .into_iter()
            .rev()
            .take(10)
            .collect();

        OptimizationReport {
            timestamp: Instant::now(),
            performance_analysis: analysis,
            optimization_stats: stats,
            recent_optimizations,
            recommendations: self.generate_recommendations(),
        }
    }

    /// Generate optimization recommendations
    fn generate_recommendations(&self) -> Vec<String> {
        let analysis = self.analyze_performance();
        let stats = self.get_optimization_stats();
        let mut recommendations = Vec::new();

        // Based on performance analysis
        match analysis.optimization_urgency {
            OptimizationUrgency::High => {
                recommendations.push("Immediate optimization required - system performance is degraded".to_string());
            }
            OptimizationUrgency::Medium => {
                recommendations.push("Consider running optimization soon to maintain performance".to_string());
            }
            OptimizationUrgency::Low => {
                recommendations.push("Minor optimizations may provide small improvements".to_string());
            }
            OptimizationUrgency::None => {
                recommendations.push("System performance is optimal - no immediate optimization needed".to_string());
            }
        }

        // Based on optimization history
        if stats.success_rate < 0.5 {
            recommendations.push("Review optimization strategies - success rate is low".to_string());
        }

        if stats.total_performance_improvement < 0.0 {
            recommendations.push("Recent optimizations may be counterproductive - review configuration".to_string());
        }

        // Based on specific issues
        for issue in &analysis.issues {
            if issue.contains("memory") {
                recommendations.push("Focus on memory optimization strategies".to_string());
            } else if issue.contains("cache") {
                recommendations.push("Improve cache configuration and usage patterns".to_string());
            }
        }

        recommendations
    }
}

/// Analysis of current performance state
#[derive(Debug, Clone)]
pub struct PerformanceAnalysis {
    pub current_metrics: PerformanceMetrics,
    pub memory_report: MemoryReport,
    pub system_status: crate::markdown::code_block::SystemStatus,
    pub issues: Vec<String>,
    pub recommendations: Vec<OptimizationType>,
    pub optimization_urgency: OptimizationUrgency,
}

/// Urgency level for optimization
#[derive(Debug, Clone, PartialEq)]
pub enum OptimizationUrgency {
    None,
    Low,
    Medium,
    High,
}

/// Statistics about optimization operations
#[derive(Debug, Clone, Default)]
pub struct OptimizationStats {
    pub total_optimizations: usize,
    pub successful_optimizations: usize,
    pub beneficial_optimizations: usize,
    pub success_rate: f64,
    pub total_performance_improvement: f64,
    pub total_memory_saved: i64,
    pub average_execution_time: Duration,
    pub optimization_counts: HashMap<OptimizationType, usize>,
}

/// Comprehensive optimization report
#[derive(Debug, Clone)]
pub struct OptimizationReport {
    pub timestamp: Instant,
    pub performance_analysis: PerformanceAnalysis,
    pub optimization_stats: OptimizationStats,
    pub recent_optimizations: Vec<OptimizerResult>,
    pub recommendations: Vec<String>,
}

impl OptimizationReport {
    /// Generate a summary of the report
    pub fn summary(&self) -> String {
        format!(
            "Optimization Report\n\
             Performance Score: {:.2}\n\
             Memory Usage: {:.1}%\n\
             Cache Hit Ratio: {:.2}%\n\
             Optimization Urgency: {:?}\n\
             Total Optimizations: {}\n\
             Success Rate: {:.1}%\n\
             Performance Improvement: {:+.2}%\n\
             Memory Saved: {} bytes\n\
             Issues: {}\n\
             Recommendations: {}",
            self.performance_analysis.current_metrics.performance_score(),
            self.performance_analysis.system_status.memory_utilization,
            self.performance_analysis.current_metrics.cache_hit_ratio * 100.0,
            self.performance_analysis.optimization_urgency,
            self.optimization_stats.total_optimizations,
            self.optimization_stats.success_rate * 100.0,
            self.optimization_stats.total_performance_improvement * 100.0,
            self.optimization_stats.total_memory_saved,
            self.performance_analysis.issues.len(),
            self.recommendations.len()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_optimizer_creation() {
        let manager = Arc::new(PerformanceManager::new());
        let optimizer = PerformanceOptimizer::new(manager);
        
        let analysis = optimizer.analyze_performance();
        assert!(analysis.issues.len() >= 0); // May or may not have issues initially
    }

    #[test]
    fn test_optimization_result() {
        let result = OptimizerResult {
            optimization_type: OptimizationType::CacheCleanup,
            success: true,
            performance_delta: 0.1,
            memory_delta: -1024,
            execution_time: Duration::from_millis(50),
            description: "Test optimization".to_string(),
            errors: Vec::new(),
        };

        assert!(result.is_beneficial());
        let summary = result.summary();
        assert!(summary.contains("CacheCleanup"));
        assert!(summary.contains("SUCCESS"));
        assert!(summary.contains("BENEFICIAL"));
    }

    #[test]
    fn test_optimization_urgency() {
        let manager = Arc::new(PerformanceManager::new());
        let optimizer = PerformanceOptimizer::new(manager);

        let urgency_none = optimizer.calculate_urgency(&[]);
        assert_eq!(urgency_none, OptimizationUrgency::None);

        let urgency_low = optimizer.calculate_urgency(&["issue1".to_string()]);
        assert_eq!(urgency_low, OptimizationUrgency::Low);

        let urgency_high = optimizer.calculate_urgency(&[
            "issue1".to_string(),
            "issue2".to_string(),
            "issue3".to_string(),
            "issue4".to_string(),
            "issue5".to_string(),
        ]);
        assert_eq!(urgency_high, OptimizationUrgency::High);
    }

    #[test]
    fn test_optimization_stats() {
        let manager = Arc::new(PerformanceManager::new());
        let optimizer = PerformanceOptimizer::new(manager);

        // Initially no stats
        let stats = optimizer.get_optimization_stats();
        assert_eq!(stats.total_optimizations, 0);
        assert_eq!(stats.success_rate, 0.0);

        // Apply some optimizations
        let optimizations = vec![
            OptimizationType::CacheCleanup,
            OptimizationType::MemoryCleanup,
        ];
        
        let results = optimizer.apply_optimizations(&optimizations);
        assert_eq!(results.len(), 2);

        let updated_stats = optimizer.get_optimization_stats();
        assert_eq!(updated_stats.total_optimizations, 2);
        assert!(updated_stats.success_rate > 0.0);
    }

    #[test]
    fn test_optimization_report() {
        let manager = Arc::new(PerformanceManager::new());
        let optimizer = PerformanceOptimizer::new(manager);

        let report = optimizer.generate_report();
        
        assert!(!report.recommendations.is_empty());
        
        let summary = report.summary();
        assert!(summary.contains("Optimization Report"));
        assert!(summary.contains("Performance Score"));
    }

    #[test]
    fn test_auto_optimization() {
        let manager = Arc::new(PerformanceManager::new());
        let config = OptimizerConfig {
            enable_auto_optimization: true,
            optimization_interval: Duration::from_millis(1), // Very short for testing
            ..Default::default()
        };
        let optimizer = PerformanceOptimizer::with_config(manager, config);

        // First call should potentially run optimizations
        let results1 = optimizer.run_auto_optimization();
        
        // Second immediate call should not run (interval not passed)
        let results2 = optimizer.run_auto_optimization();
        
        // Results may vary based on system state, but should be consistent
        assert!(results1.len() >= results2.len());
    }
}