//! Configuration data models

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
}

impl HeadingStyle {
    /// Validate heading style
    pub fn validate(&self) -> Result<(), ValidationError> {
        self.font.validate()?;
        if self.spacing_before < 0.0 || self.spacing_after < 0.0 {
            return Err(ValidationError::InvalidSpacing);
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
}

impl CodeBlockStyle {
    /// Validate code block style
    pub fn validate(&self) -> Result<(), ValidationError> {
        self.font.validate()?;
        if let Some(color) = &self.background_color {
            validate_color(color)?;
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
            
            headings.insert(level, HeadingStyle {
                font: FontConfig {
                    family: "Times New Roman".to_string(),
                    size,
                    bold: true,
                    italic: false,
                },
                spacing_before: 12.0,
                spacing_after: 6.0,
            });
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
        assert!(matches!(result.unwrap_err(), ValidationError::InvalidPageSize));
    }

    #[test]
    fn test_invalid_margins() {
        let mut config = ConversionConfig::default();
        config.document.margins.top = -10.0;
        
        let result = config.validate();
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ValidationError::InvalidMargins));
    }

    #[test]
    fn test_invalid_font_size() {
        let mut config = ConversionConfig::default();
        config.document.default_font.size = 0.0;
        
        let result = config.validate();
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ValidationError::InvalidFontSize));
    }

    #[test]
    fn test_invalid_font_family() {
        let mut config = ConversionConfig::default();
        config.document.default_font.family = "".to_string();
        
        let result = config.validate();
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ValidationError::InvalidFontFamily));
    }

    #[test]
    fn test_invalid_color() {
        let mut config = ConversionConfig::default();
        config.elements.link.color = "invalid-color".to_string();
        
        let result = config.validate();
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ValidationError::InvalidColor(_)));
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
}