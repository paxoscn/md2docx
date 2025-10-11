# Plugin Development Guide

## Overview

This guide will walk you through creating plugins for the Code Block Strategy System. Plugins allow you to extend the system with support for new programming languages or custom processing logic without modifying the core codebase.

## Table of Contents

1. [Plugin Architecture](#plugin-architecture)
2. [Getting Started](#getting-started)
3. [Basic Plugin Implementation](#basic-plugin-implementation)
4. [Advanced Plugin Features](#advanced-plugin-features)
5. [Testing Your Plugin](#testing-your-plugin)
6. [Distribution and Packaging](#distribution-and-packaging)
7. [Best Practices](#best-practices)
8. [Examples](#examples)

## Plugin Architecture

### Core Components

The plugin system consists of several key components:

- **`CodeBlockPlugin` Trait**: The main interface that plugins must implement
- **`PluginManager`**: Manages plugin lifecycle (loading, configuration, unloading)
- **`StrategyRegistry`**: Registers and manages code block processing strategies
- **Plugin Configuration**: YAML-based configuration system for plugins

### Plugin Lifecycle

1. **Loading**: Plugin is loaded into the PluginManager
2. **Initialization**: Plugin's `initialize()` method is called
3. **Strategy Creation**: Plugin creates and registers its strategy
4. **Configuration**: Plugin receives configuration from YAML files or API calls
5. **Processing**: Plugin's strategy processes code blocks
6. **Cleanup**: Plugin's `cleanup()` method is called when unloading

## Getting Started

### Prerequisites

- Rust 1.70 or later
- Understanding of the `CodeBlockStrategy` trait
- Familiarity with the core data structures (`ProcessedCodeBlock`, `ProcessingConfig`, etc.)

### Project Structure

Create a new Rust library for your plugin:

```bash
cargo new --lib my_language_plugin
cd my_language_plugin
```

Add dependencies to `Cargo.toml`:

```toml
[package]
name = "my_language_plugin"
version = "0.1.0"
edition = "2021"

[dependencies]
# Add the main crate as a dependency
md2docx = { path = "../path/to/main/crate" }

# Add any language-specific dependencies
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# Example: For a Python plugin, you might add:
# pyo3 = "0.19"

# Example: For a JavaScript plugin, you might add:
# swc_ecma_parser = "0.140"
```

## Basic Plugin Implementation

### Step 1: Define Your Plugin Structure

```rust
// src/lib.rs
use std::collections::HashMap;
use md2docx::markdown::code_block::{
    CodeBlockPlugin, CodeBlockStrategy, PluginError, PluginConfigSchema,
    ConfigOption, ConfigOptionType, ProcessedCodeBlock, ProcessingConfig,
    ProcessingError, ProcessingMetadata
};

/// Plugin for processing MyLanguage code blocks
#[derive(Debug)]
pub struct MyLanguagePlugin {
    config: MyLanguageConfig,
}

/// Configuration specific to MyLanguage processing
#[derive(Debug, Clone)]
pub struct MyLanguageConfig {
    pub enable_strict_mode: bool,
    pub max_line_length: usize,
    pub custom_rules: Vec<String>,
}

impl Default for MyLanguageConfig {
    fn default() -> Self {
        Self {
            enable_strict_mode: false,
            max_line_length: 80,
            custom_rules: Vec::new(),
        }
    }
}

impl MyLanguagePlugin {
    /// Create a new plugin instance with default configuration
    pub fn new() -> Self {
        Self {
            config: MyLanguageConfig::default(),
        }
    }
    
    /// Create a plugin with custom configuration
    pub fn with_config(config: MyLanguageConfig) -> Self {
        Self { config }
    }
}
```

### Step 2: Implement the CodeBlockPlugin Trait

```rust
impl CodeBlockPlugin for MyLanguagePlugin {
    fn name(&self) -> &str {
        "mylanguage-processor"
    }
    
    fn version(&self) -> &str {
        "1.0.0"
    }
    
    fn description(&self) -> &str {
        "MyLanguage code block processor with syntax validation and formatting"
    }
    
    fn author(&self) -> &str {
        "Your Name <your.email@example.com>"
    }
    
    fn create_strategy(&self) -> Result<Box<dyn CodeBlockStrategy>, PluginError> {
        Ok(Box::new(MyLanguageStrategy::new(self.config.clone())))
    }
    
    fn supported_languages(&self) -> Vec<String> {
        vec![
            "mylanguage".to_string(),
            "mylang".to_string(),
            "ml".to_string(),
        ]
    }
    
    fn initialize(&self) -> Result<(), PluginError> {
        // Perform any initialization logic here
        // Check dependencies, validate configuration, etc.
        
        println!("Initializing MyLanguage plugin v{}", self.version());
        
        // Example: Check if external tools are available
        if self.config.enable_strict_mode {
            // Verify that strict mode dependencies are available
            self.check_strict_mode_dependencies()?;
        }
        
        Ok(())
    }
    
    fn cleanup(&self) -> Result<(), PluginError> {
        // Clean up any resources
        println!("Cleaning up MyLanguage plugin");
        Ok(())
    }
    
    fn get_config_schema(&self) -> Option<PluginConfigSchema> {
        Some(PluginConfigSchema {
            options: vec![
                ConfigOption {
                    name: "enable_strict_mode".to_string(),
                    description: "Enable strict syntax validation".to_string(),
                    option_type: ConfigOptionType::Boolean,
                    default_value: Some("false".to_string()),
                    required: false,
                },
                ConfigOption {
                    name: "max_line_length".to_string(),
                    description: "Maximum line length for formatting".to_string(),
                    option_type: ConfigOptionType::Integer,
                    default_value: Some("80".to_string()),
                    required: false,
                },
                ConfigOption {
                    name: "custom_rules".to_string(),
                    description: "Comma-separated list of custom validation rules".to_string(),
                    option_type: ConfigOptionType::List,
                    default_value: Some("".to_string()),
                    required: false,
                },
            ],
            required_options: vec![],
        })
    }
    
    fn configure(&mut self, config: &HashMap<String, String>) -> Result<(), PluginError> {
        // Update plugin configuration based on provided options
        
        if let Some(strict_mode) = config.get("enable_strict_mode") {
            self.config.enable_strict_mode = strict_mode.parse().map_err(|_| {
                PluginError::configuration_error("Invalid value for enable_strict_mode")
            })?;
        }
        
        if let Some(max_line_length) = config.get("max_line_length") {
            self.config.max_line_length = max_line_length.parse().map_err(|_| {
                PluginError::configuration_error("Invalid value for max_line_length")
            })?;
        }
        
        if let Some(custom_rules) = config.get("custom_rules") {
            self.config.custom_rules = custom_rules
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
        }
        
        Ok(())
    }
}

impl MyLanguagePlugin {
    fn check_strict_mode_dependencies(&self) -> Result<(), PluginError> {
        // Example: Check if external validator is available
        // This is where you'd verify that required tools or libraries are installed
        
        // For demonstration, we'll just check a simple condition
        if self.config.max_line_length < 10 {
            return Err(PluginError::initialization_error(
                "max_line_length must be at least 10 for strict mode"
            ));
        }
        
        Ok(())
    }
}
```

### Step 3: Implement the Strategy

```rust
/// Strategy implementation for MyLanguage code processing
#[derive(Debug, Clone)]
pub struct MyLanguageStrategy {
    config: MyLanguageConfig,
}

impl MyLanguageStrategy {
    pub fn new(config: MyLanguageConfig) -> Self {
        Self { config }
    }
    
    /// Validate MyLanguage syntax
    fn validate_syntax(&self, code: &str) -> Result<bool, ProcessingError> {
        // Implement your language-specific syntax validation here
        
        let lines: Vec<&str> = code.lines().collect();
        
        for (line_num, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            
            // Skip empty lines and comments
            if trimmed.is_empty() || trimmed.starts_with("//") {
                continue;
            }
            
            // Example validation rules for MyLanguage
            if self.config.enable_strict_mode {
                // Strict mode validation
                if trimmed.len() > self.config.max_line_length {
                    return Err(ProcessingError::syntax_error(
                        &format!("Line exceeds maximum length of {}", self.config.max_line_length),
                        Some(line_num + 1),
                        Some(trimmed.len())
                    ));
                }
                
                // Check for required semicolons (example rule)
                if !trimmed.ends_with(';') && !trimmed.ends_with('{') && !trimmed.ends_with('}') {
                    return Err(ProcessingError::syntax_error(
                        "Missing semicolon at end of statement",
                        Some(line_num + 1),
                        Some(trimmed.len())
                    ));
                }
            }
            
            // Apply custom rules
            for rule in &self.config.custom_rules {
                if let Err(e) = self.apply_custom_rule(rule, trimmed, line_num + 1) {
                    return Err(e);
                }
            }
        }
        
        Ok(true)
    }
    
    /// Format MyLanguage code
    fn format_code(&self, code: &str) -> Result<String, ProcessingError> {
        let lines: Vec<&str> = code.lines().collect();
        let mut formatted_lines = Vec::new();
        let mut indent_level = 0;
        
        for line in lines {
            let trimmed = line.trim();
            
            if trimmed.is_empty() {
                formatted_lines.push(String::new());
                continue;
            }
            
            // Adjust indentation based on braces
            if trimmed.starts_with('}') {
                indent_level = indent_level.saturating_sub(1);
            }
            
            // Apply indentation
            let indented = format!("{}{}", "    ".repeat(indent_level), trimmed);
            formatted_lines.push(indented);
            
            // Increase indent after opening brace
            if trimmed.ends_with('{') {
                indent_level += 1;
            }
        }
        
        Ok(formatted_lines.join("\n"))
    }
    
    /// Apply a custom validation rule
    fn apply_custom_rule(&self, rule: &str, line: &str, line_num: usize) -> Result<(), ProcessingError> {
        match rule {
            "no_tabs" => {
                if line.contains('\t') {
                    return Err(ProcessingError::syntax_error(
                        "Tabs are not allowed, use spaces instead",
                        Some(line_num),
                        None
                    ));
                }
            }
            "no_trailing_whitespace" => {
                if line.ends_with(' ') || line.ends_with('\t') {
                    return Err(ProcessingError::syntax_error(
                        "Trailing whitespace is not allowed",
                        Some(line_num),
                        Some(line.len())
                    ));
                }
            }
            _ => {
                // Unknown rule - could log a warning here
            }
        }
        
        Ok(())
    }
    
    /// Generate code quality warnings
    fn check_code_quality(&self, code: &str) -> Vec<md2docx::markdown::code_block::ProcessingWarning> {
        let mut warnings = Vec::new();
        
        // Example quality checks
        if code.contains("TODO") {
            warnings.push(md2docx::markdown::code_block::ProcessingWarning::new(
                "code_quality",
                "Code contains TODO comments"
            ));
        }
        
        if code.contains("FIXME") {
            warnings.push(md2docx::markdown::code_block::ProcessingWarning::new(
                "code_quality",
                "Code contains FIXME comments"
            ));
        }
        
        // Check for very long functions (simple heuristic)
        let line_count = code.lines().count();
        if line_count > 50 {
            warnings.push(md2docx::markdown::code_block::ProcessingWarning::new(
                "code_quality",
                "Code block is very long, consider breaking it up"
            ));
        }
        
        warnings
    }
}

impl CodeBlockStrategy for MyLanguageStrategy {
    fn process(&self, code: &str, config: &ProcessingConfig) -> Result<ProcessedCodeBlock, ProcessingError> {
        let start_time = std::time::Instant::now();
        let mut errors = Vec::new();
        let mut warnings = Vec::new();
        
        // Validate syntax if enabled
        let syntax_valid = if config.enable_syntax_validation {
            match self.validate_syntax(code) {
                Ok(valid) => valid,
                Err(e) => {
                    errors.push(e);
                    false
                }
            }
        } else {
            true
        };
        
        // Format code if enabled and syntax is valid
        let formatted_code = if config.enable_formatting && syntax_valid {
            match self.format_code(code) {
                Ok(formatted) => Some(formatted),
                Err(e) => {
                    warnings.push(md2docx::markdown::code_block::ProcessingWarning::formatting_warning(&e.to_string()));
                    None
                }
            }
        } else {
            None
        };
        
        // Check code quality
        if syntax_valid {
            warnings.extend(self.check_code_quality(code));
        }
        
        // Create metadata
        let mut metadata = ProcessingMetadata::new(self.get_version())
            .with_processing_time(start_time.elapsed());
        
        metadata.is_formatted = formatted_code.is_some();
        metadata.is_validated = config.enable_syntax_validation;
        metadata.syntax_valid = syntax_valid;
        
        // Add custom attributes
        metadata = metadata.with_custom_attribute("language", "mylanguage");
        metadata = metadata.with_custom_attribute("strict_mode", &self.config.enable_strict_mode.to_string());
        
        // Build the result
        let mut processed = ProcessedCodeBlock::new(code.to_string(), Some("mylanguage".to_string()))
            .with_metadata(metadata);
        
        if let Some(formatted) = formatted_code {
            processed = processed.with_processed_code(formatted);
        }
        
        for error in errors {
            processed = processed.with_error(error);
        }
        
        for warning in warnings {
            processed = processed.with_warning(warning);
        }
        
        Ok(processed)
    }
    
    fn supports_language(&self, language: &str) -> bool {
        matches!(language.to_lowercase().as_str(), "mylanguage" | "mylang" | "ml")
    }
    
    fn get_language_name(&self) -> &'static str {
        "mylanguage"
    }
    
    fn get_priority(&self) -> u8 {
        110 // Higher than default, lower than built-in languages
    }
    
    fn get_version(&self) -> &'static str {
        "1.0.0"
    }
    
    fn get_description(&self) -> &'static str {
        "MyLanguage code processing with syntax validation and formatting"
    }
}
```

## Advanced Plugin Features

### Configuration Schema Validation

```rust
impl MyLanguagePlugin {
    fn validate_config(&self, config: &HashMap<String, String>) -> Result<(), PluginError> {
        // Validate configuration values before applying them
        
        if let Some(max_line_length) = config.get("max_line_length") {
            let length: usize = max_line_length.parse().map_err(|_| {
                PluginError::configuration_error("max_line_length must be a positive integer")
            })?;
            
            if length < 10 || length > 1000 {
                return Err(PluginError::configuration_error(
                    "max_line_length must be between 10 and 1000"
                ));
            }
        }
        
        Ok(())
    }
}
```

### External Tool Integration

```rust
use std::process::Command;

impl MyLanguageStrategy {
    fn format_with_external_tool(&self, code: &str) -> Result<String, ProcessingError> {
        // Example: Use an external formatter
        let mut child = Command::new("mylang-fmt")
            .arg("--stdin")
            .arg("--stdout")
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .map_err(|e| ProcessingError::formatting_error(&format!("Failed to start formatter: {}", e)))?;
        
        // Write code to stdin
        if let Some(stdin) = child.stdin.as_mut() {
            use std::io::Write;
            stdin.write_all(code.as_bytes())
                .map_err(|e| ProcessingError::formatting_error(&format!("Failed to write to formatter: {}", e)))?;
        }
        
        // Read formatted output
        let output = child.wait_with_output()
            .map_err(|e| ProcessingError::formatting_error(&format!("Failed to read formatter output: {}", e)))?;
        
        if output.status.success() {
            String::from_utf8(output.stdout)
                .map_err(|e| ProcessingError::formatting_error(&format!("Invalid UTF-8 from formatter: {}", e)))
        } else {
            let error_msg = String::from_utf8_lossy(&output.stderr);
            Err(ProcessingError::formatting_error(&format!("Formatter failed: {}", error_msg)))
        }
    }
}
```

### Async Processing Support

```rust
use std::future::Future;
use std::pin::Pin;

// Note: This is a conceptual example - the actual trait would need to support async
pub trait AsyncCodeBlockStrategy: Send + Sync {
    fn process_async<'a>(
        &'a self,
        code: &'a str,
        config: &'a ProcessingConfig,
    ) -> Pin<Box<dyn Future<Output = Result<ProcessedCodeBlock, ProcessingError>> + Send + 'a>>;
}

impl AsyncCodeBlockStrategy for MyLanguageStrategy {
    fn process_async<'a>(
        &'a self,
        code: &'a str,
        config: &'a ProcessingConfig,
    ) -> Pin<Box<dyn Future<Output = Result<ProcessedCodeBlock, ProcessingError>> + Send + 'a>> {
        Box::pin(async move {
            // Perform async processing here
            // This could involve network requests, file I/O, etc.
            
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
            
            // Delegate to synchronous implementation for now
            self.process(code, config)
        })
    }
}
```

## Testing Your Plugin

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_plugin_creation() {
        let plugin = MyLanguagePlugin::new();
        assert_eq!(plugin.name(), "mylanguage-processor");
        assert_eq!(plugin.version(), "1.0.0");
        assert!(!plugin.supported_languages().is_empty());
    }

    #[test]
    fn test_plugin_configuration() {
        let mut plugin = MyLanguagePlugin::new();
        
        let mut config = HashMap::new();
        config.insert("enable_strict_mode".to_string(), "true".to_string());
        config.insert("max_line_length".to_string(), "120".to_string());
        
        plugin.configure(&config).unwrap();
        
        assert!(plugin.config.enable_strict_mode);
        assert_eq!(plugin.config.max_line_length, 120);
    }

    #[test]
    fn test_strategy_creation() {
        let plugin = MyLanguagePlugin::new();
        let strategy = plugin.create_strategy().unwrap();
        
        assert_eq!(strategy.get_language_name(), "mylanguage");
        assert!(strategy.supports_language("mylanguage"));
        assert!(strategy.supports_language("mylang"));
        assert!(strategy.supports_language("ml"));
        assert!(!strategy.supports_language("python"));
    }

    #[test]
    fn test_syntax_validation() {
        let config = MyLanguageConfig {
            enable_strict_mode: true,
            max_line_length: 50,
            custom_rules: vec!["no_tabs".to_string()],
        };
        let strategy = MyLanguageStrategy::new(config);
        
        // Valid code
        let valid_code = "function test() {\n    return 42;\n}";
        assert!(strategy.validate_syntax(valid_code).unwrap());
        
        // Invalid code (contains tab)
        let invalid_code = "function test() {\n\treturn 42;\n}";
        assert!(strategy.validate_syntax(invalid_code).is_err());
    }

    #[test]
    fn test_code_formatting() {
        let strategy = MyLanguageStrategy::new(MyLanguageConfig::default());
        
        let unformatted = "function test(){return 42;}";
        let formatted = strategy.format_code(unformatted).unwrap();
        
        assert!(formatted.contains("    ")); // Should have indentation
        assert!(formatted.contains("\n"));   // Should have line breaks
    }

    #[test]
    fn test_processing_pipeline() {
        let strategy = MyLanguageStrategy::new(MyLanguageConfig::default());
        let config = ProcessingConfig::default()
            .with_syntax_validation(true)
            .with_formatting(true);
        
        let code = "function test(){return 42;}";
        let result = strategy.process(code, &config).unwrap();
        
        assert!(result.is_successful());
        assert!(result.metadata.is_validated);
        assert!(result.metadata.syntax_valid);
        assert!(result.processed_code.is_some());
    }

    #[test]
    fn test_error_handling() {
        let config = MyLanguageConfig {
            enable_strict_mode: true,
            max_line_length: 10, // Very short to trigger errors
            custom_rules: vec![],
        };
        let strategy = MyLanguageStrategy::new(config);
        let processing_config = ProcessingConfig::default()
            .with_syntax_validation(true);
        
        let long_code = "this is a very long line that exceeds the maximum length;";
        let result = strategy.process(long_code, &processing_config).unwrap();
        
        assert!(!result.is_successful());
        assert!(result.error_count() > 0);
        assert!(!result.metadata.syntax_valid);
    }
}
```

### Integration Tests

```rust
#[cfg(test)]
mod integration_tests {
    use super::*;
    use md2docx::markdown::code_block::{PluginManager, StrategyRegistry};
    use std::sync::{Arc, Mutex};

    #[test]
    fn test_plugin_loading() {
        let registry = Arc::new(Mutex::new(StrategyRegistry::new()));
        let mut manager = PluginManager::with_registry(registry.clone());
        
        let plugin = Box::new(MyLanguagePlugin::new());
        manager.load_plugin(plugin).unwrap();
        
        assert_eq!(manager.plugin_count(), 1);
        assert!(manager.is_plugin_loaded("mylanguage-processor"));
        
        // Test that the strategy is available
        let registry_guard = registry.lock().unwrap();
        let strategy = registry_guard.get_strategy("mylanguage");
        assert_eq!(strategy.get_language_name(), "mylanguage");
    }

    #[test]
    fn test_plugin_configuration_integration() {
        let registry = Arc::new(Mutex::new(StrategyRegistry::new()));
        let mut manager = PluginManager::with_registry(registry);
        
        let plugin = Box::new(MyLanguagePlugin::new());
        manager.load_plugin(plugin).unwrap();
        
        let mut config = HashMap::new();
        config.insert("enable_strict_mode".to_string(), "true".to_string());
        
        manager.configure_plugin("mylanguage-processor", &config).unwrap();
        
        // Verify configuration was applied
        let plugin_info = manager.get_plugin_info("mylanguage-processor").unwrap();
        assert_eq!(plugin_info.name, "mylanguage-processor");
    }
}
```

## Distribution and Packaging

### Creating a Crate

1. **Prepare your crate for publication**:

```toml
# Cargo.toml
[package]
name = "mylanguage-code-block-plugin"
version = "0.1.0"
edition = "2021"
description = "MyLanguage code block processing plugin for md2docx"
license = "MIT OR Apache-2.0"
repository = "https://github.com/yourusername/mylanguage-plugin"
keywords = ["markdown", "code-block", "mylanguage", "plugin"]
categories = ["text-processing", "development-tools"]

[lib]
name = "mylanguage_plugin"
crate-type = ["cdylib", "rlib"]

[dependencies]
md2docx = "0.1.0"  # Adjust version as needed
serde = { version = "1.0", features = ["derive"] }
```

2. **Add documentation**:

```rust
//! # MyLanguage Code Block Plugin
//! 
//! This plugin provides MyLanguage code block processing capabilities for the
//! md2docx markdown processor. It includes syntax validation, code formatting,
//! and quality checks specific to MyLanguage.
//! 
//! ## Features
//! 
//! - Syntax validation with configurable strictness
//! - Code formatting with customizable rules
//! - Quality checks and warnings
//! - Configurable through YAML files
//! 
//! ## Usage
//! 
//! ```rust
//! use mylanguage_plugin::MyLanguagePlugin;
//! use md2docx::markdown::code_block::PluginManager;
//! 
//! let mut manager = PluginManager::new();
//! let plugin = Box::new(MyLanguagePlugin::new());
//! manager.load_plugin(plugin).unwrap();
//! ```

/// Re-export main plugin types for convenience
pub use plugin::MyLanguagePlugin;
pub use strategy::MyLanguageStrategy;
pub use config::MyLanguageConfig;

mod plugin;
mod strategy;
mod config;
```

### Dynamic Loading Support

For dynamic plugin loading, you might want to provide a C-compatible interface:

```rust
use std::ffi::CStr;
use std::os::raw::c_char;

/// C-compatible plugin entry point
#[no_mangle]
pub extern "C" fn create_plugin() -> *mut dyn CodeBlockPlugin {
    Box::into_raw(Box::new(MyLanguagePlugin::new()))
}

/// C-compatible plugin cleanup
#[no_mangle]
pub extern "C" fn destroy_plugin(plugin: *mut dyn CodeBlockPlugin) {
    if !plugin.is_null() {
        unsafe {
            let _ = Box::from_raw(plugin);
        }
    }
}

/// Get plugin metadata as C string
#[no_mangle]
pub extern "C" fn get_plugin_name() -> *const c_char {
    "mylanguage-processor\0".as_ptr() as *const c_char
}
```

## Best Practices

### Error Handling

1. **Use specific error types**:
```rust
#[derive(Debug)]
pub enum MyLanguageError {
    SyntaxError { line: usize, column: usize, message: String },
    FormattingError(String),
    ConfigurationError(String),
}

impl From<MyLanguageError> for ProcessingError {
    fn from(err: MyLanguageError) -> Self {
        match err {
            MyLanguageError::SyntaxError { line, column, message } => {
                ProcessingError::syntax_error(&message, Some(line), Some(column))
            }
            MyLanguageError::FormattingError(msg) => {
                ProcessingError::formatting_error(&msg)
            }
            MyLanguageError::ConfigurationError(msg) => {
                ProcessingError::new("configuration_error", &msg)
            }
        }
    }
}
```

2. **Provide helpful error messages**:
```rust
fn validate_syntax(&self, code: &str) -> Result<bool, ProcessingError> {
    // Instead of: "Syntax error"
    // Use: "Missing closing brace '}' for function starting at line 5"
    
    if let Err(e) = self.parse_code(code) {
        return Err(ProcessingError::syntax_error(
            &format!("Parse error: {}. Check for missing braces or semicolons.", e),
            Some(e.line),
            Some(e.column)
        ));
    }
    
    Ok(true)
}
```

### Performance

1. **Cache expensive operations**:
```rust
use std::collections::HashMap;

pub struct MyLanguageStrategy {
    config: MyLanguageConfig,
    parse_cache: HashMap<String, bool>, // Cache syntax validation results
}

impl MyLanguageStrategy {
    fn validate_syntax_cached(&mut self, code: &str) -> Result<bool, ProcessingError> {
        if let Some(&cached_result) = self.parse_cache.get(code) {
            return Ok(cached_result);
        }
        
        let result = self.validate_syntax(code)?;
        self.parse_cache.insert(code.to_string(), result);
        Ok(result)
    }
}
```

2. **Use lazy initialization**:
```rust
use std::sync::Once;

static INIT: Once = Once::new();
static mut EXTERNAL_TOOL_AVAILABLE: bool = false;

impl MyLanguagePlugin {
    fn initialize(&self) -> Result<(), PluginError> {
        INIT.call_once(|| {
            unsafe {
                EXTERNAL_TOOL_AVAILABLE = self.check_external_tool();
            }
        });
        
        Ok(())
    }
}
```

### Configuration

1. **Provide sensible defaults**:
```rust
impl Default for MyLanguageConfig {
    fn default() -> Self {
        Self {
            enable_strict_mode: false,        // Conservative default
            max_line_length: 80,              // Common standard
            custom_rules: vec![               // Essential rules only
                "no_trailing_whitespace".to_string(),
            ],
        }
    }
}
```

2. **Validate configuration early**:
```rust
impl MyLanguagePlugin {
    fn configure(&mut self, config: &HashMap<String, String>) -> Result<(), PluginError> {
        // Validate all configuration before applying any changes
        self.validate_config(config)?;
        
        // Apply configuration only after validation succeeds
        self.apply_config(config)?;
        
        Ok(())
    }
}
```

### Documentation

1. **Document configuration options**:
```rust
/// Configuration for MyLanguage processing
#[derive(Debug, Clone)]
pub struct MyLanguageConfig {
    /// Enable strict syntax validation (default: false)
    /// 
    /// When enabled, the plugin will enforce stricter syntax rules
    /// and report more detailed error messages.
    pub enable_strict_mode: bool,
    
    /// Maximum line length for formatting (default: 80)
    /// 
    /// Lines longer than this will be wrapped during formatting.
    /// Must be between 10 and 1000.
    pub max_line_length: usize,
    
    /// Custom validation rules (default: empty)
    /// 
    /// Available rules:
    /// - "no_tabs": Disallow tab characters
    /// - "no_trailing_whitespace": Disallow trailing whitespace
    /// - "require_semicolons": Require semicolons at end of statements
    pub custom_rules: Vec<String>,
}
```

2. **Provide usage examples**:
```rust
/// # Examples
/// 
/// Basic usage:
/// ```rust
/// use mylanguage_plugin::MyLanguagePlugin;
/// 
/// let plugin = MyLanguagePlugin::new();
/// let strategy = plugin.create_strategy().unwrap();
/// ```
/// 
/// With custom configuration:
/// ```rust
/// use mylanguage_plugin::{MyLanguagePlugin, MyLanguageConfig};
/// 
/// let config = MyLanguageConfig {
///     enable_strict_mode: true,
///     max_line_length: 120,
///     custom_rules: vec!["no_tabs".to_string()],
/// };
/// let plugin = MyLanguagePlugin::with_config(config);
/// ```
impl MyLanguagePlugin {
    // Implementation...
}
```

## Examples

### Complete Plugin Example

See the `examples/` directory for complete plugin implementations:

- `examples/python_plugin.rs` - Python code processing plugin
- `examples/javascript_plugin.rs` - JavaScript/TypeScript plugin
- `examples/simple_plugin.rs` - Minimal plugin example

### Configuration Examples

Example YAML configuration for your plugin:

```yaml
# In code_block_config.yaml
languages:
  mylanguage:
    enable_syntax_validation: true
    enable_formatting: true
    custom_options:
      enable_strict_mode: "true"
      max_line_length: "100"
      custom_rules: "no_tabs,no_trailing_whitespace"

plugins:
  plugin_configs:
    "mylanguage-processor":
      enable_strict_mode: true
      max_line_length: 100
      custom_rules: "no_tabs,no_trailing_whitespace"
```

This guide should provide you with everything you need to create robust, well-tested plugins for the Code Block Strategy System. Remember to follow Rust best practices and provide comprehensive documentation for your users.