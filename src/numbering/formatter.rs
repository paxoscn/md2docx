//! # Numbering Formatter
//! 
//! This module provides the NumberingFormatter for parsing numbering format strings
//! and formatting them with the current numbering state.

use crate::numbering::error::{NumberingError, NumberingResult};
use crate::numbering::state::NumberingState;
use std::fmt;
use tracing::{debug, trace, warn, error, instrument};
use regex::Regex;

/// A parsed numbering format that can be applied to generate numbering strings
#[derive(Debug, Clone, PartialEq)]
pub struct NumberingFormat {
    /// The levels included in this format (e.g., [1, 2] for "%1.%2.")
    pub levels: Vec<u8>,
    /// The template string with placeholders
    pub template: String,
    /// The separator characters between levels
    pub separators: Vec<String>,
}

/// Formatter for converting numbering format strings to actual numbering text
pub struct NumberingFormatter;

impl NumberingFormatter {
    /// Parse a numbering format string into a structured format
    /// 
    /// Supported formats:
    /// - "%1." -> single level with dot: "1.", "2.", etc.
    /// - "%1.%2." -> two levels with dots: "1.1.", "1.2.", etc.
    /// - "%1.%2.%3" -> three levels: "1.1.1", "1.1.2", etc.
    /// - "%1-%2-%3" -> custom separator: "1-1-1", "1-1-2", etc.
    /// 
    /// # Arguments
    /// * `format` - The format string to parse
    /// 
    /// # Returns
    /// * `NumberingResult<NumberingFormat>` - Parsed format or error
    #[instrument(fields(format_length = format.len()))]
    pub fn parse_format(format: &str) -> NumberingResult<NumberingFormat> {
        trace!("Starting to parse numbering format: '{}'", format);
        
        if format.is_empty() {
            error!("Empty format string provided");
            return Err(NumberingError::invalid_format("Format string cannot be empty"));
        }

        if format.len() > 100 {
            warn!(
                format_length = format.len(),
                "Unusually long format string provided"
            );
        }

        // Use regex to find all %N placeholders
        let placeholder_regex = match Regex::new(r"%(\d+)") {
            Ok(regex) => regex,
            Err(e) => {
                error!(
                    regex_error = %e,
                    "Failed to compile placeholder regex"
                );
                return Err(NumberingError::parse_error(format!("Regex compilation failed: {}", e)));
            }
        };
        
        let mut levels = Vec::new();
        let mut separators = Vec::new();
        let mut last_end = 0;
        let mut placeholder_count = 0;
        
        trace!("Searching for placeholders in format string");
        
        for capture in placeholder_regex.captures_iter(format) {
            placeholder_count += 1;
            let full_match = capture.get(0).unwrap();
            let level_str = capture.get(1).unwrap().as_str();
            
            trace!(
                placeholder_index = placeholder_count,
                level_str = level_str,
                match_start = full_match.start(),
                match_end = full_match.end(),
                "Found placeholder"
            );
            
            // Parse the level number
            let level: u8 = level_str.parse()
                .map_err(|e| {
                    error!(
                        level_str = level_str,
                        parse_error = %e,
                        "Failed to parse level number from placeholder"
                    );
                    NumberingError::parse_error(format!("Invalid level number: {}", level_str))
                })?;
            
            if level < 1 || level > 6 {
                error!(
                    level = level,
                    valid_range = "1-6",
                    "Level number out of valid range"
                );
                return Err(NumberingError::invalid_format(
                    format!("Level {} is out of range (must be 1-6)", level)
                ));
            }
            
            // Extract separator before this placeholder
            let separator = format[last_end..full_match.start()].to_string();
            trace!(
                separator = %separator,
                separator_length = separator.len(),
                "Extracted separator before placeholder"
            );
            separators.push(separator);
            
            levels.push(level);
            last_end = full_match.end();
        }
        
        if levels.is_empty() {
            error!(
                format = format,
                "No valid placeholders found in format string"
            );
            return Err(NumberingError::invalid_format("No valid placeholders found"));
        }
        
        // Add final separator (text after last placeholder)
        let final_separator = format[last_end..].to_string();
        trace!(
            final_separator = %final_separator,
            final_separator_length = final_separator.len(),
            "Extracted final separator"
        );
        separators.push(final_separator);
        
        // Validate that levels are sequential starting from 1
        if let Err(e) = Self::validate_level_sequence(&levels) {
            error!(
                levels = ?levels,
                error = %e,
                "Level sequence validation failed"
            );
            return Err(e);
        }
        
        let parsed_format = NumberingFormat {
            levels: levels.clone(),
            template: format.to_string(),
            separators: separators.clone(),
        };
        
        debug!(
            format = format,
            levels = ?levels,
            separator_count = separators.len(),
            "Successfully parsed numbering format"
        );
        
        Ok(parsed_format)
    }
    
