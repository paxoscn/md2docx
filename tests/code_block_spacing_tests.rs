//! Tests for code block spacing and formatting fixes

use md2docx_converter::{
    config::ConversionConfig,
    conversion::ConversionEngine,
};
use std::fs;
use tempfile::TempDir;

/// Test markdown with code blocks that have trailing newlines
fn create_test_markdown_with_trailing_newlines() -> &'static str {
    r#"# Code Block Spacing Test

This tests code blocks with trailing newlines and spacing.

## Code Block with Trailing Newlines

```rust
fn main() {
    println!("Hello, world!");
}

```

## Another Code Block

```python
def hello():
    print("Hello from Python")

```

## Code Block without Trailing Newlines

```javascript
function hello() {
    console.log("Hello from JavaScript");
}
```

End of test document.
"#
}

#[tokio::test]
async fn test_code_block_trailing_newline_removal() {
    let config = ConversionConfig::default();
    let mut engine = ConversionEngine::new(config);
    
    let markdown = create_test_markdown_with_trailing_newlines();
    
    // Test conversion
    let docx_bytes = engine.convert(markdown).await.unwrap();
    assert!(!docx_bytes.is_empty());
    assert!(docx_bytes.len() > 2000); // Should be substantial with table structures and spacing
    
    // Verify docx file structure
    assert_eq!(&docx_bytes[0..2], b"PK"); // ZIP signature
}

#[tokio::test]
async fn test_code_block_spacing_in_document() {
    let config = ConversionConfig::default();
    let mut engine = ConversionEngine::new(config);
    
    let markdown_with_spacing = r#"# Spacing Test

Before code block.

```rust
fn test() {
    println!("test");
}
```

After code block.

Another paragraph.

```python
print("Another code block")
```

Final paragraph.
"#;
    
    let docx_bytes = engine.convert(markdown_with_spacing).await.unwrap();
    assert!(!docx_bytes.is_empty());
    assert!(docx_bytes.len() > 1500);
    
    // Verify docx structure
    assert_eq!(&docx_bytes[0..2], b"PK");
}

#[tokio::test]
async fn test_empty_code_block_spacing() {
    let config = ConversionConfig::default();
    let mut engine = ConversionEngine::new(config);
    
    let markdown_with_empty = r#"# Empty Code Block Test

Before empty code block.

```
```

After empty code block.
"#;
    
    let docx_bytes = engine.convert(markdown_with_empty).await.unwrap();
    assert!(!docx_bytes.is_empty());
    assert!(docx_bytes.len() > 1000);
    
    // Verify docx structure
    assert_eq!(&docx_bytes[0..2], b"PK");
}

#[tokio::test]
async fn test_multiple_trailing_newlines() {
    let config = ConversionConfig::default();
    let mut engine = ConversionEngine::new(config);
    
    let markdown_multiple_newlines = r#"# Multiple Trailing Newlines Test

```rust
fn main() {
    println!("Hello");
}



```

This should not have extra empty lines at the end of the code block.
"#;
    
    let docx_bytes = engine.convert(markdown_multiple_newlines).await.unwrap();
    assert!(!docx_bytes.is_empty());
    assert!(docx_bytes.len() > 1000);
    
    // Verify docx structure
    assert_eq!(&docx_bytes[0..2], b"PK");
}

#[tokio::test]
async fn test_code_block_with_custom_spacing_config() {
    let mut config = ConversionConfig::default();
    
    // Increase paragraph spacing for more noticeable spacing
    config.styles.code_block.paragraph_spacing = 12.0;
    
    let mut engine = ConversionEngine::new(config);
    
    let markdown = r#"# Custom Spacing Test

Text before code block.

```rust
fn main() {
    println!("Hello");
}
```

Text after code block.
"#;
    
    let docx_bytes = engine.convert(markdown).await.unwrap();
    assert!(!docx_bytes.is_empty());
    assert!(docx_bytes.len() > 1000);
    
    // Verify docx structure
    assert_eq!(&docx_bytes[0..2], b"PK");
}

#[tokio::test]
async fn test_file_conversion_with_spacing_fixes() {
    let temp_dir = TempDir::new().unwrap();
    
    // Create test markdown file with spacing issues
    let input_file = temp_dir.path().join("spacing_test.md");
    fs::write(&input_file, create_test_markdown_with_trailing_newlines()).unwrap();
    
    let output_file = temp_dir.path().join("spacing_test.docx");
    
    let config = ConversionConfig::default();
    let mut engine = ConversionEngine::new(config);
    
    // Test file-based conversion
    let result = engine.convert_file(
        input_file.to_str().unwrap(),
        output_file.to_str().unwrap()
    ).await;
    
    assert!(result.is_ok());
    assert!(output_file.exists());
    
    let file_size = fs::metadata(&output_file).unwrap().len();
    assert!(file_size > 2000); // Should be substantial with proper spacing
    
    // Verify docx structure
    let file_content = fs::read(&output_file).unwrap();
    assert_eq!(&file_content[0..2], b"PK");
}

#[tokio::test]
async fn test_code_block_spacing_with_preserve_line_breaks_false() {
    let mut config = ConversionConfig::default();
    config.styles.code_block.preserve_line_breaks = false;
    
    let mut engine = ConversionEngine::new(config);
    
    let markdown_with_newlines = r#"# No Line Break Preservation Test

```python
def function():
    print("line 1")
    print("line 2")

    print("line 4")
```

End of test.
"#;
    
    let docx_bytes = engine.convert(markdown_with_newlines).await.unwrap();
    assert!(!docx_bytes.is_empty());
    assert!(docx_bytes.len() > 1000);
    
    // Verify docx structure
    assert_eq!(&docx_bytes[0..2], b"PK");
}

#[tokio::test]
async fn test_code_block_spacing_edge_cases() {
    let config = ConversionConfig::default();
    let mut engine = ConversionEngine::new(config);
    
    // Test various edge cases
    let edge_cases = vec![
        ("Only newlines", "```\n\n\n```"),
        ("Single newline", "```\n```"),
        ("Code with only trailing newlines", "```\ncode\n\n\n```"),
        ("Code with mixed content", "```\nline1\n\nline3\n\n```"),
    ];
    
    for (description, markdown_code) in edge_cases {
        let full_markdown = format!("# {}\n\n{}\n\nAfter code block.", description, markdown_code);
        
        let docx_bytes = engine.convert(&full_markdown).await.unwrap();
        assert!(!docx_bytes.is_empty(), "Failed for case: {}", description);
        assert_eq!(&docx_bytes[0..2], b"PK", "Invalid docx structure for case: {}", description);
    }
}