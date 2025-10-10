//! # Numbering Error Types
//! 
//! This module defines error types specific to the heading numbering functionality.

use std::fmt;
use std::error::Error;

/// Errors that can occur during heading numbering operations
#[derive(Debug, Clone, PartialEq)]
pub enum NumberingError {
    /// Invalid numbering format string
    InvalidFormat(String),
    
    /// Invalid heading level (must be 1-6)
    InvalidLevel(u8),
    
    /// Error parsing numbering format
    ParseError(String),
    
    /// Counter overflow error
    CounterOverflow(u8),
    
    /// Configuration error related to numbering
    ConfigError(String),
    
    /// State management error
    StateError(String),
}

impl fmt::Display for NumberingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NumberingError::InvalidFormat(format) => {
                write!(f, "Invalid numbering format: '{}'", format)
            }
            NumberingError::InvalidLevel(level) => {
                write!(f, "Invalid heading level: {} (must be 1-6)", level)
            }
            NumberingError::ParseError(msg) => {
                write!(f, "Numbering format parsing error: {}", msg)
            }
            NumberingError::CounterOverflow(level) => {
                write!(f, "Counter overflow for heading level {}", level)
            }
            NumberingError::ConfigError(msg) => {
                write!(f, "Numbering configuration error: {}", msg)
            }
            NumberingError::StateError(msg) => {
                write!(f, "Numbering state error: {}", msg)
            }
        }
    }
}

impl Error for NumberingError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }
}

impl NumberingError {
    /// Create a new InvalidFormat error
    pub fn invalid_format<S: Into<String>>(format: S) -> Self {
        NumberingError::InvalidFormat(format.into())
    }
    
    /// Create a new InvalidLevel error
    pub fn invalid_level(level: u8) -> Self {
        NumberingError::InvalidLevel(level)
    }
    
    /// Create a new ParseError
    pub fn parse_error<S: Into<String>>(msg: S) -> Self {
        NumberingError::ParseError(msg.into())
    }
    
    /// Create a new CounterOverflow error
    pub fn counter_overflow(level: u8) -> Self {
        NumberingError::CounterOverflow(level)
    }
    
    /// Create a new ConfigError
    pub fn config_error<S: Into<String>>(msg: S) -> Self {
        NumberingError::ConfigError(msg.into())
    }
    
    /// Create a new StateError
    pub fn state_error<S: Into<String>>(msg: S) -> Self {
        NumberingError::StateError(msg.into())
    }
    
    /// Check if this error is recoverable (can continue with degraded functionality)
    pub fn is_recoverable(&self) -> bool {
        match self {
            NumberingError::InvalidFormat(_) => true,
            NumberingError::InvalidLevel(_) => false,
            NumberingError::ParseError(_) => true,
            NumberingError::CounterOverflow(_) => true,
            NumberingError::ConfigError(_) => true,
            NumberingError::StateError(_) => false,
        }
    }
    
    /// Get the error category for logging purposes
    pub fn category(&self) -> &'static str {
        match self {
            NumberingError::InvalidFormat(_) => "format",
            NumberingError::InvalidLevel(_) => "level",
            NumberingError::ParseError(_) => "parse",
            NumberingError::CounterOverflow(_) => "overflow",
            NumberingError::ConfigError(_) => "config",
            NumberingError::StateError(_) => "state",
        }
    }
}

/// Result type alias for numbering operations
pub type NumberingResult<T> = Result<T, NumberingError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let errors = vec![
            NumberingError::invalid_format("%1.%2.%3.%4"),
            NumberingError::invalid_level(7),
            NumberingError::parse_error("Invalid placeholder"),
            NumberingError::counter_overflow(3),
            NumberingError::config_error("Missing numbering field"),
            NumberingError::state_error("State corruption detected"),
        ];

        for error in errors {
            let display = format!("{}", error);
            assert!(!display.is_empty());
            println!("Error: {}", display);
        }
    }

    #[test]
    fn test_error_constructors() {
        let format_err = NumberingError::invalid_format("%invalid");
        assert!(matches!(format_err, NumberingError::InvalidFormat(_)));
        
        let level_err = NumberingError::invalid_level(0);
        assert!(matches!(level_err, NumberingError::InvalidLevel(0)));
        
        let parse_err = NumberingError::parse_error("test");
        assert!(matches!(parse_err, NumberingError::ParseError(_)));
    }

    #[test]
    fn test_is_recoverable() {
        assert!(NumberingError::invalid_format("test").is_recoverable());
        assert!(!NumberingError::invalid_level(0).is_recoverable());
        assert!(NumberingError::parse_error("test").is_recoverable());
        assert!(NumberingError::counter_overflow(1).is_recoverable());
        assert!(NumberingError::config_error("test").is_recoverable());
        assert!(!NumberingError::state_error("test").is_recoverable());
    }

    #[test]
    fn test_category() {
        assert_eq!(NumberingError::invalid_format("test").category(), "format");
        assert_eq!(NumberingError::invalid_level(0).category(), "level");
        assert_eq!(NumberingError::parse_error("test").category(), "parse");
        assert_eq!(NumberingError::counter_overflow(1).category(), "overflow");
        assert_eq!(NumberingError::config_error("test").category(), "config");
        assert_eq!(NumberingError::state_error("test").category(), "state");
    }

    #[test]
    fn test_error_equality() {
        let err1 = NumberingError::invalid_format("test");
        let err2 = NumberingError::invalid_format("test");
        let err3 = NumberingError::invalid_format("different");
        
        assert_eq!(err1, err2);
        assert_ne!(err1, err3);
    }
}