//! Memory profiling and monitoring for code block processing
//! 
//! This module provides tools to monitor memory usage patterns,
//! detect memory leaks, and optimize memory allocation strategies.

use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock};
use std::time::{Duration, Instant};
use std::thread;

/// Memory usage snapshot at a point in time
#[derive(Debug, Clone)]
pub struct MemorySnapshot {
    /// Timestamp when snapshot was taken
    pub timestamp: Instant,
    /// Total allocated memory in bytes
    pub total_allocated: usize,
    /// Memory in use by active objects
    pub active_memory: usize,
    /// Memory held by caches
    pub cache_memory: usize,
    /// Number of active allocations
    pub allocation_count: usize,
    /// Peak memory usage since last reset
    pub peak_memory: usize,
    /// Memory fragmentation estimate (0.0 to 1.0)
    pub fragmentation_ratio: f64,
}

impl MemorySnapshot {
    /// Calculate memory utilization ratio
    pub fn utilization_ratio(&self) -> f64 {
        if self.total_allocated == 0 {
            0.0
        } else {
            self.active_memory as f64 / self.total_allocated as f64
        }
    }

    /// Check if memory usage is critical
    pub fn is_critical(&self, threshold_bytes: usize) -> bool {
        self.total_allocated > threshold_bytes
    }

    /// Get memory efficiency score (0.0 to 1.0, higher is better)
    pub fn efficiency_score(&self) -> f64 {
        let utilization = self.utilization_ratio();
        let fragmentation_penalty = self.fragmentation_ratio * 0.3;
        (utilization - fragmentation_penalty).max(0.0).min(1.0)
    }
}

/// Memory allocation tracking entry
#[derive(Debug, Clone)]
struct AllocationEntry {
    size: usize,
    allocated_at: Instant,
    category: String,
    stack_trace: Option<String>,
}

/// Memory profiler for tracking allocations and usage patterns
pub struct MemoryProfiler {
    /// Active allocations being tracked
    allocations: Arc<RwLock<HashMap<usize, AllocationEntry>>>,
    /// Memory usage history
    snapshots: Arc<Mutex<Vec<MemorySnapshot>>>,
    /// Configuration
    config: MemoryProfilerConfig,
    /// Statistics
    stats: Arc<RwLock<MemoryProfilerStats>>,
    /// Background monitoring thread handle
    monitor_handle: Option<thread::JoinHandle<()>>,
    /// Shutdown signal
    shutdown: Arc<Mutex<bool>>,
}

/// Configuration for memory profiler
#[derive(Debug, Clone)]
pub struct MemoryProfilerConfig {
    /// Whether to enable detailed allocation tracking
    pub enable_allocation_tracking: bool,
    /// Whether to capture stack traces for allocations
    pub capture_stack_traces: bool,
    /// Maximum number of snapshots to keep in history
    pub max_snapshots: usize,
    /// Interval between automatic snapshots
    pub snapshot_interval: Duration,
    /// Memory threshold for warnings (bytes)
    pub warning_threshold: usize,
    /// Memory threshold for critical alerts (bytes)
    pub critical_threshold: usize,
    /// Whether to enable background monitoring
    pub enable_background_monitoring: bool,
}

impl Default for MemoryProfilerConfig {
    fn default() -> Self {
        Self {
            enable_allocation_tracking: true,
            capture_stack_traces: false, // Expensive, disabled by default
            max_snapshots: 1000,
            snapshot_interval: Duration::from_secs(10),
            warning_threshold: 100 * 1024 * 1024, // 100MB
            critical_threshold: 500 * 1024 * 1024, // 500MB
            enable_background_monitoring: true,
        }
    }
}

