//! Backward compatibility tests for the code block strategy system
//!
//! These tests ensure that existing code continues to work after implementing
//! the strategy pattern for code block processing.

use md2docx_converter::markdown::ast::{MarkdownDocument, MarkdownElement, InlineElement};
use md2docx_converter::markdown::parser::MarkdownParser;
use md2docx_converter::markdown::code_block::{CodeBlockProcessor, CodeBlockConfig, ProcessedCodeBlock};
use md2docx_converter::config::models::ConversionConfig;
use md2docx_converter::conversion::engine::ConversionEngine;

/// Test that existing code block creation still works
#[test]
fn test_code_block_creation_backward_compatibility() {
    // Test creating code blocks the old way
    let code_block = MarkdownElement::CodeBlock {
        language: Some("rust".to_string()),
        code: "fn main() {\n    println!(\"Hello, world!\");\n}".to_string(),
        processed: None,
    };
    
    // Verify the structure is as expected
    assert!(code_block.is_code_block());
    assert_eq!(code_block.get_code_block_language(), Some(&"rust".to_string()));
    assert_eq!(code_block.get_code_block_code(), Some(&"fn main() {\n    println!(\"Hello, world!\");\n}".to_string()));
    assert!(!code_block.is_code_block_processed());
    assert!(code_block.get_code_block_processed().is_none());
}

/// Test that existing AST methods still work
#[test]
fn test_ast_methods_backward_compatibility() {
    let mut doc = MarkdownDocument::new();
    
    // Add elements the old way
    doc.add_element(MarkdownElement::Heading {
        level: 1,
        text: "Test Heading".to_string(),
    });
    
    doc.add_element(MarkdownElement::CodeBlock {
        language: Some("javascript".to_string()),
        code: "console.log('hello');".to_string(),
        processed: None,
    });
    
    doc.add_element(MarkdownElement::Paragraph {
        content: vec![InlineElement::Text("Test paragraph".to_string())],
    });
    
    // Test existing methods still work
    assert_eq!(doc.elements.len(), 3);
    assert_eq!(doc.element_count(), 3);
    
    let headings = doc.get_headings();
    assert_eq!(headings.len(), 1);
    
    let code_blocks = doc.get_code_blocks();
    assert_eq!(code_blocks.len(), 1);
    
    // Test new code block methods work with old structure
    let unprocessed = doc.get_unprocessed_code_blocks();
    assert_eq!(unprocessed.len(), 1);
    
    let processed = doc.get_processed_code_blocks();
    assert_eq!(processed.len(), 0);
    
    let (processed_count, unprocessed_count) = doc.count_code_blocks_by_status();
    assert_eq!(processed_count, 0);
    assert_eq!(unprocessed_count, 1);
}

/// Test that existing parser behavior is preserved
#[test]
fn test_parser_backward_compatibility() {
    let parser = MarkdownParser::new();
    let markdown = r#"
# Test Document

This is a paragraph.

```rust
fn main() {
    println!("Hello, world!");
}
```

Another paragraph.
"#;
    
    let result = parser.parse(markdown).unwrap();
    
    // Verify structure is as expected
    assert_eq!(result.elements.len(), 4);
    
    // Check heading
    match &result.elements[0] {
        MarkdownElement::Heading { level, text } => {
            assert_eq!(*level, 1);
            assert_eq!(text, "Test Document");
        }
        _ => panic!("Expected heading"),
    }
    
    // Check first paragraph
    match &result.elements[1] {
        MarkdownElement::Paragraph { content } => {
            assert_eq!(content.len(), 1);
            match &content[0] {
                InlineElement::Text(text) => assert_eq!(text, "This is a paragraph."),
                _ => panic!("Expected text"),
            }
        }
        _ => panic!("Expected paragraph"),
    }
    
    // Check code block - this should now be processed automatically
    match &result.elements[2] {
        MarkdownElement::CodeBlock { language, code, processed } => {
            assert_eq!(language.as_ref().unwrap(), "rust");
            assert!(code.contains("fn main()"));
            // With the new system, code blocks should be processed automatically
            assert!(processed.is_some());
            let processed_block = processed.as_ref().unwrap();
            assert_eq!(processed_block.original_code, *code);
            assert_eq!(processed_block.language, *language);
        }
        _ => panic!("Expected code block"),
    }
    
    // Check last paragraph
    match &result.elements[3] {
        MarkdownElement::Paragraph { content } => {
            assert_eq!(content.len(), 1);
            match &content[0] {
                InlineElement::Text(text) => assert_eq!(text, "Another paragraph."),
                _ => panic!("Expected text"),
            }
        }
        _ => panic!("Expected paragraph"),
    }
}

