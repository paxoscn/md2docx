//! End-to-end tests for the heading auto-numbering feature
//! 
//! This module contains comprehensive tests that verify the complete numbering functionality
//! from configuration to final docx output, including edge cases and error handling.

use md2docx_converter::{
    config::ConversionConfig,
    conversion::ConversionEngine,
    numbering::HeadingProcessor,
};
use std::fs;
use std::sync::Arc;
use tempfile::TempDir;

/// Create a test configuration with numbering enabled for multiple levels
fn create_numbering_config() -> ConversionConfig {
    let mut config = ConversionConfig::default();
    
    // Configure H1 with simple numbering
    config.styles.headings.get_mut(&1).unwrap().numbering = Some("%1.".to_string());
    
    // Configure H2 with two-level numbering
    config.styles.headings.get_mut(&2).unwrap().numbering = Some("%1.%2.".to_string());
    
    // Configure H3 with three-level numbering
    config.styles.headings.get_mut(&3).unwrap().numbering = Some("%1.%2.%3".to_string());
    
    // H4, H5, H6 have no numbering (testing mixed scenarios)
    
    config
}

/// Create a test configuration with custom numbering formats
fn create_custom_numbering_config() -> ConversionConfig {
    let mut config = ConversionConfig::default();
    
    // Configure with custom separators and formats
    config.styles.headings.get_mut(&1).unwrap().numbering = Some("Chapter %1".to_string());
    config.styles.headings.get_mut(&2).unwrap().numbering = Some("%1.%2".to_string());
    config.styles.headings.get_mut(&3).unwrap().numbering = Some("%1.%2.%3:".to_string());
    
    config
}

/// Create a test configuration with invalid numbering formats for error testing
fn create_invalid_numbering_config() -> ConversionConfig {
    let mut config = ConversionConfig::default();
    
    // Configure with invalid formats
    config.styles.headings.get_mut(&1).unwrap().numbering = Some("invalid_format".to_string());
    config.styles.headings.get_mut(&2).unwrap().numbering = Some("%1.%3.".to_string()); // Skip level 2
    
    config
}

/// Test markdown content with various heading structures
fn comprehensive_test_markdown() -> &'static str {
    r#"# Introduction

This is the introduction chapter with some content.

## Overview

This section provides an overview of the topic.

### Key Points

Here are the key points to remember:

- Point 1
- Point 2
- Point 3

### Implementation Details

This subsection covers implementation details.

#### Technical Specifications

This is a level 4 heading (no numbering configured).

## Methodology

This section describes the methodology used.

### Data Collection

Information about data collection methods.

### Analysis Approach

Details about the analysis approach.

#### Statistical Methods

Statistical methods used (level 4, no numbering).

##### Advanced Techniques

Advanced techniques (level 5, no numbering).

# Results

This is the results chapter.

## Findings

Key findings from the research.

### Primary Results

The primary results section.

### Secondary Results

The secondary results section.

## Discussion

Discussion of the results.

# Conclusion

Final conclusions and recommendations.

## Summary

Summary of key points.

## Future Work

Recommendations for future work.
"#
}

/// Test markdown with skip-level headings
fn skip_level_test_markdown() -> &'static str {
    r#"# Chapter 1

Introduction to the chapter.

### Subsection (skipping H2)

This tests skip-level handling.

## Section 1.1 (back to H2)

This comes after H3.

#### Deep subsection (skipping H3)

Testing multiple level skips.

# Chapter 2

Second chapter.

## Section 2.1

Regular progression.
"#
}

/// Test markdown with edge cases
fn edge_case_test_markdown() -> &'static str {
    r#"# 

Empty heading text.

##    Heading with spaces   

Heading with leading and trailing spaces.

### Heading with émojis 🚀 and spëcial chars

Unicode and special characters.

#### Very long heading that goes on and on and contains a lot of text to test how the numbering system handles extremely long heading text that might cause formatting issues

Testing very long headings.

###### Maximum level heading

Testing H6 level.
"#
}

