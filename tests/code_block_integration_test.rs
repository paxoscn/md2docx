//! Integration tests for code block processing in the Markdown parser

use md2docx_converter::markdown::parser::MarkdownParser;
use md2docx_converter::markdown::ast::MarkdownElement;
use md2docx_converter::markdown::code_block::{CodeBlockConfig, LanguageConfig};

#[test]
fn test_code_block_processing_integration() {
    let parser = MarkdownParser::new();
    let markdown = r#"
# Test Document

Here's some Rust code:

```rust
fn main() {
    println!("Hello, world!");
}
```

And some JavaScript:

```javascript
console.log("Hello from JS!");
```

And some unknown language:

```unknown
some code here
```
"#;

    let result = parser.parse(markdown).unwrap();
    
    // Find all code blocks
    let code_blocks = result.get_code_blocks();
    assert_eq!(code_blocks.len(), 3);
    
    // Check that all code blocks have been processed
    for code_block in code_blocks {
        if let MarkdownElement::CodeBlock { language, code, processed } = code_block {
            println!("Processing code block with language: {:?}", language);
            
            // All code blocks should have been processed
            assert!(processed.is_some(), "Code block should be processed");
            
            let processed_block = processed.as_ref().unwrap();
            assert_eq!(processed_block.original_code, *code);
            assert_eq!(processed_block.language, *language);
            
            // Check that processing was successful (no errors)
            assert!(processed_block.is_successful(), "Processing should be successful");
            
            // Verify metadata
            assert!(processed_block.metadata.processing_time.as_nanos() >= 0);
            assert!(!processed_block.metadata.processor_version.is_empty());
        }
    }
}

#[test]
fn test_code_block_processing_with_custom_config() {
    // Create a custom configuration that disables processing
    let mut config = CodeBlockConfig::new();
    config.global.enable_processing = false;
    
    let parser = MarkdownParser::with_code_block_config(config);
    let markdown = "```rust\nfn main() {}\n```";
    
    let result = parser.parse(markdown).unwrap();
    let code_blocks = result.get_code_blocks();
    
    assert_eq!(code_blocks.len(), 1);
    
    if let MarkdownElement::CodeBlock { processed, .. } = &code_blocks[0] {
        // When processing is disabled, code blocks should still be processed
        // but with minimal processing (unprocessed state)
        assert!(processed.is_some());
        let processed_block = processed.as_ref().unwrap();
        assert!(processed_block.processed_code.is_none());
    }
}

#[test]
fn test_code_block_processing_with_language_specific_config() {
    // Create a configuration with language-specific settings
    let rust_config = LanguageConfig::new()
        .with_syntax_validation(true)
        .with_formatting(false);
    
    let config = CodeBlockConfig::new()
        .with_language_config("rust", rust_config);
    
    let parser = MarkdownParser::with_code_block_config(config);
    let markdown = "```rust\nfn main(){println!(\"test\");}\n```";
    
    let result = parser.parse(markdown).unwrap();
    let code_blocks = result.get_code_blocks();
    
    assert_eq!(code_blocks.len(), 1);
    
    if let MarkdownElement::CodeBlock { language, processed, .. } = &code_blocks[0] {
        assert_eq!(language.as_ref().unwrap(), "rust");
        assert!(processed.is_some());
        
        let processed_block = processed.as_ref().unwrap();
        assert_eq!(processed_block.language, Some("rust".to_string()));
        
        // The processing should have used the language-specific configuration
        assert!(processed_block.metadata.is_validated);
        // Since we disabled formatting, it shouldn't be formatted
        assert!(!processed_block.metadata.is_formatted);
    }
}

#[test]
fn test_parser_configuration_methods() {
    let mut parser = MarkdownParser::new();
    
    // Test getting the processor
    let processor = parser.get_code_block_processor();
    assert!(processor.get_config().is_processing_enabled());
    
    // Test updating configuration
    let mut new_config = CodeBlockConfig::new();
    new_config.global.enable_processing = false;
    
    parser.update_code_block_config(new_config);
    assert!(!parser.get_code_block_processor().get_config().is_processing_enabled());
}

#[test]
fn test_processing_stats() {
    let parser = MarkdownParser::new();
    let processor = parser.get_code_block_processor();
    let stats = processor.get_processing_stats();
    
    // Should have some basic configuration
    assert!(stats.processing_enabled);
    assert!(!stats.registered_aliases.is_empty()); // Should have common aliases
    
    println!("Processing stats: {}", stats.summary());
}

#[test]
fn test_error_handling_and_fallback() {
    let parser = MarkdownParser::new();
    
    // Test with potentially problematic code that might cause processing errors
    let markdown = r#"
```rust
// This might cause issues in a real processor
fn incomplete_function(
```
"#;
    
    let result = parser.parse(markdown);
    
    // Parsing should still succeed even if code processing has issues
    assert!(result.is_ok());
    
    let document = result.unwrap();
    let code_blocks = document.get_code_blocks();
    assert_eq!(code_blocks.len(), 1);
    
    // The code block should still be processed (with fallback)
    if let MarkdownElement::CodeBlock { processed, .. } = &code_blocks[0] {
        assert!(processed.is_some());
        // Even if there were processing errors, we should have a result
        let processed_block = processed.as_ref().unwrap();
        assert!(!processed_block.original_code.is_empty());
    }
}