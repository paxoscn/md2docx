//! # Numbering Logging Utilities
//! 
//! This module provides specialized logging utilities for the numbering functionality,
//! including error tracking, performance monitoring, and degradation metrics.

use crate::numbering::error::{NumberingError, NumberingResult};
use crate::numbering::state::NumberingState;
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tracing::{info, warn, error, debug, trace, event, Level};
use serde_json::json;

/// Metrics collector for numbering operations
#[derive(Debug, Clone, Default)]
pub struct NumberingMetrics {
    /// Total number of headings processed
    pub total_headings: u64,
    /// Number of successful numbering operations
    pub successful_operations: u64,
    /// Number of failed operations that used graceful degradation
    pub degraded_operations: u64,
    /// Number of critical failures
    pub critical_failures: u64,
    /// Error counts by category
    pub error_counts: HashMap<String, u64>,
    /// Processing times for performance monitoring
    pub processing_times: Vec<Duration>,
    /// State reset count
    pub state_resets: u64,
    /// Format validation failures
    pub validation_failures: u64,
}

impl NumberingMetrics {
    /// Create new metrics collector
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Record a successful operation
    pub fn record_success(&mut self, duration: Duration) {
        self.total_headings += 1;
        self.successful_operations += 1;
        self.processing_times.push(duration);
        
        trace!(
            operation = "numbering_success",
            duration_us = duration.as_micros(),
            total_headings = self.total_headings,
            "Recorded successful numbering operation"
        );
    }
    
    /// Record a degraded operation (error with fallback)
    pub fn record_degradation(&mut self, error: &NumberingError, duration: Duration) {
        self.total_headings += 1;
        self.degraded_operations += 1;
        self.processing_times.push(duration);
        
        let category = error.category().to_string();
        *self.error_counts.entry(category.clone()).or_insert(0) += 1;
        
        warn!(
            operation = "numbering_degradation",
            error_category = category,
            error = %error,
            duration_us = duration.as_micros(),
            total_degraded = self.degraded_operations,
            "Recorded degraded numbering operation"
        );
    }
    
    /// Record a critical failure
    pub fn record_failure(&mut self, error: &NumberingError, duration: Duration) {
        self.total_headings += 1;
        self.critical_failures += 1;
        self.processing_times.push(duration);
        
        let category = error.category().to_string();
        *self.error_counts.entry(category.clone()).or_insert(0) += 1;
        
        error!(
            operation = "numbering_failure",
            error_category = category,
            error = %error,
            duration_us = duration.as_micros(),
            total_failures = self.critical_failures,
            "Recorded critical numbering failure"
        );
    }
    
    /// Record a state reset
    pub fn record_state_reset(&mut self) {
        self.state_resets += 1;
        
        debug!(
            operation = "state_reset",
            total_resets = self.state_resets,
            "Recorded numbering state reset"
        );
    }
    
    /// Record a validation failure
    pub fn record_validation_failure(&mut self, format: &str, error: &NumberingError) {
        self.validation_failures += 1;
        
        error!(
            operation = "validation_failure",
            format = format,
            error = %error,
            total_validation_failures = self.validation_failures,
            "Recorded numbering format validation failure"
        );
    }
    
    /// Get success rate as percentage
    pub fn success_rate(&self) -> f64 {
        if self.total_headings == 0 {
            return 100.0;
        }
        (self.successful_operations as f64 / self.total_headings as f64) * 100.0
    }
    
    /// Get degradation rate as percentage
    pub fn degradation_rate(&self) -> f64 {
        if self.total_headings == 0 {
            return 0.0;
        }
        (self.degraded_operations as f64 / self.total_headings as f64) * 100.0
    }
    
    /// Get failure rate as percentage
    pub fn failure_rate(&self) -> f64 {
        if self.total_headings == 0 {
            return 0.0;
        }
        (self.critical_failures as f64 / self.total_headings as f64) * 100.0
    }
    
    /// Get average processing time
    pub fn average_processing_time(&self) -> Option<Duration> {
        if self.processing_times.is_empty() {
            return None;
        }
        
        let total_nanos: u128 = self.processing_times.iter()
            .map(|d| d.as_nanos())
            .sum();
        
        let avg_nanos = total_nanos / self.processing_times.len() as u128;
        Some(Duration::from_nanos(avg_nanos as u64))
    }
    
