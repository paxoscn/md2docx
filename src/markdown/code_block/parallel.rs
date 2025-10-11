//! Parallel processing support for code block processing
//! 
//! This module provides optional parallel processing capabilities to improve
//! performance when processing multiple code blocks simultaneously.

use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use std::collections::VecDeque;

use crate::markdown::code_block::{
    CodeBlockStrategy, ProcessedCodeBlock, ProcessingConfig, ProcessingError
};

/// Configuration for parallel processing
#[derive(Debug, Clone)]
pub struct ParallelConfig {
    /// Maximum number of worker threads
    pub max_workers: usize,
    /// Maximum number of tasks in the queue
    pub max_queue_size: usize,
    /// Timeout for individual tasks
    pub task_timeout: Duration,
    /// Whether to enable work stealing between threads
    pub enable_work_stealing: bool,
    /// Minimum batch size for parallel processing
    pub min_batch_size: usize,
    /// Whether to enable parallel processing statistics
    pub enable_statistics: bool,
}

impl Default for ParallelConfig {
    fn default() -> Self {
        Self {
            max_workers: num_cpus::get().min(8), // Limit to 8 threads max
            max_queue_size: 1000,
            task_timeout: Duration::from_secs(30),
            enable_work_stealing: true,
            min_batch_size: 2,
            enable_statistics: true,
        }
    }
}

/// A task to be processed in parallel
pub struct ProcessingTask {
    /// Unique identifier for this task
    pub id: u64,
    /// The strategy to use for processing
    pub strategy: Arc<dyn CodeBlockStrategy>,
    /// The code to process
    pub code: String,
    /// Processing configuration
    pub config: ProcessingConfig,
    /// Optional language hint
    pub language: Option<String>,
    /// When this task was created
    pub created_at: Instant,
    /// Priority of this task (higher = more important)
    pub priority: u8,
}

impl std::fmt::Debug for ProcessingTask {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ProcessingTask")
            .field("id", &self.id)
            .field("strategy", &self.strategy.get_language_name())
            .field("code_length", &self.code.len())
            .field("language", &self.language)
            .field("created_at", &self.created_at)
            .field("priority", &self.priority)
            .finish()
    }
}

impl ProcessingTask {
    /// Create a new processing task
    pub fn new(
        id: u64,
        strategy: Arc<dyn CodeBlockStrategy>,
        code: String,
        config: ProcessingConfig,
        language: Option<String>,
    ) -> Self {
        Self {
            id,
            strategy,
            code,
            config,
            language,
            created_at: Instant::now(),
            priority: 100, // Default priority
        }
    }

    /// Create a task with custom priority
    pub fn with_priority(mut self, priority: u8) -> Self {
        self.priority = priority;
        self
    }

    /// Get the age of this task
    pub fn age(&self) -> Duration {
        self.created_at.elapsed()
    }

    /// Check if this task has timed out
    pub fn is_timed_out(&self, timeout: Duration) -> bool {
        self.age() > timeout
    }
}

/// Result of processing a task
#[derive(Debug)]
pub struct TaskResult {
    /// The task ID
    pub task_id: u64,
    /// The processing result
    pub result: Result<ProcessedCodeBlock, ProcessingError>,
    /// Time taken to process
    pub processing_time: Duration,
    /// Which worker processed this task
    pub worker_id: usize,
}

impl TaskResult {
    /// Check if the task was successful
    pub fn is_successful(&self) -> bool {
        self.result.is_ok()
    }

    /// Get the processed code block if successful
    pub fn get_processed_block(&self) -> Option<&ProcessedCodeBlock> {
        self.result.as_ref().ok()
    }

    /// Get the error if failed
    pub fn get_error(&self) -> Option<&ProcessingError> {
        self.result.as_ref().err()
    }
}

/// Statistics about parallel processing performance
#[derive(Debug, Clone, Default)]
pub struct ParallelStatistics {
    /// Total tasks processed
    pub tasks_processed: u64,
    /// Total tasks failed
    pub tasks_failed: u64,
    /// Total tasks timed out
    pub tasks_timed_out: u64,
    /// Total processing time across all workers
    pub total_processing_time: Duration,
    /// Number of active workers
    pub active_workers: usize,
    /// Current queue size
    pub queue_size: usize,
    /// Peak queue size
    pub peak_queue_size: usize,
    /// Total time workers spent idle
    pub total_idle_time: Duration,
    /// Number of work stealing events
    pub work_stealing_events: u64,
}