/// Test that code blocks without language still work
#[test]
fn test_code_block_without_language_compatibility() {
    let parser = MarkdownParser::new();
    let markdown = r#"
```
plain code block
without language
```
"#;
    
    let result = parser.parse(markdown).unwrap();
    assert_eq!(result.elements.len(), 1);
    
    match &result.elements[0] {
        MarkdownElement::CodeBlock { language, code, processed } => {
            assert!(language.is_none());
            assert!(code.contains("plain code block"));
            // Should still be processed (with default strategy)
            assert!(processed.is_some());
            let processed_block = processed.as_ref().unwrap();
            assert_eq!(processed_block.original_code, *code);
            assert_eq!(processed_block.language, None);
        }
        _ => panic!("Expected code block"),
    }
}

/// Test that existing text extraction methods work
#[test]
fn test_text_extraction_backward_compatibility() {
    let mut doc = MarkdownDocument::new();
    
    doc.add_element(MarkdownElement::Heading {
        level: 1,
        text: "Title".to_string(),
    });
    
    doc.add_element(MarkdownElement::CodeBlock {
        language: Some("rust".to_string()),
        code: "fn test() {}".to_string(),
        processed: None,
    });
    
    // Test that text extraction still works
    let text = doc.extract_text();
    assert!(text.contains("Title"));
    assert!(text.contains("fn test() {}"));
    
    // Test individual element text extraction
    let code_text = doc.elements[1].extract_text();
    assert_eq!(code_text, "fn test() {}");
}

/// Test that processed code blocks work with text extraction
#[test]
fn test_processed_code_block_text_extraction() {
    let original_code = "fn main(){println!(\"hello\");}";
    let formatted_code = "fn main() {\n    println!(\"hello\");\n}";
    
    let processed = ProcessedCodeBlock::new(
        original_code.to_string(),
        Some("rust".to_string())
    )
    .with_processed_code(formatted_code.to_string());
    
    let mut code_block = MarkdownElement::CodeBlock {
        language: Some("rust".to_string()),
        code: original_code.to_string(),
        processed: None,
    };
    
    // Set processed result
    code_block.set_code_block_processed(processed).unwrap();
    
    // Text extraction should return processed code
    let extracted = code_block.extract_text();
    assert_eq!(extracted, formatted_code);
    
    // Final code should also return processed version
    assert_eq!(code_block.get_code_block_final_code(), Some(formatted_code));
}

/// Test that document traversal still works with processed code blocks
#[test]
fn test_document_traversal_with_processed_blocks() {
    let parser = MarkdownParser::new();
    let markdown = r#"
# Document

```rust
fn main() {}
```

```javascript
console.log("test");
```
"#;
    
    let result = parser.parse(markdown).unwrap();
    
    // Test traversal
    let mut element_count = 0;
    result.traverse(|_| element_count += 1);
    assert_eq!(element_count, 3); // heading + 2 code blocks
    
    // Test code block specific methods
    let code_blocks = result.get_code_blocks();
    assert_eq!(code_blocks.len(), 2);
    
    let rust_blocks = result.get_code_blocks_by_language("rust");
    assert_eq!(rust_blocks.len(), 1);
    
    let js_blocks = result.get_code_blocks_by_language("javascript");
    assert_eq!(js_blocks.len(), 1);
    
    // All should be processed now
    let (processed_count, unprocessed_count) = result.count_code_blocks_by_status();
    assert_eq!(processed_count, 2);
    assert_eq!(unprocessed_count, 0);
}

/// Test that the old CodeBlockProcessor interface still works
#[test]
fn test_code_block_processor_compatibility() {
    let processor = CodeBlockProcessor::new();
    
    // Test processing a code block
    let result = processor.process_code_block(
        "fn main() {\n    println!(\"Hello\");\n}",
        Some("rust")
    );
    
    assert!(result.is_ok());
    let processed = result.unwrap();
    assert_eq!(processed.original_code, "fn main() {\n    println!(\"Hello\");\n}");
    assert_eq!(processed.language, Some("rust".to_string()));
}

/// Test that configuration still works
#[test]
fn test_configuration_backward_compatibility() {
    let config = CodeBlockConfig::new();
    let processor = CodeBlockProcessor::with_config(config);
    
    // Should work the same way
    let result = processor.process_code_block("test code", None);
    assert!(result.is_ok());
}

/// Test that conversion engine still works with new code block system
#[test]
fn test_conversion_engine_compatibility() {
    let config = ConversionConfig::default();
    let mut engine = ConversionEngine::new(config);
    
    let markdown = r#"
# Test

```rust
fn main() {
    println!("Hello");
}
```
"#;
    
    // This should work without any changes to existing code
    let result = futures::executor::block_on(engine.convert(markdown));
    assert!(result.is_ok());
    
    let docx_bytes = result.unwrap();
    assert!(!docx_bytes.is_empty());
}

