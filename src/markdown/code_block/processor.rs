//! Core data structures for processed code blocks

use std::collections::HashMap;
use std::time::Duration;
use crate::markdown::code_block::{ProcessingError, ProcessingWarning};

/// Represents a processed code block with metadata and results
#[derive(Debug, Clone)]
pub struct ProcessedCodeBlock {
    pub original_code: String,
    pub processed_code: Option<String>,
    pub language: Option<String>,
    pub metadata: ProcessingMetadata,
    pub errors: Vec<ProcessingError>,
    pub warnings: Vec<ProcessingWarning>,
}

/// Metadata about the processing operation
#[derive(Debug, Clone)]
pub struct ProcessingMetadata {
    pub is_formatted: bool,
    pub is_validated: bool,
    pub syntax_valid: bool,
    pub processing_time: Duration,
    pub processor_version: String,
    pub custom_attributes: HashMap<String, String>,
}

impl ProcessedCodeBlock {
    /// Create a new processed code block
    pub fn new(original_code: String, language: Option<String>) -> Self {
        Self {
            original_code,
            processed_code: None,
            language,
            metadata: ProcessingMetadata::default(),
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }

    /// Create a processed code block with no processing applied
    pub fn unprocessed(original_code: String, language: Option<String>) -> Self {
        let mut block = Self::new(original_code, language);
        block.metadata.processing_time = Duration::from_millis(0);
        block
    }

    /// Set the processed code
    pub fn with_processed_code(mut self, processed_code: String) -> Self {
        self.processed_code = Some(processed_code);
        self.metadata.is_formatted = true;
        self
    }

    /// Mark as validated
    pub fn with_validation(mut self, syntax_valid: bool) -> Self {
        self.metadata.is_validated = true;
        self.metadata.syntax_valid = syntax_valid;
        self
    }

    /// Add an error
    pub fn with_error(mut self, error: ProcessingError) -> Self {
        self.errors.push(error);
        self
    }

    /// Add a warning
    pub fn with_warning(mut self, warning: ProcessingWarning) -> Self {
        self.warnings.push(warning);
        self
    }

    /// Set processing metadata
    pub fn with_metadata(mut self, metadata: ProcessingMetadata) -> Self {
        self.metadata = metadata;
        self
    }

    /// Get the final code (processed if available, otherwise original)
    pub fn get_final_code(&self) -> &str {
        self.processed_code.as_ref().unwrap_or(&self.original_code)
    }

    /// Check if processing was successful (no errors)
    pub fn is_successful(&self) -> bool {
        self.errors.is_empty()
    }

    /// Check if there are any warnings
    pub fn has_warnings(&self) -> bool {
        !self.warnings.is_empty()
    }

    /// Get the number of errors
    pub fn error_count(&self) -> usize {
        self.errors.len()
    }

    /// Get the number of warnings
    pub fn warning_count(&self) -> usize {
        self.warnings.len()
    }

    /// Check if the code was actually modified during processing
    pub fn was_modified(&self) -> bool {
        match &self.processed_code {
            Some(processed) => processed != &self.original_code,
            None => false,
        }
    }

    /// Get a summary of the processing results
    pub fn get_summary(&self) -> ProcessingSummary {
        ProcessingSummary {
            language: self.language.clone(),
            was_processed: self.processed_code.is_some(),
            was_modified: self.was_modified(),
            is_valid: self.metadata.syntax_valid,
            error_count: self.error_count(),
            warning_count: self.warning_count(),
            processing_time: self.metadata.processing_time,
        }
    }
}

impl ProcessingMetadata {
    /// Create new processing metadata
    pub fn new(processor_version: &str) -> Self {
        Self {
            is_formatted: false,
            is_validated: false,
            syntax_valid: true,
            processing_time: Duration::from_millis(0),
            processor_version: processor_version.to_string(),
            custom_attributes: HashMap::new(),
        }
    }

    /// Set processing time
    pub fn with_processing_time(mut self, duration: Duration) -> Self {
        self.processing_time = duration;
        self
    }

    /// Add a custom attribute
    pub fn with_custom_attribute(mut self, key: &str, value: &str) -> Self {
        self.custom_attributes.insert(key.to_string(), value.to_string());
        self
    }

    /// Get a custom attribute
    pub fn get_custom_attribute(&self, key: &str) -> Option<&String> {
        self.custom_attributes.get(key)
    }

    /// Check if a custom attribute exists
    pub fn has_custom_attribute(&self, key: &str) -> bool {
        self.custom_attributes.contains_key(key)
    }
}

impl Default for ProcessingMetadata {
    fn default() -> Self {
        Self::new("1.0.0")
    }
}

/// Summary of processing results
#[derive(Debug, Clone)]
pub struct ProcessingSummary {
    pub language: Option<String>,
    pub was_processed: bool,
    pub was_modified: bool,
    pub is_valid: bool,
    pub error_count: usize,
    pub warning_count: usize,
    pub processing_time: Duration,
}

impl ProcessingSummary {
    /// Check if processing was completely successful
    pub fn is_successful(&self) -> bool {
        self.error_count == 0
    }

