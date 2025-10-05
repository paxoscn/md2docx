//! Style application utilities for docx generation

use crate::config::{ConversionConfig, FontConfig, HeadingStyle, ParagraphStyle};

/// Utility functions for applying styles to docx elements
pub struct StyleApplicator {
    config: ConversionConfig,
}

impl StyleApplicator {
    /// Create a new style applicator
    pub fn new(config: ConversionConfig) -> Self {
        Self { config }
    }

    /// Get heading style for the given level
    pub fn get_heading_style(&self, level: u8) -> Option<&HeadingStyle> {
        self.config.styles.headings.get(&level)
    }

    /// Get paragraph style
    pub fn get_paragraph_style(&self) -> &ParagraphStyle {
        &self.config.styles.paragraph
    }

    /// Get default font configuration
    pub fn get_default_font(&self) -> &FontConfig {
        &self.config.document.default_font
    }
}