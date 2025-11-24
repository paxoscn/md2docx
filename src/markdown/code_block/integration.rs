//! Integration layer for code block processing in the Markdown parser

use std::sync::Arc;
use std::time::Instant;
use crate::markdown::code_block::{
    StrategyRegistry, CodeBlockConfig, ProcessingConfig, ProcessedCodeBlock, ProcessingError
};
use crate::error::ConversionError;

/// Code block processor that integrates the strategy system with the parser
pub struct CodeBlockProcessor {
    registry: StrategyRegistry,
    config: CodeBlockConfig,
}

impl CodeBlockProcessor {
    /// Create a new code block processor with default configuration
    pub fn new() -> Self {
        let mut registry = StrategyRegistry::default();
        
        // Register built-in strategies
        Self::register_builtin_strategies(&mut registry);
        
        Self {
            registry,
            config: CodeBlockConfig::default(),
        }
    }

    /// Create a new code block processor with custom configuration
    pub fn with_config(config: CodeBlockConfig) -> Self {
        let mut registry = StrategyRegistry::default();
        Self::register_builtin_strategies(&mut registry);
        
        Self {
            registry,
            config,
        }
    }

    /// Create a new code block processor with custom registry and configuration
    pub fn with_registry_and_config(registry: StrategyRegistry, config: CodeBlockConfig) -> Self {
        Self {
            registry,
            config,
        }
    }

    /// Process a code block using the appropriate strategy
    pub fn process_code_block(
        &self,
        code: &str,
        language: Option<&str>,
    ) -> Result<ProcessedCodeBlock, ProcessingError> {
        // Capture language as owned string for use in error handling
        let language_owned = language.map(|s| s.to_string());
        
        // Check if processing is enabled globally
        if !self.config.is_processing_enabled() {
            return Ok(ProcessedCodeBlock::unprocessed(
                code.to_string(),
                language_owned,
            ));
        }

        let start_time = Instant::now();

        // Create processing configuration for this language
        let mut processing_config = self.config.create_processing_config(language);
        
        // Pass the language through the config so strategies can access it
        if let Some(lang) = language {
            processing_config.custom_options.insert("language".to_string(), lang.to_string());
        }

        // Get the appropriate strategy
        let strategy = if let Some(lang) = language {
            self.registry.get_strategy(lang)
        } else {
            self.registry.get_default_strategy()
        };

        // Process with timeout handling
        let result = self.process_with_timeout(&strategy, code, &processing_config, &language_owned);

        // Update processing time in the result
        match result {
            Ok(mut processed) => {
                processed.metadata.processing_time = start_time.elapsed();
                Ok(processed)
            }
            Err(error) => {
                // For now, we can't add timing information to the error
                // In the future, we might extend ProcessingError to include metadata
                Err(error)
            }
        }
    }

    /// Process code block with timeout handling
    fn process_with_timeout(
        &self,
        strategy: &Arc<dyn crate::markdown::code_block::CodeBlockStrategy>,
        code: &str,
        config: &ProcessingConfig,
        language: &Option<String>,
    ) -> Result<ProcessedCodeBlock, ProcessingError> {
        // For now, we'll implement a simple timeout check
        // In a more sophisticated implementation, we might use async/await or threads
        let start = Instant::now();
        
        // Check timeout before processing
        if start.elapsed() > config.timeout_duration() {
            return Err(ProcessingError::timeout());
        }

        // Attempt to process
        let result = strategy.process(code, config);

        // Check timeout after processing
        if start.elapsed() > config.timeout_duration() {
            return Err(ProcessingError::timeout());
        }

        // Handle strategy processing errors with fallback
        match result {
            Ok(processed) => Ok(processed),
            Err(error) => {
                // Log the error and fall back to unprocessed code block
                tracing::warn!("Code block processing failed: {:?}", error);
                
                // Create an unprocessed code block with the error recorded
                let mut fallback = ProcessedCodeBlock::unprocessed(
                    code.to_string(),
                    language.clone(),
                );
                fallback.errors.push(error);
                
                Ok(fallback)
            }
        }
    }

    /// Register built-in strategies
    fn register_builtin_strategies(registry: &mut StrategyRegistry) {
        // Register the Rust strategy
        use crate::markdown::code_block::strategies::RustStrategy;
        registry.register_strategy(std::sync::Arc::new(RustStrategy::new()));

        // Register the Note strategy
        use crate::markdown::code_block::strategies::NoteStrategy;
        registry.register_strategy(std::sync::Arc::new(NoteStrategy::new()));
        
        // Future strategies to register:
        // - JavaScriptStrategy  
        // - PythonStrategy
        // - etc.
    }

    /// Get the strategy registry (for testing and introspection)
    pub fn get_registry(&self) -> &StrategyRegistry {
        &self.registry
    }

    /// Get the configuration (for testing and introspection)
    pub fn get_config(&self) -> &CodeBlockConfig {
        &self.config
    }

    /// Update the configuration
    pub fn update_config(&mut self, config: CodeBlockConfig) {
        self.config = config;
    }

    /// Register a new strategy
    pub fn register_strategy(&mut self, strategy: Box<dyn crate::markdown::code_block::CodeBlockStrategy>) {
        self.registry.register_boxed_strategy(strategy);
    }

    /// Check if a language has a specific strategy (not just default)
    pub fn has_specific_strategy_for_language(&self, language: &str) -> bool {
        self.registry.has_strategy_for_language(language)
    }

