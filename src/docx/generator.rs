//! docx document generator

use crate::config::ConversionConfig;
use crate::error::ConversionError;
use crate::markdown::{MarkdownDocument, MarkdownElement, InlineElement, ListItem};
use docx_rs::*;
use std::io::Cursor;
use std::str::FromStr;
use tracing::{info, debug, error, instrument};

/// Generator for creating docx documents from Markdown AST
pub struct DocxGenerator {
    config: ConversionConfig,
}

impl DocxGenerator {
    /// Create a new docx generator with the given configuration
    pub fn new(config: ConversionConfig) -> Self {
        Self { config }
    }

    /// Generate docx document from Markdown AST
    pub fn generate(&self, document: &MarkdownDocument) -> Result<Vec<u8>, ConversionError> {
        let mut docx = Docx::new();
        
        // Apply document-level settings
        docx = self.apply_document_settings(docx)?;
        
        // Process each markdown element
        for element in &document.elements {
            docx = self.process_element(docx, element)?;
        }
        
        // Build and return the document bytes
        let mut buf = Vec::new();
        let mut cursor = Cursor::new(&mut buf);
        docx.build()
            .pack(&mut cursor)
            .map_err(|e| ConversionError::DocxGeneration(format!("Failed to build docx: {}", e)))?;
        
        Ok(buf)
    }

    /// Apply document-level settings (page size, margins, default font)
    fn apply_document_settings(&self, mut docx: Docx) -> Result<Docx, ConversionError> {
        let doc_config = &self.config.document;
        
        // Apply page settings - docx.page_size takes width and height as separate u32 parameters
        // FIXME Commented out for unexpected small page size
        // docx = docx.page_size(
        //     doc_config.page_size.width as u32,
        //     doc_config.page_size.height as u32
        // );
        
        // Apply margins - create PageMargin and apply it
        let page_margin = PageMargin::new()
            .top(doc_config.margins.top as i32)
            .bottom(doc_config.margins.bottom as i32)
            .left(doc_config.margins.left as i32)
            .right(doc_config.margins.right as i32);
        
        docx = docx.page_margin(page_margin);
        
        Ok(docx)
    }

    /// Process a single markdown element
    fn process_element(&self, mut docx: Docx, element: &MarkdownElement) -> Result<Docx, ConversionError> {
        match element {
            MarkdownElement::Heading { level, text } => {
                docx = self.add_heading(docx, *level, text)?;
            }
            MarkdownElement::Paragraph { content } => {
                docx = self.add_paragraph(docx, content)?;
            }
            MarkdownElement::CodeBlock { language: _, code } => {
                docx = self.add_code_block(docx, code)?;
            }
            MarkdownElement::List { ordered, items } => {
                docx = self.add_list(docx, *ordered, items)?;
            }
            MarkdownElement::Table { headers, rows } => {
                docx = self.add_table(docx, headers, rows)?;
            }
            MarkdownElement::Image { alt_text, url, title: _ } => {
                docx = self.add_image(docx, alt_text, url)?;
            }
            MarkdownElement::HorizontalRule => {
                docx = self.add_horizontal_rule(docx)?;
            }
        }
        
        Ok(docx)
    }

