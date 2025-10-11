# Code Block Strategy Configuration Guide

## Overview

The Code Block Strategy System supports comprehensive configuration through YAML files, allowing you to customize processing behavior for different programming languages and use cases.

## Configuration File Structure

### Basic Structure

```yaml
# Global settings that apply to all code block processing
global:
  enable_processing: true
  default_timeout_ms: 5000
  max_cache_size: 1000

# Language-specific configurations
languages:
  rust:
    enable_syntax_validation: true
    enable_formatting: true
  
  javascript:
    enable_syntax_validation: true
    enable_formatting: false

# Plugin system configuration
plugins:
  enable_plugins: true
  plugin_paths:
    - "./plugins"
```

### Configuration Locations

The system looks for configuration files in the following order:

1. `./code_block_config.yaml` (current directory)
2. `./config/code_block_config.yaml`
3. `~/.config/md2docx/code_block_config.yaml` (user config)
4. `/etc/md2docx/code_block_config.yaml` (system config)

## Global Configuration

### Basic Settings

```yaml
global:
  # Enable or disable code block processing entirely
  enable_processing: true
  
  # Default timeout for processing operations (milliseconds)
  default_timeout_ms: 5000
  
  # Maximum number of processed code blocks to cache in memory
  max_cache_size: 1000
  
  # Log level for code block processing (error, warn, info, debug, trace)
  log_level: "info"
  
  # Enable fallback to default strategy when specific strategy fails
  enable_fallback_strategy: true
```

### Performance Settings

```yaml
global:
  # Enable parallel processing of multiple code blocks
  enable_parallel_processing: false
  
  # Maximum number of parallel threads (0 = auto-detect)
  max_parallel_threads: 0
  
  # Enable performance monitoring and metrics collection
  enable_performance_monitoring: false
  
  # Enable memory usage optimization
  enable_memory_optimization: true
```

### Advanced Settings

```yaml
global:
  # Enable experimental features (use with caution)
  enable_experimental_features: false
  
  # Custom processing pipeline configuration
  processing_pipeline:
    - "syntax_validation"
    - "formatting"
    - "optimization"
    - "quality_checks"
```

## Language-Specific Configuration

### Basic Language Configuration

Each language can have its own configuration section:

```yaml
languages:
  rust:
    # Enable syntax validation for Rust code
    enable_syntax_validation: true
    
    # Enable code formatting for Rust code
    enable_formatting: true
    
    # Enable code optimization (experimental)
    enable_optimization: false
    
    # Language-specific timeout (overrides global)
    timeout_ms: 10000
```

### Formatter Options

Different languages support different formatting options:

```yaml
languages:
  rust:
    enable_formatting: true
    formatter_options:
      edition: "2021"           # Rust edition
      max_width: 100            # Maximum line width
      hard_tabs: false          # Use spaces instead of tabs
      tab_spaces: 4             # Number of spaces per tab
      newline_style: "Unix"     # Line ending style
      indent_style: "Block"     # Indentation style
      wrap_comments: true       # Wrap long comments
      
  javascript:
    enable_formatting: true
    formatter_options:
      semicolons: true          # Add semicolons
      single_quotes: false      # Use double quotes
      tab_width: 2              # Indentation width
      trailing_comma: "es5"     # Trailing comma style
      bracket_spacing: true     # Space in object literals
      
  python:
    enable_formatting: true
    formatter_options:
      line_length: 88           # Maximum line length
      target_version: ["py38"]  # Python version targets
      skip_string_normalization: false
      
  json:
    enable_formatting: true
    formatter_options:
      indent_size: 2            # JSON indentation
      sort_keys: false          # Sort object keys
      ensure_ascii: false       # Allow unicode characters
```

### Quality Checks

Configure code quality checks for each language:

```yaml
languages:
  rust:
    quality_checks:
      warn_on_unwrap: true      # Warn about .unwrap() usage
      warn_on_panic: true       # Warn about panic! usage
      warn_on_todo: true        # Warn about TODO comments
      max_line_length: 100      # Maximum line length
      max_function_length: 50   # Maximum function length
      
  javascript:
    quality_checks:
      warn_on_console_log: true # Warn about console.log
      warn_on_debugger: true    # Warn about debugger statements
      max_complexity: 10        # Maximum cyclomatic complexity
      
  python:
    quality_checks:
      pep8_compliance: true     # Check PEP 8 compliance
      max_line_length: 88       # Maximum line length
      warn_on_unused_imports: true
```

### Custom Options

Add language-specific custom options:

```yaml
languages:
  rust:
    custom_options:
      clippy_lints: "true"              # Enable Clippy lints
      clippy_config: "clippy.toml"      # Clippy configuration file
      rustfmt_config: "rustfmt.toml"    # Rustfmt configuration file
      check_unsafe_code: "true"         # Check unsafe code blocks
      
  typescript:
    custom_options:
      tsconfig_path: "tsconfig.json"    # TypeScript config file
      type_checking: "true"             # Enable type checking
      strict_null_checks: "true"        # Strict null checks
      
  python:
    custom_options:
      black_config: "pyproject.toml"    # Black configuration
      pylint_config: ".pylintrc"        # Pylint configuration
      mypy_config: "mypy.ini"           # MyPy configuration
```

