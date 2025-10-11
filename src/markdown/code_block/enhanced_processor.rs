//! Enhanced code block processor with timeout and error handling

use std::sync::Arc;
use std::time::Instant;

use crate::markdown::code_block::{
    CodeBlockStrategy, ProcessedCodeBlock, ProcessingConfig, ProcessingError, 
    ProcessingWarning, ErrorReport, ErrorContext, TimeoutProcessor, 
    ErrorRecoveryManager, RecoveryStrategy
};

/// Enhanced processor that combines timeout handling, error recovery, and detailed reporting
pub struct EnhancedCodeBlockProcessor {
    timeout_processor: TimeoutProcessor,
    recovery_manager: ErrorRecoveryManager,
    enable_detailed_reporting: bool,
}

impl EnhancedCodeBlockProcessor {
    /// Create a new enhanced processor
    pub fn new(fallback_strategy: Arc<dyn CodeBlockStrategy>) -> Self {
        Self {
            timeout_processor: TimeoutProcessor::new(fallback_strategy),
            recovery_manager: ErrorRecoveryManager::new(),
            enable_detailed_reporting: true,
        }
    }

    /// Create an enhanced processor with custom settings
    pub fn with_settings(
        fallback_strategy: Arc<dyn CodeBlockStrategy>,
        max_concurrent: usize,
        enable_reporting: bool,
    ) -> Self {
        Self {
            timeout_processor: TimeoutProcessor::with_concurrency_limit(fallback_strategy, max_concurrent),
            recovery_manager: ErrorRecoveryManager::new(),
            enable_detailed_reporting: enable_reporting,
        }
    }

    /// Process code with comprehensive error handling and recovery
    pub fn process_with_recovery(
        &self,
        strategy: Arc<dyn CodeBlockStrategy>,
        code: &str,
        config: &ProcessingConfig,
        language: Option<&str>,
    ) -> ProcessingResult {
        let start_time = Instant::now();
        let mut error_report = if self.enable_detailed_reporting {
            Some(ErrorReport::new(ErrorContext::from_config(
                language.map(|s| s.to_string()),
                code,
                config,
                "enhanced-processor-1.0.0".to_string(),
            )))
        } else {
            None
        };

        let mut attempt = 0;
        let mut current_config = config.clone();
        let mut current_code = code.to_string();
        let mut _last_error: Option<ProcessingError> = None;

        loop {
            attempt += 1;
            
            // Try processing with current configuration
            let result = self.timeout_processor.process_with_timeout(
                strategy.clone(),
                &current_code,
                &current_config,
            );

            // Update error report if enabled
            if let Some(ref mut report) = error_report {
                report.set_processing_time(start_time.elapsed());
                report.set_final_strategy(strategy.get_language_name());
                
                for error in &result.errors {
                    report.add_error(error.clone());
                }
                for warning in &result.warnings {
                    report.add_warning(warning.clone());
                }
            }

            // If processing was successful, return the result
            if result.is_successful() {
                return ProcessingResult {
                    processed_block: result,
                    error_report,
                    recovery_attempts: attempt - 1,
                    final_strategy_used: strategy.get_language_name().to_string(),
                };
            }

            // Processing failed, try recovery
            if let Some(primary_error) = result.errors.first() {
                _last_error = Some(primary_error.clone());
                
                // Try to apply recovery strategy
                if let Some((recovered_code, recovered_config)) = self.recovery_manager.apply_recovery(
                    primary_error,
                    &current_code,
                    &current_config,
                    attempt,
                ) {
                    current_code = recovered_code;
                    current_config = recovered_config;
                    
                    if let Some(ref mut report) = error_report {
                        report.increment_recovery_attempts();
                    }
                    
                    continue; // Try again with recovery
                }
            }

            // Recovery failed or not applicable, return the failed result
            let mut final_result = result;
            
            // Add recovery failure warning if we attempted recovery
            if attempt > 1 {
                final_result.warnings.push(ProcessingWarning::fallback_warning(
                    &format!("Recovery failed after {} attempts", attempt - 1)
                ));
            }

            return ProcessingResult {
                processed_block: final_result,
                error_report,
                recovery_attempts: attempt - 1,
                final_strategy_used: strategy.get_language_name().to_string(),
            };
        }
    }

    /// Process code with simple error handling (no recovery)
    pub fn process_simple(
        &self,
        strategy: Arc<dyn CodeBlockStrategy>,
        code: &str,
        config: &ProcessingConfig,
    ) -> ProcessedCodeBlock {
        self.timeout_processor.process_with_timeout(strategy, code, config)
    }

    /// Configure recovery strategy for specific error types
    pub fn set_recovery_strategy(&mut self, error_type: &str, strategy: RecoveryStrategy) {
        self.recovery_manager.set_recovery_strategy(error_type, strategy);
    }