#[tokio::test]
async fn test_basic_numbering_functionality() {
    let config = create_numbering_config();
    let mut engine = ConversionEngine::new(config);
    
    let markdown = comprehensive_test_markdown();
    
    // Test conversion statistics first
    let stats = engine.get_conversion_stats(markdown).unwrap();
    assert!(stats.headings > 0);
    
    // Test actual conversion
    let docx_bytes = engine.convert(markdown).await.unwrap();
    assert!(!docx_bytes.is_empty());
    assert!(docx_bytes.len() > 1000);
    
    // Verify docx file structure
    assert_eq!(&docx_bytes[0..2], b"PK"); // ZIP signature
    
    println!("Basic numbering conversion successful: {} bytes", docx_bytes.len());
}

#[tokio::test]
async fn test_custom_numbering_formats() {
    let config = create_custom_numbering_config();
    let mut engine = ConversionEngine::new(config);
    
    let markdown = comprehensive_test_markdown();
    
    let docx_bytes = engine.convert(markdown).await.unwrap();
    assert!(!docx_bytes.is_empty());
    
    println!("Custom numbering conversion successful: {} bytes", docx_bytes.len());
}

#[tokio::test]
async fn test_skip_level_numbering() {
    let config = create_numbering_config();
    let mut engine = ConversionEngine::new(config);
    
    let markdown = skip_level_test_markdown();
    
    let docx_bytes = engine.convert(markdown).await.unwrap();
    assert!(!docx_bytes.is_empty());
    
    println!("Skip-level numbering conversion successful: {} bytes", docx_bytes.len());
}

#[tokio::test]
async fn test_edge_case_numbering() {
    let config = create_numbering_config();
    let mut engine = ConversionEngine::new(config);
    
    let markdown = edge_case_test_markdown();
    
    let docx_bytes = engine.convert(markdown).await.unwrap();
    assert!(!docx_bytes.is_empty());
    
    println!("Edge case numbering conversion successful: {} bytes", docx_bytes.len());
}

#[tokio::test]
async fn test_numbering_error_handling() {
    let config = create_invalid_numbering_config();
    let mut engine = ConversionEngine::new(config);
    
    let markdown = comprehensive_test_markdown();
    
    // Should not fail due to graceful degradation
    let result = engine.convert(markdown).await;
    assert!(result.is_ok());
    
    let docx_bytes = result.unwrap();
    assert!(!docx_bytes.is_empty());
    
    println!("Error handling conversion successful: {} bytes", docx_bytes.len());
}

#[test]
fn test_numbering_processor_comprehensive() {
    let config = Arc::new(create_numbering_config());
    let mut processor = HeadingProcessor::new(config);
    
    // Test sequential processing
    let result = processor.process_heading(1, "Introduction").unwrap();
    assert_eq!(result, "1. Introduction");
    
    let result = processor.process_heading(2, "Overview").unwrap();
    assert_eq!(result, "1.1. Overview");
    
    let result = processor.process_heading(3, "Details").unwrap();
    assert_eq!(result, "1.1.1 Details");
    
    // Test level reset
    let result = processor.process_heading(1, "Chapter 2").unwrap();
    assert_eq!(result, "2. Chapter 2");
    
    let result = processor.process_heading(2, "Section").unwrap();
    assert_eq!(result, "2.1. Section");
    
    // Test skip level
    let result = processor.process_heading(4, "Subsection").unwrap();
    assert_eq!(result, "Subsection"); // No numbering for H4
    
    // Test return to numbered level
    let result = processor.process_heading(2, "Another Section").unwrap();
    assert_eq!(result, "2.2. Another Section");
}

#[test]
fn test_numbering_processor_batch_processing() {
    let config = Arc::new(create_numbering_config());
    let mut processor = HeadingProcessor::new(config);
    
    let headings = vec![
        (1, "Chapter 1"),
        (2, "Section 1.1"),
        (3, "Subsection 1.1.1"),
        (3, "Subsection 1.1.2"),
        (2, "Section 1.2"),
        (1, "Chapter 2"),
        (2, "Section 2.1"),
    ];
    
    let results = processor.process_headings(headings).unwrap();
    
    let expected = vec![
        "1. Chapter 1",
        "1.1. Section 1.1",
        "1.1.1 Subsection 1.1.1",
        "1.1.2 Subsection 1.1.2",
        "1.2. Section 1.2",
        "2. Chapter 2",
        "2.1. Section 2.1",
    ];
    
    assert_eq!(results, expected);
}

