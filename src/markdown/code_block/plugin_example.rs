//! Plugin system usage examples
//! 
//! This module provides examples of how to use the plugin system to extend
//! the code block processing capabilities.

use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use crate::markdown::code_block::{
    PluginManager, StrategyRegistry, ProcessingConfig
};
use super::{JsonPlugin, XmlPlugin};

/// Example of setting up a plugin system with multiple plugins
pub fn setup_plugin_system_example() -> Result<(PluginManager, Arc<Mutex<StrategyRegistry>>), Box<dyn std::error::Error>> {
    // Create a strategy registry
    let registry = Arc::new(Mutex::new(StrategyRegistry::default()));
    
    // Create a plugin manager with the registry
    let mut manager = PluginManager::with_registry(registry.clone());
    
    // Load JSON plugin
    let json_plugin = Box::new(JsonPlugin::new());
    manager.load_plugin(json_plugin)?;
    
    // Load XML plugin
    let xml_plugin = Box::new(XmlPlugin::new());
    manager.load_plugin(xml_plugin)?;
    
    // Configure the JSON plugin
    let mut json_config = HashMap::new();
    json_config.insert("enable_formatting".to_string(), "true".to_string());
    json_config.insert("indent_size".to_string(), "4".to_string());
    manager.configure_plugin("json-processor", &json_config)?;
    
    println!("Plugin system setup complete!");
    println!("Loaded plugins:");
    for plugin_info in manager.list_plugins() {
        println!("  - {}", plugin_info.format());
    }
    
    Ok((manager, registry))
}

/// Example of processing code blocks with plugins
pub fn process_code_blocks_example() -> Result<(), Box<dyn std::error::Error>> {
    let (_manager, registry) = setup_plugin_system_example()?;
    
    // Example JSON code block
    let json_code = r#"{
  "name": "example",
  "version": "1.0.0",
  "dependencies": {
    "serde": "1.0"
  }
}"#;
    
    // Example XML code block
    let xml_code = r#"<?xml version="1.0" encoding="UTF-8"?>
<root>
    <item id="1">
        <name>Example</name>
        <value>42</value>
    </item>
</root>"#;
    
    let config = ProcessingConfig::default();
    
    // Process JSON
    {
        let reg = registry.lock().unwrap();
        let json_strategy = reg.get_strategy("json");
        
        println!("\nProcessing JSON code block:");
        match json_strategy.process(json_code, &config) {
            Ok(processed) => {
                println!("  Original length: {}", processed.original_code.len());
                println!("  Syntax valid: {}", processed.metadata.syntax_valid);
                println!("  Was formatted: {}", processed.metadata.is_formatted);
                if let Some(formatted) = &processed.processed_code {
                    println!("  Formatted length: {}", formatted.len());
                }
            }
            Err(e) => println!("  Error: {}", e),
        }
    }
    
    // Process XML
    {
        let reg = registry.lock().unwrap();
        let xml_strategy = reg.get_strategy("xml");
        
        println!("\nProcessing XML code block:");
        match xml_strategy.process(xml_code, &config) {
            Ok(processed) => {
                println!("  Original length: {}", processed.original_code.len());
                println!("  Syntax valid: {}", processed.metadata.syntax_valid);
                println!("  Processing time: {:?}", processed.metadata.processing_time);
            }
            Err(e) => println!("  Error: {}", e),
        }
    }
    
    Ok(())
}

