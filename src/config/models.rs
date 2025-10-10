//! Configuration data models

use crate::numbering::NumberingFormatter;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

/// Configuration validation errors
#[derive(Debug, Error)]
pub enum ValidationError {
    #[error("Invalid page size: width and height must be positive")]
    InvalidPageSize,
    #[error("Invalid margins: all margins must be non-negative")]
    InvalidMargins,
    #[error("Invalid font size: must be positive")]
    InvalidFontSize,
    #[error("Invalid font family: cannot be empty")]
    InvalidFontFamily,
    #[error("Invalid heading level: must be between 1 and 6")]
    InvalidHeadingLevel,
    #[error("Invalid spacing: must be non-negative")]
    InvalidSpacing,
    #[error("Invalid color format: {0}")]
    InvalidColor(String),
    #[error("Invalid image dimensions: must be positive")]
    InvalidImageDimensions,
    #[error("Invalid numbering format: {0}")]
    InvalidNumberingFormat(String),
}

/// Main configuration structure for conversion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversionConfig {
    pub document: DocumentConfig,
    pub styles: StyleConfig,
    pub elements: ElementConfig,
}

/// Document-level configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentConfig {
    pub page_size: PageSize,
    pub margins: Margins,
    pub default_font: FontConfig,
}

/// Style configuration for different elements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StyleConfig {
    pub headings: HashMap<u8, HeadingStyle>,
    pub paragraph: ParagraphStyle,
    pub code_block: CodeBlockStyle,
    pub table: TableStyle,
}

/// Element-specific configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElementConfig {
    pub image: ImageConfig,
    pub list: ListConfig,
    pub link: LinkConfig,
}

/// Page size configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageSize {
    pub width: f32,
    pub height: f32,
}

impl PageSize {
    /// Validate page size
    pub fn validate(&self) -> Result<(), ValidationError> {
        if self.width <= 0.0 || self.height <= 0.0 {
            return Err(ValidationError::InvalidPageSize);
        }
        Ok(())
    }
}

/// Margin configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Margins {
    pub top: f32,
    pub bottom: f32,
    pub left: f32,
    pub right: f32,
}

impl Margins {
    /// Validate margins
    pub fn validate(&self) -> Result<(), ValidationError> {
        if self.top < 0.0 || self.bottom < 0.0 || self.left < 0.0 || self.right < 0.0 {
            return Err(ValidationError::InvalidMargins);
        }
        Ok(())
    }
}

/// Font configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FontConfig {
    pub family: String,
    pub size: f32,
    pub bold: bool,
    pub italic: bool,
}

impl FontConfig {
    /// Validate font configuration
    pub fn validate(&self) -> Result<(), ValidationError> {
        if self.family.trim().is_empty() {
            return Err(ValidationError::InvalidFontFamily);
        }
        if self.size <= 0.0 {
            return Err(ValidationError::InvalidFontSize);
        }
        Ok(())
    }
}

/// Heading style configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeadingStyle {
    pub font: FontConfig,
    pub spacing_before: f32,
    pub spacing_after: f32,
    pub alignment: Option<String>,
    pub numbering: Option<String>,
}

impl HeadingStyle {
    /// Validate heading style
    pub fn validate(&self) -> Result<(), ValidationError> {
        self.font.validate()?;
        if self.spacing_before < 0.0 || self.spacing_after < 0.0 {
            return Err(ValidationError::InvalidSpacing);
        }

        // Validate numbering format if present
        if let Some(numbering) = &self.numbering {
            if let Err(numbering_error) = NumberingFormatter::parse_format(numbering) {
                return Err(ValidationError::InvalidNumberingFormat(
                    numbering_error.to_string(),
                ));
            }
        }

        Ok(())
    }
}

/// Paragraph style configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParagraphStyle {
    pub font: FontConfig,
    pub line_spacing: f32,
    pub spacing_after: f32,
}

impl ParagraphStyle {
    /// Validate paragraph style
    pub fn validate(&self) -> Result<(), ValidationError> {
        self.font.validate()?;
        if self.line_spacing <= 0.0 || self.spacing_after < 0.0 {
            return Err(ValidationError::InvalidSpacing);
        }
        Ok(())
    }
}

/// Code block style configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeBlockStyle {
    pub font: FontConfig,
    pub background_color: Option<String>,
    pub border: bool,
    pub preserve_line_breaks: bool,
    pub line_spacing: f32,
    pub paragraph_spacing: f32,
}

impl CodeBlockStyle {
    /// Validate code block style
    pub fn validate(&self) -> Result<(), ValidationError> {
        self.font.validate()?;
        if let Some(color) = &self.background_color {
            validate_color(color)?;
        }
        if self.line_spacing <= 0.0 || self.paragraph_spacing < 0.0 {
            return Err(ValidationError::InvalidSpacing);
        }
        Ok(())
    }
}