/// Test that existing helper methods on MarkdownElement still work
#[test]
fn test_element_helper_methods_compatibility() {
    let code_block = MarkdownElement::CodeBlock {
        language: Some("python".to_string()),
        code: "print('hello')".to_string(),
        processed: None,
    };
    
    // All existing methods should still work
    assert_eq!(code_block.element_type(), "code_block");
    assert!(code_block.has_text_content());
    assert!(code_block.is_code_block());
    assert_eq!(code_block.get_code_block_language(), Some(&"python".to_string()));
    assert_eq!(code_block.get_code_block_code(), Some(&"print('hello')".to_string()));
    assert!(!code_block.is_code_block_processed());
    
    // Non-code-block elements should still work
    let heading = MarkdownElement::Heading {
        level: 2,
        text: "Test".to_string(),
    };
    
    assert!(!heading.is_code_block());
    assert!(heading.get_code_block_language().is_none());
    assert!(heading.get_code_block_code().is_none());
    assert!(!heading.is_code_block_processed());
}

/// Test that mutable operations on documents still work
#[test]
fn test_mutable_operations_compatibility() {
    let mut doc = MarkdownDocument::new();
    
    doc.add_element(MarkdownElement::CodeBlock {
        language: Some("rust".to_string()),
        code: "fn test() {}".to_string(),
        processed: None,
    });
    
    // Test mutable access
    let mut code_blocks = doc.get_code_blocks_mut();
    assert_eq!(code_blocks.len(), 1);
    
    // Test setting processed result
    let processed = ProcessedCodeBlock::new(
        "fn test() {}".to_string(),
        Some("rust".to_string())
    );
    
    let result = code_blocks[0].set_code_block_processed(processed);
    assert!(result.is_ok());
    
    // Verify the change
    assert!(doc.get_code_blocks()[0].is_code_block_processed());
}

/// Test that serialization/deserialization patterns still work
#[test]
fn test_serialization_compatibility() {
    // Test that we can still create and manipulate documents
    // in the same way as before
    let mut doc = MarkdownDocument::new();
    
    doc.add_element(MarkdownElement::Heading {
        level: 1,
        text: "Test".to_string(),
    });
    
    doc.add_element(MarkdownElement::CodeBlock {
        language: Some("rust".to_string()),
        code: "fn main() {}".to_string(),
        processed: None,
    });
    
    // The document should behave exactly as before
    assert_eq!(doc.elements.len(), 2);
    
    // Test that the structure is still accessible in the same way
    match &doc.elements[0] {
        MarkdownElement::Heading { level, text } => {
            assert_eq!(*level, 1);
            assert_eq!(text, "Test");
        }
        _ => panic!("Expected heading"),
    }
    
    match &doc.elements[1] {
        MarkdownElement::CodeBlock { language, code, processed } => {
            assert_eq!(language.as_ref().unwrap(), "rust");
            assert_eq!(code, "fn main() {}");
            assert!(processed.is_none());
        }
        _ => panic!("Expected code block"),
    }
}

/// Test edge cases that should remain compatible
#[test]
fn test_edge_cases_compatibility() {
    // Empty code block
    let empty_code = MarkdownElement::CodeBlock {
        language: None,
        code: String::new(),
        processed: None,
    };
    
    assert!(empty_code.is_code_block());
    assert_eq!(empty_code.get_code_block_code(), Some(&String::new()));
    assert_eq!(empty_code.extract_text(), "");
    
    // Code block with whitespace only
    let whitespace_code = MarkdownElement::CodeBlock {
        language: Some("text".to_string()),
        code: "   \n\t  \n   ".to_string(),
        processed: None,
    };
    
    assert!(whitespace_code.is_code_block());
    assert_eq!(whitespace_code.extract_text(), "   \n\t  \n   ");
    
    // Very long language name
    let long_lang = "a".repeat(1000);
    let long_lang_code = MarkdownElement::CodeBlock {
        language: Some(long_lang.clone()),
        code: "test".to_string(),
        processed: None,
    };
    
    assert_eq!(long_lang_code.get_code_block_language(), Some(&long_lang));
}

/// Test that error handling patterns remain the same
#[test]
fn test_error_handling_compatibility() {
    let mut heading = MarkdownElement::Heading {
        level: 1,
        text: "Test".to_string(),
    };
    
    // Trying to set processed result on non-code-block should still fail
    let processed = ProcessedCodeBlock::new("test".to_string(), None);
    let result = heading.set_code_block_processed(processed);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "Element is not a code block");
}