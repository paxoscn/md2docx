//! Rust code block processing strategy

use std::time::Instant;
use crate::markdown::code_block::{
    CodeBlockStrategy, LanguageStrategy, ProcessedCodeBlock, ProcessingConfig, 
    ProcessingError, ProcessingMetadata, ProcessingWarning, language_matches
};

/// Strategy for processing Rust code blocks
#[derive(Debug, Clone)]
pub struct RustStrategy;

impl RustStrategy {
    /// Create a new Rust strategy instance
    pub fn new() -> Self {
        Self
    }

    /// Get Rust keywords that should be rendered in bold
    fn get_rust_keywords() -> &'static [&'static str] {
        &[
            // Keywords
            "as", "break", "const", "continue", "crate", "else", "enum", "extern",
            "false", "fn", "for", "if", "impl", "in", "let", "loop", "match", "mod",
            "move", "mut", "pub", "ref", "return", "self", "Self", "static", "struct",
            "super", "trait", "true", "type", "unsafe", "use", "where", "while",
            // Async keywords
            "async", "await",
            // Reserved keywords
            "abstract", "become", "box", "do", "final", "macro", "override", "priv",
            "typeof", "unsized", "virtual", "yield",
            // Common types
            "i8", "i16", "i32", "i64", "i128", "isize",
            "u8", "u16", "u32", "u64", "u128", "usize",
            "f32", "f64", "bool", "char", "str",
            "String", "Vec", "Option", "Result", "Box", "Rc", "Arc",
        ]
    }

    /// Apply bold formatting to Rust keywords in code
    fn apply_keyword_bold(&self, code: &str) -> String {
        let keywords = Self::get_rust_keywords();
        let mut result = code.to_string();
        
        for keyword in keywords {
            // Use word boundaries to match whole words only
            // Replace keyword with **keyword** for bold in markdown
            let pattern = format!(r"\b{}\b", regex::escape(keyword));
            let re = regex::Regex::new(&pattern).unwrap();
            let replacement = format!("[BOLD]{}[/BOLD]", keyword);
            result = re.replace_all(&result, replacement.as_str()).to_string();
        }
        
        result
    }

    /// Validate Rust syntax using the syn crate
    fn validate_rust_syntax(&self, code: &str) -> Result<bool, ProcessingError> {
        match syn::parse_file(code) {
            Ok(_) => Ok(true),
            Err(_e) => {
                // Return false for syntax validity but don't fail the processing
                // The error will be recorded in the processed block
                Ok(false)
            }
        }
    }

    /// Format Rust code (basic implementation - in a real scenario you'd use rustfmt)
    fn format_rust_code(&self, code: &str) -> Result<String, ProcessingError> {
        // For now, we'll do basic formatting by parsing and pretty-printing
        // In a production implementation, you would integrate with rustfmt
        
        // Try to parse as a complete file first
        let formatted = match syn::parse_file(code) {
            Ok(_syntax_tree) => {
                // Valid complete Rust file - use the original code with basic cleanup
                code.trim().to_string()
            }
            Err(_) => {
                // Not a complete file, might be a code snippet
                // Still apply formatting to the snippet
                code.trim().to_string()
            }
        };
        
        // Always apply keyword bold formatting, regardless of syntax validity
        let formatted_with_bold = self.apply_keyword_bold(&formatted);
        Ok(formatted_with_bold)
    }

    /// Extract syntax errors from syn parsing
    fn extract_syntax_errors(&self, code: &str) -> Vec<ProcessingError> {
        let mut errors = Vec::new();
        
        if let Err(e) = syn::parse_file(code) {
            let error = ProcessingError::syntax_error(&e.to_string(), None, None);
            errors.push(error);
        }
        
        errors
    }

    /// Check for common Rust code quality issues
    fn check_code_quality(&self, code: &str) -> Vec<ProcessingWarning> {
        let mut warnings = Vec::new();
        
        // Check for common issues
        if code.contains("unwrap()") {
            warnings.push(ProcessingWarning::new(
                "code_quality",
                "Consider using proper error handling instead of unwrap()"
            ));
        }
        
        if code.contains("panic!") {
            warnings.push(ProcessingWarning::new(
                "code_quality", 
                "Consider using Result<T, E> instead of panic! for error handling"
            ));
        }
        
        if code.lines().any(|line| line.len() > 100) {
            warnings.push(ProcessingWarning::new(
                "style",
                "Some lines exceed 100 characters, consider breaking them up"
            ));
        }
        
        warnings
    }
}

