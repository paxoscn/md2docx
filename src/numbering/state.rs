//! # Numbering State Management
//! 
//! This module provides the NumberingState structure for managing heading counters
//! across different heading levels, including support for skip-level headings.

use crate::numbering::error::{NumberingError, NumberingResult};
use std::fmt;
use tracing::{debug, warn, error, info, trace};

/// Numbering state manager that tracks counters for each heading level
/// 
/// This structure maintains counters for heading levels H1-H6 and provides
/// methods to increment, reset, and query these counters. It handles skip-level
/// headings (e.g., H1 directly to H3) by properly managing intermediate levels.
#[derive(Debug, Clone, PartialEq)]
pub struct NumberingState {
    /// Counters for heading levels H1-H6 (indices 0-5)
    counters: [u32; 6],
    /// Track which levels have been explicitly processed (not just initialized)
    processed_levels: [bool; 6],
}

impl NumberingState {
    /// Create a new numbering state with all counters initialized to 1
    pub fn new() -> Self {
        debug!("Initializing new numbering state");
        Self {
            counters: [1; 6],
            processed_levels: [false; 6],
        }
    }

    /// Process a heading at the specified level, handling increments and resets
    /// 
    /// This is the main method that should be called when encountering a heading.
    /// It handles the logic for incrementing the current level and resetting lower levels.
    /// 
    /// # Arguments
    /// * `level` - Heading level (1-6 for H1-H6)
    /// 
    /// # Returns
    /// * `NumberingResult<()>` - Ok if successful, error if level is invalid
    pub fn process_heading(&mut self, level: u8) -> NumberingResult<()> {
        if level < 1 || level > 6 {
            error!(
                level = level,
                valid_range = "1-6",
                "Invalid heading level provided to numbering state"
            );
            return Err(NumberingError::invalid_level(level));
        }

        let index = (level - 1) as usize;
        let old_counter = self.counters[index];
        let was_processed = self.processed_levels[index];
        
        debug!(
            level = level,
            old_counter = old_counter,
            was_processed = was_processed,
            "Processing heading level in numbering state"
        );
        
        // Reset all lower levels (higher numbers) first
        if let Err(e) = self.reset_lower_levels(level) {
            error!(
                level = level,
                error = %e,
                "Failed to reset lower levels during heading processing"
            );
            return Err(e);
        }
        
        // If this level has been explicitly processed before, increment it
        if self.processed_levels[index] {
            if let Err(e) = self.increment_level(level) {
                // Handle counter overflow gracefully
                if matches!(e, NumberingError::CounterOverflow(_)) {
                    warn!(
                        level = level,
                        old_counter = old_counter,
                        "Counter overflow detected, reset to 1"
                    );
                    // Counter was already reset to 1 by increment_level
                } else {
                    error!(
                        level = level,
                        error = %e,
                        "Failed to increment counter during heading processing"
                    );
                    return Err(e);
                }
            }
        } else {
            // First time explicitly processing this level, mark as processed but don't increment (starts at 1)
            self.processed_levels[index] = true;
            debug!("Marked level {} as processed for the first time", level);
        }
        
        let new_counter = self.counters[index];
        debug!(
            level = level,
            old_counter = old_counter,
            new_counter = new_counter,
            was_processed = was_processed,
            now_processed = self.processed_levels[index],
            "Successfully processed heading level"
        );
        
        Ok(())
    }

    /// Increment the counter for the specified heading level
    /// 
    /// # Arguments
    /// * `level` - Heading level (1-6 for H1-H6)
    /// 
    /// # Returns
    /// * `NumberingResult<()>` - Ok if successful, error if level is invalid
    pub fn increment_level(&mut self, level: u8) -> NumberingResult<()> {
        if level < 1 || level > 6 {
            error!(
                level = level,
                valid_range = "1-6",
                "Invalid heading level for counter increment"
            );
            return Err(NumberingError::invalid_level(level));
        }

        let index = (level - 1) as usize;
        let old_value = self.counters[index];
        
        // Check for counter overflow
        if self.counters[index] == u32::MAX {
            error!(
                level = level,
                current_value = old_value,
                max_value = u32::MAX,
                "Counter overflow detected for heading level"
            );
            
            // Reset to 1 to allow continued operation
            self.counters[index] = 1;
            
            warn!(
                level = level,
                old_value = old_value,
                new_value = self.counters[index],
                "Counter reset to 1 due to overflow"
            );
            
            return Err(NumberingError::counter_overflow(level));
        } else {
            self.counters[index] += 1;
        }

        debug!(
            level = level,
            old_value = old_value,
            new_value = self.counters[index],
            "Successfully incremented heading level counter"
        );
        
        Ok(())
    }

