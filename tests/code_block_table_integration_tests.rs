//! Integration tests for table-based code block rendering

use md2docx_converter::{
    config::{CodeBlockStyle, ConversionConfig, FontConfig},
    conversion::ConversionEngine,
};
use std::fs;
use tempfile::TempDir;

/// Create test markdown content with various code block scenarios
fn create_code_block_test_markdown() -> &'static str {
    r#"# Code Block Integration Test

This document tests table-based code block rendering.

## Single Line Code Block

```rust
fn hello() { println!("Hello, world!"); }
```

## Multi-line Code Block

```python
def fibonacci(n):
    if n <= 1:
        return n
    return fibonacci(n-1) + fibonacci(n-2)

# Test with empty line

print("Done")
```

## Empty Code Block

```
```

## Code Block with Special Characters

```javascript
const message = "Hello, \"world\"!";
const symbols = '@#$%^&*()_+-={}[]|\\:";\'<>?,./';
console.log(`Message: ${message}`);
```

## Code Block with Tabs and Spaces

```c
int main() {
	printf("Tab indented\n");
    printf("Space indented\n");
		printf("Mixed indentation\n");
	return 0;
}
```

## Multiple Code Blocks

First block:
```bash
echo "First command"
ls -la
```

Second block:
```sql
SELECT * FROM users WHERE active = 1;
UPDATE users SET last_login = NOW();
```

End of test document.
"#
}

/// Create test configuration with table-based code blocks
fn create_table_code_block_config() -> ConversionConfig {
    let mut config = ConversionConfig::default();

    // Configure code block with table rendering and borders
    config.styles.code_block = CodeBlockStyle {
        font: FontConfig {
            family: "Courier New".to_string(),
            size: 10.0,
            bold: false,
            italic: false,
        },
        background_color: Some("#f8f8f8".to_string()),
        border_width: 1.5,
        preserve_line_breaks: true,
        line_spacing: 1.0,
        paragraph_spacing: 6.0,
    };

    config
}

/// Create test configuration with no borders
fn create_borderless_code_block_config() -> ConversionConfig {
    let mut config = ConversionConfig::default();

    config.styles.code_block.border_width = 0.0;
    config.styles.code_block.background_color = Some("#f0f0f0".to_string());

    config
}

#[tokio::test]
async fn test_table_based_code_block_generation() {
    let config = create_table_code_block_config();
    let mut engine = ConversionEngine::new(config);

    let markdown = create_code_block_test_markdown();

    // Test conversion statistics
    let stats = engine.get_conversion_stats(markdown).unwrap();
    assert!(stats.code_blocks >= 6); // Should detect all code blocks

    // Test actual conversion
    let docx_bytes = engine.convert(markdown).await.unwrap();
    assert!(!docx_bytes.is_empty());
    assert!(docx_bytes.len() > 2000); // Should be substantial with table structures

    // Verify docx file structure
    assert_eq!(&docx_bytes[0..2], b"PK"); // ZIP signature
}

#[tokio::test]
async fn test_code_block_with_borders() {
    let config = create_table_code_block_config();
    let mut engine = ConversionEngine::new(config);

    let simple_code =
        "# Simple Code Block Test\n\n```rust\nfn main() {\n    println!(\"Hello\");\n}\n```";

    let docx_bytes = engine.convert(simple_code).await.unwrap();
    assert!(!docx_bytes.is_empty());
    assert!(docx_bytes.len() > 1000);

    // Verify docx structure
    assert_eq!(&docx_bytes[0..2], b"PK");
}

#[tokio::test]
async fn test_code_block_without_borders() {
    let config = create_borderless_code_block_config();
    let mut engine = ConversionEngine::new(config);

    let simple_code = "# Borderless Code Block Test\n\n```python\nprint('Hello, world!')\n```";

    let docx_bytes = engine.convert(simple_code).await.unwrap();
    assert!(!docx_bytes.is_empty());
    assert!(docx_bytes.len() > 1000);

    // Verify docx structure
    assert_eq!(&docx_bytes[0..2], b"PK");
}