/// Example of dynamic plugin loading and unloading
pub fn dynamic_plugin_management_example() -> Result<(), Box<dyn std::error::Error>> {
    let registry = Arc::new(Mutex::new(StrategyRegistry::default()));
    let mut manager = PluginManager::with_registry(registry.clone());
    
    println!("Initial plugin count: {}", manager.plugin_count());
    
    // Load plugins dynamically
    println!("\nLoading JSON plugin...");
    let json_plugin = Box::new(JsonPlugin::new());
    manager.load_plugin(json_plugin)?;
    println!("Plugin count after loading JSON: {}", manager.plugin_count());
    
    println!("\nLoading XML plugin...");
    let xml_plugin = Box::new(XmlPlugin::new());
    manager.load_plugin(xml_plugin)?;
    println!("Plugin count after loading XML: {}", manager.plugin_count());
    
    // List all plugins
    println!("\nCurrently loaded plugins:");
    for plugin_info in manager.list_plugins() {
        println!("  - {} v{} ({})", 
                 plugin_info.name, 
                 plugin_info.version, 
                 plugin_info.supported_languages.join(", "));
    }
    
    // Test language support
    println!("\nTesting language support:");
    println!("  JSON support: {:?}", manager.get_plugin_for_language("json"));
    println!("  XML support: {:?}", manager.get_plugin_for_language("xml"));
    println!("  HTML support: {:?}", manager.get_plugin_for_language("html"));
    println!("  Unknown support: {:?}", manager.get_plugin_for_language("unknown"));
    
    // Unload a plugin
    println!("\nUnloading JSON plugin...");
    manager.unload_plugin("json-processor")?;
    println!("Plugin count after unloading JSON: {}", manager.plugin_count());
    
    // Clear all plugins
    println!("\nClearing all plugins...");
    manager.clear()?;
    println!("Final plugin count: {}", manager.plugin_count());
    
    Ok(())
}

/// Example of plugin configuration
pub fn plugin_configuration_example() -> Result<(), Box<dyn std::error::Error>> {
    let registry = Arc::new(Mutex::new(StrategyRegistry::default()));
    let mut manager = PluginManager::with_registry(registry.clone());
    
    // Load JSON plugin
    let json_plugin = Box::new(JsonPlugin::new());
    manager.load_plugin(json_plugin)?;
    
    // Get plugin info before configuration
    if let Some(info) = manager.get_plugin_info("json-processor") {
        println!("Plugin: {}", info.format());
    }
    
    // Configure the plugin
    let mut config = HashMap::new();
    config.insert("enable_formatting".to_string(), "true".to_string());
    config.insert("enable_validation".to_string(), "true".to_string());
    config.insert("indent_size".to_string(), "2".to_string());
    
    println!("\nConfiguring JSON plugin with:");
    for (key, value) in &config {
        println!("  {}: {}", key, value);
    }
    
    manager.configure_plugin("json-processor", &config)?;
    println!("Configuration applied successfully!");
    
    // Test the configured plugin
    let json_code = r#"{"compact":true,"data":[1,2,3]}"#;
    let processing_config = ProcessingConfig::default();
    
    {
        let reg = registry.lock().unwrap();
        let strategy = reg.get_strategy("json");
        
        match strategy.process(json_code, &processing_config) {
            Ok(processed) => {
                println!("\nProcessed JSON:");
                println!("  Original: {}", processed.original_code);
                if let Some(formatted) = &processed.processed_code {
                    println!("  Formatted:\n{}", formatted);
                }
            }
            Err(e) => println!("  Error: {}", e),
        }
    }
    
    Ok(())
}

/// Run all plugin examples
pub fn run_all_examples() {
    println!("=== Plugin System Examples ===\n");
    
    println!("1. Setting up plugin system:");
    if let Err(e) = setup_plugin_system_example() {
        println!("Error: {}", e);
    }
    
    println!("\n2. Processing code blocks:");
    if let Err(e) = process_code_blocks_example() {
        println!("Error: {}", e);
    }
    
    println!("\n3. Dynamic plugin management:");
    if let Err(e) = dynamic_plugin_management_example() {
        println!("Error: {}", e);
    }
    
    println!("\n4. Plugin configuration:");
    if let Err(e) = plugin_configuration_example() {
        println!("Error: {}", e);
    }
    
    println!("\n=== Examples Complete ===");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_setup_plugin_system_example() {
        let result = setup_plugin_system_example();
        assert!(result.is_ok());
        
        let (manager, _registry) = result.unwrap();
        assert_eq!(manager.plugin_count(), 2);
    }

    #[test]
    fn test_process_code_blocks_example() {
        let result = process_code_blocks_example();
        assert!(result.is_ok());
    }

    #[test]
    fn test_dynamic_plugin_management_example() {
        let result = dynamic_plugin_management_example();
        assert!(result.is_ok());
    }

    #[test]
    fn test_plugin_configuration_example() {
        let result = plugin_configuration_example();
        assert!(result.is_ok());
    }
}