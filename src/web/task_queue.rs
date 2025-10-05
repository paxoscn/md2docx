//! Async task queue for handling conversion jobs

use crate::conversion::ConversionEngine;
use crate::error::ConversionError;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    sync::Arc,
};
use tokio::sync::{mpsc, RwLock};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

/// Task status enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TaskStatus {
    Pending,
    Processing,
    Completed,
    Failed,
}

/// Conversion task definition
#[derive(Debug, Clone)]
pub struct ConversionTask {
    pub id: String,
    pub markdown_content: String,
    pub config_yaml: Option<String>,
    pub filename: Option<String>,
    pub created_at: std::time::SystemTime,
}

/// Task result containing the converted docx data or error
#[derive(Debug, Clone)]
pub struct TaskResult {
    pub task_id: String,
    pub status: TaskStatus,
    pub result: Option<Vec<u8>>,
    pub error: Option<String>,
    pub completed_at: Option<std::time::SystemTime>,
}

/// Task queue for managing async conversion jobs
pub struct TaskQueue {
    sender: mpsc::UnboundedSender<ConversionTask>,
    results: Arc<RwLock<HashMap<String, TaskResult>>>,
    conversion_engine: Arc<ConversionEngine>,
}

impl TaskQueue {
    /// Create a new task queue with the specified number of workers
    pub fn new(conversion_engine: Arc<ConversionEngine>, worker_count: usize) -> Self {
        let (sender, mut receiver) = mpsc::unbounded_channel();
        let results = Arc::new(RwLock::new(HashMap::new()));
        
        // Create worker senders for task distribution
        
        // Spawn task distributor
        let worker_senders: Vec<mpsc::UnboundedSender<ConversionTask>> = (0..worker_count)
            .map(|_| {
                let (worker_sender, worker_receiver) = mpsc::unbounded_channel();
                let worker_results = results.clone();
                let worker_engine = conversion_engine.clone();
                
                tokio::spawn(async move {
                    Self::worker_loop(0, worker_receiver, worker_results, worker_engine).await;
                });
                
                worker_sender
            })
            .collect();
        
        let worker_senders = Arc::new(worker_senders);
        let worker_index = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        
        // Spawn distributor task
        {
            let worker_senders = worker_senders.clone();
            let worker_index = worker_index.clone();
            
            tokio::spawn(async move {
                while let Some(task) = receiver.recv().await {
                    let index = worker_index.fetch_add(1, std::sync::atomic::Ordering::Relaxed) % worker_senders.len();
                    if let Err(e) = worker_senders[index].send(task) {
                        error!("Failed to send task to worker {}: {}", index, e);
                    }
                }
            });
        }
        
        Self {
            sender,
            results,
            conversion_engine,
        }
    }

