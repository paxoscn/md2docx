//! Error and warning types for code block processing

use std::fmt;

/// Represents different types of processing errors
#[derive(Debug, Clone)]
pub struct ProcessingError {
    pub error_type: String,
    pub message: String,
    pub line: Option<usize>,
    pub column: Option<usize>,
    pub severity: ErrorSeverity,
}

/// Represents processing warnings
#[derive(Debug, Clone)]
pub struct ProcessingWarning {
    pub warning_type: String,
    pub message: String,
    pub line: Option<usize>,
    pub column: Option<usize>,
}

/// Error severity levels
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ErrorSeverity {
    Low,
    Medium,
    High,
    Critical,
}

impl ProcessingError {
    /// Create a new processing error
    pub fn new(error_type: &str, message: &str) -> Self {
        Self {
            error_type: error_type.to_string(),
            message: message.to_string(),
            line: None,
            column: None,
            severity: ErrorSeverity::Medium,
        }
    }

    /// Create a syntax error
    pub fn syntax_error(message: &str, line: Option<usize>, column: Option<usize>) -> Self {
        Self {
            error_type: "syntax_error".to_string(),
            message: message.to_string(),
            line,
            column,
            severity: ErrorSeverity::High,
        }
    }

    /// Create a formatting error
    pub fn formatting_error(message: &str) -> Self {
        Self {
            error_type: "formatting_error".to_string(),
            message: message.to_string(),
            line: None,
            column: None,
            severity: ErrorSeverity::Low,
        }
    }

    /// Create a timeout error
    pub fn timeout() -> Self {
        Self {
            error_type: "timeout".to_string(),
            message: "Processing timed out".to_string(),
            line: None,
            column: None,
            severity: ErrorSeverity::Critical,
        }
    }

    /// Create a timeout error with custom message
    pub fn timeout_with_message(message: &str) -> Self {
        Self {
            error_type: "timeout".to_string(),
            message: message.to_string(),
            line: None,
            column: None,
            severity: ErrorSeverity::Critical,
        }
    }

    /// Create a validation error
    pub fn validation_error(message: &str) -> Self {
        Self {
            error_type: "validation_error".to_string(),
            message: message.to_string(),
            line: None,
            column: None,
            severity: ErrorSeverity::High,
        }
    }

    /// Create a system overload error
    pub fn system_overload(message: &str) -> Self {
        Self {
            error_type: "system_overload".to_string(),
            message: message.to_string(),
            line: None,
            column: None,
            severity: ErrorSeverity::High,
        }
    }

    /// Create a dependency error (e.g., missing external tools)
    pub fn dependency_error(message: &str) -> Self {
        Self {
            error_type: "dependency_error".to_string(),
            message: message.to_string(),
            line: None,
            column: None,
            severity: ErrorSeverity::Medium,
        }
    }

    /// Create a configuration error
    pub fn configuration_error(message: &str) -> Self {
        Self {
            error_type: "configuration_error".to_string(),
            message: message.to_string(),
            line: None,
            column: None,
            severity: ErrorSeverity::Medium,
        }
    }

    /// Create an internal error (unexpected system error)
    pub fn internal_error(message: &str) -> Self {
        Self {
            error_type: "internal_error".to_string(),
            message: message.to_string(),
            line: None,
            column: None,
            severity: ErrorSeverity::Critical,
        }
    }

    /// Create a resource exhaustion error
    pub fn resource_exhaustion(message: &str) -> Self {
        Self {
            error_type: "resource_exhaustion".to_string(),
            message: message.to_string(),
            line: None,
            column: None,
            severity: ErrorSeverity::High,
        }
    }

    /// Set the line and column information
    pub fn with_position(mut self, line: usize, column: usize) -> Self {
        self.line = Some(line);
        self.column = Some(column);
        self
    }

    /// Set the severity level
    pub fn with_severity(mut self, severity: ErrorSeverity) -> Self {
        self.severity = severity;
        self
    }
}

impl ProcessingWarning {
    /// Create a new processing warning
    pub fn new(warning_type: &str, message: &str) -> Self {
        Self {
            warning_type: warning_type.to_string(),
            message: message.to_string(),
            line: None,
            column: None,
        }
    }

    /// Create a formatting warning
    pub fn formatting_warning(message: &str) -> Self {
        Self {
            warning_type: "formatting_warning".to_string(),
            message: message.to_string(),
            line: None,
            column: None,
        }
    }

