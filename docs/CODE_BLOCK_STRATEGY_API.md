# Code Block Strategy System API Documentation

## Overview

The Code Block Strategy System provides a flexible, extensible architecture for processing different types of code blocks in Markdown documents. It uses the Strategy pattern to allow different processing logic for different programming languages.

## Core Components

### CodeBlockStrategy Trait

The `CodeBlockStrategy` trait defines the interface that all code block processing strategies must implement.

```rust
pub trait CodeBlockStrategy: Send + Sync {
    /// Process a code block according to the strategy's rules
    fn process(&self, code: &str, config: &ProcessingConfig) -> Result<ProcessedCodeBlock, ProcessingError>;
    
    /// Check if this strategy supports the given language
    fn supports_language(&self, language: &str) -> bool;
    
    /// Get the primary language name this strategy handles
    fn get_language_name(&self) -> &'static str;
    
    /// Get the priority of this strategy (higher numbers = higher priority)
    fn get_priority(&self) -> u8 { 100 }
    
    /// Get the version of this strategy implementation
    fn get_version(&self) -> &'static str { "1.0.0" }
    
    /// Get a description of what this strategy does
    fn get_description(&self) -> &'static str { "Generic code block processing strategy" }
}
```

#### Key Methods

- **`process()`**: The main method that processes a code block and returns a `ProcessedCodeBlock` with results
- **`supports_language()`**: Determines if this strategy can handle a specific language
- **`get_language_name()`**: Returns the primary language identifier for this strategy
- **`get_priority()`**: Used for conflict resolution when multiple strategies support the same language

### ProcessedCodeBlock

Represents the result of processing a code block, containing the original code, processed code (if any), metadata, and any errors or warnings.

```rust
pub struct ProcessedCodeBlock {
    pub original_code: String,
    pub processed_code: Option<String>,
    pub language: Option<String>,
    pub metadata: ProcessingMetadata,
    pub errors: Vec<ProcessingError>,
    pub warnings: Vec<ProcessingWarning>,
}
```

#### Key Methods

```rust
impl ProcessedCodeBlock {
    /// Create a new processed code block
    pub fn new(original_code: String, language: Option<String>) -> Self;
    
    /// Get the final code (processed if available, otherwise original)
    pub fn get_final_code(&self) -> &str;
    
    /// Check if processing was successful (no errors)
    pub fn is_successful(&self) -> bool;
    
    /// Check if there are any warnings
    pub fn has_warnings(&self) -> bool;
    
    /// Check if the code was actually modified during processing
    pub fn was_modified(&self) -> bool;
    
    /// Get a summary of the processing results
    pub fn get_summary(&self) -> ProcessingSummary;
    
    // Builder methods
    pub fn with_processed_code(self, processed_code: String) -> Self;
    pub fn with_validation(self, syntax_valid: bool) -> Self;
    pub fn with_error(self, error: ProcessingError) -> Self;
    pub fn with_warning(self, warning: ProcessingWarning) -> Self;
    pub fn with_metadata(self, metadata: ProcessingMetadata) -> Self;
}
```

### ProcessingConfig

Configuration options that control how code blocks are processed.

```rust
pub struct ProcessingConfig {
    pub enable_syntax_validation: bool,
    pub enable_formatting: bool,
    pub enable_optimization: bool,
    pub timeout_ms: u64,
    pub custom_options: HashMap<String, String>,
}
```

#### Default Configuration

```rust
impl Default for ProcessingConfig {
    fn default() -> Self {
        Self {
            enable_syntax_validation: true,
            enable_formatting: false,
            enable_optimization: false,
            timeout_ms: 5000,
            custom_options: HashMap::new(),
        }
    }
}
```

### ProcessingMetadata

Contains metadata about the processing operation.

```rust
pub struct ProcessingMetadata {
    pub is_formatted: bool,
    pub is_validated: bool,
    pub syntax_valid: bool,
    pub processing_time: Duration,
    pub processor_version: String,
    pub custom_attributes: HashMap<String, String>,
}
```

## Built-in Strategies

### DefaultStrategy

A fallback strategy that performs no processing but provides basic metadata.

```rust
let strategy = DefaultStrategy::new();
let config = ProcessingConfig::default();
let result = strategy.process("some code", &config)?;
```

### RustStrategy

Processes Rust code with syntax validation and formatting capabilities.

