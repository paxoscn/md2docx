//! Timeout and error handling mechanisms for code block processing

use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Instant;
use std::sync::mpsc::{self, Receiver, Sender};

use crate::markdown::code_block::{
    CodeBlockStrategy, ProcessedCodeBlock, ProcessingConfig, ProcessingError, ProcessingMetadata
};

/// Result of a timeout operation
#[derive(Debug)]
pub enum TimeoutResult<T> {
    /// Operation completed successfully within timeout
    Success(T),
    /// Operation timed out
    Timeout,
    /// Operation failed with an error
    Error(ProcessingError),
}

/// Processor wrapper that adds timeout and error handling capabilities
pub struct TimeoutProcessor {
    fallback_strategy: Arc<dyn CodeBlockStrategy>,
    max_concurrent_operations: usize,
    active_operations: Arc<Mutex<usize>>,
}

impl TimeoutProcessor {
    /// Create a new timeout processor
    pub fn new(fallback_strategy: Arc<dyn CodeBlockStrategy>) -> Self {
        Self {
            fallback_strategy,
            max_concurrent_operations: 10, // Default limit
            active_operations: Arc::new(Mutex::new(0)),
        }
    }

    /// Create a timeout processor with custom concurrency limit
    pub fn with_concurrency_limit(
        fallback_strategy: Arc<dyn CodeBlockStrategy>,
        max_concurrent: usize,
    ) -> Self {
        Self {
            fallback_strategy,
            max_concurrent_operations: max_concurrent,
            active_operations: Arc::new(Mutex::new(0)),
        }
    }

    /// Process code with timeout and error handling
    pub fn process_with_timeout(
        &self,
        strategy: Arc<dyn CodeBlockStrategy>,
        code: &str,
        config: &ProcessingConfig,
    ) -> ProcessedCodeBlock {
        let start_time = Instant::now();
        
        // Check if we can start a new operation
        if !self.try_acquire_operation_slot() {
            return self.create_overload_error_block(code, config, start_time);
        }

        let result = self.execute_with_timeout(strategy.clone(), code, config);
        
        // Release the operation slot
        self.release_operation_slot();

        match result {
            TimeoutResult::Success(block) => block,
            TimeoutResult::Timeout => {
                self.handle_timeout_fallback(code, config, start_time)
            }
            TimeoutResult::Error(error) => {
                self.handle_error_fallback(code, config, error, start_time)
            }
        }
    }

    /// Execute processing with timeout using threads
    fn execute_with_timeout(
        &self,
        strategy: Arc<dyn CodeBlockStrategy>,
        code: &str,
        config: &ProcessingConfig,
    ) -> TimeoutResult<ProcessedCodeBlock> {
        let timeout = config.timeout_duration();
        let (tx, rx): (Sender<Result<ProcessedCodeBlock, ProcessingError>>, Receiver<Result<ProcessedCodeBlock, ProcessingError>>) = mpsc::channel();
        
        let code_owned = code.to_string();
        let config_owned = config.clone();
        
        // Spawn processing thread
        let handle = thread::spawn(move || {
            let result = strategy.process(&code_owned, &config_owned);
            let _ = tx.send(result); // Ignore send errors (receiver might have timed out)
        });

        // Wait for result or timeout
        match rx.recv_timeout(timeout) {
            Ok(Ok(block)) => {
                // Join the thread to clean up
                let _ = handle.join();
                TimeoutResult::Success(block)
            }
            Ok(Err(error)) => {
                // Join the thread to clean up
                let _ = handle.join();
                TimeoutResult::Error(error)
            }
            Err(_) => {
                // Timeout occurred - we can't safely kill the thread, but we can abandon it
                // The thread will complete eventually and the channel send will fail
                TimeoutResult::Timeout
            }
        }
    }

