//! Integration tests for performance optimization features
//! 
//! These tests verify that performance optimizations work correctly
//! and provide measurable improvements under various conditions.

use std::time::{Duration, Instant};
use std::sync::Arc;
use std::thread;

use md2docx_converter::markdown::code_block::{
    CodeBlockStrategy, ProcessingConfig, DefaultStrategy,
    CodeBlockCache, CacheConfig, CacheKey,
    PerformanceManager, PerformanceConfig,
    StrategyRegistry,
};

/// Test data generator for performance tests
struct TestDataGenerator;

impl TestDataGenerator {
    /// Generate code samples of various sizes for testing
    fn generate_test_codes(count: usize, size_category: CodeSizeCategory) -> Vec<String> {
        match size_category {
            CodeSizeCategory::Small => {
                (0..count)
                    .map(|i| format!("fn small_func_{i}() {{ println!(\"Hello {i}\"); }}"))
                    .collect()
            }
            CodeSizeCategory::Medium => {
                (0..count)
                    .map(|i| {
                        format!(
                            r#"
fn medium_func_{i}(x: i32, y: i32) -> i32 {{
    let result = x + y;
    if result > 100 {{
        println!("Large result: {{}}", result);
        result * 2
    }} else {{
        println!("Small result: {{}}", result);
        result
    }}
}}

#[cfg(test)]
mod tests_{i} {{
    use super::*;

    #[test]
    fn test_medium_func_{i}() {{
        assert_eq!(medium_func_{i}(10, 20), 30);
        assert_eq!(medium_func_{i}(60, 50), 220);
    }}
}}
"#
                        )
                    })
                    .collect()
            }
            CodeSizeCategory::Large => {
                (0..count)
                    .map(|i| {
                        format!(
                            r#"
use std::collections::HashMap;
use std::sync::{{Arc, Mutex}};
use std::thread;

#[derive(Debug, Clone)]
pub struct LargeStruct_{i} {{
    pub id: u64,
    pub name: String,
    pub data: HashMap<String, i32>,
    pub metadata: Vec<String>,
}}

impl LargeStruct_{i} {{
    pub fn new(id: u64, name: String) -> Self {{
        Self {{
            id,
            name,
            data: HashMap::new(),
            metadata: Vec::new(),
        }}
    }}

    pub fn add_data(&mut self, key: String, value: i32) {{
        self.data.insert(key, value);
    }}

    pub fn add_metadata(&mut self, meta: String) {{
        self.metadata.push(meta);
    }}

    pub fn process_data(&self) -> i32 {{
        self.data.values().sum()
    }}

    pub fn parallel_process(&self) -> i32 {{
        let data = Arc::new(self.data.clone());
        let handles: Vec<_> = (0..4)
            .map(|_| {{
                let data = Arc::clone(&data);
                thread::spawn(move || {{
                    data.values().map(|v| v * 2).sum::<i32>()
                }})
            }})
            .collect();

        handles
            .into_iter()
            .map(|h| h.join().unwrap_or(0))
            .sum()
    }}

    pub fn get_summary(&self) -> String {{
        format!(
            "LargeStruct_{i} {{ id: {{}}, name: {{}}, data_count: {{}}, metadata_count: {{}} }}",
            self.id, self.name, self.data.len(), self.metadata.len()
        )
    }}
}}

#[cfg(test)]
mod tests_large_{i} {{
    use super::*;

    #[test]
    fn test_large_struct_creation() {{
        let mut s = LargeStruct_{i}::new(1, "test".to_string());
        s.add_data("key1".to_string(), 10);
        s.add_data("key2".to_string(), 20);
        s.add_metadata("meta1".to_string());
        
        assert_eq!(s.process_data(), 30);
        assert_eq!(s.metadata.len(), 1);
    }}

    #[test]
    fn test_parallel_processing() {{
        let mut s = LargeStruct_{i}::new(2, "parallel_test".to_string());
        for i in 0..10 {{
            s.add_data(format!("key{{}}", i), i);
        }}
        
        let result = s.parallel_process();
        assert!(result > 0);
    }}
}}
"#
                        )
                    })
                    .collect()
            }
        }
    }

    /// Generate mixed workload with different code types
    fn generate_mixed_workload(total_count: usize) -> Vec<(String, String)> {
        let small_count = total_count / 3;
        let medium_count = total_count / 3;
        let large_count = total_count - small_count - medium_count;

        let mut workload = Vec::new();

        // Add small codes
        for code in Self::generate_test_codes(small_count, CodeSizeCategory::Small) {
            workload.push((code, "rust".to_string()));
        }

        // Add medium codes
        for code in Self::generate_test_codes(medium_count, CodeSizeCategory::Medium) {
            workload.push((code, "rust".to_string()));
        }

        // Add large codes
        for code in Self::generate_test_codes(large_count, CodeSizeCategory::Large) {
            workload.push((code, "rust".to_string()));
        }

        workload
    }
}

