//! Integration tests for timeout and error handling mechanisms

use std::sync::Arc;
use std::time::Duration;
use md2docx_converter::markdown::code_block::{
    CodeBlockStrategy, ProcessedCodeBlock, ProcessingConfig, ProcessingError,
    EnhancedCodeBlockProcessor, TimeoutProcessor, ErrorRecoveryManager,
    RecoveryStrategy, DefaultStrategy
};

/// Mock strategy that simulates slow processing
struct SlowStrategy {
    delay_ms: u64,
    should_fail: bool,
}

impl SlowStrategy {
    fn new(delay_ms: u64, should_fail: bool) -> Self {
        Self { delay_ms, should_fail }
    }
}

impl CodeBlockStrategy for SlowStrategy {
    fn process(&self, code: &str, _config: &ProcessingConfig) -> Result<ProcessedCodeBlock, ProcessingError> {
        // Simulate processing delay
        std::thread::sleep(Duration::from_millis(self.delay_ms));
        
        if self.should_fail {
            Err(ProcessingError::syntax_error("Simulated failure", Some(1), Some(1)))
        } else {
            Ok(ProcessedCodeBlock::new(code.to_string(), Some("mock".to_string())))
        }
    }
    
    fn supports_language(&self, language: &str) -> bool {
        language == "mock"
    }
    
    fn get_language_name(&self) -> &'static str {
        "mock"
    }
}

/// Mock strategy that always fails
struct FailingStrategy;

impl CodeBlockStrategy for FailingStrategy {
    fn process(&self, _code: &str, _config: &ProcessingConfig) -> Result<ProcessedCodeBlock, ProcessingError> {
        Err(ProcessingError::internal_error("Always fails"))
    }
    
    fn supports_language(&self, language: &str) -> bool {
        language == "failing"
    }
    
    fn get_language_name(&self) -> &'static str {
        "failing"
    }
}

#[test]
fn test_timeout_handling() {
    let fallback = Arc::new(DefaultStrategy::new());
    let timeout_processor = TimeoutProcessor::new(fallback);
    
    // Create a slow strategy that takes longer than the timeout
    let slow_strategy = Arc::new(SlowStrategy::new(2000, false)); // 2 seconds
    let config = ProcessingConfig::default().with_timeout_ms(500); // 500ms timeout
    
    let code = "fn main() { println!(\"Hello, world!\"); }";
    let result = timeout_processor.process_with_timeout(slow_strategy, code, &config);
    
    // Should have timed out and fallen back
    assert!(!result.errors.is_empty());
    assert!(result.errors.iter().any(|e| e.error_type == "timeout"));
    assert_eq!(result.original_code, code);
}

#[test]
fn test_error_recovery() {
    let fallback = Arc::new(DefaultStrategy::new());
    let processor = EnhancedCodeBlockProcessor::new(fallback);
    
    // Use a failing strategy
    let failing_strategy = Arc::new(FailingStrategy);
    let config = ProcessingConfig::default();
    let code = "invalid code";
    
    let result = processor.process_with_recovery(
        failing_strategy,
        code,
        &config,
        Some("failing"),
    );
    
    // Should have failed but attempted recovery
    assert!(!result.is_successful());
    assert!(result.processed_block.error_count() > 0);
    
    // Should have error report
    assert!(result.error_report.is_some());
    let report = result.error_report.unwrap();
    assert!(report.total_error_count() > 0);
}

#[test]
fn test_system_overload_handling() {
    let fallback = Arc::new(DefaultStrategy::new());
    let processor = Arc::new(TimeoutProcessor::with_concurrency_limit(fallback, 1)); // Only 1 concurrent operation
    
    let strategy = Arc::new(SlowStrategy::new(1000, false)); // 1 second delay
    let config = ProcessingConfig::default();
    let code = "fn main() {}";
    
    // Start first operation (will take the only slot)
    let handle = std::thread::spawn({
        let processor = processor.clone();
        let strategy = strategy.clone();
        let config = config.clone();
        let code = code.to_string();
        move || {
            processor.process_with_timeout(strategy, &code, &config)
        }
    });
    
    // Give the first operation time to start
    std::thread::sleep(Duration::from_millis(100));
    
    // Try second operation (should be rejected due to overload)
    let result = processor.process_with_timeout(strategy, code, &config);
    
    // Should have been rejected due to system overload
    assert!(!result.is_successful());
    assert!(result.errors.iter().any(|e| e.error_type == "system_overload"));
    
    // Wait for first operation to complete
    let _first_result = handle.join().unwrap();
}

#[test]
fn test_error_recovery_manager() {
    let mut manager = ErrorRecoveryManager::new();
    
    // Test default recovery strategies
    assert!(matches!(
        manager.get_recovery_strategy("timeout"),
        RecoveryStrategy::Fallback
    ));
    
    // Test custom recovery strategy
    manager.set_recovery_strategy("custom_error", RecoveryStrategy::Skip);
    assert!(matches!(
        manager.get_recovery_strategy("custom_error"),
        RecoveryStrategy::Skip
    ));
    
    // Test retry configuration creation
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
    assert_eq!(retry_config.timeout_ms, 2500); // Half of original
}