    /// Handle timeout by falling back to default strategy
    fn handle_timeout_fallback(
        &self,
        code: &str,
        config: &ProcessingConfig,
        start_time: Instant,
    ) -> ProcessedCodeBlock {
        let timeout_error = ProcessingError::timeout()
            .with_severity(crate::markdown::code_block::ErrorSeverity::Critical);

        // Try fallback strategy with a shorter timeout
        let fallback_config = ProcessingConfig {
            timeout_ms: 1000, // 1 second for fallback
            ..config.clone()
        };

        match self.fallback_strategy.process(code, &fallback_config) {
            Ok(mut block) => {
                block.errors.push(timeout_error);
                block.metadata.processing_time = start_time.elapsed();
                block
            }
            Err(_) => {
                // Even fallback failed, create minimal block
                self.create_minimal_error_block(code, config, timeout_error, start_time)
            }
        }
    }

    /// Handle processing error by falling back to default strategy
    fn handle_error_fallback(
        &self,
        code: &str,
        config: &ProcessingConfig,
        original_error: ProcessingError,
        start_time: Instant,
    ) -> ProcessedCodeBlock {
        // Try fallback strategy
        match self.fallback_strategy.process(code, config) {
            Ok(mut block) => {
                block.errors.push(original_error);
                block.metadata.processing_time = start_time.elapsed();
                block
            }
            Err(_) => {
                // Even fallback failed, create minimal block
                self.create_minimal_error_block(code, config, original_error, start_time)
            }
        }
    }

    /// Create a minimal error block when all processing fails
    fn create_minimal_error_block(
        &self,
        code: &str,
        config: &ProcessingConfig,
        error: ProcessingError,
        start_time: Instant,
    ) -> ProcessedCodeBlock {
        let language = config.custom_options.get("language").cloned();
        
        let metadata = ProcessingMetadata::new("timeout-processor-1.0.0")
            .with_processing_time(start_time.elapsed());

        ProcessedCodeBlock::new(code.to_string(), language)
            .with_metadata(metadata)
            .with_error(error)
            .with_validation(false) // Mark as not validated due to failure
    }

    /// Create an error block for system overload
    fn create_overload_error_block(
        &self,
        code: &str,
        config: &ProcessingConfig,
        start_time: Instant,
    ) -> ProcessedCodeBlock {
        let overload_error = ProcessingError::new(
            "system_overload",
            "Too many concurrent processing operations, request rejected"
        ).with_severity(crate::markdown::code_block::ErrorSeverity::High);

        self.create_minimal_error_block(code, config, overload_error, start_time)
    }

    /// Try to acquire a slot for a new operation
    fn try_acquire_operation_slot(&self) -> bool {
        if let Ok(mut count) = self.active_operations.lock() {
            if *count < self.max_concurrent_operations {
                *count += 1;
                true
            } else {
                false
            }
        } else {
            false // Lock poisoned, reject operation
        }
    }

    /// Release an operation slot
    fn release_operation_slot(&self) {
        if let Ok(mut count) = self.active_operations.lock() {
            if *count > 0 {
                *count -= 1;
            }
        }
    }

    /// Get current number of active operations
    pub fn get_active_operations(&self) -> usize {
        self.active_operations.lock().map(|guard| *guard).unwrap_or(0)
    }

    /// Get maximum concurrent operations limit
    pub fn get_max_concurrent_operations(&self) -> usize {
        self.max_concurrent_operations
    }

    /// Check if the processor is currently overloaded
    pub fn is_overloaded(&self) -> bool {
        self.get_active_operations() >= self.max_concurrent_operations
    }
}

/// Error recovery strategies
#[derive(Debug, Clone)]
pub enum RecoveryStrategy {
    /// Use fallback strategy
    Fallback,
    /// Return original code with error
    ReturnOriginal,
    /// Retry with different configuration
    RetryWithConfig(ProcessingConfig),
    /// Skip processing entirely
    Skip,
}

/// Error recovery manager
pub struct ErrorRecoveryManager {
    recovery_strategies: std::collections::HashMap<String, RecoveryStrategy>,
    max_retry_attempts: usize,
}

