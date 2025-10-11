//! Error types for the Markdown to docx converter

use thiserror::Error;
use std::fmt;

/// Main error type for conversion operations
#[derive(Debug, Error)]
pub enum ConversionError {
    #[error("Markdown parsing failed: {0}")]
    MarkdownParsing(String),
    
    #[error("Docx generation failed: {0}")]
    DocxGeneration(String),
    
    #[error("Configuration error: {0}")]
    Configuration(#[from] ConfigError),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    
    #[error("File processing error: {0}")]
    FileProcessing(String),
    
    #[error("Batch processing error: {processed} of {total} files processed successfully")]
    BatchProcessing { processed: usize, total: usize },
    
    #[error("Resource limit exceeded: {0}")]
    ResourceLimit(String),
    
    #[error("Timeout error: {0}")]
    Timeout(String),
    
    #[error("Validation error: {0}")]
    Validation(String),
    
    #[error("Code block processing error: {0}")]
    ProcessingError(String),
}

/// Configuration-specific error types
#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("Invalid YAML format: {0}")]
    InvalidYaml(String),
    
    #[error("LLM API error: {0}")]
    LlmApi(String),
    
    #[error("Validation failed: {0}")]
    Validation(String),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("YAML parsing error: {0}")]
    YamlParsing(#[from] serde_yaml::Error),
    
    #[error("Configuration not found: {0}")]
    NotFound(String),
    
    #[error("Configuration update failed: {0}")]
    UpdateFailed(String),
    
    #[error("Invalid configuration parameter: {parameter} - {reason}")]
    InvalidParameter { parameter: String, reason: String },
    
    #[error("Configuration merge conflict: {0}")]
    MergeConflict(String),
}

/// Web API specific error types
#[derive(Debug, Error)]
pub enum WebError {
    #[error("Invalid request: {0}")]
    InvalidRequest(String),
    
    #[error("Authentication failed: {0}")]
    Authentication(String),
    
    #[error("Rate limit exceeded: {0}")]
    RateLimit(String),
    
    #[error("Request timeout: {0}")]
    Timeout(String),
    
    #[error("Internal server error: {0}")]
    Internal(#[from] ConversionError),
    
    #[error("Bad request: {field} is {issue}")]
    BadRequest { field: String, issue: String },
    
    #[error("File upload error: {0}")]
    FileUpload(String),
}

/// CLI specific error types
#[derive(Debug, Error)]
pub enum CliError {
    #[error("Invalid command line arguments: {0}")]
    InvalidArgs(String),
    
    #[error("File not found: {0}")]
    FileNotFound(String),
    
    #[error("Permission denied: {0}")]
    PermissionDenied(String),
    