    /// Log comprehensive metrics summary
    pub fn log_summary(&self) {
        let avg_time = self.average_processing_time()
            .map(|d| d.as_micros())
            .unwrap_or(0);
        
        info!(
            total_headings = self.total_headings,
            successful_operations = self.successful_operations,
            degraded_operations = self.degraded_operations,
            critical_failures = self.critical_failures,
            success_rate_percent = self.success_rate(),
            degradation_rate_percent = self.degradation_rate(),
            failure_rate_percent = self.failure_rate(),
            average_processing_time_us = avg_time,
            state_resets = self.state_resets,
            validation_failures = self.validation_failures,
            "Numbering functionality metrics summary"
        );
    }
    
    /// Check if metrics indicate concerning patterns
    pub fn check_health(&self) -> HealthStatus {
        let degradation_rate = self.degradation_rate();
        let failure_rate = self.failure_rate();
        
        if failure_rate > 10.0 {
            HealthStatus::Critical
        } else if degradation_rate > 25.0 || failure_rate > 5.0 {
            HealthStatus::Warning
        } else if degradation_rate > 10.0 || failure_rate > 1.0 {
            HealthStatus::Degraded
        } else {
            HealthStatus::Healthy
        }
    }
    
    /// Log health status
    pub fn log_health_status(&self) {
        let status = self.check_health();
        let level = match status {
            HealthStatus::Healthy => Level::DEBUG,
            HealthStatus::Degraded => Level::INFO,
            HealthStatus::Warning => Level::WARN,
            HealthStatus::Critical => Level::ERROR,
        };
        
        match level {
            Level::DEBUG => debug!(
                health_status = ?status,
                success_rate = self.success_rate(),
                degradation_rate = self.degradation_rate(),
                failure_rate = self.failure_rate(),
                total_operations = self.total_headings,
                "Numbering functionality health status"
            ),
            Level::INFO => info!(
                health_status = ?status,
                success_rate = self.success_rate(),
                degradation_rate = self.degradation_rate(),
                failure_rate = self.failure_rate(),
                total_operations = self.total_headings,
                "Numbering functionality health status"
            ),
            Level::WARN => warn!(
                health_status = ?status,
                success_rate = self.success_rate(),
                degradation_rate = self.degradation_rate(),
                failure_rate = self.failure_rate(),
                total_operations = self.total_headings,
                "Numbering functionality health status"
            ),
            Level::ERROR => error!(
                health_status = ?status,
                success_rate = self.success_rate(),
                degradation_rate = self.degradation_rate(),
                failure_rate = self.failure_rate(),
                total_operations = self.total_headings,
                "Numbering functionality health status"
            ),
            _ => debug!(
                health_status = ?status,
                success_rate = self.success_rate(),
                degradation_rate = self.degradation_rate(),
                failure_rate = self.failure_rate(),
                total_operations = self.total_headings,
                "Numbering functionality health status"
            ),
        }
    }
}

/// Health status of numbering functionality
#[derive(Debug, Clone, PartialEq)]
pub enum HealthStatus {
    /// All operations successful
    Healthy,
    /// Some degradation but within acceptable limits
    Degraded,
    /// Concerning levels of degradation or failures
    Warning,
    /// High failure rates requiring attention
    Critical,
}

/// Logging utilities for numbering operations
pub struct NumberingLogger;

impl NumberingLogger {
    /// Log the start of a numbering operation
    pub fn log_operation_start(level: u8, text: &str, has_numbering: bool) -> Instant {
        trace!(
            operation = "numbering_start",
            level = level,
            text_length = text.len(),
            has_numbering = has_numbering,
            "Starting heading numbering operation"
        );
        Instant::now()
    }
    
    /// Log successful numbering operation
    pub fn log_operation_success(
        level: u8,
        original_text: &str,
        numbered_text: &str,
        start_time: Instant,
    ) {
        let duration = start_time.elapsed();
        
        debug!(
            operation = "numbering_success",
            level = level,
            original_length = original_text.len(),
            numbered_length = numbered_text.len(),
            duration_us = duration.as_micros(),
            "Successfully completed heading numbering"
        );
    }
    
