//! Performance test runner for code block processing system
//! 
//! This binary provides a comprehensive performance testing suite
//! that can be used to benchmark and validate the performance
//! characteristics of the code block processing system.

use std::collections::HashMap;
use std::time::{Duration, Instant};
use std::sync::Arc;
use std::thread;

use md2docx_converter::markdown::code_block::{
    PerformanceManager, PerformanceConfig,
    CodeBlockCache, CacheConfig,
    MemoryProfiler, MemoryProfilerConfig,
    PerformanceOptimizer, OptimizerConfig,
    ProcessingConfig, DefaultStrategy,
};

/// Test configuration
#[derive(Debug, Clone)]
struct TestConfig {
    /// Number of code blocks to process
    pub code_block_count: usize,
    /// Size category of code blocks
    pub code_size: CodeSize,
    /// Number of concurrent threads
    pub thread_count: usize,
    /// Duration to run the test
    pub test_duration: Duration,
    /// Whether to enable caching
    pub enable_caching: bool,
    /// Whether to enable memory profiling
    pub enable_memory_profiling: bool,
    /// Whether to enable optimization
    pub enable_optimization: bool,
}

impl Default for TestConfig {
    fn default() -> Self {
        Self {
            code_block_count: 1000,
            code_size: CodeSize::Medium,
            thread_count: 4,
            test_duration: Duration::from_secs(60),
            enable_caching: true,
            enable_memory_profiling: true,
            enable_optimization: true,
        }
    }
}

/// Code size categories for testing
#[derive(Debug, Clone, Copy)]
enum CodeSize {
    Small,   // ~100 bytes
    Medium,  // ~1KB
    Large,   // ~10KB
    Huge,    // ~100KB
}

/// Test results
#[derive(Debug, Clone)]
struct TestResults {
    pub test_name: String,
    pub config: TestConfig,
    pub duration: Duration,
    pub total_operations: usize,
    pub successful_operations: usize,
    pub throughput: f64, // operations per second
    pub average_latency: Duration,
    pub p95_latency: Duration,
    pub p99_latency: Duration,
    pub memory_usage: MemoryUsageStats,
    pub cache_stats: CacheStats,
    pub error_count: usize,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone, Default)]
struct MemoryUsageStats {
    pub initial_memory: usize,
    pub peak_memory: usize,
    pub final_memory: usize,
    pub memory_efficiency: f64,
}

#[derive(Debug, Clone, Default)]
struct CacheStats {
    pub hit_ratio: f64,
    pub total_requests: u64,
    pub cache_size: usize,
}

impl TestResults {
    /// Calculate success rate
    pub fn success_rate(&self) -> f64 {
        if self.total_operations == 0 {
            0.0
        } else {
            self.successful_operations as f64 / self.total_operations as f64
        }
    }

    /// Check if test passed performance criteria
    pub fn is_passing(&self) -> bool {
        self.success_rate() > 0.95 && self.throughput > 10.0 && self.error_count == 0
    }

    /// Generate a summary report
    pub fn summary(&self) -> String {
        format!(
            "Test: {}\n\
             Duration: {:?}\n\
             Operations: {}/{} ({:.1}% success)\n\
             Throughput: {:.1} ops/sec\n\
             Latency: avg={:?}, p95={:?}, p99={:?}\n\
             Memory: initial={:.1}MB, peak={:.1}MB, final={:.1}MB, efficiency={:.1}%\n\
             Cache: hit_ratio={:.1}%, requests={}, size={:.1}MB\n\
             Errors: {}\n\
             Status: {}",
            self.test_name,
            self.duration,
            self.successful_operations,
            self.total_operations,
            self.success_rate() * 100.0,
            self.throughput,
            self.average_latency,
            self.p95_latency,
            self.p99_latency,
            self.memory_usage.initial_memory as f64 / (1024.0 * 1024.0),
            self.memory_usage.peak_memory as f64 / (1024.0 * 1024.0),
            self.memory_usage.final_memory as f64 / (1024.0 * 1024.0),
            self.memory_usage.memory_efficiency * 100.0,
            self.cache_stats.hit_ratio * 100.0,
            self.cache_stats.total_requests,
            self.cache_stats.cache_size as f64 / (1024.0 * 1024.0),
            self.error_count,
            if self.is_passing() { "PASS" } else { "FAIL" }
        )
    }
}