#[test]
fn test_enhanced_processor_batch_processing() {
    let fallback = Arc::new(DefaultStrategy::new());
    let processor = EnhancedCodeBlockProcessor::new(fallback);
    
    let requests = vec![
        md2docx_converter::markdown::code_block::ProcessingRequest::new(
            Arc::new(DefaultStrategy::new()),
            "fn main() {}".to_string(),
            ProcessingConfig::default(),
            Some("rust".to_string()),
        ),
        md2docx_converter::markdown::code_block::ProcessingRequest::new(
            Arc::new(FailingStrategy),
            "invalid code".to_string(),
            ProcessingConfig::default(),
            Some("failing".to_string()),
        ),
    ];
    
    let results = processor.process_batch(requests);
    
    assert_eq!(results.len(), 2);
    assert!(results[0].is_successful()); // First should succeed
    assert!(!results[1].is_successful()); // Second should fail
}

#[test]
fn test_detailed_error_reporting() {
    let fallback = Arc::new(DefaultStrategy::new());
    let processor = EnhancedCodeBlockProcessor::new(fallback);
    
    let failing_strategy = Arc::new(FailingStrategy);
    let config = ProcessingConfig::default();
    let code = "invalid code that will fail";
    
    let result = processor.process_with_recovery(
        failing_strategy,
        code,
        &config,
        Some("failing"),
    );
    
    // Should have detailed error report
    assert!(result.error_report.is_some());
    let report = result.error_report.unwrap();
    
    // Check report contents
    assert!(report.total_error_count() > 0);
    assert_eq!(report.context.language, Some("failing".to_string()));
    assert_eq!(report.context.code_length, code.len());
    
    // Generate human-readable report
    let report_text = report.generate_report();
    assert!(report_text.contains("Error Report for failing Processing"));
    assert!(report_text.contains("Code Length:"));
    assert!(report_text.contains("ERRORS:"));
}

#[test]
fn test_processing_result_summary() {
    let fallback = Arc::new(DefaultStrategy::new());
    let processor = EnhancedCodeBlockProcessor::new(fallback);
    
    // Test successful processing
    let success_strategy = Arc::new(DefaultStrategy::new());
    let config = ProcessingConfig::default();
    let code = "fn main() {}";
    
    let result = processor.process_with_recovery(
        success_strategy,
        code,
        &config,
        Some("rust"),
    );
    
    let summary = result.get_summary();
    assert!(summary.success);
    assert_eq!(summary.error_count, 0);
    assert_eq!(summary.warning_count, 0);
    assert_eq!(summary.recovery_attempts, 0);
    assert!(!summary.had_critical_errors);
    
    // Test failed processing
    let failing_strategy = Arc::new(FailingStrategy);
    let result = processor.process_with_recovery(
        failing_strategy,
        code,
        &config,
        Some("failing"),
    );
    
    let summary = result.get_summary();
    assert!(!summary.success);
    assert!(summary.error_count > 0);
}

#[test]
fn test_system_load_monitoring() {
    let fallback = Arc::new(DefaultStrategy::new());
    let processor = EnhancedCodeBlockProcessor::with_settings(fallback, 5, true);
    
    let load_info = processor.get_system_load();
    assert_eq!(load_info.max_concurrent_operations, 5);
    assert_eq!(load_info.active_operations, 0);
    assert!(!load_info.is_overloaded);
    assert_eq!(load_info.load_percentage(), 0.0);
    assert!(!load_info.is_high_load());
    assert_eq!(load_info.available_capacity(), 5);
}

#[test]
fn test_error_severity_handling() {
    let fallback = Arc::new(DefaultStrategy::new());
    let processor = EnhancedCodeBlockProcessor::new(fallback);
    
    // Create a strategy that produces different error severities
    struct MultiErrorStrategy;
    
    impl CodeBlockStrategy for MultiErrorStrategy {
        fn process(&self, code: &str, _config: &ProcessingConfig) -> Result<ProcessedCodeBlock, ProcessingError> {
            let mut block = ProcessedCodeBlock::new(code.to_string(), Some("multi".to_string()));
            
            // Add errors of different severities
            block = block.with_error(ProcessingError::timeout()); // Critical
            block = block.with_error(ProcessingError::syntax_error("Syntax issue", None, None)); // High
            block = block.with_error(ProcessingError::formatting_error("Format issue")); // Low
            
            Ok(block)
        }
        
        fn supports_language(&self, language: &str) -> bool {
            language == "multi"
        }
        
        fn get_language_name(&self) -> &'static str {
            "multi"
        }
    }
    
    let multi_strategy = Arc::new(MultiErrorStrategy);
    let config = ProcessingConfig::default();
    let code = "test code";
    
    let result = processor.process_with_recovery(
        multi_strategy,
        code,
        &config,
        Some("multi"),
    );
    
    // Should have error report with different severity levels
    assert!(result.error_report.is_some());
    let report = result.error_report.unwrap();
    
    assert!(report.has_critical_errors());
    assert!(report.has_high_severity_errors());
    
    let summary = report.get_summary();
    assert_eq!(summary.critical_errors, 1);
    assert_eq!(summary.high_errors, 1);
    assert_eq!(summary.low_errors, 1);
    assert_eq!(summary.medium_errors, 0);
}