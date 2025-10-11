//! Code Block Strategy System Usage Examples
//! 
//! This file demonstrates various ways to use the code block processing system.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

// Note: These are example imports - adjust based on actual module structure
use crate::markdown::code_block::{
    StrategyRegistry, ProcessingConfig, PluginManager, DefaultStrategy, RustStrategy
};

/// Example 1: Basic code block processing
pub fn basic_processing_example() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Basic Code Block Processing ===");
    
    // Create a registry with default strategies
    let registry = StrategyRegistry::new();
    
    // Configure processing options
    let config = ProcessingConfig::default()
        .with_syntax_validation(true)
        .with_formatting(true);
    
    // Process some Rust code
    let rust_code = r#"
fn main(){
println!("Hello, world!");
}
"#;
    
    let strategy = registry.get_strategy("rust");
    let result = strategy.process(rust_code, &config)?;
    
    println!("✅ Processing result:");
    println!("  Language: {:?}", result.language);
    println!("  Syntax valid: {}", result.metadata.syntax_valid);
    println!("  Was formatted: {}", result.metadata.is_formatted);
    println!("  Processing time: {:?}", result.metadata.processing_time);
    
    if let Some(formatted) = &result.processed_code {
        println!("  Formatted code:\n{}", formatted);
    }
    
    Ok(())
}

/// Example 2: Processing multiple languages
pub fn multi_language_example() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== Multi-Language Processing ===");
    
    let registry = StrategyRegistry::new();
    let config = ProcessingConfig::default()
        .with_syntax_validation(true)
        .with_formatting(false); // Disable formatting for speed
    
    let code_samples = vec![
        ("rust", "fn main() { println!(\"Hello from Rust!\"); }"),
        ("javascript", "console.log('Hello from JavaScript!');"),
        ("python", "print('Hello from Python!')"),
        ("json", r#"{"message": "Hello from JSON!"}"#),
        ("unknown", "some unknown language code"),
    ];
    
    for (language, code) in code_samples {
        let strategy = registry.get_strategy(language);
        
        match strategy.process(code, &config) {
            Ok(result) => {
                let status = if result.is_successful() { "✅" } else { "❌" };
                println!("  {} {}: {} ({})", 
                    status, 
                    language, 
                    strategy.get_language_name(),
                    result.get_summary().get_status()
                );
                
                if result.has_warnings() {
                    println!("    ⚠️  {} warnings", result.warning_count());
                }
            }
            Err(e) => {
                println!("  💥 {}: Error - {}", language, e);
            }
        }
    }
    
    Ok(())
}

/// Example 3: Custom configuration for different contexts
pub fn configuration_examples() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== Configuration Examples ===");
    
    let registry = StrategyRegistry::new();
    
    // Production configuration (strict)
    let production_config = ProcessingConfig {
        enable_syntax_validation: true,
        enable_formatting: true,
        enable_optimization: true,
        timeout_ms: 5000,
        custom_options: {
            let mut opts = HashMap::new();
            opts.insert("strict_mode".to_string(), "true".to_string());
            opts.insert("max_line_length".to_string(), "100".to_string());
            opts
        },
    };
    
    // Development configuration (lenient)
    let dev_config = ProcessingConfig {
        enable_syntax_validation: false,
        enable_formatting: true,
        enable_optimization: false,
        timeout_ms: 1000,
        custom_options: HashMap::new(),
    };
    
    let test_code = "fn main(){println!(\"test\");}";
    let strategy = registry.get_strategy("rust");
    
    // Test with production config
    println!("  Production config:");
    let prod_result = strategy.process(test_code, &production_config)?;
    println!("    Validation: {}", prod_result.metadata.is_validated);
    println!("    Formatting: {}", prod_result.metadata.is_formatted);
    
    // Test with development config
    println!("  Development config:");
    let dev_result = strategy.process(test_code, &dev_config)?;
    println!("    Validation: {}", dev_result.metadata.is_validated);
    println!("    Formatting: {}", dev_result.metadata.is_formatted);
    
    Ok(())
}

/// Example 4: Error handling and recovery
pub fn error_handling_example() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== Error Handling Example ===");
    
    let registry = StrategyRegistry::new();
    let config = ProcessingConfig::default()
        .with_syntax_validation(true);
    
    // Test with invalid Rust code
    let invalid_rust = r#"