    /// Submit a new conversion task to the queue
    pub async fn submit_task(&self, task: ConversionTask) -> Result<String, ConversionError> {
        let task_id = task.id.clone();
        
        // Initialize task result as pending
        let task_result = TaskResult {
            task_id: task_id.clone(),
            status: TaskStatus::Pending,
            result: None,
            error: None,
            completed_at: None,
        };
        
        // Store initial result
        {
            let mut results = self.results.write().await;
            results.insert(task_id.clone(), task_result);
        }
        
        // Send task to workers
        self.sender.send(task).map_err(|e| {
            ConversionError::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to submit task: {}", e),
            ))
        })?;
        
        info!("Submitted task {} to queue", task_id);
        Ok(task_id)
    }

    /// Get the status and result of a task
    pub async fn get_task_result(&self, task_id: &str) -> Option<TaskResult> {
        let results = self.results.read().await;
        results.get(task_id).cloned()
    }

    /// Get all task results (for monitoring/debugging)
    pub async fn get_all_results(&self) -> HashMap<String, TaskResult> {
        let results = self.results.read().await;
        results.clone()
    }

    /// Clean up completed tasks older than the specified duration
    pub async fn cleanup_old_tasks(&self, max_age: std::time::Duration) {
        let mut results = self.results.write().await;
        let now = std::time::SystemTime::now();
        
        let mut to_remove = Vec::new();
        
        for (task_id, result) in results.iter() {
            if let Some(completed_at) = result.completed_at {
                if let Ok(age) = now.duration_since(completed_at) {
                    if age > max_age {
                        to_remove.push(task_id.clone());
                    }
                }
            }
        }
        
        for task_id in to_remove {
            results.remove(&task_id);
            debug!("Cleaned up old task: {}", task_id);
        }
    }

    /// Worker loop that processes tasks from the queue
    async fn worker_loop(
        worker_id: usize,
        mut receiver: mpsc::UnboundedReceiver<ConversionTask>,
        results: Arc<RwLock<HashMap<String, TaskResult>>>,
        engine: Arc<ConversionEngine>,
    ) {
        info!("Worker {} started", worker_id);
        
        while let Some(task) = receiver.recv().await {
            debug!("Worker {} processing task {}", worker_id, task.id);
            
            // Update task status to processing
            {
                let mut results_guard = results.write().await;
                if let Some(result) = results_guard.get_mut(&task.id) {
                    result.status = TaskStatus::Processing;
                }
            }
            
            // Process the task
            let task_result = Self::process_task(&task, &engine).await;
            
            // Update task result
            {
                let mut results_guard = results.write().await;
                results_guard.insert(task.id.clone(), task_result);
            }
            
            debug!("Worker {} completed task {}", worker_id, task.id);
        }
        
        warn!("Worker {} shutting down", worker_id);
    }

    /// Process a single conversion task
    async fn process_task(task: &ConversionTask, engine: &ConversionEngine) -> TaskResult {
        let start_time = std::time::Instant::now();
        
        match engine.convert(&task.markdown_content).await {
            Ok(docx_bytes) => {
                let duration = start_time.elapsed();
                info!(
                    "Task {} completed successfully in {:?}, generated {} bytes",
                    task.id,
                    duration,
                    docx_bytes.len()
                );
                
                TaskResult {
                    task_id: task.id.clone(),
                    status: TaskStatus::Completed,
                    result: Some(docx_bytes),
                    error: None,
                    completed_at: Some(std::time::SystemTime::now()),
                }
            }
            Err(e) => {
                let duration = start_time.elapsed();
                error!(
                    "Task {} failed after {:?}: {}",
                    task.id,
                    duration,
                    e
                );
                
                TaskResult {
                    task_id: task.id.clone(),
                    status: TaskStatus::Failed,
                    result: None,
                    error: Some(e.to_string()),
                    completed_at: Some(std::time::SystemTime::now()),
                }
            }
        }
    }
}

/// Helper function to create a new conversion task
pub fn create_conversion_task(
    markdown_content: String,
    config_yaml: Option<String>,
    filename: Option<String>,
) -> ConversionTask {
    ConversionTask {
        id: Uuid::new_v4().to_string(),
        markdown_content,
        config_yaml,
        filename,
        created_at: std::time::SystemTime::now(),
    }
}

/// Task queue manager that handles cleanup and monitoring
pub struct TaskQueueManager {
    task_queue: Arc<TaskQueue>,
    cleanup_interval: std::time::Duration,
    max_task_age: std::time::Duration,
}

impl TaskQueueManager {
    /// Create a new task queue manager
    pub fn new(
        conversion_engine: Arc<ConversionEngine>,
        worker_count: usize,
        cleanup_interval: std::time::Duration,
        max_task_age: std::time::Duration,
    ) -> Self {
        let task_queue = Arc::new(TaskQueue::new(conversion_engine, worker_count));
        
        Self {
            task_queue,
            cleanup_interval,
            max_task_age,
        }
    }

    /// Get a reference to the task queue
    pub fn queue(&self) -> Arc<TaskQueue> {
        self.task_queue.clone()
    }

    /// Start the cleanup task that runs periodically
    pub fn start_cleanup_task(&self) {
        let queue = self.task_queue.clone();
        let interval = self.cleanup_interval;
        let max_age = self.max_task_age;
        
        tokio::spawn(async move {
            let mut cleanup_timer = tokio::time::interval(interval);
            
            loop {
                cleanup_timer.tick().await;
                
                debug!("Running task cleanup");
                queue.cleanup_old_tasks(max_age).await;
            }
        });
        
        info!("Started task cleanup with interval {:?}, max age {:?}", interval, max_age);
    }

    /// Get queue statistics for monitoring
    pub async fn get_queue_stats(&self) -> QueueStats {
        let all_results = self.task_queue.get_all_results().await;
        
        let mut stats = QueueStats {
            total_tasks: all_results.len(),
            pending_tasks: 0,
            processing_tasks: 0,
            completed_tasks: 0,
            failed_tasks: 0,
        };
        
        for result in all_results.values() {
            match result.status {
                TaskStatus::Pending => stats.pending_tasks += 1,
                TaskStatus::Processing => stats.processing_tasks += 1,
                TaskStatus::Completed => stats.completed_tasks += 1,
                TaskStatus::Failed => stats.failed_tasks += 1,
            }
        }
        
        stats
    }
}