## Configuration Inheritance

### Extending Base Configurations

You can extend configurations from other languages:

```yaml
languages:
  # Base JavaScript configuration
  javascript:
    enable_syntax_validation: true
    enable_formatting: true
    formatter_options:
      semicolons: true
      tab_width: 2
      
  # TypeScript extends JavaScript
  typescript:
    extends: "javascript"       # Inherit from JavaScript config
    custom_options:
      type_checking: "true"     # Add TypeScript-specific options
      
  # JSX extends JavaScript with modifications
  jsx:
    extends: "javascript"
    formatter_options:
      jsx_bracket_same_line: false  # Override specific options
```

### Configuration Profiles

Define reusable configuration profiles:

```yaml
profiles:
  # Strict profile for production code
  strict:
    enable_syntax_validation: true
    enable_formatting: true
    quality_checks:
      max_line_length: 80
      warn_on_todos: true
      
  # Lenient profile for examples and demos
  lenient:
    enable_syntax_validation: false
    enable_formatting: true
    quality_checks:
      max_line_length: 120
      warn_on_todos: false

languages:
  rust:
    profile: "strict"           # Use the strict profile
    # Additional language-specific overrides
    formatter_options:
      edition: "2021"
      
  python:
    profile: "lenient"          # Use the lenient profile
```

## Plugin Configuration

### Plugin System Settings

```yaml
plugins:
  # Enable the plugin system
  enable_plugins: true
  
  # Directories to search for plugins
  plugin_paths:
    - "./plugins"
    - "~/.config/md2docx/plugins"
    - "/usr/local/lib/md2docx/plugins"
    
  # Automatically load plugins from search paths
  auto_load_plugins: true
  
  # Plugin loading timeout (milliseconds)
  plugin_load_timeout_ms: 30000
```

### Plugin-Specific Configuration

Configure individual plugins:

```yaml
plugins:
  plugin_configs:
    # Configuration for the JSON processor plugin
    "json-processor":
      enable_formatting: true
      indent_size: 4
      validate_schema: false
      
    # Configuration for a custom Python plugin
    "python-advanced":
      enable_type_hints: true
      check_imports: true
      format_docstrings: true
      
    # Configuration for an XML processor plugin
    "xml-processor":
      validate_dtd: false
      pretty_print: true
      preserve_whitespace: false
```

## Environment-Specific Configuration

### Development vs Production

Use different configurations for different environments:

```yaml
# development.yaml
global:
  log_level: "debug"
  enable_performance_monitoring: true
  
languages:
  rust:
    enable_syntax_validation: false  # Faster for development
    enable_formatting: true
    
---
# production.yaml
global:
  log_level: "warn"
  enable_performance_monitoring: false
  
languages:
  rust:
    enable_syntax_validation: true   # Strict for production
    enable_formatting: true
    quality_checks:
      warn_on_unwrap: true
```

### Conditional Configuration

Use environment variables or conditions:

```yaml
global:
  # Use environment variable for log level
  log_level: "${LOG_LEVEL:-info}"
  
  # Enable debug features only in development
  enable_debug_features: "${NODE_ENV:-production}" == "development"
  
languages:
  rust:
    # Use different timeouts based on environment
    timeout_ms: "${RUST_TIMEOUT:-5000}"
```

## Configuration Validation

### Schema Validation

The system validates configuration against a schema:

```yaml
# This configuration will be validated
global:
  enable_processing: true       # Must be boolean
  default_timeout_ms: 5000      # Must be positive integer
  max_cache_size: 1000          # Must be positive integer
  
languages:
  rust:
    enable_syntax_validation: true  # Must be boolean
    formatter_options:
      max_width: 100            # Must be integer between 1-1000
```

### Error Handling

Invalid configurations are handled gracefully:

```yaml
# Invalid configuration example
global:
  default_timeout_ms: "invalid"  # Error: must be integer
  
languages:
  rust:
    unknown_option: true         # Warning: unknown option ignored
```

## Configuration Examples

### Minimal Configuration

```yaml
# Minimal configuration - uses defaults for everything else
global:
  enable_processing: true
  
languages:
  rust:
    enable_formatting: true
```

### Comprehensive Configuration