    /// Log numbering error with context
    pub fn log_numbering_error(
        level: u8,
        text: &str,
        error: &NumberingError,
        start_time: Instant,
        used_fallback: bool,
    ) {
        let duration = start_time.elapsed();
        
        let log_level = if error.is_recoverable() && used_fallback {
            Level::WARN
        } else {
            Level::ERROR
        };
        
        if used_fallback {
            warn!(
                operation = "numbering_error",
                level = level,
                text_length = text.len(),
                error = %error,
                error_category = error.category(),
                recoverable = error.is_recoverable(),
                used_fallback = used_fallback,
                duration_us = duration.as_micros(),
                "Numbering operation encountered error but used fallback"
            );
        } else {
            error!(
                operation = "numbering_error",
                level = level,
                text_length = text.len(),
                error = %error,
                error_category = error.category(),
                recoverable = error.is_recoverable(),
                used_fallback = used_fallback,
                duration_us = duration.as_micros(),
                "Numbering operation encountered error"
            );
        }
    }
    
    /// Log state management operations
    pub fn log_state_operation(operation: &str, level: Option<u8>, state: &NumberingState) {
        debug!(
            operation = operation,
            level = level,
            state_counters = ?state.get_all_counters(),
            "Numbering state operation"
        );
    }
    
    /// Log format validation results
    pub fn log_format_validation(format: &str, result: &NumberingResult<()>) {
        match result {
            Ok(_) => {
                trace!(
                    operation = "format_validation",
                    format = format,
                    result = "valid",
                    "Numbering format validation successful"
                );
            }
            Err(e) => {
                error!(
                    operation = "format_validation",
                    format = format,
                    error = %e,
                    error_category = e.category(),
                    result = "invalid",
                    "Numbering format validation failed"
                );
            }
        }
    }
    
    /// Log configuration changes
    pub fn log_config_change(
        old_numbered_levels: &[u8],
        new_numbered_levels: &[u8],
    ) {
        let added_levels: Vec<u8> = new_numbered_levels.iter()
            .filter(|level| !old_numbered_levels.contains(level))
            .copied()
            .collect();
        
        let removed_levels: Vec<u8> = old_numbered_levels.iter()
            .filter(|level| !new_numbered_levels.contains(level))
            .copied()
            .collect();
        
        info!(
            operation = "config_change",
            old_levels = ?old_numbered_levels,
            new_levels = ?new_numbered_levels,
            added_levels = ?added_levels,
            removed_levels = ?removed_levels,
            "Numbering configuration changed"
        );
    }
    
    /// Log batch processing results
    pub fn log_batch_results(
        total_headings: usize,
        successful: usize,
        degraded: usize,
        failed: usize,
        duration: Duration,
    ) {
        let success_rate = (successful as f64 / total_headings as f64) * 100.0;
        let degradation_rate = (degraded as f64 / total_headings as f64) * 100.0;
        let failure_rate = (failed as f64 / total_headings as f64) * 100.0;
        
        info!(
            operation = "batch_processing",
            total_headings = total_headings,
            successful = successful,
            degraded = degraded,
            failed = failed,
            success_rate = success_rate,
            degradation_rate = degradation_rate,
            failure_rate = failure_rate,
            duration_ms = duration.as_millis(),
            "Batch heading processing completed"
        );
    }
    
    /// Log performance metrics
    pub fn log_performance_metrics(
        operation_count: u64,
        total_duration: Duration,
        avg_duration: Duration,
        max_duration: Duration,
        min_duration: Duration,
    ) {
        info!(
            operation_count = operation_count,
            total_duration_ms = total_duration.as_millis(),
            average_duration_us = avg_duration.as_micros(),
            max_duration_us = max_duration.as_micros(),
            min_duration_us = min_duration.as_micros(),
            operations_per_second = operation_count as f64 / total_duration.as_secs_f64(),
            "Numbering performance metrics"
        );
    }
}

/// Macro for timing numbering operations
#[macro_export]
macro_rules! time_numbering_operation {
    ($operation:expr, $block:block) => {{
        let start = std::time::Instant::now();
        let result = $block;
        let duration = start.elapsed();
        
        tracing::trace!(
            operation = $operation,
            duration_us = duration.as_micros(),
            "Timed numbering operation"
        );
        
        result
    }};
}

