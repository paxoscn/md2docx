//! Main conversion engine that orchestrates Markdown parsing and docx generation

use crate::config::ConversionConfig;
use crate::docx::DocxGenerator;
use crate::error::ConversionError;
use crate::markdown::MarkdownParser;
use std::fs;
use std::path::Path;
use tracing::{info, debug, error, instrument};

/// Main conversion engine that coordinates the conversion process
pub struct ConversionEngine {
    config: ConversionConfig,
    markdown_parser: MarkdownParser,
    docx_generator: DocxGenerator,
}

impl ConversionEngine {
    /// Create a new conversion engine with the given configuration
    pub fn new(config: ConversionConfig) -> Self {
        info!("Creating new conversion engine with config");
        debug!("Configuration: {:?}", config);
        
        Self {
            config: config.clone(),
            markdown_parser: MarkdownParser::new(),
            docx_generator: DocxGenerator::new(config),
        }
    }

    /// Convert Markdown string to docx bytes
    #[instrument(skip(self, markdown), fields(markdown_length = markdown.len()))]
    pub async fn convert(&mut self, markdown: &str) -> Result<Vec<u8>, ConversionError> {
        info!("Starting Markdown to docx conversion");
        debug!("Markdown content length: {} characters", markdown.len());
        
        // Step 1: Parse Markdown to AST
        debug!("Parsing Markdown to AST");
        let document = self.markdown_parser.parse(markdown)
            .map_err(|e| {
                error!("Failed to parse Markdown: {}", e);
                e
            })?;
        
        info!("Successfully parsed Markdown into {} elements", document.elements.len());
        debug!("Document elements: {:?}", document.elements);
        
        // Step 2: Generate docx from AST
        debug!("Generating docx from AST");
        let docx_bytes = self.docx_generator.generate(&document)
            .map_err(|e| {
                error!("Failed to generate docx: {}", e);
                e
            })?;
        
        info!("Successfully generated docx document ({} bytes)", docx_bytes.len());
        Ok(docx_bytes)
    }

    /// Convert Markdown file to docx file
    #[instrument(skip(self), fields(input_path, output_path))]
    pub async fn convert_file(&mut self, input_path: &str, output_path: &str) -> Result<(), ConversionError> {
        info!("Starting file conversion from {} to {}", input_path, output_path);
        
        // Validate input file exists
        if !Path::new(input_path).exists() {
            let error_msg = format!("Input file does not exist: {}", input_path);
            error!("{}", error_msg);
            return Err(ConversionError::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                error_msg,
            )));
        }
        
        // Read Markdown file
        debug!("Reading Markdown file: {}", input_path);
        let markdown_content = fs::read_to_string(input_path)
            .map_err(|e| {
                error!("Failed to read input file {}: {}", input_path, e);
                ConversionError::Io(e)
            })?;
        
        info!("Read {} characters from input file", markdown_content.len());
        
        // Convert to docx
        let docx_bytes = self.convert(&markdown_content).await?;
        
        // Create output directory if it doesn't exist
        if let Some(parent) = Path::new(output_path).parent() {
            if !parent.exists() {
                debug!("Creating output directory: {:?}", parent);
                fs::create_dir_all(parent)
                    .map_err(|e| {
                        error!("Failed to create output directory {:?}: {}", parent, e);
                        ConversionError::Io(e)
                    })?;
            }
        }
        
        // Write docx file
        debug!("Writing docx file: {}", output_path);
        fs::write(output_path, docx_bytes)
            .map_err(|e| {
                error!("Failed to write output file {}: {}", output_path, e);
                ConversionError::Io(e)
            })?;
        
        info!("Successfully converted {} to {}", input_path, output_path);
        Ok(())
    }

    /// Convert multiple Markdown files to docx files
    #[instrument(skip(self, files))]
    pub async fn convert_batch(&mut self, files: &[(String, String)]) -> Result<Vec<Result<(), ConversionError>>, ConversionError> {
        info!("Starting batch conversion of {} files", files.len());
        
        let mut results = Vec::new();
        
        for (i, (input_path, output_path)) in files.iter().enumerate() {
            info!("Processing file {} of {}: {}", i + 1, files.len(), input_path);
            
            let result = self.convert_file(input_path, output_path).await;
            
            match &result {
                Ok(_) => info!("Successfully converted: {} -> {}", input_path, output_path),
                Err(e) => error!("Failed to convert {}: {}", input_path, e),
            }
            
            results.push(result);
        }
        
        let successful = results.iter().filter(|r| r.is_ok()).count();
        let failed = results.len() - successful;
        
        info!("Batch conversion completed: {} successful, {} failed", successful, failed);
        
        Ok(results)
    }

    /// Get current configuration
    pub fn config(&self) -> &ConversionConfig {
        &self.config
    }

    /// Update configuration and recreate generators
    #[instrument(skip(self, config))]
    pub fn update_config(&mut self, config: ConversionConfig) {
        info!("Updating conversion engine configuration");
        debug!("New configuration: {:?}", config);
        
        self.config = config.clone();
        self.docx_generator = DocxGenerator::new(config);
        
        info!("Configuration updated successfully");
    }

    /// Validate configuration
    pub fn validate_config(&self) -> Result<(), ConversionError> {
        debug!("Validating configuration");
        
        self.config.validate()
            .map_err(|e| {
                error!("Configuration validation failed: {}", e);
                ConversionError::Configuration(crate::error::ConfigError::Validation(e.to_string()))
            })?;
        
        debug!("Configuration validation successful");
        Ok(())
    }

    /// Get conversion statistics for a markdown string
    #[instrument(skip(self, markdown))]
    pub fn get_conversion_stats(&self, markdown: &str) -> Result<ConversionStats, ConversionError> {
        debug!("Calculating conversion statistics");
        
        let document = self.markdown_parser.parse(markdown)?;
        
        let mut stats = ConversionStats::default();
        stats.total_elements = document.elements.len();
        
        for element in &document.elements {
            match element {
                crate::markdown::ast::MarkdownElement::Heading { .. } => stats.headings += 1,
                crate::markdown::ast::MarkdownElement::Paragraph { .. } => stats.paragraphs += 1,
                crate::markdown::ast::MarkdownElement::CodeBlock { .. } => stats.code_blocks += 1,
                crate::markdown::ast::MarkdownElement::List { .. } => stats.lists += 1,
                crate::markdown::ast::MarkdownElement::Table { .. } => stats.tables += 1,
                crate::markdown::ast::MarkdownElement::Image { .. } => stats.images += 1,
                crate::markdown::ast::MarkdownElement::HorizontalRule => stats.horizontal_rules += 1,
            }
        }
        
        debug!("Conversion statistics: {:?}", stats);
        Ok(stats)
    }
}