    /// Add a heading to the document
    fn add_heading(&self, mut docx: Docx, level: u8, text: &str) -> Result<Docx, ConversionError> {
        // Get heading style from config, fallback to level 1 if not found
        let heading_style = self.config.styles.headings.get(&level)
            .unwrap_or_else(|| self.config.styles.headings.get(&1).unwrap());
        
        let mut run = Run::new()
            .add_text(text)
            .fonts(RunFonts::new().ascii(&heading_style.font.family).east_asia(&heading_style.font.family))
            .size((heading_style.font.size * 2.0) as usize); // docx uses half-points
        
        // Apply bold/italic conditionally
        if heading_style.font.bold {
            run = run.bold();
        }
        if heading_style.font.italic {
            run = run.italic();
        }
        
        // Create paragraph with spacing
        let paragraph = Paragraph::new().add_run(run);

        // Adding alignment
        let paragraph = match heading_style.alignment.clone() {
            Some(alignment) => {
                match AlignmentType::from_str(alignment.as_str()) {
                    Ok(alignment_type) => {
                        paragraph.align(alignment_type)
                    },
                    Err(e) => {
                        error!("Unknown alignment: {}. Error: {}", alignment, e);
                        paragraph
                    }
                }
            },
            None => {
                paragraph
            }
        };
        
        // Apply paragraph spacing - for now, we'll skip this as docx-rs API is different
        // TODO: Implement proper spacing when docx-rs API is clarified
        
        docx = docx.add_paragraph(paragraph);
        Ok(docx)
    }

    /// Add a paragraph to the document
    fn add_paragraph(&self, mut docx: Docx, content: &[InlineElement]) -> Result<Docx, ConversionError> {
        let mut paragraph = Paragraph::new()
        .indent(None, Some(SpecialIndentType::FirstLine(720)), None, None);
        
        for inline in content {
            let run = self.create_run_from_inline(inline)?;
            paragraph = paragraph.add_run(run);
        }
        
        // Apply paragraph style settings - for now, we'll skip spacing as docx-rs API is different
        // TODO: Implement proper paragraph spacing and line spacing when docx-rs API is clarified
        
        docx = docx.add_paragraph(paragraph);
        Ok(docx)
    }

    /// Create a run from an inline element
    fn create_run_from_inline(&self, inline: &InlineElement) -> Result<Run, ConversionError> {
        let base_font = &self.config.styles.paragraph.font;
        
        match inline {
            InlineElement::Text(text) => {
                let mut run = Run::new()
                    .add_text(text)
                    .fonts(RunFonts::new().ascii(&base_font.family).east_asia(&base_font.family))
                    .size((base_font.size * 2.0) as usize);
                
                // Apply base font formatting
                if base_font.bold {
                    run = run.bold();
                }
                if base_font.italic {
                    run = run.italic();
                }
                
                Ok(run)
            }
            InlineElement::Bold(text) => {
                Ok(Run::new()
                    .add_text(text)
                    .fonts(RunFonts::new().ascii(&base_font.family).east_asia(&base_font.family))
                    .size((base_font.size * 2.0) as usize)
                    .bold())
            }
            InlineElement::Italic(text) => {
                Ok(Run::new()
                    .add_text(text)
                    .fonts(RunFonts::new().ascii(&base_font.family).east_asia(&base_font.family))
                    .size((base_font.size * 2.0) as usize)
                    .italic())
            }
            InlineElement::Strikethrough(text) => {
                Ok(Run::new()
                    .add_text(text)
                    .fonts(RunFonts::new().ascii(&base_font.family).east_asia(&base_font.family))
                    .size((base_font.size * 2.0) as usize)
                    .strike())
            }
            InlineElement::Code(text) => {
                let code_font = &self.config.styles.code_block.font;
                let mut run = Run::new()
                    .add_text(text)
                    .fonts(RunFonts::new().ascii(&code_font.family).east_asia(&code_font.family))
                    .size((code_font.size * 2.0) as usize);
                
                // Apply code font formatting
                if code_font.bold {
                    run = run.bold();
                }
                if code_font.italic {
                    run = run.italic();
                }
                
                // Add background color if specified
                if let Some(bg_color) = &self.config.styles.code_block.background_color {
                    let color = bg_color.trim_start_matches('#');
                    run = run.highlight(color);
                }
                
                Ok(run)
            }
            InlineElement::Link { text, url: _, title: _ } => {
                let link_color = self.config.elements.link.color.trim_start_matches('#');
                let mut run = Run::new()
                    .add_text(text)
                    .fonts(RunFonts::new().ascii(&base_font.family).east_asia(&base_font.family))
                    .size((base_font.size * 2.0) as usize)
                    .color(link_color);
                
                // Add underline if configured
                if self.config.elements.link.underline {
                    run = run.underline("single");
                }
                
                Ok(run)
            }
        }
    }