    /// Create a deprecation warning
    pub fn deprecation_warning(message: &str) -> Self {
        Self {
            warning_type: "deprecation_warning".to_string(),
            message: message.to_string(),
            line: None,
            column: None,
        }
    }

    /// Create a performance warning
    pub fn performance_warning(message: &str) -> Self {
        Self {
            warning_type: "performance_warning".to_string(),
            message: message.to_string(),
            line: None,
            column: None,
        }
    }

    /// Create a compatibility warning
    pub fn compatibility_warning(message: &str) -> Self {
        Self {
            warning_type: "compatibility_warning".to_string(),
            message: message.to_string(),
            line: None,
            column: None,
        }
    }

    /// Create a fallback warning (when fallback strategy was used)
    pub fn fallback_warning(message: &str) -> Self {
        Self {
            warning_type: "fallback_warning".to_string(),
            message: message.to_string(),
            line: None,
            column: None,
        }
    }

    /// Create a timeout warning (processing took longer than expected but didn't timeout)
    pub fn timeout_warning(message: &str) -> Self {
        Self {
            warning_type: "timeout_warning".to_string(),
            message: message.to_string(),
            line: None,
            column: None,
        }
    }

    /// Create a validation warning (minor validation issues)
    pub fn validation_warning(message: &str) -> Self {
        Self {
            warning_type: "validation_warning".to_string(),
            message: message.to_string(),
            line: None,
            column: None,
        }
    }

    /// Set the line and column information
    pub fn with_position(mut self, line: usize, column: usize) -> Self {
        self.line = Some(line);
        self.column = Some(column);
        self
    }
}

impl fmt::Display for ProcessingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match (self.line, self.column) {
            (Some(line), Some(column)) => {
                write!(f, "{} at line {}, column {}: {}", 
                       self.error_type, line, column, self.message)
            }
            (Some(line), None) => {
                write!(f, "{} at line {}: {}", 
                       self.error_type, line, self.message)
            }
            _ => {
                write!(f, "{}: {}", self.error_type, self.message)
            }
        }
    }
}

impl fmt::Display for ProcessingWarning {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match (self.line, self.column) {
            (Some(line), Some(column)) => {
                write!(f, "{} at line {}, column {}: {}", 
                       self.warning_type, line, column, self.message)
            }
            (Some(line), None) => {
                write!(f, "{} at line {}: {}", 
                       self.warning_type, line, self.message)
            }
            _ => {
                write!(f, "{}: {}", self.warning_type, self.message)
            }
        }
    }
}

impl std::error::Error for ProcessingError {}

/// Error report containing detailed information about processing failures
#[derive(Debug, Clone)]
pub struct ErrorReport {
    pub errors: Vec<ProcessingError>,
    pub warnings: Vec<ProcessingWarning>,
    pub processing_time: std::time::Duration,
    pub recovery_attempts: usize,
    pub final_strategy_used: String,
    pub context: ErrorContext,
}

/// Context information for error reporting
#[derive(Debug, Clone)]
pub struct ErrorContext {
    pub language: Option<String>,
    pub code_length: usize,
    pub config_summary: String,
    pub timestamp: std::time::SystemTime,
    pub processor_version: String,
}

impl ErrorReport {
    /// Create a new error report
    pub fn new(context: ErrorContext) -> Self {
        Self {
            errors: Vec::new(),
            warnings: Vec::new(),
            processing_time: std::time::Duration::from_millis(0),
            recovery_attempts: 0,
            final_strategy_used: "unknown".to_string(),
            context,
        }
    }

    /// Add an error to the report
    pub fn add_error(&mut self, error: ProcessingError) {
        self.errors.push(error);
    }

    /// Add a warning to the report
    pub fn add_warning(&mut self, warning: ProcessingWarning) {
        self.warnings.push(warning);
    }

    /// Set processing time
    pub fn set_processing_time(&mut self, duration: std::time::Duration) {
        self.processing_time = duration;
    }

    /// Increment recovery attempts
    pub fn increment_recovery_attempts(&mut self) {
        self.recovery_attempts += 1;
    }

    /// Set the final strategy that was used
    pub fn set_final_strategy(&mut self, strategy_name: &str) {
        self.final_strategy_used = strategy_name.to_string();
    }

    /// Check if there are any critical errors
    pub fn has_critical_errors(&self) -> bool {
        self.errors.iter().any(|e| e.severity == ErrorSeverity::Critical)
    }