impl ErrorRecoveryManager {
    /// Create a new error recovery manager
    pub fn new() -> Self {
        let mut strategies = std::collections::HashMap::new();
        
        // Default recovery strategies for different error types
        strategies.insert("timeout".to_string(), RecoveryStrategy::Fallback);
        strategies.insert("syntax_error".to_string(), RecoveryStrategy::ReturnOriginal);
        strategies.insert("formatting_error".to_string(), RecoveryStrategy::ReturnOriginal);
        strategies.insert("validation_error".to_string(), RecoveryStrategy::ReturnOriginal);
        strategies.insert("system_overload".to_string(), RecoveryStrategy::Skip);
        
        Self {
            recovery_strategies: strategies,
            max_retry_attempts: 3,
        }
    }

    /// Get recovery strategy for an error type
    pub fn get_recovery_strategy(&self, error_type: &str) -> RecoveryStrategy {
        self.recovery_strategies
            .get(error_type)
            .cloned()
            .unwrap_or(RecoveryStrategy::ReturnOriginal)
    }

    /// Set recovery strategy for an error type
    pub fn set_recovery_strategy(&mut self, error_type: &str, strategy: RecoveryStrategy) {
        self.recovery_strategies.insert(error_type.to_string(), strategy);
    }

    /// Apply recovery strategy to a failed processing attempt
    pub fn apply_recovery(
        &self,
        error: &ProcessingError,
        original_code: &str,
        original_config: &ProcessingConfig,
        attempt: usize,
    ) -> Option<(String, ProcessingConfig)> {
        if attempt >= self.max_retry_attempts {
            return None; // Max retries exceeded
        }

        match self.get_recovery_strategy(&error.error_type) {
            RecoveryStrategy::Fallback => {
                // Will be handled by timeout processor
                None
            }
            RecoveryStrategy::ReturnOriginal => {
                // Return original code with no processing
                Some((original_code.to_string(), original_config.clone()))
            }
            RecoveryStrategy::RetryWithConfig(retry_config) => {
                // Retry with modified configuration
                Some((original_code.to_string(), retry_config))
            }
            RecoveryStrategy::Skip => {
                // Skip processing entirely
                None
            }
        }
    }

    /// Create a retry configuration with reduced complexity
    pub fn create_retry_config(&self, original_config: &ProcessingConfig) -> ProcessingConfig {
        ProcessingConfig {
            enable_syntax_validation: false, // Disable validation on retry
            enable_formatting: false,        // Disable formatting on retry
            enable_optimization: false,
            timeout_ms: original_config.timeout_ms / 2, // Reduce timeout
            custom_options: original_config.custom_options.clone(),
        }
    }
}