/// Statistics about a conversion
#[derive(Debug, Default, Clone)]
pub struct ConversionStats {
    pub total_elements: usize,
    pub headings: usize,
    pub paragraphs: usize,
    pub code_blocks: usize,
    pub lists: usize,
    pub tables: usize,
    pub images: usize,
    pub horizontal_rules: usize,
}

impl ConversionStats {
    /// Get a summary string of the statistics
    pub fn summary(&self) -> String {
        format!(
            "Total elements: {}, Headings: {}, Paragraphs: {}, Code blocks: {}, Lists: {}, Tables: {}, Images: {}, Horizontal rules: {}",
            self.total_elements,
            self.headings,
            self.paragraphs,
            self.code_blocks,
            self.lists,
            self.tables,
            self.images,
            self.horizontal_rules
        )
    }
}
#[
cfg(test)]
mod tests {
    use super::*;
    use crate::config::ConversionConfig;
    use std::fs;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_convert_simple_markdown() {
        let config = ConversionConfig::default();
        let mut engine = ConversionEngine::new(config);
        
        let markdown = "# Hello World\n\nThis is a **bold** paragraph.";
        let result = engine.convert(markdown).await;
        
        assert!(result.is_ok());
        let docx_bytes = result.unwrap();
        assert!(!docx_bytes.is_empty());
    }