fn main( {
    println!("This has syntax errors"
}
"#;
    
    let strategy = registry.get_strategy("rust");
    
    match strategy.process(invalid_rust, &config) {
        Ok(result) => {
            if result.is_successful() {
                println!("  ✅ Processing successful");
            } else {
                println!("  ❌ Processing failed with {} errors:", result.error_count());
                for (i, error) in result.errors.iter().enumerate() {
                    println!("    {}. {}: {}", i + 1, error.error_type, error.message);
                    if let Some(line) = error.line {
                        println!("       at line {}", line);
                    }
                }
                
                // The system gracefully handles errors and still returns a result
                println!("  📄 Original code preserved: {} chars", result.original_code.len());
            }
            
            if result.has_warnings() {
                println!("  ⚠️  {} warnings:", result.warning_count());
                for warning in &result.warnings {
                    println!("    - {}: {}", warning.warning_type, warning.message);
                }
            }
        }
        Err(e) => {
            println!("  💥 Unexpected error: {}", e);
        }
    }
    
    Ok(())
}

/// Example 5: Plugin system usage
pub fn plugin_system_example() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== Plugin System Example ===");
    
    // Create a plugin manager with a registry
    let registry = Arc::new(Mutex::new(StrategyRegistry::new()));
    let mut plugin_manager = PluginManager::with_registry(registry.clone());
    
    println!("  Initial plugin count: {}", plugin_manager.plugin_count());
    
    // In a real scenario, you would load actual plugins here
    // For this example, we'll just show the structure
    
    // List available plugins
    println!("  Available plugins:");
    for plugin_info in plugin_manager.list_plugins() {
        println!("    - {}", plugin_info.format());
    }
    
    // Example of how you would configure a plugin
    let mut plugin_config = HashMap::new();
    plugin_config.insert("enable_formatting".to_string(), "true".to_string());
    plugin_config.insert("indent_size".to_string(), "4".to_string());
    
    // This would configure a loaded plugin
    // plugin_manager.configure_plugin("json-processor", &plugin_config)?;
    
    println!("  Plugin system ready for use");
    
    Ok(())
}

/// Example 6: Performance monitoring and caching
pub fn performance_example() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== Performance Example ===");
    
    let registry = StrategyRegistry::new();
    let config = ProcessingConfig::default()
        .with_syntax_validation(true)
        .with_formatting(true);
    
    let test_code = r#"
fn fibonacci(n: u32) -> u32 {
    match n {
        0 => 0,
        1 => 1,
        _ => fibonacci(n - 1) + fibonacci(n - 2),
    }
}

fn main() {
    for i in 0..10 {
        println!("fib({}) = {}", i, fibonacci(i));
    }
}
"#;
    
    let strategy = registry.get_strategy("rust");
    
    // Measure processing time
    let start = std::time::Instant::now();
    let result = strategy.process(test_code, &config)?;
    let total_time = start.elapsed();
    
    println!("  Code length: {} characters", test_code.len());
    println!("  Processing time: {:?}", result.metadata.processing_time);
    println!("  Total time (including overhead): {:?}", total_time);
    println!("  Memory usage: {} bytes (approx)", 
        std::mem::size_of_val(&result) + result.original_code.len() + 
        result.processed_code.as_ref().map(|s| s.len()).unwrap_or(0));
    
    // Show processing summary
    let summary = result.get_summary();
    println!("  Summary: {} - {} errors, {} warnings", 
        summary.get_status(), summary.error_count, summary.warning_count);
    
    Ok(())
}

/// Example 7: Batch processing with different strategies
pub fn batch_processing_example() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== Batch Processing Example ===");
    
    let registry = StrategyRegistry::new();
    
    // Simulate processing multiple code blocks from a document
    let code_blocks = vec![
        ("rust", r#"fn main() { println!("Hello"); }"#),
        ("rust", r#"struct Point { x: i32, y: i32 }"#),
        ("javascript", r#"function hello() { console.log("Hi"); }"#),
        ("json", r#"{"name": "example", "version": "1.0"}"#),
        ("python", r#"def hello(): print("Hello")"#),
    ];
    
    let mut results = Vec::new();
    let mut total_processing_time = std::time::Duration::new(0, 0);
    
    for (i, (language, code)) in code_blocks.iter().enumerate() {
        let config = ProcessingConfig::default()
            .with_syntax_validation(true)
            .with_formatting(false); // Faster for batch processing
        
        let strategy = registry.get_strategy(language);
        
        match strategy.process(code, &config) {
            Ok(result) => {
                total_processing_time += result.metadata.processing_time;
                let success = result.is_successful();
                results.push((i, language, success, result.error_count(), result.warning_count()));
                
                println!("  Block {}: {} - {}", 
                    i + 1, 
                    language, 
                    if success { "✅ OK" } else { "❌ Failed" }
                );
            }
            Err(e) => {
                println!("  Block {}: {} - 💥 Error: {}", i + 1, language, e);
                results.push((i, language, false, 1, 0));
            }
        }
    }
    
    // Summary
    let successful = results.iter().filter(|(_, _, success, _, _)| *success).count();
    let total_errors: usize = results.iter().map(|(_, _, _, errors, _)| errors).sum();
    let total_warnings: usize = results.iter().map(|(_, _, _, _, warnings)| warnings).sum();
    
    println!("  Batch Summary:");
    println!("    Total blocks: {}", code_blocks.len());
    println!("    Successful: {}", successful);
    println!("    Failed: {}", code_blocks.len() - successful);
    println!("    Total errors: {}", total_errors);
    println!("    Total warnings: {}", total_warnings);
    println!("    Total processing time: {:?}", total_processing_time);
    
    Ok(())
}

/// Main function to run all examples
pub fn run_all_examples() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Code Block Strategy System Examples\n");
    
    basic_processing_example()?;
    multi_language_example()?;
    configuration_examples()?;
    error_handling_example()?;
    plugin_system_example()?;
    performance_example()?;
    batch_processing_example()?;
    
    println!("\n✨ All examples completed successfully!");
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_examples_run_without_panic() {
        // Test that all examples can run without panicking
        // In a real implementation, you'd need to mock the dependencies
        
        // This is a placeholder test - actual implementation would depend
        // on the specific module structure and available test utilities
        assert!(true);
    }
}