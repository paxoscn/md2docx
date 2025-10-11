//! Simple Plugin Example
//! 
//! This example demonstrates how to create a minimal plugin for processing
//! a custom language called "SimpleScript".

use std::collections::HashMap;
use std::time::Instant;

// Note: Adjust imports based on actual module structure
use md2docx_converter::markdown::code_block::{
    CodeBlockPlugin, CodeBlockStrategy, PluginError, ProcessedCodeBlock,
    ProcessingConfig, ProcessingError, ProcessingMetadata, ProcessingWarning
};

/// A simple plugin for processing "SimpleScript" code blocks
#[derive(Debug)]
pub struct SimpleScriptPlugin;

impl SimpleScriptPlugin {
    pub fn new() -> Self {
        Self
    }
}

impl CodeBlockPlugin for SimpleScriptPlugin {
    fn name(&self) -> &str {
        "simplescript-processor"
    }
    
    fn version(&self) -> &str {
        "1.0.0"
    }
    
    fn description(&self) -> &str {
        "Simple processor for SimpleScript language"
    }
    
    fn author(&self) -> &str {
        "Example Developer <dev@example.com>"
    }
    
    fn create_strategy(&self) -> Result<Box<dyn CodeBlockStrategy>, PluginError> {
        Ok(Box::new(SimpleScriptStrategy::new()))
    }
    
    fn supported_languages(&self) -> Vec<String> {
        vec![
            "simplescript".to_string(),
            "simple".to_string(),
            "ss".to_string(),
        ]
    }
    
    fn initialize(&self) -> Result<(), PluginError> {
        println!("Initializing SimpleScript plugin");
        Ok(())
    }
    
    fn cleanup(&self) -> Result<(), PluginError> {
        println!("Cleaning up SimpleScript plugin");
        Ok(())
    }
}

/// Strategy for processing SimpleScript code
#[derive(Debug, Clone)]
pub struct SimpleScriptStrategy;

impl SimpleScriptStrategy {
    pub fn new() -> Self {
        Self
    }
    
    /// Simple syntax validation for SimpleScript
    /// 
    /// SimpleScript rules:
    /// - Lines starting with "print" are print statements
    /// - Lines starting with "set" are variable assignments
    /// - Lines starting with "#" are comments
    /// - Empty lines are allowed
    fn validate_syntax(&self, code: &str) -> Result<bool, ProcessingError> {
        for (line_num, line) in code.lines().enumerate() {
            let trimmed = line.trim();
            
            // Skip empty lines and comments
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }
            
            // Check for valid statement types
            if !trimmed.starts_with("print ") && 
               !trimmed.starts_with("set ") &&
               !trimmed.starts_with("if ") &&
               !trimmed.starts_with("end") {
                return Err(ProcessingError::syntax_error(
                    &format!("Invalid statement: '{}'. Expected 'print', 'set', 'if', or 'end'", trimmed),
                    Some(line_num + 1),
                    None
                ));
            }
            
            // Check for basic syntax requirements
            if trimmed.starts_with("print ") && trimmed.len() < 7 {
                return Err(ProcessingError::syntax_error(
                    "Print statement requires a value",
                    Some(line_num + 1),
                    None
                ));
            }
            
            if trimmed.starts_with("set ") && !trimmed.contains('=') {
                return Err(ProcessingError::syntax_error(
                    "Set statement requires '=' for assignment",
                    Some(line_num + 1),
                    None
                ));
            }
        }
        