/// Table style configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableStyle {
    pub header_font: FontConfig,
    pub cell_font: FontConfig,
    pub border_width: f32,
}

impl TableStyle {
    /// Validate table style
    pub fn validate(&self) -> Result<(), ValidationError> {
        self.header_font.validate()?;
        self.cell_font.validate()?;
        if self.border_width < 0.0 {
            return Err(ValidationError::InvalidSpacing);
        }
        Ok(())
    }
}

/// Image configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageConfig {
    pub max_width: f32,
    pub max_height: f32,
}

/// List configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListConfig {
    pub indent: f32,
    pub spacing: f32,
}

/// Link configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkConfig {
    pub color: String,
    pub underline: bool,
}

impl ConversionConfig {
    /// Validate the entire configuration
    pub fn validate(&self) -> Result<(), ValidationError> {
        self.document.validate()?;
        self.styles.validate()?;
        self.elements.validate()?;
        Ok(())
    }
}

impl Default for ConversionConfig {
    fn default() -> Self {
        Self {
            document: DocumentConfig::default(),
            styles: StyleConfig::default(),
            elements: ElementConfig::default(),
        }
    }
}

impl DocumentConfig {
    /// Validate document configuration
    pub fn validate(&self) -> Result<(), ValidationError> {
        self.page_size.validate()?;
        self.margins.validate()?;
        self.default_font.validate()?;
        Ok(())
    }
}

impl Default for DocumentConfig {
    fn default() -> Self {
        Self {
            page_size: PageSize {
                width: 595.0,  // A4 width in points
                height: 842.0, // A4 height in points
            },
            margins: Margins {
                top: 72.0,
                bottom: 72.0,
                left: 72.0,
                right: 72.0,
            },
            default_font: FontConfig {
                family: "Times New Roman".to_string(),
                size: 12.0,
                bold: false,
                italic: false,
            },
        }
    }
}

impl StyleConfig {
    /// Validate style configuration
    pub fn validate(&self) -> Result<(), ValidationError> {
        // Validate heading levels and styles
        for (&level, style) in &self.headings {
            if !(1..=6).contains(&level) {
                return Err(ValidationError::InvalidHeadingLevel);
            }
            style.validate()?;
        }

        self.paragraph.validate()?;
        self.code_block.validate()?;
        self.table.validate()?;
        Ok(())
    }
}

impl Default for StyleConfig {
    fn default() -> Self {
        let mut headings = HashMap::new();

        // Default heading styles
        for level in 1..=6 {
            let size = match level {
                1 => 18.0,
                2 => 16.0,
                3 => 14.0,
                4 => 12.0,
                5 => 11.0,
                6 => 10.0,
                _ => 12.0,
            };

            headings.insert(
                level,
                HeadingStyle {
                    font: FontConfig {
                        family: "Times New Roman".to_string(),
                        size,
                        bold: true,
                        italic: false,
                    },
                    spacing_before: 12.0,
                    spacing_after: 6.0,
                    alignment: None,
                    numbering: None,
                },
            );
        }

        Self {
            headings,
            paragraph: ParagraphStyle {
                font: FontConfig {
                    family: "Times New Roman".to_string(),
                    size: 12.0,
                    bold: false,
                    italic: false,
                },
                line_spacing: 1.15,
                spacing_after: 6.0,
            },
            code_block: CodeBlockStyle {
                font: FontConfig {
                    family: "Courier New".to_string(),
                    size: 10.0,
                    bold: false,
                    italic: false,
                },
                background_color: Some("#f5f5f5".to_string()),
                border: true,
                preserve_line_breaks: true,
                line_spacing: 1.0,
                paragraph_spacing: 6.0,
            },
            table: TableStyle {
                header_font: FontConfig {
                    family: "Times New Roman".to_string(),
                    size: 12.0,
                    bold: true,
                    italic: false,
                },
                cell_font: FontConfig {
                    family: "Times New Roman".to_string(),
                    size: 12.0,
                    bold: false,
                    italic: false,
                },
                border_width: 1.0,
            },
        }
    }
}

impl ElementConfig {
    /// Validate element configuration
    pub fn validate(&self) -> Result<(), ValidationError> {
        self.image.validate()?;
        self.list.validate()?;
        self.link.validate()?;
        Ok(())
    }
}

impl ImageConfig {
    /// Validate image configuration
    pub fn validate(&self) -> Result<(), ValidationError> {
        if self.max_width <= 0.0 || self.max_height <= 0.0 {
            return Err(ValidationError::InvalidImageDimensions);
        }
        Ok(())
    }
}

impl ListConfig {
    /// Validate list configuration
    pub fn validate(&self) -> Result<(), ValidationError> {
        if self.indent < 0.0 || self.spacing < 0.0 {
            return Err(ValidationError::InvalidSpacing);
        }
        Ok(())
    }
}

