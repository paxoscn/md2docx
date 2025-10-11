# Performance Testing Guide

This document describes the comprehensive performance testing system for the code block processing strategy implementation.

## Overview

The performance testing system provides multiple layers of testing and monitoring:

1. **Unit Performance Tests** - Test individual components
2. **Integration Performance Tests** - Test complete workflows
3. **Benchmark Suite** - Criterion-based micro-benchmarks
4. **Comprehensive Test Runner** - Full system performance validation
5. **Memory Profiling** - Memory usage analysis and leak detection
6. **Performance Optimization** - Automated performance tuning
7. **Continuous Monitoring** - Real-time performance tracking

## Quick Start

### Running All Performance Tests

```bash
# Run the complete performance test suite
./scripts/run_performance_tests.sh
```

### Running Individual Test Types

```bash
# Unit tests with performance focus
cargo test --lib --release

# Integration performance tests
cargo test --test performance_integration_tests --release

# Criterion benchmarks
cargo bench --bench code_block_performance

# Comprehensive performance test runner
cargo run --release --bin performance-test-runner
```

### Performance Monitoring

```bash
# Single performance measurement
python3 scripts/performance_monitor.py

# Continuous monitoring (every 5 minutes)
python3 scripts/performance_monitor.py --continuous --interval 300

# Quick tests for faster feedback
python3 scripts/performance_monitor.py --continuous --quick --interval 60

# Generate performance plots
python3 scripts/performance_monitor.py --plot
```

## Test Categories

### 1. Baseline Performance Tests

Tests basic performance characteristics with default settings:

- **Throughput**: Operations per second
- **Latency**: Average, P95, P99 response times
- **Memory Usage**: Peak and average memory consumption
- **Success Rate**: Percentage of successful operations

**Expected Thresholds:**
- Throughput: > 10 ops/sec
- Average Latency: < 1000ms
- Success Rate: > 95%

### 2. Scalability Tests

Tests performance under different load conditions:

- Small load: 100 code blocks
- Medium load: 1,000 code blocks  
- Large load: 10,000 code blocks
- Huge load: 100,000 code blocks

**Key Metrics:**
- Throughput scaling
- Memory usage growth
- Latency degradation

### 3. Memory Stress Tests

Tests memory usage patterns with different code sizes:

- **Small blocks**: ~100 bytes each
- **Medium blocks**: ~1KB each
- **Large blocks**: ~10KB each
- **Huge blocks**: ~100KB each

**Validation:**
- Memory efficiency
- Garbage collection effectiveness
- Memory leak detection

### 4. Cache Performance Tests

Compares performance with caching enabled vs disabled:

- Cache hit ratio measurement
- Cache memory usage
- Performance improvement quantification

**Expected Results:**
- Cache hit ratio: > 30%
- Performance improvement: 2-5x for repeated operations

### 5. Concurrent Processing Tests

Tests performance under concurrent load:

- 1, 2, 4, 8, 16 concurrent threads
- Thread safety validation
- Scalability measurement

**Key Metrics:**
- Parallel efficiency
- Resource contention
- Throughput scaling

### 6. Long-Running Stability Tests

Tests system stability over extended periods:

- 5-10 minute continuous operation
- Memory leak detection
- Performance degradation monitoring

### 7. Optimization Effectiveness Tests

Tests the automatic optimization system:

- Performance improvement measurement
- Optimization trigger validation
- Resource usage optimization

## Performance Benchmarks

### Criterion Benchmarks

Located in `benches/code_block_performance.rs`, these provide detailed micro-benchmarks:

```bash
# Run all benchmarks
cargo bench

# Run specific benchmark group
cargo bench -- single_processing
cargo bench -- batch_processing
cargo bench -- cache_performance
```

**Benchmark Categories:**
- Single code block processing
- Batch processing with different sizes
- Cache hit/miss performance
- Memory allocation patterns
- Strategy registry operations
- Performance manager operations
- Concurrent processing

### Benchmark Configuration

Benchmarks can be configured via `benches/benchmark_config.toml`:

```toml
[scenarios.baseline]
name = "Baseline Performance"
code_block_count = 1000
code_size = "medium"
thread_count = 1
enable_caching = true
duration_seconds = 30
```

## Memory Profiling

### Memory Profiler Features

The integrated memory profiler provides:

- **Allocation Tracking**: Track all memory allocations
- **Leak Detection**: Identify potential memory leaks
- **Usage Patterns**: Analyze memory usage over time
- **Efficiency Metrics**: Calculate memory efficiency scores

### Using the Memory Profiler

```rust
use md2docx::markdown::code_block::MemoryProfiler;

let profiler = MemoryProfiler::new();

// Track allocation
profiler.track_allocation(ptr as usize, size, "cache");

// Track deallocation  
profiler.track_deallocation(ptr as usize);

// Generate report
let report = profiler.generate_report();
println!("Memory Report: {}", report.summary());
```

### Memory Analysis

The profiler provides several analysis features:

- **Trend Analysis**: Memory usage trends over time
- **Leak Detection**: Identify long-lived allocations
- **Efficiency Scoring**: Memory utilization efficiency
- **Fragmentation Analysis**: Memory fragmentation levels

## Performance Optimization

### Automatic Optimization

The performance optimizer can automatically improve system performance:

```rust
use md2docx::markdown::code_block::PerformanceOptimizer;

let optimizer = PerformanceOptimizer::new(performance_manager);

// Run optimization
let results = optimizer.run_auto_optimization();

// Check results
for result in results {
    println!("Optimization: {}", result.summary());
}
```

### Optimization Types