    #[error("Conversion failed: {0}")]
    ConversionFailed(#[from] ConversionError),
    
    #[error("Configuration error: {0}")]
    Configuration(#[from] ConfigError),
    
    #[error("Interactive prompt failed: {0}")]
    InteractivePrompt(String),
}

impl ConversionError {
    /// Create a new markdown parsing error
    pub fn markdown_parsing<S: Into<String>>(msg: S) -> Self {
        Self::MarkdownParsing(msg.into())
    }
    
    /// Create a new docx generation error
    pub fn docx_generation<S: Into<String>>(msg: S) -> Self {
        Self::DocxGeneration(msg.into())
    }
    
    /// Create a new file processing error
    pub fn file_processing<S: Into<String>>(msg: S) -> Self {
        Self::FileProcessing(msg.into())
    }
    
    /// Create a new resource limit error
    pub fn resource_limit<S: Into<String>>(msg: S) -> Self {
        Self::ResourceLimit(msg.into())
    }
    
    /// Create a new timeout error
    pub fn timeout<S: Into<String>>(msg: S) -> Self {
        Self::Timeout(msg.into())
    }
    
    /// Create a new validation error
    pub fn validation<S: Into<String>>(msg: S) -> Self {
        Self::Validation(msg.into())
    }
    
    /// Create a new processing error
    pub fn processing_error<S: Into<String>>(msg: S) -> Self {
        Self::ProcessingError(msg.into())
    }
    
    /// Check if this error is recoverable
    pub fn is_recoverable(&self) -> bool {
        match self {
            Self::MarkdownParsing(_) => false,
            Self::DocxGeneration(_) => false,
            Self::Configuration(_) => true,
            Self::Io(_) => true,
            Self::Http(_) => true,
            Self::Serialization(_) => false,
            Self::FileProcessing(_) => true,
            Self::BatchProcessing { .. } => true,
            Self::ResourceLimit(_) => true,
            Self::Timeout(_) => true,
            Self::Validation(_) => true,
            Self::ProcessingError(_) => true,
        }
    }
    
    /// Get error category for logging and metrics
    pub fn category(&self) -> ErrorCategory {
        match self {
            Self::MarkdownParsing(_) => ErrorCategory::Parsing,
            Self::DocxGeneration(_) => ErrorCategory::Generation,
            Self::Configuration(_) => ErrorCategory::Configuration,
            Self::Io(_) => ErrorCategory::Io,
            Self::Http(_) => ErrorCategory::Network,
            Self::Serialization(_) => ErrorCategory::Serialization,
            Self::FileProcessing(_) => ErrorCategory::FileProcessing,
            Self::BatchProcessing { .. } => ErrorCategory::BatchProcessing,
            Self::ResourceLimit(_) => ErrorCategory::Resource,
            Self::Timeout(_) => ErrorCategory::Timeout,
            Self::Validation(_) => ErrorCategory::Validation,
            Self::ProcessingError(_) => ErrorCategory::FileProcessing,
        }
    }
}

impl ConfigError {
    /// Create a new invalid YAML error
    pub fn invalid_yaml<S: Into<String>>(msg: S) -> Self {
        Self::InvalidYaml(msg.into())
    }
    
    /// Create a new LLM API error
    pub fn llm_api<S: Into<String>>(msg: S) -> Self {
        Self::LlmApi(msg.into())
    }
    
    /// Create a new validation error
    pub fn validation<S: Into<String>>(msg: S) -> Self {
        Self::Validation(msg.into())
    }
    
    /// Create a new not found error
    pub fn not_found<S: Into<String>>(msg: S) -> Self {
        Self::NotFound(msg.into())
    }
    
    /// Create a new update failed error
    pub fn update_failed<S: Into<String>>(msg: S) -> Self {
        Self::UpdateFailed(msg.into())
    }
    
    /// Create a new invalid parameter error
    pub fn invalid_parameter<S: Into<String>>(parameter: S, reason: S) -> Self {
        Self::InvalidParameter {
            parameter: parameter.into(),
            reason: reason.into(),
        }
    }
    
    /// Create a new merge conflict error
    pub fn merge_conflict<S: Into<String>>(msg: S) -> Self {
        Self::MergeConflict(msg.into())
    }
}

impl WebError {
    /// Create a new invalid request error
    pub fn invalid_request<S: Into<String>>(msg: S) -> Self {
        Self::InvalidRequest(msg.into())
    }
    
    /// Create a new authentication error
    pub fn authentication<S: Into<String>>(msg: S) -> Self {
        Self::Authentication(msg.into())
    }
    
    /// Create a new rate limit error
    pub fn rate_limit<S: Into<String>>(msg: S) -> Self {
        Self::RateLimit(msg.into())
    }
    
    /// Create a new timeout error
    pub fn timeout<S: Into<String>>(msg: S) -> Self {
        Self::Timeout(msg.into())
    }
    
    /// Create a new bad request error
    pub fn bad_request<S: Into<String>>(field: S, issue: S) -> Self {
        Self::BadRequest {
            field: field.into(),
            issue: issue.into(),
        }
    }
    
    /// Create a new file upload error
    pub fn file_upload<S: Into<String>>(msg: S) -> Self {
        Self::FileUpload(msg.into())
    }
}

impl CliError {
    /// Create a new invalid args error
    pub fn invalid_args<S: Into<String>>(msg: S) -> Self {
        Self::InvalidArgs(msg.into())
    }
    
    /// Create a new file not found error
    pub fn file_not_found<S: Into<String>>(msg: S) -> Self {
        Self::FileNotFound(msg.into())
    }
    
    /// Create a new permission denied error
    pub fn permission_denied<S: Into<String>>(msg: S) -> Self {
        Self::PermissionDenied(msg.into())
    }
    
    /// Create a new interactive prompt error
    pub fn interactive_prompt<S: Into<String>>(msg: S) -> Self {
        Self::InteractivePrompt(msg.into())
    }
}

/// Error categories for logging and metrics
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorCategory {
    Parsing,
    Generation,
    Configuration,
    Io,
    Network,
    Serialization,
    FileProcessing,
    BatchProcessing,
    Resource,
    Timeout,
    Validation,
}

impl fmt::Display for ErrorCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Parsing => write!(f, "parsing"),
            Self::Generation => write!(f, "generation"),
            Self::Configuration => write!(f, "configuration"),
            Self::Io => write!(f, "io"),
            Self::Network => write!(f, "network"),
            Self::Serialization => write!(f, "serialization"),
            Self::FileProcessing => write!(f, "file_processing"),
            Self::BatchProcessing => write!(f, "batch_processing"),
            Self::Resource => write!(f, "resource"),
            Self::Timeout => write!(f, "timeout"),
            Self::Validation => write!(f, "validation"),
        }
    }
}