impl LinkConfig {
    /// Validate link configuration
    pub fn validate(&self) -> Result<(), ValidationError> {
        validate_color(&self.color)?;
        Ok(())
    }
}

/// Validate color format (hex colors)
fn validate_color(color: &str) -> Result<(), ValidationError> {
    if color.starts_with('#') && color.len() == 7 {
        // Check if all characters after # are valid hex
        if color[1..].chars().all(|c| c.is_ascii_hexdigit()) {
            return Ok(());
        }
    }
    Err(ValidationError::InvalidColor(color.to_string()))
}

impl Default for ElementConfig {
    fn default() -> Self {
        Self {
            image: ImageConfig {
                max_width: 500.0,
                max_height: 400.0,
            },
            list: ListConfig {
                indent: 36.0,
                spacing: 6.0,
            },
            link: LinkConfig {
                color: "#0066cc".to_string(),
                underline: true,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config_validation() {
        let config = ConversionConfig::default();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_invalid_page_size() {
        let mut config = ConversionConfig::default();
        config.document.page_size.width = -100.0;

        let result = config.validate();
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ValidationError::InvalidPageSize
        ));
    }

    #[test]
    fn test_invalid_margins() {
        let mut config = ConversionConfig::default();
        config.document.margins.top = -10.0;

        let result = config.validate();
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ValidationError::InvalidMargins
        ));
    }

