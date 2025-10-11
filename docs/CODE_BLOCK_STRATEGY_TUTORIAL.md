# Code Block Strategy System Tutorial

## Introduction

This tutorial will guide you through using the Code Block Strategy System to process different types of code blocks in your Markdown documents. You'll learn how to configure the system, use built-in strategies, and create custom strategies for your specific needs.

## Table of Contents

1. [Getting Started](#getting-started)
2. [Basic Usage](#basic-usage)
3. [Configuration](#configuration)
4. [Built-in Strategies](#built-in-strategies)
5. [Creating Custom Strategies](#creating-custom-strategies)
6. [Plugin Development](#plugin-development)
7. [Advanced Features](#advanced-features)
8. [Troubleshooting](#troubleshooting)

## Getting Started

### Prerequisites

Before using the Code Block Strategy System, ensure you have:

- Rust 1.70 or later
- The `syn` crate for Rust syntax processing
- The `serde_json` crate for JSON processing (if using JSON strategies)

### Basic Setup

First, create a strategy registry and configure it with the strategies you need:

```rust
use crate::markdown::code_block::{StrategyRegistry, ProcessingConfig};

// Create a new registry with built-in strategies
let mut registry = StrategyRegistry::new();

// The registry comes pre-loaded with:
// - DefaultStrategy (fallback for all languages)
// - RustStrategy (for Rust code processing)
```

## Basic Usage

### Processing a Single Code Block

Here's how to process a simple Rust code block:

```rust
use crate::markdown::code_block::{StrategyRegistry, ProcessingConfig};

let registry = StrategyRegistry::new();
let strategy = registry.get_strategy("rust");

let config = ProcessingConfig::default()
    .with_syntax_validation(true)
    .with_formatting(true);

let rust_code = r#"
fn main(){
println!("Hello, world!");
}
"#;

match strategy.process(rust_code, &config) {
    Ok(result) => {
        if result.is_successful() {
            println!("✅ Processing successful!");
            
            // Get the final code (formatted if formatting was applied)
            println!("Final code:\n{}", result.get_final_code());
            
            // Check if the code was modified
            if result.was_modified() {
                println!("📝 Code was formatted");
            }
            
            // Display processing metadata
            let summary = result.get_summary();
            println!("⏱️  Processing time: {:?}", summary.processing_time);
            println!("✔️  Syntax valid: {}", summary.is_valid);
        } else {
            println!("❌ Processing failed with {} errors:", result.error_count());
            for error in &result.errors {
                println!("  - {}: {}", error.error_type, error.message);
            }
        }
        
        // Display any warnings
        if result.has_warnings() {
            println!("⚠️  {} warnings:", result.warning_count());
            for warning in &result.warnings {
                println!("  - {}: {}", warning.warning_type, warning.message);
            }
        }
    }
    Err(e) => {
        println!("💥 Processing error: {}", e);
    }
}
```

### Processing Multiple Code Blocks

When processing multiple code blocks, you can reuse the same strategy and configuration:

```rust
let registry = StrategyRegistry::new();
let config = ProcessingConfig::default()
    .with_syntax_validation(true)
    .with_formatting(false); // Disable formatting for faster processing

let code_blocks = vec![
    ("rust", "fn main() { println!(\"Hello\"); }"),
    ("javascript", "console.log('Hello, world!');"),
    ("json", r#"{"name": "example", "version": "1.0.0"}"#),
    ("unknown", "some unknown code"),
];

for (language, code) in code_blocks {
    let strategy = registry.get_strategy(language);
    
    match strategy.process(code, &config) {
        Ok(result) => {
            println!("📄 {} code: {}", language, 
                if result.is_successful() { "✅ OK" } else { "❌ Failed" });
        }
        Err(e) => {
            println!("📄 {} code: 💥 Error - {}", language, e);
        }
    }
}
```

## Configuration

### Global Configuration

Create a configuration that applies to all code block processing:

```rust
use std::collections::HashMap;

let mut custom_options = HashMap::new();
custom_options.insert("max_line_length".to_string(), "100".to_string());
custom_options.insert("strict_mode".to_string(), "true".to_string());

let config = ProcessingConfig {
    enable_syntax_validation: true,
    enable_formatting: true,
    enable_optimization: false,
    timeout_ms: 10000, // 10 seconds
    custom_options,
};
```

### Language-Specific Configuration

You can create different configurations for different languages:

```rust
// Configuration for production code (strict)
let production_config = ProcessingConfig::default()
    .with_syntax_validation(true)
    .with_formatting(true)
    .with_timeout(5000);

// Configuration for examples (lenient)
let example_config = ProcessingConfig::default()
    .with_syntax_validation(false)
    .with_formatting(true)
    .with_timeout(1000);

// Use different configs based on context
let config = if is_production_code {
    &production_config
} else {
    &example_config
};
```

### Loading Configuration from YAML

You can load configuration from a YAML file (see `examples/code_block_config.yaml`):

```rust
// This is a conceptual example - actual implementation may vary
use serde_yaml;
use std::fs;

#[derive(serde::Deserialize)]
struct CodeBlockConfig {
    global: GlobalConfig,
    languages: std::collections::HashMap<String, LanguageConfig>,
}

let config_content = fs::read_to_string("code_block_config.yaml")?;
let config: CodeBlockConfig = serde_yaml::from_str(&config_content)?;

// Convert to ProcessingConfig for specific language
let rust_config = ProcessingConfig {
    enable_syntax_validation: config.languages.get("rust")
        .map(|c| c.enable_syntax_validation)
        .unwrap_or(config.global.enable_processing),
    enable_formatting: config.languages.get("rust")
        .map(|c| c.enable_formatting)
        .unwrap_or(false),
    // ... other fields
};
```

## Built-in Strategies

### DefaultStrategy

The default strategy provides basic processing for any language:

```rust
let strategy = DefaultStrategy::new();
let config = ProcessingConfig::default();

let result = strategy.process("any code here", &config)?;
// Result will have:
// - original_code: the input code
// - processed_code: None (no processing applied)
// - syntax_valid: true (assumed valid)
// - is_formatted: false
```

**Use cases:**
- Fallback for unsupported languages
- Basic code block handling without processing
- Testing and development

### RustStrategy

The Rust strategy provides comprehensive Rust code processing:

```rust
let strategy = RustStrategy::new();
let config = ProcessingConfig::default()
    .with_syntax_validation(true)
    .with_formatting(true);

let rust_code = r#"
use std::collections::HashMap;

fn main(){
let mut map=HashMap::new();
map.insert("key".to_string(),"value".to_string());
println!("{:?}",map);
}
"#;

let result = strategy.process(rust_code, &config)?;

if result.is_successful() {
    // The code will be validated and potentially formatted
    println!("Processed Rust code:\n{}", result.get_final_code());
}
```

**Features:**
- **Syntax Validation**: Uses `syn` crate to parse and validate Rust syntax
- **Basic Formatting**: Applies consistent formatting rules
- **Code Quality Warnings**: Detects common issues like `unwrap()` usage
- **Error Reporting**: Provides detailed syntax error messages

**Supported Language Identifiers:**
- `rust`
- `rs`

## Creating Custom Strategies

### Simple Custom Strategy

Here's how to create a simple strategy for Python code:

```rust
use crate::markdown::code_block::{
    CodeBlockStrategy, ProcessedCodeBlock, ProcessingConfig, ProcessingError, ProcessingMetadata
};
use std::time::Instant;

#[derive(Debug, Clone)]
pub struct PythonStrategy;

impl PythonStrategy {
    pub fn new() -> Self {
        Self
    }
    
    fn validate_python_syntax(&self, code: &str) -> Result<bool, ProcessingError> {
        // Simple validation - check for basic Python syntax
        let lines: Vec<&str> = code.lines().collect();
        
        for (line_num, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            
            // Skip empty lines and comments
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }
            
            // Check for basic syntax issues
            if trimmed.ends_with(':') {
                // This should be followed by an indented block
                if line_num + 1 < lines.len() {
                    let next_line = lines[line_num + 1].trim();
                    if !next_line.is_empty() && !next_line.starts_with(' ') && !next_line.starts_with('\t') {
                        return Err(ProcessingError::syntax_error(
                            "Expected indented block after colon",
                            Some(line_num + 2),
                            None
                        ));
                    }
                }
            }
        }
        
        Ok(true)
    }
    
    fn format_python_code(&self, code: &str) -> Result<String, ProcessingError> {
        // Simple formatting - normalize indentation
        let lines: Vec<&str> = code.lines().collect();
        let mut formatted_lines = Vec::new();
        let mut indent_level = 0;
        
        for line in lines {
            let trimmed = line.trim();
            
            if trimmed.is_empty() {
                formatted_lines.push(String::new());
                continue;
            }
            
            // Decrease indent for certain keywords
            if trimmed.starts_with("except") || trimmed.starts_with("elif") || 
               trimmed.starts_with("else") || trimmed.starts_with("finally") {
                indent_level = indent_level.saturating_sub(1);
            }
            
            // Add proper indentation
            let indented = format!("{}{}", "    ".repeat(indent_level), trimmed);
            formatted_lines.push(indented);
            
            // Increase indent after colon
            if trimmed.ends_with(':') {
                indent_level += 1;
            }
        }
        
        Ok(formatted_lines.join("\n"))
    }
}

impl CodeBlockStrategy for PythonStrategy {
    fn process(&self, code: &str, config: &ProcessingConfig) -> Result<ProcessedCodeBlock, ProcessingError> {
        let start_time = Instant::now();
        let mut errors = Vec::new();
        let mut warnings = Vec::new();
        
        // Validate syntax if enabled
        let syntax_valid = if config.enable_syntax_validation {
            match self.validate_python_syntax(code) {
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
            match self.format_python_code(code) {
                Ok(formatted) => Some(formatted),
                Err(e) => {
                    warnings.push(crate::markdown::code_block::ProcessingWarning::formatting_warning(&e.to_string()));
                    None
                }
            }
        } else {
            None
        };
        
        // Create metadata
        let mut metadata = ProcessingMetadata::new(self.get_version())
            .with_processing_time(start_time.elapsed());
        
        metadata.is_formatted = formatted_code.is_some();
        metadata.is_validated = config.enable_syntax_validation;
        metadata.syntax_valid = syntax_valid;
        
        // Build the result
        let mut processed = ProcessedCodeBlock::new(code.to_string(), Some("python".to_string()))
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
        matches!(language.to_lowercase().as_str(), "python" | "py")
    }
    
    fn get_language_name(&self) -> &'static str {
        "python"
    }
    
    fn get_priority(&self) -> u8 {
        120 // Higher than default
    }
    
    fn get_version(&self) -> &'static str {
        "1.0.0"
    }
    
    fn get_description(&self) -> &'static str {
        "Python code processing with basic syntax validation and formatting"
    }
}
```

### Registering Your Custom Strategy

```rust
let mut registry = StrategyRegistry::new();

// Register your custom strategy
registry.register_strategy(Box::new(PythonStrategy::new()));

// Now you can use it
let strategy = registry.get_strategy("python");
let config = ProcessingConfig::default().with_syntax_validation(true);
let result = strategy.process("print('Hello, Python!')", &config)?;
```

## Plugin Development

### Creating a Plugin

Plugins provide a way to package and distribute custom strategies:

```rust
use crate::markdown::code_block::{CodeBlockPlugin, PluginError};
use std::collections::HashMap;

#[derive(Debug)]
pub struct PythonPlugin {
    config: PythonPluginConfig,
}

#[derive(Debug, Clone)]
struct PythonPluginConfig {
    enable_pep8_checks: bool,
    max_line_length: usize,
}

impl Default for PythonPluginConfig {
    fn default() -> Self {
        Self {
            enable_pep8_checks: true,
            max_line_length: 79,
        }
    }
}

impl PythonPlugin {
    pub fn new() -> Self {
        Self {
            config: PythonPluginConfig::default(),
        }
    }
}

impl CodeBlockPlugin for PythonPlugin {
    fn name(&self) -> &str {
        "python-processor"
    }
    
    fn version(&self) -> &str {
        "1.0.0"
    }
    
    fn description(&self) -> &str {
        "Python code processor with PEP 8 compliance checking"
    }
    
    fn author(&self) -> &str {
        "Your Name <your.email@example.com>"
    }
    
    fn create_strategy(&self) -> Result<Box<dyn CodeBlockStrategy>, PluginError> {
        Ok(Box::new(PythonStrategy::new()))
    }
    
    fn supported_languages(&self) -> Vec<String> {
        vec!["python".to_string(), "py".to_string()]
    }
    
    fn initialize(&self) -> Result<(), PluginError> {
        // Perform any initialization here
        println!("Initializing Python plugin...");
        Ok(())
    }
    
    fn configure(&mut self, config: &HashMap<String, String>) -> Result<(), PluginError> {
        if let Some(pep8_checks) = config.get("enable_pep8_checks") {
            self.config.enable_pep8_checks = pep8_checks.parse().map_err(|_| {
                PluginError::configuration_error("Invalid value for enable_pep8_checks")
            })?;
        }
        
        if let Some(max_line_length) = config.get("max_line_length") {
            self.config.max_line_length = max_line_length.parse().map_err(|_| {
                PluginError::configuration_error("Invalid value for max_line_length")
            })?;
        }
        
        Ok(())
    }
}
```

### Using the Plugin Manager

```rust
use crate::markdown::code_block::PluginManager;
use std::sync::{Arc, Mutex};

// Create a plugin manager with a registry
let registry = Arc::new(Mutex::new(StrategyRegistry::new()));
let mut plugin_manager = PluginManager::with_registry(registry.clone());

// Load your plugin
let python_plugin = Box::new(PythonPlugin::new());
plugin_manager.load_plugin(python_plugin)?;

// Configure the plugin
let mut config = HashMap::new();
config.insert("enable_pep8_checks".to_string(), "true".to_string());
config.insert("max_line_length".to_string(), "88".to_string());
plugin_manager.configure_plugin("python-processor", &config)?;

// List loaded plugins
println!("Loaded plugins:");
for plugin_info in plugin_manager.list_plugins() {
    println!("  - {}", plugin_info.format());
}

// Use the strategy from the plugin
let registry_guard = registry.lock().unwrap();
let strategy = registry_guard.get_strategy("python");
// ... use the strategy
```

## Advanced Features

### Caching Results

For better performance, you can implement caching:

```rust
use std::collections::HashMap;
use std::hash::{Hash, Hasher};

struct CodeBlockCache {
    cache: HashMap<String, ProcessedCodeBlock>,
    max_size: usize,
}

impl CodeBlockCache {
    fn new(max_size: usize) -> Self {
        Self {
            cache: HashMap::new(),
            max_size,
        }
    }
    
    fn get_or_process<F>(&mut self, code: &str, language: &str, processor: F) -> Result<ProcessedCodeBlock, ProcessingError>
    where
        F: FnOnce() -> Result<ProcessedCodeBlock, ProcessingError>,
    {
        let cache_key = format!("{}:{}", language, code);
        
        if let Some(cached) = self.cache.get(&cache_key) {
            return Ok(cached.clone());
        }
        
        let result = processor()?;
        
        // Add to cache if not full
        if self.cache.len() < self.max_size {
            self.cache.insert(cache_key, result.clone());
        }
        
        Ok(result)
    }
}

// Usage
let mut cache = CodeBlockCache::new(100);
let registry = StrategyRegistry::new();
let config = ProcessingConfig::default();

let result = cache.get_or_process("fn main() {}", "rust", || {
    let strategy = registry.get_strategy("rust");
    strategy.process("fn main() {}", &config)
})?;
```

### Timeout Handling

Handle long-running processing operations:

```rust
use std::time::{Duration, Instant};

fn process_with_timeout(
    strategy: &dyn CodeBlockStrategy,
    code: &str,
    config: &ProcessingConfig,
) -> Result<ProcessedCodeBlock, ProcessingError> {
    let start = Instant::now();
    let timeout = Duration::from_millis(config.timeout_ms);
    
    // In a real implementation, you'd use async/await or threads
    // This is a simplified synchronous example
    
    let result = strategy.process(code, config)?;
    
    if start.elapsed() > timeout {
        return Err(ProcessingError::timeout_error());
    }
    
    Ok(result)
}
```

### Parallel Processing

Process multiple code blocks in parallel:

```rust
use std::thread;
use std::sync::Arc;

fn process_code_blocks_parallel(
    registry: Arc<StrategyRegistry>,
    code_blocks: Vec<(String, String)>, // (language, code) pairs
    config: Arc<ProcessingConfig>,
) -> Vec<Result<ProcessedCodeBlock, ProcessingError>> {
    let handles: Vec<_> = code_blocks
        .into_iter()
        .map(|(language, code)| {
            let registry = Arc::clone(&registry);
            let config = Arc::clone(&config);
            
            thread::spawn(move || {
                let strategy = registry.get_strategy(&language);
                strategy.process(&code, &config)
            })
        })
        .collect();
    
    handles
        .into_iter()
        .map(|handle| handle.join().unwrap())
        .collect()
}
```

## Troubleshooting

### Common Issues

#### 1. Strategy Not Found

**Problem**: Getting the default strategy instead of the expected one.

```rust
let strategy = registry.get_strategy("rust");
// Returns DefaultStrategy instead of RustStrategy
```

**Solution**: Check that the strategy is properly registered and the language name matches:

```rust
// Check if strategy is registered
let strategies = registry.list_strategies();
for strategy in strategies {
    println!("Registered: {} (supports: {})", 
        strategy.get_language_name(), 
        strategy.get_description());
}

// Ensure language matching is case-insensitive
let strategy = registry.get_strategy("RUST"); // Should work
let strategy = registry.get_strategy("rust"); // Should work
```

#### 2. Processing Errors

**Problem**: Code processing fails with syntax errors.

```rust
let result = strategy.process(code, &config)?;
if !result.is_successful() {
    // Handle errors
}
```

**Solution**: Check the error details and adjust your code or configuration:

```rust
if !result.is_successful() {
    println!("Processing failed:");
    for error in &result.errors {
        println!("  {}:{} - {} ({})", 
            error.line.unwrap_or(0),
            error.column.unwrap_or(0),
            error.message,
            error.error_type);
    }
    
    // Try with more lenient configuration
    let lenient_config = ProcessingConfig::default()
        .with_syntax_validation(false);
    let retry_result = strategy.process(code, &lenient_config)?;
}
```

#### 3. Plugin Loading Issues

**Problem**: Plugin fails to load or initialize.

```rust
match plugin_manager.load_plugin(plugin) {
    Ok(_) => println!("Plugin loaded successfully"),
    Err(e) => println!("Plugin loading failed: {}", e),
}
```

**Solution**: Check plugin implementation and dependencies:

```rust
// Ensure plugin implements all required methods
impl CodeBlockPlugin for MyPlugin {
    fn name(&self) -> &str { "my-plugin" }
    fn version(&self) -> &str { "1.0.0" }
    fn description(&self) -> &str { "My custom plugin" }
    
    fn create_strategy(&self) -> Result<Box<dyn CodeBlockStrategy>, PluginError> {
        // Ensure this doesn't panic or return errors
        Ok(Box::new(MyStrategy::new()))
    }
    
    fn supported_languages(&self) -> Vec<String> {
        vec!["mylang".to_string()]
    }
    
    fn initialize(&self) -> Result<(), PluginError> {
        // Check dependencies here
        Ok(())
    }
}
```

### Performance Issues

#### 1. Slow Processing

**Symptoms**: Code block processing takes too long.

**Solutions**:
- Enable caching for repeated code blocks
- Reduce timeout values for faster failure
- Disable expensive features like formatting for large documents
- Use parallel processing for multiple code blocks

```rust
// Optimized configuration for performance
let fast_config = ProcessingConfig {
    enable_syntax_validation: false, // Skip validation for speed
    enable_formatting: false,        // Skip formatting for speed
    enable_optimization: true,       // Enable optimizations
    timeout_ms: 1000,               // Short timeout
    custom_options: HashMap::new(),
};
```

#### 2. Memory Usage

**Symptoms**: High memory usage when processing many code blocks.

**Solutions**:
- Implement result caching with size limits
- Clear processed results when no longer needed
- Use streaming processing for large documents

```rust
// Memory-efficient processing
let mut cache = CodeBlockCache::new(50); // Limit cache size
// Process in batches and clear results periodically
```

### Debugging Tips

1. **Enable Detailed Logging**: Add debug prints to understand processing flow
2. **Check Strategy Priority**: Higher priority strategies override lower ones
3. **Validate Configuration**: Ensure configuration values are correct
4. **Test with Simple Cases**: Start with minimal code examples
5. **Check Dependencies**: Ensure required crates are available

```rust
// Debug helper function
fn debug_processing_result(result: &ProcessedCodeBlock) {
    println!("=== Processing Result Debug ===");
    println!("Language: {:?}", result.language);
    println!("Original length: {}", result.original_code.len());
    println!("Processed: {}", result.processed_code.is_some());
    println!("Modified: {}", result.was_modified());
    println!("Syntax valid: {}", result.metadata.syntax_valid);
    println!("Errors: {}", result.error_count());
    println!("Warnings: {}", result.warning_count());
    println!("Processing time: {:?}", result.metadata.processing_time);
    println!("==============================");
}
```

This tutorial should give you a comprehensive understanding of how to use and extend the Code Block Strategy System. For more detailed API information, see the [API Documentation](CODE_BLOCK_STRATEGY_API.md).