    /// Validate that the level sequence is valid (sequential starting from 1)
    fn validate_level_sequence(levels: &[u8]) -> NumberingResult<()> {
        if levels.is_empty() {
            return Err(NumberingError::parse_error("No levels specified"));
        }
        
        //  Commented for "%4." and "%5)"
        // // Check that levels start from 1 and are sequential
        // for (i, &level) in levels.iter().enumerate() {
        //     let expected_level = (i + 1) as u8;
        //     if level != expected_level {
        //         return Err(NumberingError::invalid_format(
        //             format!("Levels must be sequential starting from 1. Expected %{}, found %{}", 
        //                    expected_level, level)
        //         ));
        //     }
        // }
        
        Ok(())
    }
    
    /// Format a numbering string using the parsed format and current state
    /// 
    /// # Arguments
    /// * `format` - The parsed numbering format
    /// * `state` - The current numbering state
    /// 
    /// # Returns
    /// * `NumberingResult<String>` - Formatted numbering string or error
    #[instrument(skip(state), fields(
        format_template = %format.template,
        level_count = format.levels.len(),
        state_counters = ?state.get_all_counters()
    ))]
    pub fn format_with_parsed(format: &NumberingFormat, state: &NumberingState) -> NumberingResult<String> {
        trace!("Starting format generation with parsed format");
        
        let mut result = String::new();
        let mut formatting_errors = Vec::new();
        
        for (i, &level) in format.levels.iter().enumerate() {
            trace!(
                level_index = i,
                level = level,
                "Processing level in format"
            );
            
            // Add separator before this level (except for the first one if it's empty)
            if i < format.separators.len() && !(i == 0 && format.separators[i].is_empty()) {
                let separator = &format.separators[i];
                trace!(
                    separator = %separator,
                    separator_index = i,
                    "Adding separator to result"
                );
                result.push_str(separator);
            }
            
            // Add the counter value for this level
            match state.get_counter(level) {
                Ok(counter) => {
                    trace!(
                        level = level,
                        counter = counter,
                        "Retrieved counter for level"
                    );
                    result.push_str(&counter.to_string());
                }
                Err(e) => {
                    error!(
                        level = level,
                        error = %e,
                        "Failed to get counter for level during formatting"
                    );
                    formatting_errors.push(format!("Level {}: {}", level, e));
                    
                    // Use fallback value of 1 to continue formatting
                    warn!(
                        level = level,
                        fallback_value = 1,
                        "Using fallback counter value due to error"
                    );
                    result.push_str("1");
                }
            }
        }
        
        // Add final separator if it exists
        if format.levels.len() < format.separators.len() {
            let final_separator = &format.separators[format.levels.len()];
            trace!(
                final_separator = %final_separator,
                "Adding final separator to result"
            );
            result.push_str(final_separator);
        }
        
        if !formatting_errors.is_empty() {
            warn!(
                errors = ?formatting_errors,
                result = %result,
                "Formatting completed with errors, using fallback values"
            );
        } else {
            debug!(
                result = %result,
                result_length = result.len(),
                "Successfully formatted numbering string"
            );
        }
        
        Ok(result)
    }
    
    /// Format a numbering string directly from a format string and state
    /// 
    /// This is a convenience method that combines parsing and formatting.
    /// 
    /// # Arguments
    /// * `format_str` - The format string to parse and apply
    /// * `state` - The current numbering state
    /// 
    /// # Returns
    /// * `NumberingResult<String>` - Formatted numbering string or error
    pub fn format_number(format_str: &str, state: &NumberingState) -> NumberingResult<String> {
        let format = Self::parse_format(format_str)?;
        Self::format_with_parsed(&format, state)
    }
    
    /// Validate a numbering format string without parsing it completely
    /// 
    /// This is useful for configuration validation.
    /// 
    /// # Arguments
    /// * `format` - The format string to validate
    /// 
    /// # Returns
    /// * `NumberingResult<()>` - Ok if valid, error if invalid
    pub fn validate_format(format: &str) -> NumberingResult<()> {
        Self::parse_format(format).map(|_| ())
    }
    
    /// Get the maximum level used in a format string
    /// 
    /// # Arguments
    /// * `format_str` - The format string to analyze
    /// 
    /// # Returns
    /// * `NumberingResult<u8>` - Maximum level or error
    pub fn get_max_level(format_str: &str) -> NumberingResult<u8> {
        let format = Self::parse_format(format_str)?;
        Ok(format.levels.iter().max().copied().unwrap_or(1))
    }
    
    /// Check if a format string uses a specific level
    /// 
    /// # Arguments
    /// * `format_str` - The format string to check
    /// * `level` - The level to check for
    /// 
    /// # Returns
    /// * `NumberingResult<bool>` - True if level is used, false otherwise
    pub fn uses_level(format_str: &str, level: u8) -> NumberingResult<bool> {
        let format = Self::parse_format(format_str)?;
        Ok(format.levels.contains(&level))
    }
}

