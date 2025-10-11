// Example demonstrating how to use the code block configuration system

use md2docx_converter::markdown::code_block::{CodeBlockConfig, LanguageConfig};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Example 1: Create a configuration programmatically
    println!("=== Example 1: Programmatic Configuration ===");
    
    let config = CodeBlockConfig::new()
        .with_language_config("rust", 
            LanguageConfig::new()
                .with_syntax_validation(true)
                .with_formatting(true)
                .with_formatter_option("edition", "2021")
                .with_custom_option("clippy", "true")
        )
        .with_language_config("python", 
            LanguageConfig::new()
                .with_syntax_validation(true)
                .with_formatting(true)
                .with_formatter_option("line_length", "88")
        );
    
    // Create processing config for Rust
    let rust_processing_config = config.create_processing_config(Some("rust"));
    println!("Rust processing config:");
    println!("  Syntax validation: {}", rust_processing_config.enable_syntax_validation);
    println!("  Formatting: {}", rust_processing_config.enable_formatting);
    println!("  Timeout: {}ms", rust_processing_config.timeout_ms);
    
    // Example 2: Load configuration from YAML file
    println!("\n=== Example 2: Load from YAML File ===");
    
    match CodeBlockConfig::from_file("examples/code_block_config.yaml") {
        Ok(loaded_config) => {
            println!("Successfully loaded configuration from file");
            println!("Global processing enabled: {}", loaded_config.is_processing_enabled());
            println!("Configured languages: {:?}", loaded_config.get_configured_languages());
            
            // Test JavaScript configuration
            let js_config = loaded_config.get_language_config("javascript");
            println!("JavaScript config:");
            println!("  Syntax validation: {}", js_config.enable_syntax_validation);
            println!("  Formatting: {}", js_config.enable_formatting);
            if let Some(semicolons) = js_config.formatter_options.get("semicolons") {
                println!("  Semicolons: {}", semicolons);
            }
        }
        Err(e) => {
            println!("Failed to load config: {}", e);
        }
    }
    
    // Example 3: Create configuration with common languages
    println!("\n=== Example 3: Common Languages Configuration ===");
    
    let common_config = CodeBlockConfig::with_common_languages();
    println!("Common languages configuration created");
    println!("Languages: {:?}", common_config.get_configured_languages());
    
    // Example 4: Configuration validation and normalization
    println!("\n=== Example 4: Configuration Validation ===");
    
    let yaml_with_issues = r#"
global:
  enable_processing: true
  default_timeout_ms: 50  # Too low, will be normalized
  max_cache_size: 5       # Too low, will be normalized
  enable_parallel_processing: false
languages:
  RUST:  # Will be normalized to lowercase
    enable_syntax_validation: true
    enable_formatting: true
    formatter_options: {}
    custom_options: {}
  unknown_language:  # Advanced features will be disabled
    enable_syntax_validation: true
    enable_formatting: true
    formatter_options: {}
    custom_options: {}
"#;
    
    match CodeBlockConfig::from_yaml(yaml_with_issues) {
        Ok(validated_config) => {
            println!("Configuration validated and normalized:");
            println!("  Timeout normalized to: {}ms", validated_config.global.default_timeout_ms);
            println!("  Cache size normalized to: {}", validated_config.global.max_cache_size);
            println!("  Has 'rust' config: {}", validated_config.has_language_config("rust"));
            println!("  Has 'RUST' config: {}", validated_config.has_language_config("RUST"));
            
            let unknown_config = validated_config.get_language_config("unknown_language");
            println!("  Unknown language - syntax validation: {}", unknown_config.enable_syntax_validation);
            println!("  Unknown language - formatting: {}", unknown_config.enable_formatting);
        }
        Err(e) => {
            println!("Validation failed: {}", e);
        }
    }
    
    // Example 5: Save configuration to file
    println!("\n=== Example 5: Save Configuration ===");
    
    let save_config = CodeBlockConfig::with_common_languages();
    match save_config.save_to_file("examples/generated_config.yaml") {
        Ok(()) => {
            println!("Configuration saved to examples/generated_config.yaml");
        }
        Err(e) => {
            println!("Failed to save config: {}", e);
        }
    }
    
    // Example 6: Configuration merging
    println!("\n=== Example 6: Configuration Merging ===");
    
    let mut base_config = CodeBlockConfig::new();
    base_config.global.default_timeout_ms = 3000;
    
    let override_config = CodeBlockConfig::new()
        .with_language_config("rust", 
            LanguageConfig::new()
                .with_syntax_validation(false)
                .with_formatting(true)
        );
    
    base_config.merge_with(&override_config);
    println!("Configurations merged successfully");
    
    let merged_rust_config = base_config.get_language_config("rust");
    println!("Merged Rust config - formatting: {}", merged_rust_config.enable_formatting);
    
    Ok(())
}