/// Performance test suite
struct PerformanceTestSuite {
    performance_manager: Arc<PerformanceManager>,
    memory_profiler: Arc<MemoryProfiler>,
    optimizer: Arc<PerformanceOptimizer>,
}

impl PerformanceTestSuite {
    /// Create a new test suite
    fn new() -> Self {
        let performance_config = PerformanceConfig {
            cache: CacheConfig {
                max_entries: 10000,
                max_memory_bytes: 100 * 1024 * 1024, // 100MB
                ..Default::default()
            },
            enable_adaptive_optimization: true,
            ..Default::default()
        };

        let performance_manager = Arc::new(PerformanceManager::with_config(performance_config));
        
        let memory_profiler_config = MemoryProfilerConfig {
            enable_allocation_tracking: true,
            enable_background_monitoring: true,
            ..Default::default()
        };
        let memory_profiler = Arc::new(MemoryProfiler::with_config(memory_profiler_config));

        let optimizer_config = OptimizerConfig {
            enable_auto_optimization: true,
            optimization_interval: Duration::from_secs(30),
            ..Default::default()
        };
        let optimizer = Arc::new(PerformanceOptimizer::with_config(
            Arc::clone(&performance_manager),
            optimizer_config,
        ));

        Self {
            performance_manager,
            memory_profiler,
            optimizer,
        }
    }

    /// Run all performance tests
    fn run_all_tests(&self) -> Vec<TestResults> {
        let mut results = Vec::new();

        println!("Starting comprehensive performance test suite...\n");

        // Test 1: Baseline performance
        results.push(self.run_baseline_test());

        // Test 2: Scalability tests
        results.extend(self.run_scalability_tests());

        // Test 3: Memory stress tests
        results.extend(self.run_memory_stress_tests());

        // Test 4: Cache performance tests
        results.extend(self.run_cache_performance_tests());

        // Test 5: Concurrent processing tests
        results.extend(self.run_concurrent_tests());

        // Test 6: Long-running stability test
        results.push(self.run_stability_test());

        // Test 7: Optimization effectiveness test
        results.push(self.run_optimization_test());

        results
    }

    /// Run baseline performance test
    fn run_baseline_test(&self) -> TestResults {
        println!("Running baseline performance test...");
        
        let config = TestConfig {
            code_block_count: 1000,
            code_size: CodeSize::Medium,
            thread_count: 1,
            test_duration: Duration::from_secs(30),
            enable_caching: true,
            enable_memory_profiling: true,
            enable_optimization: false,
        };

        self.run_single_test("baseline_performance", config)
    }

    /// Run scalability tests with different loads
    fn run_scalability_tests(&self) -> Vec<TestResults> {
        println!("Running scalability tests...");
        
        let mut results = Vec::new();
        let counts = vec![100, 500, 1000, 5000, 10000];

        for count in counts {
            let config = TestConfig {
                code_block_count: count,
                code_size: CodeSize::Medium,
                thread_count: 1,
                test_duration: Duration::from_secs(60),
                enable_caching: true,
                enable_memory_profiling: true,
                enable_optimization: false,
            };

            let result = self.run_single_test(&format!("scalability_{}_blocks", count), config);
            results.push(result);
        }

        results
    }

    /// Run memory stress tests
    fn run_memory_stress_tests(&self) -> Vec<TestResults> {
        println!("Running memory stress tests...");
        
        let mut results = Vec::new();
        let sizes = vec![CodeSize::Small, CodeSize::Medium, CodeSize::Large, CodeSize::Huge];

        for size in sizes {
            let config = TestConfig {
                code_block_count: 1000,
                code_size: size,
                thread_count: 1,
                test_duration: Duration::from_secs(45),
                enable_caching: true,
                enable_memory_profiling: true,
                enable_optimization: false,
            };

            let result = self.run_single_test(&format!("memory_stress_{:?}", size), config);
            results.push(result);
        }

        results
    }

    /// Run cache performance tests
    fn run_cache_performance_tests(&self) -> Vec<TestResults> {
        println!("Running cache performance tests...");
        
        let mut results = Vec::new();

        // Test with cache enabled
        let config_with_cache = TestConfig {
            code_block_count: 2000,
            code_size: CodeSize::Medium,
            thread_count: 1,
            test_duration: Duration::from_secs(45),
            enable_caching: true,
            enable_memory_profiling: true,
            enable_optimization: false,
        };

        results.push(self.run_single_test("cache_enabled", config_with_cache.clone()));

        // Test with cache disabled
        let config_no_cache = TestConfig {
            enable_caching: false,
            ..config_with_cache
        };

        results.push(self.run_single_test("cache_disabled", config_no_cache));

        results
    }

