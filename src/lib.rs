//! # Markdown to docx Converter
//! 
//! A configurable Markdown to docx converter that supports:
//! - YAML-based configuration for formatting rules
//! - Natural language configuration updates via LLM integration
//! - Web API, CLI, and web interface
//! - Batch processing capabilities

pub mod config;
pub mod conversion;
pub mod error;
pub mod llm;
pub mod markdown;
pub mod docx;
pub mod web;
pub mod logging;

#[cfg(test)]
pub mod test_utils;

#[cfg(test)]
pub mod test_coverage;

// Re-export main types for convenience
pub use config::{ConversionConfig, ConfigurationService};
pub use conversion::ConversionEngine;
pub use error::{ConversionError, ConfigError, WebError, CliError};
pub use markdown::MarkdownParser;
pub use docx::DocxGenerator;
pub use logging::{LoggingConfig, init_logging};

/// Result type alias for the library
pub type Result<T> = std::result::Result<T, ConversionError>;