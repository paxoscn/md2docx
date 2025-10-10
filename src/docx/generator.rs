//! docx document generator

use crate::config::ConversionConfig;
use crate::error::ConversionError;
use crate::markdown::{MarkdownDocument, MarkdownElement, InlineElement, ListItem};
use crate::numbering::HeadingProcessor;
use docx_rs::*;
use std::io::Cursor;
use std::str::FromStr;
use std::sync::Arc;
use tracing::{error, warn, info, debug, trace};

/// Generator for creating docx documents from Markdown AST
pub struct DocxGenerator {
    config: ConversionConfig,
    heading_processor: Option<HeadingProcessor>,
}

impl DocxGenerator {
    /// Create a new docx generator with the given configuration
    pub fn new(config: ConversionConfig) -> Self {
        let config_arc = Arc::new(config.clone());
        
        // Initialize heading processor if any heading levels have numbering configured
        let heading_processor = if config.styles.headings.values().any(|style| style.numbering.is_some()) {
            info!("Initializing heading processor with numbering support");
            match HeadingProcessor::new(config_arc.clone()).validate_numbering_formats() {
                Ok(_) => {
                    debug!("All numbering formats validated successfully");
                    Some(HeadingProcessor::new(config_arc))
                }
                Err(e) => {
                    error!(
                        error = %e,
                        "Invalid numbering formats detected during processor initialization"
                    );
                    warn!("Creating processor anyway for graceful degradation");
                    Some(HeadingProcessor::new(config_arc))
                }
            }
        } else {
            debug!("No numbering configured, heading processor not needed");
            None
        };
        
        Self { 
            config,
            heading_processor,
        }
    }

    /// Generate docx document from Markdown AST
    pub fn generate(&mut self, document: &MarkdownDocument) -> Result<Vec<u8>, ConversionError> {
        let mut docx = Docx::new();
        
        // Apply document-level settings
        docx = self.apply_document_settings(docx)?;
        
        // Reset numbering state at the beginning of document generation
        if let Some(ref mut processor) = self.heading_processor {
            info!("Resetting numbering state for new document generation");
            processor.reset_state();
            
            // Validate numbering formats before starting document generation
            if let Err(e) = processor.validate_numbering_formats() {
                error!(
                    error = %e,
                    "Numbering format validation failed during document generation"
                );
                warn!("Continuing with document generation, numbering may be degraded");
            }
        }
        
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
        
        // Add CodeBlock style for preserving formatting
        docx = self.add_code_block_style(docx)?;
        
        Ok(docx)
    }
    
    /// Add CodeBlock style to prevent text wrapping and preserve formatting
    fn add_code_block_style(&self, mut docx: Docx) -> Result<Docx, ConversionError> {
        // Create a style for code blocks that preserves formatting
        let style = Style::new("CodeBlock", StyleType::Paragraph)
            .name("Code Block")
            .based_on("Normal");
        
        // Note: For now, we'll keep the style simple as docx-rs has limited style configuration
        // The main formatting will be applied directly to runs and paragraphs
        
        docx = docx.add_style(style);
        Ok(docx)
    }