        Ok(true)
    }
    
    /// Simple formatting for SimpleScript
    fn format_code(&self, code: &str) -> Result<String, ProcessingError> {
        let lines: Vec<&str> = code.lines().collect();
        let mut formatted_lines = Vec::new();
        let mut indent_level = 0;
        
        for line in lines {
            let trimmed = line.trim();
            
            if trimmed.is_empty() {
                formatted_lines.push(String::new());
                continue;
            }
            
            // Decrease indent for 'end' statements
            if trimmed == "end" {
                indent_level = indent_level.saturating_sub(1);
            }
            
            // Apply indentation
            let indented = if trimmed.starts_with('#') {
                // Comments don't get indented
                trimmed.to_string()
            } else {
                format!("{}{}", "  ".repeat(indent_level), trimmed)
            };
            
            formatted_lines.push(indented);
            
            // Increase indent after 'if' statements
            if trimmed.starts_with("if ") {
                indent_level += 1;
            }
        }
        
        Ok(formatted_lines.join("\n"))
    }
    
    /// Check for code quality issues
    fn check_quality(&self, code: &str) -> Vec<ProcessingWarning> {
        let mut warnings = Vec::new();
        
        // Check for very long lines
        for (line_num, line) in code.lines().enumerate() {
            if line.len() > 80 {
                warnings.push(ProcessingWarning::new(
                    "style",
                    &format!("Line {} is longer than 80 characters", line_num + 1)
                ));
            }
        }
        
        // Check for missing comments
        let total_lines = code.lines().filter(|line| !line.trim().is_empty()).count();
        let comment_lines = code.lines().filter(|line| line.trim().starts_with('#')).count();
        
        if total_lines > 5 && comment_lines == 0 {
            warnings.push(ProcessingWarning::new(
                "documentation",
                "Consider adding comments to explain your code"
            ));
        }
        
        warnings
    }
}

impl CodeBlockStrategy for SimpleScriptStrategy {
    fn process(&self, code: &str, config: &ProcessingConfig) -> Result<ProcessedCodeBlock, ProcessingError> {
        let start_time = Instant::now();
        let mut errors = Vec::new();
        let mut warnings = Vec::new();
        
        // Validate syntax if enabled
        let syntax_valid = if config.enable_syntax_validation {
            match self.validate_syntax(code) {
                Ok(valid) => valid,
                Err(e) => {
                    errors.push(e);
                    false
                }
            }
        } else {
            true
        };
        
        // Format code if enabled and syntax is valid
        let formatted_code = if config.enable_formatting && syntax_valid {
            match self.format_code(code) {
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
            warnings.extend(self.check_quality(code));
        }
        
        // Create metadata
        let mut metadata = ProcessingMetadata::new(self.get_version())
            .with_processing_time(start_time.elapsed());
        
        metadata.is_formatted = formatted_code.is_some();
        metadata.is_validated = config.enable_syntax_validation;
        metadata.syntax_valid = syntax_valid;
        
        // Add custom attributes
        metadata = metadata.with_custom_attribute("language", "simplescript");
        metadata = metadata.with_custom_attribute("processor", "simple_example");
        
        // Build the result
        let mut processed = ProcessedCodeBlock::new(code.to_string(), Some("simplescript".to_string()))
            .with_metadata(metadata);
        
        if let Some(formatted) = formatted_code {
            processed = processed.with_processed_code(formatted);
        }
        
        for error in errors {
            processed = processed.with_error(error);
        }
        
        for warning in warnings {
            processed = processed.with_warning(warning);
        }
        
        Ok(processed)
    }
    
    fn supports_language(&self, language: &str) -> bool {
        matches!(language.to_lowercase().as_str(), "simplescript" | "simple" | "ss")
    }
    
    fn get_language_name(&self) -> &'static str {
        "simplescript"
    }
    
    fn get_priority(&self) -> u8 {
        100
    }
    
    fn get_version(&self) -> &'static str {
        "1.0.0"
    }
    
    fn get_description(&self) -> &'static str {
        "Simple processor for SimpleScript language with basic validation and formatting"
    }
}

/// Example usage of the SimpleScript plugin
pub fn example_usage() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== SimpleScript Plugin Example ===");
    
    // Create the plugin
    let plugin = SimpleScriptPlugin::new();
    println!("Created plugin: {} v{}", plugin.name(), plugin.version());
    
    // Initialize the plugin
    plugin.initialize()?;
    
    // Create a strategy
    let strategy = plugin.create_strategy()?;
    
    // Test with valid SimpleScript code
    let valid_code = r#"