    /// Reset all counters for levels lower than the specified level
    /// 
    /// # Arguments
    /// * `level` - Heading level (1-6 for H1-H6)
    /// 
    /// # Returns
    /// * `NumberingResult<()>` - Ok if successful, error if level is invalid
    pub fn reset_lower_levels(&mut self, level: u8) -> NumberingResult<()> {
        if level < 1 || level > 6 {
            error!(
                level = level,
                valid_range = "1-6",
                "Invalid heading level for lower level reset"
            );
            return Err(NumberingError::invalid_level(level));
        }

        let start_index = level as usize; // This is the index after the current level
        let mut reset_count = 0;
        let mut reset_levels = Vec::new();
        
        debug!(
            current_level = level,
            reset_range = format!("{}-6", level + 1),
            "Resetting lower heading levels"
        );
        
        for i in start_index..6 {
            let level_num = (i + 1) as u8;
            let old_value = self.counters[i];
            let _was_processed = self.processed_levels[i];
            
            if self.counters[i] != 1 || self.processed_levels[i] {
                if self.counters[i] != 1 {
                    debug!(
                        level = level_num,
                        old_value = old_value,
                        new_value = 1,
                        "Resetting counter for lower level"
                    );
                    self.counters[i] = 1;
                }
                
                if self.processed_levels[i] {
                    debug!(
                        level = level_num,
                        "Marking lower level as not processed"
                    );
                    self.processed_levels[i] = false;
                }
                
                reset_count += 1;
                reset_levels.push(level_num);
            }
        }

        if reset_count > 0 {
            info!(
                current_level = level,
                reset_levels = ?reset_levels,
                reset_count = reset_count,
                "Successfully reset lower heading levels"
            );
        } else {
            trace!(
                current_level = level,
                "No lower levels needed resetting"
            );
        }

        Ok(())
    }



    /// Get the current counter value for the specified heading level
    /// 
    /// # Arguments
    /// * `level` - Heading level (1-6 for H1-H6)
    /// 
    /// # Returns
    /// * `NumberingResult<u32>` - Current counter value or error if level is invalid
    pub fn get_counter(&self, level: u8) -> NumberingResult<u32> {
        if level < 1 || level > 6 {
            error!("Invalid heading level for counter retrieval: {}", level);
            return Err(NumberingError::invalid_level(level));
        }

        let counter = self.counters[(level - 1) as usize];
        debug!("Retrieved counter for level {}: {}", level, counter);
        Ok(counter)
    }

    /// Get all counter values as a slice
    pub fn get_all_counters(&self) -> &[u32; 6] {
        &self.counters
    }

    /// Check if a specific level has been processed
    pub fn is_level_processed(&self, level: u8) -> NumberingResult<bool> {
        if level < 1 || level > 6 {
            return Err(NumberingError::invalid_level(level));
        }
        Ok(self.processed_levels[(level - 1) as usize])
    }

    /// Reset the entire state to initial values
    pub fn reset_all(&mut self) {
        let old_counters = self.counters;
        let old_processed = self.processed_levels;
        
        info!("Resetting all numbering state to initial values");
        debug!(
            old_counters = ?old_counters,
            old_processed = ?old_processed,
            "State before reset"
        );
        
        self.counters = [1; 6];
        self.processed_levels = [false; 6];
        
        debug!(
            new_counters = ?self.counters,
            new_processed = ?self.processed_levels,
            "State after reset"
        );
    }