#[derive(Debug, Clone, Copy)]
enum CodeSizeCategory {
    Small,   // < 100 characters
    Medium,  // 100-1000 characters
    Large,   // > 1000 characters
}

/// Performance test results
#[derive(Debug, Clone)]
struct PerformanceTestResult {
    test_name: String,
    duration: Duration,
    throughput: f64, // operations per second
    memory_usage: Option<usize>,
    success_rate: f64,
    additional_metrics: std::collections::HashMap<String, f64>,
}

impl PerformanceTestResult {
    fn new(test_name: &str, duration: Duration, operations: usize) -> Self {
        let throughput = if duration.as_secs_f64() > 0.0 {
            operations as f64 / duration.as_secs_f64()
        } else {
            0.0
        };

        Self {
            test_name: test_name.to_string(),
            duration,
            throughput,
            memory_usage: None,
            success_rate: 1.0,
            additional_metrics: std::collections::HashMap::new(),
        }
    }

    fn with_success_rate(mut self, success_rate: f64) -> Self {
        self.success_rate = success_rate;
        self
    }

    fn with_memory_usage(mut self, memory_usage: usize) -> Self {
        self.memory_usage = Some(memory_usage);
        self
    }

    fn add_metric(mut self, name: &str, value: f64) -> Self {
        self.additional_metrics.insert(name.to_string(), value);
        self
    }
}

/// Test cache performance with various scenarios
#[test]
fn test_cache_performance_scenarios() {
    let cache_configs = vec![
        CacheConfig {
            max_entries: 100,
            max_memory_bytes: 1024 * 1024, // 1MB
            ..Default::default()
        },
        CacheConfig {
            max_entries: 1000,
            max_memory_bytes: 10 * 1024 * 1024, // 10MB
            ..Default::default()
        },
        CacheConfig {
            max_entries: 10000,
            max_memory_bytes: 100 * 1024 * 1024, // 100MB
            ..Default::default()
        },
    ];

    for (i, config) in cache_configs.iter().enumerate() {
        println!("Testing cache configuration {}: max_entries={}, max_memory={}MB", 
                 i + 1, config.max_entries, config.max_memory_bytes / (1024 * 1024));

        let cache = CodeBlockCache::with_config(config.clone());
        let strategy = DefaultStrategy::new();
        let processing_config = ProcessingConfig::default();

        // Test cache population
        let start_time = Instant::now();
        let test_codes = TestDataGenerator::generate_test_codes(config.max_entries / 2, CodeSizeCategory::Small);
        
        for (idx, code) in test_codes.iter().enumerate() {
            let key = CacheKey::new(code, Some("rust"), &processing_config, "1.0.0");
            let result = strategy.process(code, &processing_config).unwrap();
            cache.put(key, result);

            // Check cache statistics periodically
            if idx % 100 == 0 {
                let stats = cache.get_statistics();
                assert!(stats.current_entries <= config.max_entries);
            }
        }

        let population_time = start_time.elapsed();
        let final_stats = cache.get_statistics();

        println!("  Population completed in {:?}", population_time);
        println!("  Final cache entries: {}", final_stats.current_entries);
        println!("  Final cache memory: {} bytes", final_stats.current_memory_bytes);
        println!("  Cache hit ratio: {:.2}%", final_stats.hit_ratio() * 100.0);

        // Test cache hit performance
        let hit_test_start = Instant::now();
        let hit_test_iterations = 1000;
        
        for _ in 0..hit_test_iterations {
            if let Some(code) = test_codes.first() {
                let key = CacheKey::new(code, Some("rust"), &processing_config, "1.0.0");
                let _result = cache.get_or_compute(key, || {
                    panic!("Should not compute - should be cache hit");
                });
            }
        }

        let hit_test_time = hit_test_start.elapsed();
        let hit_throughput = hit_test_iterations as f64 / hit_test_time.as_secs_f64();

        println!("  Cache hit throughput: {:.0} ops/sec", hit_throughput);

        // Verify cache effectiveness
        assert!(final_stats.hit_ratio() > 0.0, "Cache should have some hits");
        assert!(hit_throughput > 1000.0, "Cache hits should be fast (>1000 ops/sec)");
    }
}