# This is a SimpleScript program
set x = 10
set y = 20
print x
print y
if x > 5
  print "x is greater than 5"
end
"#;
    
    let config = ProcessingConfig::default()
        .with_syntax_validation(true)
        .with_formatting(true);
    
    println!("\nProcessing valid SimpleScript code:");
    match strategy.process(valid_code, &config) {
        Ok(result) => {
            println!("✅ Processing successful!");
            println!("  Syntax valid: {}", result.metadata.syntax_valid);
            println!("  Was formatted: {}", result.metadata.is_formatted);
            println!("  Processing time: {:?}", result.metadata.processing_time);
            
            if let Some(formatted) = &result.processed_code {
                println!("  Formatted code:\n{}", formatted);
            }
            
            if result.has_warnings() {
                println!("  Warnings:");
                for warning in &result.warnings {
                    println!("    - {}: {}", warning.warning_type, warning.message);
                }
            }
        }
        Err(e) => {
            println!("❌ Processing failed: {}", e);
        }
    }
    
    // Test with invalid SimpleScript code
    let invalid_code = r#"
invalid_statement
set x
print
"#;
    
    println!("\nProcessing invalid SimpleScript code:");
    match strategy.process(invalid_code, &config) {
        Ok(result) => {
            if result.is_successful() {
                println!("✅ Processing successful (unexpected)");
            } else {
                println!("❌ Processing failed as expected:");
                for error in &result.errors {
                    println!("  - Line {}: {}", 
                        error.line.unwrap_or(0), 
                        error.message);
                }
            }
        }
        Err(e) => {
            println!("💥 Unexpected error: {}", e);
        }
    }
    
    // Cleanup
    plugin.cleanup()?;
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_creation() {
        let plugin = SimpleScriptPlugin::new();
        assert_eq!(plugin.name(), "simplescript-processor");
        assert_eq!(plugin.version(), "1.0.0");
        assert!(plugin.supported_languages().contains(&"simplescript".to_string()));
    }

    #[test]
    fn test_strategy_creation() {
        let plugin = SimpleScriptPlugin::new();
        let strategy = plugin.create_strategy().unwrap();
        
        assert_eq!(strategy.get_language_name(), "simplescript");
        assert!(strategy.supports_language("simplescript"));
        assert!(strategy.supports_language("simple"));
        assert!(strategy.supports_language("ss"));
        assert!(!strategy.supports_language("python"));
    }

    #[test]
    fn test_syntax_validation() {
        let strategy = SimpleScriptStrategy::new();
        
        // Valid code
        let valid = "print hello\nset x = 5";
        assert!(strategy.validate_syntax(valid).unwrap());
        
        // Invalid code
        let invalid = "invalid_statement";
        assert!(strategy.validate_syntax(invalid).is_err());
    }

    #[test]
    fn test_code_formatting() {
        let strategy = SimpleScriptStrategy::new();
        
        let unformatted = "print hello\nif x > 5\nprint greater\nend";
        let formatted = strategy.format_code(unformatted).unwrap();
        
        assert!(formatted.contains("  print greater")); // Should be indented
    }

    #[test]
    fn test_processing_pipeline() {
        let strategy = SimpleScriptStrategy::new();
        let config = ProcessingConfig::default()
            .with_syntax_validation(true)
            .with_formatting(true);
        
        let code = "print hello\nset x = 5";
        let result = strategy.process(code, &config).unwrap();
        
        assert!(result.is_successful());
        assert!(result.metadata.syntax_valid);
        assert!(result.metadata.is_validated);
    }
}
fn main
() -> Result<(), Box<dyn std::error::Error>> {
    println!("Simple Plugin Example");
    Ok(())
}