impl Default for ErrorRecoveryManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::markdown::code_block::strategy::DefaultStrategy;
    use std::sync::Arc;

    #[test]
    fn test_timeout_processor_creation() {
        let fallback = Arc::new(DefaultStrategy::new());
        let processor = TimeoutProcessor::new(fallback);
        
        assert_eq!(processor.get_max_concurrent_operations(), 10);
        assert_eq!(processor.get_active_operations(), 0);
        assert!(!processor.is_overloaded());
    }

    #[test]
    fn test_timeout_processor_with_concurrency_limit() {
        let fallback = Arc::new(DefaultStrategy::new());
        let processor = TimeoutProcessor::with_concurrency_limit(fallback, 5);
        
        assert_eq!(processor.get_max_concurrent_operations(), 5);
    }

    #[test]
    fn test_operation_slot_management() {
        let fallback = Arc::new(DefaultStrategy::new());
        let processor = TimeoutProcessor::with_concurrency_limit(fallback, 2);
        
        // Should be able to acquire slots
        assert!(processor.try_acquire_operation_slot());
        assert_eq!(processor.get_active_operations(), 1);
        
        assert!(processor.try_acquire_operation_slot());
        assert_eq!(processor.get_active_operations(), 2);
        assert!(processor.is_overloaded());
        
        // Should not be able to acquire more slots
        assert!(!processor.try_acquire_operation_slot());
        
        // Release a slot
        processor.release_operation_slot();
        assert_eq!(processor.get_active_operations(), 1);
        assert!(!processor.is_overloaded());
        
        // Should be able to acquire again
        assert!(processor.try_acquire_operation_slot());
    }

    #[test]
    fn test_successful_processing() {
        let fallback = Arc::new(DefaultStrategy::new());
        let processor = TimeoutProcessor::new(fallback);
        let strategy = Arc::new(DefaultStrategy::new());
        
        let config = ProcessingConfig::default();
        let code = "fn main() {}";
        
        let result = processor.process_with_timeout(strategy, code, &config);
        
        assert!(result.is_successful());
        assert_eq!(result.original_code, code);
    }

    #[test]
    fn test_overload_handling() {
        let fallback = Arc::new(DefaultStrategy::new());
        let processor = TimeoutProcessor::with_concurrency_limit(fallback, 0); // No slots available
        let strategy = Arc::new(DefaultStrategy::new());
        
        let config = ProcessingConfig::default();
        let code = "fn main() {}";
        
        let result = processor.process_with_timeout(strategy, code, &config);
        
        assert!(!result.is_successful());
        assert_eq!(result.errors.len(), 1);
        assert_eq!(result.errors[0].error_type, "system_overload");
    }

    #[test]
    fn test_error_recovery_manager() {
        let manager = ErrorRecoveryManager::new();
        
        // Test default strategies
        assert!(matches!(
            manager.get_recovery_strategy("timeout"),
            RecoveryStrategy::Fallback
        ));
        
        assert!(matches!(
            manager.get_recovery_strategy("syntax_error"),
            RecoveryStrategy::ReturnOriginal
        ));
        
        assert!(matches!(
            manager.get_recovery_strategy("unknown_error"),
            RecoveryStrategy::ReturnOriginal
        ));
    }

    #[test]
    fn test_recovery_strategy_setting() {
        let mut manager = ErrorRecoveryManager::new();
        
        manager.set_recovery_strategy("custom_error", RecoveryStrategy::Skip);
        
        assert!(matches!(
            manager.get_recovery_strategy("custom_error"),
            RecoveryStrategy::Skip
        ));
    }

    #[test]
    fn test_retry_config_creation() {
        let manager = ErrorRecoveryManager::new();
        let original_config = ProcessingConfig {
            enable_syntax_validation: true,
            enable_formatting: true,
            enable_optimization: true,
            timeout_ms: 5000,
            custom_options: std::collections::HashMap::new(),
        };
        
        let retry_config = manager.create_retry_config(&original_config);
        
        assert!(!retry_config.enable_syntax_validation);
        assert!(!retry_config.enable_formatting);
        assert!(!retry_config.enable_optimization);
        assert_eq!(retry_config.timeout_ms, 2500); // Half of original
    }

    #[test]
    fn test_apply_recovery_max_attempts() {
        let manager = ErrorRecoveryManager::new();
        let error = ProcessingError::new("test_error", "Test error");
        let code = "test code";
        let config = ProcessingConfig::default();
        
        // Should return None when max attempts exceeded
        let result = manager.apply_recovery(&error, code, &config, 5);
        assert!(result.is_none());
    }

    #[test]
    fn test_apply_recovery_return_original() {
        let manager = ErrorRecoveryManager::new();
        let error = ProcessingError::syntax_error("Invalid syntax", None, None);
        let code = "test code";
        let config = ProcessingConfig::default();
        
        let result = manager.apply_recovery(&error, code, &config, 1);
        assert!(result.is_some());
        
        let (recovered_code, recovered_config) = result.unwrap();
        assert_eq!(recovered_code, code);
        assert_eq!(recovered_config.timeout_ms, config.timeout_ms);
    }

    // Note: Testing actual timeout behavior is complex in unit tests
    // as it involves real timing and threading. Integration tests
    // would be more appropriate for testing timeout scenarios.
}