    /// Check if there were any issues (errors or warnings)
    pub fn has_issues(&self) -> bool {
        self.error_count > 0 || self.warning_count > 0
    }

    /// Get a human-readable status
    pub fn get_status(&self) -> &'static str {
        if self.error_count > 0 {
            "failed"
        } else if self.warning_count > 0 {
            "warning"
        } else if self.was_processed {
            "success"
        } else {
            "skipped"
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::markdown::code_block::{ProcessingError, ProcessingWarning};

    #[test]
    fn test_processed_code_block_creation() {
        let block = ProcessedCodeBlock::new(
            "fn main() {}".to_string(),
            Some("rust".to_string())
        );

        assert_eq!(block.original_code, "fn main() {}");
        assert_eq!(block.language, Some("rust".to_string()));
        assert!(block.processed_code.is_none());
        assert!(block.errors.is_empty());
        assert!(block.warnings.is_empty());
    }

    #[test]
    fn test_unprocessed_code_block() {
        let block = ProcessedCodeBlock::unprocessed(
            "console.log('hello')".to_string(),
            Some("javascript".to_string())
        );

        assert_eq!(block.get_final_code(), "console.log('hello')");
        assert!(block.is_successful());
        assert!(!block.has_warnings());
    }

    #[test]
    fn test_processed_code_block_with_changes() {
        let block = ProcessedCodeBlock::new(
            "fn main(){println!(\"hello\");}".to_string(),
            Some("rust".to_string())
        )
        .with_processed_code("fn main() {\n    println!(\"hello\");\n}".to_string())
        .with_validation(true);

        assert!(block.was_modified());
        assert!(block.metadata.is_formatted);
        assert!(block.metadata.is_validated);
        assert!(block.metadata.syntax_valid);
        assert_eq!(block.get_final_code(), "fn main() {\n    println!(\"hello\");\n}");
    }

    #[test]
    fn test_code_block_with_errors() {
        let error = ProcessingError::syntax_error("Missing semicolon", Some(1), Some(10));
        let warning = ProcessingWarning::formatting_warning("Inconsistent indentation");

        let block = ProcessedCodeBlock::new(
            "invalid code".to_string(),
            Some("rust".to_string())
        )
        .with_error(error)
        .with_warning(warning)
        .with_validation(false);

        assert!(!block.is_successful());
        assert!(block.has_warnings());
        assert_eq!(block.error_count(), 1);
        assert_eq!(block.warning_count(), 1);
        assert!(!block.metadata.syntax_valid);
    }

    #[test]
    fn test_processing_metadata() {
        let metadata = ProcessingMetadata::new("2.0.0")
            .with_processing_time(Duration::from_millis(150))
            .with_custom_attribute("formatter", "rustfmt");

        assert_eq!(metadata.processor_version, "2.0.0");
        assert_eq!(metadata.processing_time, Duration::from_millis(150));
        assert_eq!(metadata.get_custom_attribute("formatter"), Some(&"rustfmt".to_string()));
        assert!(metadata.has_custom_attribute("formatter"));
        assert!(!metadata.has_custom_attribute("nonexistent"));
    }

    #[test]
    fn test_processing_summary() {
        let block = ProcessedCodeBlock::new(
            "fn main() {}".to_string(),
            Some("rust".to_string())
        )
        .with_processed_code("fn main() {\n}".to_string())
        .with_validation(true);

        let summary = block.get_summary();
        assert_eq!(summary.language, Some("rust".to_string()));
        assert!(summary.was_processed);
        assert!(summary.was_modified);
        assert!(summary.is_valid);
        assert_eq!(summary.error_count, 0);
        assert_eq!(summary.warning_count, 0);
        assert!(summary.is_successful());
        assert!(!summary.has_issues());
        assert_eq!(summary.get_status(), "success");
    }

    #[test]
    fn test_summary_with_errors() {
        let error = ProcessingError::syntax_error("Invalid syntax", None, None);
        let block = ProcessedCodeBlock::new(
            "invalid".to_string(),
            Some("rust".to_string())
        )
        .with_error(error);

        let summary = block.get_summary();
        assert!(!summary.is_successful());
        assert!(summary.has_issues());
        assert_eq!(summary.get_status(), "failed");
    }

    #[test]
    fn test_summary_with_warnings() {
        let warning = ProcessingWarning::formatting_warning("Style issue");
        let block = ProcessedCodeBlock::new(
            "fn main() {}".to_string(),
            Some("rust".to_string())
        )
        .with_processed_code("fn main() {}".to_string())
        .with_warning(warning);

        let summary = block.get_summary();
        assert!(summary.is_successful());
        assert!(summary.has_issues());
        assert_eq!(summary.get_status(), "warning");
    }

    #[test]
    fn test_unmodified_processed_code() {
        let original = "fn main() {}".to_string();
        let block = ProcessedCodeBlock::new(original.clone(), Some("rust".to_string()))
            .with_processed_code(original);

        assert!(!block.was_modified());
        let summary = block.get_summary();
        assert!(!summary.was_modified);
    }
}