    /// Get the counters for levels up to and including the specified level
    /// 
    /// This is useful for generating multi-level numbering like "1.2.3"
    /// 
    /// # Arguments
    /// * `level` - Maximum heading level to include (1-6 for H1-H6)
    /// 
    /// # Returns
    /// * `NumberingResult<Vec<u32>>` - Vector of counter values or error if level is invalid
    pub fn get_counters_up_to_level(&self, level: u8) -> NumberingResult<Vec<u32>> {
        if level < 1 || level > 6 {
            return Err(NumberingError::invalid_level(level));
        }

        let mut result = Vec::new();
        for i in 0..(level as usize) {
            result.push(self.counters[i]);
        }
        
        debug!("Retrieved counters up to level {}: {:?}", level, result);
        Ok(result)
    }
}

impl Default for NumberingState {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for NumberingState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "NumberingState[")?;
        for (i, counter) in self.counters.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            let processed_marker = if self.processed_levels[i] { "*" } else { "" };
            write!(f, "H{}: {}{}", i + 1, counter, processed_marker)?;
        }
        write!(f, "]")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_numbering_state() {
        let state = NumberingState::new();
        assert_eq!(state.counters, [1; 6]);
        assert_eq!(state.processed_levels, [false; 6]);
    }

    #[test]
    fn test_process_heading_sequential() {
        let mut state = NumberingState::new();
        
        // Process H1 - should be counter 1
        assert!(state.process_heading(1).is_ok());
        assert_eq!(state.get_counter(1).unwrap(), 1);
        assert!(state.is_level_processed(1).unwrap());
        
        // Process another H1 - should increment to 2
        assert!(state.process_heading(1).is_ok());
        assert_eq!(state.get_counter(1).unwrap(), 2);
        
        // Process H2 - should be counter 1, H1 stays at 2
        assert!(state.process_heading(2).is_ok());
        assert_eq!(state.get_counter(1).unwrap(), 2);
        assert_eq!(state.get_counter(2).unwrap(), 1);
        assert!(state.is_level_processed(2).unwrap());
    }

    #[test]
    fn test_process_heading_skip_level() {
        let mut state = NumberingState::new();
        
        // Process H1
        assert!(state.process_heading(1).is_ok());
        assert_eq!(state.get_counter(1).unwrap(), 1);
        
        // Skip H2 and go directly to H3
        assert!(state.process_heading(3).is_ok());
        assert_eq!(state.get_counter(1).unwrap(), 1);
        assert_eq!(state.get_counter(2).unwrap(), 1); // Should remain at default
        assert_eq!(state.get_counter(3).unwrap(), 1);
        
        // Check processing status
        assert!(state.is_level_processed(1).unwrap());
        assert!(!state.is_level_processed(2).unwrap()); // Not explicitly processed
        assert!(state.is_level_processed(3).unwrap());
    }

    #[test]
    fn test_increment_level() {
        let mut state = NumberingState::new();
        
        // Test valid level increment
        assert!(state.increment_level(1).is_ok());
        assert_eq!(state.get_counter(1).unwrap(), 2);
        
        // Test invalid level
        assert!(state.increment_level(0).is_err());
        assert!(state.increment_level(7).is_err());
    }

    #[test]
    fn test_reset_lower_levels() {
        let mut state = NumberingState::new();
        
        // Set up some counters
        state.process_heading(1).unwrap(); // H1 = 1
        state.increment_level(1).unwrap(); // H1 = 2
        state.process_heading(2).unwrap(); // H2 = 1
        state.increment_level(2).unwrap(); // H2 = 2
        state.process_heading(3).unwrap(); // H3 = 1
        state.increment_level(3).unwrap(); // H3 = 2
        
        // Reset lower levels from level 2 (this resets levels 3, 4, 5, 6)
        assert!(state.reset_lower_levels(2).is_ok());
        
        // Level 1 and 2 should be unchanged, levels 3+ should be reset to 1
        assert_eq!(state.get_counter(1).unwrap(), 2);
        assert_eq!(state.get_counter(2).unwrap(), 2);
        assert_eq!(state.get_counter(3).unwrap(), 1);
        assert_eq!(state.get_counter(4).unwrap(), 1);
        
        // Levels 3+ should be marked as not processed
        assert!(!state.is_level_processed(3).unwrap());
        assert!(!state.is_level_processed(4).unwrap());
    }

    #[test]
    fn test_get_counter() {
        let state = NumberingState::new();
        
        // Test valid levels
        for level in 1..=6 {
            assert_eq!(state.get_counter(level).unwrap(), 1);
        }
        
        // Test invalid levels
        assert!(state.get_counter(0).is_err());
        assert!(state.get_counter(7).is_err());
    }

    #[test]
    fn test_get_counters_up_to_level() {
        let mut state = NumberingState::new();
        
        // Set up some counters
        state.process_heading(1).unwrap();
        state.increment_level(1).unwrap(); // H1 = 2
        state.process_heading(2).unwrap(); // H2 = 1
        state.increment_level(2).unwrap(); // H2 = 2
        state.process_heading(3).unwrap(); // H3 = 1
        
        // Test getting counters up to different levels
        assert_eq!(state.get_counters_up_to_level(1).unwrap(), vec![2]);
        assert_eq!(state.get_counters_up_to_level(2).unwrap(), vec![2, 2]);
        assert_eq!(state.get_counters_up_to_level(3).unwrap(), vec![2, 2, 1]);
        
        // Test invalid level
        assert!(state.get_counters_up_to_level(0).is_err());
        assert!(state.get_counters_up_to_level(7).is_err());
    }

    #[test]
    fn test_reset_all() {
        let mut state = NumberingState::new();
        
        // Set up some state
        state.process_heading(1).unwrap();
        state.increment_level(1).unwrap();
        state.process_heading(2).unwrap();
        
        // Reset all
        state.reset_all();
        
        // Check that everything is back to initial state
        assert_eq!(state.counters, [1; 6]);
        assert_eq!(state.processed_levels, [false; 6]);
    }

    #[test]
    fn test_counter_overflow() {
        let mut state = NumberingState::new();
        state.counters[0] = u32::MAX;
        
        // Should handle overflow gracefully
        let result = state.increment_level(1);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), NumberingError::CounterOverflow(1)));
        assert_eq!(state.get_counter(1).unwrap(), 1); // Should reset to 1
    }

    #[test]
    fn test_complex_skip_level_scenario() {
        let mut state = NumberingState::new();
        
        // H1
        state.process_heading(1).unwrap();
        assert_eq!(state.get_counter(1).unwrap(), 1);
        
        // Skip to H4 directly
        state.process_heading(4).unwrap();
        assert_eq!(state.get_counter(1).unwrap(), 1);
        assert_eq!(state.get_counter(2).unwrap(), 1); // Should remain at default
        assert_eq!(state.get_counter(3).unwrap(), 1); // Should remain at default
        assert_eq!(state.get_counter(4).unwrap(), 1);
        
        // Only explicitly processed levels should be marked as processed
        assert!(state.is_level_processed(1).unwrap());
        assert!(!state.is_level_processed(2).unwrap());
        assert!(!state.is_level_processed(3).unwrap());
        assert!(state.is_level_processed(4).unwrap());
        
        // Go back to H2 - first time processing H2, so should stay at 1
        state.process_heading(2).unwrap();
        assert_eq!(state.get_counter(2).unwrap(), 1);
        
        // H3 and H4 should be reset
        assert_eq!(state.get_counter(3).unwrap(), 1);
        assert_eq!(state.get_counter(4).unwrap(), 1);
        assert!(!state.is_level_processed(3).unwrap());
        assert!(!state.is_level_processed(4).unwrap());
    }

    #[test]
    fn test_display() {
        let mut state = NumberingState::new();
        state.process_heading(1).unwrap();
        state.process_heading(2).unwrap();
        
        let display = format!("{}", state);
        assert!(display.contains("NumberingState["));
        assert!(display.contains("H1: 1*")); // Processed levels have *
        assert!(display.contains("H2: 1*"));
        assert!(display.contains("H3: 1")); // Unprocessed levels don't have *
    }
}