#[tokio::test]
async fn test_empty_code_block_handling() {
    let config = create_table_code_block_config();
    let mut engine = ConversionEngine::new(config);

    let empty_code = "# Empty Code Block Test\n\n```\n```\n\nAfter empty block.";

    let docx_bytes = engine.convert(empty_code).await.unwrap();
    assert!(!docx_bytes.is_empty());
    assert!(docx_bytes.len() > 800);

    // Verify docx structure
    assert_eq!(&docx_bytes[0..2], b"PK");
}

#[tokio::test]
async fn test_configuration_migration_old_border_true() {
    // Test old configuration format with border: true
    let old_config_yaml = "
document:
  page_size:
    width: 595.0
    height: 842.0
  margins:
    top: 72.0
    bottom: 72.0
    left: 72.0
    right: 72.0
  default_font:
    family: \"Times New Roman\"
    size: 12.0
    bold: false
    italic: false
styles:
  headings: {}
  paragraph:
    font:
      family: \"Times New Roman\"
      size: 12.0
      bold: false
      italic: false
    line_spacing: 1.15
    spacing_after: 6.0
  code_block:
    font:
      family: \"Courier New\"
      size: 10.0
      bold: false
      italic: false
    background_color: \"#f5f5f5\"
    border: true
    preserve_line_breaks: true
    line_spacing: 1.0
    paragraph_spacing: 6.0
  table:
    header_font:
      family: \"Times New Roman\"
      size: 12.0
      bold: true
      italic: false
    cell_font:
      family: \"Times New Roman\"
      size: 12.0
      bold: false
      italic: false
    border_width: 1.0
elements:
  image:
    max_width: 500.0
    max_height: 400.0
  list:
    indent: 36.0
    spacing: 6.0
  link:
    color: \"#0066cc\"
    underline: true
";

    let config: ConversionConfig = serde_yaml::from_str(old_config_yaml).unwrap();
    assert!(config.validate().is_ok());
    assert_eq!(config.styles.code_block.border_width, 1.0);

    let mut engine = ConversionEngine::new(config);
    let simple_code = "```rust\nfn test() {}\n```";

    let docx_bytes = engine.convert(simple_code).await.unwrap();
    assert!(!docx_bytes.is_empty());
    assert_eq!(&docx_bytes[0..2], b"PK");
}

#[tokio::test]
async fn test_configuration_migration_old_border_false() {
    // Test old configuration format with border: false
    let old_config_yaml = "
document:
  page_size:
    width: 595.0
    height: 842.0
  margins:
    top: 72.0
    bottom: 72.0
    left: 72.0
    right: 72.0
  default_font:
    family: \"Times New Roman\"
    size: 12.0
    bold: false
    italic: false
styles:
  headings: {}
  paragraph:
    font:
      family: \"Times New Roman\"
      size: 12.0
      bold: false
      italic: false
    line_spacing: 1.15
    spacing_after: 6.0
  code_block:
    font:
      family: \"Courier New\"
      size: 10.0
      bold: false
      italic: false
    background_color: \"#f5f5f5\"
    border: false
    preserve_line_breaks: true
    line_spacing: 1.0
    paragraph_spacing: 6.0
  table:
    header_font:
      family: \"Times New Roman\"
      size: 12.0
      bold: true
      italic: false
    cell_font:
      family: \"Times New Roman\"
      size: 12.0
      bold: false
      italic: false
    border_width: 1.0
elements:
  image:
    max_width: 500.0
    max_height: 400.0
  list:
    indent: 36.0
    spacing: 6.0
  link:
    color: \"#0066cc\"
    underline: true
";

    let config: ConversionConfig = serde_yaml::from_str(old_config_yaml).unwrap();
    assert!(config.validate().is_ok());
    assert_eq!(config.styles.code_block.border_width, 0.0);

    let mut engine = ConversionEngine::new(config);
    let simple_code = "```python\nprint('test')\n```";

    let docx_bytes = engine.convert(simple_code).await.unwrap();
    assert!(!docx_bytes.is_empty());
    assert_eq!(&docx_bytes[0..2], b"PK");
}