    /// Check if the processor is currently overloaded
    pub fn is_overloaded(&self) -> bool {
        self.timeout_processor.is_overloaded()
    }

    /// Get current system load information
    pub fn get_system_load(&self) -> SystemLoadInfo {
        SystemLoadInfo {
            active_operations: self.timeout_processor.get_active_operations(),
            max_concurrent_operations: self.timeout_processor.get_max_concurrent_operations(),
            is_overloaded: self.is_overloaded(),
        }
    }

    /// Enable or disable detailed error reporting
    pub fn set_detailed_reporting(&mut self, enable: bool) {
        self.enable_detailed_reporting = enable;
    }

    /// Process multiple code blocks with batch error handling
    pub fn process_batch(
        &self,
        requests: Vec<ProcessingRequest>,
    ) -> Vec<ProcessingResult> {
        requests
            .into_iter()
            .map(|req| {
                self.process_with_recovery(
                    req.strategy,
                    &req.code,
                    &req.config,
                    req.language.as_deref(),
                )
            })
            .collect()
    }

    /// Create a retry configuration for failed processing
    pub fn create_retry_config(&self, original_config: &ProcessingConfig) -> ProcessingConfig {
        self.recovery_manager.create_retry_config(original_config)
    }
}

/// Result of enhanced processing including error reporting
#[derive(Debug)]
pub struct ProcessingResult {
    pub processed_block: ProcessedCodeBlock,
    pub error_report: Option<ErrorReport>,
    pub recovery_attempts: usize,
    pub final_strategy_used: String,
}

impl ProcessingResult {
    /// Check if processing was successful
    pub fn is_successful(&self) -> bool {
        self.processed_block.is_successful()
    }

    /// Check if recovery was attempted
    pub fn had_recovery_attempts(&self) -> bool {
        self.recovery_attempts > 0
    }

    /// Get a summary of the processing result
    pub fn get_summary(&self) -> ProcessingResultSummary {
        ProcessingResultSummary {
            success: self.is_successful(),
            error_count: self.processed_block.error_count(),
            warning_count: self.processed_block.warning_count(),
            recovery_attempts: self.recovery_attempts,
            final_strategy: self.final_strategy_used.clone(),
            processing_time: self.processed_block.metadata.processing_time,
            had_critical_errors: self.error_report
                .as_ref()
                .map(|r| r.has_critical_errors())
                .unwrap_or(false),
        }
    }

    /// Generate a detailed report if error reporting was enabled
    pub fn generate_detailed_report(&self) -> Option<String> {
        self.error_report.as_ref().map(|report| report.generate_report())
    }
}

/// Request for processing a code block
pub struct ProcessingRequest {
    pub strategy: Arc<dyn CodeBlockStrategy>,
    pub code: String,
    pub config: ProcessingConfig,
    pub language: Option<String>,
}

impl ProcessingRequest {
    /// Create a new processing request
    pub fn new(
        strategy: Arc<dyn CodeBlockStrategy>,
        code: String,
        config: ProcessingConfig,
        language: Option<String>,
    ) -> Self {
        Self {
            strategy,
            code,
            config,
            language,
        }
    }
}

/// Summary of processing result
#[derive(Debug, Clone)]
pub struct ProcessingResultSummary {
    pub success: bool,
    pub error_count: usize,
    pub warning_count: usize,
    pub recovery_attempts: usize,
    pub final_strategy: String,
    pub processing_time: std::time::Duration,
    pub had_critical_errors: bool,
}

/// Information about system load
#[derive(Debug, Clone)]
pub struct SystemLoadInfo {
    pub active_operations: usize,
    pub max_concurrent_operations: usize,
    pub is_overloaded: bool,
}

impl SystemLoadInfo {
    /// Get the load percentage (0.0 to 1.0)
    pub fn load_percentage(&self) -> f64 {
        if self.max_concurrent_operations == 0 {
            0.0
        } else {
            self.active_operations as f64 / self.max_concurrent_operations as f64
        }
    }

    /// Check if the system is under high load (>80%)
    pub fn is_high_load(&self) -> bool {
        self.load_percentage() > 0.8
    }