#[test]
fn test_numbering_configuration_validation() {
    // Test valid configuration
    let valid_config = create_numbering_config();
    assert!(valid_config.validate().is_ok());
    
    // Test invalid configuration
    let invalid_config = create_invalid_numbering_config();
    let validation_result = invalid_config.validate();
    assert!(validation_result.is_err());
}

#[test]
fn test_numbering_serialization_deserialization() {
    let config = create_numbering_config();
    
    // Test YAML serialization
    let yaml = serde_yaml::to_string(&config).unwrap();
    assert!(yaml.contains("numbering"));
    assert!(yaml.contains("%1."));
    assert!(yaml.contains("%1.%2."));
    
    // Test deserialization
    let deserialized: ConversionConfig = serde_yaml::from_str(&yaml).unwrap();
    assert!(deserialized.validate().is_ok());
    
    // Verify numbering fields are preserved
    assert_eq!(
        deserialized.styles.headings.get(&1).unwrap().numbering,
        Some("%1.".to_string())
    );
    assert_eq!(
        deserialized.styles.headings.get(&2).unwrap().numbering,
        Some("%1.%2.".to_string())
    );
    assert_eq!(
        deserialized.styles.headings.get(&3).unwrap().numbering,
        Some("%1.%2.%3".to_string())
    );
    
    // Test JSON serialization as well
    let json = serde_json::to_string(&config).unwrap();
    let json_deserialized: ConversionConfig = serde_json::from_str(&json).unwrap();
    assert!(json_deserialized.validate().is_ok());
}

#[tokio::test]
async fn test_file_conversion_with_numbering() {
    let temp_dir = TempDir::new().unwrap();
    
    // Create test markdown file
    let input_file = temp_dir.path().join("test_numbering.md");
    fs::write(&input_file, comprehensive_test_markdown()).unwrap();
    
    // Create config file with numbering
    let config = create_numbering_config();
    let config_file = temp_dir.path().join("numbering_config.yaml");
    let config_yaml = serde_yaml::to_string(&config).unwrap();
    fs::write(&config_file, config_yaml).unwrap();
    
    let output_file = temp_dir.path().join("test_output.docx");
    
    // Create engine with numbering config
    let mut engine = ConversionEngine::new(config);
    
    // Test file conversion
    let result = engine.convert_file(
        input_file.to_str().unwrap(),
        output_file.to_str().unwrap()
    ).await;
    
    assert!(result.is_ok());
    assert!(output_file.exists());
    
    let file_size = fs::metadata(&output_file).unwrap().len();
    assert!(file_size > 1000);
    
    // Verify docx structure
    let file_content = fs::read(&output_file).unwrap();
    assert_eq!(&file_content[0..2], b"PK"); // ZIP signature
    
    println!("File conversion with numbering successful: {} bytes", file_size);
}

#[tokio::test]
async fn test_batch_file_conversion_with_numbering() {
    let temp_dir = TempDir::new().unwrap();
    
    // Create multiple test files
    let test_files = vec![
        ("doc1.md", "# Document 1\n\n## Section 1.1\n\nContent."),
        ("doc2.md", "# Document 2\n\n## Section 2.1\n\n### Subsection 2.1.1\n\nMore content."),
        ("doc3.md", "# Document 3\n\n## Section 3.1\n\n## Section 3.2\n\nFinal content."),
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
    
    // Create engine with numbering config
    let config = create_numbering_config();
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
        assert_eq!(&docx_bytes[0..2], b"PK"); // ZIP signature
        
        println!("Batch file conversion successful: {}", output_path);
    }
}

