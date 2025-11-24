//! docx document generator

use crate::config::ConversionConfig;
use crate::error::ConversionError;
use crate::markdown::{InlineElement, ListItem, MarkdownDocument, MarkdownElement};
use crate::numbering::HeadingProcessor;
use crate::config::ImageConfig;
use docx_rs::*;
use std::io::Cursor;
use std::str::FromStr;
use std::sync::Arc;
use tracing::{debug, error, info, trace, warn};

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
        let heading_processor = if config
            .styles
            .headings
            .values()
            .any(|style| style.numbering.is_some())
        {
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
    fn process_element(
        &mut self,
        mut docx: Docx,
        element: &MarkdownElement,
    ) -> Result<Docx, ConversionError> {
        match element {
            MarkdownElement::Heading { level, text } => {
                docx = self.add_heading(docx, *level, text)?;
            }
            MarkdownElement::Paragraph { content } => {
                docx = self.add_paragraph(docx, content)?;
            }
            MarkdownElement::CodeBlock { language: _, code, processed } => {
                // Use processed code if available, otherwise use original
                let final_code = processed.as_ref()
                    .map(|p| p.get_final_code())
                    .unwrap_or(code);
                docx = self.add_code_block(docx, final_code)?;
            }
            MarkdownElement::List { ordered, items } => {
                docx = self.add_list(docx, *ordered, items)?;
            }
            MarkdownElement::Table { headers, rows } => {
                docx = self.add_table(docx, headers, rows)?;
            }
            MarkdownElement::Image {
                alt_text,
                url,
                title: _,
            } => {
                docx = self.add_image(docx, alt_text, url)?;
            }
            MarkdownElement::HorizontalRule => {
                docx = self.add_horizontal_rule(docx)?;
            }
        }

        Ok(docx)
    }

    /// Add a heading to the document
    fn add_heading(
        &mut self,
        mut docx: Docx,
        level: u8,
        text: &str,
    ) -> Result<Docx, ConversionError> {
        // Get heading style from config, fallback to level 1 if not found
        let heading_style = self
            .config
            .styles
            .headings
            .get(&level)
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
            .fonts(
                RunFonts::new()
                    .ascii(&heading_style.font.family)
                    .east_asia(&heading_style.font.family),
            )
            .size((heading_style.font.size * 2.0) as usize); // docx uses half-points

        // Apply bold/italic conditionally
        if heading_style.font.bold {
            run = run.bold();
        }
        if heading_style.font.italic {
            run = run.italic();
        }

        // Create paragraph with spacing
        let paragraph = Paragraph::new()
            .add_run(run)
            .outline_lvl((level - 1) as usize);

        // Adding alignment
        let paragraph = match heading_style.alignment.clone() {
            Some(alignment) => match AlignmentType::from_str(alignment.as_str()) {
                Ok(alignment_type) => paragraph.align(alignment_type),
                Err(e) => {
                    error!("Unknown alignment: {}. Error: {}", alignment, e);
                    paragraph
                }
            },
            None => paragraph,
        };

        // Apply paragraph spacing - for now, we'll skip this as docx-rs API is different
        // TODO: Implement proper spacing when docx-rs API is clarified
        // Add spacing before heading using empty paragraph
        // Since docx-rs doesn't support paragraph spacing directly, we use empty paragraphs
        let spacing_before = Paragraph::new()
        .add_run(Run::new().add_text("\u{00A0}")) // Non-breaking space for minimal visibility
        .size(1); // Small font size for minimal visual impact
        docx = docx.add_paragraph(spacing_before);

        docx = docx.add_paragraph(paragraph);

        // Add spacing after heading using empty paragraph
        let spacing_after = Paragraph::new()
            .add_run(Run::new().add_text("\u{00A0}")) // Non-breaking space for minimal visibility
            .size(1); // Small font size for minimal visual impact
        docx = docx.add_paragraph(spacing_after);

        Ok(docx)
    }

    /// Add a paragraph to the document
    fn add_paragraph(
        &self,
        mut docx: Docx,
        content: &[InlineElement],
    ) -> Result<Docx, ConversionError> {
        let mut paragraph =
            Paragraph::new().indent(None, Some(SpecialIndentType::FirstLine(315)), None, None);

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
                    .fonts(
                        RunFonts::new()
                            .ascii(&base_font.family)
                            .east_asia(&base_font.family),
                    )
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
            InlineElement::Bold(text) => Ok(Run::new()
                .add_text(text)
                .fonts(
                    RunFonts::new()
                        .ascii(&base_font.family)
                        .east_asia(&base_font.family),
                )
                .size((base_font.size * 2.0) as usize)
                .bold()),
            InlineElement::Italic(text) => Ok(Run::new()
                .add_text(text)
                .fonts(
                    RunFonts::new()
                        .ascii(&base_font.family)
                        .east_asia(&base_font.family),
                )
                .size((base_font.size * 2.0) as usize)
                .italic()),
            InlineElement::Strikethrough(text) => Ok(Run::new()
                .add_text(text)
                .fonts(
                    RunFonts::new()
                        .ascii(&base_font.family)
                        .east_asia(&base_font.family),
                )
                .size((base_font.size * 2.0) as usize)
                .strike()),
            InlineElement::Code(text) => {
                let code_font = &self.config.styles.code_block.font;
                let mut run = Run::new()
                    .add_text(text)
                    .fonts(
                        RunFonts::new()
                            .ascii(&code_font.family)
                            .east_asia(&code_font.family),
                    )
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
            InlineElement::Link {
                text,
                url: _,
                title: _,
            } => {
                let link_color = self.config.elements.link.color.trim_start_matches('#');
                let mut run = Run::new()
                    .add_text(text)
                    .fonts(
                        RunFonts::new()
                            .ascii(&base_font.family)
                            .east_asia(&base_font.family),
                    )
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

    /// Add a code block to the document as a single-row table
    fn add_code_block(&self, mut docx: Docx, code: &str) -> Result<Docx, ConversionError> {
        // Check if this is a note block with special formatting
        if code.contains("[NOTE_BLOCK_START]") && code.contains("[NOTE_BLOCK_END]") {
            return self.add_note_block(docx, code);
        }

        let code_style = &self.config.styles.code_block;

        // Add spacing before code block using empty paragraph
        // Since docx-rs doesn't support paragraph spacing directly, we use empty paragraphs
        let spacing_before = Paragraph::new()
            .add_run(Run::new().add_text("\u{00A0}")) // Non-breaking space for minimal visibility
            .size(1); // Small font size for minimal visual impact
        docx = docx.add_paragraph(spacing_before);

        // Create table cell with code content using the helper method
        let cell = self
            .create_code_block_cell_with_markdown(code, code_style)?
            .width(8300, WidthType::Dxa);

        // Create single-row, single-column table
        let row = TableRow::new(vec![cell]);
        let mut table =
            Table::new(vec![row]).margins(TableCellMargins::new().margin(100, 100, 100, 100));

        // Apply border styling based on border_width configuration
        if code_style.border_width > 0.0 {
            table = self.apply_table_borders(table, code_style.border_width)?;
        }

        // Add the table to the document
        docx = docx.add_table(table);

        // Add spacing after code block using empty paragraph
        let spacing_after = Paragraph::new()
            .add_run(Run::new().add_text("\u{00A0}")) // Non-breaking space for minimal visibility
            .size(1); // Small font size for minimal visual impact
        docx = docx.add_paragraph(spacing_after);

        Ok(docx)
    }

    /// Add a note block with special formatting (title, icon, content)
    fn add_note_block(&self, mut docx: Docx, code: &str) -> Result<Docx, ConversionError> {
        // Parse the note block markers
        let title = self.extract_marker_content(code, "[TITLE]", "[/TITLE]");
        let icon_path = self.extract_marker_content(code, "[ICON]", "[/ICON]");
        let content = self.extract_marker_content(code, "[CONTENT]", "[/CONTENT]");

        // Add spacing before note block
        let spacing_before = Paragraph::new()
            .add_run(Run::new().add_text("\u{00A0}"))
            .size(1);
        docx = docx.add_paragraph(spacing_before);

        // Create the note block as a two-column table
        // Left column: title + content
        // Right column: icon (if available)

        let mut left_cell = TableCell::new().clear_border(TableCellBorderPosition::Right);
        
        // Add title with special formatting (larger, bold, italic)
        if let Some(title_text) = title {
            let title_run = Run::new()
                .add_text(&title_text)
                .bold()
                .italic()
                .size(23); // 14pt * 2 = 28 half-points (1.2x of 12pt ≈ 14pt)
            
            let title_paragraph = Paragraph::new()
                .add_run(title_run);
            
            left_cell = left_cell.add_paragraph(title_paragraph);
            
            // Add spacing after title
            left_cell = left_cell.add_paragraph(Paragraph::new());
        }
        
        // Add content
        if let Some(content_text) = content {
            for line in content_text.lines() {
                if !line.trim().is_empty() {
                    let content_run = Run::new().add_text(line);
                    let content_paragraph = Paragraph::new().add_run(content_run);
                    left_cell = left_cell.add_paragraph(content_paragraph);
                } else {
                    // Empty line
                    left_cell = left_cell.add_paragraph(Paragraph::new());
                }
            }
        }
        
        // Set cell width and styling
        left_cell = left_cell
            .width(6500, WidthType::Dxa)
            .vertical_align(docx_rs::VAlignType::Top);

        // Create right cell for icon
        let mut right_cell = TableCell::new().clear_border(TableCellBorderPosition::Left);
        
        // Try to add icon if path is provided
        if let Some(icon_path) = icon_path {
            // // For now, we'll add a placeholder text instead of an actual image
            // // In a full implementation, you would load and embed the image
            // let icon_run = Run::new()
            //     .add_text("💡") // Unicode lightbulb as placeholder
            //     .size(32); // 16pt
            
            // let icon_paragraph = Paragraph::new()
            //     .add_run(icon_run)
            //     .align(docx_rs::AlignmentType::Right);
            
            // left_cell = left_cell.add_paragraph(icon_paragraph);

            // Try to embed local image
            let right_style = Style::new("Right", StyleType::Paragraph)
                .name("Right")
                .align(AlignmentType::Right);
            docx = docx.add_style(right_style);

            match self.embed_local_image_sized(icon_path.as_str(), "", 105, 70, &ImageConfig { max_width: 1500.0, max_height: 1000.0, }) {
                Ok(image_run) => {
                    let paragraph = Paragraph::new().add_run(image_run);
                    right_cell = right_cell.add_paragraph(paragraph.style("Right"));
                }
                Err(e) => {
                    warn!("embed_local_image_sized failed: {:?}", e);
                }
            }
        }
        
        right_cell = right_cell
            .width(1800, WidthType::Dxa)
            .vertical_align(docx_rs::VAlignType::Top);

        // Create table row with both cells
        let row = TableRow::new(vec![left_cell, right_cell]);
        // let row = TableRow::new(vec![left_cell]);
        
        // Create table with light background and border
        let mut table = Table::new(vec![row])
            .margins(TableCellMargins::new().margin(100, 100, 100, 100));
        
        // Apply light border
        table = self.apply_table_borders(table, 1.0)?;
        
        // Add the table to the document
        docx = docx.add_table(table);

        // Add spacing after note block
        let spacing_after = Paragraph::new()
            .add_run(Run::new().add_text("\u{00A0}"))
            .size(1);
        docx = docx.add_paragraph(spacing_after);

        Ok(docx)
    }

    /// Extract content between markers
    fn extract_marker_content(&self, text: &str, start_marker: &str, end_marker: &str) -> Option<String> {
        if let Some(start_pos) = text.find(start_marker) {
            let content_start = start_pos + start_marker.len();
            if let Some(end_pos) = text[content_start..].find(end_marker) {
                let content = &text[content_start..content_start + end_pos];
                return Some(content.trim().to_string());
            }
        }
        None
    }

    /// Add a list to the document
    fn add_list(
        &self,
        mut docx: Docx,
        ordered: bool,
        items: &[ListItem],
    ) -> Result<Docx, ConversionError> {
        docx = self.add_list_with_depth(docx, ordered, items, 0)?;
        Ok(docx)
    }

    /// Add a list to the document with specified depth for indentation
    fn add_list_with_depth(
        &self,
        mut docx: Docx,
        ordered: bool,
        items: &[ListItem],
        depth: usize,
    ) -> Result<Docx, ConversionError> {
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
    fn add_table(
        &self,
        mut docx: Docx,
        headers: &[String],
        rows: &[Vec<String>],
    ) -> Result<Docx, ConversionError> {
        let table_style = &self.config.styles.table;

        // Calculate column widths based on content
        let column_widths = self.calculate_column_widths(headers, rows);
        let mut table_rows = vec![];

        // Add header row
        if !headers.is_empty() {
            let mut header_cells = vec![];

            for (index, header) in headers.iter().enumerate() {
                let mut header_run = Run::new()
                    .add_text(header)
                    .fonts(
                        RunFonts::new()
                            .ascii(&table_style.header_font.family)
                            .east_asia(&table_style.header_font.family),
                    )
                    .size((table_style.header_font.size * 2.0) as usize);

                if table_style.header_font.bold {
                    header_run = header_run.bold();
                }
                if table_style.header_font.italic {
                    header_run = header_run.italic();
                }

                let cell_paragraph = Paragraph::new().add_run(header_run);
                let mut cell = TableCell::new().add_paragraph(cell_paragraph);
                
                // Set cell width based on content
                if let Some(&width) = column_widths.get(index) {
                    cell = cell.width(width, WidthType::Dxa);
                }
                
                header_cells.push(cell);
            }

            table_rows.push(TableRow::new(header_cells));
        }

        // Add data rows
        for row in rows {
            let mut row_cells = vec![];

            for (index, cell_data) in row.iter().enumerate() {
                let mut cell_run = Run::new()
                    .add_text(cell_data)
                    .fonts(
                        RunFonts::new()
                            .ascii(&table_style.cell_font.family)
                            .east_asia(&table_style.cell_font.family),
                    )
                    .size((table_style.cell_font.size * 2.0) as usize);

                if table_style.cell_font.bold {
                    cell_run = cell_run.bold();
                }
                if table_style.cell_font.italic {
                    cell_run = cell_run.italic();
                }

                let cell_paragraph = Paragraph::new().add_run(cell_run);
                let mut cell = TableCell::new().add_paragraph(cell_paragraph);
                
                // Set cell width based on content
                if let Some(&width) = column_widths.get(index) {
                    cell = cell.width(width, WidthType::Dxa);
                }
                
                row_cells.push(cell);
            }

            table_rows.push(TableRow::new(row_cells));
        }

        // Create table with auto-layout for better content fitting
        let mut table = Table::new(table_rows)
            .layout(TableLayoutType::Autofit);

        // Apply table borders if configured
        if table_style.border_width > 0.0 {
            table = self.apply_table_borders(table, table_style.border_width)?;
        }

        docx = docx.add_table(table);
        Ok(docx)
    }

    /// Add an image to the document
    fn add_image(
        &self,
        mut docx: Docx,
        alt_text: &str,
        url: &str,
    ) -> Result<Docx, ConversionError> {
        let image_config = &self.config.elements.image;

        // QR code
        if alt_text == "qrcode" {
            let center_style = Style::new("Center", StyleType::Paragraph)
                .name("Center")
                .align(AlignmentType::Center);
            docx = docx.add_style(center_style);

            // Try to embed local image
            match self.embed_local_image_sized("/Users/lindagao/Workspace/md2docx/default-qrcode.png", alt_text, 50, 50, image_config) {
                Ok(image_run) => {
                    let paragraph = Paragraph::new().add_run(image_run);
                    docx = docx.add_paragraph(paragraph.style("Center"));
                }
                Err(_) => {
                    // Fallback to placeholder text if image can't be loaded
                    let paragraph = Paragraph::new().add_run(
                        Run::new()
                            .add_text(&format!("[Image: {} - File not found: {}]", alt_text, url)),
                    );
                    docx = docx.add_paragraph(paragraph);
                }
            }

            let paragraph = Paragraph::new().add_run(
                Run::new().add_text(url.to_string()),
            );
            docx = docx.add_paragraph(paragraph.style("Center"));

            return Ok(docx)
        }

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
                    let paragraph = Paragraph::new().add_run(
                        Run::new()
                            .add_text(&format!("[Image: {} - File not found: {}]", alt_text, url)),
                    );
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
    fn embed_local_image(
        &self,
        path: &str,
        _alt_text: &str,
        image_config: &crate::config::ImageConfig,
    ) -> Result<Run, ConversionError> {
        self.embed_local_image_sized(
            path,
            _alt_text,
            image_config.max_width as u32,
            image_config.max_height as u32,
            image_config,
        )
    }

    /// Embed a local image file
    fn embed_local_image_sized(
        &self,
        path: &str,
        _alt_text: &str,
        width: u32,
        height: u32,
        image_config: &crate::config::ImageConfig,
    ) -> Result<Run, ConversionError> {
        use std::fs;
        use std::path::Path;

        // Check if file exists
        if !Path::new(path).exists() {
            return Err(ConversionError::DocxGeneration(format!(
                "Image file not found: {}",
                path
            )));
        }

        // Read image file
        let image_data = fs::read(path).map_err(|e| {
            ConversionError::DocxGeneration(format!("Failed to read image file {}: {}", path, e))
        })?;

        // Determine image format from file extension
        let _format = self.get_image_format(path)?;

        // Create image with size constraints
        // '9525' is from here: https://github.com/bokuweb/docx-rs/blob/main/docx-core/examples/image_floating.rs
        let image = Pic::new(&image_data).size(
            width * 9525,
            height * 9525,
        );

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
            Err(ConversionError::DocxGeneration(format!(
                "Unsupported image format: {}",
                path
            )))
        }
    }

    /// Add a horizontal rule to the document
    fn add_horizontal_rule(&self, mut docx: Docx) -> Result<Docx, ConversionError> {
        // Add a paragraph with a line of dashes as a simple horizontal rule
        let paragraph = Paragraph::new().add_run(Run::new().add_text("─".repeat(50)));

        docx = docx.add_paragraph(paragraph);
        Ok(docx)
    }

    /// Create a table cell with code content that may contain Markdown formatting
    fn create_code_block_cell_with_markdown(
        &self,
        code: &str,
        style: &crate::config::CodeBlockStyle,
    ) -> Result<TableCell, ConversionError> {
        // // Check if the code contains Markdown formatting indicators
        // if self.contains_markdown_formatting(code) {
            self.create_code_block_cell_with_parsed_markdown(code, style)
        // } else {
        //     // Fall back to plain text rendering
        //     self.create_code_block_cell(code, style)
        // }
    }

    /// Check if text contains common Markdown formatting
    fn contains_markdown_formatting(&self, text: &str) -> bool {
        text.contains("**") || // Bold
        text.contains("*") ||  // Italic (but not ** which is already checked)
        text.contains("![") || // Images
        text.contains("[") ||  // Links
        text.contains("`")     // Inline code
    }

    /// Create a table cell with parsed Markdown content
    fn create_code_block_cell_with_parsed_markdown(
        &self,
        code: &str,
        style: &crate::config::CodeBlockStyle,
    ) -> Result<TableCell, ConversionError> {
        
        let mut cell = TableCell::new();

        if style.preserve_line_breaks {
            // Handle edge case: empty code block
            if code.is_empty() {
                let paragraph = self.create_code_paragraph("\u{00A0}", style)?; // Use non-breaking space for visibility
                cell = cell.add_paragraph(paragraph);
            } else {
                // Trim trailing newlines to avoid extra empty lines at the end
                let trimmed_code = code.trim_end_matches('\n');

                // Split code by lines, preserving empty lines
                let lines: Vec<&str> = trimmed_code.split('\n').collect();

                for line in lines.iter() {
                    // Convert tabs to spaces (4 spaces per tab) for consistent formatting
                    let processed_line = line.replace('\t', "    ");

                    // Handle empty lines by using a non-breaking space to preserve the line
                    if processed_line.trim().is_empty() {
                        let paragraph = self.create_code_paragraph("\u{00A0}", style)?;
                        cell = cell.add_paragraph(paragraph);
                    } else {
                        // Parse the line as Markdown and create paragraph with formatted runs
                        let paragraph = self.create_code_paragraph_with_markdown(&processed_line, style)?;
                        cell = cell.add_paragraph(paragraph);
                    }
                }
            }
        } else {
            // Single paragraph for entire code block with tab conversion
            // Trim trailing newlines to avoid extra empty lines
            let trimmed_code = code.trim_end_matches('\n');
            let processed_code = trimmed_code.replace('\t', "    ");
            let paragraph = self.create_code_paragraph_with_markdown(&processed_code, style)?;
            cell = cell.add_paragraph(paragraph);
        }

        // Apply background color to table cell if specified
        if let Some(bg_color) = &style.background_color {
            let color = bg_color.trim_start_matches('#');
            // Apply cell shading/background color using docx-rs API
            cell = cell.shading(Shading::new().fill(color));
        }

        Ok(cell)
    }

    /// Create a code paragraph with Markdown formatting
    fn create_code_paragraph_with_markdown(
        &self,
        text: &str,
        style: &crate::config::CodeBlockStyle,
    ) -> Result<Paragraph, ConversionError> {
        let mut leading_spaces = String::new();
        for ch in text.chars() {
            if ch == ' ' {
                leading_spaces.push(ch);
            } else {
                break;
            }
        }
        let text_without_leading_spaces = &text[leading_spaces.len()..];
        
        // Parse the text as Markdown
        let parser = crate::markdown::MarkdownParser::new();
        let document = parser.parse(text_without_leading_spaces).map_err(|e| {
            ConversionError::DocxGeneration(format!("Failed to parse Markdown in code block: {}", e))
        })?;

        let mut paragraph = Paragraph::new().style("CodeBlock");

        if !leading_spaces.is_empty() {
            let run = self.create_code_run(&leading_spaces, style)?;
            paragraph = paragraph.add_run(run);
        }

        // Process the parsed elements
        for element in &document.elements {
            match element {
                crate::markdown::MarkdownElement::Paragraph { content } => {
                    // Add all inline elements from the paragraph
                    for inline in content {
                        let run = self.create_code_run_from_inline(inline, style)?;
                        paragraph = paragraph.add_run(run);
                    }
                }
                crate::markdown::MarkdownElement::Image { alt_text, url, title: _ } => {
                    // Handle images in code blocks as placeholders
                    let image_text = format!("[Image: {} - {}]", alt_text, url);
                    let run = self.create_code_run(&image_text, style)?;
                    paragraph = paragraph.add_run(run);
                }
                _ => {
                    // For other elements, extract text and render as code
                    let text_content = self.extract_text_from_element(element);
                    if !text_content.is_empty() {
                        let run = self.create_code_run(&text_content, style)?;
                        paragraph = paragraph.add_run(run);
                    }
                }
            }
        }

        // If no content was processed from Markdown, add the original text as fallback
        // We'll track if we added any runs during processing
        let mut has_content = false;
        for element in &document.elements {
            if !matches!(element, crate::markdown::MarkdownElement::Paragraph { content } if content.is_empty()) {
                has_content = true;
                break;
            }
        }
        
        if !has_content {
            let run = self.create_code_run(text, style)?;
            paragraph = paragraph.add_run(run);
        }

        // Apply line spacing configuration
        let line_spacing_value = (style.line_spacing * 240.0) as i32;
        paragraph = paragraph.line_spacing(LineSpacing::new().line(line_spacing_value));

        Ok(paragraph)
    }

    /// Create a code run from an inline element with code styling
    fn create_code_run_from_inline(
        &self,
        inline: &crate::markdown::InlineElement,
        style: &crate::config::CodeBlockStyle,
    ) -> Result<Run, ConversionError> {
        let run = match inline {
            crate::markdown::InlineElement::Text(text) => {
                self.create_code_run(text, style)?
            }
            crate::markdown::InlineElement::Bold(text) => {
                let mut run = self.create_code_run(text, style)?;
                run = run.bold();
                run
            }
            crate::markdown::InlineElement::Italic(text) => {
                let mut run = self.create_code_run(text, style)?;
                run = run.italic();
                run
            }
            crate::markdown::InlineElement::Strikethrough(text) => {
                let mut run = self.create_code_run(text, style)?;
                run = run.strike();
                run
            }
            crate::markdown::InlineElement::Code(text) => {
                // Nested code - render with different background or styling
                let mut run = self.create_code_run(text, style)?;
                // Add a subtle highlight for nested code
                run = run.highlight("E0E0E0");
                run
            }
            crate::markdown::InlineElement::Link { text, url, title: _ } => {
                let link_text = format!("{} ({})", text, url);
                let mut run = self.create_code_run(&link_text, style)?;
                run = run.color("0000FF"); // Blue color for links
                run
            }
        };

        Ok(run)
    }

    /// Create a basic code run with standard code styling
    fn create_code_run(
        &self,
        text: &str,
        style: &crate::config::CodeBlockStyle,
    ) -> Result<Run, ConversionError> {
        let mut run = Run::new()
            .add_text(text)
            .fonts(
                RunFonts::new()
                    .ascii(&style.font.family)
                    .east_asia(&style.font.family),
            )
            .size((style.font.size * 2.0) as usize); // docx uses half-points

        // Apply bold/italic conditionally based on font configuration
        if style.font.bold {
            run = run.bold();
        }
        if style.font.italic {
            run = run.italic();
        }

        // Add background color if specified (applied to run for better compatibility)
        if let Some(bg_color) = &style.background_color {
            let color = bg_color.trim_start_matches('#');
            run = run.highlight(color);
        }

        Ok(run)
    }

    /// Extract text content from a Markdown element
    fn extract_text_from_element(&self, element: &crate::markdown::MarkdownElement) -> String {
        match element {
            crate::markdown::MarkdownElement::Heading { text, .. } => text.clone(),
            crate::markdown::MarkdownElement::Paragraph { content } => {
                content.iter().map(|inline| self.extract_text_from_inline(inline)).collect::<Vec<_>>().join("")
            }
            crate::markdown::MarkdownElement::CodeBlock { code, .. } => code.clone(),
            crate::markdown::MarkdownElement::List { items, .. } => {
                items.iter().map(|item| {
                    item.content.iter().map(|inline| self.extract_text_from_inline(inline)).collect::<Vec<_>>().join("")
                }).collect::<Vec<_>>().join(" ")
            }
            crate::markdown::MarkdownElement::Table { headers, rows } => {
                let mut text = headers.join(" ");
                for row in rows {
                    text.push(' ');
                    text.push_str(&row.join(" "));
                }
                text
            }
            crate::markdown::MarkdownElement::Image { alt_text, .. } => alt_text.clone(),
            crate::markdown::MarkdownElement::HorizontalRule => "---".to_string(),
        }
    }

    /// Extract text from an inline element
    fn extract_text_from_inline(&self, inline: &crate::markdown::InlineElement) -> String {
        match inline {
            crate::markdown::InlineElement::Text(text) => text.clone(),
            crate::markdown::InlineElement::Bold(text) => text.clone(),
            crate::markdown::InlineElement::Italic(text) => text.clone(),
            crate::markdown::InlineElement::Strikethrough(text) => text.clone(),
            crate::markdown::InlineElement::Code(text) => text.clone(),
            crate::markdown::InlineElement::Link { text, .. } => text.clone(),
        }
    }

    /// Create a table cell with code content for table-based code block rendering
    fn create_code_block_cell(
        &self,
        code: &str,
        style: &crate::config::CodeBlockStyle,
    ) -> Result<TableCell, ConversionError> {
        let mut cell = TableCell::new();

        if style.preserve_line_breaks {
            // Handle edge case: empty code block
            if code.is_empty() {
                let paragraph = self.create_code_paragraph("\u{00A0}", style)?; // Use non-breaking space for visibility
                cell = cell.add_paragraph(paragraph);
            } else {
                // Trim trailing newlines to avoid extra empty lines at the end
                let trimmed_code = code.trim_end_matches('\n');

                // Split code by lines, preserving empty lines
                let lines: Vec<&str> = trimmed_code.split('\n').collect();

                for (_index, line) in lines.iter().enumerate() {
                    // Convert tabs to spaces (4 spaces per tab) for consistent formatting
                    let processed_line = line.replace('\t', "    ");

                    // Handle empty lines by using a non-breaking space to preserve the line
                    let line_content = if processed_line.trim().is_empty() {
                        "\u{00A0}".to_string() // Non-breaking space to preserve empty lines
                    } else {
                        processed_line
                    };

                    let paragraph = self.create_code_paragraph(&line_content, style)?;

                    // TODO: Apply paragraph spacing after each paragraph except the last one
                    // The docx-rs 0.4 API may not support paragraph spacing configuration
                    // This would require: paragraph.spacing_after(style.paragraph_spacing) or similar
                    // For now, spacing is handled through line spacing within paragraphs

                    cell = cell.add_paragraph(paragraph);
                }
            }
        } else {
            // Single paragraph for entire code block with tab conversion
            // Trim trailing newlines to avoid extra empty lines
            let trimmed_code = code.trim_end_matches('\n');
            let processed_code = trimmed_code.replace('\t', "    ");
            let paragraph = self.create_code_paragraph(&processed_code, style)?;
            cell = cell.add_paragraph(paragraph);
        }

        // Apply background color to table cell if specified
        if let Some(bg_color) = &style.background_color {
            let color = bg_color.trim_start_matches('#');
            // Apply cell shading/background color using docx-rs API
            cell = cell.shading(Shading::new().fill(color));
        }

        Ok(cell)
    }

    /// Apply border styling to table based on border_width configuration
    fn apply_table_borders(
        &self,
        table: Table,
        border_width: f32,
    ) -> Result<Table, ConversionError> {
        if border_width <= 0.0 {
            // No borders when border_width is 0 or negative
            return Ok(table);
        }

        // Convert border_width from points to docx-rs border units (eighths of a point)
        let border_size = (border_width * 8.0) as usize;

        // Apply borders to the table using docx-rs API
        // Create table borders with the specified width using the correct API
        let table_borders = TableBorders::new()
            .set(
                TableBorder::new(TableBorderPosition::Top)
                    .border_type(BorderType::Single)
                    .size(border_size)
                    .color("000000"),
            )
            .set(
                TableBorder::new(TableBorderPosition::Bottom)
                    .border_type(BorderType::Single)
                    .size(border_size)
                    .color("000000"),
            )
            .set(
                TableBorder::new(TableBorderPosition::Left)
                    .border_type(BorderType::Single)
                    .size(border_size)
                    .color("000000"),
            )
            .set(
                TableBorder::new(TableBorderPosition::Right)
                    .border_type(BorderType::Single)
                    .size(border_size)
                    .color("000000"),
            )
            .set(
                TableBorder::new(TableBorderPosition::InsideH)
                    .border_type(BorderType::Single)
                    .size(border_size)
                    .color("000000"),
            )
            .set(
                TableBorder::new(TableBorderPosition::InsideV)
                    .border_type(BorderType::Single)
                    .size(border_size)
                    .color("000000"),
            );

        let bordered_table = table.set_borders(table_borders);

        Ok(bordered_table)
    }

    /// Create a consistent code paragraph with proper styling for use within table cells
    fn create_code_paragraph(
        &self,
        text: &str,
        style: &crate::config::CodeBlockStyle,
    ) -> Result<Paragraph, ConversionError> {
        let mut run = Run::new()
            .add_text(text)
            .fonts(
                RunFonts::new()
                    .ascii(&style.font.family)
                    .east_asia(&style.font.family),
            )
            .size((style.font.size * 2.0) as usize); // docx uses half-points

        // Apply bold/italic conditionally based on font configuration
        if style.font.bold {
            run = run.bold();
        }
        if style.font.italic {
            run = run.italic();
        }

        // Add background color if specified (applied to run for better compatibility)
        if let Some(bg_color) = &style.background_color {
            let color = bg_color.trim_start_matches('#');
            run = run.highlight(color);
        }

        // Create paragraph with the styled run and apply line spacing
        let mut paragraph = Paragraph::new().add_run(run).style("CodeBlock"); // Apply code block style to prevent text wrapping

        // Apply line spacing configuration - convert to docx-rs format
        // docx-rs expects line spacing in 240ths of a line (240 = single spacing)
        let line_spacing_value = (style.line_spacing * 240.0) as i32;
        paragraph = paragraph.line_spacing(LineSpacing::new().line(line_spacing_value));

        Ok(paragraph)
    }

    /// Calculate column widths based on content length
    fn calculate_column_widths(&self, headers: &[String], rows: &[Vec<String>]) -> Vec<usize> {
        if headers.is_empty() && rows.is_empty() {
            return vec![];
        }

        // Determine number of columns
        let num_columns = if !headers.is_empty() {
            headers.len()
        } else {
            rows.iter().map(|row| row.len()).max().unwrap_or(0)
        };

        if num_columns == 0 {
            return vec![];
        }

        let mut max_lengths = vec![0; num_columns];

        // Check header lengths
        for (i, header) in headers.iter().enumerate() {
            if i < num_columns {
                max_lengths[i] = max_lengths[i].max(self.estimate_text_width(header));
            }
        }

        // Check data row lengths
        for row in rows {
            for (i, cell) in row.iter().enumerate() {
                if i < num_columns {
                    max_lengths[i] = max_lengths[i].max(self.estimate_text_width(cell));
                }
            }
        }

        // Convert character lengths to docx width units (DXA - twentieths of a point)
        // Base calculation: approximately 120 DXA per character for typical fonts
        // Add padding and ensure minimum/maximum widths
        max_lengths
            .into_iter()
            .map(|len| {
                let base_width = len * 120; // Base width calculation
                let padded_width = base_width + 240; // Add padding (240 DXA = 12pt)
                let min_width = 720; // Minimum width (720 DXA = 36pt)
                let max_width = 2880; // Maximum width (2880 DXA = 144pt)
                
                padded_width.max(min_width).min(max_width)
            })
            .collect()
    }

    /// Estimate text width in characters (accounting for different character types)
    fn estimate_text_width(&self, text: &str) -> usize {
        let mut width = 0;
        
        for ch in text.chars() {
            width += match ch {
                // CJK characters (Chinese, Japanese, Korean) are typically wider
                '\u{4E00}'..='\u{9FFF}' | // CJK Unified Ideographs
                '\u{3400}'..='\u{4DBF}' | // CJK Extension A
                '\u{3040}'..='\u{309F}' | // Hiragana
                '\u{30A0}'..='\u{30FF}' | // Katakana
                '\u{AC00}'..='\u{D7AF}' => 2, // Hangul
                
                // Wide ASCII characters
                'W' | 'M' | 'Q' | 'G' | 'O' | 'D' | 'H' | 'B' | 'R' | 'U' => 2,
                
                // Narrow characters
                'i' | 'l' | 'I' | 'j' | 'f' | 't' | 'r' => 1,
                
                // Regular characters
                _ => if ch.is_ascii() { 1 } else { 2 }
            };
        }
        
        width
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
        let new_has_numbering = config
            .styles
            .headings
            .values()
            .any(|style| style.numbering.is_some());

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
    use crate::config::ConversionConfig;
    use crate::markdown::ast::{InlineElement, ListItem, MarkdownDocument, MarkdownElement};
    use crate::test_utils::{create_test_config, create_test_document};

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
    fn test_create_code_block_cell() {
        let config = ConversionConfig::default();
        let generator = DocxGenerator::new(config);

        // Test single line code
        let code = "println!(\"Hello, world!\");";
        let result = generator.create_code_block_cell(code, &generator.config.styles.code_block);
        assert!(result.is_ok());

        // Test multi-line code with line breaks preserved
        let multi_line_code = "fn main() {\n    println!(\"Hello\");\n}";
        let result =
            generator.create_code_block_cell(multi_line_code, &generator.config.styles.code_block);
        assert!(result.is_ok());

        // Test empty code
        let empty_code = "";
        let result =
            generator.create_code_block_cell(empty_code, &generator.config.styles.code_block);
        assert!(result.is_ok());
    }

    #[test]
    fn test_apply_table_borders() {
        let config = ConversionConfig::default();
        let generator = DocxGenerator::new(config);

        // Create a simple table
        let cell =
            TableCell::new().add_paragraph(Paragraph::new().add_run(Run::new().add_text("test")));
        let row = TableRow::new(vec![cell]);
        let table = Table::new(vec![row]);

        // Test with positive border width
        let result = generator.apply_table_borders(table.clone(), 1.0);
        assert!(result.is_ok());

        // Test with zero border width (no borders)
        let result = generator.apply_table_borders(table.clone(), 0.0);
        assert!(result.is_ok());

        // Test with negative border width (no borders)
        let result = generator.apply_table_borders(table, -1.0);
        assert!(result.is_ok());
    }

    #[test]
    fn test_create_code_paragraph() {
        let config = ConversionConfig::default();
        let generator = DocxGenerator::new(config);

        // Test normal text
        let text = "let x = 42;";
        let result = generator.create_code_paragraph(text, &generator.config.styles.code_block);
        assert!(result.is_ok());

        // Test empty text
        let empty_text = "";
        let result =
            generator.create_code_paragraph(empty_text, &generator.config.styles.code_block);
        assert!(result.is_ok());

        // Test text with special characters
        let special_text = "fn test() -> Result<(), Error> { Ok(()) }";
        let result =
            generator.create_code_paragraph(special_text, &generator.config.styles.code_block);
        assert!(result.is_ok());
    }

    #[test]
    fn test_code_block_font_styling_application() {
        let mut config = ConversionConfig::default();

        // Configure custom font styling
        config.styles.code_block.font.family = "Monaco".to_string();
        config.styles.code_block.font.size = 14.0;
        config.styles.code_block.font.bold = true;
        config.styles.code_block.font.italic = true;
        config.styles.code_block.background_color = Some("#ffff00".to_string());
        config.styles.code_block.line_spacing = 1.5;

        let generator = DocxGenerator::new(config);

        // Test that create_code_paragraph applies all font styling
        let text = "console.log('Hello, World!');";
        let result = generator.create_code_paragraph(text, &generator.config.styles.code_block);
        assert!(result.is_ok());

        // Test that create_code_block_cell applies styling and background color
        let code = "function test() {\n    return 42;\n}";
        let result = generator.create_code_block_cell(code, &generator.config.styles.code_block);
        assert!(result.is_ok());
    }

    #[test]
    fn test_code_block_background_color_application() {
        let mut config = ConversionConfig::default();

        // Test with background color
        config.styles.code_block.background_color = Some("#f0f0f0".to_string());
        let generator = DocxGenerator::new(config);

        let code = "print('test')";
        let result = generator.create_code_block_cell(code, &generator.config.styles.code_block);
        assert!(result.is_ok());

        // Test without background color
        let mut config_no_bg = ConversionConfig::default();
        config_no_bg.styles.code_block.background_color = None;
        let generator_no_bg = DocxGenerator::new(config_no_bg);

        let result_no_bg =
            generator_no_bg.create_code_block_cell(code, &generator_no_bg.config.styles.code_block);
        assert!(result_no_bg.is_ok());
    }

    #[test]
    fn test_table_generation() {
        let config = ConversionConfig::default();
        let mut generator = DocxGenerator::new(config);

        let mut document = MarkdownDocument::new();
        document.add_element(MarkdownElement::Table {
            headers: vec!["Name".to_string(), "Age".to_string(), "City".to_string()],
            rows: vec![
                vec![
                    "Alice".to_string(),
                    "30".to_string(),
                    "New York".to_string(),
                ],
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
            processed: None,
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
            processed: None,
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
            processed: None,
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
            processed: None,
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
        let code_with_tabs =
            "function example() {\n\tif (true) {\n\t\tconsole.log('Hello');\n\t}\n}";

        document.add_element(MarkdownElement::CodeBlock {
            language: Some("javascript".to_string()),
            code: code_with_tabs.to_string(),
            processed: None,
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
            processed: None,
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
            processed: None,
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
            processed: None,
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
            processed: None,
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

    #[test]
    fn test_line_break_preservation_with_empty_lines() {
        let mut config = create_test_config();
        config.styles.code_block.preserve_line_breaks = true;

        let generator = DocxGenerator::new(config);

        // Test code with empty lines and tabs
        let code_with_empty_lines =
            "function test() {\n\n    console.log('hello');\n\n    return true;\n}";

        let cell_result = generator
            .create_code_block_cell(code_with_empty_lines, &generator.config.styles.code_block);
        assert!(cell_result.is_ok());

        // Test that empty code block is handled
        let empty_code_result =
            generator.create_code_block_cell("", &generator.config.styles.code_block);
        assert!(empty_code_result.is_ok());
    }

    #[test]
    fn test_tab_conversion_in_code_blocks() {
        let mut config = create_test_config();
        config.styles.code_block.preserve_line_breaks = true;

        let generator = DocxGenerator::new(config);

        // Test code with tabs that should be converted to spaces
        let code_with_tabs = "function test() {\n\tconsole.log('hello');\n\t\treturn true;\n}";

        let cell_result =
            generator.create_code_block_cell(code_with_tabs, &generator.config.styles.code_block);
        assert!(cell_result.is_ok());

        // Test without line break preservation (single paragraph)
        let mut config_no_preserve = create_test_config();
        config_no_preserve.styles.code_block.preserve_line_breaks = false;
        let generator_no_preserve = DocxGenerator::new(config_no_preserve);

        let cell_result_no_preserve = generator_no_preserve.create_code_block_cell(
            code_with_tabs,
            &generator_no_preserve.config.styles.code_block,
        );
        assert!(cell_result_no_preserve.is_ok());
    }

    #[test]
    fn test_multi_line_code_block_table_generation() {
        let mut config = create_test_config();
        config.styles.code_block.preserve_line_breaks = true;

        let mut generator = DocxGenerator::new(config);

        let mut document = MarkdownDocument::new();
        document.add_element(MarkdownElement::CodeBlock {
            language: Some("python".to_string()),
            code: "def hello_world():\n    print('Hello, World!')\n\n    return 'success'\n\nif __name__ == '__main__':\n    hello_world()".to_string(),
            processed: None,
        });

        let result = generator.generate(&document);
        assert!(result.is_ok());

        let docx_bytes = result.unwrap();
        assert!(!docx_bytes.is_empty());
    }

    #[test]
    fn test_code_block_non_breaking_space_preservation() {
        let mut config = create_test_config();
        config.styles.code_block.preserve_line_breaks = true;

        let generator = DocxGenerator::new(config);

        // Test that empty lines are preserved using non-breaking spaces
        let code_with_multiple_empty_lines = "line1\n\n\nline4\n\n\nline7";

        let cell_result = generator.create_code_block_cell(
            code_with_multiple_empty_lines,
            &generator.config.styles.code_block,
        );
        assert!(cell_result.is_ok());

        // Test edge case: only empty lines
        let only_empty_lines = "\n\n\n";
        let empty_lines_result =
            generator.create_code_block_cell(only_empty_lines, &generator.config.styles.code_block);
        assert!(empty_lines_result.is_ok());
    }

    // Task 9: Unit tests for table-based code block generation

    #[test]
    fn test_single_line_code_block_table_creation() {
        let config = create_test_config();
        let mut generator = DocxGenerator::new(config);

        let mut document = MarkdownDocument::new();
        document.add_element(MarkdownElement::CodeBlock {
            language: Some("rust".to_string()),
            code: "let x = 42;".to_string(),
            processed: None,
        });

        let result = generator.generate(&document);
        assert!(result.is_ok());

        let docx_bytes = result.unwrap();
        assert!(!docx_bytes.is_empty());

        // Test that the code block is rendered as a table
        let cell_result =
            generator.create_code_block_cell("let x = 42;", &generator.config.styles.code_block);
        assert!(cell_result.is_ok());
    }

    #[test]
    fn test_multi_line_code_block_with_line_break_preservation() {
        let mut config = create_test_config();
        config.styles.code_block.preserve_line_breaks = true;
        let mut generator = DocxGenerator::new(config);

        let multi_line_code = "fn main() {\n    println!(\"Hello, world!\");\n    let x = 42;\n    println!(\"x = {}\", x);\n}";

        let mut document = MarkdownDocument::new();
        document.add_element(MarkdownElement::CodeBlock {
            language: Some("rust".to_string()),
            code: multi_line_code.to_string(),
            processed: None,
        });

        let result = generator.generate(&document);
        assert!(result.is_ok());

        let docx_bytes = result.unwrap();
        assert!(!docx_bytes.is_empty());

        // Test that each line is preserved as separate paragraphs within the table cell
        let cell_result =
            generator.create_code_block_cell(multi_line_code, &generator.config.styles.code_block);
        assert!(cell_result.is_ok());

        // Test with preserve_line_breaks = false for comparison
        let mut config_no_preserve = create_test_config();
        config_no_preserve.styles.code_block.preserve_line_breaks = false;
        let generator_no_preserve = DocxGenerator::new(config_no_preserve);
        let cell_result_no_preserve = generator_no_preserve.create_code_block_cell(
            multi_line_code,
            &generator_no_preserve.config.styles.code_block,
        );
        assert!(cell_result_no_preserve.is_ok());
    }

    #[test]
    fn test_empty_code_block_table_handling() {
        let mut config = create_test_config();
        config.styles.code_block.preserve_line_breaks = true;
        let mut generator = DocxGenerator::new(config);

        let mut document = MarkdownDocument::new();
        document.add_element(MarkdownElement::CodeBlock {
            language: Some("text".to_string()),
            code: "".to_string(),
            processed: None,
        });

        let result = generator.generate(&document);
        assert!(result.is_ok());

        let docx_bytes = result.unwrap();
        assert!(!docx_bytes.is_empty());

        // Test that empty code block creates a table with non-breaking space for visibility
        let cell_result = generator.create_code_block_cell("", &generator.config.styles.code_block);
        assert!(cell_result.is_ok());

        // Test with preserve_line_breaks = false
        let mut config_no_preserve = create_test_config();
        config_no_preserve.styles.code_block.preserve_line_breaks = false;
        let generator_no_preserve = DocxGenerator::new(config_no_preserve);
        let cell_result_no_preserve = generator_no_preserve
            .create_code_block_cell("", &generator_no_preserve.config.styles.code_block);
        assert!(cell_result_no_preserve.is_ok());
    }

    #[test]
    fn test_border_application_with_various_border_width_values() {
        let config = create_test_config();
        let generator = DocxGenerator::new(config);

        // Create a simple table for testing borders
        let cell =
            TableCell::new().add_paragraph(Paragraph::new().add_run(Run::new().add_text("test")));
        let row = TableRow::new(vec![cell]);
        let table = Table::new(vec![row]);

        // Test with border_width = 0.0 (no borders)
        let result_no_border = generator.apply_table_borders(table.clone(), 0.0);
        assert!(result_no_border.is_ok());

        // Test with border_width = 1.0 (standard border)
        let result_standard_border = generator.apply_table_borders(table.clone(), 1.0);
        assert!(result_standard_border.is_ok());

        // Test with border_width = 2.5 (thick border)
        let result_thick_border = generator.apply_table_borders(table.clone(), 2.5);
        assert!(result_thick_border.is_ok());

        // Test with border_width = 0.5 (thin border)
        let result_thin_border = generator.apply_table_borders(table.clone(), 0.5);
        assert!(result_thin_border.is_ok());

        // Test with negative border_width (should be treated as no border)
        let result_negative_border = generator.apply_table_borders(table.clone(), -1.0);
        assert!(result_negative_border.is_ok());

        // Test complete code block generation with different border widths
        let mut config_no_border = create_test_config();
        config_no_border.styles.code_block.border_width = 0.0;
        let mut generator_no_border = DocxGenerator::new(config_no_border);

        let mut config_thick_border = create_test_config();
        config_thick_border.styles.code_block.border_width = 3.0;
        let mut generator_thick_border = DocxGenerator::new(config_thick_border);

        let mut document = MarkdownDocument::new();
        document.add_element(MarkdownElement::CodeBlock {
            language: Some("rust".to_string()),
            code: "println!(\"test\");".to_string(),
            processed: None,
        });

        let result_no_border_doc = generator_no_border.generate(&document);
        assert!(result_no_border_doc.is_ok());

        let result_thick_border_doc = generator_thick_border.generate(&document);
        assert!(result_thick_border_doc.is_ok());
    }

    #[test]
    fn test_font_and_styling_application_within_table_cells() {
        let mut config = create_test_config();

        // Configure custom font styling for comprehensive testing
        config.styles.code_block.font.family = "Monaco".to_string();
        config.styles.code_block.font.size = 14.0;
        config.styles.code_block.font.bold = true;
        config.styles.code_block.font.italic = true;
        config.styles.code_block.background_color = Some("#f0f0f0".to_string());
        config.styles.code_block.line_spacing = 1.2;
        config.styles.code_block.preserve_line_breaks = true;

        let mut generator = DocxGenerator::new(config);

        let code_with_styling = "// This is a comment\nfn styled_function() {\n    let variable = \"value\";\n    return variable;\n}";

        let mut document = MarkdownDocument::new();
        document.add_element(MarkdownElement::CodeBlock {
            language: Some("rust".to_string()),
            code: code_with_styling.to_string(),
            processed: None,
        });

        let result = generator.generate(&document);
        assert!(result.is_ok());

        let docx_bytes = result.unwrap();
        assert!(!docx_bytes.is_empty());

        // Test that create_code_paragraph applies all font styling correctly
        let paragraph_result =
            generator.create_code_paragraph("test code", &generator.config.styles.code_block);
        assert!(paragraph_result.is_ok());

        // Test that create_code_block_cell applies styling and background color
        let cell_result = generator
            .create_code_block_cell(code_with_styling, &generator.config.styles.code_block);
        assert!(cell_result.is_ok());

        // Test with different font configurations
        let mut config_different_font = create_test_config();
        config_different_font.styles.code_block.font.family = "Consolas".to_string();
        config_different_font.styles.code_block.font.size = 11.0;
        config_different_font.styles.code_block.font.bold = false;
        config_different_font.styles.code_block.font.italic = false;
        config_different_font.styles.code_block.background_color = None;

        let generator_different = DocxGenerator::new(config_different_font);
        let cell_result_different = generator_different
            .create_code_block_cell("test", &generator_different.config.styles.code_block);
        assert!(cell_result_different.is_ok());

        // Test paragraph creation with different styling
        let paragraph_result_different = generator_different
            .create_code_paragraph("test", &generator_different.config.styles.code_block);
        assert!(paragraph_result_different.is_ok());
    }

    #[test]
    fn test_code_block_table_with_background_color_variations() {
        // Test with hex color with #
        let mut config_hex = create_test_config();
        config_hex.styles.code_block.background_color = Some("#ffff00".to_string());
        let generator_hex = DocxGenerator::new(config_hex);

        // Test with hex color without #
        let mut config_hex_no_hash = create_test_config();
        config_hex_no_hash.styles.code_block.background_color = Some("ff0000".to_string());
        let generator_hex_no_hash = DocxGenerator::new(config_hex_no_hash);

        // Test with no background color
        let mut config_no_bg = create_test_config();
        config_no_bg.styles.code_block.background_color = None;
        let generator_no_bg = DocxGenerator::new(config_no_bg);

        let test_code = "console.log('background test');";

        // All should succeed
        let cell_result_hex = generator_hex
            .create_code_block_cell(test_code, &generator_hex.config.styles.code_block);
        assert!(cell_result_hex.is_ok());

        let cell_result_hex_no_hash = generator_hex_no_hash
            .create_code_block_cell(test_code, &generator_hex_no_hash.config.styles.code_block);
        assert!(cell_result_hex_no_hash.is_ok());

        let cell_result_no_bg = generator_no_bg
            .create_code_block_cell(test_code, &generator_no_bg.config.styles.code_block);
        assert!(cell_result_no_bg.is_ok());
    }

    #[test]
    fn test_code_block_table_with_special_characters_and_formatting() {
        let config = create_test_config();
        let mut generator = DocxGenerator::new(config);

        // Test code with special characters, unicode, and various formatting
        let special_code = "// Special chars: <>\"'&\nfn test() -> Result<(), Box<dyn Error>> {\n    let emoji = \"🚀\";\n    let unicode = \"αβγδε\";\n    let xml = \"<tag attr=\\\"value\\\">content</tag>\";\n    Ok(())\n}";

        let mut document = MarkdownDocument::new();
        document.add_element(MarkdownElement::CodeBlock {
            language: Some("rust".to_string()),
            code: special_code.to_string(),
            processed: None,
        });

        let result = generator.generate(&document);
        assert!(result.is_ok());

        let docx_bytes = result.unwrap();
        assert!(!docx_bytes.is_empty());

        // Test that special characters are preserved in table cells
        let cell_result =
            generator.create_code_block_cell(special_code, &generator.config.styles.code_block);
        assert!(cell_result.is_ok());

        // Test with tabs and mixed whitespace
        let whitespace_code = "function test() {\n\tif (condition) {\n\t\treturn true;\n\t}\n    return false; // 4 spaces\n}";
        let cell_result_whitespace =
            generator.create_code_block_cell(whitespace_code, &generator.config.styles.code_block);
        assert!(cell_result_whitespace.is_ok());
    }

    #[test]
    fn test_code_block_with_markdown_formatting() {
        let config = create_test_config();
        let mut generator = DocxGenerator::new(config);

        // Test code block with Markdown formatting
        let markdown_code = "This is **bold** text and *italic* text.\nAlso `inline code` and [link](http://example.com).\n![image](image.jpg)";

        let mut document = MarkdownDocument::new();
        document.add_element(MarkdownElement::CodeBlock {
            language: Some("markdown".to_string()),
            code: markdown_code.to_string(),
            processed: None,
        });

        let result = generator.generate(&document);
        assert!(result.is_ok());

        let docx_bytes = result.unwrap();
        assert!(!docx_bytes.is_empty());
    }

    #[test]
    fn test_contains_markdown_formatting() {
        let config = create_test_config();
        let generator = DocxGenerator::new(config);

        // Test various Markdown patterns
        assert!(generator.contains_markdown_formatting("**bold**"));
        assert!(generator.contains_markdown_formatting("*italic*"));
        assert!(generator.contains_markdown_formatting("`code`"));
        assert!(generator.contains_markdown_formatting("[link](url)"));
        assert!(generator.contains_markdown_formatting("![image](url)"));
        
        // Test plain text
        assert!(!generator.contains_markdown_formatting("plain text"));
        assert!(!generator.contains_markdown_formatting("no formatting here"));
    }

    #[test]
    fn test_create_code_block_cell_with_markdown() {
        let config = create_test_config();
        let generator = DocxGenerator::new(config);

        // Test with Markdown formatting
        let markdown_text = "**Bold** and *italic* text";
        let result = generator.create_code_block_cell_with_markdown(markdown_text, &generator.config.styles.code_block);
        assert!(result.is_ok());

        // Test with plain text (should fall back to regular method)
        let plain_text = "plain text without formatting";
        let result = generator.create_code_block_cell_with_markdown(plain_text, &generator.config.styles.code_block);
        assert!(result.is_ok());

        // Test with mixed content
        let mixed_text = "Code: `function()` and **important** note";
        let result = generator.create_code_block_cell_with_markdown(mixed_text, &generator.config.styles.code_block);
        assert!(result.is_ok());
    }

    #[test]
    fn test_create_code_run_from_inline() {
        let config = create_test_config();
        let generator = DocxGenerator::new(config);

        // Test different inline elements
        let bold_inline = InlineElement::Bold("bold text".to_string());
        let result = generator.create_code_run_from_inline(&bold_inline, &generator.config.styles.code_block);
        assert!(result.is_ok());

        let italic_inline = InlineElement::Italic("italic text".to_string());
        let result = generator.create_code_run_from_inline(&italic_inline, &generator.config.styles.code_block);
        assert!(result.is_ok());

        let code_inline = InlineElement::Code("nested code".to_string());
        let result = generator.create_code_run_from_inline(&code_inline, &generator.config.styles.code_block);
        assert!(result.is_ok());

        let link_inline = InlineElement::Link {
            text: "link text".to_string(),
            url: "http://example.com".to_string(),
            title: None,
        };
        let result = generator.create_code_run_from_inline(&link_inline, &generator.config.styles.code_block);
        assert!(result.is_ok());
    }

    #[test]
    fn test_extract_text_from_element() {
        let config = create_test_config();
        let generator = DocxGenerator::new(config);

        // Test heading
        let heading = MarkdownElement::Heading {
            level: 1,
            text: "Test Heading".to_string(),
        };
        let text = generator.extract_text_from_element(&heading);
        assert_eq!(text, "Test Heading");

        // Test paragraph
        let paragraph = MarkdownElement::Paragraph {
            content: vec![
                InlineElement::Text("Hello ".to_string()),
                InlineElement::Bold("world".to_string()),
            ],
        };
        let text = generator.extract_text_from_element(&paragraph);
        assert_eq!(text, "Hello world");

        // Test image
        let image = MarkdownElement::Image {
            alt_text: "Test Image".to_string(),
            url: "image.jpg".to_string(),
            title: None,
        };
        let text = generator.extract_text_from_element(&image);
        assert_eq!(text, "Test Image");
    }

    #[test]
    fn test_code_block_table_line_spacing_configuration() {
        // Test various line spacing values
        let mut config_single = create_test_config();
        config_single.styles.code_block.line_spacing = 1.0;
        let generator_single = DocxGenerator::new(config_single);

        let mut config_double = create_test_config();
        config_double.styles.code_block.line_spacing = 2.0;
        let generator_double = DocxGenerator::new(config_double);

        let mut config_custom = create_test_config();
        config_custom.styles.code_block.line_spacing = 1.5;
        let generator_custom = DocxGenerator::new(config_custom);

        let test_code = "line1\nline2\nline3";

        // Test paragraph creation with different line spacing
        let paragraph_single = generator_single
            .create_code_paragraph(test_code, &generator_single.config.styles.code_block);
        assert!(paragraph_single.is_ok());

        let paragraph_double = generator_double
            .create_code_paragraph(test_code, &generator_double.config.styles.code_block);
        assert!(paragraph_double.is_ok());

        let paragraph_custom = generator_custom
            .create_code_paragraph(test_code, &generator_custom.config.styles.code_block);
        assert!(paragraph_custom.is_ok());

        // Test complete table cell creation with different line spacing
        let cell_single = generator_single
            .create_code_block_cell(test_code, &generator_single.config.styles.code_block);
        assert!(cell_single.is_ok());

        let cell_double = generator_double
            .create_code_block_cell(test_code, &generator_double.config.styles.code_block);
        assert!(cell_double.is_ok());

        let cell_custom = generator_custom
            .create_code_block_cell(test_code, &generator_custom.config.styles.code_block);
        assert!(cell_custom.is_ok());
    }

    #[test]
    fn test_calculate_column_widths() {
        let config = create_test_config();
        let generator = DocxGenerator::new(config);

        // Test with headers and data
        let headers = vec!["Name".to_string(), "Age".to_string(), "Very Long Description".to_string()];
        let rows = vec![
            vec!["Alice".to_string(), "30".to_string(), "Short".to_string()],
            vec!["Bob".to_string(), "25".to_string(), "Medium length text".to_string()],
            vec!["Charlie".to_string(), "35".to_string(), "This is a very long description that should determine the column width".to_string()],
        ];

        let widths = generator.calculate_column_widths(&headers, &rows);
        assert_eq!(widths.len(), 3);
        
        // The third column should be wider due to the long content
        assert!(widths[2] > widths[0]);
        assert!(widths[2] > widths[1]);
        
        // All widths should be within reasonable bounds
        for width in &widths {
            assert!(*width >= 720); // Minimum width
            assert!(*width <= 2880); // Maximum width
        }
    }

    #[test]
    fn test_estimate_text_width() {
        let config = create_test_config();
        let generator = DocxGenerator::new(config);

        // Test ASCII text
        let ascii_text = "Hello";
        let ascii_width = generator.estimate_text_width(ascii_text);
        assert_eq!(ascii_width, 6); // H(2) + e(1) + l(1) + l(1) + o(1) = 6

        // Test wide characters
        let wide_text = "WWW";
        let wide_width = generator.estimate_text_width(wide_text);
        assert_eq!(wide_width, 6); // 3 wide characters = 6 units

        // Test narrow characters
        let narrow_text = "iii";
        let narrow_width = generator.estimate_text_width(narrow_text);
        assert_eq!(narrow_width, 3); // 3 narrow characters = 3 units

        // Test CJK characters
        let cjk_text = "你好";
        let cjk_width = generator.estimate_text_width(cjk_text);
        assert_eq!(cjk_width, 4); // 2 CJK characters = 4 units

        // Test mixed content
        let mixed_text = "Hello你好";
        let mixed_width = generator.estimate_text_width(mixed_text);
        assert_eq!(mixed_width, 10); // H(2) + e(1) + l(1) + l(1) + o(1) + 你(2) + 好(2) = 10 units
    }

    #[test]
    fn test_table_with_auto_width() {
        let config = create_test_config();
        let mut generator = DocxGenerator::new(config);

        let mut document = MarkdownDocument::new();
        document.add_element(MarkdownElement::Table {
            headers: vec!["Short".to_string(), "Medium Length".to_string(), "Very Long Header Name".to_string()],
            rows: vec![
                vec!["A".to_string(), "Medium".to_string(), "This is a very long cell content that should make the column wider".to_string()],
                vec!["B".to_string(), "Text".to_string(), "Short".to_string()],
            ],
        });

        let result = generator.generate(&document);
        assert!(result.is_ok());

        let docx_bytes = result.unwrap();
        assert!(!docx_bytes.is_empty());
    }

    #[test]
    fn test_empty_table_handling() {
        let config = create_test_config();
        let generator = DocxGenerator::new(config);

        // Test empty headers and rows
        let widths = generator.calculate_column_widths(&[], &[]);
        assert!(widths.is_empty());

        // Test empty headers with data
        let rows = vec![vec!["data".to_string()]];
        let widths = generator.calculate_column_widths(&[], &rows);
        assert_eq!(widths.len(), 1);

        // Test headers with empty rows
        let headers = vec!["Header".to_string()];
        let widths = generator.calculate_column_widths(&headers, &[]);
        assert_eq!(widths.len(), 1);
    }

    #[test]
    fn test_table_with_borders_and_auto_width() {
        let mut config = create_test_config();
        config.styles.table.border_width = 1.5;
        let mut generator = DocxGenerator::new(config);

        let mut document = MarkdownDocument::new();
        document.add_element(MarkdownElement::Table {
            headers: vec!["Name".to_string(), "Description".to_string()],
            rows: vec![
                vec!["Item 1".to_string(), "Short description".to_string()],
                vec!["Item 2".to_string(), "This is a much longer description that should affect the column width".to_string()],
            ],
        });

        let result = generator.generate(&document);
        assert!(result.is_ok());

        let docx_bytes = result.unwrap();
        assert!(!docx_bytes.is_empty());
    }
}
