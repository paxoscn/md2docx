//! Strategy trait and core strategy implementations

use std::time::Instant;
use crate::markdown::code_block::{
    ProcessedCodeBlock, ProcessingConfig, ProcessingError, ProcessingMetadata
};

/// Trait that defines the interface for code block processing strategies
pub trait CodeBlockStrategy: Send + Sync {
    /// Process a code block according to the strategy's rules
    fn process(&self, code: &str, config: &ProcessingConfig) -> Result<ProcessedCodeBlock, ProcessingError>;
    
    /// Check if this strategy supports the given language
    fn supports_language(&self, language: &str) -> bool;
    
    /// Get the primary language name this strategy handles
    fn get_language_name(&self) -> &'static str;
    
    /// Get the priority of this strategy (higher numbers = higher priority)
    /// Used for conflict resolution when multiple strategies support the same language
    fn get_priority(&self) -> u8 {
        100 // Default priority
    }
    
    /// Get the version of this strategy implementation
    fn get_version(&self) -> &'static str {
        "1.0.0"
    }
    
    /// Get a description of what this strategy does
    fn get_description(&self) -> &'static str {
        "Generic code block processing strategy"
    }
}

/// Default strategy that provides basic processing for any code block
#[derive(Debug, Clone)]
pub struct DefaultStrategy;

impl DefaultStrategy {
    /// Create a new default strategy instance
    pub fn new() -> Self {
        Self
    }
}

impl CodeBlockStrategy for DefaultStrategy {
    fn process(&self, code: &str, config: &ProcessingConfig) -> Result<ProcessedCodeBlock, ProcessingError> {
        let start_time = Instant::now();
        
        let metadata = ProcessingMetadata::new(self.get_version())
            .with_processing_time(start_time.elapsed());
        
        // Get the language from config if available
        let language = config.custom_options.get("language").cloned();
        
        let processed = ProcessedCodeBlock::new(code.to_string(), language)
            .with_metadata(metadata)
            .with_validation(true); // Assume valid since we don't validate
        
        Ok(processed)
    }
    
    fn supports_language(&self, _language: &str) -> bool {
        true // Default strategy supports all languages as fallback
    }
    
    fn get_language_name(&self) -> &'static str {
        "default"
    }
    
    fn get_priority(&self) -> u8 {
        0 // Lowest priority - only used as fallback
    }
    
    fn get_version(&self) -> &'static str {
        "1.0.0"
    }
    
    fn get_description(&self) -> &'static str {
        "Default fallback strategy that performs no processing"
    }
}

impl Default for DefaultStrategy {
    fn default() -> Self {
        Self::new()
    }
}

/// Base trait for language-specific strategies
pub trait LanguageStrategy: CodeBlockStrategy {
    /// Validate the syntax of the code
    fn validate_syntax(&self, code: &str) -> Result<bool, ProcessingError>;
    
    /// Format the code according to language conventions
    fn format_code(&self, code: &str) -> Result<String, ProcessingError>;
    
    /// Get language-specific file extensions
    fn get_file_extensions(&self) -> Vec<&'static str> {
        vec![]
    }
    
    /// Get common alternative names for this language
    fn get_language_aliases(&self) -> Vec<&'static str> {
        vec![]
    }
}

/// Helper function to check if a language matches any of the given patterns
pub fn language_matches(language: &str, patterns: &[&str]) -> bool {
    let normalized = language.to_lowercase();
    patterns.iter().any(|pattern| {
        pattern.to_lowercase() == normalized
    })
}

/// Helper function to create a basic processed code block with timing
pub fn create_basic_processed_block(
    original_code: &str,
    language: Option<&str>,
    start_time: Instant,
    version: &str,
) -> ProcessedCodeBlock {
    let metadata = ProcessingMetadata::new(version)
        .with_processing_time(start_time.elapsed());
    
    ProcessedCodeBlock::new(original_code.to_string(), language.map(|s| s.to_string()))
        .with_metadata(metadata)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::markdown::code_block::ProcessingConfig;

    #[test]
    fn test_default_strategy_creation() {
        let strategy = DefaultStrategy::new();
        assert_eq!(strategy.get_language_name(), "default");
        assert_eq!(strategy.get_priority(), 0);
        assert_eq!(strategy.get_version(), "1.0.0");
        assert!(strategy.supports_language("rust"));
        assert!(strategy.supports_language("javascript"));
        assert!(strategy.supports_language("unknown"));
    }

    #[test]
    fn test_default_strategy_process() {
        let strategy = DefaultStrategy::new();
        let config = ProcessingConfig::default();
        let code = "fn main() { println!(\"Hello, world!\"); }";
        
        let result = strategy.process(code, &config);
        assert!(result.is_ok());
        
        let processed = result.unwrap();
        assert_eq!(processed.original_code, code);
        assert!(processed.processed_code.is_none());
        assert!(processed.errors.is_empty());
        assert!(processed.warnings.is_empty());
        assert!(processed.metadata.syntax_valid);
        assert!(!processed.metadata.is_formatted);
        assert!(processed.metadata.is_validated);
    }

    #[test]
    fn test_language_matches_helper() {
        assert!(language_matches("rust", &["rust", "rs"]));
        assert!(language_matches("RUST", &["rust", "rs"]));
        assert!(language_matches("Rs", &["rust", "rs"]));
        assert!(!language_matches("python", &["rust", "rs"]));
        assert!(!language_matches("", &["rust", "rs"]));
    }

    #[test]
    fn test_create_basic_processed_block() {
        let start_time = Instant::now();
        let code = "console.log('hello');";
        let language = Some("javascript");
        
        let block = create_basic_processed_block(code, language, start_time, "2.0.0");
        
        assert_eq!(block.original_code, code);
        assert_eq!(block.language, Some("javascript".to_string()));
        assert_eq!(block.metadata.processor_version, "2.0.0");
        assert!(block.metadata.processing_time >= std::time::Duration::from_nanos(0));
    }

    #[test]
    fn test_default_strategy_default_impl() {
        let strategy1 = DefaultStrategy::default();
        let strategy2 = DefaultStrategy::new();
        
        assert_eq!(strategy1.get_language_name(), strategy2.get_language_name());
        assert_eq!(strategy1.get_priority(), strategy2.get_priority());
    }

    #[test]
    fn test_strategy_trait_methods() {
        let strategy = DefaultStrategy::new();
        
        // Test all trait methods have reasonable defaults/implementations
        assert!(strategy.supports_language("any_language"));
        assert_eq!(strategy.get_priority(), 0);
        assert!(!strategy.get_version().is_empty());
        assert!(!strategy.get_description().is_empty());
        assert!(!strategy.get_language_name().is_empty());
    }

    #[test]
    fn test_processed_block_from_default_strategy() {
        let strategy = DefaultStrategy::new();
        let config = ProcessingConfig::default();
        let code = "# This is a comment\nprint('hello')";
        
        let result = strategy.process(code, &config).unwrap();
        
        // Verify the processed block has expected properties
        assert_eq!(result.get_final_code(), code);
        assert!(result.is_successful());
        assert!(!result.has_warnings());
        assert_eq!(result.error_count(), 0);
        assert_eq!(result.warning_count(), 0);
        
        let summary = result.get_summary();
        assert_eq!(summary.get_status(), "skipped"); // No processing was done
        assert!(summary.is_successful());
        assert!(!summary.has_issues());
    }
}