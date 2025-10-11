//! Final integration verification for the code block strategy system

use md2docx_converter::markdown::{MarkdownParser, MarkdownElement};
use md2docx_converter::markdown::code_block::CodeBlockProcessor;

#[test]
fn test_end_to_end_integration() {
    // Test that the complete system works end-to-end
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

And code without a language:

```
This is generic code
```
"#;

    // Parse with default configuration
    let parser = MarkdownParser::new();
    let document = parser.parse(markdown).expect("Failed to parse markdown");
    
    // Verify we have the expected elements
    let mut code_blocks = 0;
    let mut processed_blocks = 0;
    
    for element in &document.elements {
        if let MarkdownElement::CodeBlock { language: _, code: _, processed } = element {
            code_blocks += 1;
            if processed.is_some() {
                processed_blocks += 1;
            }
        }
    }
    
    // We should have found 3 code blocks
    assert_eq!(code_blocks, 3);
    
    // All should be processed (even if just with default strategy)
    assert_eq!(processed_blocks, 3);
    
    println!("✅ End-to-end integration test passed!");
    println!("   - Found {} code blocks", code_blocks);
    println!("   - Processed {} code blocks", processed_blocks);
}

#[test]
fn test_code_block_processor_direct() {
    // Test the processor directly
    let processor = CodeBlockProcessor::new();
    
    // Test Rust code
    let rust_code = "fn main() { println!(\"Hello\"); }";
    let result = processor.process_code_block(rust_code, Some("rust"))
        .expect("Failed to process Rust code");
    
    assert_eq!(result.original_code, rust_code);
    assert_eq!(result.language, Some("rust".to_string()));
    assert!(result.is_successful());
    
    // Test unknown language
    let unknown_code = "some unknown code";
    let result = processor.process_code_block(unknown_code, Some("unknown"))
        .expect("Failed to process unknown code");
    
    assert_eq!(result.original_code, unknown_code);
    assert_eq!(result.language, Some("unknown".to_string()));
    assert!(result.is_successful()); // Should still succeed with default strategy
    
    println!("✅ Direct processor test passed!");
}

#[test]
fn test_system_components() {
    // Test that all major components are working
    let processor = CodeBlockProcessor::new();
    let stats = processor.get_processing_stats();
    
    // Verify basic functionality
    assert!(stats.processing_enabled);
    assert!(stats.registered_strategies >= 0);
    
    // Test registry
    let registry = processor.get_registry();
    let default_strategy = registry.get_default_strategy();
    assert_eq!(default_strategy.get_language_name(), "default");
    
    println!("✅ System components test passed!");
    println!("   - Processing enabled: {}", stats.processing_enabled);
    println!("   - Registered strategies: {}", stats.registered_strategies);
    println!("   - Default strategy: {}", default_strategy.get_language_name());
}

#[test]
fn test_backward_compatibility() {
    // Ensure the system works with existing code
    let parser = MarkdownParser::new();
    
    let simple_markdown = "```rust\nfn test() {}\n```";
    let document = parser.parse(simple_markdown).expect("Failed to parse");
    
    // Should have one code block
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
    assert_eq!(language.as_ref().unwrap(), "rust");
    assert!(code.contains("fn test()"));
    assert!(processed.is_some()); // Should be processed
    
    println!("✅ Backward compatibility test passed!");
}