```rust
let strategy = RustStrategy::new();
let config = ProcessingConfig::default()
    .with_syntax_validation(true)
    .with_formatting(true);
let result = strategy.process("fn main() {}", &config)?;
```

#### Features

- **Syntax Validation**: Uses the `syn` crate to parse and validate Rust syntax
- **Code Formatting**: Basic formatting using syn's pretty-printing capabilities
- **Code Quality Checks**: Warns about common issues like `unwrap()` usage and `panic!` calls
- **Error Reporting**: Detailed syntax error messages with context

## Strategy Registry

The `StrategyRegistry` manages all available code block processing strategies.

```rust
pub struct StrategyRegistry {
    // Internal implementation
}

impl StrategyRegistry {
    /// Create a new registry with default strategies
    pub fn new() -> Self;
    
    /// Register a new strategy
    pub fn register_strategy(&mut self, strategy: Box<dyn CodeBlockStrategy>);
    
    /// Get a strategy for a specific language
    pub fn get_strategy(&self, language: &str) -> &dyn CodeBlockStrategy;
    
    /// Get the default fallback strategy
    pub fn get_default_strategy(&self) -> &dyn CodeBlockStrategy;
    
    /// List all registered strategies
    pub fn list_strategies(&self) -> Vec<&dyn CodeBlockStrategy>;
}
```

## Error Handling

### ProcessingError

Represents errors that occur during code block processing.

```rust
pub struct ProcessingError {
    pub error_type: String,
    pub message: String,
    pub line: Option<usize>,
    pub column: Option<usize>,
    pub severity: ErrorSeverity,
}

impl ProcessingError {
    pub fn syntax_error(message: &str, line: Option<usize>, column: Option<usize>) -> Self;
    pub fn formatting_error(message: &str) -> Self;
    pub fn timeout_error() -> Self;
    pub fn validation_error(message: &str) -> Self;
}
```

### ProcessingWarning

Represents warnings that don't prevent processing but indicate potential issues.

```rust
pub struct ProcessingWarning {
    pub warning_type: String,
    pub message: String,
    pub line: Option<usize>,
    pub column: Option<usize>,
}

impl ProcessingWarning {
    pub fn new(warning_type: &str, message: &str) -> Self;
    pub fn formatting_warning(message: &str) -> Self;
    pub fn style_warning(message: &str) -> Self;
}
```

## Plugin System

### CodeBlockPlugin Trait

Interface for creating plugins that provide new code block processing strategies.

```rust
pub trait CodeBlockPlugin: Send + Sync {
    /// Get the unique name of this plugin
    fn name(&self) -> &str;
    
    /// Get the version of this plugin
    fn version(&self) -> &str;
    
    /// Get a description of what this plugin provides
    fn description(&self) -> &str;
    
    /// Create a new strategy instance
    fn create_strategy(&self) -> Result<Box<dyn CodeBlockStrategy>, PluginError>;
    
    /// Get the languages this plugin supports
    fn supported_languages(&self) -> Vec<String>;
    
    /// Initialize the plugin (optional)
    fn initialize(&self) -> Result<(), PluginError> { Ok(()) }
    
    /// Cleanup the plugin (optional)
    fn cleanup(&self) -> Result<(), PluginError> { Ok(()) }
    
    /// Configure the plugin with options (optional)
    fn configure(&mut self, config: &HashMap<String, String>) -> Result<(), PluginError> { Ok(()) }
}
```

### PluginManager

Manages loading, configuration, and lifecycle of plugins.

```rust
pub struct PluginManager {
    // Internal implementation
}

impl PluginManager {
    /// Create a new plugin manager
    pub fn new() -> Self;
    
    /// Create a plugin manager with a strategy registry
    pub fn with_registry(registry: Arc<Mutex<StrategyRegistry>>) -> Self;
    
    /// Load a plugin
    pub fn load_plugin(&mut self, plugin: Box<dyn CodeBlockPlugin>) -> Result<(), PluginError>;
    
    /// Unload a plugin by name
    pub fn unload_plugin(&mut self, plugin_name: &str) -> Result<(), PluginError>;
    
    /// Configure a plugin
    pub fn configure_plugin(&mut self, plugin_name: &str, config: &HashMap<String, String>) -> Result<(), PluginError>;
    
    /// List all loaded plugins
    pub fn list_plugins(&self) -> Vec<PluginInfo>;
    
    /// Get plugin information
    pub fn get_plugin_info(&self, plugin_name: &str) -> Option<PluginInfo>;
}
```