/// Queue statistics for monitoring
#[derive(Debug, Clone, Serialize)]
pub struct QueueStats {
    pub total_tasks: usize,
    pub pending_tasks: usize,
    pub processing_tasks: usize,
    pub completed_tasks: usize,
    pub failed_tasks: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::ConversionConfig;

    fn create_test_engine() -> Arc<ConversionEngine> {
        let config = ConversionConfig::default();
        Arc::new(ConversionEngine::new(config))
    }

    #[tokio::test]
    async fn test_task_creation() {
        let task = create_conversion_task(
            "# Test".to_string(),
            None,
            Some("test.docx".to_string()),
        );
        
        assert!(!task.id.is_empty());
        assert_eq!(task.markdown_content, "# Test");
        assert_eq!(task.filename, Some("test.docx".to_string()));
    }

    #[tokio::test]
    async fn test_task_queue_creation() {
        let engine = create_test_engine();
        let queue = TaskQueue::new(engine, 2);
        
        // Queue should be created successfully
        let stats = queue.get_all_results().await;
        assert!(stats.is_empty());
    }

    #[tokio::test]
    async fn test_task_submission() {
        let engine = create_test_engine();
        let queue = TaskQueue::new(engine, 1);
        
        let task = create_conversion_task(
            "# Test Document".to_string(),
            None,
            None,
        );
        
        let task_id = queue.submit_task(task).await.unwrap();
        assert!(!task_id.is_empty());
        
        // Task should be in results
        let result = queue.get_task_result(&task_id).await;
        assert!(result.is_some());
        
        let result = result.unwrap();
        assert_eq!(result.task_id, task_id);
        // Status should be pending or processing
        assert!(matches!(result.status, TaskStatus::Pending | TaskStatus::Processing));
    }

    #[tokio::test]
    async fn test_task_processing() {
        let engine = create_test_engine();
        let queue = TaskQueue::new(engine, 1);
        
        let task = create_conversion_task(
            "# Test Document\n\nThis is a test.".to_string(),
            None,
            None,
        );
        
        let task_id = queue.submit_task(task).await.unwrap();
        
        // Wait for processing to complete
        let mut attempts = 0;
        let max_attempts = 50; // 5 seconds with 100ms intervals
        
        loop {
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
            
            if let Some(result) = queue.get_task_result(&task_id).await {
                if matches!(result.status, TaskStatus::Completed | TaskStatus::Failed) {
                    // Task completed
                    assert_eq!(result.status, TaskStatus::Completed);
                    assert!(result.result.is_some());
                    assert!(result.error.is_none());
                    break;
                }
            }
            
            attempts += 1;
            if attempts >= max_attempts {
                panic!("Task did not complete within expected time");
            }
        }
    }

    #[tokio::test]
    async fn test_task_queue_manager() {
        let engine = create_test_engine();
        let manager = TaskQueueManager::new(
            engine,
            2,
            std::time::Duration::from_secs(60),
            std::time::Duration::from_secs(300),
        );
        
        let stats = manager.get_queue_stats().await;
        assert_eq!(stats.total_tasks, 0);
        assert_eq!(stats.pending_tasks, 0);
        assert_eq!(stats.processing_tasks, 0);
        assert_eq!(stats.completed_tasks, 0);
        assert_eq!(stats.failed_tasks, 0);
    }

    #[tokio::test]
    async fn test_cleanup_old_tasks() {
        let engine = create_test_engine();
        let queue = TaskQueue::new(engine, 1);
        
        // Submit a task
        let task = create_conversion_task("# Test".to_string(), None, None);
        let task_id = queue.submit_task(task).await.unwrap();
        
        // Wait for completion
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        
        // Manually set completion time to past
        {
            let mut results = queue.results.write().await;
            if let Some(result) = results.get_mut(&task_id) {
                result.completed_at = Some(
                    std::time::SystemTime::now() - std::time::Duration::from_secs(3600)
                );
            }
        }
        
        // Cleanup tasks older than 1 hour
        queue.cleanup_old_tasks(std::time::Duration::from_secs(1800)).await;
        
        // Task should be removed
        let result = queue.get_task_result(&task_id).await;
        assert!(result.is_none());
    }
}