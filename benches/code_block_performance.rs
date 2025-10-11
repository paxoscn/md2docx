//! Performance benchmarks for code block processing
//! 
//! This module contains comprehensive benchmarks to test the performance
//! of the code block strategy system under various conditions.

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId, Throughput};
use std::time::Duration;
use std::sync::Arc;

use md2docx::markdown::code_block::{
    CodeBlockStrategy, ProcessingConfig, DefaultStrategy,
    CodeBlockCache, CacheConfig, CacheKey,
    PerformanceManager, PerformanceConfig,
    StrategyRegistry,
};

// Import Rust strategy if available
#[cfg(feature = "rust-strategy")]
use md2docx::markdown::code_block::strategies::RustStrategy;

/// Generate test code samples of various sizes
fn generate_code_samples() -> Vec<(String, &'static str, usize)> {
    vec![
        // Small code blocks (< 100 chars)
        ("fn main() {}".to_string(), "rust", 13),
        ("console.log('hello');".to_string(), "javascript", 21),
        ("print('hello')".to_string(), "python", 14),
        
        // Medium code blocks (100-1000 chars)
        (format!("fn fibonacci(n: u32) -> u32 {{\n    match n {{\n        0 => 0,\n        1 => 1,\n        _ => fibonacci(n - 1) + fibonacci(n - 2),\n    }}\n}}\n\nfn main() {{\n    for i in 0..10 {{\n        println!(\"fibonacci({{}}) = {{}}\", i, fibonacci(i));\n    }}\n}}"), "rust", 250),
        
        // Large code blocks (> 1000 chars)
        (generate_large_rust_code(), "rust", 2000),
        (generate_large_javascript_code(), "javascript", 1800),
        (generate_large_python_code(), "python", 1500),
    ]
}