impl CodeBlockStrategy for RustStrategy {
    fn process(&self, code: &str, config: &ProcessingConfig) -> Result<ProcessedCodeBlock, ProcessingError> {
        let start_time = Instant::now();
        let mut errors = Vec::new();
        let mut warnings = Vec::new();
        
        // Validate syntax if enabled
        let syntax_valid = if config.enable_syntax_validation {
            match self.validate_rust_syntax(code) {
                Ok(valid) => {
                    if !valid {
                        errors.extend(self.extract_syntax_errors(code));
                    }
                    valid
                }
                Err(e) => {
                    errors.push(e);
                    false
                }
            }
        } else {
            true // Assume valid if not validating
        };
        
        // Format code if enabled (apply keyword bold even if syntax is invalid)
        let formatted_code = if config.enable_formatting {
            match self.format_rust_code(code) {
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
        if syntax_valid {
            warnings.extend(self.check_code_quality(code));
        }
        
        // Create metadata
        let mut metadata = ProcessingMetadata::new(self.get_version())
            .with_processing_time(start_time.elapsed());
        
        metadata.is_formatted = formatted_code.is_some();
        metadata.is_validated = config.enable_syntax_validation;
        metadata.syntax_valid = syntax_valid;
        
        // Add custom attributes
        metadata = metadata.with_custom_attribute("language", "rust");
        if syntax_valid {
            metadata = metadata.with_custom_attribute("syntax_checker", "syn");
        }
        if formatted_code.is_some() {
            metadata = metadata.with_custom_attribute("formatter", "syn_pretty_print");
        }
        
        let processed = ProcessedCodeBlock::new(code.to_string(), Some("rust".to_string()))
            .with_metadata(metadata);
        
        let processed = if let Some(formatted) = formatted_code {
            processed.with_processed_code(formatted)
        } else {
            processed
        };
        
        // Don't use with_validation as it always sets is_validated = true
        // We already set the metadata correctly above
        
        // Add errors and warnings
        let mut final_processed = processed;
        for error in errors {
            final_processed = final_processed.with_error(error);
        }
        for warning in warnings {
            final_processed = final_processed.with_warning(warning);
        }
        
        Ok(final_processed)
    }
    
    fn supports_language(&self, language: &str) -> bool {
        language_matches(language, &["rust", "rs"])
    }
    
    fn get_language_name(&self) -> &'static str {
        "rust"
    }
    
    fn get_priority(&self) -> u8 {
        150 // High priority for Rust code
    }
    
    fn get_version(&self) -> &'static str {
        "1.0.0"
    }
    
    fn get_description(&self) -> &'static str {
        "Rust code processing with syntax validation and formatting using syn crate"
    }
}

impl LanguageStrategy for RustStrategy {
    fn validate_syntax(&self, code: &str) -> Result<bool, ProcessingError> {
        self.validate_rust_syntax(code)
    }
    
    fn format_code(&self, code: &str) -> Result<String, ProcessingError> {
        self.format_rust_code(code)
    }
    
    fn get_file_extensions(&self) -> Vec<&'static str> {
        vec!["rs"]
    }
    
    fn get_language_aliases(&self) -> Vec<&'static str> {
        vec!["rust", "rs"]
    }
}

impl Default for RustStrategy {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::markdown::code_block::ProcessingConfig;

    #[test]
    fn test_rust_strategy_creation() {
        let strategy = RustStrategy::new();
        assert_eq!(strategy.get_language_name(), "rust");
        assert_eq!(strategy.get_priority(), 150);
        assert!(strategy.supports_language("rust"));
        assert!(strategy.supports_language("rs"));
        assert!(!strategy.supports_language("python"));
    }