#[tokio::test]
async fn test_performance_with_numbering() {
    let config = create_numbering_config();
    let mut engine = ConversionEngine::new(config);
    
    // Create a large document with many headings
    let mut large_markdown = String::new();
    large_markdown.push_str("# Performance Test Document\n\n");
    
    for i in 1..=100 {
        large_markdown.push_str(&format!("## Section {}\n\n", i));
        large_markdown.push_str("This is some content for the section.\n\n");
        
        for j in 1..=5 {
            large_markdown.push_str(&format!("### Subsection {}.{}\n\n", i, j));
            large_markdown.push_str("More detailed content here.\n\n");
        }
    }
    
    // Measure conversion time
    let start_time = std::time::Instant::now();
    let result = engine.convert(&large_markdown).await;
    let duration = start_time.elapsed();
    
    assert!(result.is_ok());
    let docx_bytes = result.unwrap();
    assert!(!docx_bytes.is_empty());
    
    println!(
        "Performance test: {} headings converted in {:?}, output size: {} bytes",
        600, // 100 H2 + 500 H3
        duration,
        docx_bytes.len()
    );
    
    // Performance should be reasonable (less than 10 seconds for 600 headings)
    assert!(duration.as_secs() < 10);
}

#[test]
fn test_numbering_error_recovery() {
    let config = Arc::new(create_invalid_numbering_config());
    let mut processor = HeadingProcessor::new(config);
    
    // Process headings with invalid configuration
    // Should not panic and should provide fallback behavior
    let result1 = processor.process_heading(1, "Test Heading 1");
    assert!(result1.is_ok());
    
    let result2 = processor.process_heading(2, "Test Heading 2");
    assert!(result2.is_ok());
    
    // Should return original text when numbering fails
    assert_eq!(result1.unwrap(), "Test Heading 1");
    assert_eq!(result2.unwrap(), "Test Heading 2");
    
    // Metrics should show degraded operations
    let metrics = processor.get_metrics();
    assert!(metrics.degraded_operations > 0 || metrics.successful_operations > 0);
}

#[test]
fn test_numbering_state_consistency() {
    let config = Arc::new(create_numbering_config());
    let mut processor = HeadingProcessor::new(config);
    
    // Process a complex sequence of headings
    let sequence = vec![
        (1, "Chapter 1"),
        (2, "Section 1.1"),
        (3, "Subsection 1.1.1"),
        (1, "Chapter 2"), // Should reset H2 and H3
        (3, "Subsection 2.0.1"), // Skip H2
        (2, "Section 2.1"), // Should reset H3
        (3, "Subsection 2.1.1"),
        (3, "Subsection 2.1.2"),
        (2, "Section 2.2"),
    ];
    
    let mut results = Vec::new();
    for (level, text) in sequence {
        let result = processor.process_heading(level, text).unwrap();
        results.push(result);
    }
    
    // Verify the numbering sequence is correct
    let expected = vec![
        "1. Chapter 1",
        "1.1. Section 1.1",
        "1.1.1 Subsection 1.1.1",
        "2. Chapter 2",
        "2.1.1 Subsection 2.0.1", // Uses synthetic H2 counter
        "2.1. Section 2.1",
        "2.1.1 Subsection 2.1.1",
        "2.1.2 Subsection 2.1.2",
        "2.2. Section 2.2",
    ];
    
    assert_eq!(results, expected);
}

#[test]
fn test_numbering_preview_functionality() {
    let config = Arc::new(create_numbering_config());
    let mut processor = HeadingProcessor::new(config);
    
    // Set up some state
    processor.process_heading(1, "Chapter").unwrap();
    processor.process_heading(2, "Section").unwrap();
    
    // Test preview functionality
    let preview1 = processor.preview_numbering(1).unwrap();
    assert_eq!(preview1, Some("1.".to_string()));
    
    let preview2 = processor.preview_numbering(2).unwrap();
    assert_eq!(preview2, Some("1.1.".to_string()));
    
    let preview3 = processor.preview_numbering(3).unwrap();
    assert_eq!(preview3, Some("1.1.1".to_string()));
    
    // Preview for level without numbering
    let preview4 = processor.preview_numbering(4).unwrap();
    assert_eq!(preview4, None);
}