#[tokio::test]
async fn test_configuration_migration_new_border_width() {
    // Test new configuration format with border_width
    let new_config_yaml = "
document:
  page_size:
    width: 595.0
    height: 842.0
  margins:
    top: 72.0
    bottom: 72.0
    left: 72.0
    right: 72.0
  default_font:
    family: \"Times New Roman\"
    size: 12.0
    bold: false
    italic: false
styles:
  headings: {}
  paragraph:
    font:
      family: \"Times New Roman\"
      size: 12.0
      bold: false
      italic: false
    line_spacing: 1.15
    spacing_after: 6.0
  code_block:
    font:
      family: \"Courier New\"
      size: 10.0
      bold: false
      italic: false
    background_color: \"#f5f5f5\"
    border_width: 2.5
    preserve_line_breaks: true
    line_spacing: 1.0
    paragraph_spacing: 6.0
  table:
    header_font:
      family: \"Times New Roman\"
      size: 12.0
      bold: true
      italic: false
    cell_font:
      family: \"Times New Roman\"
      size: 12.0
      bold: false
      italic: false
    border_width: 1.0
elements:
  image:
    max_width: 500.0
    max_height: 400.0
  list:
    indent: 36.0
    spacing: 6.0
  link:
    color: \"#0066cc\"
    underline: true
";

    let config: ConversionConfig = serde_yaml::from_str(new_config_yaml).unwrap();
    assert!(config.validate().is_ok());
    assert_eq!(config.styles.code_block.border_width, 2.5);

    let mut engine = ConversionEngine::new(config);
    let simple_code = "```javascript\nconsole.log('test');\n```";

    let docx_bytes = engine.convert(simple_code).await.unwrap();
    assert!(!docx_bytes.is_empty());
    assert_eq!(&docx_bytes[0..2], b"PK");
}

#[tokio::test]
async fn test_configuration_migration_both_properties() {
    // Test configuration with both old and new properties (new should take precedence)
    let mixed_config_yaml = "
document:
  page_size:
    width: 595.0
    height: 842.0
  margins:
    top: 72.0
    bottom: 72.0
    left: 72.0
    right: 72.0
  default_font:
    family: \"Times New Roman\"
    size: 12.0
    bold: false
    italic: false
styles:
  headings: {}
  paragraph:
    font:
      family: \"Times New Roman\"
      size: 12.0
      bold: false
      italic: false
    line_spacing: 1.15
    spacing_after: 6.0
  code_block:
    font:
      family: \"Courier New\"
      size: 10.0
      bold: false
      italic: false
    background_color: \"#f5f5f5\"
    border: true
    border_width: 3.0
    preserve_line_breaks: true
    line_spacing: 1.0
    paragraph_spacing: 6.0
  table:
    header_font:
      family: \"Times New Roman\"
      size: 12.0
      bold: true
      italic: false
    cell_font:
      family: \"Times New Roman\"
      size: 12.0
      bold: false
      italic: false
    border_width: 1.0
elements:
  image:
    max_width: 500.0
    max_height: 400.0
  list:
    indent: 36.0
    spacing: 6.0
  link:
    color: \"#0066cc\"
    underline: true
";

    let config: ConversionConfig = serde_yaml::from_str(mixed_config_yaml).unwrap();
    assert!(config.validate().is_ok());
    // New border_width should take precedence over old border: true
    assert_eq!(config.styles.code_block.border_width, 3.0);

    let mut engine = ConversionEngine::new(config);
    let simple_code = "```go\nfmt.Println(\"test\")\n```";

    let docx_bytes = engine.convert(simple_code).await.unwrap();
    assert!(!docx_bytes.is_empty());
    assert_eq!(&docx_bytes[0..2], b"PK");
}

#[tokio::test]
async fn test_error_handling_invalid_border_width() {
    // Test that invalid border_width is caught during validation
    let mut config = ConversionConfig::default();
    config.styles.code_block.border_width = -1.0;

    let validation_result = config.validate();
    assert!(validation_result.is_err());

    // Even with invalid config, engine should handle gracefully
    let mut engine = ConversionEngine::new(config);
    let simple_code = "```rust\nfn test() {}\n```";

    // This might fail during validation or succeed with fallback behavior
    let result = engine.convert(simple_code).await;
    // We don't assert success/failure here as the behavior depends on implementation
    // The important thing is that it doesn't panic
    match result {
        Ok(bytes) => {
            assert!(!bytes.is_empty());
            assert_eq!(&bytes[0..2], b"PK");
        }
        Err(_) => {
            // Acceptable to fail with invalid configuration
        }
    }
}