/// Statistics collected by the memory profiler
#[derive(Debug, Clone, Default)]
pub struct MemoryProfilerStats {
    /// Total number of allocations tracked
    pub total_allocations: u64,
    /// Total number of deallocations tracked
    pub total_deallocations: u64,
    /// Peak memory usage observed
    pub peak_memory_usage: usize,
    /// Number of memory warnings issued
    pub warning_count: u64,
    /// Number of critical memory alerts
    pub critical_alert_count: u64,
    /// Total bytes allocated over lifetime
    pub total_bytes_allocated: u64,
    /// Total bytes deallocated over lifetime
    pub total_bytes_deallocated: u64,
    /// Number of potential memory leaks detected
    pub potential_leaks: u64,
}

impl MemoryProfilerStats {
    /// Calculate current memory leak estimate
    pub fn estimated_leaked_bytes(&self) -> u64 {
        self.total_bytes_allocated.saturating_sub(self.total_bytes_deallocated)
    }

    /// Get allocation/deallocation balance
    pub fn allocation_balance(&self) -> i64 {
        self.total_allocations as i64 - self.total_deallocations as i64
    }

    /// Check if there are signs of memory leaks
    pub fn has_potential_leaks(&self) -> bool {
        self.potential_leaks > 0 || self.allocation_balance() > 1000
    }
}

impl MemoryProfiler {
    /// Create a new memory profiler with default configuration
    pub fn new() -> Self {
        Self::with_config(MemoryProfilerConfig::default())
    }

    /// Create a memory profiler with custom configuration
    pub fn with_config(config: MemoryProfilerConfig) -> Self {
        let profiler = Self {
            allocations: Arc::new(RwLock::new(HashMap::new())),
            snapshots: Arc::new(Mutex::new(Vec::new())),
            config: config.clone(),
            stats: Arc::new(RwLock::new(MemoryProfilerStats::default())),
            monitor_handle: None,
            shutdown: Arc::new(Mutex::new(false)),
        };

        profiler
    }

    /// Start background monitoring
    pub fn start_monitoring(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if !self.config.enable_background_monitoring {
            return Ok(());
        }

        if self.monitor_handle.is_some() {
            return Err("Monitoring already started".into());
        }

        let allocations = Arc::clone(&self.allocations);
        let snapshots = Arc::clone(&self.snapshots);
        let stats = Arc::clone(&self.stats);
        let shutdown = Arc::clone(&self.shutdown);
        let config = self.config.clone();

        let handle = thread::spawn(move || {
            let mut last_snapshot = Instant::now();

            loop {
                // Check shutdown signal
                if let Ok(shutdown_flag) = shutdown.lock() {
                    if *shutdown_flag {
                        break;
                    }
                }

                // Take snapshot if interval has passed
                if last_snapshot.elapsed() >= config.snapshot_interval {
                    let snapshot = Self::create_snapshot(&allocations, &config);
                    
                    // Check thresholds and update stats
                    if let Ok(mut stats_guard) = stats.write() {
                        if snapshot.total_allocated > config.warning_threshold {
                            stats_guard.warning_count += 1;
                        }
                        if snapshot.total_allocated > config.critical_threshold {
                            stats_guard.critical_alert_count += 1;
                        }
                        stats_guard.peak_memory_usage = stats_guard.peak_memory_usage.max(snapshot.total_allocated);
                    }

                    // Store snapshot
                    if let Ok(mut snapshots_guard) = snapshots.lock() {
                        snapshots_guard.push(snapshot);
                        
                        // Limit snapshot history
                        if snapshots_guard.len() > config.max_snapshots {
                            snapshots_guard.remove(0);
                        }
                    }

                    last_snapshot = Instant::now();
                }

                // Sleep for a short interval
                thread::sleep(Duration::from_millis(100));
            }
        });

        self.monitor_handle = Some(handle);
        Ok(())
    }

    /// Stop background monitoring
    pub fn stop_monitoring(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Signal shutdown
        if let Ok(mut shutdown_flag) = self.shutdown.lock() {
            *shutdown_flag = true;
        }

        // Wait for monitor thread to finish
        if let Some(handle) = self.monitor_handle.take() {
            handle.join().map_err(|_| "Failed to join monitor thread")?;
        }

        Ok(())
    }

