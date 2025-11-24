//! Note code block processing strategy
//! 
//! This strategy processes "note" type code blocks by:
//! - Making the first line bold, italic, and larger
//! - Adding a small tip icon in the top-right corner of the rendered table

use std::time::Instant;
use crate::markdown::code_block::{
    CodeBlockStrategy, ProcessedCodeBlock, ProcessingConfig, 
    ProcessingError, ProcessingMetadata, language_matches
};

/// Strategy for processing note-type code blocks
#[derive(Debug, Clone)]
pub struct NoteStrategy {
    /// Path to the tip icon image
    tip_icon_path: String,
}

impl NoteStrategy {
    /// Create a new Note strategy instance with default icon path
    pub fn new() -> Self {
        Self {
            tip_icon_path: "default-qrcode.png".to_string(),
        }
    }

    /// Create a new Note strategy instance with custom icon path
    pub fn with_icon_path(icon_path: String) -> Self {
        Self {
            tip_icon_path: icon_path,
        }
    }

    /// Process the note content by formatting the first line and adding icon
    fn format_note_content(&self, content: &str) -> Result<String, ProcessingError> {
        let lines: Vec<&str> = content.lines().collect();
        
        if lines.is_empty() {
            return Ok(content.to_string());
        }

        let mut result = String::new();
        
        // Create a table structure with the icon in the top-right corner
        result.push_str("<table style=\"width: 100%; border: 1px solid #e0e0e0; border-radius: 8px; padding: 16px; background-color: #f8f9fa;\">\n");
        result.push_str("<tr>\n");
        result.push_str("<td style=\"vertical-align: top;\">\n\n");
        
        // Format the first line: bold, italic, and larger font
        let first_line = lines[0].trim();
        if !first_line.is_empty() {
            result.push_str(&format!(
                "<span style=\"font-size: 1.2em; font-weight: bold; font-style: italic;\">{}</span>\n\n",
                first_line
            ));
        }
        
        // Add remaining lines as normal text
        for line in lines.iter().skip(1) {
            result.push_str(line);
            result.push('\n');
        }
        
        result.push_str("</td>\n");
        
        // Add the tip icon in the top-right corner
        result.push_str("<td style=\"width: 48px; vertical-align: top; text-align: right;\">\n");
        result.push_str(&format!(
            "<img src=\"{}\" alt=\"Tip\" style=\"width: 32px; height: 32px; opacity: 0.7;\" />\n",
            self.tip_icon_path
        ));
        result.push_str("</td>\n");
        
        result.push_str("</tr>\n");
        result.push_str("</table>\n");
        
        Ok(result)
    }

    /// Get the icon path
    pub fn get_icon_path(&self) -> &str {
        &self.tip_icon_path
    }

    /// Set a new icon path
    pub fn set_icon_path(&mut self, path: String) {
        self.tip_icon_path = path;
    }
}

impl CodeBlockStrategy for NoteStrategy {
    fn process(&self, code: &str, config: &ProcessingConfig) -> Result<ProcessedCodeBlock, ProcessingError> {
        let start_time = Instant::now();
        
        // Format the note content if formatting is enabled
        let formatted_code = if config.enable_formatting {
            match self.format_note_content(code) {
                Ok(formatted) => Some(formatted),
                Err(e) => {
                    return Err(e);
                }
            }
        } else {
            None
        };
        
        // Create metadata
        let mut metadata = ProcessingMetadata::new(self.get_version())
            .with_processing_time(start_time.elapsed());
        
        metadata.is_formatted = formatted_code.is_some();
        metadata.is_validated = false; // No syntax validation for notes
        metadata.syntax_valid = true; // Always valid
        
        // Add custom attributes
        metadata = metadata.with_custom_attribute("language", "note");
        metadata = metadata.with_custom_attribute("formatter", "note_formatter");
        metadata = metadata.with_custom_attribute("icon_path", &self.tip_icon_path);
        
        let processed = ProcessedCodeBlock::new(code.to_string(), Some("note".to_string()))
            .with_metadata(metadata);
        
        let processed = if let Some(formatted) = formatted_code {
            processed.with_processed_code(formatted)
        } else {
            processed
        };
        
        Ok(processed)
    }
    
    fn supports_language(&self, language: &str) -> bool {
        language_matches(language, &["note", "notes", "tip", "tips", "hint"])
    }
    
    fn get_language_name(&self) -> &'static str {
        "note"
    }
    
    fn get_priority(&self) -> u8 {
        120 // Medium-high priority
    }
    
    fn get_version(&self) -> &'static str {
        "1.0.0"
    }
    
    fn get_description(&self) -> &'static str {
        "Note/tip code block processing with styled first line and icon"
    }
}

impl Default for NoteStrategy {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::markdown::code_block::ProcessingConfig;

    #[test]
    fn test_note_strategy_creation() {
        let strategy = NoteStrategy::new();
        assert_eq!(strategy.get_language_name(), "note");
        assert_eq!(strategy.get_priority(), 120);
        assert!(strategy.supports_language("note"));
        assert!(strategy.supports_language("tip"));
        assert!(!strategy.supports_language("rust"));
    }

