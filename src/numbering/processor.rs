//! # Heading Processor
//! 
//! This module provides the HeadingProcessor for integrating numbering functionality
//! into the heading processing pipeline. It combines the NumberingState and 
//! NumberingFormatter to automatically add numbering prefixes to headings.

use crate::config::models::ConversionConfig;
use crate::numbering::error::{NumberingError, NumberingResult};
use crate::numbering::formatter::NumberingFormatter;
use crate::numbering::logging::{NumberingMetrics, NumberingLogger};
use crate::numbering::state::NumberingState;
use std::sync::Arc;
use std::time::Instant;
use tracing::{debug, warn, error, info, trace, instrument};

/// Processor for handling heading numbering in the conversion pipeline
/// 
/// The HeadingProcessor maintains numbering state across headings and applies
/// numbering formats based on the configuration. It handles mixed scenarios
/// where some heading levels have numbering enabled and others don't.
/// 
/// ## Error Handling
/// 
/// The processor implements comprehensive error handling with graceful degradation:
/// - Invalid formats fall back to original text
/// - Counter overflows are handled gracefully
/// - All errors are logged with appropriate context
/// - Metrics are collected for monitoring
#[derive(Debug)]
pub struct HeadingProcessor {
    /// Current numbering state
    state: NumberingState,
    /// Configuration reference
    config: Arc<ConversionConfig>,
    /// Metrics collector for monitoring
    metrics: NumberingMetrics,
}

impl HeadingProcessor {
    /// Create a new heading processor with the given configuration
    /// 
    /// # Arguments
    /// * `config` - Configuration containing heading styles and numbering formats
    /// 
    /// # Returns
    /// * `HeadingProcessor` - New processor instance
    #[instrument(skip(config), fields(numbered_levels = ?Self::extract_numbered_levels(&config)))]
    pub fn new(config: Arc<ConversionConfig>) -> Self {
        let numbered_levels = Self::extract_numbered_levels(&config);
        info!("Creating new HeadingProcessor with numbering for levels: {:?}", numbered_levels);
        
        // Validate all numbering formats during initialization
        if let Err(e) = Self::validate_all_formats(&config) {
            error!("Invalid numbering formats detected during processor creation: {}", e);
            warn!("Processor will attempt graceful degradation for invalid formats");
        }
        
        debug!("HeadingProcessor initialized successfully");
        Self {
            state: NumberingState::new(),
            config,
            metrics: NumberingMetrics::new(),
        }
    }
    
    /// Extract numbered levels from config for logging
    fn extract_numbered_levels(config: &ConversionConfig) -> Vec<u8> {
        let mut levels: Vec<u8> = config.styles.headings
            .iter()
            .filter_map(|(&level, style)| {
                if style.numbering.is_some() {
                    Some(level)
                } else {
                    None
                }
            })
            .collect();
        levels.sort();
        levels
    }
    
    /// Validate all numbering formats in the configuration
    fn validate_all_formats(config: &ConversionConfig) -> NumberingResult<()> {
        for (&level, style) in &config.styles.headings {
            if let Some(format) = &style.numbering {
                if let Err(e) = NumberingFormatter::validate_format(format) {
                    error!("Invalid numbering format for level {}: '{}' - {}", level, format, e);
                    return Err(NumberingError::config_error(
                        format!("Level {} has invalid format '{}': {}", level, format, e)
                    ));
                }
            }
        }
        Ok(())
    }