    /// Add a code block to the document
    fn add_code_block(&self, mut docx: Docx, code: &str) -> Result<Docx, ConversionError> {
        let code_style = &self.config.styles.code_block;
        
        let mut run = Run::new()
            .add_text(code)
            .fonts(RunFonts::new().ascii(&code_style.font.family).east_asia(&code_style.font.family))
            .size((code_style.font.size * 2.0) as usize);
        
        // Apply bold/italic conditionally
        if code_style.font.bold {
            run = run.bold();
        }
        if code_style.font.italic {
            run = run.italic();
        }
        
        let paragraph = Paragraph::new().add_run(run);
        docx = docx.add_paragraph(paragraph);
        Ok(docx)
    }

    /// Add a list to the document
    fn add_list(&self, mut docx: Docx, ordered: bool, items: &[ListItem]) -> Result<Docx, ConversionError> {
        docx = self.add_list_with_depth(docx, ordered, items, 0)?;
        Ok(docx)
    }

    /// Add a list to the document with specified depth for indentation
    fn add_list_with_depth(&self, mut docx: Docx, ordered: bool, items: &[ListItem], depth: usize) -> Result<Docx, ConversionError> {
        let list_config = &self.config.elements.list;
        let _indent_amount = list_config.indent * (depth + 1) as f32;
        
        for (index, item) in items.iter().enumerate() {
            let bullet = if ordered {
                format!("{}. ", index + 1)
            } else {
                match depth % 3 {
                    0 => "• ".to_string(),
                    1 => "◦ ".to_string(),
                    _ => "▪ ".to_string(),
                }
            };
            
            let mut paragraph = Paragraph::new();
            
            // TODO: Add proper indentation when docx-rs API supports it
            // For now, we'll use spaces for indentation
            if depth > 0 {
                let indent_spaces = "    ".repeat(depth);
                paragraph = paragraph.add_run(Run::new().add_text(&indent_spaces));
            }
            
            // Add bullet/number
            paragraph = paragraph.add_run(Run::new().add_text(&bullet));
            
            // Add item content
            for inline in &item.content {
                let run = self.create_run_from_inline(inline)?;
                paragraph = paragraph.add_run(run);
            }
            
            docx = docx.add_paragraph(paragraph);
            
            // Handle sub-items recursively with increased depth
            if !item.sub_items.is_empty() {
                docx = self.add_list_with_depth(docx, ordered, &item.sub_items, depth + 1)?;
            }
        }
        
        Ok(docx)
    }

    /// Add a table to the document
    fn add_table(&self, mut docx: Docx, headers: &[String], rows: &[Vec<String>]) -> Result<Docx, ConversionError> {
        let table_style = &self.config.styles.table;
        
        let mut table_rows = vec![];
        
        // Add header row
        if !headers.is_empty() {
            let mut header_cells = vec![];
            
            for header in headers {
                let mut header_run = Run::new()
                    .add_text(header)
                    .fonts(RunFonts::new().ascii(&table_style.header_font.family).east_asia(&table_style.header_font.family))
                    .size((table_style.header_font.size * 2.0) as usize);
                
                if table_style.header_font.bold {
                    header_run = header_run.bold();
                }
                if table_style.header_font.italic {
                    header_run = header_run.italic();
                }
                
                let cell_paragraph = Paragraph::new().add_run(header_run);
                let cell = TableCell::new().add_paragraph(cell_paragraph);
                // TODO: Add cell borders when docx-rs API supports it properly
                header_cells.push(cell);
            }
            
            table_rows.push(TableRow::new(header_cells));
        }
        
        // Add data rows
        for row in rows {
            let mut row_cells = vec![];
            
            for cell_data in row {
                let mut cell_run = Run::new()
                    .add_text(cell_data)
                    .fonts(RunFonts::new().ascii(&table_style.cell_font.family).east_asia(&table_style.cell_font.family))
                    .size((table_style.cell_font.size * 2.0) as usize);
                
                if table_style.cell_font.bold {
                    cell_run = cell_run.bold();
                }
                if table_style.cell_font.italic {
                    cell_run = cell_run.italic();
                }
                
                let cell_paragraph = Paragraph::new().add_run(cell_run);
                let cell = TableCell::new().add_paragraph(cell_paragraph);
                // TODO: Add cell borders when docx-rs API supports it properly
                row_cells.push(cell);
            }
            
            table_rows.push(TableRow::new(row_cells));
        }
        
        let table = Table::new(table_rows);
        // TODO: Set table-wide borders when docx-rs API supports it properly
        docx = docx.add_table(table);
        Ok(docx)
    }