impl ParallelStatistics {
    /// Calculate average processing time per task
    pub fn average_processing_time(&self) -> Duration {
        if self.tasks_processed == 0 {
            Duration::from_millis(0)
        } else {
            self.total_processing_time / self.tasks_processed as u32
        }
    }

    /// Calculate success rate
    pub fn success_rate(&self) -> f64 {
        let total = self.tasks_processed + self.tasks_failed;
        if total == 0 {
            0.0
        } else {
            self.tasks_processed as f64 / total as f64
        }
    }

    /// Calculate throughput (tasks per second)
    pub fn throughput(&self, elapsed_time: Duration) -> f64 {
        if elapsed_time.as_secs_f64() == 0.0 {
            0.0
        } else {
            self.tasks_processed as f64 / elapsed_time.as_secs_f64()
        }
    }

    /// Check if the system is overloaded
    pub fn is_overloaded(&self, max_queue_size: usize) -> bool {
        self.queue_size as f64 / max_queue_size as f64 > 0.8
    }
}

/// A thread-safe work queue for parallel processing
struct WorkQueue {
    tasks: Mutex<VecDeque<ProcessingTask>>,
    max_size: usize,
}

impl WorkQueue {
    fn new(max_size: usize) -> Self {
        Self {
            tasks: Mutex::new(VecDeque::new()),
            max_size,
        }
    }

    /// Add a task to the queue
    fn push(&self, task: ProcessingTask) -> Result<(), ProcessingTask> {
        if let Ok(mut queue) = self.tasks.lock() {
            if queue.len() >= self.max_size {
                return Err(task);
            }
            
            // Insert based on priority (higher priority first)
            let insert_pos = queue
                .iter()
                .position(|t| t.priority < task.priority)
                .unwrap_or(queue.len());
            
            queue.insert(insert_pos, task);
            Ok(())
        } else {
            Err(task)
        }
    }

    /// Pop a task from the queue
    fn pop(&self) -> Option<ProcessingTask> {
        if let Ok(mut queue) = self.tasks.lock() {
            queue.pop_front()
        } else {
            None
        }
    }

    /// Get current queue size
    fn size(&self) -> usize {
        if let Ok(queue) = self.tasks.lock() {
            queue.len()
        } else {
            0
        }
    }

    /// Check if queue is empty
    fn is_empty(&self) -> bool {
        self.size() == 0
    }

    /// Clear all tasks from the queue
    fn clear(&self) -> Vec<ProcessingTask> {
        if let Ok(mut queue) = self.tasks.lock() {
            queue.drain(..).collect()
        } else {
            Vec::new()
        }
    }

    /// Steal work from this queue (for work stealing)
    fn steal_work(&self, count: usize) -> Vec<ProcessingTask> {
        if let Ok(mut queue) = self.tasks.lock() {
            let steal_count = count.min(queue.len() / 2); // Steal at most half
            let len = queue.len();
            queue.drain(len - steal_count..).collect()
        } else {
            Vec::new()
        }
    }
}

/// Parallel processor for code blocks
pub struct ParallelProcessor {
    config: ParallelConfig,
    work_queue: Arc<WorkQueue>,
    statistics: Arc<Mutex<ParallelStatistics>>,
    next_task_id: Arc<Mutex<u64>>,
    is_running: Arc<Mutex<bool>>,
    worker_handles: Vec<std::thread::JoinHandle<()>>,
}

impl ParallelProcessor {
    /// Create a new parallel processor
    pub fn new() -> Self {
        Self::with_config(ParallelConfig::default())
    }

    /// Create a parallel processor with custom configuration
    pub fn with_config(config: ParallelConfig) -> Self {
        let work_queue = Arc::new(WorkQueue::new(config.max_queue_size));
        let statistics = Arc::new(Mutex::new(ParallelStatistics::default()));
        let next_task_id = Arc::new(Mutex::new(0));
        let is_running = Arc::new(Mutex::new(false));

        Self {
            config,
            work_queue,
            statistics,
            next_task_id,
            is_running,
            worker_handles: Vec::new(),
        }
    }