    #[tokio::test]
    async fn test_convert_complex_markdown() {
        let config = ConversionConfig::default();
        let mut engine = ConversionEngine::new(config);
        
        let markdown = r#"
# Main Title

This is a paragraph with **bold** and *italic* text.

## Subtitle

Here's a code block:

```rust
fn main() {
    println!("Hello, world!");
}
```

### List Example

- Item 1
- Item 2
- Item 3

### Table Example

| Name | Age | City |
|------|-----|------|
| Alice | 30 | NYC |
| Bob | 25 | LA |

![Test Image](https://example.com/image.jpg)

---

That's all!
"#;
        
        let result = engine.convert(markdown).await;
        assert!(result.is_ok());
        
        let docx_bytes = result.unwrap();
        assert!(!docx_bytes.is_empty());
    }

    #[tokio::test]
    async fn test_convert_file() {
        let config = ConversionConfig::default();
        let mut engine = ConversionEngine::new(config);
        
        // Create temporary directory
        let temp_dir = TempDir::new().unwrap();
        let input_path = temp_dir.path().join("test.md");
        let output_path = temp_dir.path().join("test.docx");
        
        // Write test markdown file
        let markdown = "# Test Document\n\nThis is a test paragraph.";
        fs::write(&input_path, markdown).unwrap();
        
        // Convert file
        let result = engine.convert_file(
            input_path.to_str().unwrap(),
            output_path.to_str().unwrap()
        ).await;
        
        assert!(result.is_ok());
        assert!(output_path.exists());
        
        // Check that output file is not empty
        let output_size = fs::metadata(&output_path).unwrap().len();
        assert!(output_size > 0);
    }

    #[tokio::test]
    async fn test_convert_file_nonexistent_input() {
        let config = ConversionConfig::default();
        let mut engine = ConversionEngine::new(config);
        
        let result = engine.convert_file("nonexistent.md", "output.docx").await;
        assert!(result.is_err());
        
        match result.unwrap_err() {
            ConversionError::Io(_) => {}, // Expected
            _ => panic!("Expected IO error for nonexistent file"),
        }
    }

    #[tokio::test]
    async fn test_convert_batch() {
        let config = ConversionConfig::default();
        let mut engine = ConversionEngine::new(config);
        
        // Create temporary directory
        let temp_dir = TempDir::new().unwrap();
        
        // Create test files
        let files = vec![
            ("test1.md", "# Document 1\n\nContent 1"),
            ("test2.md", "# Document 2\n\nContent 2"),
            ("test3.md", "# Document 3\n\nContent 3"),
        ];
        
        let mut file_pairs = Vec::new();
        
        for (filename, content) in files {
            let input_path = temp_dir.path().join(filename);
            let output_path = temp_dir.path().join(filename.replace(".md", ".docx"));
            
            fs::write(&input_path, content).unwrap();
            
            file_pairs.push((
                input_path.to_string_lossy().to_string(),
                output_path.to_string_lossy().to_string(),
            ));
        }
        
        // Convert batch
        let results = engine.convert_batch(&file_pairs).await.unwrap();
        
        // Check all conversions succeeded
        assert_eq!(results.len(), 3);
        for result in results {
            assert!(result.is_ok());
        }
        
        // Check output files exist
        for (_, output_path) in file_pairs {
            assert!(Path::new(&output_path).exists());
        }
    }

    #[test]
    fn test_update_config() {
        let mut config = ConversionConfig::default();
        config.document.default_font.size = 14.0;
        
        let mut engine = ConversionEngine::new(config.clone());
        
        // Update config
        config.document.default_font.size = 16.0;
        engine.update_config(config.clone());
        
        assert_eq!(engine.config().document.default_font.size, 16.0);
    }

    #[test]
    fn test_validate_config() {
        let config = ConversionConfig::default();
        let engine = ConversionEngine::new(config);
        
        let result = engine.validate_config();
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_invalid_config() {
        let mut config = ConversionConfig::default();
        config.document.page_size.width = -100.0; // Invalid
        
        let engine = ConversionEngine::new(config);
        
        let result = engine.validate_config();
        assert!(result.is_err());
    }

    #[test]
    fn test_conversion_stats() {
        let config = ConversionConfig::default();
        let engine = ConversionEngine::new(config);
        
        let markdown = r#"
# Title

This is a paragraph.

## Subtitle

```rust
code block
```

- List item

| Table | Header |
|-------|--------|
| Cell  | Data   |

![Image](image.jpg)

---
"#;
        
        let stats = engine.get_conversion_stats(markdown).unwrap();
        
        assert_eq!(stats.headings, 2);
        assert_eq!(stats.paragraphs, 1);
        assert_eq!(stats.code_blocks, 1);
        assert_eq!(stats.lists, 1);
        assert_eq!(stats.tables, 1);
        assert_eq!(stats.images, 1);
        assert_eq!(stats.horizontal_rules, 1);
        assert_eq!(stats.total_elements, 8);
    }

    #[test]
    fn test_conversion_stats_summary() {
        let stats = ConversionStats {
            total_elements: 5,
            headings: 2,
            paragraphs: 2,
            code_blocks: 1,
            lists: 0,
            tables: 0,
            images: 0,
            horizontal_rules: 0,
        };
        
        let summary = stats.summary();
        assert!(summary.contains("Total elements: 5"));
        assert!(summary.contains("Headings: 2"));
        assert!(summary.contains("Paragraphs: 2"));
        assert!(summary.contains("Code blocks: 1"));
    }
}