    /// Add an image to the document
    fn add_image(&self, mut docx: Docx, alt_text: &str, url: &str) -> Result<Docx, ConversionError> {
        let image_config = &self.config.elements.image;
        
        // Check if it's a local file path
        if self.is_local_image_path(url) {
            // Try to embed local image
            match self.embed_local_image(url, alt_text, image_config) {
                Ok(image_run) => {
                    let paragraph = Paragraph::new().add_run(image_run);
                    docx = docx.add_paragraph(paragraph);
                }
                Err(_) => {
                    // Fallback to placeholder text if image can't be loaded
                    let paragraph = Paragraph::new()
                        .add_run(Run::new().add_text(&format!("[Image: {} - File not found: {}]", alt_text, url)));
                    docx = docx.add_paragraph(paragraph);
                }
            }
        } else {
            // For remote URLs, add a placeholder with the URL
            let paragraph = Paragraph::new()
                .add_run(Run::new().add_text(&format!("[Image: {} - URL: {}]", alt_text, url)));
            docx = docx.add_paragraph(paragraph);
        }
        
        Ok(docx)
    }

    /// Check if a URL is a local image path
    fn is_local_image_path(&self, url: &str) -> bool {
        !url.starts_with("http://") && !url.starts_with("https://") && !url.starts_with("ftp://")
    }

    /// Embed a local image file
    fn embed_local_image(&self, path: &str, _alt_text: &str, image_config: &crate::config::ImageConfig) -> Result<Run, ConversionError> {
        use std::fs;
        use std::path::Path;
        
        // Check if file exists
        if !Path::new(path).exists() {
            return Err(ConversionError::DocxGeneration(format!("Image file not found: {}", path)));
        }
        
        // Read image file
        let image_data = fs::read(path)
            .map_err(|e| ConversionError::DocxGeneration(format!("Failed to read image file {}: {}", path, e)))?;
        
        // Determine image format from file extension
        let _format = self.get_image_format(path)?;
        
        // Create image with size constraints
        let image = Pic::new(&image_data)
            .size(image_config.max_width as u32, image_config.max_height as u32);
        
        // Create run with the image
        let run = Run::new().add_image(image);
        
        Ok(run)
    }

    /// Get image format from file extension
    fn get_image_format(&self, path: &str) -> Result<&'static str, ConversionError> {
        let path_lower = path.to_lowercase();
        