    /// Track a memory allocation
    pub fn track_allocation(&self, ptr: usize, size: usize, category: &str) {
        if !self.config.enable_allocation_tracking {
            return;
        }

        let entry = AllocationEntry {
            size,
            allocated_at: Instant::now(),
            category: category.to_string(),
            stack_trace: if self.config.capture_stack_traces {
                Some(self.capture_stack_trace())
            } else {
                None
            },
        };

        if let Ok(mut allocations) = self.allocations.write() {
            allocations.insert(ptr, entry);
        }

        // Update statistics
        if let Ok(mut stats) = self.stats.write() {
            stats.total_allocations += 1;
            stats.total_bytes_allocated += size as u64;
        }
    }

    /// Track a memory deallocation
    pub fn track_deallocation(&self, ptr: usize) {
        if !self.config.enable_allocation_tracking {
            return;
        }

        let deallocated_size = if let Ok(mut allocations) = self.allocations.write() {
            allocations.remove(&ptr).map(|entry| entry.size).unwrap_or(0)
        } else {
            0
        };

        // Update statistics
        if let Ok(mut stats) = self.stats.write() {
            stats.total_deallocations += 1;
            stats.total_bytes_deallocated += deallocated_size as u64;
        }
    }

    /// Take a memory snapshot
    pub fn take_snapshot(&self) -> MemorySnapshot {
        Self::create_snapshot(&self.allocations, &self.config)
    }

    /// Create a memory snapshot from current state
    fn create_snapshot(
        allocations: &Arc<RwLock<HashMap<usize, AllocationEntry>>>,
        _config: &MemoryProfilerConfig,
    ) -> MemorySnapshot {
        let (total_allocated, allocation_count, cache_memory) = if let Ok(allocations_guard) = allocations.read() {
            let total = allocations_guard.values().map(|entry| entry.size).sum();
            let count = allocations_guard.len();
            let cache = allocations_guard
                .values()
                .filter(|entry| entry.category.contains("cache"))
                .map(|entry| entry.size)
                .sum();
            (total, count, cache)
        } else {
            (0, 0, 0)
        };

        // Estimate fragmentation (simplified)
        let fragmentation_ratio = if allocation_count > 0 {
            // Higher allocation count relative to total memory suggests more fragmentation
            (allocation_count as f64 / (total_allocated / 1024).max(1) as f64).min(1.0)
        } else {
            0.0
        };

        MemorySnapshot {
            timestamp: Instant::now(),
            total_allocated,
            active_memory: total_allocated, // Simplified - assume all tracked memory is active
            cache_memory,
            allocation_count,
            peak_memory: total_allocated, // Will be updated by monitoring thread
            fragmentation_ratio,
        }
    }

    /// Get memory usage history
    pub fn get_snapshot_history(&self) -> Vec<MemorySnapshot> {
        if let Ok(snapshots) = self.snapshots.lock() {
            snapshots.clone()
        } else {
            Vec::new()
        }
    }

    /// Get current statistics
    pub fn get_statistics(&self) -> MemoryProfilerStats {
        if let Ok(stats) = self.stats.read() {
            stats.clone()
        } else {
            MemoryProfilerStats::default()
        }
    }