/// Macro for logging numbering state changes
#[macro_export]
macro_rules! log_state_change {
    ($operation:expr, $level:expr, $old_state:expr, $new_state:expr) => {
        tracing::debug!(
            operation = $operation,
            level = $level,
            old_counters = ?$old_state.get_all_counters(),
            new_counters = ?$new_state.get_all_counters(),
            "Numbering state changed"
        );
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::numbering::error::NumberingError;
    use std::time::Duration;

    #[test]
    fn test_metrics_creation() {
        let metrics = NumberingMetrics::new();
        assert_eq!(metrics.total_headings, 0);
        assert_eq!(metrics.successful_operations, 0);
        assert_eq!(metrics.degraded_operations, 0);
        assert_eq!(metrics.critical_failures, 0);
    }

    #[test]
    fn test_success_recording() {
        let mut metrics = NumberingMetrics::new();
        let duration = Duration::from_millis(10);
        
        metrics.record_success(duration);
        
        assert_eq!(metrics.total_headings, 1);
        assert_eq!(metrics.successful_operations, 1);
        assert_eq!(metrics.success_rate(), 100.0);
    }

    #[test]
    fn test_degradation_recording() {
        let mut metrics = NumberingMetrics::new();
        let error = NumberingError::invalid_format("test");
        let duration = Duration::from_millis(15);
        
        metrics.record_degradation(&error, duration);
        
        assert_eq!(metrics.total_headings, 1);
        assert_eq!(metrics.degraded_operations, 1);
        assert_eq!(metrics.degradation_rate(), 100.0);
        assert_eq!(metrics.error_counts.get("format"), Some(&1));
    }

    #[test]
    fn test_failure_recording() {
        let mut metrics = NumberingMetrics::new();
        let error = NumberingError::invalid_level(0);
        let duration = Duration::from_millis(5);
        
        metrics.record_failure(&error, duration);
        
        assert_eq!(metrics.total_headings, 1);
        assert_eq!(metrics.critical_failures, 1);
        assert_eq!(metrics.failure_rate(), 100.0);
        assert_eq!(metrics.error_counts.get("level"), Some(&1));
    }

    #[test]
    fn test_mixed_operations() {
        let mut metrics = NumberingMetrics::new();
        
        // Record various operations
        metrics.record_success(Duration::from_millis(10));
        metrics.record_success(Duration::from_millis(12));
        metrics.record_degradation(&NumberingError::invalid_format("test"), Duration::from_millis(15));
        metrics.record_failure(&NumberingError::invalid_level(0), Duration::from_millis(5));
        
        assert_eq!(metrics.total_headings, 4);
        assert_eq!(metrics.successful_operations, 2);
        assert_eq!(metrics.degraded_operations, 1);
        assert_eq!(metrics.critical_failures, 1);
        assert_eq!(metrics.success_rate(), 50.0);
        assert_eq!(metrics.degradation_rate(), 25.0);
        assert_eq!(metrics.failure_rate(), 25.0);
    }

    #[test]
    fn test_average_processing_time() {
        let mut metrics = NumberingMetrics::new();
        
        metrics.record_success(Duration::from_millis(10));
        metrics.record_success(Duration::from_millis(20));
        
        let avg = metrics.average_processing_time().unwrap();
        assert_eq!(avg, Duration::from_millis(15));
    }

    #[test]
    fn test_health_status() {
        let mut metrics = NumberingMetrics::new();
        
        // Healthy state
        for _ in 0..10 {
            metrics.record_success(Duration::from_millis(10));
        }
        assert_eq!(metrics.check_health(), HealthStatus::Healthy);
        
        // Degraded state
        metrics.record_degradation(&NumberingError::invalid_format("test"), Duration::from_millis(10));
        assert_eq!(metrics.check_health(), HealthStatus::Degraded);
        
        // Warning state
        for _ in 0..3 {
            metrics.record_degradation(&NumberingError::invalid_format("test"), Duration::from_millis(10));
        }
        assert_eq!(metrics.check_health(), HealthStatus::Warning);
        
        // Critical state
        for _ in 0..2 {
            metrics.record_failure(&NumberingError::invalid_level(0), Duration::from_millis(5));
        }
        assert_eq!(metrics.check_health(), HealthStatus::Critical);
    }

    #[test]
    fn test_state_reset_recording() {
        let mut metrics = NumberingMetrics::new();
        
        metrics.record_state_reset();
        metrics.record_state_reset();
        
        assert_eq!(metrics.state_resets, 2);
    }

    #[test]
    fn test_validation_failure_recording() {
        let mut metrics = NumberingMetrics::new();
        let error = NumberingError::invalid_format("bad format");
        
        metrics.record_validation_failure("%invalid", &error);
        
        assert_eq!(metrics.validation_failures, 1);
    }
}