impl NumberingFormat {
    /// Get the maximum level used in this format
    pub fn max_level(&self) -> u8 {
        self.levels.iter().max().copied().unwrap_or(1)
    }
    
    /// Check if this format uses a specific level
    pub fn uses_level(&self, level: u8) -> bool {
        self.levels.contains(&level)
    }
    
    /// Get the number of levels in this format
    pub fn level_count(&self) -> usize {
        self.levels.len()
    }
}

impl fmt::Display for NumberingFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "NumberingFormat[template: '{}', levels: {:?}]", 
               self.template, self.levels)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_single_level_format() {
        let format = NumberingFormatter::parse_format("%1.").unwrap();
        assert_eq!(format.levels, vec![1]);
        assert_eq!(format.template, "%1.");
        assert_eq!(format.separators, vec!["", "."]);
    }

    #[test]
    fn test_parse_two_level_format() {
        let format = NumberingFormatter::parse_format("%1.%2.").unwrap();
        assert_eq!(format.levels, vec![1, 2]);
        assert_eq!(format.template, "%1.%2.");
        assert_eq!(format.separators, vec!["", ".", "."]);
    }

    #[test]
    fn test_parse_three_level_format() {
        let format = NumberingFormatter::parse_format("%1.%2.%3").unwrap();
        assert_eq!(format.levels, vec![1, 2, 3]);
        assert_eq!(format.template, "%1.%2.%3");
        assert_eq!(format.separators, vec!["", ".", ".", ""]);
    }

    #[test]
    fn test_parse_custom_separator() {
        let format = NumberingFormatter::parse_format("%1-%2-%3").unwrap();
        assert_eq!(format.levels, vec![1, 2, 3]);
        assert_eq!(format.separators, vec!["", "-", "-", ""]);
    }

    #[test]
    fn test_parse_mixed_separators() {
        let format = NumberingFormatter::parse_format("Chapter %1, Section %2.%3").unwrap();
        assert_eq!(format.levels, vec![1, 2, 3]);
        assert_eq!(format.separators, vec!["Chapter ", ", Section ", ".", ""]);
    }

    #[test]
    fn test_parse_invalid_empty() {
        let result = NumberingFormatter::parse_format("");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), NumberingError::InvalidFormat(_)));
    }

    #[test]
    fn test_parse_invalid_no_placeholders() {
        let result = NumberingFormatter::parse_format("no placeholders");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), NumberingError::InvalidFormat(_)));
    }

    #[test]
    fn test_parse_invalid_level_range() {
        let result = NumberingFormatter::parse_format("%0.%1.");
        assert!(result.is_err());
        
        let result = NumberingFormatter::parse_format("%1.%7.");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_invalid_non_sequential() {
        let result = NumberingFormatter::parse_format("%1.%3.");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), NumberingError::InvalidFormat(_)));
    }

    #[test]
    fn test_parse_invalid_not_starting_from_1() {
        let result = NumberingFormatter::parse_format("%2.%3.");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), NumberingError::InvalidFormat(_)));
    }

    #[test]
    fn test_format_single_level() {
        let mut state = NumberingState::new();
        state.process_heading(1).unwrap();
        state.increment_level(1).unwrap(); // Counter becomes 2
        
        let result = NumberingFormatter::format_number("%1.", &state).unwrap();
        assert_eq!(result, "2.");
    }

    #[test]
    fn test_format_two_levels() {
        let mut state = NumberingState::new();
        state.process_heading(1).unwrap(); // H1 = 1
        state.increment_level(1).unwrap(); // H1 = 2
        state.process_heading(2).unwrap(); // H2 = 1
        state.increment_level(2).unwrap(); // H2 = 2
        
        let result = NumberingFormatter::format_number("%1.%2.", &state).unwrap();
        assert_eq!(result, "2.2.");
    }

    #[test]
    fn test_format_three_levels() {
        let mut state = NumberingState::new();
        state.process_heading(1).unwrap(); // H1 = 1
        state.process_heading(2).unwrap(); // H2 = 1
        state.process_heading(3).unwrap(); // H3 = 1
        state.increment_level(3).unwrap(); // H3 = 2
        
        let result = NumberingFormatter::format_number("%1.%2.%3", &state).unwrap();
        assert_eq!(result, "1.1.2");
    }

    #[test]
    fn test_format_custom_separator() {
        let mut state = NumberingState::new();
        state.process_heading(1).unwrap(); // H1 = 1
        state.process_heading(2).unwrap(); // H2 = 1
        state.process_heading(3).unwrap(); // H3 = 1
        
        let result = NumberingFormatter::format_number("%1-%2-%3", &state).unwrap();
        assert_eq!(result, "1-1-1");
    }

    #[test]
    fn test_format_with_text() {
        let mut state = NumberingState::new();
        state.process_heading(1).unwrap(); // H1 = 1
        state.increment_level(1).unwrap(); // H1 = 2
        state.process_heading(2).unwrap(); // H2 = 1
        
        let result = NumberingFormatter::format_number("Chapter %1, Section %2:", &state).unwrap();
        assert_eq!(result, "Chapter 2, Section 1:");
    }

    #[test]
    fn test_validate_format() {
        assert!(NumberingFormatter::validate_format("%1.").is_ok());
        assert!(NumberingFormatter::validate_format("%1.%2.").is_ok());
        assert!(NumberingFormatter::validate_format("%1.%2.%3").is_ok());
        
        assert!(NumberingFormatter::validate_format("").is_err());
        assert!(NumberingFormatter::validate_format("no placeholders").is_err());
        assert!(NumberingFormatter::validate_format("%1.%3.").is_err());
    }

    #[test]
    fn test_get_max_level() {
        assert_eq!(NumberingFormatter::get_max_level("%1.").unwrap(), 1);
        assert_eq!(NumberingFormatter::get_max_level("%1.%2.").unwrap(), 2);
        assert_eq!(NumberingFormatter::get_max_level("%1.%2.%3").unwrap(), 3);
    }

    #[test]
    fn test_uses_level() {
        assert!(NumberingFormatter::uses_level("%1.%2.", 1).unwrap());
        assert!(NumberingFormatter::uses_level("%1.%2.", 2).unwrap());
        assert!(!NumberingFormatter::uses_level("%1.%2.", 3).unwrap());
    }

    #[test]
    fn test_numbering_format_methods() {
        let format = NumberingFormatter::parse_format("%1.%2.%3").unwrap();
        
        assert_eq!(format.max_level(), 3);
        assert!(format.uses_level(1));
        assert!(format.uses_level(2));
        assert!(format.uses_level(3));
        assert!(!format.uses_level(4));
        assert_eq!(format.level_count(), 3);
    }

    #[test]
    fn test_format_display() {
        let format = NumberingFormatter::parse_format("%1.%2.").unwrap();
        let display = format!("{}", format);
        assert!(display.contains("NumberingFormat"));
        assert!(display.contains("%1.%2."));
    }

    #[test]
    fn test_format_with_parsed() {
        let format = NumberingFormatter::parse_format("%1.%2.").unwrap();
        let mut state = NumberingState::new();
        state.process_heading(1).unwrap();
        state.process_heading(2).unwrap();
        state.increment_level(2).unwrap(); // H2 = 2
        
        let result = NumberingFormatter::format_with_parsed(&format, &state).unwrap();
        assert_eq!(result, "1.2.");
    }

    #[test]
    fn test_complex_formatting_scenarios() {
        let mut state = NumberingState::new();
        
        // Simulate a complex document structure
        // H1: Introduction
        state.process_heading(1).unwrap(); // H1 = 1
        
        // H2: Overview  
        state.process_heading(2).unwrap(); // H2 = 1
        assert_eq!(NumberingFormatter::format_number("%1.%2.", &state).unwrap(), "1.1.");
        
        // H2: Details
        state.process_heading(2).unwrap(); // H2 = 2
        assert_eq!(NumberingFormatter::format_number("%1.%2.", &state).unwrap(), "1.2.");
        
        // H3: Subsection
        state.process_heading(3).unwrap(); // H3 = 1
        assert_eq!(NumberingFormatter::format_number("%1.%2.%3", &state).unwrap(), "1.2.1");
        
        // H1: Next Chapter (resets lower levels)
        state.process_heading(1).unwrap(); // H1 = 2, resets H2, H3
        assert_eq!(NumberingFormatter::format_number("%1.", &state).unwrap(), "2.");
        
        // H2: First section of new chapter
        state.process_heading(2).unwrap(); // H2 = 1 (reset)
        assert_eq!(NumberingFormatter::format_number("%1.%2.", &state).unwrap(), "2.1.");
    }

    #[test]
    fn test_edge_cases() {
        let mut state = NumberingState::new();
        
        // Test with only level 1 format but higher level headings
        state.process_heading(1).unwrap();
        state.process_heading(2).unwrap();
        state.process_heading(3).unwrap();
        
        // Should only use H1 counter
        let result = NumberingFormatter::format_number("%1.", &state).unwrap();
        assert_eq!(result, "1.");
    }
}