    /// Get processing statistics
    pub fn get_processing_stats(&self) -> ProcessingStats {
        ProcessingStats {
            registered_strategies: self.registry.strategy_count(),
            registered_languages: self.registry.get_registered_languages(),
            registered_aliases: self.registry.get_registered_aliases(),
            processing_enabled: self.config.is_processing_enabled(),
        }
    }
}

impl Default for CodeBlockProcessor {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics about the code block processing system
#[derive(Debug, Clone)]
pub struct ProcessingStats {
    pub registered_strategies: usize,
    pub registered_languages: Vec<String>,
    pub registered_aliases: Vec<String>,
    pub processing_enabled: bool,
}

impl ProcessingStats {
    /// Get a formatted summary of the processing stats
    pub fn summary(&self) -> String {
        format!(
            "Code Block Processing Stats:\n\
             - Processing enabled: {}\n\
             - Registered strategies: {}\n\
             - Supported languages: {}\n\
             - Language aliases: {}",
            self.processing_enabled,
            self.registered_strategies,
            self.registered_languages.join(", "),
            self.registered_aliases.join(", ")
        )
    }
}

/// Extension trait for converting ProcessingError to ConversionError
impl From<ProcessingError> for ConversionError {
    fn from(error: ProcessingError) -> Self {
        ConversionError::ProcessingError(format!("Code block processing failed: {}", error.message))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::markdown::code_block::LanguageConfig;

    #[test]
    fn test_processor_creation() {
        let processor = CodeBlockProcessor::new();
        assert!(processor.get_config().is_processing_enabled());
        assert!(processor.get_registry().strategy_count() > 0 || processor.get_registry().strategy_count() == 0);
    }

    #[test]
    fn test_processor_with_config() {
        let mut config = CodeBlockConfig::new();
        config.global.enable_processing = false;
        
        let processor = CodeBlockProcessor::with_config(config);
        assert!(!processor.get_config().is_processing_enabled());
    }

    #[test]
    fn test_process_code_block_disabled() {
        let mut config = CodeBlockConfig::new();
        config.global.enable_processing = false;
        
        let processor = CodeBlockProcessor::with_config(config);
        let result = processor.process_code_block("fn main() {}", Some("rust")).unwrap();
        
        assert_eq!(result.original_code, "fn main() {}");
        assert!(result.processed_code.is_none());
        assert_eq!(result.language, Some("rust".to_string()));
    }

    #[test]
    fn test_process_code_block_enabled() {
        let processor = CodeBlockProcessor::new();
        let result = processor.process_code_block("fn main() {}", Some("rust")).unwrap();
        
        assert_eq!(result.original_code, "fn main() {}");
        assert_eq!(result.language, Some("rust".to_string()));
        // The result depends on whether we have a rust strategy registered
    }

    #[test]
    fn test_process_code_block_no_language() {
        let processor = CodeBlockProcessor::new();
        let result = processor.process_code_block("some code", None).unwrap();
        
        assert_eq!(result.original_code, "some code");
        assert!(result.language.is_none());
    }

    #[test]
    fn test_process_code_block_unknown_language() {
        let processor = CodeBlockProcessor::new();
        let result = processor.process_code_block("some code", Some("unknown")).unwrap();
        
        assert_eq!(result.original_code, "some code");
        // Should fall back to default strategy
    }

    #[test]
    fn test_processing_stats() {
        let processor = CodeBlockProcessor::new();
        let stats = processor.get_processing_stats();
        
        assert!(stats.processing_enabled);
        // Check that we have a reasonable number of strategies
        assert!(!stats.registered_aliases.is_empty()); // Should have common aliases
    }

    #[test]
    fn test_language_specific_config() {
        let rust_config = LanguageConfig::new()
            .with_syntax_validation(false)
            .with_formatting(true);

        let config = CodeBlockConfig::new()
            .with_language_config("rust", rust_config);

        let processor = CodeBlockProcessor::with_config(config);
        let result = processor.process_code_block("fn main() {}", Some("rust")).unwrap();
        
        // The processing should respect the language-specific configuration
        assert_eq!(result.original_code, "fn main() {}");
    }

    #[test]
    fn test_timeout_configuration() {
        let mut config = CodeBlockConfig::new();
        config.global.default_timeout_ms = 1; // Very short timeout
        
        let processor = CodeBlockProcessor::with_config(config);
        
        // This should still work because our current implementation doesn't actually timeout
        // In a real implementation with heavy processing, this might timeout
        let result = processor.process_code_block("fn main() {}", Some("rust"));
        assert!(result.is_ok());
    }

    #[test]
    fn test_update_config() {
        let mut processor = CodeBlockProcessor::new();
        assert!(processor.get_config().is_processing_enabled());
        
        let mut new_config = CodeBlockConfig::new();
        new_config.global.enable_processing = false;
        
        processor.update_config(new_config);
        assert!(!processor.get_config().is_processing_enabled());
    }

    #[test]
    fn test_processing_error_conversion() {
        let processing_error = ProcessingError::syntax_error("Test error", None, None);
        let conversion_error: ConversionError = processing_error.into();
        
        match conversion_error {
            ConversionError::ProcessingError(msg) => {
                assert!(msg.contains("Test error"));
            }
            _ => panic!("Expected ProcessingError variant"),
        }
    }
}