    #[test]
    fn test_note_strategy_with_custom_icon() {
        let strategy = NoteStrategy::with_icon_path("custom-icon.png".to_string());
        assert_eq!(strategy.get_icon_path(), "custom-icon.png");
    }

    #[test]
    fn test_note_content_formatting() {
        let strategy = NoteStrategy::new();
        let content = "Important Note\nThis is the body of the note.\nIt can have multiple lines.";
        
        let result = strategy.format_note_content(content);
        assert!(result.is_ok());
        
        let formatted = result.unwrap();
        // Check that it contains HTML table structure
        assert!(formatted.contains("<table"));
        assert!(formatted.contains("</table>"));
        // Check that first line is styled
        assert!(formatted.contains("font-weight: bold"));
        assert!(formatted.contains("font-style: italic"));
        assert!(formatted.contains("font-size: 1.2em"));
        // Check that icon is included
        assert!(formatted.contains("<img"));
        assert!(formatted.contains("default-qrcode.png"));
    }

    #[test]
    fn test_note_strategy_process_with_formatting() {
        let strategy = NoteStrategy::new();
        let config = ProcessingConfig::default()
            .with_formatting(true);
        
        let code = "Pro Tip\nAlways test your code before committing.";
        
        let result = strategy.process(code, &config);
        assert!(result.is_ok());
        
        let processed = result.unwrap();
        assert_eq!(processed.language, Some("note".to_string()));
        assert!(processed.metadata.is_formatted);
        assert!(processed.processed_code.is_some());
        assert!(processed.is_successful());
        
        let formatted = processed.processed_code.unwrap();
        assert!(formatted.contains("Pro Tip"));
        assert!(formatted.contains("font-weight: bold"));
    }

    #[test]
    fn test_note_strategy_process_without_formatting() {
        let strategy = NoteStrategy::new();
        let config = ProcessingConfig::default()
            .with_formatting(false);
        
        let code = "Simple note\nNo formatting applied.";
        
        let result = strategy.process(code, &config);
        assert!(result.is_ok());
        
        let processed = result.unwrap();
        assert!(!processed.metadata.is_formatted);
        assert!(processed.processed_code.is_none());
    }

    #[test]
    fn test_note_strategy_empty_content() {
        let strategy = NoteStrategy::new();
        let result = strategy.format_note_content("");
        assert!(result.is_ok());
    }

    #[test]
    fn test_note_strategy_single_line() {
        let strategy = NoteStrategy::new();
        let content = "Single line note";
        
        let result = strategy.format_note_content(content);
        assert!(result.is_ok());
        
        let formatted = result.unwrap();
        assert!(formatted.contains("Single line note"));
        assert!(formatted.contains("font-weight: bold"));
    }

    #[test]
    fn test_note_strategy_metadata() {
        let strategy = NoteStrategy::new();
        let config = ProcessingConfig::default()
            .with_formatting(true);
        
        let code = "Test Note\nContent here.";
        
        let result = strategy.process(code, &config);
        assert!(result.is_ok());
        
        let processed = result.unwrap();
        assert_eq!(processed.metadata.get_custom_attribute("language"), Some(&"note".to_string()));
        assert_eq!(processed.metadata.get_custom_attribute("formatter"), Some(&"note_formatter".to_string()));
        assert_eq!(processed.metadata.get_custom_attribute("icon_path"), Some(&"default-qrcode.png".to_string()));
    }

    #[test]
    fn test_note_strategy_supports_aliases() {
        let strategy = NoteStrategy::new();
        
        assert!(strategy.supports_language("note"));
        assert!(strategy.supports_language("notes"));
        assert!(strategy.supports_language("tip"));
        assert!(strategy.supports_language("tips"));
        assert!(strategy.supports_language("hint"));
        assert!(strategy.supports_language("NOTE"));
        assert!(strategy.supports_language("TIP"));
    }

    #[test]
    fn test_set_icon_path() {
        let mut strategy = NoteStrategy::new();
        assert_eq!(strategy.get_icon_path(), "default-qrcode.png");
        
        strategy.set_icon_path("new-icon.svg".to_string());
        assert_eq!(strategy.get_icon_path(), "new-icon.svg");
    }

    #[test]
    fn test_formatted_output_structure() {
        let strategy = NoteStrategy::new();
        let content = "Title Line\nFirst body line\nSecond body line";
        
        let result = strategy.format_note_content(content);
        assert!(result.is_ok());
        
        let formatted = result.unwrap();
        
        // Verify table structure
        assert!(formatted.contains("<table"));
        assert!(formatted.contains("<tr>"));
        assert!(formatted.contains("<td"));
        assert!(formatted.contains("</td>"));
        assert!(formatted.contains("</tr>"));
        assert!(formatted.contains("</table>"));
        
        // Verify styling
        assert!(formatted.contains("border-radius"));
        assert!(formatted.contains("background-color"));
        
        // Verify content
        assert!(formatted.contains("Title Line"));
        assert!(formatted.contains("First body line"));
        assert!(formatted.contains("Second body line"));
    }
}