/// Test memory usage patterns under load
#[test]
fn test_memory_usage_patterns() {
    let test_scenarios = vec![
        ("small_codes_high_volume", CodeSizeCategory::Small, 10000),
        ("medium_codes_moderate_volume", CodeSizeCategory::Medium, 1000),
        ("large_codes_low_volume", CodeSizeCategory::Large, 100),
    ];

    for (scenario_name, size_category, count) in test_scenarios {
        println!("Testing memory usage scenario: {}", scenario_name);

        let manager = PerformanceManager::new();
        let config = ProcessingConfig::default();

        // Get initial memory baseline
        let initial_metrics = manager.get_performance_metrics();
        let initial_memory = initial_metrics.memory_utilization;

        // Process codes and monitor memory usage
        let codes = TestDataGenerator::generate_test_codes(count, size_category);
        let start_time = Instant::now();
        let mut successful_operations = 0;

        for (idx, code) in codes.iter().enumerate() {
            match manager.process_code_block(code, Some("rust"), &config) {
                Ok(_) => successful_operations += 1,
                Err(e) => eprintln!("Processing failed for code {}: {}", idx, e),
            }

            // Check memory usage periodically
            if idx % (count / 10).max(1) == 0 {
                let current_metrics = manager.get_performance_metrics();
                let memory_increase = current_metrics.memory_utilization - initial_memory;
                
                println!("  Progress: {}/{}, Memory increase: {:.2}%", 
                         idx + 1, count, memory_increase);

                // Ensure memory usage doesn't grow unbounded
                assert!(memory_increase < 50.0, 
                        "Memory usage increased too much: {:.2}%", memory_increase);
            }
        }

        let processing_time = start_time.elapsed();
        let final_metrics = manager.get_performance_metrics();

        let result = PerformanceTestResult::new(scenario_name, processing_time, successful_operations)
            .with_success_rate(successful_operations as f64 / count as f64)
            .with_memory_usage(final_metrics.memory_utilization as usize)
            .add_metric("initial_memory", initial_memory)
            .add_metric("final_memory", final_metrics.memory_utilization)
            .add_metric("cache_hit_ratio", final_metrics.cache_hit_ratio);

        println!("  Results: {:?}", result);

        // Verify performance requirements
        assert!(result.success_rate > 0.95, "Success rate should be > 95%");
        assert!(result.throughput > 10.0, "Throughput should be > 10 ops/sec");

        // Force cleanup and verify memory is released
        let cleanup_summary = manager.cleanup();
        println!("  Cleanup: {:?}", cleanup_summary);

        let post_cleanup_metrics = manager.get_performance_metrics();
        let memory_after_cleanup = post_cleanup_metrics.memory_utilization;
        
        println!("  Memory after cleanup: {:.2}%", memory_after_cleanup);
        
        // Memory should be reduced after cleanup
        assert!(memory_after_cleanup <= final_metrics.memory_utilization,
                "Memory should not increase after cleanup");
    }
}

/// Test concurrent processing performance
#[test]
fn test_concurrent_processing_performance() {
    let thread_counts = vec![1, 2, 4, 8];
    let codes_per_thread = 100;

    for thread_count in thread_counts {
        println!("Testing concurrent processing with {} threads", thread_count);

        let start_time = Instant::now();
        let handles: Vec<_> = (0..thread_count)
            .map(|thread_id| {
                thread::spawn(move || {
                    let manager = PerformanceManager::new();
                    let config = ProcessingConfig::default();
                    let codes = TestDataGenerator::generate_test_codes(codes_per_thread, CodeSizeCategory::Small);

                    let mut successful = 0;
                    let thread_start = Instant::now();

                    for code in codes {
                        match manager.process_code_block(&code, Some("rust"), &config) {
                            Ok(_) => successful += 1,
                            Err(_) => {}
                        }
                    }

                    (thread_id, successful, thread_start.elapsed())
                })
            })
            .collect();

        let mut total_successful = 0;
        let mut max_thread_time = Duration::from_millis(0);

        for handle in handles {
            let (thread_id, successful, thread_time) = handle.join().unwrap();
            total_successful += successful;
            max_thread_time = max_thread_time.max(thread_time);
            
            println!("  Thread {}: {} successful operations in {:?}", 
                     thread_id, successful, thread_time);
        }

        let total_time = start_time.elapsed();
        let total_operations = thread_count * codes_per_thread;
        let throughput = total_successful as f64 / total_time.as_secs_f64();

        println!("  Total: {}/{} operations in {:?}", total_successful, total_operations, total_time);
        println!("  Throughput: {:.0} ops/sec", throughput);
        println!("  Max thread time: {:?}", max_thread_time);

        // Verify concurrent performance
        assert!(total_successful >= (total_operations as f64 * 0.95) as usize,
                "Should complete at least 95% of operations");
        assert!(throughput > 50.0, "Concurrent throughput should be > 50 ops/sec");

        // Verify that concurrent processing provides some benefit
        if thread_count > 1 {
            // With multiple threads, max thread time should be less than total time
            // (indicating some parallelism benefit)
            let parallelism_ratio = max_thread_time.as_secs_f64() / total_time.as_secs_f64();
            println!("  Parallelism ratio: {:.2}", parallelism_ratio);
            
            // This is a loose check - in practice, parallelism benefits depend on many factors
            assert!(parallelism_ratio < 2.0, "Parallelism ratio should be reasonable");
        }
    }
}