    /// Start the parallel processor
    pub fn start(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if let Ok(mut running) = self.is_running.lock() {
            if *running {
                return Err("Processor is already running".into());
            }
            *running = true;
        }

        // Start worker threads
        for worker_id in 0..self.config.max_workers {
            let work_queue = self.work_queue.clone();
            let statistics = self.statistics.clone();
            let is_running = self.is_running.clone();
            let config = self.config.clone();

            let handle = std::thread::spawn(move || {
                Self::worker_loop(worker_id, work_queue, statistics, is_running, config);
            });

            self.worker_handles.push(handle);
        }

        Ok(())
    }

    /// Stop the parallel processor
    pub fn stop(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Signal workers to stop
        if let Ok(mut running) = self.is_running.lock() {
            *running = false;
        }

        // Wait for all workers to finish
        while let Some(handle) = self.worker_handles.pop() {
            if let Err(e) = handle.join() {
                eprintln!("Worker thread panicked: {:?}", e);
            }
        }

        Ok(())
    }

    /// Submit a task for parallel processing
    pub fn submit_task(
        &self,
        strategy: Arc<dyn CodeBlockStrategy>,
        code: String,
        config: ProcessingConfig,
        language: Option<String>,
    ) -> Result<u64, Box<dyn std::error::Error + Send + Sync>> {
        let task_id = self.get_next_task_id();
        let task = ProcessingTask::new(task_id, strategy, code, config, language);

        match self.work_queue.push(task) {
            Ok(()) => {
                // Update statistics
                if self.config.enable_statistics {
                    if let Ok(mut stats) = self.statistics.lock() {
                        stats.queue_size = self.work_queue.size();
                        stats.peak_queue_size = stats.peak_queue_size.max(stats.queue_size);
                    }
                }
                Ok(task_id)
            }
            Err(_) => Err("Work queue is full".into()),
        }
    }

    /// Submit multiple tasks as a batch
    pub fn submit_batch(
        &self,
        tasks: Vec<(Arc<dyn CodeBlockStrategy>, String, ProcessingConfig, Option<String>)>,
    ) -> Result<Vec<u64>, Box<dyn std::error::Error + Send + Sync>> {
        let mut task_ids = Vec::new();

        for (strategy, code, config, language) in tasks {
            let task_id = self.submit_task(strategy, code, config, language)?;
            task_ids.push(task_id);
        }

        Ok(task_ids)
    }

    /// Process tasks sequentially (fallback when parallel processing is not beneficial)
    pub fn process_sequential(
        &self,
        tasks: Vec<(Arc<dyn CodeBlockStrategy>, String, ProcessingConfig, Option<String>)>,
    ) -> Vec<TaskResult> {
        tasks
            .into_iter()
            .enumerate()
            .map(|(index, (strategy, code, config, _language))| {
                let start_time = Instant::now();
                let result = strategy.process(&code, &config);
                let processing_time = start_time.elapsed();

                TaskResult {
                    task_id: index as u64,
                    result,
                    processing_time,
                    worker_id: 0, // Sequential processing uses worker 0
                }
            })
            .collect()
    }

    /// Decide whether to use parallel or sequential processing
    pub fn process_adaptive(
        &self,
        tasks: Vec<(Arc<dyn CodeBlockStrategy>, String, ProcessingConfig, Option<String>)>,
    ) -> Result<Vec<TaskResult>, Box<dyn std::error::Error + Send + Sync>> {
        // Use sequential processing if batch is too small or processor is not running
        if tasks.len() < self.config.min_batch_size || !self.is_running() {
            return Ok(self.process_sequential(tasks));
        }

        // Check if system is overloaded
        if self.is_overloaded() {
            return Ok(self.process_sequential(tasks));
        }

        // Use parallel processing
        let task_ids = self.submit_batch(tasks)?;
        
        // For this simple implementation, we'll return empty results
        // In a real implementation, you'd have a way to collect results
        Ok(task_ids
            .into_iter()
            .map(|task_id| TaskResult {
                task_id,
                result: Err(ProcessingError::timeout()),
                processing_time: Duration::from_millis(0),
                worker_id: 0,
            })
            .collect())
    }