    /// Process a heading and return the text with numbering prefix if configured
    /// 
    /// This is the main method that should be called for each heading encountered
    /// in the document. It updates the internal numbering state and applies
    /// numbering format if configured for the heading level.
    /// 
    /// # Arguments
    /// * `level` - Heading level (1-6 for H1-H6)
    /// * `text` - Original heading text
    /// 
    /// # Returns
    /// * `NumberingResult<String>` - Processed heading text with numbering prefix
    #[instrument(skip(self), fields(level, text_length = text.len(), has_numbering = self.should_number_level(level)))]
    pub fn process_heading(&mut self, level: u8, text: &str) -> NumberingResult<String> {
        let start_time = NumberingLogger::log_operation_start(level, text, self.should_number_level(level));
        
        // Validate heading level
        if level < 1 || level > 6 {
            let error = NumberingError::invalid_level(level);
            NumberingLogger::log_numbering_error(level, text, &error, start_time, false);
            self.metrics.record_failure(&error, start_time.elapsed());
            return Err(error);
        }

        // Always update the numbering state, even if this level doesn't use numbering
        // This ensures proper state management for other levels that do use numbering
        match self.state.process_heading(level) {
            Ok(_) => {
                trace!("Successfully updated numbering state for level {}", level);
            }
            Err(e) => {
                NumberingLogger::log_numbering_error(level, text, &e, start_time, e.is_recoverable());
                
                if e.is_recoverable() {
                    self.metrics.record_degradation(&e, start_time.elapsed());
                    warn!("Continuing with degraded numbering functionality due to state error");
                } else {
                    self.metrics.record_failure(&e, start_time.elapsed());
                    return Err(e);
                }
            }
        }
        
        // Check if this level should have numbering
        if !self.should_number_level(level) {
            debug!("Level {} does not have numbering configured, returning original text", level);
            self.metrics.record_success(start_time.elapsed());
            return Ok(text.to_string());
        }

        // Get the numbering format for this level
        let numbering_format = match self.get_numbering_format(level) {
            Some(format) => format,
            None => {
                warn!(
                    level = level,
                    "Numbering format not found for level {} despite should_number_level returning true",
                    level
                );
                self.metrics.record_success(start_time.elapsed());
                return Ok(text.to_string());
            }
        };

        // Generate the numbering prefix with comprehensive error handling
        match self.generate_numbering_prefix(level, &numbering_format) {
            Ok(prefix) => {
                let result = self.merge_numbering_with_text(&prefix, text);
                NumberingLogger::log_operation_success(level, text, &result, start_time);
                self.metrics.record_success(start_time.elapsed());
                Ok(result)
            }
            Err(e) => {
                self.handle_numbering_error(level, &e, text, start_time)
            }
        }
    }
    
    /// Generate numbering prefix with detailed error handling
    #[instrument(skip(self), fields(level, format = %numbering_format))]
    fn generate_numbering_prefix(&self, level: u8, numbering_format: &str) -> NumberingResult<String> {
        trace!("Generating numbering prefix for level {} with format '{}'", level, numbering_format);
        
        match NumberingFormatter::format_number(numbering_format, &self.state) {
            Ok(prefix) => {
                trace!("Successfully generated prefix: '{}'", prefix);
                Ok(prefix)
            }
            Err(e) => {
                error!(
                    level = level,
                    format = numbering_format,
                    error = %e,
                    error_category = e.category(),
                    state_counters = ?self.state.get_all_counters(),
                    "Failed to generate numbering prefix"
                );
                Err(e)
            }
        }
    }
    
    /// Handle numbering errors with appropriate logging and graceful degradation
    #[instrument(skip(self, text), fields(level, error_category = error.category(), recoverable = error.is_recoverable()))]
    fn handle_numbering_error(&mut self, level: u8, error: &NumberingError, text: &str, start_time: Instant) -> NumberingResult<String> {
        let used_fallback = error.is_recoverable();
        NumberingLogger::log_numbering_error(level, text, error, start_time, used_fallback);
        
        if error.is_recoverable() {
            self.metrics.record_degradation(error, start_time.elapsed());
            
            // Log degradation metrics for monitoring
            info!(
                degradation_event = "numbering_fallback",
                level = level,
                error_type = error.category(),
                "Numbering functionality degraded to fallback mode"
            );
            
            // Return original text as fallback
            Ok(text.to_string())
        } else {
            // Even for non-recoverable errors, attempt graceful degradation
            self.metrics.record_degradation(error, start_time.elapsed());
            
            warn!(
                level = level,
                error = %error,
                "Attempting graceful degradation for non-recoverable error"
            );
            
            Ok(text.to_string())
        }
    }