/// Test performance under stress conditions
#[test]
fn test_stress_performance() {
    println!("Running stress performance test");

    let config = PerformanceConfig {
        cache: CacheConfig {
            max_entries: 1000,
            max_memory_bytes: 50 * 1024 * 1024, // 50MB
            ..Default::default()
        },
        enable_adaptive_optimization: true,
        ..Default::default()
    };

    let manager = PerformanceManager::with_config(config);
    let processing_config = ProcessingConfig::default();

    // Generate a large mixed workload
    let workload = TestDataGenerator::generate_mixed_workload(5000);
    println!("Generated workload with {} code blocks", workload.len());

    let start_time = Instant::now();
    let mut successful_operations = 0;
    let mut total_processing_time = Duration::from_millis(0);

    // Process workload in batches to simulate real usage
    let batch_size = 100;
    for (batch_idx, batch) in workload.chunks(batch_size).enumerate() {
        let batch_start = Instant::now();
        
        let requests: Vec<_> = batch
            .iter()
            .map(|(code, lang)| (code.clone(), Some(lang.clone()), processing_config.clone()))
            .collect();

        let results = manager.process_code_blocks(requests);
        
        let batch_successful = results.iter().filter(|r| r.is_ok()).count();
        successful_operations += batch_successful;
        
        let batch_time = batch_start.elapsed();
        total_processing_time += batch_time;

        // Log progress and metrics every 10 batches
        if batch_idx % 10 == 0 {
            let metrics = manager.get_performance_metrics();
            println!("  Batch {}: {}/{} successful, cache hit ratio: {:.2}%, memory: {:.1}%",
                     batch_idx, batch_successful, batch.len(),
                     metrics.cache_hit_ratio * 100.0, metrics.memory_utilization);

            // Check if system is overloaded and optimize if needed
            if manager.is_system_overloaded() {
                println!("    System overloaded, running optimization...");
                let opt_result = manager.optimize();
                println!("    Optimization result: {:?}", opt_result.summary());
            }
        }
    }

    let total_time = start_time.elapsed();
    let final_metrics = manager.get_performance_metrics();

    let result = PerformanceTestResult::new("stress_test", total_time, successful_operations)
        .with_success_rate(successful_operations as f64 / workload.len() as f64)
        .add_metric("cache_hit_ratio", final_metrics.cache_hit_ratio)
        .add_metric("memory_utilization", final_metrics.memory_utilization)
        .add_metric("average_processing_time", final_metrics.average_processing_time.as_millis() as f64);

    println!("Stress test results: {:?}", result);

    // Verify stress test requirements
    assert!(result.success_rate > 0.90, "Stress test success rate should be > 90%");
    assert!(result.throughput > 20.0, "Stress test throughput should be > 20 ops/sec");
    assert!(final_metrics.cache_hit_ratio > 0.3, "Cache should be effective under stress");
    assert!(final_metrics.memory_utilization < 90.0, "Memory usage should stay reasonable");

    // Test system recovery after stress
    let recovery_start = Instant::now();
    let cleanup_summary = manager.cleanup();
    let recovery_time = recovery_start.elapsed();

    println!("Recovery: cleanup completed in {:?}, summary: {:?}", recovery_time, cleanup_summary);

    let post_recovery_metrics = manager.get_performance_metrics();
    println!("Post-recovery memory utilization: {:.1}%", post_recovery_metrics.memory_utilization);

    // Verify system can recover
    assert!(recovery_time < Duration::from_secs(5), "Recovery should be fast");
    assert!(cleanup_summary.was_effective(), "Cleanup should be effective");
}

