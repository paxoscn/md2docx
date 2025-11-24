//! Test for Note Block DOCX generation

use md2docx_converter::config::ConversionConfig;
use md2docx_converter::docx::DocxGenerator;
use md2docx_converter::markdown::ast::{MarkdownDocument, MarkdownElement};
use md2docx_converter::markdown::code_block::{
    NoteStrategy, ProcessingConfig, CodeBlockStrategy
};

#[test]
fn test_note_block_docx_generation() {
    // Create a note strategy and process content
    let strategy = NoteStrategy::new();
    let config = ProcessingConfig::default().with_formatting(true);
    
    let note_content = "Important Notice\nThis is a critical piece of information.\nPlease read carefully.";
    let result = strategy.process(note_content, &config).unwrap();
    
    // Verify the processed code contains markers
    assert!(result.processed_code.is_some());
    let processed = result.processed_code.as_ref().unwrap();
    assert!(processed.contains("[NOTE_BLOCK_START]"));
    assert!(processed.contains("[TITLE]Important Notice[/TITLE]"));
    
    // Create a markdown document with the processed code block
    let mut document = MarkdownDocument::new();
    document.add_element(MarkdownElement::CodeBlock {
        language: Some("note".to_string()),
        code: note_content.to_string(),
        processed: Some(result),
    });
    
    // Generate DOCX
    let conversion_config = ConversionConfig::default();
    let mut generator = DocxGenerator::new(conversion_config);
    
    let docx_result = generator.generate(&document);
    assert!(docx_result.is_ok(), "DOCX generation should succeed");
}

#[test]
fn test_note_block_with_tip_alias() {
    let strategy = NoteStrategy::new();
    let config = ProcessingConfig::default().with_formatting(true);
    
    let tip_content = "Pro Tip\nAlways test your code before committing.";
    let result = strategy.process(tip_content, &config).unwrap();
    
    let mut document = MarkdownDocument::new();
    document.add_element(MarkdownElement::CodeBlock {
        language: Some("tip".to_string()),
        code: tip_content.to_string(),
        processed: Some(result),
    });
    
    let conversion_config = ConversionConfig::default();
    let mut generator = DocxGenerator::new(conversion_config);
    
    let docx_result = generator.generate(&document);
    assert!(docx_result.is_ok());
}

#[test]
fn test_note_block_single_line() {
    let strategy = NoteStrategy::new();
    let config = ProcessingConfig::default().with_formatting(true);
    
    let single_line = "Remember to save your work!";
    let result = strategy.process(single_line, &config).unwrap();
    
    let mut document = MarkdownDocument::new();
    document.add_element(MarkdownElement::CodeBlock {
        language: Some("hint".to_string()),
        code: single_line.to_string(),
        processed: Some(result),
    });
    
    let conversion_config = ConversionConfig::default();
    let mut generator = DocxGenerator::new(conversion_config);
    
    let docx_result = generator.generate(&document);
    assert!(docx_result.is_ok());
}

#[test]
fn test_note_block_multiline_content() {
    let strategy = NoteStrategy::new();
    let config = ProcessingConfig::default().with_formatting(true);
    
    let multiline = "Security Warning\nNever hardcode sensitive information:\n- API keys\n- Passwords\n- Private keys";
    let result = strategy.process(multiline, &config).unwrap();
    
    let mut document = MarkdownDocument::new();
    document.add_element(MarkdownElement::CodeBlock {
        language: Some("note".to_string()),
        code: multiline.to_string(),
        processed: Some(result),
    });
    
    let conversion_config = ConversionConfig::default();
    let mut generator = DocxGenerator::new(conversion_config);
    
    let docx_result = generator.generate(&document);
    assert!(docx_result.is_ok());
}

#[test]
fn test_mixed_code_blocks() {
    // Test that note blocks and regular code blocks can coexist
    let strategy = NoteStrategy::new();
    let config = ProcessingConfig::default().with_formatting(true);
    
    let note_content = "Important\nThis is a note.";
    let note_result = strategy.process(note_content, &config).unwrap();
    
    let mut document = MarkdownDocument::new();
    
    // Add a regular code block
    document.add_element(MarkdownElement::CodeBlock {
        language: Some("rust".to_string()),
        code: "fn main() {\n    println!(\"Hello\");\n}".to_string(),
        processed: None,
    });
    
    // Add a note block
    document.add_element(MarkdownElement::CodeBlock {
        language: Some("note".to_string()),
        code: note_content.to_string(),
        processed: Some(note_result),
    });
    
    // Add another regular code block
    document.add_element(MarkdownElement::CodeBlock {
        language: Some("python".to_string()),
        code: "print('Hello, World!')".to_string(),
        processed: None,
    });
    
    let conversion_config = ConversionConfig::default();
    let mut generator = DocxGenerator::new(conversion_config);
    
    let docx_result = generator.generate(&document);
    assert!(docx_result.is_ok());
}
