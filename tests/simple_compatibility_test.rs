//! Simple backward compatibility verification test

use md2docx_converter::markdown::ast::{MarkdownDocument, MarkdownElement};
use md2docx_converter::markdown::parser::MarkdownParser;

#[test]
fn test_basic_code_block_compatibility() {
    // Test that we can still create code blocks the old way
    let code_block = MarkdownElement::CodeBlock {
        language: Some("rust".to_string()),
        code: "fn main() {}".to_string(),
        processed: None,
    };
    
    assert!(code_block.is_code_block());
    assert_eq!(code_block.get_code_block_language(), Some(&"rust".to_string()));
    assert!(!code_block.is_code_block_processed());
}

#[test]
fn test_parser_still_works() {
    let parser = MarkdownParser::new();
    let markdown = "```rust\nfn main() {}\n```";
    let result = parser.parse(markdown).unwrap();
    
    assert_eq!(result.elements.len(), 1);
    match &result.elements[0] {
        MarkdownElement::CodeBlock { language, code, processed } => {
            assert_eq!(language.as_ref().unwrap(), "rust");
            assert!(code.contains("fn main()"));
            // With new system, should be processed automatically
            assert!(processed.is_some());
        }
        _ => panic!("Expected code block"),
    }
}

#[test]
fn test_document_methods_work() {
    let mut doc = MarkdownDocument::new();
    doc.add_element(MarkdownElement::CodeBlock {
        language: Some("rust".to_string()),
        code: "test".to_string(),
        processed: None,
    });
    
    let code_blocks = doc.get_code_blocks();
    assert_eq!(code_blocks.len(), 1);
    
    let (processed, unprocessed) = doc.count_code_blocks_by_status();
    // Initially unprocessed since we created it manually
    assert_eq!(processed, 0);
    assert_eq!(unprocessed, 1);
}