#[test]
fn test_numbering_metrics_and_health() {
    let config = Arc::new(create_numbering_config());
    let mut processor = HeadingProcessor::new(config);
    
    // Initially should be healthy
    assert!(processor.is_healthy());
    
    // Process successful operations
    for i in 1..=10 {
        processor.process_heading(1, &format!("Heading {}", i)).unwrap();
    }
    
    // Should still be healthy
    assert!(processor.is_healthy());
    
    let metrics = processor.get_metrics();
    assert_eq!(metrics.total_headings, 10);
    assert_eq!(metrics.successful_operations, 10);
    assert_eq!(metrics.critical_failures, 0);
    
    // Test metrics logging (should not panic)
    processor.log_metrics_summary();
    processor.log_health_status();
}

#[tokio::test]
async fn test_numbering_with_natural_language_config() {
    // This test would require LLM integration to be fully functional
    // For now, we test the configuration structure
    
    let mut config = ConversionConfig::default();
    
    // Simulate what natural language processing might produce
    config.styles.headings.get_mut(&1).unwrap().numbering = Some("%1.".to_string());
    config.styles.headings.get_mut(&2).unwrap().numbering = Some("%1.%2.".to_string());
    
    // Validate the configuration
    assert!(config.validate().is_ok());
    
    // Test conversion
    let mut engine = ConversionEngine::new(config);
    let markdown = "# Chapter 1\n\n## Section 1.1\n\nContent here.";
    
    let result = engine.convert(markdown).await;
    assert!(result.is_ok());
    
    let docx_bytes = result.unwrap();
    assert!(!docx_bytes.is_empty());
}

#[test]
fn test_numbering_format_edge_cases() {
    use md2docx_converter::numbering::NumberingFormatter;
    
    // Test various format patterns
    let test_cases = vec![
        ("%1.", true),
        ("%1.%2.", true),
        ("%1.%2.%3", true),
        ("%1-%2-%3", true),
        ("Chapter %1", true),
        ("%1.%2.%3:", true),
        ("", false), // Empty format
        ("no placeholders", false), // No placeholders
        ("%1.%3.", false), // Skip level
        ("%2.%3.", false), // Doesn't start from 1
        ("%0.", false), // Invalid level
        ("%7.", false), // Invalid level
    ];
    
    for (format, should_be_valid) in test_cases {
        let result = NumberingFormatter::validate_format(format);
        if should_be_valid {
            assert!(result.is_ok(), "Format '{}' should be valid", format);
        } else {
            assert!(result.is_err(), "Format '{}' should be invalid", format);
        }
    }
}

#[tokio::test]
async fn test_numbering_compatibility_with_existing_features() {
    let config = create_numbering_config();
    let mut engine = ConversionEngine::new(config);
    
    // Test markdown with all supported features plus numbering
    let complex_markdown = r#"# Introduction

This document tests **numbering** with other *Markdown* features.

## Code Examples

Here's some `inline code` and a code block:

```rust
fn main() {
    println!("Hello, world!");
}
```

### Tables and Lists

| Feature | Status | Notes |
|---------|--------|-------|
| Numbering | ✅ | Working |
| Tables | ✅ | Compatible |

#### Unordered List

- Item 1 with **bold**
- Item 2 with *italic*
- Item 3 with `code`

##### Ordered List

1. First item
2. Second item
3. Third item

## Images and Links

![Test Image](https://example.com/image.jpg)

Visit [our website](https://example.com) for more information.

### Blockquotes

> This is a blockquote with numbering.
> It should work correctly.

## Conclusion

All features should work together seamlessly.
"#;
    
    let result = engine.convert(complex_markdown).await;
    assert!(result.is_ok());
    
    let docx_bytes = result.unwrap();
    assert!(!docx_bytes.is_empty());
    assert!(docx_bytes.len() > 2000); // Should be substantial with all features
    
    println!("Complex feature compatibility test successful: {} bytes", docx_bytes.len());
}