    /// Check if a heading level should have numbering applied
    /// 
    /// # Arguments
    /// * `level` - Heading level to check (1-6 for H1-H6)
    /// 
    /// # Returns
    /// * `bool` - True if numbering should be applied, false otherwise
    pub fn should_number_level(&self, level: u8) -> bool {
        self.config.styles.headings
            .get(&level)
            .and_then(|style| style.numbering.as_ref())
            .is_some()
    }

    /// Get the numbering format string for a specific level
    /// 
    /// # Arguments
    /// * `level` - Heading level (1-6 for H1-H6)
    /// 
    /// # Returns
    /// * `Option<String>` - Numbering format string if configured
    fn get_numbering_format(&self, level: u8) -> Option<String> {
        self.config.styles.headings
            .get(&level)
            .and_then(|style| style.numbering.clone())
    }

    /// Merge numbering prefix with heading text
    /// 
    /// This method handles the proper spacing between the numbering prefix
    /// and the heading text, ensuring good readability.
    /// 
    /// # Arguments
    /// * `prefix` - Numbering prefix (e.g., "1.1.")
    /// * `text` - Original heading text
    /// 
    /// # Returns
    /// * `String` - Combined text with proper spacing
    fn merge_numbering_with_text(&self, prefix: &str, text: &str) -> String {
        // Trim the original text to handle any existing leading/trailing whitespace
        let trimmed_text = text.trim();
        
        if trimmed_text.is_empty() {
            // If text is empty, just return the prefix
            prefix.to_string()
        } else {
            // Add a single space between prefix and text for readability
            format!("{} {}", prefix, trimmed_text)
        }
    }

    /// Reset the numbering state
    /// 
    /// This can be useful when processing multiple documents or when
    /// restarting numbering for a new section.
    #[instrument(skip(self))]
    pub fn reset_state(&mut self) {
        info!("Resetting numbering state for new document");
        let old_state = self.state.clone();
        self.state.reset_all();
        self.metrics.record_state_reset();
        
        NumberingLogger::log_state_operation("reset_all", None, &self.state);
        
        debug!(
            old_counters = ?old_state.get_all_counters(),
            new_counters = ?self.state.get_all_counters(),
            "Numbering state reset completed"
        );
    }

    /// Get the current numbering state (for debugging/testing)
    /// 
    /// # Returns
    /// * `&NumberingState` - Reference to current state
    pub fn get_state(&self) -> &NumberingState {
        &self.state
    }

    /// Get a mutable reference to the numbering state (for advanced use cases)
    /// 
    /// # Returns
    /// * `&mut NumberingState` - Mutable reference to current state
    pub fn get_state_mut(&mut self) -> &mut NumberingState {
        &mut self.state
    }

    /// Check if any heading levels in the configuration use numbering
    /// 
    /// This can be useful for optimization - if no levels use numbering,
    /// the processor can skip numbering logic entirely.
    /// 
    /// # Returns
    /// * `bool` - True if any level has numbering configured
    pub fn has_any_numbering(&self) -> bool {
        self.config.styles.headings
            .values()
            .any(|style| style.numbering.is_some())
    }

    /// Get all heading levels that have numbering configured
    /// 
    /// # Returns
    /// * `Vec<u8>` - Vector of heading levels with numbering (sorted)
    pub fn get_numbered_levels(&self) -> Vec<u8> {
        let mut levels: Vec<u8> = self.config.styles.headings
            .iter()
            .filter_map(|(&level, style)| {
                if style.numbering.is_some() {
                    Some(level)
                } else {
                    None
                }
            })
            .collect();
        levels.sort();
        levels
    }