/// Error context for enhanced debugging
#[derive(Debug, Clone)]
pub struct ErrorContext {
    pub operation: String,
    pub file_path: Option<String>,
    pub line_number: Option<usize>,
    pub additional_info: std::collections::HashMap<String, String>,
}

impl ErrorContext {
    /// Create a new error context
    pub fn new<S: Into<String>>(operation: S) -> Self {
        Self {
            operation: operation.into(),
            file_path: None,
            line_number: None,
            additional_info: std::collections::HashMap::new(),
        }
    }
    
    /// Add file path to context
    pub fn with_file_path<S: Into<String>>(mut self, path: S) -> Self {
        self.file_path = Some(path.into());
        self
    }
    
    /// Add line number to context
    pub fn with_line_number(mut self, line: usize) -> Self {
        self.line_number = Some(line);
        self
    }
    
    /// Add additional information
    pub fn with_info<K: Into<String>, V: Into<String>>(mut self, key: K, value: V) -> Self {
        self.additional_info.insert(key.into(), value.into());
        self
    }
}

/// Result type alias for conversion operations
pub type ConversionResult<T> = Result<T, ConversionError>;

/// Result type alias for configuration operations
pub type ConfigResult<T> = Result<T, ConfigError>;

/// Result type alias for web operations
pub type WebResult<T> = Result<T, WebError>;

/// Result type alias for CLI operations
pub type CliResult<T> = Result<T, CliError>;
#[cfg(test)]
mod tests {
    use super::*;
    use std::io;

    #[test]
    fn test_conversion_error_creation() {
        let error = ConversionError::markdown_parsing("Test parsing error");
        assert!(matches!(error, ConversionError::MarkdownParsing(_)));
        assert_eq!(error.to_string(), "Markdown parsing failed: Test parsing error");
    }

    #[test]
    fn test_conversion_error_recoverable() {
        assert!(!ConversionError::markdown_parsing("test").is_recoverable());
        assert!(!ConversionError::docx_generation("test").is_recoverable());
        assert!(ConversionError::file_processing("test").is_recoverable());
        assert!(ConversionError::timeout("test").is_recoverable());
    }

    #[test]
    fn test_conversion_error_category() {
        assert_eq!(
            ConversionError::markdown_parsing("test").category(),
            ErrorCategory::Parsing
        );
        assert_eq!(
            ConversionError::docx_generation("test").category(),
            ErrorCategory::Generation
        );
        assert_eq!(
            ConversionError::file_processing("test").category(),
            ErrorCategory::FileProcessing
        );
    }

