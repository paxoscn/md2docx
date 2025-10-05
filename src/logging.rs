//! Logging configuration and utilities

use tracing::Level;
use tracing_subscriber::{
    fmt::{self, format::FmtSpan},
    layer::SubscriberExt,
    util::SubscriberInitExt,
    EnvFilter,
};
use std::io;

/// Logging configuration
#[derive(Debug, Clone)]
pub struct LoggingConfig {
    /// Log level (trace, debug, info, warn, error)
    pub level: Level,
    /// Whether to include file and line information
    pub include_location: bool,
    /// Whether to use JSON format for structured logging
    pub json_format: bool,
    /// Whether to include span information
    pub include_spans: bool,
    /// Custom log target filter
    pub target_filter: Option<String>,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: Level::INFO,
            include_location: false,
            json_format: false,
            include_spans: true,
            target_filter: None,
        }
    }
}

impl LoggingConfig {
    /// Create a new logging configuration
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Set the log level
    pub fn with_level(mut self, level: Level) -> Self {
        self.level = level;
        self
    }
    
    /// Enable location information (file and line)
    pub fn with_location(mut self, include: bool) -> Self {
        self.include_location = include;
        self
    }
    
    /// Enable JSON format for structured logging
    pub fn with_json_format(mut self, json: bool) -> Self {
        self.json_format = json;
        self
    }
    
    /// Enable span information
    pub fn with_spans(mut self, include: bool) -> Self {
        self.include_spans = include;
        self
    }
    
    /// Set target filter (e.g., "md2docx_converter" to only show logs from this crate)
    pub fn with_target_filter<S: Into<String>>(mut self, filter: S) -> Self {
        self.target_filter = Some(filter.into());
        self
    }
    
    /// Create configuration from environment variables
    pub fn from_env() -> Self {
        let mut config = Self::default();
        
        // Set level from RUST_LOG or MD2DOCX_LOG_LEVEL
        if let Ok(level_str) = std::env::var("MD2DOCX_LOG_LEVEL")
            .or_else(|_| std::env::var("RUST_LOG")) {
            if let Ok(level) = level_str.parse::<Level>() {
                config.level = level;
            }
        }
        
        // Enable JSON format if MD2DOCX_LOG_JSON is set
        if std::env::var("MD2DOCX_LOG_JSON").is_ok() {
            config.json_format = true;
        }
        
        // Enable location if MD2DOCX_LOG_LOCATION is set
        if std::env::var("MD2DOCX_LOG_LOCATION").is_ok() {
            config.include_location = true;
        }
        
        // Set target filter from MD2DOCX_LOG_TARGET
        if let Ok(target) = std::env::var("MD2DOCX_LOG_TARGET") {
            config.target_filter = Some(target);
        }
        
        config
    }
}

/// Initialize logging with the given configuration
pub fn init_logging(config: LoggingConfig) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let env_filter = create_env_filter(&config)?;
    
    if config.json_format {
        init_json_logging(config, env_filter)
    } else {
        init_pretty_logging(config, env_filter)
    }
}

/// Initialize pretty (human-readable) logging
fn init_pretty_logging(
    config: LoggingConfig,
    env_filter: EnvFilter,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fmt_layer = fmt::layer()
        .with_target(true)
        .with_thread_ids(false)
        .with_thread_names(false)
        .with_file(config.include_location)
        .with_line_number(config.include_location)
        .with_span_events(if config.include_spans {
            FmtSpan::NEW | FmtSpan::CLOSE
        } else {
            FmtSpan::NONE
        })
        .with_writer(io::stderr);
    
    tracing_subscriber::registry()
        .with(env_filter)
        .with(fmt_layer)
        .try_init()?;
    
    Ok(())
}

/// Initialize JSON logging for structured output
fn init_json_logging(
    config: LoggingConfig,
    env_filter: EnvFilter,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fmt_layer = fmt::layer()
        .with_target(true)
        .with_file(config.include_location)
        .with_line_number(config.include_location)
        .with_writer(io::stderr)
        .compact(); // Use compact format instead of json for now
    
    tracing_subscriber::registry()
        .with(env_filter)
        .with(fmt_layer)
        .try_init()?;
    
    Ok(())
}

/// Create environment filter based on configuration
fn create_env_filter(config: &LoggingConfig) -> Result<EnvFilter, Box<dyn std::error::Error + Send + Sync>> {
    let mut filter = EnvFilter::new("");
    
    // Set base level
    let level_str = match config.level {
        Level::TRACE => "trace",
        Level::DEBUG => "debug",
        Level::INFO => "info",
        Level::WARN => "warn",
        Level::ERROR => "error",
    };
    
    // Apply target filter if specified
    if let Some(target) = &config.target_filter {
        filter = filter.add_directive(format!("{}={}", target, level_str).parse()?);
    } else {
        filter = filter.add_directive(level_str.parse()?);
    }
    
    // Allow environment override
    if let Ok(env_filter) = std::env::var("RUST_LOG") {
        filter = EnvFilter::new(env_filter);
    }
    
    Ok(filter)
}