    /// Check if the processor is running
    pub fn is_running(&self) -> bool {
        if let Ok(running) = self.is_running.lock() {
            *running
        } else {
            false
        }
    }

    /// Check if the system is overloaded
    pub fn is_overloaded(&self) -> bool {
        if let Ok(stats) = self.statistics.lock() {
            stats.is_overloaded(self.config.max_queue_size)
        } else {
            false
        }
    }

    /// Get current statistics
    pub fn get_statistics(&self) -> ParallelStatistics {
        if let Ok(stats) = self.statistics.lock() {
            let mut stats_copy = stats.clone();
            stats_copy.queue_size = self.work_queue.size();
            stats_copy
        } else {
            ParallelStatistics::default()
        }
    }

    /// Clear all pending tasks
    pub fn clear_queue(&self) -> Vec<ProcessingTask> {
        self.work_queue.clear()
    }

    /// Get the next task ID
    fn get_next_task_id(&self) -> u64 {
        if let Ok(mut id) = self.next_task_id.lock() {
            *id += 1;
            *id
        } else {
            0
        }
    }

    /// Worker thread main loop
    fn worker_loop(
        _worker_id: usize,
        work_queue: Arc<WorkQueue>,
        statistics: Arc<Mutex<ParallelStatistics>>,
        is_running: Arc<Mutex<bool>>,
        config: ParallelConfig,
    ) {
        let mut idle_start: Option<Instant> = None;

        loop {
            // Check if we should stop
            if let Ok(running) = is_running.lock() {
                if !*running {
                    break;
                }
            }

            // Try to get a task
            if let Some(task) = work_queue.pop() {
                // End idle time tracking
                if let Some(idle_time_start) = idle_start.take() {
                    let idle_duration = idle_time_start.elapsed();
                    if config.enable_statistics {
                        if let Ok(mut stats) = statistics.lock() {
                            stats.total_idle_time += idle_duration;
                        }
                    }
                }

                // Process the task
                let start_time = Instant::now();
                let result = if task.is_timed_out(config.task_timeout) {
                    Err(ProcessingError::timeout())
                } else {
                    task.strategy.process(&task.code, &task.config)
                };
                let processing_time = start_time.elapsed();

                // Update statistics
                if config.enable_statistics {
                    if let Ok(mut stats) = statistics.lock() {
                        if result.is_ok() {
                            stats.tasks_processed += 1;
                        } else {
                            stats.tasks_failed += 1;
                        }
                        stats.total_processing_time += processing_time;
                        stats.active_workers = config.max_workers; // Simplified
                    }
                }

                // In a real implementation, you'd send the result somewhere
                // For now, we just drop it
            } else {
                // No task available, start idle time tracking
                if idle_start.is_none() {
                    idle_start = Some(Instant::now());
                }

                // Sleep briefly to avoid busy waiting
                std::thread::sleep(Duration::from_millis(10));

                // Try work stealing if enabled
                if config.enable_work_stealing {
                    // In a real implementation, you'd try to steal work from other workers
                    // This is a simplified version
                }
            }
        }
    }
}

impl Drop for ParallelProcessor {
    fn drop(&mut self) {
        let _ = self.stop();
    }
}

/// Utility functions for parallel processing
pub mod utils {
    use super::*;

    /// Estimate the optimal number of workers for a given workload
    pub fn estimate_optimal_workers(
        task_count: usize,
        average_task_duration: Duration,
        target_completion_time: Duration,
    ) -> usize {
        if average_task_duration.is_zero() || target_completion_time.is_zero() {
            return 1;
        }

        let sequential_time = average_task_duration * task_count as u32;
        let required_parallelism = sequential_time.as_secs_f64() / target_completion_time.as_secs_f64();
        
        let optimal_workers = required_parallelism.ceil() as usize;
        
        // Limit to reasonable bounds
        optimal_workers.max(1).min(num_cpus::get() * 2)
    }