    /// Run concurrent processing tests
    fn run_concurrent_tests(&self) -> Vec<TestResults> {
        println!("Running concurrent processing tests...");
        
        let mut results = Vec::new();
        let thread_counts = vec![1, 2, 4, 8, 16];

        for thread_count in thread_counts {
            let config = TestConfig {
                code_block_count: 1000,
                code_size: CodeSize::Medium,
                thread_count,
                test_duration: Duration::from_secs(45),
                enable_caching: true,
                enable_memory_profiling: true,
                enable_optimization: false,
            };

            let result = self.run_single_test(&format!("concurrent_{}_threads", thread_count), config);
            results.push(result);
        }

        results
    }

    /// Run long-running stability test
    fn run_stability_test(&self) -> TestResults {
        println!("Running stability test...");
        
        let config = TestConfig {
            code_block_count: 10000,
            code_size: CodeSize::Medium,
            thread_count: 4,
            test_duration: Duration::from_secs(300), // 5 minutes
            enable_caching: true,
            enable_memory_profiling: true,
            enable_optimization: true,
        };

        self.run_single_test("stability_long_running", config)
    }

    /// Run optimization effectiveness test
    fn run_optimization_test(&self) -> TestResults {
        println!("Running optimization effectiveness test...");
        
        let config = TestConfig {
            code_block_count: 5000,
            code_size: CodeSize::Large,
            thread_count: 8,
            test_duration: Duration::from_secs(120),
            enable_caching: true,
            enable_memory_profiling: true,
            enable_optimization: true,
        };

        self.run_single_test("optimization_effectiveness", config)
    }

