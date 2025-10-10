//! # Numbering Module
//! 
//! This module provides automatic heading numbering functionality for the markdown to docx converter.
//! It supports configurable numbering formats and maintains state across heading levels.
//! 
//! ## Error Handling and Logging
//! 
//! The numbering module implements comprehensive error handling with graceful degradation:
//! - Invalid formats fall back to original text
//! - Counter overflows reset to 1 and continue
//! - State errors are logged but don't stop processing
//! - All operations are instrumented for monitoring
//! 
//! ## Monitoring and Metrics
//! 
//! The module provides detailed metrics and health monitoring:
//! - Success/failure rates
//! - Performance metrics
//! - Error categorization
//! - Degradation tracking

pub mod error;
pub mod formatter;
pub mod logging;
pub mod processor;
pub mod state;

#[cfg(test)]
mod tests;

// Re-export main types
pub use error::{NumberingError, NumberingResult};
pub use formatter::{NumberingFormatter, NumberingFormat};
pub use logging::{NumberingMetrics, NumberingLogger, HealthStatus};
pub use processor::HeadingProcessor;
pub use state::NumberingState;