/// Logging utilities for common operations
pub mod utils {
    use tracing::{info, warn, error, debug};
    use crate::error::ConversionError;
    use std::time::{Duration, Instant};
    
    /// Log an error with context
    pub fn log_error(error: &ConversionError, context: &str) {
        let category = error.category();
        let recoverable = error.is_recoverable();
        
        error!(
            error = %error,
            category = %category,
            recoverable = recoverable,
            context = context,
            "Operation failed"
        );
    }
    
    /// Log a warning with context
    pub fn log_warning<S: AsRef<str>>(message: S, context: &str) {
        warn!(
            message = message.as_ref(),
            context = context,
            "Warning occurred"
        );
    }
    
    /// Log successful operation with timing
    pub fn log_success(operation: &str, duration: Duration) {
        info!(
            operation = operation,
            duration_ms = duration.as_millis(),
            "Operation completed successfully"
        );
    }
    
    /// Log operation start
    pub fn log_operation_start(operation: &str) -> Instant {
        debug!(
            operation = operation,
            "Starting operation"
        );
        Instant::now()
    }
    
    /// Log operation completion
    pub fn log_operation_end(operation: &str, start_time: Instant) {
        let duration = start_time.elapsed();
        log_success(operation, duration);
    }
    
    /// Log file processing statistics
    pub fn log_file_stats(file_path: &str, size_bytes: u64, elements_count: usize) {
        info!(
            file_path = file_path,
            size_bytes = size_bytes,
            elements_count = elements_count,
            "File processing statistics"
        );
    }
    
    /// Log batch processing results
    pub fn log_batch_results(total: usize, successful: usize, failed: usize, duration: Duration) {
        info!(
            total = total,
            successful = successful,
            failed = failed,
            success_rate = (successful as f64 / total as f64) * 100.0,
            duration_ms = duration.as_millis(),
            "Batch processing completed"
        );
    }
    
    /// Log memory usage (if available)
    #[cfg(target_os = "linux")]
    pub fn log_memory_usage() {
        if let Ok(status) = std::fs::read_to_string("/proc/self/status") {
            for line in status.lines() {
                if line.starts_with("VmRSS:") {
                    if let Some(memory_kb) = line.split_whitespace().nth(1) {
                        if let Ok(memory_kb) = memory_kb.parse::<u64>() {
                            debug!(
                                memory_kb = memory_kb,
                                memory_mb = memory_kb / 1024,
                                "Current memory usage"
                            );
                        }
                    }
                    break;
                }
            }
        }
    }
    
    #[cfg(not(target_os = "linux"))]
    pub fn log_memory_usage() {
        // Memory logging not implemented for this platform
        debug!("Memory usage logging not available on this platform");
    }
}

/// Structured logging macros for common patterns
#[macro_export]
macro_rules! log_conversion_start {
    ($input_type:expr, $input_size:expr) => {
        tracing::info!(
            input_type = $input_type,
            input_size = $input_size,
            "Starting conversion"
        );
    };
}

#[macro_export]
macro_rules! log_conversion_end {
    ($output_size:expr, $duration:expr) => {
        tracing::info!(
            output_size = $output_size,
            duration_ms = $duration.as_millis(),
            "Conversion completed"
        );
    };
}

#[macro_export]
macro_rules! log_config_update {
    ($field:expr, $old_value:expr, $new_value:expr) => {
        tracing::info!(
            field = $field,
            old_value = $old_value,
            new_value = $new_value,
            "Configuration updated"
        );
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use tracing::Level;

    #[test]
    fn test_logging_config_creation() {
        let config = LoggingConfig::new()
            .with_level(Level::DEBUG)
            .with_location(true)
            .with_json_format(true)
            .with_target_filter("md2docx_converter");
        
        assert_eq!(config.level, Level::DEBUG);
        assert!(config.include_location);
        assert!(config.json_format);
        assert_eq!(config.target_filter, Some("md2docx_converter".to_string()));
    }

    #[test]
    fn test_logging_config_from_env() {
        // Set environment variables
        std::env::set_var("MD2DOCX_LOG_LEVEL", "debug");
        std::env::set_var("MD2DOCX_LOG_JSON", "1");
        std::env::set_var("MD2DOCX_LOG_LOCATION", "1");
        std::env::set_var("MD2DOCX_LOG_TARGET", "test_target");
        
        let config = LoggingConfig::from_env();
        
        assert_eq!(config.level, Level::DEBUG);
        assert!(config.json_format);
        assert!(config.include_location);
        assert_eq!(config.target_filter, Some("test_target".to_string()));
        
        // Clean up
        std::env::remove_var("MD2DOCX_LOG_LEVEL");
        std::env::remove_var("MD2DOCX_LOG_JSON");
        std::env::remove_var("MD2DOCX_LOG_LOCATION");
        std::env::remove_var("MD2DOCX_LOG_TARGET");
    }

    #[test]
    fn test_default_config() {
        let config = LoggingConfig::default();
        
        assert_eq!(config.level, Level::INFO);
        assert!(!config.include_location);
        assert!(!config.json_format);
        assert!(config.include_spans);
        assert_eq!(config.target_filter, None);
    }
}