    /// Run a single performance test
    fn run_single_test(&self, test_name: &str, config: TestConfig) -> TestResults {
        println!("  Running test: {}", test_name);

        // Start memory profiling if enabled
        if config.enable_memory_profiling {
            // Memory profiler would be started here
        }

        let start_time = Instant::now();
        let initial_memory = if config.enable_memory_profiling {
            self.memory_profiler.take_snapshot().total_allocated
        } else {
            0
        };

        let mut total_operations = 0;
        let mut successful_operations = 0;
        let mut latencies = Vec::new();
        let mut errors = Vec::new();
        let mut peak_memory = initial_memory;

        // Generate test codes
        let test_codes = self.generate_test_codes(config.code_block_count, config.code_size);
        let processing_config = ProcessingConfig::default();

        if config.thread_count == 1 {
            // Single-threaded execution
            for (i, code) in test_codes.iter().enumerate() {
                if start_time.elapsed() > config.test_duration {
                    break;
                }

                let op_start = Instant::now();
                match self.performance_manager.process_code_block(code, Some("rust"), &processing_config) {
                    Ok(_) => {
                        successful_operations += 1;
                        latencies.push(op_start.elapsed());
                    }
                    Err(e) => {
                        errors.push(format!("Operation {}: {}", i, e));
                    }
                }
                total_operations += 1;

                // Update peak memory
                if config.enable_memory_profiling && i % 100 == 0 {
                    let current_memory = self.memory_profiler.take_snapshot().total_allocated;
                    peak_memory = peak_memory.max(current_memory);
                }

                // Run optimization periodically if enabled
                if config.enable_optimization && i % 1000 == 0 && i > 0 {
                    let _opt_results = self.optimizer.run_auto_optimization();
                }
            }
        } else {
            // Multi-threaded execution
            let codes_per_thread = test_codes.len() / config.thread_count;
            let handles: Vec<_> = (0..config.thread_count)
                .map(|thread_id| {
                    let thread_codes: Vec<_> = test_codes
                        .iter()
                        .skip(thread_id * codes_per_thread)
                        .take(codes_per_thread)
                        .cloned()
                        .collect();
                    
                    let manager = Arc::clone(&self.performance_manager);
                    let config_clone = processing_config.clone();
                    let test_duration = config.test_duration;
                    let enable_optimization = config.enable_optimization;
                    let optimizer = Arc::clone(&self.optimizer);

                    thread::spawn(move || {
                        let thread_start = Instant::now();
                        let mut thread_successful = 0;
                        let mut thread_total = 0;
                        let mut thread_latencies = Vec::new();
                        let mut thread_errors = Vec::new();

                        for (i, code) in thread_codes.iter().enumerate() {
                            if thread_start.elapsed() > test_duration {
                                break;
                            }

                            let op_start = Instant::now();
                            match manager.process_code_block(code, Some("rust"), &config_clone) {
                                Ok(_) => {
                                    thread_successful += 1;
                                    thread_latencies.push(op_start.elapsed());
                                }
                                Err(e) => {
                                    thread_errors.push(format!("Thread {} Operation {}: {}", thread_id, i, e));
                                }
                            }
                            thread_total += 1;

                            // Run optimization periodically if enabled
                            if enable_optimization && i % 500 == 0 && i > 0 {
                                let _opt_results = optimizer.run_auto_optimization();
                            }
                        }

                        (thread_successful, thread_total, thread_latencies, thread_errors)
                    })
                })
                .collect();

            // Collect results from all threads
            for handle in handles {
                let (thread_successful, thread_total, thread_latencies, thread_errors) = handle.join().unwrap();
                successful_operations += thread_successful;
                total_operations += thread_total;
                latencies.extend(thread_latencies);
                errors.extend(thread_errors);
            }

            // Update peak memory for multi-threaded case
            if config.enable_memory_profiling {
                peak_memory = self.memory_profiler.take_snapshot().total_allocated;
            }
        }

        let duration = start_time.elapsed();
        let final_memory = if config.enable_memory_profiling {
            self.memory_profiler.take_snapshot().total_allocated
        } else {
            0
        };

        // Calculate latency percentiles
        latencies.sort();
        let average_latency = if !latencies.is_empty() {
            latencies.iter().sum::<Duration>() / latencies.len() as u32
        } else {
            Duration::from_millis(0)
        };

        let p95_latency = if !latencies.is_empty() {
            let index = (latencies.len() as f64 * 0.95) as usize;
            latencies.get(index).copied().unwrap_or(Duration::from_millis(0))
        } else {
            Duration::from_millis(0)
        };

        let p99_latency = if !latencies.is_empty() {
            let index = (latencies.len() as f64 * 0.99) as usize;
            latencies.get(index).copied().unwrap_or(Duration::from_millis(0))
        } else {
            Duration::from_millis(0)
        };

        // Calculate throughput
        let throughput = if duration.as_secs_f64() > 0.0 {
            successful_operations as f64 / duration.as_secs_f64()
        } else {
            0.0
        };

        // Get cache statistics
        let cache_stats = if config.enable_caching {
            let perf_metrics = self.performance_manager.get_performance_metrics();
            CacheStats {
                hit_ratio: perf_metrics.cache_hit_ratio,
                total_requests: perf_metrics.total_processed,
                cache_size: 0, // Would get from cache size info
            }
        } else {
            CacheStats::default()
        };

        // Calculate memory efficiency
        let memory_efficiency = if peak_memory > 0 {
            (successful_operations * 1000) as f64 / peak_memory as f64 // Operations per KB
        } else {
            0.0
        };

        let memory_usage = MemoryUsageStats {
            initial_memory,
            peak_memory,
            final_memory,
            memory_efficiency,
        };

        TestResults {
            test_name: test_name.to_string(),
            config,
            duration,
            total_operations,
            successful_operations,
            throughput,
            average_latency,
            p95_latency,
            p99_latency,
            memory_usage,
            cache_stats,
            error_count: errors.len(),
            errors,
        }
    }