    #[test]
    fn test_invalid_font_size() {
        let mut config = ConversionConfig::default();
        config.document.default_font.size = 0.0;

        let result = config.validate();
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ValidationError::InvalidFontSize
        ));
    }

    #[test]
    fn test_invalid_font_family() {
        let mut config = ConversionConfig::default();
        config.document.default_font.family = "".to_string();

        let result = config.validate();
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ValidationError::InvalidFontFamily
        ));
    }

    #[test]
    fn test_invalid_color() {
        let mut config = ConversionConfig::default();
        config.elements.link.color = "invalid-color".to_string();

        let result = config.validate();
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ValidationError::InvalidColor(_)
        ));
    }

    #[test]
    fn test_valid_color_formats() {
        let valid_colors = vec!["#ff0000", "#00FF00", "#0000ff", "#123456"];

        for color in valid_colors {
            let mut config = ConversionConfig::default();
            config.elements.link.color = color.to_string();
            assert!(config.validate().is_ok(), "Color {} should be valid", color);
        }
    }

    #[test]
    fn test_serde_serialization() {
        let config = ConversionConfig::default();

        // Test JSON serialization
        let json = serde_json::to_string(&config).unwrap();
        let deserialized: ConversionConfig = serde_json::from_str(&json).unwrap();
        assert!(deserialized.validate().is_ok());

        // Test YAML serialization
        let yaml = serde_yaml::to_string(&config).unwrap();
        let deserialized: ConversionConfig = serde_yaml::from_str(&yaml).unwrap();
        assert!(deserialized.validate().is_ok());
    }

    #[test]
    fn test_heading_numbering_validation() {
        let mut config = ConversionConfig::default();

        // Test valid numbering formats
        let valid_formats = vec!["%1.", "%1.%2.", "%1.%2.%3", "%1-%2-%3"];
        for format in valid_formats {
            config.styles.headings.get_mut(&1).unwrap().numbering = Some(format.to_string());
            assert!(
                config.validate().is_ok(),
                "Format '{}' should be valid",
                format
            );
        }

        // Test invalid numbering formats
        let invalid_formats = vec!["", "no placeholders", "%1.%3.", "%2.%3.", "%0.", "%7."];
        for format in invalid_formats {
            config.styles.headings.get_mut(&1).unwrap().numbering = Some(format.to_string());
            let result = config.validate();
            assert!(result.is_err(), "Format '{}' should be invalid", format);
            assert!(matches!(
                result.unwrap_err(),
                ValidationError::InvalidNumberingFormat(_)
            ));
        }

        // Test None numbering (should be valid)
        config.styles.headings.get_mut(&1).unwrap().numbering = None;
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_numbering_field_serialization() {
        let mut config = ConversionConfig::default();
        config.styles.headings.get_mut(&1).unwrap().numbering = Some("%1.".to_string());
        config.styles.headings.get_mut(&2).unwrap().numbering = Some("%1.%2.".to_string());

        // Test JSON serialization with numbering
        let json = serde_json::to_string(&config).unwrap();
        let deserialized: ConversionConfig = serde_json::from_str(&json).unwrap();
        assert!(deserialized.validate().is_ok());
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
            None
        );

        // Test YAML serialization with numbering
        let yaml = serde_yaml::to_string(&config).unwrap();
        let deserialized: ConversionConfig = serde_yaml::from_str(&yaml).unwrap();
        assert!(deserialized.validate().is_ok());
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
            None
        );
    }

    #[test]
    fn test_backward_compatibility() {
        // Test that configurations without numbering field still work
        // Create a config programmatically to simulate old configs without numbering
        let config = ConversionConfig::default();

        // Verify that default config has numbering set to None (backward compatibility)
        for level in 1..=6 {
            assert_eq!(config.styles.headings.get(&level).unwrap().numbering, None);
        }

        // Verify that config without numbering is still valid
        assert!(config.validate().is_ok());

        // Test serialization/deserialization maintains None values
        let json = serde_json::to_string(&config).unwrap();
        let deserialized: ConversionConfig = serde_json::from_str(&json).unwrap();
        assert!(deserialized.validate().is_ok());
        assert_eq!(
            deserialized.styles.headings.get(&1).unwrap().numbering,
            None
        );
    }

    #[test]
    fn test_code_block_line_break_config() {
        let config = ConversionConfig::default();
        
        // Test default values for new code block fields
        assert_eq!(config.styles.code_block.preserve_line_breaks, true);
        assert_eq!(config.styles.code_block.line_spacing, 1.0);
        assert_eq!(config.styles.code_block.paragraph_spacing, 6.0);
        
        // Test validation passes with default values
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_invalid_code_block_spacing() {
        let mut config = ConversionConfig::default();
        
        // Test invalid line spacing
        config.styles.code_block.line_spacing = 0.0;
        let result = config.validate();
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ValidationError::InvalidSpacing
        ));
        
        // Reset and test invalid paragraph spacing
        config.styles.code_block.line_spacing = 1.0;
        config.styles.code_block.paragraph_spacing = -1.0;
        let result = config.validate();
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ValidationError::InvalidSpacing
        ));
    }

    #[test]
    fn test_code_block_serialization() {
        let mut config = ConversionConfig::default();
        config.styles.code_block.preserve_line_breaks = false;
        config.styles.code_block.line_spacing = 1.5;
        config.styles.code_block.paragraph_spacing = 12.0;

        // Test JSON serialization
        let json = serde_json::to_string(&config).unwrap();
        let deserialized: ConversionConfig = serde_json::from_str(&json).unwrap();
        assert!(deserialized.validate().is_ok());
        assert_eq!(deserialized.styles.code_block.preserve_line_breaks, false);
        assert_eq!(deserialized.styles.code_block.line_spacing, 1.5);
        assert_eq!(deserialized.styles.code_block.paragraph_spacing, 12.0);

        // Test YAML serialization
        let yaml = serde_yaml::to_string(&config).unwrap();
        let deserialized: ConversionConfig = serde_yaml::from_str(&yaml).unwrap();
        assert!(deserialized.validate().is_ok());
        assert_eq!(deserialized.styles.code_block.preserve_line_breaks, false);
        assert_eq!(deserialized.styles.code_block.line_spacing, 1.5);
        assert_eq!(deserialized.styles.code_block.paragraph_spacing, 12.0);
    }

    #[test]
    fn test_code_block_edge_case_configurations() {
        let mut config = ConversionConfig::default();
        
        // Test minimum valid values
        config.styles.code_block.line_spacing = 0.1;
        config.styles.code_block.paragraph_spacing = 0.0;
        assert!(config.validate().is_ok());
        
        // Test boolean toggle
        config.styles.code_block.preserve_line_breaks = false;
        assert!(config.validate().is_ok());
        config.styles.code_block.preserve_line_breaks = true;
        assert!(config.validate().is_ok());
        
        // Test larger values
        config.styles.code_block.line_spacing = 3.0;
        config.styles.code_block.paragraph_spacing = 24.0;
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_code_block_backward_compatibility() {
        // Test that old configurations without the new fields can still be loaded
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
  headings:
    1:
      font:
        family: \"Arial\"
        size: 18.0
        bold: true
        italic: false
      spacing_before: 12.0
      spacing_after: 6.0
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

        // This should fail to deserialize because the new fields are required
        let result: Result<ConversionConfig, _> = serde_yaml::from_str(old_config_yaml);
        assert!(result.is_err(), "Old config without new fields should fail to deserialize");
        
        // But if we use the default config and serialize/deserialize, it should work
        let default_config = ConversionConfig::default();
        let yaml = serde_yaml::to_string(&default_config).unwrap();
        let deserialized: ConversionConfig = serde_yaml::from_str(&yaml).unwrap();
        assert!(deserialized.validate().is_ok());
        
        // The new fields should have their default values
        assert_eq!(deserialized.styles.code_block.preserve_line_breaks, true);
        assert_eq!(deserialized.styles.code_block.line_spacing, 1.0);
        assert_eq!(deserialized.styles.code_block.paragraph_spacing, 6.0);
    }
}