## Usage Examples

### Basic Usage

```rust
use crate::markdown::code_block::{StrategyRegistry, ProcessingConfig};

// Create a registry with default strategies
let registry = StrategyRegistry::new();

// Get a strategy for Rust code
let strategy = registry.get_strategy("rust");

// Configure processing options
let config = ProcessingConfig::default()
    .with_syntax_validation(true)
    .with_formatting(true);

// Process some Rust code
let code = r#"
fn main() {
    println!("Hello, world!");
}
"#;

let result = strategy.process(code, &config)?;

if result.is_successful() {
    println!("Processing successful!");
    if let Some(formatted) = &result.processed_code {
        println!("Formatted code:\n{}", formatted);
    }
} else {
    println!("Processing failed with {} errors", result.error_count());
    for error in &result.errors {
        println!("Error: {}", error.message);
    }
}
```

### Custom Strategy Implementation

```rust
use crate::markdown::code_block::{CodeBlockStrategy, ProcessedCodeBlock, ProcessingConfig, ProcessingError};

#[derive(Debug, Clone)]
pub struct PythonStrategy;

impl CodeBlockStrategy for PythonStrategy {
    fn process(&self, code: &str, config: &ProcessingConfig) -> Result<ProcessedCodeBlock, ProcessingError> {
        let start_time = std::time::Instant::now();
        
        // Implement Python-specific processing logic here
        let syntax_valid = self.validate_python_syntax(code)?;
        
        let metadata = ProcessingMetadata::new(self.get_version())
            .with_processing_time(start_time.elapsed());
        
        let processed = ProcessedCodeBlock::new(code.to_string(), Some("python".to_string()))
            .with_metadata(metadata)
            .with_validation(syntax_valid);
        
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
}

impl PythonStrategy {
    fn validate_python_syntax(&self, code: &str) -> Result<bool, ProcessingError> {
        // Implement Python syntax validation
        // This is a simplified example
        Ok(!code.trim().is_empty())
    }
}
```

### Plugin Development

```rust
use crate::markdown::code_block::{CodeBlockPlugin, PluginError};

#[derive(Debug)]
pub struct MyCustomPlugin;

impl CodeBlockPlugin for MyCustomPlugin {
    fn name(&self) -> &str {
        "my-custom-plugin"
    }
    
    fn version(&self) -> &str {
        "1.0.0"
    }
    
    fn description(&self) -> &str {
        "Custom code block processor for my language"
    }
    
    fn create_strategy(&self) -> Result<Box<dyn CodeBlockStrategy>, PluginError> {
        Ok(Box::new(MyCustomStrategy::new()))
    }
    
    fn supported_languages(&self) -> Vec<String> {
        vec!["mylang".to_string()]
    }
}
```

## Configuration

The system supports both global and language-specific configuration through YAML files. See the [Configuration Guide](CODE_BLOCK_STRATEGY_CONFIG.md) for detailed information.

## Performance Considerations

- **Caching**: Processed results can be cached to avoid reprocessing identical code blocks
- **Timeouts**: Processing operations have configurable timeouts to prevent hanging
- **Parallel Processing**: Multiple code blocks can be processed in parallel (experimental)
- **Memory Management**: The system includes memory usage optimization features

## Thread Safety

All core components are designed to be thread-safe:

- `CodeBlockStrategy` implementations must be `Send + Sync`
- `StrategyRegistry` can be safely shared between threads using `Arc<Mutex<>>`
- `PluginManager` supports concurrent plugin operations

## Error Recovery

The system includes robust error recovery mechanisms:

- **Fallback Strategy**: If a specific strategy fails, the system falls back to the default strategy
- **Graceful Degradation**: Syntax validation failures don't prevent basic processing
- **Error Isolation**: Errors in one code block don't affect processing of other blocks

## Extensibility

The system is designed for easy extension:

- **Plugin Architecture**: Add new language support through plugins
- **Strategy Interface**: Implement custom processing logic by implementing `CodeBlockStrategy`
- **Configuration System**: Flexible configuration options for all strategies
- **Metadata System**: Custom attributes can be added to processing metadata