    /// Check if there are any high severity errors
    pub fn has_high_severity_errors(&self) -> bool {
        self.errors.iter().any(|e| matches!(e.severity, ErrorSeverity::High | ErrorSeverity::Critical))
    }

    /// Get total error count
    pub fn total_error_count(&self) -> usize {
        self.errors.len()
    }

    /// Get total warning count
    pub fn total_warning_count(&self) -> usize {
        self.warnings.len()
    }

    /// Get errors by severity
    pub fn get_errors_by_severity(&self, severity: ErrorSeverity) -> Vec<&ProcessingError> {
        self.errors.iter().filter(|e| e.severity == severity).collect()
    }

    /// Get a summary of the error report
    pub fn get_summary(&self) -> ErrorSummary {
        ErrorSummary {
            total_errors: self.total_error_count(),
            total_warnings: self.total_warning_count(),
            critical_errors: self.get_errors_by_severity(ErrorSeverity::Critical).len(),
            high_errors: self.get_errors_by_severity(ErrorSeverity::High).len(),
            medium_errors: self.get_errors_by_severity(ErrorSeverity::Medium).len(),
            low_errors: self.get_errors_by_severity(ErrorSeverity::Low).len(),
            processing_time: self.processing_time,
            recovery_attempts: self.recovery_attempts,
            final_strategy: self.final_strategy_used.clone(),
        }
    }

    /// Generate a human-readable report
    pub fn generate_report(&self) -> String {
        let mut report = String::new();
        
        report.push_str(&format!("Error Report for {} Processing\n", 
            self.context.language.as_deref().unwrap_or("unknown")));
        report.push_str(&format!("Timestamp: {:?}\n", self.context.timestamp));
        report.push_str(&format!("Code Length: {} characters\n", self.context.code_length));
        report.push_str(&format!("Processing Time: {:?}\n", self.processing_time));
        report.push_str(&format!("Recovery Attempts: {}\n", self.recovery_attempts));
        report.push_str(&format!("Final Strategy: {}\n", self.final_strategy_used));
        report.push_str(&format!("Processor Version: {}\n\n", self.context.processor_version));

        if !self.errors.is_empty() {
            report.push_str("ERRORS:\n");
            for (i, error) in self.errors.iter().enumerate() {
                report.push_str(&format!("  {}. [{:?}] {}\n", i + 1, error.severity, error));
            }
            report.push('\n');
        }

        if !self.warnings.is_empty() {
            report.push_str("WARNINGS:\n");
            for (i, warning) in self.warnings.iter().enumerate() {
                report.push_str(&format!("  {}. {}\n", i + 1, warning));
            }
            report.push('\n');
        }

        let summary = self.get_summary();
        report.push_str(&format!("SUMMARY: {} errors, {} warnings", 
            summary.total_errors, summary.total_warnings));
        
        if summary.critical_errors > 0 {
            report.push_str(&format!(" ({} critical)", summary.critical_errors));
        }

        report
    }
}

/// Summary of error report
#[derive(Debug, Clone)]
pub struct ErrorSummary {
    pub total_errors: usize,
    pub total_warnings: usize,
    pub critical_errors: usize,
    pub high_errors: usize,
    pub medium_errors: usize,
    pub low_errors: usize,
    pub processing_time: std::time::Duration,
    pub recovery_attempts: usize,
    pub final_strategy: String,
}

impl ErrorContext {
    /// Create a new error context
    pub fn new(
        language: Option<String>,
        code_length: usize,
        config_summary: String,
        processor_version: String,
    ) -> Self {
        Self {
            language,
            code_length,
            config_summary,
            timestamp: std::time::SystemTime::now(),
            processor_version,
        }
    }