    /// Analyze memory usage patterns
    pub fn analyze_patterns(&self) -> MemoryAnalysis {
        let snapshots = self.get_snapshot_history();
        let stats = self.get_statistics();

        if snapshots.is_empty() {
            return MemoryAnalysis::default();
        }

        // Calculate trends
        let memory_trend = if snapshots.len() >= 2 {
            let first = &snapshots[0];
            let last = &snapshots[snapshots.len() - 1];
            let time_diff = last.timestamp.duration_since(first.timestamp).as_secs_f64();
            
            if time_diff > 0.0 {
                (last.total_allocated as f64 - first.total_allocated as f64) / time_diff
            } else {
                0.0
            }
        } else {
            0.0
        };

        // Calculate average memory usage
        let avg_memory = snapshots.iter().map(|s| s.total_allocated).sum::<usize>() / snapshots.len();

        // Find peak usage
        let peak_memory = snapshots.iter().map(|s| s.total_allocated).max().unwrap_or(0);

        // Calculate efficiency
        let avg_efficiency = snapshots.iter().map(|s| s.efficiency_score()).sum::<f64>() / snapshots.len() as f64;

        // Detect potential issues
        let mut issues = Vec::new();
        
        if memory_trend > 1024.0 * 1024.0 { // Growing by more than 1MB/sec
            issues.push("Memory usage is growing rapidly".to_string());
        }
        
        if stats.has_potential_leaks() {
            issues.push("Potential memory leaks detected".to_string());
        }
        
        if avg_efficiency < 0.5 {
            issues.push("Low memory efficiency detected".to_string());
        }

        MemoryAnalysis {
            memory_trend_bytes_per_sec: memory_trend,
            average_memory_usage: avg_memory,
            peak_memory_usage: peak_memory,
            average_efficiency: avg_efficiency,
            potential_issues: issues,
            leak_estimate_bytes: stats.estimated_leaked_bytes(),
            fragmentation_level: snapshots.last().map(|s| s.fragmentation_ratio).unwrap_or(0.0),
        }
    }

    /// Find potential memory leaks
    pub fn find_potential_leaks(&self, age_threshold: Duration) -> Vec<PotentialLeak> {
        let mut leaks = Vec::new();
        let now = Instant::now();

        if let Ok(allocations) = self.allocations.read() {
            for (ptr, entry) in allocations.iter() {
                if now.duration_since(entry.allocated_at) > age_threshold {
                    leaks.push(PotentialLeak {
                        ptr: *ptr,
                        size: entry.size,
                        age: now.duration_since(entry.allocated_at),
                        category: entry.category.clone(),
                        stack_trace: entry.stack_trace.clone(),
                    });
                }
            }
        }

        // Sort by size (largest first)
        leaks.sort_by(|a, b| b.size.cmp(&a.size));
        leaks
    }

    /// Generate a memory report
    pub fn generate_report(&self) -> MemoryReport {
        let snapshot = self.take_snapshot();
        let stats = self.get_statistics();
        let analysis = self.analyze_patterns();
        let potential_leaks = self.find_potential_leaks(Duration::from_secs(300)); // 5 minutes

        let recommendations = self.generate_recommendations(&analysis);
        MemoryReport {
            timestamp: Instant::now(),
            current_snapshot: snapshot,
            statistics: stats,
            analysis,
            potential_leaks,
            recommendations,
        }
    }

    /// Generate optimization recommendations
    fn generate_recommendations(&self, analysis: &MemoryAnalysis) -> Vec<String> {
        let mut recommendations = Vec::new();

        if analysis.memory_trend_bytes_per_sec > 1024.0 * 1024.0 {
            recommendations.push("Consider implementing more aggressive garbage collection".to_string());
        }

        if analysis.average_efficiency < 0.6 {
            recommendations.push("Memory efficiency is low - consider optimizing data structures".to_string());
        }

        if analysis.leak_estimate_bytes > 10 * 1024 * 1024 {
            recommendations.push("Significant memory leaks detected - review allocation patterns".to_string());
        }

        if analysis.fragmentation_level > 0.7 {
            recommendations.push("High memory fragmentation - consider memory pooling".to_string());
        }

        if recommendations.is_empty() {
            recommendations.push("Memory usage appears optimal".to_string());
        }

        recommendations
    }

    /// Capture stack trace (simplified implementation)
    fn capture_stack_trace(&self) -> String {
        // In a real implementation, this would capture actual stack traces
        // For now, return a placeholder
        format!("Stack trace captured at {}", Instant::now().elapsed().as_millis())
    }