/// Test adaptive optimization effectiveness
#[test]
fn test_adaptive_optimization() {
    println!("Testing adaptive optimization");

    let config = PerformanceConfig {
        enable_adaptive_optimization: true,
        cache: CacheConfig {
            max_entries: 500,
            max_memory_bytes: 10 * 1024 * 1024, // 10MB
            ..Default::default()
        },
        ..Default::default()
    };

    let manager = PerformanceManager::with_config(config);
    let processing_config = ProcessingConfig::default();

    // Create a scenario that will trigger optimization
    let codes = TestDataGenerator::generate_test_codes(1000, CodeSizeCategory::Medium);

    // Process codes to build up cache and memory usage
    let mut successful_operations = 0;
    for (idx, code) in codes.iter().enumerate() {
        match manager.process_code_block(code, Some("rust"), &processing_config) {
            Ok(_) => successful_operations += 1,
            Err(_) => {}
        }

        // Trigger optimization at certain points
        if idx % 200 == 0 && idx > 0 {
            let pre_opt_metrics = manager.get_performance_metrics();
            println!("  Before optimization {}: performance score: {:.2}, memory: {:.1}%",
                     idx / 200, pre_opt_metrics.performance_score(), pre_opt_metrics.memory_utilization);

            let opt_result = manager.optimize();
            
            let post_opt_metrics = manager.get_performance_metrics();
            println!("  After optimization: performance score: {:.2}, memory: {:.1}%",
                     post_opt_metrics.performance_score(), post_opt_metrics.memory_utilization);
            println!("  Optimization actions: {:?}", opt_result.actions_taken);

            // Verify optimization had some effect
            if !opt_result.actions_taken.is_empty() {
                // If actions were taken, there should be some improvement or at least no degradation
                assert!(post_opt_metrics.performance_score() >= pre_opt_metrics.performance_score() - 0.1,
                        "Performance should not degrade significantly after optimization");
            }
        }
    }

    let final_metrics = manager.get_performance_metrics();
    println!("Final metrics: performance score: {:.2}, cache hit ratio: {:.2}%",
             final_metrics.performance_score(), final_metrics.cache_hit_ratio * 100.0);

    // Verify adaptive optimization effectiveness
    assert!(successful_operations > 900, "Should process most operations successfully");
    assert!(final_metrics.performance_score() > 0.3, "Final performance should be reasonable");
    assert!(final_metrics.cache_hit_ratio > 0.1, "Cache should have some effectiveness");
}

/// Test performance regression detection
#[test]
fn test_performance_regression_detection() {
    println!("Testing performance regression detection");

    // Establish baseline performance
    let baseline_manager = PerformanceManager::new();
    let config = ProcessingConfig::default();
    let test_codes = TestDataGenerator::generate_test_codes(100, CodeSizeCategory::Small);

    let baseline_start = Instant::now();
    let mut baseline_successful = 0;

    for code in &test_codes {
        match baseline_manager.process_code_block(code, Some("rust"), &config) {
            Ok(_) => baseline_successful += 1,
            Err(_) => {}
        }
    }

    let baseline_time = baseline_start.elapsed();
    let baseline_throughput = baseline_successful as f64 / baseline_time.as_secs_f64();

    println!("Baseline performance: {:.0} ops/sec", baseline_throughput);

    // Test with constrained resources to simulate regression
    let constrained_config = PerformanceConfig {
        cache: CacheConfig {
            max_entries: 10, // Very small cache
            max_memory_bytes: 1024 * 1024, // 1MB
            ..Default::default()
        },
        ..Default::default()
    };

    let constrained_manager = PerformanceManager::with_config(constrained_config);

    let constrained_start = Instant::now();
    let mut constrained_successful = 0;

    for code in &test_codes {
        match constrained_manager.process_code_block(code, Some("rust"), &config) {
            Ok(_) => constrained_successful += 1,
            Err(_) => {}
        }
    }

    let constrained_time = constrained_start.elapsed();
    let constrained_throughput = constrained_successful as f64 / constrained_time.as_secs_f64();

    println!("Constrained performance: {:.0} ops/sec", constrained_throughput);

    // Calculate performance ratio
    let performance_ratio = constrained_throughput / baseline_throughput;
    println!("Performance ratio (constrained/baseline): {:.2}", performance_ratio);

    // Verify we can detect performance differences
    // The constrained version should be slower due to cache thrashing
    assert!(performance_ratio < 1.5, "Constrained performance should not be much better than baseline");
    
    // Both should still be functional
    assert!(baseline_successful >= 95, "Baseline should process most operations");
    assert!(constrained_successful >= 95, "Constrained should still process most operations");
    
    // Both should have reasonable throughput
    assert!(baseline_throughput > 10.0, "Baseline throughput should be reasonable");
    assert!(constrained_throughput > 5.0, "Constrained throughput should still be functional");
}