    /// Create error context from processing config
    pub fn from_config(
        language: Option<String>,
        code: &str,
        config: &crate::markdown::code_block::ProcessingConfig,
        processor_version: String,
    ) -> Self {
        let config_summary = format!(
            "validation:{}, formatting:{}, timeout:{}ms",
            config.enable_syntax_validation,
            config.enable_formatting,
            config.timeout_ms
        );

        Self::new(language, code.len(), config_summary, processor_version)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_processing_error_creation() {
        let error = ProcessingError::new("test_error", "Test message");
        assert_eq!(error.error_type, "test_error");
        assert_eq!(error.message, "Test message");
        assert_eq!(error.severity, ErrorSeverity::Medium);
        assert!(error.line.is_none());
        assert!(error.column.is_none());
    }

    #[test]
    fn test_syntax_error() {
        let error = ProcessingError::syntax_error("Invalid syntax", Some(10), Some(5));
        assert_eq!(error.error_type, "syntax_error");
        assert_eq!(error.line, Some(10));
        assert_eq!(error.column, Some(5));
        assert_eq!(error.severity, ErrorSeverity::High);
    }

    #[test]
    fn test_error_with_position() {
        let error = ProcessingError::new("test", "message")
            .with_position(5, 10)
            .with_severity(ErrorSeverity::Critical);
        
        assert_eq!(error.line, Some(5));
        assert_eq!(error.column, Some(10));
        assert_eq!(error.severity, ErrorSeverity::Critical);
    }

    #[test]
    fn test_warning_creation() {
        let warning = ProcessingWarning::new("test_warning", "Test warning message");
        assert_eq!(warning.warning_type, "test_warning");
        assert_eq!(warning.message, "Test warning message");
    }

    #[test]
    fn test_error_display() {
        let error = ProcessingError::syntax_error("Missing semicolon", Some(10), Some(5));
        let display = format!("{}", error);
        assert!(display.contains("syntax_error"));
        assert!(display.contains("line 10"));
        assert!(display.contains("column 5"));
        assert!(display.contains("Missing semicolon"));
    }

    #[test]
    fn test_warning_display() {
        let warning = ProcessingWarning::formatting_warning("Inconsistent indentation")
            .with_position(15, 0);
        let display = format!("{}", warning);
        assert!(display.contains("formatting_warning"));
        assert!(display.contains("line 15"));
        assert!(display.contains("Inconsistent indentation"));
    }

    #[test]
    fn test_additional_error_types() {
        let timeout_error = ProcessingError::timeout_with_message("Custom timeout message");
        assert_eq!(timeout_error.error_type, "timeout");
        assert_eq!(timeout_error.message, "Custom timeout message");
        assert_eq!(timeout_error.severity, ErrorSeverity::Critical);

        let system_error = ProcessingError::system_overload("Too many requests");
        assert_eq!(system_error.error_type, "system_overload");
        assert_eq!(system_error.severity, ErrorSeverity::High);

        let dependency_error = ProcessingError::dependency_error("Missing rustfmt");
        assert_eq!(dependency_error.error_type, "dependency_error");
        assert_eq!(dependency_error.severity, ErrorSeverity::Medium);

        let config_error = ProcessingError::configuration_error("Invalid config");
        assert_eq!(config_error.error_type, "configuration_error");

        let internal_error = ProcessingError::internal_error("Unexpected panic");
        assert_eq!(internal_error.error_type, "internal_error");
        assert_eq!(internal_error.severity, ErrorSeverity::Critical);

        let resource_error = ProcessingError::resource_exhaustion("Out of memory");
        assert_eq!(resource_error.error_type, "resource_exhaustion");
        assert_eq!(resource_error.severity, ErrorSeverity::High);
    }

    #[test]
    fn test_additional_warning_types() {
        let perf_warning = ProcessingWarning::performance_warning("Slow processing");
        assert_eq!(perf_warning.warning_type, "performance_warning");

        let compat_warning = ProcessingWarning::compatibility_warning("Old syntax");
        assert_eq!(compat_warning.warning_type, "compatibility_warning");

        let fallback_warning = ProcessingWarning::fallback_warning("Used default strategy");
        assert_eq!(fallback_warning.warning_type, "fallback_warning");

        let timeout_warning = ProcessingWarning::timeout_warning("Processing took 4.5s");
        assert_eq!(timeout_warning.warning_type, "timeout_warning");

        let validation_warning = ProcessingWarning::validation_warning("Minor syntax issue");
        assert_eq!(validation_warning.warning_type, "validation_warning");
    }

    #[test]
    fn test_error_context_creation() {
        use crate::markdown::code_block::ProcessingConfig;
        
        let context = ErrorContext::new(
            Some("rust".to_string()),
            100,
            "test config".to_string(),
            "1.0.0".to_string(),
        );

        assert_eq!(context.language, Some("rust".to_string()));
        assert_eq!(context.code_length, 100);
        assert_eq!(context.config_summary, "test config");
        assert_eq!(context.processor_version, "1.0.0");

        // Test from_config method
        let config = ProcessingConfig::default();
        let context2 = ErrorContext::from_config(
            Some("rust".to_string()),
            "fn main() {}",
            &config,
            "2.0.0".to_string(),
        );
        
        assert_eq!(context2.language, Some("rust".to_string()));
        assert_eq!(context2.code_length, 12);
        assert!(context2.config_summary.contains("validation:true"));
        assert_eq!(context2.processor_version, "2.0.0");
    }

    #[test]
    fn test_error_report_creation() {
        let context = ErrorContext::new(
            Some("rust".to_string()),
            50,
            "test config".to_string(),
            "1.0.0".to_string(),
        );

        let mut report = ErrorReport::new(context);
        
        assert_eq!(report.total_error_count(), 0);
        assert_eq!(report.total_warning_count(), 0);
        assert!(!report.has_critical_errors());
        assert!(!report.has_high_severity_errors());

        // Add errors and warnings
        report.add_error(ProcessingError::syntax_error("Invalid syntax", Some(1), Some(5)));
        report.add_error(ProcessingError::timeout());
        report.add_warning(ProcessingWarning::formatting_warning("Style issue"));

        assert_eq!(report.total_error_count(), 2);
        assert_eq!(report.total_warning_count(), 1);
        assert!(report.has_critical_errors());
        assert!(report.has_high_severity_errors());
    }

    #[test]
    fn test_error_report_summary() {
        let context = ErrorContext::new(
            Some("rust".to_string()),
            50,
            "test config".to_string(),
            "1.0.0".to_string(),
        );

        let mut report = ErrorReport::new(context);
        report.add_error(ProcessingError::timeout()); // Critical
        report.add_error(ProcessingError::syntax_error("Error", None, None)); // High
        report.add_error(ProcessingError::formatting_error("Format error")); // Low
        report.add_warning(ProcessingWarning::performance_warning("Slow"));
        report.set_processing_time(std::time::Duration::from_millis(1500));
        report.increment_recovery_attempts();
        report.set_final_strategy("fallback");

        let summary = report.get_summary();
        assert_eq!(summary.total_errors, 3);
        assert_eq!(summary.total_warnings, 1);
        assert_eq!(summary.critical_errors, 1);
        assert_eq!(summary.high_errors, 1);
        assert_eq!(summary.low_errors, 1);
        assert_eq!(summary.medium_errors, 0);
        assert_eq!(summary.processing_time, std::time::Duration::from_millis(1500));
        assert_eq!(summary.recovery_attempts, 1);
        assert_eq!(summary.final_strategy, "fallback");
    }

    #[test]
    fn test_error_report_generation() {
        let context = ErrorContext::new(
            Some("rust".to_string()),
            50,
            "test config".to_string(),
            "1.0.0".to_string(),
        );

        let mut report = ErrorReport::new(context);
        report.add_error(ProcessingError::syntax_error("Missing semicolon", Some(5), Some(10)));
        report.add_warning(ProcessingWarning::formatting_warning("Inconsistent indentation"));
        report.set_final_strategy("rust_strategy");

        let report_text = report.generate_report();
        
        assert!(report_text.contains("Error Report for rust Processing"));
        assert!(report_text.contains("Code Length: 50 characters"));
        assert!(report_text.contains("Final Strategy: rust_strategy"));
        assert!(report_text.contains("ERRORS:"));
        assert!(report_text.contains("Missing semicolon"));
        assert!(report_text.contains("WARNINGS:"));
        assert!(report_text.contains("Inconsistent indentation"));
        assert!(report_text.contains("SUMMARY: 1 errors, 1 warnings"));
    }

    #[test]
    fn test_errors_by_severity() {
        let context = ErrorContext::new(None, 0, "".to_string(), "1.0.0".to_string());
        let mut report = ErrorReport::new(context);
        
        report.add_error(ProcessingError::timeout()); // Critical
        report.add_error(ProcessingError::syntax_error("Error", None, None)); // High
        report.add_error(ProcessingError::formatting_error("Format error")); // Low
        
        let critical_errors = report.get_errors_by_severity(ErrorSeverity::Critical);
        assert_eq!(critical_errors.len(), 1);
        assert_eq!(critical_errors[0].error_type, "timeout");
        
        let high_errors = report.get_errors_by_severity(ErrorSeverity::High);
        assert_eq!(high_errors.len(), 1);
        assert_eq!(high_errors[0].error_type, "syntax_error");
        
        let low_errors = report.get_errors_by_severity(ErrorSeverity::Low);
        assert_eq!(low_errors.len(), 1);
        assert_eq!(low_errors[0].error_type, "formatting_error");
    }
}