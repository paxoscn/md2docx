//! Bash code block processing strategy

use std::time::Instant;
use crate::markdown::code_block::{
    CodeBlockStrategy, LanguageStrategy, ProcessedCodeBlock, ProcessingConfig, 
    ProcessingError, ProcessingMetadata, ProcessingWarning, language_matches
};

/// Strategy for processing Bash code blocks
#[derive(Debug, Clone)]
pub struct BashStrategy;

impl BashStrategy {
    /// Create a new Bash strategy instance
    pub fn new() -> Self {
        Self
    }

    /// Apply italic formatting to Bash comments
    fn apply_comment_italic(&self, code: &str) -> String {
        let mut result = String::new();
        
        for line in code.lines() {
            // Check if line contains a comment (# character)
            if let Some(comment_pos) = line.find('#') {
                // Add the code part before the comment
                result.push_str(&line[..comment_pos]);
                // Add the comment with italic formatting
                result.push_str("[ITALIC]");
                result.push_str(&line[comment_pos..]);
                result.push_str("[/ITALIC]");
            } else {
                // No comment, add line as-is
                result.push_str(line);
            }
            result.push('\n');
        }
        
        // Remove trailing newline if original didn't have one
        if !code.ends_with('\n') && result.ends_with('\n') {
            result.pop();
        }
        
        result
    }

    /// Format Bash code
    fn format_bash_code(&self, code: &str) -> Result<String, ProcessingError> {
        // Apply comment italic formatting
        let formatted = code.trim().to_string();
        let formatted_with_italic = self.apply_comment_italic(&formatted);
        Ok(formatted_with_italic)
    }

    /// Check for common Bash code quality issues
    fn check_code_quality(&self, code: &str) -> Vec<ProcessingWarning> {
        let mut warnings = Vec::new();
        
        // Check for common issues
        if code.contains("rm -rf") && !code.contains("rm -rf /") {
            warnings.push(ProcessingWarning::new(
                "code_quality",
                "Be careful with 'rm -rf' command"
            ));
        }
        
        if code.lines().any(|line| line.len() > 120) {
            warnings.push(ProcessingWarning::new(
                "style",
                "Some lines exceed 120 characters, consider breaking them up"
            ));
        }
        
        warnings
    }
}

impl CodeBlockStrategy for BashStrategy {
    fn process(&self, code: &str, config: &ProcessingConfig) -> Result<ProcessedCodeBlock, ProcessingError> {
        let start_time = Instant::now();
        let mut warnings = Vec::new();
        
        // Format code if enabled
        let formatted_code = if config.enable_formatting {
            match self.format_bash_code(code) {
                Ok(formatted) => Some(formatted),
                Err(e) => {
                    warnings.push(ProcessingWarning::formatting_warning(&e.to_string()));
                    None
                }
            }
        } else {
            None
        };
        
        // Check code quality
        warnings.extend(self.check_code_quality(code));
        
        // Create metadata
        let mut metadata = ProcessingMetadata::new(self.get_version())
            .with_processing_time(start_time.elapsed());
        
        metadata.is_formatted = formatted_code.is_some();
        metadata.is_validated = false;
        metadata.syntax_valid = true;
        
        // Add custom attributes
        metadata = metadata.with_custom_attribute("language", "bash");
        if formatted_code.is_some() {
            metadata = metadata.with_custom_attribute("formatter", "bash_comment_italic");
        }
        
        let processed = ProcessedCodeBlock::new(code.to_string(), Some("bash".to_string()))
            .with_metadata(metadata);
        
        let processed = if let Some(formatted) = formatted_code {
            processed.with_processed_code(formatted)
        } else {
            processed
        };
        
        // Add warnings
        let mut final_processed = processed;
        for warning in warnings {
            final_processed = final_processed.with_warning(warning);
        }
        
        Ok(final_processed)
    }
    
    fn supports_language(&self, language: &str) -> bool {
        language_matches(language, &["bash", "sh", "shell", "zsh"])
    }
    
    fn get_language_name(&self) -> &'static str {
        "bash"
    }
    
    fn get_priority(&self) -> u8 {
        100 // Medium priority for Bash code
    }
    
    fn get_version(&self) -> &'static str {
        "1.0.0"
    }
    
    fn get_description(&self) -> &'static str {
        "Bash code processing with comment italic formatting"
    }
}