        if path_lower.ends_with(".png") {
            Ok("png")
        } else if path_lower.ends_with(".jpg") || path_lower.ends_with(".jpeg") {
            Ok("jpeg")
        } else if path_lower.ends_with(".gif") {
            Ok("gif")
        } else if path_lower.ends_with(".bmp") {
            Ok("bmp")
        } else {
            Err(ConversionError::DocxGeneration(format!("Unsupported image format: {}", path)))
        }
    }

    /// Add a horizontal rule to the document
    fn add_horizontal_rule(&self, mut docx: Docx) -> Result<Docx, ConversionError> {
        // Add a paragraph with a line of dashes as a simple horizontal rule
        let paragraph = Paragraph::new()
            .add_run(Run::new().add_text("─".repeat(50)));
        
        docx = docx.add_paragraph(paragraph);
        Ok(docx)
    }

    /// Get current configuration
    pub fn config(&self) -> &ConversionConfig {
        &self.config
    }

    /// Update configuration
    pub fn update_config(&mut self, config: ConversionConfig) {
        self.config = config;
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::{create_test_config, create_test_document, create_simple_document};
    use crate::config::ConversionConfig;
    use crate::markdown::ast::{MarkdownDocument, MarkdownElement, InlineElement, ListItem};

    #[test]
    fn test_heading_generation() {
        let config = ConversionConfig::default();
        let generator = DocxGenerator::new(config);
        
        let mut document = MarkdownDocument::new();
        document.add_element(MarkdownElement::Heading {
            level: 1,
            text: "Test Heading".to_string(),
        });
        
        let result = generator.generate(&document);
        assert!(result.is_ok());
        
        let docx_bytes = result.unwrap();
        assert!(!docx_bytes.is_empty());
    }

    #[test]
    fn test_paragraph_with_formatting() {
        let config = ConversionConfig::default();
        let generator = DocxGenerator::new(config);
        
        let mut document = MarkdownDocument::new();
        document.add_element(MarkdownElement::Paragraph {
            content: vec![
                InlineElement::Text("This is ".to_string()),
                InlineElement::Bold("bold".to_string()),
                InlineElement::Text(" and ".to_string()),
                InlineElement::Italic("italic".to_string()),
                InlineElement::Text(" and ".to_string()),
                InlineElement::Strikethrough("strikethrough".to_string()),
                InlineElement::Text(" text.".to_string()),
            ],
        });
        
        let result = generator.generate(&document);
        assert!(result.is_ok());
        
        let docx_bytes = result.unwrap();
        assert!(!docx_bytes.is_empty());
    }

    #[test]
    fn test_table_generation() {
        let config = ConversionConfig::default();
        let generator = DocxGenerator::new(config);
        
        let mut document = MarkdownDocument::new();
        document.add_element(MarkdownElement::Table {
            headers: vec!["Name".to_string(), "Age".to_string(), "City".to_string()],
            rows: vec![
                vec!["Alice".to_string(), "30".to_string(), "New York".to_string()],
                vec!["Bob".to_string(), "25".to_string(), "London".to_string()],
                vec!["Charlie".to_string(), "35".to_string(), "Tokyo".to_string()],
            ],
        });
        
        let result = generator.generate(&document);
        assert!(result.is_ok());
        
        let docx_bytes = result.unwrap();
        assert!(!docx_bytes.is_empty());
    }

    #[test]
    fn test_list_generation() {
        let config = ConversionConfig::default();
        let generator = DocxGenerator::new(config);
        
        let mut document = MarkdownDocument::new();
        document.add_element(MarkdownElement::List {
            ordered: false,
            items: vec![
                ListItem::new(vec![InlineElement::Text("First item".to_string())]),
                ListItem::new(vec![InlineElement::Text("Second item".to_string())]),
                ListItem::new(vec![InlineElement::Text("Third item".to_string())]),
            ],
        });
        
        let result = generator.generate(&document);
        assert!(result.is_ok());
        
        let docx_bytes = result.unwrap();
        assert!(!docx_bytes.is_empty());
    }

    #[test]
    fn test_image_placeholder_generation() {
        let config = ConversionConfig::default();
        let generator = DocxGenerator::new(config);
        
        let mut document = MarkdownDocument::new();
        document.add_element(MarkdownElement::Image {
            alt_text: "Test Image".to_string(),
            url: "https://example.com/image.jpg".to_string(),
            title: Some("Image Title".to_string()),
        });
        
        let result = generator.generate(&document);
        assert!(result.is_ok());
        
        let docx_bytes = result.unwrap();
        assert!(!docx_bytes.is_empty());
    }

    #[test]
    fn test_code_block_generation() {
        let config = create_test_config();
        let generator = DocxGenerator::new(config);
        
        let mut document = MarkdownDocument::new();
        document.add_element(MarkdownElement::CodeBlock {
            language: Some("rust".to_string()),
            code: "fn main() {\n    println!(\"Hello, world!\");\n}".to_string(),
        });
        
        let result = generator.generate(&document);
        assert!(result.is_ok());
        
        let docx_bytes = result.unwrap();
        assert!(!docx_bytes.is_empty());
    }

    #[test]
    fn test_horizontal_rule_generation() {
        let config = create_test_config();
        let generator = DocxGenerator::new(config);
        
        let mut document = MarkdownDocument::new();
        document.add_element(MarkdownElement::HorizontalRule);
        
        let result = generator.generate(&document);
        assert!(result.is_ok());
        
        let docx_bytes = result.unwrap();
        assert!(!docx_bytes.is_empty());
    }

    #[test]
    fn test_complex_document_generation() {
        let config = create_test_config();
        let generator = DocxGenerator::new(config);
        let document = create_test_document();
        
        let result = generator.generate(&document);
        assert!(result.is_ok());
        
        let docx_bytes = result.unwrap();
        assert!(!docx_bytes.is_empty());
        // Complex document should be larger
        assert!(docx_bytes.len() > 1000);
    }

    #[test]
    fn test_empty_document_generation() {
        let config = create_test_config();
        let generator = DocxGenerator::new(config);
        let document = MarkdownDocument::new();
        
        let result = generator.generate(&document);
        assert!(result.is_ok());
        
        let docx_bytes = result.unwrap();
        assert!(!docx_bytes.is_empty());
    }

    #[test]
    fn test_inline_code_generation() {
        let config = create_test_config();
        let generator = DocxGenerator::new(config);
        
        let mut document = MarkdownDocument::new();
        document.add_element(MarkdownElement::Paragraph {
            content: vec![
                InlineElement::Text("Use ".to_string()),
                InlineElement::Code("println!()".to_string()),
                InlineElement::Text(" to print.".to_string()),
            ],
        });
        
        let result = generator.generate(&document);
        assert!(result.is_ok());
        
        let docx_bytes = result.unwrap();
        assert!(!docx_bytes.is_empty());
    }

    #[test]
    fn test_link_generation() {
        let config = create_test_config();
        let generator = DocxGenerator::new(config);
        
        let mut document = MarkdownDocument::new();
        document.add_element(MarkdownElement::Paragraph {
            content: vec![
                InlineElement::Text("Visit ".to_string()),
                InlineElement::Link {
                    text: "Google".to_string(),
                    url: "https://google.com".to_string(),
                    title: None,
                },
                InlineElement::Text(" for search.".to_string()),
            ],
        });
        
        let result = generator.generate(&document);
        assert!(result.is_ok());
        
        let docx_bytes = result.unwrap();
        assert!(!docx_bytes.is_empty());
    }

    #[test]
    fn test_multiple_headings() {
        let config = create_test_config();
        let generator = DocxGenerator::new(config);
        
        let mut document = MarkdownDocument::new();
        for level in 1..=6 {
            document.add_element(MarkdownElement::Heading {
                level,
                text: format!("Heading Level {}", level),
            });
        }
        
        let result = generator.generate(&document);
        assert!(result.is_ok());
        
        let docx_bytes = result.unwrap();
        assert!(!docx_bytes.is_empty());
    }

    #[test]
    fn test_config_application() {
        let mut config = create_test_config();
        config.document.default_font.size = 16.0;
        config.document.default_font.family = "Arial".to_string();
        
        let generator = DocxGenerator::new(config.clone());
        
        // Verify config is stored
        assert_eq!(generator.config.document.default_font.size, 16.0);
        assert_eq!(generator.config.document.default_font.family, "Arial");
    }
}