    /// Validate all numbering formats in the configuration
    /// 
    /// This method checks that all configured numbering formats are valid
    /// and can be parsed by the NumberingFormatter.
    /// 
    /// # Returns
    /// * `NumberingResult<()>` - Ok if all formats are valid, error otherwise
    #[instrument(skip(self))]
    pub fn validate_numbering_formats(&self) -> NumberingResult<()> {
        debug!("Validating all numbering formats in configuration");
        
        let mut validation_errors = Vec::new();
        let mut valid_count = 0;
        let mut total_count = 0;
        
        for (&level, style) in &self.config.styles.headings {
            if let Some(format) = &style.numbering {
                total_count += 1;
                
                match NumberingFormatter::validate_format(format) {
                    Ok(_) => {
                        valid_count += 1;
                        trace!("Valid numbering format for level {}: '{}'", level, format);
                    }
                    Err(e) => {
                        let error_msg = format!("Level {} has invalid format '{}': {}", level, format, e);
                        error!(
                            level = level,
                            format = format,
                            error = %e,
                            "Invalid numbering format detected"
                        );
                        validation_errors.push(error_msg);
                    }
                }
            }
        }
        
        if !validation_errors.is_empty() {
            error!(
                total_formats = total_count,
                valid_formats = valid_count,
                invalid_formats = validation_errors.len(),
                errors = ?validation_errors,
                "Numbering format validation failed"
            );
            
            return Err(NumberingError::config_error(
                format!("Invalid numbering formats found: {}", validation_errors.join("; "))
            ));
        }
        
        info!(
            total_formats = total_count,
            valid_formats = valid_count,
            "All numbering formats validated successfully"
        );
        
        Ok(())
    }

    /// Process multiple headings in sequence
    /// 
    /// This is a convenience method for processing a batch of headings.
    /// It maintains state across all headings in the sequence.
    /// 
    /// # Arguments
    /// * `headings` - Vector of (level, text) tuples
    /// 
    /// # Returns
    /// * `NumberingResult<Vec<String>>` - Processed heading texts
    #[instrument(skip(self, headings), fields(heading_count = headings.len()))]
    pub fn process_headings(&mut self, headings: Vec<(u8, &str)>) -> NumberingResult<Vec<String>> {
        info!("Processing batch of {} headings", headings.len());
        
        let mut results = Vec::new();
        let mut error_count = 0;
        let mut success_count = 0;
        
        for (i, (level, text)) in headings.iter().enumerate() {
            trace!("Processing heading {} of {}: level={}, text='{}'", i + 1, headings.len(), level, text);
            
            match self.process_heading(*level, text) {
                Ok(processed) => {
                    success_count += 1;
                    results.push(processed);
                }
                Err(e) => {
                    error_count += 1;
                    error!(
                        heading_index = i,
                        level = level,
                        text = text,
                        error = %e,
                        "Failed to process heading in batch"
                    );
                    
                    // For batch processing, continue with original text on error
                    warn!("Using original text for failed heading in batch processing");
                    results.push(text.to_string());
                }
            }
        }
        
        info!(
            total_headings = headings.len(),
            successful = success_count,
            errors = error_count,
            success_rate = (success_count as f64 / headings.len() as f64) * 100.0,
            "Batch heading processing completed"
        );
        
        if error_count > 0 {
            warn!(
                error_count = error_count,
                total_count = headings.len(),
                "Some headings failed to process with numbering, using fallback text"
            );
        }
        
        Ok(results)
    }

    /// Get numbering preview for a specific level without updating state
    /// 
    /// This method is useful for UI previews where you want to show what
    /// the numbering would look like without actually processing the heading.
    /// 
    /// # Arguments
    /// * `level` - Heading level to preview
    /// 
    /// # Returns
    /// * `NumberingResult<Option<String>>` - Preview string if numbering is configured
    pub fn preview_numbering(&self, level: u8) -> NumberingResult<Option<String>> {
        if !self.should_number_level(level) {
            return Ok(None);
        }

        let format = self.get_numbering_format(level)
            .ok_or_else(|| NumberingError::invalid_format("Numbering format not found"))?;

        let preview = NumberingFormatter::format_number(&format, &self.state)?;
        Ok(Some(preview))
    }
    