    /// Get available capacity
    pub fn available_capacity(&self) -> usize {
        self.max_concurrent_operations.saturating_sub(self.active_operations)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::markdown::code_block::strategy::DefaultStrategy;

    #[test]
    fn test_enhanced_processor_creation() {
        let fallback = Arc::new(DefaultStrategy::new());
        let processor = EnhancedCodeBlockProcessor::new(fallback);
        
        assert!(!processor.is_overloaded());
        assert!(processor.enable_detailed_reporting);
    }

    #[test]
    fn test_enhanced_processor_with_settings() {
        let fallback = Arc::new(DefaultStrategy::new());
        let processor = EnhancedCodeBlockProcessor::with_settings(fallback, 5, false);
        
        assert!(!processor.enable_detailed_reporting);
        let load_info = processor.get_system_load();
        assert_eq!(load_info.max_concurrent_operations, 5);
    }

    #[test]
    fn test_simple_processing() {
        let fallback = Arc::new(DefaultStrategy::new());
        let processor = EnhancedCodeBlockProcessor::new(fallback);
        let strategy = Arc::new(DefaultStrategy::new());
        
        let config = ProcessingConfig::default();
        let code = "fn main() {}";
        
        let result = processor.process_simple(strategy, code, &config);
        
        assert!(result.is_successful());
        assert_eq!(result.original_code, code);
    }

    #[test]
    fn test_processing_with_recovery() {
        let fallback = Arc::new(DefaultStrategy::new());
        let processor = EnhancedCodeBlockProcessor::new(fallback);
        let strategy = Arc::new(DefaultStrategy::new());
        
        let config = ProcessingConfig::default();
        let code = "fn main() {}";
        
        let result = processor.process_with_recovery(
            strategy,
            code,
            &config,
            Some("rust"),
        );
        
        assert!(result.is_successful());
        assert!(!result.had_recovery_attempts());
        assert!(result.error_report.is_some());
    }

    #[test]
    fn test_system_load_info() {
        let fallback = Arc::new(DefaultStrategy::new());
        let processor = EnhancedCodeBlockProcessor::with_settings(fallback, 10, true);
        
        let load_info = processor.get_system_load();
        assert_eq!(load_info.active_operations, 0);
        assert_eq!(load_info.max_concurrent_operations, 10);
        assert!(!load_info.is_overloaded);
        assert_eq!(load_info.load_percentage(), 0.0);
        assert!(!load_info.is_high_load());
        assert_eq!(load_info.available_capacity(), 10);
    }

    #[test]
    fn test_processing_request() {
        let strategy = Arc::new(DefaultStrategy::new());
        let config = ProcessingConfig::default();
        let code = "console.log('hello');".to_string();
        let language = Some("javascript".to_string());
        
        let request = ProcessingRequest::new(strategy, code.clone(), config, language.clone());
        
        assert_eq!(request.code, code);
        assert_eq!(request.language, language);
    }

    #[test]
    fn test_processing_result_summary() {
        let fallback = Arc::new(DefaultStrategy::new());
        let processor = EnhancedCodeBlockProcessor::new(fallback);
        let strategy = Arc::new(DefaultStrategy::new());
        
        let config = ProcessingConfig::default();
        let code = "fn main() {}";
        
        let result = processor.process_with_recovery(
            strategy,
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
    }

    #[test]
    fn test_batch_processing() {
        let fallback = Arc::new(DefaultStrategy::new());
        let processor = EnhancedCodeBlockProcessor::new(fallback);
        let strategy = Arc::new(DefaultStrategy::new());
        
        let requests = vec![
            ProcessingRequest::new(
                strategy.clone(),
                "fn main() {}".to_string(),
                ProcessingConfig::default(),
                Some("rust".to_string()),
            ),
            ProcessingRequest::new(
                strategy.clone(),
                "console.log('hello');".to_string(),
                ProcessingConfig::default(),
                Some("javascript".to_string()),
            ),
        ];
        
        let results = processor.process_batch(requests);
        
        assert_eq!(results.len(), 2);
        assert!(results[0].is_successful());
        assert!(results[1].is_successful());
    }

    #[test]
    fn test_retry_config_creation() {
        let fallback = Arc::new(DefaultStrategy::new());
        let processor = EnhancedCodeBlockProcessor::new(fallback);
        
        let original_config = ProcessingConfig {
            enable_syntax_validation: true,
            enable_formatting: true,
            enable_optimization: true,
            timeout_ms: 5000,
            custom_options: std::collections::HashMap::new(),
        };
        
        let retry_config = processor.create_retry_config(&original_config);
        
        assert!(!retry_config.enable_syntax_validation);
        assert!(!retry_config.enable_formatting);
        assert!(!retry_config.enable_optimization);
        assert_eq!(retry_config.timeout_ms, 2500);
    }

    #[test]
    fn test_recovery_strategy_configuration() {
        let fallback = Arc::new(DefaultStrategy::new());
        let mut processor = EnhancedCodeBlockProcessor::new(fallback);
        
        processor.set_recovery_strategy("custom_error", RecoveryStrategy::Skip);
        
        // The recovery strategy is set internally, we can't directly test it
        // but we can verify the processor still works
        assert!(!processor.is_overloaded());
    }

    #[test]
    fn test_detailed_reporting_toggle() {
        let fallback = Arc::new(DefaultStrategy::new());
        let mut processor = EnhancedCodeBlockProcessor::new(fallback);
        
        assert!(processor.enable_detailed_reporting);
        
        processor.set_detailed_reporting(false);
        assert!(!processor.enable_detailed_reporting);
        
        processor.set_detailed_reporting(true);
        assert!(processor.enable_detailed_reporting);
    }
}