- **Cache Resize**: Adjust cache size based on usage
- **Cache Cleanup**: Remove stale cache entries
- **Memory Cleanup**: Force garbage collection
- **Strategy Preload**: Preload frequently used strategies
- **Config Tuning**: Adjust configuration parameters
- **Resource Rebalancing**: Redistribute system resources

### Optimization Triggers

Optimizations are triggered when:

- Performance score < 60%
- Memory utilization > 80%
- Cache hit ratio < 40%
- System overload detected

## Performance Monitoring

### Real-Time Monitoring

The monitoring system provides continuous performance tracking:

```bash
# Start monitoring dashboard
python3 scripts/performance_monitor.py --continuous

# Monitor with custom interval
python3 scripts/performance_monitor.py --continuous --interval 120

# Generate performance plots
python3 scripts/performance_monitor.py --plot --save-plot
```

### Metrics Collected

- **Throughput**: Operations per second
- **Latency**: Response time percentiles
- **Memory Usage**: Current and peak memory
- **Cache Performance**: Hit ratios and efficiency
- **Error Rates**: Success/failure rates
- **System Health**: Overall performance scores

### Trend Analysis

The monitoring system provides trend analysis:

- Performance improvement/degradation
- Memory usage patterns
- Cache effectiveness trends
- System stability metrics

## Performance Thresholds

### Pass/Fail Criteria

Tests use the following thresholds:

```toml
[thresholds]
min_throughput = 10.0              # ops/sec
max_average_latency = 1000         # milliseconds
max_p95_latency = 5000            # milliseconds
max_p99_latency = 10000           # milliseconds
min_success_rate = 95.0           # percentage
max_memory_growth_rate = 1.0      # MB/sec
min_cache_hit_ratio = 30.0        # percentage
```

### Performance Targets

**Baseline Performance:**
- Throughput: 50-100 ops/sec
- Average Latency: 10-50ms
- Memory Efficiency: > 60%
- Cache Hit Ratio: > 50%

**Scalability Targets:**
- Linear throughput scaling up to 4 threads
- Memory usage growth < O(n²)
- Latency increase < 2x at 10x load

**Stability Targets:**
- No memory leaks over 10 minutes
- Performance degradation < 10% over time
- Error rate < 1%

## Troubleshooting Performance Issues

### Common Performance Problems

1. **Low Throughput**
   - Check for blocking operations
   - Verify cache effectiveness
   - Review algorithm complexity

2. **High Memory Usage**
   - Check for memory leaks
   - Review cache size settings
   - Analyze allocation patterns

3. **Poor Cache Performance**
   - Review cache configuration
   - Check cache key generation
   - Analyze access patterns

4. **Concurrency Issues**
   - Check for lock contention
   - Review thread safety
   - Analyze resource sharing

### Performance Debugging

```bash
# Run with detailed profiling
RUST_LOG=debug cargo run --release --bin performance-test-runner

# Memory leak detection with Valgrind
valgrind --tool=memcheck --leak-check=full \
  cargo run --release --bin performance-test-runner

# CPU profiling with perf
perf record cargo run --release --bin performance-test-runner
perf report
```

### Performance Tuning

1. **Cache Tuning**
   ```rust
   let cache_config = CacheConfig {
       max_entries: 10000,
       max_memory_bytes: 100 * 1024 * 1024, // 100MB
       entry_ttl_seconds: 3600,
   };
   ```

2. **Memory Management**
   ```rust
   let memory_config = MemoryConfig {
       max_memory_usage: 500 * 1024 * 1024, // 500MB
       cleanup_threshold: 0.8,
       gc_interval: Duration::from_secs(60),
   };
   ```

3. **Parallel Processing**
   ```rust
   let parallel_config = ParallelConfig {
       max_workers: num_cpus::get(),
       queue_size: 1000,
       timeout: Duration::from_secs(30),
   };
   ```

## Continuous Integration

### CI Performance Tests

Include performance tests in CI pipeline:

```yaml
# .github/workflows/performance.yml
name: Performance Tests
on: [push, pull_request]

jobs:
  performance:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      - name: Install Rust
        uses: actions-rs/toolchain@v1
      - name: Run Performance Tests
        run: ./scripts/run_performance_tests.sh
      - name: Upload Results
        uses: actions/upload-artifact@v2
        with:
          name: performance-results
          path: performance_results/
```

### Performance Regression Detection

The test suite includes regression detection:

- Compare with previous test runs
- Alert on performance degradation > 10%
- Track performance trends over time

## Best Practices

### Writing Performance Tests

1. **Use Realistic Data**: Test with representative code samples
2. **Measure Multiple Metrics**: Don't focus only on throughput
3. **Include Edge Cases**: Test with extreme inputs
4. **Warm Up**: Allow for JIT compilation and cache warming
5. **Statistical Significance**: Run multiple iterations

### Performance Optimization

1. **Profile First**: Identify bottlenecks before optimizing
2. **Measure Impact**: Quantify optimization benefits
3. **Consider Trade-offs**: Balance speed vs memory vs complexity
4. **Test Thoroughly**: Ensure optimizations don't break functionality
5. **Document Changes**: Record optimization rationale and results

### Monitoring and Alerting

1. **Set Realistic Thresholds**: Based on actual usage patterns
2. **Monitor Trends**: Look for gradual degradation
3. **Alert on Anomalies**: Detect sudden performance changes
4. **Regular Reviews**: Periodically review performance data
5. **Capacity Planning**: Use metrics for resource planning

## References

- [Criterion.rs Documentation](https://bheisler.github.io/criterion.rs/book/)
- [Rust Performance Book](https://nnethercote.github.io/perf-book/)
- [Memory Profiling in Rust](https://github.com/koute/memory-profiler)
- [Benchmarking Best Practices](https://github.com/rust-lang/rfcs/blob/master/text/2616-object-safe-for-dispatch.md)