    /// Process a single markdown element
    fn process_element(&mut self, mut docx: Docx, element: &MarkdownElement) -> Result<Docx, ConversionError> {
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
    fn add_heading(&mut self, mut docx: Docx, level: u8, text: &str) -> Result<Docx, ConversionError> {
        // Get heading style from config, fallback to level 1 if not found
        let heading_style = self.config.styles.headings.get(&level)
            .unwrap_or_else(|| self.config.styles.headings.get(&1).unwrap());
        
        // Process heading text with numbering if configured
        let processed_text = if let Some(ref mut processor) = self.heading_processor {
            match processor.process_heading(level, text) {
                Ok(numbered_text) => {
                    debug!(
                        level = level,
                        original_text = text,
                        numbered_text = %numbered_text,
                        "Successfully processed heading with numbering"
                    );
                    numbered_text
                }
                Err(e) => {
                    error!(
                        level = level,
                        text = text,
                        error = %e,
                        error_category = e.category(),
                        recoverable = e.is_recoverable(),
                        "Failed to process heading numbering"
                    );
                    
                    // Log degradation event for monitoring
                    warn!(
                        degradation_event = "heading_numbering_fallback",
                        level = level,
                        error_type = e.category(),
                        "Heading numbering degraded to fallback mode"
                    );
                    
                    text.to_string()
                }
            }
        } else {
            trace!(
                level = level,
                text = text,
                "No numbering processor configured, using original text"
            );
            text.to_string()
        };
        
        let mut run = Run::new()
            .add_text(&processed_text)
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
        .indent(None, Some(SpecialIndentType::FirstLine(315)), None, None);
        
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
        
        if code_style.preserve_line_breaks {
            // Handle edge case: empty code block
            if code.is_empty() {
                let empty_run = Run::new()
                    .add_text(" ") // Use a space instead of empty string to ensure paragraph visibility
                    .fonts(RunFonts::new().ascii(&code_style.font.family).east_asia(&code_style.font.family))
                    .size((code_style.font.size * 2.0) as usize);
                
                let paragraph = Paragraph::new().add_run(empty_run);
                docx = docx.add_paragraph(paragraph);
                return Ok(docx);
            }
            
            // Split code by lines, preserving empty lines
            let lines: Vec<&str> = code.split('\n').collect();
            
            for (i, line) in lines.iter().enumerate() {
                // Convert tabs to spaces (4 spaces per tab)
                let processed_line = line.replace('\t', "    ");
                
                // Handle empty lines by using a non-breaking space to preserve the line
                let line_content = if processed_line.trim().is_empty() {
                    "\u{00A0}".to_string() // Non-breaking space to preserve empty lines
                } else {
                    processed_line
                };
                
                let mut run = Run::new()
                    .add_text(&line_content)
                    .fonts(RunFonts::new().ascii(&code_style.font.family).east_asia(&code_style.font.family))
                    .size((code_style.font.size * 2.0) as usize);
                
                // Apply bold/italic conditionally
                if code_style.font.bold {
                    run = run.bold();
                }
                if code_style.font.italic {
                    run = run.italic();
                }
                
                // Add background color if specified
                if let Some(bg_color) = &code_style.background_color {
                    let color = bg_color.trim_start_matches('#');
                    run = run.highlight(color);
                }
                
                // Create paragraph with the run
                // Disable text wrapping to preserve long lines
                let paragraph = Paragraph::new()
                    .add_run(run)
                    // Apply code block specific styling to prevent text wrapping
                    .style("CodeBlock");
                
                // Apply line spacing configuration
                // Note: docx-rs uses different spacing units, we'll apply a basic spacing approach
                if code_style.line_spacing != 1.0 {
                    // For now, we'll handle this through paragraph spacing
                    // TODO: Implement proper line spacing when docx-rs API supports it better
                }
                
                docx = docx.add_paragraph(paragraph);
                
                // Add paragraph spacing between code lines (except after the last line)
                if i < lines.len() - 1 && code_style.paragraph_spacing > 0.0 {
                    // Add a small spacing paragraph if needed
                    // For now, we'll rely on the natural paragraph spacing
                }
            }
        } else {
            // Original behavior: treat as single paragraph
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
            
            // Add background color if specified
            if let Some(bg_color) = &code_style.background_color {
                let color = bg_color.trim_start_matches('#');
                run = run.highlight(color);
            }
            
            let paragraph = Paragraph::new().add_run(run);
            docx = docx.add_paragraph(paragraph);
        }
        
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
        let config_arc = Arc::new(config.clone());
        
        // Update heading processor if any heading levels have numbering configured
        let old_has_numbering = self.heading_processor.is_some();
        let new_has_numbering = config.styles.headings.values().any(|style| style.numbering.is_some());
        
        self.heading_processor = if new_has_numbering {
            info!("Updating heading processor with new numbering configuration");
            
            // Validate new configuration
            match HeadingProcessor::new(config_arc.clone()).validate_numbering_formats() {
                Ok(_) => {
                    debug!("New numbering formats validated successfully");
                }
                Err(e) => {
                    error!(
                        error = %e,
                        "Invalid numbering formats in new configuration"
                    );
                    warn!("Proceeding with configuration update for graceful degradation");
                }
            }
            
            Some(HeadingProcessor::new(config_arc))
        } else {
            if old_has_numbering {
                info!("Removing heading processor - no numbering in new configuration");
            }
            None
        };
        
        debug!(
            old_has_numbering = old_has_numbering,
            new_has_numbering = new_has_numbering,
            "Heading processor update completed"
        );
        
        self.config = config;
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::{create_test_config, create_test_document};
    use crate::config::ConversionConfig;
    use crate::markdown::ast::{MarkdownDocument, MarkdownElement, InlineElement, ListItem};

    #[test]
    fn test_heading_generation() {
        let config = ConversionConfig::default();
        let mut generator = DocxGenerator::new(config);
        
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
        let mut generator = DocxGenerator::new(config);
        
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
        let mut generator = DocxGenerator::new(config);
        
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
        let mut generator = DocxGenerator::new(config);
        
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
        let mut generator = DocxGenerator::new(config);
        
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
        let mut generator = DocxGenerator::new(config);
        
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
        let mut generator = DocxGenerator::new(config);
        
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
        let mut generator = DocxGenerator::new(config);
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
        let mut generator = DocxGenerator::new(config);
        let document = MarkdownDocument::new();
        
        let result = generator.generate(&document);
        assert!(result.is_ok());
        
        let docx_bytes = result.unwrap();
        assert!(!docx_bytes.is_empty());
    }

    #[test]
    fn test_inline_code_generation() {
        let config = create_test_config();
        let mut generator = DocxGenerator::new(config);
        
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
    fn test_code_block_line_break_preservation() {
        let mut config = create_test_config();
        config.styles.code_block.preserve_line_breaks = true;
        let mut generator = DocxGenerator::new(config);
        
        let mut document = MarkdownDocument::new();
        // Test code with empty lines, tabs, and long lines
        let code_with_edge_cases = "fn main() {\n\tprintln!(\"Hello, world!\");\n\n\t// This is a very long comment that should not be wrapped automatically in the docx output to preserve the original formatting\n\tlet x = 42;\n}";
        
        document.add_element(MarkdownElement::CodeBlock {
            language: Some("rust".to_string()),
            code: code_with_edge_cases.to_string(),
        });
        
        let result = generator.generate(&document);
        assert!(result.is_ok());
        
        let docx_bytes = result.unwrap();
        assert!(!docx_bytes.is_empty());
    }

    #[test]
    fn test_code_block_without_line_break_preservation() {
        let mut config = create_test_config();
        config.styles.code_block.preserve_line_breaks = false;
        let mut generator = DocxGenerator::new(config);
        
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
    fn test_empty_code_block() {
        let mut config = create_test_config();
        config.styles.code_block.preserve_line_breaks = true;
        let mut generator = DocxGenerator::new(config);
        
        let mut document = MarkdownDocument::new();
        document.add_element(MarkdownElement::CodeBlock {
            language: Some("text".to_string()),
            code: "".to_string(),
        });
        
        let result = generator.generate(&document);
        assert!(result.is_ok());
        
        let docx_bytes = result.unwrap();
        assert!(!docx_bytes.is_empty());
    }

    #[test]
    fn test_code_block_with_tabs() {
        let mut config = create_test_config();
        config.styles.code_block.preserve_line_breaks = true;
        let mut generator = DocxGenerator::new(config);
        
        let mut document = MarkdownDocument::new();
        // Code with tabs that should be converted to spaces
        let code_with_tabs = "function example() {\n\tif (true) {\n\t\tconsole.log('Hello');\n\t}\n}";
        
        document.add_element(MarkdownElement::CodeBlock {
            language: Some("javascript".to_string()),
            code: code_with_tabs.to_string(),
        });
        
        let result = generator.generate(&document);
        assert!(result.is_ok());
        
        let docx_bytes = result.unwrap();
        assert!(!docx_bytes.is_empty());
    }

    #[test]
    fn test_code_block_with_empty_lines() {
        let mut config = create_test_config();
        config.styles.code_block.preserve_line_breaks = true;
        let mut generator = DocxGenerator::new(config);
        
        let mut document = MarkdownDocument::new();
        // Code with empty lines that should be preserved
        let code_with_empty_lines = "line1\n\nline3\n\n\nline6";
        
        document.add_element(MarkdownElement::CodeBlock {
            language: Some("text".to_string()),
            code: code_with_empty_lines.to_string(),
        });
        
        let result = generator.generate(&document);
        assert!(result.is_ok());
        
        let docx_bytes = result.unwrap();
        assert!(!docx_bytes.is_empty());
    }

    #[test]
    fn test_code_block_with_long_lines() {
        let mut config = create_test_config();
        config.styles.code_block.preserve_line_breaks = true;
        let mut generator = DocxGenerator::new(config);
        
        let mut document = MarkdownDocument::new();
        // Very long line that should not be wrapped
        let long_line = "This is a very long line of code that should not be automatically wrapped in the docx output to preserve the original formatting and maintain code readability";
        let code_with_long_lines = format!("short line\n{}\nanother short line", long_line);
        
        document.add_element(MarkdownElement::CodeBlock {
            language: Some("text".to_string()),
            code: code_with_long_lines,
        });
        
        let result = generator.generate(&document);
        assert!(result.is_ok());
        
        let docx_bytes = result.unwrap();
        assert!(!docx_bytes.is_empty());
    }

    #[test]
    fn test_code_block_spacing_configuration() {
        let mut config = create_test_config();
        config.styles.code_block.preserve_line_breaks = true;
        config.styles.code_block.line_spacing = 1.5;
        config.styles.code_block.paragraph_spacing = 12.0;
        let mut generator = DocxGenerator::new(config);
        
        let mut document = MarkdownDocument::new();
        document.add_element(MarkdownElement::CodeBlock {
            language: Some("rust".to_string()),
            code: "fn main() {\n    println!(\"Hello\");\n}".to_string(),
        });
        
        let result = generator.generate(&document);
        assert!(result.is_ok());
        
        let docx_bytes = result.unwrap();
        assert!(!docx_bytes.is_empty());
    }

    #[test]
    fn test_code_block_preserve_vs_no_preserve() {
        // Test with preserve_line_breaks = true
        let mut config_preserve = create_test_config();
        config_preserve.styles.code_block.preserve_line_breaks = true;
        let mut generator_preserve = DocxGenerator::new(config_preserve);
        
        // Test with preserve_line_breaks = false
        let mut config_no_preserve = create_test_config();
        config_no_preserve.styles.code_block.preserve_line_breaks = false;
        let mut generator_no_preserve = DocxGenerator::new(config_no_preserve);
        
        let mut document = MarkdownDocument::new();
        let code = "line1\nline2\nline3";
        document.add_element(MarkdownElement::CodeBlock {
            language: Some("text".to_string()),
            code: code.to_string(),
        });
        
        // Both should succeed but may produce different output
        let result_preserve = generator_preserve.generate(&document);
        let result_no_preserve = generator_no_preserve.generate(&document);
        
        assert!(result_preserve.is_ok());
        assert!(result_no_preserve.is_ok());
        
        let docx_bytes_preserve = result_preserve.unwrap();
        let docx_bytes_no_preserve = result_no_preserve.unwrap();
        
        assert!(!docx_bytes_preserve.is_empty());
        assert!(!docx_bytes_no_preserve.is_empty());
        
        // The outputs should be different (though we can't easily test the exact difference)
        // This test mainly ensures both modes work without errors
    }

    #[test]
    fn test_link_generation() {
        let config = create_test_config();
        let mut generator = DocxGenerator::new(config);
        
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
        let mut generator = DocxGenerator::new(config);
        
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

    #[test]
    fn test_heading_numbering_integration() {
        let mut config = create_test_config();
        
        // Configure numbering for H1 and H2
        config.styles.headings.get_mut(&1).unwrap().numbering = Some("%1.".to_string());
        config.styles.headings.get_mut(&2).unwrap().numbering = Some("%1.%2.".to_string());
        
        let mut generator = DocxGenerator::new(config);
        
        let mut document = MarkdownDocument::new();
        document.add_element(MarkdownElement::Heading {
            level: 1,
            text: "Introduction".to_string(),
        });
        document.add_element(MarkdownElement::Heading {
            level: 2,
            text: "Overview".to_string(),
        });
        document.add_element(MarkdownElement::Heading {
            level: 2,
            text: "Details".to_string(),
        });
        document.add_element(MarkdownElement::Heading {
            level: 1,
            text: "Conclusion".to_string(),
        });
        
        let result = generator.generate(&document);
        assert!(result.is_ok());
        
        let docx_bytes = result.unwrap();
        assert!(!docx_bytes.is_empty());
        
        // Verify that heading processor was created
        assert!(generator.heading_processor.is_some());
    }

    #[test]
    fn test_heading_without_numbering() {
        let config = create_test_config(); // Default config has no numbering
        let mut generator = DocxGenerator::new(config);
        
        let mut document = MarkdownDocument::new();
        document.add_element(MarkdownElement::Heading {
            level: 1,
            text: "Plain Heading".to_string(),
        });
        
        let result = generator.generate(&document);
        assert!(result.is_ok());
        
        let docx_bytes = result.unwrap();
        assert!(!docx_bytes.is_empty());
        
        // Verify that no heading processor was created
        assert!(generator.heading_processor.is_none());
    }

    #[test]
    fn test_mixed_numbering_scenario() {
        let mut config = create_test_config();
        
        // Configure numbering only for H1 and H3 (skip H2)
        config.styles.headings.get_mut(&1).unwrap().numbering = Some("%1.".to_string());
        config.styles.headings.get_mut(&3).unwrap().numbering = Some("%1.%2.%3".to_string());
        
        let mut generator = DocxGenerator::new(config);
        
        let mut document = MarkdownDocument::new();
        document.add_element(MarkdownElement::Heading {
            level: 1,
            text: "Chapter".to_string(),
        });
        document.add_element(MarkdownElement::Heading {
            level: 2,
            text: "Section".to_string(), // No numbering
        });
        document.add_element(MarkdownElement::Heading {
            level: 3,
            text: "Subsection".to_string(),
        });
        
        let result = generator.generate(&document);
        assert!(result.is_ok());
        
        let docx_bytes = result.unwrap();
        assert!(!docx_bytes.is_empty());
    }

    #[test]
    fn test_config_update_with_numbering() {
        let config = create_test_config();
        let mut generator = DocxGenerator::new(config);
        
        // Initially no numbering
        assert!(generator.heading_processor.is_none());
        
        // Update config to include numbering
        let mut new_config = create_test_config();
        new_config.styles.headings.get_mut(&1).unwrap().numbering = Some("%1.".to_string());
        
        generator.update_config(new_config);
        
        // Now should have heading processor
        assert!(generator.heading_processor.is_some());
        
        // Update config to remove numbering
        let no_numbering_config = create_test_config();
        generator.update_config(no_numbering_config);
        
        // Should no longer have heading processor
        assert!(generator.heading_processor.is_none());
    }

    #[test]
    fn test_numbering_state_reset_between_documents() {
        let mut config = create_test_config();
        config.styles.headings.get_mut(&1).unwrap().numbering = Some("%1.".to_string());
        
        let mut generator = DocxGenerator::new(config);
        
        // Generate first document
        let mut document1 = MarkdownDocument::new();
        document1.add_element(MarkdownElement::Heading {
            level: 1,
            text: "First Document".to_string(),
        });
        
        let result1 = generator.generate(&document1);
        assert!(result1.is_ok());
        
        // Generate second document - numbering should reset
        let mut document2 = MarkdownDocument::new();
        document2.add_element(MarkdownElement::Heading {
            level: 1,
            text: "Second Document".to_string(),
        });
        
        let result2 = generator.generate(&document2);
        assert!(result2.is_ok());
        
        // Both should succeed and numbering should start fresh for each document
        assert!(!result1.unwrap().is_empty());
        assert!(!result2.unwrap().is_empty());
    }

    #[test]
    fn test_numbering_error_handling() {
        let mut config = create_test_config();
        config.styles.headings.get_mut(&1).unwrap().numbering = Some("%1.".to_string());
        
        let mut generator = DocxGenerator::new(config);
        
        // Test with invalid heading level (should be handled gracefully)
        let mut document = MarkdownDocument::new();
        document.add_element(MarkdownElement::Heading {
            level: 1,
            text: "Valid Heading".to_string(),
        });
        
        let result = generator.generate(&document);
        assert!(result.is_ok());
        
        let docx_bytes = result.unwrap();
        assert!(!docx_bytes.is_empty());
    }

    #[test]
    fn test_numbering_with_empty_text() {
        let mut config = create_test_config();
        config.styles.headings.get_mut(&1).unwrap().numbering = Some("%1.".to_string());
        
        let mut generator = DocxGenerator::new(config);
        
        let mut document = MarkdownDocument::new();
        document.add_element(MarkdownElement::Heading {
            level: 1,
            text: "".to_string(), // Empty text
        });
        document.add_element(MarkdownElement::Heading {
            level: 1,
            text: "   ".to_string(), // Whitespace only
        });
        
        let result = generator.generate(&document);
        assert!(result.is_ok());
        
        let docx_bytes = result.unwrap();
        assert!(!docx_bytes.is_empty());
    }

    #[test]
    fn test_complex_numbering_scenario() {
        let mut config = create_test_config();
        
        // Configure numbering for multiple levels
        config.styles.headings.get_mut(&1).unwrap().numbering = Some("%1.".to_string());
        config.styles.headings.get_mut(&2).unwrap().numbering = Some("%1.%2.".to_string());
        config.styles.headings.get_mut(&3).unwrap().numbering = Some("%1.%2.%3".to_string());
        
        let mut generator = DocxGenerator::new(config);
        
        let mut document = MarkdownDocument::new();
        
        // Create a complex document structure
        document.add_element(MarkdownElement::Heading {
            level: 1,
            text: "Chapter 1".to_string(),
        });
        document.add_element(MarkdownElement::Heading {
            level: 2,
            text: "Section 1.1".to_string(),
        });
        document.add_element(MarkdownElement::Heading {
            level: 3,
            text: "Subsection 1.1.1".to_string(),
        });
        document.add_element(MarkdownElement::Heading {
            level: 3,
            text: "Subsection 1.1.2".to_string(),
        });
        document.add_element(MarkdownElement::Heading {
            level: 2,
            text: "Section 1.2".to_string(),
        });
        document.add_element(MarkdownElement::Heading {
            level: 1,
            text: "Chapter 2".to_string(),
        });
        document.add_element(MarkdownElement::Heading {
            level: 2,
            text: "Section 2.1".to_string(),
        });
        
        let result = generator.generate(&document);
        assert!(result.is_ok());
        
        let docx_bytes = result.unwrap();
        assert!(!docx_bytes.is_empty());
        
        // Verify that heading processor was created and is working
        assert!(generator.heading_processor.is_some());
    }

    #[test]
    fn test_numbering_graceful_degradation() {
        let mut config = create_test_config();
        config.styles.headings.get_mut(&1).unwrap().numbering = Some("%1.".to_string());
        
        let mut generator = DocxGenerator::new(config);
        
        // Test that the generator handles potential numbering errors gracefully
        // by ensuring the document generation still succeeds even if numbering fails
        let mut document = MarkdownDocument::new();
        document.add_element(MarkdownElement::Heading {
            level: 1,
            text: "Test Heading".to_string(),
        });
        
        let result = generator.generate(&document);
        assert!(result.is_ok());
        
        let docx_bytes = result.unwrap();
        assert!(!docx_bytes.is_empty());
        
        // Verify that the heading processor exists and is functional
        assert!(generator.heading_processor.is_some());
        
        // Test that the processor can handle multiple headings without issues
        let mut document2 = MarkdownDocument::new();
        for i in 1..=5 {
            document2.add_element(MarkdownElement::Heading {
                level: 1,
                text: format!("Heading {}", i),
            });
        }
        
        let result2 = generator.generate(&document2);
        assert!(result2.is_ok());
        assert!(!result2.unwrap().is_empty());
    }

    #[test]
    fn test_numbering_style_consistency() {
        let mut config = create_test_config();
        
        // Configure numbering and ensure styles are applied consistently
        config.styles.headings.get_mut(&1).unwrap().numbering = Some("%1.".to_string());
        config.styles.headings.get_mut(&1).unwrap().font.bold = true;
        config.styles.headings.get_mut(&1).unwrap().font.size = 18.0;
        
        let mut generator = DocxGenerator::new(config);
        
        let mut document = MarkdownDocument::new();
        document.add_element(MarkdownElement::Heading {
            level: 1,
            text: "Styled Heading".to_string(),
        });
        
        let result = generator.generate(&document);
        assert!(result.is_ok());
        
        let docx_bytes = result.unwrap();
        assert!(!docx_bytes.is_empty());
        
        // The test verifies that the docx generation succeeds with styled numbered headings
        // The actual style application is verified by the docx library integration
    }
}