```yaml
# Complete configuration example
global:
  enable_processing: true
  default_timeout_ms: 10000
  max_cache_size: 2000
  log_level: "info"
  enable_parallel_processing: true
  max_parallel_threads: 4

performance:
  enable_lazy_loading: true
  enable_memory_optimization: true
  enable_disk_cache: false

error_handling:
  continue_on_error: true
  max_errors: 10
  include_error_context: true

languages:
  rust:
    enable_syntax_validation: true
    enable_formatting: true
    timeout_ms: 15000
    formatter_options:
      edition: "2021"
      max_width: 100
      tab_spaces: 4
    quality_checks:
      warn_on_unwrap: true
      warn_on_panic: true
      max_line_length: 100
    custom_options:
      clippy_lints: "true"
      
  javascript:
    enable_syntax_validation: true
    enable_formatting: true
    formatter_options:
      semicolons: true
      single_quotes: false
      tab_width: 2
    quality_checks:
      warn_on_console_log: true
      max_complexity: 10
      
  python:
    enable_syntax_validation: true
    enable_formatting: true
    formatter_options:
      line_length: 88
      target_version: ["py38", "py39"]
    quality_checks:
      pep8_compliance: true
      warn_on_unused_imports: true

plugins:
  enable_plugins: true
  plugin_paths:
    - "./plugins"
    - "~/.config/md2docx/plugins"
  auto_load_plugins: true
  plugin_configs:
    "json-processor":
      indent_size: 2
      sort_keys: false
```

### Language-Specific Examples

#### Rust Configuration

```yaml
languages:
  rust:
    enable_syntax_validation: true
    enable_formatting: true
    enable_optimization: false
    timeout_ms: 10000
    
    formatter_options:
      edition: "2021"
      max_width: 100
      hard_tabs: false
      tab_spaces: 4
      newline_style: "Unix"
      indent_style: "Block"
      wrap_comments: true
      format_code_in_doc_comments: true
      normalize_comments: true
      
    quality_checks:
      warn_on_unwrap: true
      warn_on_panic: true
      warn_on_todo: true
      warn_on_fixme: true
      max_line_length: 100
      max_function_length: 100
      check_unsafe_code: true
      
    custom_options:
      clippy_lints: "true"
      clippy_config: "clippy.toml"
      rustfmt_config: "rustfmt.toml"
      cargo_check: "true"
```

#### JavaScript/TypeScript Configuration

```yaml
languages:
  javascript:
    enable_syntax_validation: true
    enable_formatting: true
    
    formatter_options:
      semicolons: true
      single_quotes: false
      tab_width: 2
      use_tabs: false
      trailing_comma: "es5"
      bracket_spacing: true
      jsx_bracket_same_line: false
      arrow_parens: "always"
      
    quality_checks:
      warn_on_console_log: true
      warn_on_debugger: true
      warn_on_alert: true
      max_complexity: 10
      max_depth: 4
      
    custom_options:
      eslint_config: ".eslintrc.json"
      prettier_config: ".prettierrc"
      babel_config: ".babelrc"
      
  typescript:
    extends: "javascript"
    custom_options:
      tsconfig_path: "tsconfig.json"
      type_checking: "true"
      strict_null_checks: "true"
      no_implicit_any: "true"
```

#### Python Configuration

```yaml
languages:
  python:
    enable_syntax_validation: true
    enable_formatting: true
    
    formatter_options:
      line_length: 88
      target_version: ["py38", "py39", "py310"]
      skip_string_normalization: false
      skip_magic_trailing_comma: false
      
    quality_checks:
      pep8_compliance: true
      max_line_length: 88
      max_function_length: 50
      warn_on_unused_imports: true
      check_type_hints: true
      
    custom_options:
      black_config: "pyproject.toml"
      pylint_config: ".pylintrc"
      mypy_config: "mypy.ini"
      isort_config: ".isort.cfg"
```

## Configuration Best Practices

### 1. Start Simple

Begin with a minimal configuration and add complexity as needed:

```yaml
# Start with this
global:
  enable_processing: true
  
languages:
  rust:
    enable_formatting: true
```

### 2. Use Profiles for Different Contexts

```yaml
profiles:
  development:
    enable_syntax_validation: false
    log_level: "debug"
    
  production:
    enable_syntax_validation: true
    log_level: "warn"
    
  ci:
    enable_syntax_validation: true
    enable_formatting: false  # Faster CI builds
    timeout_ms: 30000
```

### 3. Document Your Configuration

```yaml
# Configuration for MyProject
# Last updated: 2024-01-15
# Maintainer: dev-team@myproject.com

global:
  # We use longer timeouts because our CI is slow
  default_timeout_ms: 15000
  
languages:
  rust:
    # Enable strict checking for production code
    enable_syntax_validation: true
    
    # Use 2021 edition features
    formatter_options:
      edition: "2021"
```

### 4. Validate Your Configuration

Use the built-in validation to check your configuration:

```bash
# Validate configuration file
md2docx validate-config code_block_config.yaml

# Test configuration with sample code
md2docx test-config --config code_block_config.yaml --language rust sample.rs
```

### 5. Version Your Configuration

Keep your configuration files in version control and document changes:

```yaml
# Configuration version: 2.1.0
# Changes in this version:
# - Added TypeScript support
# - Updated Rust formatter options
# - Enabled parallel processing

metadata:
  version: "2.1.0"
  created: "2024-01-01"
  updated: "2024-01-15"
  
global:
  # ... rest of configuration
```

This configuration guide should help you customize the Code Block Strategy System to meet your specific needs. For more information about specific options, see the [API Documentation](CODE_BLOCK_STRATEGY_API.md).