    #[test]
    fn test_config_error_creation() {
        let error = ConfigError::invalid_yaml("Invalid syntax");
        assert!(matches!(error, ConfigError::InvalidYaml(_)));
        assert_eq!(error.to_string(), "Invalid YAML format: Invalid syntax");
        
        let error = ConfigError::invalid_parameter("font_size", "must be positive");
        assert!(matches!(error, ConfigError::InvalidParameter { .. }));
        assert_eq!(error.to_string(), "Invalid configuration parameter: font_size - must be positive");
    }

    #[test]
    fn test_web_error_creation() {
        let error = WebError::bad_request("markdown", "missing");
        assert!(matches!(error, WebError::BadRequest { .. }));
        assert_eq!(error.to_string(), "Bad request: markdown is missing");
        
        let error = WebError::rate_limit("Too many requests");
        assert!(matches!(error, WebError::RateLimit(_)));
        assert_eq!(error.to_string(), "Rate limit exceeded: Too many requests");
    }

    #[test]
    fn test_cli_error_creation() {
        let error = CliError::file_not_found("/path/to/file.md");
        assert!(matches!(error, CliError::FileNotFound(_)));
        assert_eq!(error.to_string(), "File not found: /path/to/file.md");
        
        let error = CliError::permission_denied("Cannot write to output directory");
        assert!(matches!(error, CliError::PermissionDenied(_)));
        assert_eq!(error.to_string(), "Permission denied: Cannot write to output directory");
    }

    #[test]
    fn test_error_category_display() {
        assert_eq!(ErrorCategory::Parsing.to_string(), "parsing");
        assert_eq!(ErrorCategory::Generation.to_string(), "generation");
        assert_eq!(ErrorCategory::Configuration.to_string(), "configuration");
        assert_eq!(ErrorCategory::Io.to_string(), "io");
        assert_eq!(ErrorCategory::Network.to_string(), "network");
    }

    #[test]
    fn test_error_context() {
        let mut context = ErrorContext::new("convert_file")
            .with_file_path("/path/to/file.md")
            .with_line_number(42)
            .with_info("file_size", "1024")
            .with_info("encoding", "utf-8");
        
        assert_eq!(context.operation, "convert_file");
        assert_eq!(context.file_path, Some("/path/to/file.md".to_string()));
        assert_eq!(context.line_number, Some(42));
        assert_eq!(context.additional_info.get("file_size"), Some(&"1024".to_string()));
        assert_eq!(context.additional_info.get("encoding"), Some(&"utf-8".to_string()));
    }

    #[test]
    fn test_error_from_conversions() {
        // Test ConfigError -> ConversionError conversion
        let config_error = ConfigError::validation("Invalid config");
        let conversion_error: ConversionError = config_error.into();
        assert!(matches!(conversion_error, ConversionError::Configuration(_)));
        
        // Test ConversionError -> WebError conversion
        let conversion_error = ConversionError::markdown_parsing("Parse error");
        let web_error: WebError = conversion_error.into();
        assert!(matches!(web_error, WebError::Internal(_)));
        
        // Test ConversionError -> CliError conversion
        let conversion_error = ConversionError::file_processing("File error");
        let cli_error: CliError = conversion_error.into();
        assert!(matches!(cli_error, CliError::ConversionFailed(_)));
    }

    #[test]
    fn test_batch_processing_error() {
        let error = ConversionError::BatchProcessing { processed: 3, total: 5 };
        assert_eq!(error.to_string(), "Batch processing error: 3 of 5 files processed successfully");
        assert!(error.is_recoverable());
        assert_eq!(error.category(), ErrorCategory::BatchProcessing);
    }

    #[test]
    fn test_io_error_conversion() {
        let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "File not found");
        let conversion_error: ConversionError = io_error.into();
        assert!(matches!(conversion_error, ConversionError::Io(_)));
        assert!(conversion_error.is_recoverable());
    }

    #[test]
    fn test_serde_error_conversion() {
        let json_str = r#"{"invalid": json"#;
        let serde_error = serde_json::from_str::<serde_json::Value>(json_str).unwrap_err();
        let conversion_error: ConversionError = serde_error.into();
        assert!(matches!(conversion_error, ConversionError::Serialization(_)));
        assert!(!conversion_error.is_recoverable());
    }
}