    /// Reset all statistics and history
    pub fn reset(&self) {
        if let Ok(mut allocations) = self.allocations.write() {
            allocations.clear();
        }

        if let Ok(mut snapshots) = self.snapshots.lock() {
            snapshots.clear();
        }

        if let Ok(mut stats) = self.stats.write() {
            *stats = MemoryProfilerStats::default();
        }
    }
}

impl Drop for MemoryProfiler {
    fn drop(&mut self) {
        let _ = self.stop_monitoring();
    }
}

/// Analysis of memory usage patterns
#[derive(Debug, Clone, Default)]
pub struct MemoryAnalysis {
    /// Memory growth trend in bytes per second
    pub memory_trend_bytes_per_sec: f64,
    /// Average memory usage over time
    pub average_memory_usage: usize,
    /// Peak memory usage observed
    pub peak_memory_usage: usize,
    /// Average memory efficiency score
    pub average_efficiency: f64,
    /// List of potential issues detected
    pub potential_issues: Vec<String>,
    /// Estimated leaked memory in bytes
    pub leak_estimate_bytes: u64,
    /// Memory fragmentation level (0.0 to 1.0)
    pub fragmentation_level: f64,
}

/// Information about a potential memory leak
#[derive(Debug, Clone)]
pub struct PotentialLeak {
    /// Pointer address
    pub ptr: usize,
    /// Size of the allocation
    pub size: usize,
    /// How long the allocation has been active
    pub age: Duration,
    /// Category of the allocation
    pub category: String,
    /// Stack trace if available
    pub stack_trace: Option<String>,
}

/// Comprehensive memory report
#[derive(Debug, Clone)]
pub struct MemoryReport {
    /// When the report was generated
    pub timestamp: Instant,
    /// Current memory snapshot
    pub current_snapshot: MemorySnapshot,
    /// Profiler statistics
    pub statistics: MemoryProfilerStats,
    /// Memory usage analysis
    pub analysis: MemoryAnalysis,
    /// Potential memory leaks found
    pub potential_leaks: Vec<PotentialLeak>,
    /// Optimization recommendations
    pub recommendations: Vec<String>,
}

impl MemoryReport {
    /// Generate a human-readable summary
    pub fn summary(&self) -> String {
        format!(
            "Memory Report ({})\n\
             Current Usage: {:.1} MB ({} allocations)\n\
             Peak Usage: {:.1} MB\n\
             Efficiency: {:.1}%\n\
             Trend: {:.1} KB/sec\n\
             Potential Leaks: {} ({:.1} MB)\n\
             Issues: {}\n\
             Recommendations: {}",
            self.timestamp.elapsed().as_secs(),
            self.current_snapshot.total_allocated as f64 / (1024.0 * 1024.0),
            self.current_snapshot.allocation_count,
            self.analysis.peak_memory_usage as f64 / (1024.0 * 1024.0),
            self.analysis.average_efficiency * 100.0,
            self.analysis.memory_trend_bytes_per_sec / 1024.0,
            self.potential_leaks.len(),
            self.analysis.leak_estimate_bytes as f64 / (1024.0 * 1024.0),
            self.analysis.potential_issues.len(),
            self.recommendations.len()
        )
    }