impl LanguageStrategy for BashStrategy {
    fn validate_syntax(&self, _code: &str) -> Result<bool, ProcessingError> {
        // Bash syntax validation is complex and would require external tools
        // For now, we'll skip validation
        Ok(true)
    }
    
    fn format_code(&self, code: &str) -> Result<String, ProcessingError> {
        self.format_bash_code(code)
    }
    
    fn get_file_extensions(&self) -> Vec<&'static str> {
        vec!["sh", "bash"]
    }
    
    fn get_language_aliases(&self) -> Vec<&'static str> {
        vec!["bash", "sh", "shell", "zsh"]
    }
}

impl Default for BashStrategy {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::markdown::code_block::ProcessingConfig;

    #[test]
    fn test_bash_strategy_creation() {
        let strategy = BashStrategy::new();
        assert_eq!(strategy.get_language_name(), "bash");
        assert_eq!(strategy.get_priority(), 100);
        assert!(strategy.supports_language("bash"));
        assert!(strategy.supports_language("sh"));
        assert!(strategy.supports_language("shell"));
        assert!(!strategy.supports_language("python"));
    }

    #[test]
    fn test_comment_italic_formatting() {
        let strategy = BashStrategy::new();
        let code = r#"#!/bin/bash
# This is a comment
echo "Hello" # inline comment
ls -la"#;
        
        let formatted = strategy.apply_comment_italic(code);
        
        // Check that comments are wrapped in [ITALIC] tags
        assert!(formatted.contains("[ITALIC]#!/bin/bash[/ITALIC]"));
        assert!(formatted.contains("[ITALIC]# This is a comment[/ITALIC]"));
        assert!(formatted.contains("[ITALIC]# inline comment[/ITALIC]"));
        
        // Check that non-comment code is not affected
        assert!(formatted.contains("echo \"Hello\""));
        assert!(formatted.contains("ls -la"));
    }

    #[test]
    fn test_format_code_includes_italic_comments() {
        let strategy = BashStrategy::new();
        let code = r#"# Comment at start
echo "test" # inline comment"#;
        
        let result = strategy.format_code(code);
        assert!(result.is_ok());
        
        let formatted = result.unwrap();
        assert!(formatted.contains("[ITALIC]# Comment at start[/ITALIC]"));
        assert!(formatted.contains("[ITALIC]# inline comment[/ITALIC]"));
    }

    #[test]
    fn test_process_with_formatting_includes_italic_comments() {
        let strategy = BashStrategy::new();
        let config = ProcessingConfig::default()
            .with_formatting(true);
        
        let code = r#"# This is a comment
echo "Hello World" # inline comment"#;
        
        let result = strategy.process(code, &config);
        assert!(result.is_ok());
        
        let processed = result.unwrap();
        assert!(processed.processed_code.is_some());
        
        let formatted = processed.processed_code.unwrap();
        assert!(formatted.contains("[ITALIC]# This is a comment[/ITALIC]"));
        assert!(formatted.contains("[ITALIC]# inline comment[/ITALIC]"));
    }

    #[test]
    fn test_bash_strategy_no_processing() {
        let strategy = BashStrategy::new();
        let config = ProcessingConfig::default()
            .with_formatting(false);
        
        let code = r#"echo "Hello""#;
        
        let result = strategy.process(code, &config);
        assert!(result.is_ok());
        
        let processed = result.unwrap();
        assert!(!processed.metadata.is_formatted);
        assert!(processed.processed_code.is_none());
    }

    #[test]
    fn test_language_strategy_trait_methods() {
        let strategy = BashStrategy::new();
        
        let extensions = strategy.get_file_extensions();
        assert!(extensions.contains(&"sh"));
        assert!(extensions.contains(&"bash"));
        
        let aliases = strategy.get_language_aliases();
        assert!(aliases.contains(&"bash"));
        assert!(aliases.contains(&"sh"));
        assert!(aliases.contains(&"shell"));
    }

    #[test]
    fn test_bash_strategy_supports_language_case_insensitive() {
        let strategy = BashStrategy::new();
        
        assert!(strategy.supports_language("bash"));
        assert!(strategy.supports_language("BASH"));
        assert!(strategy.supports_language("Bash"));
        assert!(strategy.supports_language("sh"));
        assert!(strategy.supports_language("SH"));
    }
}