    /// Calculate the overhead of parallel processing
    pub fn calculate_parallel_overhead(
        sequential_time: Duration,
        parallel_time: Duration,
        worker_count: usize,
    ) -> f64 {
        if sequential_time.is_zero() {
            return 0.0;
        }

        let theoretical_parallel_time = sequential_time.as_secs_f64() / worker_count as f64;
        let actual_overhead = parallel_time.as_secs_f64() - theoretical_parallel_time;
        
        actual_overhead / sequential_time.as_secs_f64()
    }

    /// Determine if parallel processing is beneficial for a given workload
    pub fn should_use_parallel_processing(
        task_count: usize,
        estimated_task_duration: Duration,
        parallel_overhead: Duration,
    ) -> bool {
        if task_count < 2 {
            return false;
        }

        let sequential_time = estimated_task_duration * task_count as u32;
        let parallel_time = estimated_task_duration + parallel_overhead;
        
        parallel_time < sequential_time
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::markdown::code_block::strategy::DefaultStrategy;

    #[test]
    fn test_parallel_config_default() {
        let config = ParallelConfig::default();
        assert!(config.max_workers > 0);
        assert!(config.max_queue_size > 0);
        assert!(config.task_timeout > Duration::from_millis(0));
    }

    #[test]
    fn test_processing_task_creation() {
        let strategy = Arc::new(DefaultStrategy::new());
        let config = ProcessingConfig::default();
        let task = ProcessingTask::new(
            1,
            strategy,
            "fn main() {}".to_string(),
            config,
            Some("rust".to_string()),
        );

        assert_eq!(task.id, 1);
        assert_eq!(task.code, "fn main() {}");
        assert_eq!(task.language, Some("rust".to_string()));
        assert_eq!(task.priority, 100);
    }

    #[test]
    fn test_task_with_priority() {
        let strategy = Arc::new(DefaultStrategy::new());
        let config = ProcessingConfig::default();
        let task = ProcessingTask::new(1, strategy, "code".to_string(), config, None)
            .with_priority(200);

        assert_eq!(task.priority, 200);
    }

    #[test]
    fn test_work_queue_operations() {
        let queue = WorkQueue::new(10);
        
        assert!(queue.is_empty());
        assert_eq!(queue.size(), 0);

        let strategy = Arc::new(DefaultStrategy::new());
        let config = ProcessingConfig::default();
        let task = ProcessingTask::new(1, strategy, "code".to_string(), config, None);

        assert!(queue.push(task).is_ok());
        assert_eq!(queue.size(), 1);
        assert!(!queue.is_empty());

        let popped = queue.pop();
        assert!(popped.is_some());
        assert_eq!(popped.unwrap().id, 1);
        assert!(queue.is_empty());
    }

    #[test]
    fn test_work_queue_priority() {
        let queue = WorkQueue::new(10);
        let strategy = Arc::new(DefaultStrategy::new());
        let config = ProcessingConfig::default();

        // Add tasks with different priorities
        let low_priority = ProcessingTask::new(1, strategy.clone(), "code1".to_string(), config.clone(), None)
            .with_priority(50);
        let high_priority = ProcessingTask::new(2, strategy.clone(), "code2".to_string(), config.clone(), None)
            .with_priority(150);

        queue.push(low_priority).unwrap();
        queue.push(high_priority).unwrap();

        // High priority task should come out first
        let first = queue.pop().unwrap();
        assert_eq!(first.id, 2);
        assert_eq!(first.priority, 150);

        let second = queue.pop().unwrap();
        assert_eq!(second.id, 1);
        assert_eq!(second.priority, 50);
    }

    #[test]
    fn test_work_queue_capacity() {
        let queue = WorkQueue::new(2);
        let strategy = Arc::new(DefaultStrategy::new());
        let config = ProcessingConfig::default();

        // Fill the queue
        let task1 = ProcessingTask::new(1, strategy.clone(), "code1".to_string(), config.clone(), None);
        let task2 = ProcessingTask::new(2, strategy.clone(), "code2".to_string(), config.clone(), None);
        let task3 = ProcessingTask::new(3, strategy.clone(), "code3".to_string(), config.clone(), None);

        assert!(queue.push(task1).is_ok());
        assert!(queue.push(task2).is_ok());
        
        // Third task should be rejected
        assert!(queue.push(task3).is_err());
    }

    #[test]
    fn test_parallel_processor_creation() {
        let processor = ParallelProcessor::new();
        assert!(!processor.is_running());
        assert!(!processor.is_overloaded());
    }

    #[test]
    fn test_parallel_statistics() {
        let stats = ParallelStatistics::default();
        assert_eq!(stats.tasks_processed, 0);
        assert_eq!(stats.tasks_failed, 0);
        assert_eq!(stats.average_processing_time(), Duration::from_millis(0));
        assert_eq!(stats.success_rate(), 0.0);
    }

    #[test]
    fn test_statistics_calculations() {
        let mut stats = ParallelStatistics::default();
        stats.tasks_processed = 8;
        stats.tasks_failed = 2;
        stats.total_processing_time = Duration::from_secs(10);

        assert_eq!(stats.success_rate(), 0.8);
        assert_eq!(stats.average_processing_time(), Duration::from_millis(1250));
        
        let throughput = stats.throughput(Duration::from_secs(5));
        assert_eq!(throughput, 1.6); // 8 tasks / 5 seconds
    }

    #[test]
    fn test_task_result() {
        let strategy = Arc::new(DefaultStrategy::new());
        let config = ProcessingConfig::default();
        let processed_block = ProcessedCodeBlock::new("code".to_string(), Some("rust".to_string()));
        
        let success_result = TaskResult {
            task_id: 1,
            result: Ok(processed_block),
            processing_time: Duration::from_millis(100),
            worker_id: 0,
        };

        assert!(success_result.is_successful());
        assert!(success_result.get_processed_block().is_some());
        assert!(success_result.get_error().is_none());

        let error_result = TaskResult {
            task_id: 2,
            result: Err(ProcessingError::syntax_error("Test error", None, None)),
            processing_time: Duration::from_millis(50),
            worker_id: 1,
        };

        assert!(!error_result.is_successful());
        assert!(error_result.get_processed_block().is_none());
        assert!(error_result.get_error().is_some());
    }

    #[test]
    fn test_utils_optimal_workers() {
        let optimal = utils::estimate_optimal_workers(
            100, // 100 tasks
            Duration::from_millis(100), // 100ms per task
            Duration::from_secs(2), // Want to complete in 2 seconds
        );

        // 100 tasks * 100ms = 10 seconds sequential
        // To complete in 2 seconds, need 10/2 = 5 workers
        assert_eq!(optimal, 5);
    }

    #[test]
    fn test_utils_parallel_overhead() {
        let overhead = utils::calculate_parallel_overhead(
            Duration::from_secs(10), // Sequential time
            Duration::from_secs(3),  // Actual parallel time
            4, // 4 workers
        );

        // Theoretical parallel time: 10/4 = 2.5 seconds
        // Actual parallel time: 3 seconds
        // Overhead: (3 - 2.5) / 10 = 0.05 = 5%
        assert!((overhead - 0.05).abs() < 0.01);
    }

    #[test]
    fn test_utils_should_use_parallel() {
        // Case where parallel is beneficial
        assert!(utils::should_use_parallel_processing(
            10, // 10 tasks
            Duration::from_millis(100), // 100ms per task
            Duration::from_millis(50), // 50ms overhead
        ));

        // Case where parallel is not beneficial (high overhead)
        assert!(!utils::should_use_parallel_processing(
            2, // 2 tasks
            Duration::from_millis(100), // 100ms per task
            Duration::from_millis(500), // 500ms overhead
        ));

        // Case with single task
        assert!(!utils::should_use_parallel_processing(
            1, // 1 task
            Duration::from_millis(100),
            Duration::from_millis(10),
        ));
    }
}

// Add num_cpus dependency for CPU detection
// This would normally be in Cargo.toml:
// num_cpus = "1.0"
// For testing purposes, we'll provide a simple fallback
#[cfg(not(feature = "num_cpus"))]
mod num_cpus {
    pub fn get() -> usize {
        4 // Default fallback
    }
}