    /// Check if the memory usage is healthy
    pub fn is_healthy(&self) -> bool {
        self.analysis.potential_issues.is_empty() 
            && self.potential_leaks.len() < 10
            && self.analysis.average_efficiency > 0.5
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_profiler_creation() {
        let profiler = MemoryProfiler::new();
        let snapshot = profiler.take_snapshot();
        
        assert_eq!(snapshot.total_allocated, 0);
        assert_eq!(snapshot.allocation_count, 0);
    }

    #[test]
    fn test_allocation_tracking() {
        let profiler = MemoryProfiler::new();
        
        // Track some allocations
        profiler.track_allocation(0x1000, 1024, "test");
        profiler.track_allocation(0x2000, 2048, "cache");
        
        let snapshot = profiler.take_snapshot();
        assert_eq!(snapshot.total_allocated, 3072);
        assert_eq!(snapshot.allocation_count, 2);
        assert_eq!(snapshot.cache_memory, 2048);
        
        // Track deallocation
        profiler.track_deallocation(0x1000);
        
        let snapshot2 = profiler.take_snapshot();
        assert_eq!(snapshot2.total_allocated, 2048);
        assert_eq!(snapshot2.allocation_count, 1);
    }

    #[test]
    fn test_statistics() {
        let profiler = MemoryProfiler::new();
        
        profiler.track_allocation(0x1000, 1024, "test");
        profiler.track_allocation(0x2000, 2048, "test");
        profiler.track_deallocation(0x1000);
        
        let stats = profiler.get_statistics();
        assert_eq!(stats.total_allocations, 2);
        assert_eq!(stats.total_deallocations, 1);
        assert_eq!(stats.total_bytes_allocated, 3072);
        assert_eq!(stats.total_bytes_deallocated, 1024);
        assert_eq!(stats.estimated_leaked_bytes(), 2048);
    }

    #[test]
    fn test_potential_leak_detection() {
        let profiler = MemoryProfiler::new();
        
        // Simulate old allocation
        profiler.track_allocation(0x1000, 1024, "old_allocation");
        
        // Wait a bit (in real test, we'd mock time)
        std::thread::sleep(Duration::from_millis(10));
        
        let leaks = profiler.find_potential_leaks(Duration::from_millis(5));
        assert_eq!(leaks.len(), 1);
        assert_eq!(leaks[0].size, 1024);
        assert_eq!(leaks[0].category, "old_allocation");
    }

    #[test]
    fn test_memory_analysis() {
        let mut profiler = MemoryProfiler::new();
        
        // Simulate some memory usage pattern
        for i in 0..10 {
            profiler.track_allocation(0x1000 + i, 1024 * (i + 1), "test");
            
            // Take snapshot
            let snapshot = profiler.take_snapshot();
            if let Ok(mut snapshots) = profiler.snapshots.lock() {
                snapshots.push(snapshot);
            }
        }
        
        let analysis = profiler.analyze_patterns();
        assert!(analysis.average_memory_usage > 0);
        assert!(analysis.peak_memory_usage > 0);
    }

    #[test]
    fn test_memory_report() {
        let profiler = MemoryProfiler::new();
        
        profiler.track_allocation(0x1000, 1024, "test");
        profiler.track_allocation(0x2000, 2048, "cache");
        
        let report = profiler.generate_report();
        
        assert_eq!(report.current_snapshot.total_allocated, 3072);
        assert!(!report.recommendations.is_empty());
        
        let summary = report.summary();
        assert!(!summary.is_empty());
        assert!(summary.contains("Memory Report"));
    }

    #[test]
    fn test_memory_snapshot_calculations() {
        let snapshot = MemorySnapshot {
            timestamp: Instant::now(),
            total_allocated: 1024 * 1024, // 1MB
            active_memory: 512 * 1024,    // 512KB
            cache_memory: 256 * 1024,     // 256KB
            allocation_count: 100,
            peak_memory: 2 * 1024 * 1024, // 2MB
            fragmentation_ratio: 0.3,
        };
        
        assert_eq!(snapshot.utilization_ratio(), 0.5);
        assert!(!snapshot.is_critical(2 * 1024 * 1024));
        assert!(snapshot.is_critical(512 * 1024));
        
        let efficiency = snapshot.efficiency_score();
        assert!(efficiency > 0.0 && efficiency <= 1.0);
    }

    #[test]
    fn test_profiler_config() {
        let config = MemoryProfilerConfig {
            enable_allocation_tracking: false,
            ..Default::default()
        };
        
        let profiler = MemoryProfiler::with_config(config);
        
        // Should not track when disabled
        profiler.track_allocation(0x1000, 1024, "test");
        let snapshot = profiler.take_snapshot();
        assert_eq!(snapshot.allocation_count, 0);
    }
}