    /// Generate test code samples
    fn generate_test_codes(&self, count: usize, size: CodeSize) -> Vec<String> {
        (0..count)
            .map(|i| match size {
                CodeSize::Small => format!("fn small_func_{}() {{ println!(\"Hello {}\"); }}", i, i),
                CodeSize::Medium => format!(
                    r#"
fn medium_func_{}(x: i32, y: i32) -> i32 {{
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
mod tests_{} {{
    use super::*;

    #[test]
    fn test_medium_func_{}() {{
        assert_eq!(medium_func_{}(10, 20), 30);
        assert_eq!(medium_func_{}(60, 50), 220);
    }}
}}
"#,
                    i, i, i, i, i
                ),
                CodeSize::Large => self.generate_large_code(i),
                CodeSize::Huge => self.generate_huge_code(i),
            })
            .collect()
    }

    fn generate_large_code(&self, index: usize) -> String {
        format!(
            r#"
use std::collections::HashMap;
use std::sync::{{Arc, Mutex}};

#[derive(Debug, Clone)]
pub struct LargeStruct{} {{
    pub id: u64,
    pub name: String,
    pub data: HashMap<String, i32>,
    pub metadata: Vec<String>,
}}

impl LargeStruct{} {{
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

    pub fn process_data(&self) -> i32 {{
        self.data.values().sum()
    }}

    pub fn get_summary(&self) -> String {{
        format!("LargeStruct{} with {{}} items", {}, self.data.len())
    }}
}}

pub fn process_large_struct{}() -> LargeStruct{} {{
    let mut s = LargeStruct{}::new({}, "test_struct{}".to_string());
    for i in 0..50 {{
        s.add_data(format!("key_{{}}", i), i);
    }}
    s
}}
"#,
            index, index, index, index, index, index, index, index, index
        )
    }

    fn generate_huge_code(&self, index: usize) -> String {
        let mut code = format!(
            r#"
// Huge code block {} - comprehensive data processing system
use std::collections::{{HashMap, BTreeMap, HashSet}};
use std::sync::{{Arc, Mutex, RwLock}};
use std::thread;
use std::time::{{Duration, Instant}};

#[derive(Debug, Clone)]
pub struct HugeDataProcessor_{} {{
    pub processors: Vec<DataProcessor_{}>,
    pub cache: Arc<RwLock<HashMap<String, ProcessedData_{}>>>,
    pub config: ProcessorConfig_{},
    pub stats: ProcessorStats_{},
}}
"#,
            index, index, index, index, index, index
        );

        // Add many struct definitions and implementations
        for i in 0..10 {
            code.push_str(&format!(
                r#"
#[derive(Debug, Clone)]
pub struct DataProcessor_{}_{}  {{
    pub id: usize,
    pub data: Vec<i32>,
    pub metadata: HashMap<String, String>,
}}

impl DataProcessor_{}_{} {{
    pub fn new(id: usize) -> Self {{
        Self {{
            id,
            data: Vec::new(),
            metadata: HashMap::new(),
        }}
    }}

    pub fn process(&mut self, input: &[i32]) -> Vec<i32> {{
        self.data.extend_from_slice(input);
        self.data.iter().map(|x| x * 2).collect()
    }}

    pub fn get_stats(&self) -> (usize, i32) {{
        (self.data.len(), self.data.iter().sum())
    }}
}}
"#,
                index, i, index, i
            ));
        }

        code
    }
}

/// Main function to run performance tests
fn main() {
    println!("Code Block Processing Performance Test Suite");
    println!("============================================\n");

    let test_suite = PerformanceTestSuite::new();
    let results = test_suite.run_all_tests();

    println!("\n\nPerformance Test Results Summary");
    println!("================================\n");

    let mut passing_tests = 0;
    let mut total_tests = results.len();

    for result in &results {
        println!("{}\n", result.summary());
        if result.is_passing() {
            passing_tests += 1;
        }
    }

    println!("Overall Results:");
    println!("  Total Tests: {}", total_tests);
    println!("  Passing Tests: {}", passing_tests);
    println!("  Failing Tests: {}", total_tests - passing_tests);
    println!("  Success Rate: {:.1}%", (passing_tests as f64 / total_tests as f64) * 100.0);

    // Generate performance report
    let mut performance_summary = HashMap::new();
    for result in &results {
        performance_summary.insert(result.test_name.clone(), result.throughput);
    }

    println!("\nThroughput Summary (ops/sec):");
    let mut sorted_results: Vec<_> = performance_summary.iter().collect();
    sorted_results.sort_by(|a, b| b.1.partial_cmp(a.1).unwrap());
    
    for (test_name, throughput) in sorted_results {
        println!("  {}: {:.1}", test_name, throughput);
    }

    // Exit with appropriate code
    if passing_tests == total_tests {
        println!("\n✅ All performance tests passed!");
        std::process::exit(0);
    } else {
        println!("\n❌ Some performance tests failed!");
        std::process::exit(1);
    }
}