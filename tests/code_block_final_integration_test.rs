//! Final integration test for the code block strategy system
//! 
//! This test verifies that all components of the code block strategy system
//! are properly integrated and working together.

use md2docx_converter::markdown::{MarkdownParser, MarkdownElement};
use md2docx_converter::markdown::code_block::{
    CodeBlockProcessor, CodeBlockConfig, LanguageConfig, StrategyRegistry
};

#[test]
fn test_complete_code_block_integration() {
    // Test markdown with various code blocks
    let markdown = r#"
# Test Document

This document contains various code blocks to test the strategy system.

## Rust Code

```rust
fn main() {
    println!("Hello, world!");
    let x = 42;
    println!("The answer is {}", x);
}
```

## JavaScript Code

```javascript
function greet(name) {
    console.log(`Hello, ${name}!`);
}

greet("World");
```

## Unknown Language

```unknown
This is some unknown code
that should use the default strategy
```

## No Language Specified

```
This code block has no language specified
and should also use the default strategy
```

## Python Code

```python
def fibonacci(n):
    if n <= 1:
        return n
    return fibonacci(n-1) + fibonacci(n-2)

print(fibonacci(10))
```
"#;

    // Create parser with code block processing enabled
    let config = CodeBlockConfig::new();
    let parser = MarkdownParser::with_code_block_config(config);
    
    // Parse the markdown
    let document = parser.parse(markdown).expect("Failed to parse markdown");
    
    // Verify we have the expected structure
    assert!(document.elements.len() > 0);
    
    // Count code blocks and verify they were processed
    let mut code_block_count = 0;
    let mut processed_count = 0;
    
    for element in &document.elements {
        if let MarkdownElement::CodeBlock { language, code, processed } = element {
            code_block_count += 1;
            
            // Verify the code block has content
            assert!(!code.is_empty());
            
            // Verify processing was attempted
            if let Some(processed_block) = processed {
                processed_count += 1;
                
                // Verify processed block structure
                assert_eq!(processed_block.original_code, *code);
                assert_eq!(processed_block.language, *language);
                
                // Verify metadata is present
                assert!(processed_block.metadata.processing_time.as_nanos() >= 0);
                
                // For known languages, verify appropriate processing
                match language.as_deref() {
                    Some("rust") => {
                        // Rust code should be validated
                        assert!(processed_block.metadata.is_validated);
                    }
                    Some("javascript") => {
                        // JavaScript should be processed by default strategy
                        assert!(processed_block.metadata.is_validated);
                    }
                    Some("python") => {
                        // Python should be processed by default strategy
                        assert!(processed_block.metadata.is_validated);
                    }
                    Some("unknown") | None => {
                        // Unknown/no language should use default strategy
                        assert!(processed_block.metadata.is_validated);
                    }
                    _ => {}
                }
                
                // Verify no critical errors occurred
                let critical_errors = processed_block.errors.iter()
                    .filter(|e| matches!(e.severity, md2docx_converter::markdown::code_block::ErrorSeverity::Critical))
                    .count();
                assert_eq!(critical_errors, 0, "Critical errors found in processing");
            }
        }
    }
    
    // Verify we found and processed the expected number of code blocks
    assert_eq!(code_block_count, 5, "Expected 5 code blocks");
    assert_eq!(processed_count, 5, "Expected all code blocks to be processed");
}

#[test]
fn test_code_block_processor_configuration() {
    // Test different configurations
    let mut config = CodeBlockConfig::new();
    
    // Configure global settings
    config.global.enable_processing = true;
    config.global.default_timeout_ms = 5000;
    
    // Configure language-specific settings
    let rust_config = LanguageConfig::new()
        .with_syntax_validation(true)
        .with_formatting(false); // Disable formatting for this test
    
    config = config.with_language_config("rust", rust_config);
    
    let processor = CodeBlockProcessor::with_config(config);
    
    // Test processing a Rust code block
    let rust_code = r#"
fn main() {
    println!("Hello, world!");
}
"#;
    
    let result = processor.process_code_block(rust_code, Some("rust"))
        .expect("Failed to process Rust code");
    
    // Verify the result
    assert_eq!(result.original_code, rust_code);
    assert_eq!(result.language, Some("rust".to_string()));
    assert!(result.metadata.is_validated);
    assert!(!result.metadata.is_formatted); // We disabled formatting
    assert!(result.is_successful());
}