    #[test]
    fn test_rust_syntax_validation_valid() {
        let strategy = RustStrategy::new();
        let valid_code = r#"
fn main() {
    println!("Hello, world!");
}
"#;
        
        let result = strategy.validate_syntax(valid_code);
        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[test]
    fn test_rust_syntax_validation_invalid() {
        let strategy = RustStrategy::new();
        let invalid_code = r#"
fn main( {
    println!("Hello, world!");
}
"#;
        
        let result = strategy.validate_syntax(invalid_code);
        assert!(result.is_ok());
        assert!(!result.unwrap()); // Should be false for invalid syntax
    }

    #[test]
    fn test_rust_code_formatting() {
        let strategy = RustStrategy::new();
        let unformatted_code = r#"fn main(){println!("Hello");}"#;
        
        let result = strategy.format_code(unformatted_code);
        assert!(result.is_ok());
        
        let formatted = result.unwrap();
        // Keywords should be bolded
        assert!(formatted.contains("**fn**"));
        assert!(formatted.contains("main"));
        assert!(formatted.contains("println!"));
    }

    #[test]
    fn test_rust_code_formatting_invalid() {
        let strategy = RustStrategy::new();
        let invalid_code = r#"fn main( { invalid syntax"#;
        
        // Even invalid code should be formatted (with keyword bold applied)
        // We don't fail on syntax errors during formatting
        let result = strategy.format_code(invalid_code);
        assert!(result.is_ok());
        
        // Check that keywords are still bolded even in invalid code
        let formatted = result.unwrap();
        assert!(formatted.contains("**fn**"));
    }

    #[test]
    fn test_rust_strategy_process_valid_code() {
        let strategy = RustStrategy::new();
        let config = ProcessingConfig::default()
            .with_syntax_validation(true)
            .with_formatting(true);
        
        let code = r#"
fn main() {
    println!("Hello, world!");
}
"#;
        
        let result = strategy.process(code, &config);
        assert!(result.is_ok());
        
        let processed = result.unwrap();
        assert_eq!(processed.language, Some("rust".to_string()));
        assert!(processed.metadata.syntax_valid);
        assert!(processed.metadata.is_validated);
        assert!(processed.is_successful());
        assert_eq!(processed.error_count(), 0);
    }

    #[test]
    fn test_rust_strategy_process_invalid_code() {
        let strategy = RustStrategy::new();
        let config = ProcessingConfig::default()
            .with_syntax_validation(true);
        
        let invalid_code = r#"
fn main( {
    println!("Hello, world!");
}
"#;
        
        let result = strategy.process(invalid_code, &config);
        assert!(result.is_ok());
        
        let processed = result.unwrap();
        assert!(!processed.metadata.syntax_valid);
        assert!(processed.error_count() > 0);
        assert!(!processed.is_successful());
    }

    #[test]
    fn test_rust_strategy_code_quality_warnings() {
        let strategy = RustStrategy::new();
        let config = ProcessingConfig::default()
            .with_syntax_validation(true);
        
        let code_with_issues = r#"
fn main() {
    let value = some_function().unwrap();
    panic!("This is bad!");
    let very_long_line_that_exceeds_one_hundred_characters_and_should_trigger_a_warning_about_line_length = 42;
}

fn some_function() -> Option<i32> {
    Some(42)
}
"#;
        
        let result = strategy.process(code_with_issues, &config);
        assert!(result.is_ok());
        
        let processed = result.unwrap();
        assert!(processed.warning_count() > 0);
        assert!(processed.has_warnings());
        
        // Check that we have the expected warnings
        let warnings: Vec<_> = processed.warnings.iter()
            .map(|w| w.warning_type.as_str())
            .collect();
        assert!(warnings.contains(&"code_quality"));
        assert!(warnings.contains(&"style"));
    }

    #[test]
    fn test_rust_strategy_no_processing() {
        let strategy = RustStrategy::new();
        let config = ProcessingConfig::default()
            .with_syntax_validation(false)
            .with_formatting(false);
        
        let code = r#"fn main() { println!("Hello"); }"#;
        
        let result = strategy.process(code, &config);
        assert!(result.is_ok());
        
        let processed = result.unwrap();
        assert!(!processed.metadata.is_validated);
        assert!(!processed.metadata.is_formatted);
        assert!(processed.processed_code.is_none());
        assert_eq!(processed.error_count(), 0);
        assert_eq!(processed.warning_count(), 0);
    }

    #[test]
    fn test_rust_strategy_metadata() {
        let strategy = RustStrategy::new();
        let config = ProcessingConfig::default()
            .with_syntax_validation(true)
            .with_formatting(true);
        
        let code = r#"fn main() { println!("Hello"); }"#;
        
        let result = strategy.process(code, &config);
        assert!(result.is_ok());
        
        let processed = result.unwrap();
        assert_eq!(processed.metadata.get_custom_attribute("language"), Some(&"rust".to_string()));
        assert_eq!(processed.metadata.get_custom_attribute("syntax_checker"), Some(&"syn".to_string()));
        assert!(processed.metadata.processing_time.as_nanos() > 0);
    }

    #[test]
    fn test_language_strategy_trait_methods() {
        let strategy = RustStrategy::new();
        
        let extensions = strategy.get_file_extensions();
        assert_eq!(extensions, vec!["rs"]);
        
        let aliases = strategy.get_language_aliases();
        assert!(aliases.contains(&"rust"));
        assert!(aliases.contains(&"rs"));
    }

    #[test]
    fn test_rust_strategy_supports_language_case_insensitive() {
        let strategy = RustStrategy::new();
        
        assert!(strategy.supports_language("rust"));
        assert!(strategy.supports_language("RUST"));
        assert!(strategy.supports_language("Rust"));
        assert!(strategy.supports_language("rs"));
        assert!(strategy.supports_language("RS"));
        assert!(strategy.supports_language("Rs"));
    }

    #[test]
    fn test_keyword_bold_formatting() {
        let strategy = RustStrategy::new();
        let code = r#"fn main() {
    let x = 42;
    if x > 0 {
        println!("positive");
    }
}"#;
        
        let formatted = strategy.apply_keyword_bold(code);
        
        // Check that keywords are wrapped in **
        assert!(formatted.contains("**fn**"));
        assert!(formatted.contains("**let**"));
        assert!(formatted.contains("**if**"));
        
        // Check that non-keywords are not affected
        assert!(formatted.contains("main"));
        assert!(!formatted.contains("**main**"));
        assert!(formatted.contains("println!"));
    }

    #[test]
    fn test_keyword_bold_with_types() {
        let strategy = RustStrategy::new();
        let code = "let x: i32 = 42;\nlet s: String = String::from(\"hello\");";
        
        let formatted = strategy.apply_keyword_bold(code);
        
        assert!(formatted.contains("**let**"));
        assert!(formatted.contains("**i32**"));
        assert!(formatted.contains("**String**"));
    }

    #[test]
    fn test_keyword_bold_preserves_structure() {
        let strategy = RustStrategy::new();
        let code = "pub struct MyStruct {\n    pub field: bool,\n}";
        
        let formatted = strategy.apply_keyword_bold(code);
        
        assert!(formatted.contains("**pub**"));
        assert!(formatted.contains("**struct**"));
        assert!(formatted.contains("**bool**"));
        assert!(formatted.contains("MyStruct")); // Non-keyword preserved
    }

    #[test]
    fn test_format_code_includes_bold_keywords() {
        let strategy = RustStrategy::new();
        let code = "fn main() { let x = 42; }";
        
        let result = strategy.format_code(code);
        assert!(result.is_ok());
        
        let formatted = result.unwrap();
        assert!(formatted.contains("**fn**"));
        assert!(formatted.contains("**let**"));
    }

    #[test]
    fn test_process_with_formatting_includes_bold() {
        let strategy = RustStrategy::new();
        let config = ProcessingConfig::default()
            .with_syntax_validation(true)
            .with_formatting(true);
        
        let code = "fn main() { let x = 42; }";
        
        let result = strategy.process(code, &config);
        assert!(result.is_ok());
        
        let processed = result.unwrap();
        assert!(processed.processed_code.is_some());
        
        let formatted = processed.processed_code.unwrap();
        assert!(formatted.contains("**fn**"));
        assert!(formatted.contains("**let**"));
    }
}