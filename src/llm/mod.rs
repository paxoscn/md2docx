//! LLM integration module for natural language configuration updates

pub mod client;
pub mod prompts;

pub use client::{LlmClient, LlmConfig, LlmProvider};
pub use prompts::*;