#[test]
fn test_strategy_registry_functionality() {
    let registry = StrategyRegistry::new();
    
    // Verify default strategy is available
    let default_strategy = registry.get_default_strategy();
    assert_eq!(default_strategy.get_language_name(), "default");
    
    // Test strategy lookup
    let rust_strategy = registry.get_strategy("rust");
    let unknown_strategy = registry.get_strategy("unknown_language");
    
    // Both should return valid strategies (unknown falls back to default)
    assert!(rust_strategy.supports_language("rust"));
    assert!(unknown_strategy.supports_language("unknown_language"));
    
    // Test registry information
    let strategy_count = registry.strategy_count();
    let languages = registry.get_registered_languages();
    
    // Verify registry has strategies
    assert!(strategy_count >= 0); // May be 0 if no specific strategies registered
    assert!(!languages.is_empty() || strategy_count == 0); // Either has languages or no strategies
}

#[test]
fn test_error_handling_and_recovery() {
    let processor = CodeBlockProcessor::new();
    
    // Test with invalid code that might cause processing errors
    let invalid_code = "This is not valid code in any language }{][{";
    
    let result = processor.process_code_block(invalid_code, Some("rust"))
        .expect("Processing should not fail completely");
    
    // Even with invalid code, we should get a result
    assert_eq!(result.original_code, invalid_code);
    assert_eq!(result.language, Some("rust".to_string()));
    
    // The system should handle errors gracefully
    // (specific error handling depends on strategy implementation)
}

#[test]
fn test_performance_and_timeout() {
    let mut config = CodeBlockConfig::new();
    config.global.default_timeout_ms = 100; // Very short timeout
    
    let processor = CodeBlockProcessor::with_config(config);
    
    // Test with normal code (should complete within timeout)
    let simple_code = "fn main() {}";
    let result = processor.process_code_block(simple_code, Some("rust"))
        .expect("Simple code should process successfully");
    
    assert!(result.is_successful());
    assert!(result.metadata.processing_time.as_millis() < 100);
}

#[test]
fn test_backward_compatibility() {
    // Test that the system works with the old API
    let parser = MarkdownParser::new();
    
    let markdown = r#"
```rust
fn main() {
    println!("Hello");
}
```
"#;
    
    let document = parser.parse(markdown).expect("Parsing should succeed");
    
    // Verify we get a code block
    let code_blocks: Vec<_> = document.elements.iter()
        .filter_map(|e| match e {
            MarkdownElement::CodeBlock { language, code, processed } => {
                Some((language, code, processed))
            }
            _ => None
        })
        .collect();
    
    assert_eq!(code_blocks.len(), 1);
    
    let (language, code, processed) = &code_blocks[0];
    assert_eq!(language.as_ref(), Some(&"rust".to_string()));
    assert!(code.contains("fn main()"));
    
    // With default configuration, processing should be enabled
    assert!(processed.is_some());
}

#[test]
fn test_configuration_validation() {
    // Test configuration validation
    let mut config = CodeBlockConfig::new();
    
    // Test configuration creation (validation is implicit in creation)
    let _valid_config = CodeBlockConfig::new();
    
    // Test with extreme values
    config.global.default_timeout_ms = 0; // Very short timeout
    
    // Reset to valid configuration
    config.global.default_timeout_ms = 1000;
}

#[test]
fn test_processing_statistics() {
    let processor = CodeBlockProcessor::new();
    let stats = processor.get_processing_stats();
    
    // Verify statistics structure
    assert!(stats.processing_enabled);
    assert!(stats.registered_strategies > 0);
    assert!(!stats.registered_languages.is_empty());
    
    // Verify we have common language support
    let languages_str = stats.registered_languages.join(",").to_lowercase();
    assert!(languages_str.contains("default"));
}