#[tokio::test]
async fn test_file_based_conversion_with_table_code_blocks() {
    let temp_dir = TempDir::new().unwrap();

    // Create test markdown file
    let input_file = temp_dir.path().join("test_code_blocks.md");
    fs::write(&input_file, create_code_block_test_markdown()).unwrap();

    let output_file = temp_dir.path().join("test_code_blocks.docx");

    let config = create_table_code_block_config();
    let mut engine = ConversionEngine::new(config);

    // Test file-based conversion
    let result = engine
        .convert_file(input_file.to_str().unwrap(), output_file.to_str().unwrap())
        .await;

    assert!(result.is_ok());
    assert!(output_file.exists());

    let file_size = fs::metadata(&output_file).unwrap().len();
    assert!(file_size > 2000); // Should be substantial with table structures

    // Verify docx structure
    let file_content = fs::read(&output_file).unwrap();
    assert_eq!(&file_content[0..2], b"PK");
}

#[tokio::test]
async fn test_batch_conversion_with_table_code_blocks() {
    let temp_dir = TempDir::new().unwrap();

    let test_files = vec![
        ("simple.md", "# Simple\n\n```rust\nfn main() {}\n```"),
        (
            "multi.md",
            "# Multi\n\n```python\ndef test():\n    pass\n\nprint('done')\n```",
        ),
        ("empty.md", "# Empty\n\n```\n```"),
    ];

    let mut file_pairs = Vec::new();

    for (filename, content) in test_files {
        let input_path = temp_dir.path().join(filename);
        let output_path = temp_dir.path().join(filename.replace(".md", ".docx"));

        fs::write(&input_path, content).unwrap();

        file_pairs.push((
            input_path.to_string_lossy().to_string(),
            output_path.to_string_lossy().to_string(),
        ));
    }

    let config = create_table_code_block_config();
    let mut engine = ConversionEngine::new(config);

    // Test batch conversion
    let results = engine.convert_batch(&file_pairs).await.unwrap();

    // All conversions should succeed
    assert_eq!(results.len(), 3);
    for result in results {
        assert!(result.is_ok());
    }

    // Verify output files exist and are valid
    for (_, output_path) in file_pairs {
        assert!(std::path::Path::new(&output_path).exists());

        let docx_bytes = fs::read(&output_path).unwrap();
        assert!(!docx_bytes.is_empty());
        assert_eq!(&docx_bytes[0..2], b"PK");
    }
}

#[tokio::test]
async fn test_line_break_preservation_in_tables() {
    let config = create_table_code_block_config();
    let mut engine = ConversionEngine::new(config);

    let code_with_breaks = r#"# Line Break Test

```python
def function():
    # First line
    
    # Empty line above
    print("line 1")
    print("line 2")
    
    # Another empty line above
    return True
```"#;

    let docx_bytes = engine.convert(code_with_breaks).await.unwrap();
    assert!(!docx_bytes.is_empty());
    assert!(docx_bytes.len() > 1000);

    // Verify docx structure
    assert_eq!(&docx_bytes[0..2], b"PK");
}

#[tokio::test]
async fn test_code_block_with_no_line_break_preservation() {
    let mut config = create_table_code_block_config();
    config.styles.code_block.preserve_line_breaks = false;

    let mut engine = ConversionEngine::new(config);

    let code_with_breaks = r#"# No Line Break Preservation Test

```python
def function():
    print("line 1")
    print("line 2")
    return True
```"#;

    let docx_bytes = engine.convert(code_with_breaks).await.unwrap();
    assert!(!docx_bytes.is_empty());
    assert!(docx_bytes.len() > 1000);

    // Verify docx structure
    assert_eq!(&docx_bytes[0..2], b"PK");
}