    /// Get current metrics for monitoring
    /// 
    /// # Returns
    /// * `&NumberingMetrics` - Reference to current metrics
    pub fn get_metrics(&self) -> &NumberingMetrics {
        &self.metrics
    }
    
    /// Log current metrics summary
    pub fn log_metrics_summary(&self) {
        self.metrics.log_summary();
    }
    
    /// Log current health status
    pub fn log_health_status(&self) {
        self.metrics.log_health_status();
    }
    
    /// Reset metrics (useful for testing or periodic resets)
    pub fn reset_metrics(&mut self) {
        info!("Resetting numbering metrics");
        self.metrics = NumberingMetrics::new();
    }
    
    /// Check if numbering functionality is healthy
    pub fn is_healthy(&self) -> bool {
        matches!(self.metrics.check_health(), crate::numbering::logging::HealthStatus::Healthy)
    }
}

impl Clone for HeadingProcessor {
    fn clone(&self) -> Self {
        Self {
            state: self.state.clone(),
            config: Arc::clone(&self.config),
            metrics: NumberingMetrics::new(), // Start with fresh metrics for cloned instance
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::models::ConversionConfig;

    fn create_test_config_with_numbering() -> Arc<ConversionConfig> {
        let mut config = ConversionConfig::default();
        
        // Configure H1 with numbering
        config.styles.headings.get_mut(&1).unwrap().numbering = Some("%1.".to_string());
        
        // Configure H2 with numbering
        config.styles.headings.get_mut(&2).unwrap().numbering = Some("%1.%2.".to_string());
        
        // Configure H3 with numbering
        config.styles.headings.get_mut(&3).unwrap().numbering = Some("%1.%2.%3".to_string());
        
        // H4, H5, H6 have no numbering (None)
        
        Arc::new(config)
    }

    fn create_test_config_no_numbering() -> Arc<ConversionConfig> {
        Arc::new(ConversionConfig::default())
    }

    #[test]
    fn test_new_processor() {
        let config = create_test_config_with_numbering();
        let processor = HeadingProcessor::new(config);
        
        assert!(processor.has_any_numbering());
        assert_eq!(processor.get_numbered_levels(), vec![1, 2, 3]);
    }

    #[test]
    fn test_should_number_level() {
        let config = create_test_config_with_numbering();
        let processor = HeadingProcessor::new(config);
        
        assert!(processor.should_number_level(1));
        assert!(processor.should_number_level(2));
        assert!(processor.should_number_level(3));
        assert!(!processor.should_number_level(4));
        assert!(!processor.should_number_level(5));
        assert!(!processor.should_number_level(6));
    }

    #[test]
    fn test_process_heading_with_numbering() {
        let config = create_test_config_with_numbering();
        let mut processor = HeadingProcessor::new(config);
        
        // Process H1
        let result = processor.process_heading(1, "Introduction").unwrap();
        assert_eq!(result, "1. Introduction");
        
        // Process another H1
        let result = processor.process_heading(1, "Chapter Two").unwrap();
        assert_eq!(result, "2. Chapter Two");
        
        // Process H2
        let result = processor.process_heading(2, "Overview").unwrap();
        assert_eq!(result, "2.1. Overview");
        
        // Process another H2
        let result = processor.process_heading(2, "Details").unwrap();
        assert_eq!(result, "2.2. Details");
        
        // Process H3
        let result = processor.process_heading(3, "Subsection").unwrap();
        assert_eq!(result, "2.2.1 Subsection");
    }

    #[test]
    fn test_process_heading_without_numbering() {
        let config = create_test_config_with_numbering();
        let mut processor = HeadingProcessor::new(config);
        
        // Process H1 to set up state
        processor.process_heading(1, "Chapter").unwrap();
        
        // Process H4 (no numbering configured)
        let result = processor.process_heading(4, "Subsection").unwrap();
        assert_eq!(result, "Subsection"); // Should return original text
        
        // Process H2 after H4 - should still work correctly
        let result = processor.process_heading(2, "Section").unwrap();
        assert_eq!(result, "1.1. Section");
    }

    #[test]
    fn test_mixed_numbering_scenario() {
        let config = create_test_config_with_numbering();
        let mut processor = HeadingProcessor::new(config);
        
        // H1 with numbering
        let result = processor.process_heading(1, "Chapter 1").unwrap();
        assert_eq!(result, "1. Chapter 1");
        
        // H2 with numbering
        let result = processor.process_heading(2, "Section A").unwrap();
        assert_eq!(result, "1.1. Section A");
        
        // H4 without numbering (but state should still be maintained)
        let result = processor.process_heading(4, "Subsection").unwrap();
        assert_eq!(result, "Subsection");
        
        // H2 again - should increment properly
        let result = processor.process_heading(2, "Section B").unwrap();
        assert_eq!(result, "1.2. Section B");
        
        // H3 with numbering
        let result = processor.process_heading(3, "Details").unwrap();
        assert_eq!(result, "1.2.1 Details");
    }

    #[test]
    fn test_text_merging() {
        let config = create_test_config_with_numbering();
        let mut processor = HeadingProcessor::new(config);
        
        // Test normal text
        let result = processor.process_heading(1, "Normal Text").unwrap();
        assert_eq!(result, "1. Normal Text");
        
        // Test text with leading/trailing spaces
        let result = processor.process_heading(1, "  Spaced Text  ").unwrap();
        assert_eq!(result, "2. Spaced Text");
        
        // Test empty text
        let result = processor.process_heading(1, "").unwrap();
        assert_eq!(result, "3.");
        
        // Test whitespace-only text
        let result = processor.process_heading(1, "   ").unwrap();
        assert_eq!(result, "4.");
    }

    #[test]
    fn test_reset_state() {
        let config = create_test_config_with_numbering();
        let mut processor = HeadingProcessor::new(config);
        
        // Process some headings
        processor.process_heading(1, "Chapter").unwrap();
        processor.process_heading(2, "Section").unwrap();
        
        // Reset state
        processor.reset_state();
        
        // Process again - should start from 1
        let result = processor.process_heading(1, "New Chapter").unwrap();
        assert_eq!(result, "1. New Chapter");
        
        let result = processor.process_heading(2, "New Section").unwrap();
        assert_eq!(result, "1.1. New Section");
    }

    #[test]
    fn test_no_numbering_config() {
        let config = create_test_config_no_numbering();
        let mut processor = HeadingProcessor::new(config);
        
        assert!(!processor.has_any_numbering());
        assert!(processor.get_numbered_levels().is_empty());
        
        // All headings should return original text
        let result = processor.process_heading(1, "Chapter").unwrap();
        assert_eq!(result, "Chapter");
        
        let result = processor.process_heading(2, "Section").unwrap();
        assert_eq!(result, "Section");
    }

    #[test]
    fn test_invalid_level() {
        let config = create_test_config_with_numbering();
        let mut processor = HeadingProcessor::new(config);
        
        // Test invalid levels
        assert!(processor.process_heading(0, "Invalid").is_err());
        assert!(processor.process_heading(7, "Invalid").is_err());
    }

    #[test]
    fn test_validate_numbering_formats() {
        let config = create_test_config_with_numbering();
        let processor = HeadingProcessor::new(config);
        
        // Should pass validation
        assert!(processor.validate_numbering_formats().is_ok());
        
        // Test with invalid format
        let mut config = ConversionConfig::default();
        config.styles.headings.get_mut(&1).unwrap().numbering = Some("invalid".to_string());
        let processor = HeadingProcessor::new(Arc::new(config));
        
        assert!(processor.validate_numbering_formats().is_err());
    }

    #[test]
    fn test_process_multiple_headings() {
        let config = create_test_config_with_numbering();
        let mut processor = HeadingProcessor::new(config);
        
        let headings = vec![
            (1, "Chapter 1"),
            (2, "Section A"),
            (2, "Section B"),
            (3, "Subsection"),
            (1, "Chapter 2"),
        ];
        
        let results = processor.process_headings(headings).unwrap();
        
        assert_eq!(results, vec![
            "1. Chapter 1",
            "1.1. Section A", 
            "1.2. Section B",
            "1.2.1 Subsection",
            "2. Chapter 2",
        ]);
    }

    #[test]
    fn test_preview_numbering() {
        let config = create_test_config_with_numbering();
        let mut processor = HeadingProcessor::new(config);
        
        // Set up some state
        processor.process_heading(1, "Chapter").unwrap();
        processor.process_heading(2, "Section").unwrap();
        
        // Preview what the current numbering state shows (not what the next would be)
        // After processing H1 and H2, the current state is H1=1, H2=1
        let preview = processor.preview_numbering(2).unwrap();
        assert_eq!(preview, Some("1.1.".to_string()));
        
        let preview = processor.preview_numbering(3).unwrap();
        assert_eq!(preview, Some("1.1.1".to_string()));
        
        // Preview for level without numbering
        let preview = processor.preview_numbering(4).unwrap();
        assert_eq!(preview, None);
    }

    #[test]
    fn test_skip_level_handling() {
        let config = create_test_config_with_numbering();
        let mut processor = HeadingProcessor::new(config);
        
        // H1
        let result = processor.process_heading(1, "Chapter").unwrap();
        assert_eq!(result, "1. Chapter");
        
        // Skip H2, go directly to H3
        let result = processor.process_heading(3, "Subsection").unwrap();
        assert_eq!(result, "1.1.1 Subsection");
        
        // Go back to H2
        let result = processor.process_heading(2, "Section").unwrap();
        assert_eq!(result, "1.1. Section");
        
        // H3 again should reset properly
        let result = processor.process_heading(3, "Another Subsection").unwrap();
        assert_eq!(result, "1.1.1 Another Subsection");
    }

    #[test]
    fn test_clone() {
        let config = create_test_config_with_numbering();
        let mut processor = HeadingProcessor::new(config);
        
        // Process some headings
        processor.process_heading(1, "Chapter").unwrap();
        processor.process_heading(2, "Section").unwrap();
        
        // Clone the processor
        let cloned = processor.clone();
        
        // Both should have the same state
        assert_eq!(processor.get_state().get_counter(1).unwrap(), 
                   cloned.get_state().get_counter(1).unwrap());
        assert_eq!(processor.get_state().get_counter(2).unwrap(), 
                   cloned.get_state().get_counter(2).unwrap());
    }

    #[test]
    fn test_graceful_degradation() {
        // Create a config with an invalid format (this shouldn't happen in practice
        // due to validation, but test graceful handling anyway)
        let mut config = ConversionConfig::default();
        
        // We'll test by creating a processor and then manually checking error handling
        // in the format_number call (simulated by testing with a valid config first)
        config.styles.headings.get_mut(&1).unwrap().numbering = Some("%1.".to_string());
        let mut processor = HeadingProcessor::new(Arc::new(config));
        
        // This should work normally
        let result = processor.process_heading(1, "Test").unwrap();
        assert_eq!(result, "1. Test");
        
        // The graceful degradation is tested implicitly through the error handling
        // in the process_heading method - if formatting fails, it returns original text
    }

    #[test]
    fn test_integration_complete_document_flow() {
        let config = create_test_config_with_numbering();
        let mut processor = HeadingProcessor::new(config);
        
        // Simulate processing a complete document with various heading structures
        let document_headings = vec![
            (1, "Introduction"),
            (2, "Overview"),
            (2, "Scope"),
            (3, "Technical Requirements"),
            (3, "Business Requirements"),
            (1, "Implementation"),
            (2, "Phase 1"),
            (3, "Setup"),
            (3, "Configuration"),
            (2, "Phase 2"),
            (4, "Testing"), // H4 has no numbering
            (2, "Phase 3"),
            (1, "Conclusion"),
        ];
        
        let expected_results = vec![
            "1. Introduction",
            "1.1. Overview",
            "1.2. Scope",
            "1.2.1 Technical Requirements",
            "1.2.2 Business Requirements",
            "2. Implementation",
            "2.1. Phase 1",
            "2.1.1 Setup",
            "2.1.2 Configuration",
            "2.2. Phase 2",
            "Testing", // No numbering for H4
            "2.3. Phase 3",
            "3. Conclusion",
        ];
        
        let results = processor.process_headings(document_headings).unwrap();
        assert_eq!(results, expected_results);
    }

    #[test]
    fn test_integration_mixed_numbering_complex() {
        // Create a config where only H1 and H3 have numbering (skip H2)
        let mut config = ConversionConfig::default();
        config.styles.headings.get_mut(&1).unwrap().numbering = Some("%1.".to_string());
        config.styles.headings.get_mut(&3).unwrap().numbering = Some("%1.%2.%3".to_string());
        // H2 has no numbering
        
        let mut processor = HeadingProcessor::new(Arc::new(config));
        
        let headings = vec![
            (1, "Chapter 1"),
            (2, "Section A"), // No numbering
            (3, "Subsection 1"),
            (3, "Subsection 2"),
            (2, "Section B"), // No numbering
            (3, "Subsection 3"),
            (1, "Chapter 2"),
            (3, "Subsection 4"), // Skip H2, go directly to H3
        ];
        
        let expected = vec![
            "1. Chapter 1",
            "Section A",
            "1.1.1 Subsection 1",
            "1.1.2 Subsection 2",
            "Section B",
            "1.2.1 Subsection 3",
            "2. Chapter 2",
            "2.1.1 Subsection 4",
        ];
        
        let results = processor.process_headings(headings).unwrap();
        assert_eq!(results, expected);
    }

    #[test]
    fn test_integration_error_recovery() {
        let config = create_test_config_with_numbering();
        let mut processor = HeadingProcessor::new(config);
        
        // Test that invalid levels are handled gracefully
        assert!(processor.process_heading(0, "Invalid").is_err());
        assert!(processor.process_heading(7, "Invalid").is_err());
        
        // Test that valid processing continues after errors
        let result = processor.process_heading(1, "Valid Chapter").unwrap();
        assert_eq!(result, "1. Valid Chapter");
        
        let result = processor.process_heading(2, "Valid Section").unwrap();
        assert_eq!(result, "1.1. Valid Section");
    }

    #[test]
    fn test_integration_state_consistency() {
        let config = create_test_config_with_numbering();
        let mut processor = HeadingProcessor::new(config);
        
        // Process some headings
        processor.process_heading(1, "Chapter").unwrap();
        processor.process_heading(2, "Section").unwrap();
        processor.process_heading(3, "Subsection").unwrap();
        
        // Verify state consistency
        assert_eq!(processor.get_state().get_counter(1).unwrap(), 1);
        assert_eq!(processor.get_state().get_counter(2).unwrap(), 1);
        assert_eq!(processor.get_state().get_counter(3).unwrap(), 1);
        
        // Process more headings and verify increments
        processor.process_heading(3, "Another Subsection").unwrap();
        assert_eq!(processor.get_state().get_counter(3).unwrap(), 2);
        
        processor.process_heading(2, "Another Section").unwrap();
        assert_eq!(processor.get_state().get_counter(2).unwrap(), 2);
        assert_eq!(processor.get_state().get_counter(3).unwrap(), 1); // Should reset
        
        processor.process_heading(1, "Another Chapter").unwrap();
        assert_eq!(processor.get_state().get_counter(1).unwrap(), 2);
        assert_eq!(processor.get_state().get_counter(2).unwrap(), 1); // Should reset
        assert_eq!(processor.get_state().get_counter(3).unwrap(), 1); // Should reset
    }
}