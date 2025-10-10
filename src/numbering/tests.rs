//! # Comprehensive Tests for Numbering Error Handling and Logging
//! 
//! This module contains tests specifically focused on error handling,
//! graceful degradation, and logging functionality of the numbering system.

#[cfg(test)]
mod error_handling_tests {
    use crate::config::models::ConversionConfig;
    use crate::numbering::{HeadingProcessor, NumberingError};
    use std::sync::Arc;

    fn create_test_config_with_invalid_format() -> Arc<ConversionConfig> {
        let mut config = ConversionConfig::default();
        
        // Set up an invalid format that should trigger errors
        config.styles.headings.get_mut(&1).unwrap().numbering = Some("invalid_format".to_string());
        
        Arc::new(config)
    }

    fn create_test_config_with_valid_numbering() -> Arc<ConversionConfig> {
        let mut config = ConversionConfig::default();
        
        config.styles.headings.get_mut(&1).unwrap().numbering = Some("%1.".to_string());
        config.styles.headings.get_mut(&2).unwrap().numbering = Some("%1.%2.".to_string());
        config.styles.headings.get_mut(&3).unwrap().numbering = Some("%1.%2.%3".to_string());
        
        Arc::new(config)
    }

    #[test]
    fn test_invalid_heading_level_error_handling() {
        let config = create_test_config_with_valid_numbering();
        let mut processor = HeadingProcessor::new(config);
        
        // Test invalid levels
        let result = processor.process_heading(0, "Invalid Level");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), NumberingError::InvalidLevel(0)));
        
        let result = processor.process_heading(7, "Invalid Level");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), NumberingError::InvalidLevel(7)));
        
        // Check that metrics recorded the failures
        let metrics = processor.get_metrics();
        assert_eq!(metrics.critical_failures, 2);
        assert_eq!(metrics.total_headings, 2);
    }

    #[test]
    fn test_graceful_degradation_with_invalid_format() {
        let config = create_test_config_with_invalid_format();
        let mut processor = HeadingProcessor::new(config);
        
        // This should not fail due to graceful degradation
        let result = processor.process_heading(1, "Test Heading");
        assert!(result.is_ok());
        
        // Should return original text when numbering fails
        assert_eq!(result.unwrap(), "Test Heading");
        
        // Check that metrics recorded the degradation
        let metrics = processor.get_metrics();
        assert!(metrics.degraded_operations > 0 || metrics.successful_operations > 0);
    }

    #[test]
    fn test_format_validation_error_handling() {
        let config = create_test_config_with_valid_numbering();
        let processor = HeadingProcessor::new(config);
        
        // This should pass
        let result = processor.validate_numbering_formats();
        assert!(result.is_ok());
        
        // Test with invalid config
        let invalid_config = create_test_config_with_invalid_format();
        let invalid_processor = HeadingProcessor::new(invalid_config);
        
        let result = invalid_processor.validate_numbering_formats();
        assert!(result.is_err());
    }

    #[test]
    fn test_state_reset_error_recovery() {
        let config = create_test_config_with_valid_numbering();
        let mut processor = HeadingProcessor::new(config);
        
        // Process some headings
        processor.process_heading(1, "Chapter 1").unwrap();
        processor.process_heading(2, "Section 1").unwrap();
        
        // Reset state
        processor.reset_state();
        
        // Should start fresh
        let result = processor.process_heading(1, "New Chapter");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "1. New Chapter");
        
        // Check that reset was recorded in metrics
        let metrics = processor.get_metrics();
        assert_eq!(metrics.state_resets, 1);
    }

    #[test]
    fn test_error_categorization() {
        // Test that different error types are properly categorized
        let format_error = NumberingError::invalid_format("test");
        assert_eq!(format_error.category(), "format");
        assert!(format_error.is_recoverable());
        
        let level_error = NumberingError::invalid_level(0);
        assert_eq!(level_error.category(), "level");
        assert!(!level_error.is_recoverable());
        
        let overflow_error = NumberingError::counter_overflow(1);
        assert_eq!(overflow_error.category(), "overflow");
        assert!(overflow_error.is_recoverable());
        
        let parse_error = NumberingError::parse_error("test");
        assert_eq!(parse_error.category(), "parse");
        assert!(parse_error.is_recoverable());
        
        let config_error = NumberingError::config_error("test");
        assert_eq!(config_error.category(), "config");
        assert!(config_error.is_recoverable());
        
        let state_error = NumberingError::state_error("test");
        assert_eq!(state_error.category(), "state");
        assert!(!state_error.is_recoverable());
    }
}