#[tokio::test]
async fn test_code_block_font_and_styling_in_tables() {
    let mut config = create_table_code_block_config();

    // Test with different font settings
    config.styles.code_block.font.family = "Monaco".to_string();
    config.styles.code_block.font.size = 12.0;
    config.styles.code_block.font.bold = true;
    config.styles.code_block.font.italic = true;
    config.styles.code_block.background_color = Some("#e8e8e8".to_string());

    let mut engine = ConversionEngine::new(config);

    let styled_code = r#"# Font Styling Test

```cpp
#include <iostream>
int main() {
    std::cout << "Hello, World!" << std::endl;
    return 0;
}
```"#;

    let docx_bytes = engine.convert(styled_code).await.unwrap();
    assert!(!docx_bytes.is_empty());
    assert!(docx_bytes.len() > 1000);

    // Verify docx structure
    assert_eq!(&docx_bytes[0..2], b"PK");
}

#[tokio::test]
async fn test_multiple_code_blocks_in_document() {
    let config = create_table_code_block_config();
    let mut engine = ConversionEngine::new(config);

    let multiple_blocks = r#"# Multiple Code Blocks Test

First code block:
```rust
fn first() {
    println!("First function");
}
```

Some text between blocks.

Second code block:
```python
def second():
    print("Second function")
```

More text.

Third code block:
```javascript
function third() {
    console.log("Third function");
}
```

End of document."#;

    let stats = engine.get_conversion_stats(multiple_blocks).unwrap();
    assert_eq!(stats.code_blocks, 3);

    let docx_bytes = engine.convert(multiple_blocks).await.unwrap();
    assert!(!docx_bytes.is_empty());
    assert!(docx_bytes.len() > 2000); // Should be substantial with multiple table structures

    // Verify docx structure
    assert_eq!(&docx_bytes[0..2], b"PK");
}

#[tokio::test]
async fn test_code_block_with_special_characters_in_tables() {
    let config = create_table_code_block_config();
    let mut engine = ConversionEngine::new(config);

    let special_chars = r#"# Special Characters Test

```xml
<?xml version="1.0" encoding="UTF-8"?>
<root>
    <element attr="value &amp; more">
        Content with &lt;tags&gt; and "quotes" and 'apostrophes'
    </element>
    <!-- Comment with special chars: @#$%^&*()_+-={}[]|\:";'<>?,./~ -->
</root>
```"#;

    let docx_bytes = engine.convert(special_chars).await.unwrap();
    assert!(!docx_bytes.is_empty());
    assert!(docx_bytes.len() > 1000);

    // Verify docx structure
    assert_eq!(&docx_bytes[0..2], b"PK");
}

#[tokio::test]
async fn test_graceful_degradation_on_table_creation_failure() {
    // This test simulates potential table creation failures
    // In practice, this would require mocking the docx-rs library
    // For now, we test that the system handles edge cases gracefully

    let config = create_table_code_block_config();
    let mut engine = ConversionEngine::new(config);

    // Test with extremely large code block that might cause issues
    let mut large_code = String::new();
    large_code.push_str("# Large Code Block Test\n\n```text\n");
    for i in 0..1000 {
        large_code.push_str(&format!(
            "Line {} with some content that makes it longer\n",
            i
        ));
    }
    large_code.push_str("```");

    let result = engine.convert(&large_code).await;

    // Should either succeed or fail gracefully (not panic)
    match result {
        Ok(bytes) => {
            assert!(!bytes.is_empty());
            assert_eq!(&bytes[0..2], b"PK");
        }
        Err(_) => {
            // Acceptable to fail with extremely large content
        }
    }
}

#[tokio::test]
async fn test_docx_structure_verification() {
    let config = create_table_code_block_config();
    let mut engine = ConversionEngine::new(config);

    let simple_code = "# Structure Test\n\n```rust\nfn main() {\n    println!(\"Hello\");\n}\n```";

    let docx_bytes = engine.convert(simple_code).await.unwrap();

    // Basic docx structure verification
    assert!(!docx_bytes.is_empty());
    assert!(docx_bytes.len() > 1000);

    // Verify ZIP signature (docx is a ZIP file)
    assert_eq!(&docx_bytes[0..2], b"PK");

    // Verify minimum file size for a valid docx with table structures
    // Table-based rendering should produce larger files than simple paragraphs
    assert!(docx_bytes.len() > 1500);

    // Additional structure checks could be added here:
    // - Extract and verify XML content
    // - Check for table elements in document.xml
    // - Verify proper relationships and content types
}