fn generate_large_rust_code() -> String {
    format!(r#"
use std::collections::HashMap;
use std::sync::{{Arc, Mutex}};
use std::thread;
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct DataProcessor {{
    data: Arc<Mutex<HashMap<String, i32>>>,
    config: ProcessorConfig,
}}

#[derive(Debug, Clone)]
pub struct ProcessorConfig {{
    pub batch_size: usize,
    pub timeout: Duration,
    pub retry_count: u32,
}}

impl DataProcessor {{
    pub fn new(config: ProcessorConfig) -> Self {{
        Self {{
            data: Arc::new(Mutex::new(HashMap::new())),
            config,
        }}
    }}

    pub fn process_batch(&self, items: Vec<String>) -> Result<Vec<i32>, ProcessingError> {{
        let mut results = Vec::new();
        
        for item in items.chunks(self.config.batch_size) {{
            let chunk_results = self.process_chunk(item)?;
            results.extend(chunk_results);
        }}
        
        Ok(results)
    }}

    fn process_chunk(&self, chunk: &[String]) -> Result<Vec<i32>, ProcessingError> {{
        let handles: Vec<_> = chunk
            .iter()
            .map(|item| {{
                let item = item.clone();
                let data = Arc::clone(&self.data);
                
                thread::spawn(move || {{
                    let mut data = data.lock().unwrap();
                    let value = item.len() as i32;
                    data.insert(item.clone(), value);
                    value
                }})
            }})
            .collect();

        let mut results = Vec::new();
        for handle in handles {{
            results.push(handle.join().map_err(|_| ProcessingError::ThreadPanic)?);
        }}
        
        Ok(results)
    }}
}}

#[derive(Debug)]
pub enum ProcessingError {{
    ThreadPanic,
    Timeout,
    InvalidData(String),
}}

impl std::fmt::Display for ProcessingError {{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {{
        match self {{
            ProcessingError::ThreadPanic => write!(f, "Thread panicked"),
            ProcessingError::Timeout => write!(f, "Operation timed out"),
            ProcessingError::InvalidData(msg) => write!(f, "Invalid data: {{}}", msg),
        }}
    }}
}}

impl std::error::Error for ProcessingError {{}}
"#)
}

fn generate_large_javascript_code() -> String {
    format!(r#"
class DataProcessor {{
    constructor(config) {{
        this.data = new Map();
        this.config = config;
        this.workers = [];
    }}

    async processBatch(items) {{
        const chunks = this.chunkArray(items, this.config.batchSize);
        const promises = chunks.map(chunk => this.processChunk(chunk));
        
        try {{
            const results = await Promise.all(promises);
            return results.flat();
        }} catch (error) {{
            throw new ProcessingError(`Batch processing failed: ${{error.message}}`);
        }}
    }}

    async processChunk(chunk) {{
        return new Promise((resolve, reject) => {{
            const worker = new Worker('processor-worker.js');
            
            const timeout = setTimeout(() => {{
                worker.terminate();
                reject(new ProcessingError('Processing timeout'));
            }}, this.config.timeout);

            worker.onmessage = (event) => {{
                clearTimeout(timeout);
                worker.terminate();
                
                if (event.data.error) {{
                    reject(new ProcessingError(event.data.error));
                }} else {{
                    // Store results in data map
                    event.data.results.forEach((result, index) => {{
                        this.data.set(chunk[index], result);
                    }});
                    resolve(event.data.results);
                }}
            }};

            worker.onerror = (error) => {{
                clearTimeout(timeout);
                worker.terminate();
                reject(new ProcessingError(`Worker error: ${{error.message}}`));
            }};

            worker.postMessage({{ chunk, config: this.config }});
        }});
    }}

    chunkArray(array, size) {{
        const chunks = [];
        for (let i = 0; i < array.length; i += size) {{
            chunks.push(array.slice(i, i + size));
        }}
        return chunks;
    }}

    getProcessedData() {{
        return Object.fromEntries(this.data);
    }}

    cleanup() {{
        this.workers.forEach(worker => worker.terminate());
        this.workers = [];
        this.data.clear();
    }}
}}

class ProcessingError extends Error {{
    constructor(message) {{
        super(message);
        this.name = 'ProcessingError';
    }}
}}

// Usage example
const processor = new DataProcessor({{
    batchSize: 10,
    timeout: 5000,
    retryCount: 3
}});

async function main() {{
    const items = Array.from({{ length: 1000 }}, (_, i) => `item-${{i}}`);
    
    try {{
        const results = await processor.processBatch(items);
        console.log(`Processed ${{results.length}} items`);
        console.log('Sample results:', results.slice(0, 5));
    }} catch (error) {{
        console.error('Processing failed:', error.message);
    }} finally {{
        processor.cleanup();
    }}
}}

main().catch(console.error);
"#)
}

fn generate_large_python_code() -> String {
    format!(r#"
import asyncio
import concurrent.futures
import time
from typing import List, Dict, Any, Optional
from dataclasses import dataclass
from enum import Enum

class ProcessingStatus(Enum):
    PENDING = "pending"
    PROCESSING = "processing"
    COMPLETED = "completed"
    FAILED = "failed"

@dataclass
class ProcessingConfig:
    batch_size: int = 10
    timeout: float = 5.0
    retry_count: int = 3
    max_workers: int = 4

@dataclass
class ProcessingResult:
    item_id: str
    status: ProcessingStatus
    result: Optional[Any] = None
    error: Optional[str] = None
    processing_time: float = 0.0

class DataProcessor:
    def __init__(self, config: ProcessingConfig):
        self.config = config
        self.data: Dict[str, Any] = {{}}
        self.executor = concurrent.futures.ThreadPoolExecutor(
            max_workers=config.max_workers
        )
        
    async def process_batch(self, items: List[str]) -> List[ProcessingResult]:
        """Process a batch of items with concurrent execution."""
        chunks = self._chunk_items(items, self.config.batch_size)
        
        tasks = [
            self._process_chunk_async(chunk, chunk_id)
            for chunk_id, chunk in enumerate(chunks)
        ]
        
        try:
            chunk_results = await asyncio.gather(*tasks, return_exceptions=True)
            
            # Flatten results and handle exceptions
            results = []
            for chunk_result in chunk_results:
                if isinstance(chunk_result, Exception):
                    # Create error results for the chunk
                    error_msg = str(chunk_result)
                    chunk_size = len(chunks[len(results) // self.config.batch_size])
                    for i in range(chunk_size):
                        results.append(ProcessingResult(
                            item_id=f"unknown-{{i}}",
                            status=ProcessingStatus.FAILED,
                            error=error_msg
                        ))
                else:
                    results.extend(chunk_result)
                    
            return results
            
        except Exception as e:
            raise ProcessingError(f"Batch processing failed: {{e}}")
    
    async def _process_chunk_async(self, chunk: List[str], chunk_id: int) -> List[ProcessingResult]:
        """Process a chunk of items asynchronously."""
        loop = asyncio.get_event_loop()
        
        try:
            # Run the synchronous processing in a thread pool
            future = loop.run_in_executor(
                self.executor,
                self._process_chunk_sync,
                chunk,
                chunk_id
            )
            
            # Wait for completion with timeout
            results = await asyncio.wait_for(
                future,
                timeout=self.config.timeout
            )
            
            return results
            
        except asyncio.TimeoutError:
            raise ProcessingError(f"Chunk {{chunk_id}} processing timed out")
        except Exception as e:
            raise ProcessingError(f"Chunk {{chunk_id}} processing failed: {{e}}")
    
    def _process_chunk_sync(self, chunk: List[str], chunk_id: int) -> List[ProcessingResult]:
        """Synchronously process a chunk of items."""
        results = []
        
        for item in chunk:
            start_time = time.time()
            
            try:
                # Simulate processing work
                processed_value = self._process_single_item(item)
                
                # Store in data dictionary
                self.data[item] = processed_value
                
                result = ProcessingResult(
                    item_id=item,
                    status=ProcessingStatus.COMPLETED,
                    result=processed_value,
                    processing_time=time.time() - start_time
                )
                
            except Exception as e:
                result = ProcessingResult(
                    item_id=item,
                    status=ProcessingStatus.FAILED,
                    error=str(e),
                    processing_time=time.time() - start_time
                )
            
            results.append(result)
        
        return results
    
    def _process_single_item(self, item: str) -> Dict[str, Any]:
        """Process a single item and return the result."""
        # Simulate some computational work
        time.sleep(0.001)  # 1ms processing time
        
        return {{
            "original": item,
            "length": len(item),
            "hash": hash(item),
            "processed_at": time.time(),
            "metadata": {{
                "type": "string",
                "encoding": "utf-8",
                "checksum": sum(ord(c) for c in item)
            }}
        }}
    
    def _chunk_items(self, items: List[str], chunk_size: int) -> List[List[str]]:
        """Split items into chunks of specified size."""
        return [
            items[i:i + chunk_size]
            for i in range(0, len(items), chunk_size)
        ]
    
    def get_statistics(self) -> Dict[str, Any]:
        """Get processing statistics."""
        return {{
            "total_items": len(self.data),
            "data_size_bytes": sum(
                len(str(k)) + len(str(v)) for k, v in self.data.items()
            ),
            "config": self.config.__dict__
        }}
    
    def cleanup(self):
        """Clean up resources."""
        self.executor.shutdown(wait=True)
        self.data.clear()

class ProcessingError(Exception):
    """Custom exception for processing errors."""
    pass

# Usage example
async def main():
    config = ProcessingConfig(
        batch_size=20,
        timeout=10.0,
        retry_count=2,
        max_workers=8
    )
    
    processor = DataProcessor(config)
    
    try:
        # Generate test items
        items = [f"item-{{i:04d}}" for i in range(1000)]
        
        print(f"Processing {{len(items)}} items...")
        start_time = time.time()
        
        results = await processor.process_batch(items)
        
        processing_time = time.time() - start_time
        
        # Analyze results
        successful = [r for r in results if r.status == ProcessingStatus.COMPLETED]
        failed = [r for r in results if r.status == ProcessingStatus.FAILED]
        
        print(f"Processing completed in {{processing_time:.2f}}s")
        print(f"Successful: {{len(successful)}}, Failed: {{len(failed)}}")
        
        if successful:
            avg_time = sum(r.processing_time for r in successful) / len(successful)
            print(f"Average processing time per item: {{avg_time*1000:.2f}}ms")
        
        stats = processor.get_statistics()
        print(f"Statistics: {{stats}}")
        
    except ProcessingError as e:
        print(f"Processing error: {{e}}")
    except Exception as e:
        print(f"Unexpected error: {{e}}")
    finally:
        processor.cleanup()

if __name__ == "__main__":
    asyncio.run(main())
"#)
}

/// Benchmark single code block processing
fn bench_single_processing(c: &mut Criterion) {
    let samples = generate_code_samples();
    let strategy = DefaultStrategy::new();
    let config = ProcessingConfig::default();
    
    let mut group = c.benchmark_group("single_processing");
    
    for (code, language, size) in samples {
        group.throughput(Throughput::Bytes(size as u64));
        group.bench_with_input(
            BenchmarkId::new("default_strategy", format!("{}_{}bytes", language, size)),
            &(code.as_str(), language),
            |b, (code, language)| {
                b.iter(|| {
                    let result = strategy.process(black_box(code), black_box(&config));
                    black_box(result)
                });
            },
        );
    }
    
    group.finish();
}

/// Benchmark batch processing with different sizes
fn bench_batch_processing(c: &mut Criterion) {
    let batch_sizes = vec![1, 10, 50, 100, 500];
    let base_code = "fn main() { println!(\"Hello, world!\"); }";
    let config = ProcessingConfig::default();
    
    let mut group = c.benchmark_group("batch_processing");
    
    for batch_size in batch_sizes {
        let codes: Vec<String> = (0..batch_size)
            .map(|i| format!("fn main{i}() {{ println!(\"Hello, world {i}!\"); }}"))
            .collect();
        
        group.throughput(Throughput::Elements(batch_size as u64));
        group.bench_with_input(
            BenchmarkId::new("sequential", batch_size),
            &codes,
            |b, codes| {
                b.iter(|| {
                    let strategy = DefaultStrategy::new();
                    let results: Vec<_> = codes
                        .iter()
                        .map(|code| strategy.process(black_box(code), black_box(&config)))
                        .collect();
                    black_box(results)
                });
            },
        );
    }
    
    group.finish();
}

/// Benchmark cache performance
fn bench_cache_performance(c: &mut Criterion) {
    let cache = CodeBlockCache::new();
    let config = ProcessingConfig::default();
    let strategy = DefaultStrategy::new();
    
    // Pre-populate cache with some entries
    for i in 0..100 {
        let code = format!("fn test{i}() {{ println!(\"test {i}\"); }}");
        let key = CacheKey::new(&code, Some("rust"), &config, "1.0.0");
        let result = strategy.process(&code, &config).unwrap();
        cache.put(key, result);
    }
    
    let mut group = c.benchmark_group("cache_performance");
    
    // Benchmark cache hits
    group.bench_function("cache_hit", |b| {
        let code = "fn test50() { println!(\"test 50\"); }";
        let key = CacheKey::new(code, Some("rust"), &config, "1.0.0");
        
        b.iter(|| {
            let result = cache.get_or_compute(key.clone(), || {
                panic!("Should not compute - should be cache hit");
            });
            black_box(result)
        });
    });
    
    // Benchmark cache misses
    group.bench_function("cache_miss", |b| {
        let mut counter = 0;
        b.iter(|| {
            counter += 1;
            let code = format!("fn unique{counter}() {{ println!(\"unique {counter}\"); }}");
            let key = CacheKey::new(&code, Some("rust"), &config, "1.0.0");
            
            let result = cache.get_or_compute(key, || {
                strategy.process(&code, &config)
                    .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)
            });
            black_box(result)
        });
    });
    
    group.finish();
}

/// Benchmark memory usage patterns
fn bench_memory_usage(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_usage");
    
    // Test memory allocation patterns for different code sizes
    let sizes = vec![100, 1000, 10000, 100000];
    
    for size in sizes {
        let code = "x".repeat(size);
        
        group.throughput(Throughput::Bytes(size as u64));
        group.bench_with_input(
            BenchmarkId::new("memory_allocation", size),
            &code,
            |b, code| {
                b.iter(|| {
                    let strategy = DefaultStrategy::new();
                    let config = ProcessingConfig::default();
                    let result = strategy.process(black_box(code), black_box(&config));
                    
                    // Force allocation and deallocation
                    if let Ok(processed) = result {
                        let _clone = processed.clone();
                        black_box(processed);
                    }
                });
            },
        );
    }
    
    group.finish();
}

/// Benchmark strategy registry performance
fn bench_strategy_registry(c: &mut Criterion) {
    let mut registry = StrategyRegistry::new();
    
    // Register multiple strategies
    registry.register_strategy("default", Box::new(DefaultStrategy::new()));
    
    #[cfg(feature = "rust-strategy")]
    registry.register_strategy("rust", Box::new(RustStrategy::new()));
    
    let mut group = c.benchmark_group("strategy_registry");
    
    // Benchmark strategy lookup
    group.bench_function("strategy_lookup", |b| {
        b.iter(|| {
            let strategy = registry.get_strategy(black_box("rust"));
            black_box(strategy);
        });
    });
    
    // Benchmark strategy registration
    group.bench_function("strategy_registration", |b| {
        b.iter(|| {
            let mut temp_registry = StrategyRegistry::new();
            temp_registry.register_strategy("test", Box::new(DefaultStrategy::new()));
            black_box(temp_registry);
        });
    });
    
    group.finish();
}

/// Benchmark performance manager operations
fn bench_performance_manager(c: &mut Criterion) {
    let manager = PerformanceManager::new();
    let config = ProcessingConfig::default();
    
    let mut group = c.benchmark_group("performance_manager");
    
    // Benchmark single code block processing through manager
    group.bench_function("managed_processing", |b| {
        b.iter(|| {
            let result = manager.process_code_block(
                black_box("fn main() {}"),
                black_box(Some("rust")),
                black_box(&config),
            );
            black_box(result)
        });
    });
    
    // Benchmark batch processing through manager
    group.bench_function("managed_batch_processing", |b| {
        let requests = vec![
            ("fn main() {}".to_string(), Some("rust".to_string()), config.clone()),
            ("console.log('hello');".to_string(), Some("javascript".to_string()), config.clone()),
            ("print('hello')".to_string(), Some("python".to_string()), config.clone()),
        ];
        
        b.iter(|| {
            let results = manager.process_code_blocks(black_box(requests.clone()));
            black_box(results)
        });
    });
    
    // Benchmark metrics collection
    group.bench_function("metrics_collection", |b| {
        b.iter(|| {
            let metrics = manager.get_performance_metrics();
            black_box(metrics)
        });
    });
    
    // Benchmark system status
    group.bench_function("system_status", |b| {
        b.iter(|| {
            let status = manager.get_system_status();
            black_box(status)
        });
    });
    
    group.finish();
}

/// Benchmark concurrent processing
fn bench_concurrent_processing(c: &mut Criterion) {
    let mut group = c.benchmark_group("concurrent_processing");
    
    // Test different levels of concurrency
    let thread_counts = vec![1, 2, 4, 8];
    let codes_per_thread = 10;
    
    for thread_count in thread_counts {
        group.bench_with_input(
            BenchmarkId::new("concurrent_threads", thread_count),
            &thread_count,
            |b, &thread_count| {
                b.iter(|| {
                    let handles: Vec<_> = (0..thread_count)
                        .map(|thread_id| {
                            std::thread::spawn(move || {
                                let strategy = DefaultStrategy::new();
                                let config = ProcessingConfig::default();
                                
                                let mut results = Vec::new();
                                for i in 0..codes_per_thread {
                                    let code = format!("fn thread{thread_id}_func{i}() {{ println!(\"Hello from thread {thread_id}, function {i}\"); }}");
                                    let result = strategy.process(&code, &config);
                                    results.push(result);
                                }
                                results
                            })
                        })
                        .collect();
                    
                    let results: Vec<_> = handles
                        .into_iter()
                        .map(|h| h.join().unwrap())
                        .collect();
                    
                    black_box(results)
                });
            },
        );
    }
    
    group.finish();
}

criterion_group!(
    benches,
    bench_single_processing,
    bench_batch_processing,
    bench_cache_performance,
    bench_memory_usage,
    bench_strategy_registry,
    bench_performance_manager,
    bench_concurrent_processing
);

criterion_main!(benches);