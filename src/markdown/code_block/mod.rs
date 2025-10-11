//! Code block processing strategy system
//! 
//! This module provides a strategy pattern implementation for processing different
//! types of code blocks in Markdown documents. Each programming language can have
//! its own processing strategy for syntax validation, formatting, and other
//! language-specific operations.

pub mod strategy;
pub mod registry;
pub mod error;
pub mod config;
pub mod processor;
pub mod strategies;
pub mod integration;
pub mod timeout;
pub mod enhanced_processor;
pub mod cache;
pub mod lazy_loading;
pub mod parallel;
pub mod memory;
pub mod performance;
pub mod memory_profiler;
pub mod performance_optimizer;
pub mod plugin;
pub mod example_plugin;
pub mod plugin_example;

pub use strategy::*;
pub use registry::*;
pub use error::*;
pub use config::*;
pub use processor::*;
pub use strategies::*;
pub use integration::*;
pub use timeout::*;
pub use enhanced_processor::*;
pub use cache::*;
pub use lazy_loading::*;
pub use parallel::*;
pub use memory::*;
pub use performance::*;
pub use memory_profiler::*;
pub use performance_optimizer::*;
pub use plugin::*;
pub use example_plugin::*;
pub use plugin_example::*;