//! Plugin system for code block processing strategies
//! 
//! This module provides a plugin architecture that allows dynamic loading
//! and registration of code block processing strategies. Plugins can be
//! loaded at runtime to extend the system with new language support.

use std::sync::Arc;
use std::collections::HashMap;
use crate::markdown::code_block::{CodeBlockStrategy, StrategyRegistry};

/// Trait that defines the interface for code block processing plugins
/// 
/// Plugins are responsible for creating and configuring code block processing
/// strategies. They provide metadata about themselves and can create strategy
/// instances on demand.
pub trait CodeBlockPlugin: Send + Sync {
    /// Get the unique name of this plugin
    fn name(&self) -> &str;
    
    /// Get the version of this plugin
    fn version(&self) -> &str;
    
    /// Get a description of what this plugin provides
    fn description(&self) -> &str;
    
    /// Get the author/maintainer of this plugin
    fn author(&self) -> &str {
        "Unknown"
    }
    
    /// Create a new strategy instance
    /// 
    /// This method should return a new instance of the strategy that this
    /// plugin provides. The strategy will be registered with the system.
    fn create_strategy(&self) -> Result<Box<dyn CodeBlockStrategy>, PluginError>;
    
    /// Get the languages this plugin supports
    /// 
    /// Returns a list of language names that this plugin can handle.
    /// This is used for plugin discovery and conflict resolution.
    fn supported_languages(&self) -> Vec<String>;
    
    /// Initialize the plugin
    /// 
    /// Called when the plugin is loaded. Can be used to perform any
    /// necessary setup or validation.
    fn initialize(&self) -> Result<(), PluginError> {
        Ok(())
    }
    
    /// Cleanup the plugin
    /// 
    /// Called when the plugin is unloaded. Should clean up any resources.
    fn cleanup(&self) -> Result<(), PluginError> {
        Ok(())
    }
    
    /// Get plugin configuration schema
    /// 
    /// Returns a description of the configuration options this plugin accepts.
    fn get_config_schema(&self) -> Option<PluginConfigSchema> {
        None
    }
    
    /// Configure the plugin with the given options
    fn configure(&mut self, _config: &HashMap<String, String>) -> Result<(), PluginError> {
        Ok(())
    }
}

/// Error type for plugin operations
#[derive(Debug, Clone)]
pub struct PluginError {
    pub message: String,
    pub error_type: PluginErrorType,
}

impl PluginError {
    /// Create a new plugin error
    pub fn new(error_type: PluginErrorType, message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            error_type,
        }
    }
    
    /// Create a loading error
    pub fn loading_error(message: impl Into<String>) -> Self {
        Self::new(PluginErrorType::LoadingError, message)
    }
    
    /// Create an initialization error
    pub fn initialization_error(message: impl Into<String>) -> Self {
        Self::new(PluginErrorType::InitializationError, message)
    }
    
    /// Create a configuration error
    pub fn configuration_error(message: impl Into<String>) -> Self {
        Self::new(PluginErrorType::ConfigurationError, message)
    }
    
    /// Create a strategy creation error
    pub fn strategy_creation_error(message: impl Into<String>) -> Self {
        Self::new(PluginErrorType::StrategyCreationError, message)
    }
}

impl std::fmt::Display for PluginError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}: {}", self.error_type, self.message)
    }
}

impl std::error::Error for PluginError {}

/// Types of plugin errors
#[derive(Debug, Clone, PartialEq)]
pub enum PluginErrorType {
    LoadingError,
    InitializationError,
    ConfigurationError,
    StrategyCreationError,
    DependencyError,
    VersionMismatchError,
}

/// Configuration schema for plugins
#[derive(Debug, Clone)]
pub struct PluginConfigSchema {
    pub options: Vec<ConfigOption>,
    pub required_options: Vec<String>,
}

/// A configuration option for a plugin
#[derive(Debug, Clone)]
pub struct ConfigOption {
    pub name: String,
    pub description: String,
    pub option_type: ConfigOptionType,
    pub default_value: Option<String>,
    pub required: bool,
}

/// Types of configuration options
#[derive(Debug, Clone)]
pub enum ConfigOptionType {
    String,
    Integer,
    Boolean,
    Float,
    List,
}

/// Plugin manager that handles loading, registration, and management of plugins
pub struct PluginManager {
    plugins: HashMap<String, Box<dyn CodeBlockPlugin>>,
    loaded_strategies: HashMap<String, String>, // language -> plugin_name mapping
    registry: Option<Arc<std::sync::Mutex<StrategyRegistry>>>,
}

impl PluginManager {
    /// Create a new plugin manager
    pub fn new() -> Self {
        Self {
            plugins: HashMap::new(),
            loaded_strategies: HashMap::new(),
            registry: None,
        }
    }
    
    /// Create a new plugin manager with a strategy registry
    pub fn with_registry(registry: Arc<std::sync::Mutex<StrategyRegistry>>) -> Self {
        Self {
            plugins: HashMap::new(),
            loaded_strategies: HashMap::new(),
            registry: Some(registry),
        }
    }
    
    /// Set the strategy registry for this plugin manager
    pub fn set_registry(&mut self, registry: Arc<std::sync::Mutex<StrategyRegistry>>) {
        self.registry = Some(registry);
    }
    
    /// Load a plugin into the manager
    pub fn load_plugin(&mut self, mut plugin: Box<dyn CodeBlockPlugin>) -> Result<(), PluginError> {
        let plugin_name = plugin.name().to_string();
        
        // Check if plugin is already loaded
        if self.plugins.contains_key(&plugin_name) {
            return Err(PluginError::loading_error(format!(
                "Plugin '{}' is already loaded", plugin_name
            )));
        }
        
        // Initialize the plugin
        plugin.initialize()?;
        
        // Create and register the strategy
        let strategy = plugin.create_strategy()?;
        
        if let Some(registry) = &self.registry {
            let mut reg = registry.lock().map_err(|e| {
                PluginError::loading_error(format!("Failed to lock registry: {}", e))
            })?;
            
            reg.register_boxed_strategy(strategy);
            
            // Track which languages this plugin provides
            for language in plugin.supported_languages() {
                self.loaded_strategies.insert(language.to_lowercase(), plugin_name.clone());
            }
        }
        
        // Store the plugin
        self.plugins.insert(plugin_name, plugin);
        
        Ok(())
    }
    
    /// Unload a plugin by name
    pub fn unload_plugin(&mut self, plugin_name: &str) -> Result<(), PluginError> {
        if let Some(mut plugin) = self.plugins.remove(plugin_name) {
            // Cleanup the plugin
            plugin.cleanup()?;
            
            // Remove language mappings
            self.loaded_strategies.retain(|_, name| name != plugin_name);
            
            // Note: We don't remove strategies from the registry as that would
            // require more complex tracking. In a real implementation, you might
            // want to implement strategy removal from the registry.
            
            Ok(())
        } else {
            Err(PluginError::loading_error(format!(
                "Plugin '{}' is not loaded", plugin_name
            )))
        }
    }
    
    /// Get information about a loaded plugin
    pub fn get_plugin_info(&self, plugin_name: &str) -> Option<PluginInfo> {
        self.plugins.get(plugin_name).map(|plugin| PluginInfo {
            name: plugin.name().to_string(),
            version: plugin.version().to_string(),
            description: plugin.description().to_string(),
            author: plugin.author().to_string(),
            supported_languages: plugin.supported_languages(),
        })
    }
    
    /// List all loaded plugins
    pub fn list_plugins(&self) -> Vec<PluginInfo> {
        self.plugins.values().map(|plugin| PluginInfo {
            name: plugin.name().to_string(),
            version: plugin.version().to_string(),
            description: plugin.description().to_string(),
            author: plugin.author().to_string(),
            supported_languages: plugin.supported_languages(),
        }).collect()
    }
    
    /// Register all loaded plugins with the strategy registry
    pub fn register_all_strategies(&self) -> Result<(), PluginError> {
        if let Some(registry) = &self.registry {
            let mut reg = registry.lock().map_err(|e| {
                PluginError::loading_error(format!("Failed to lock registry: {}", e))
            })?;
            
            for plugin in self.plugins.values() {
                let strategy = plugin.create_strategy()?;
                reg.register_boxed_strategy(strategy);
            }
        }
        
        Ok(())
    }
    
    /// Configure a plugin with the given options
    pub fn configure_plugin(&mut self, plugin_name: &str, config: &HashMap<String, String>) -> Result<(), PluginError> {
        if let Some(plugin) = self.plugins.get_mut(plugin_name) {
            plugin.configure(config)
        } else {
            Err(PluginError::configuration_error(format!(
                "Plugin '{}' is not loaded", plugin_name
            )))
        }
    }
    
    /// Get the plugin that provides support for a given language
    pub fn get_plugin_for_language(&self, language: &str) -> Option<&str> {
        self.loaded_strategies.get(&language.to_lowercase()).map(|s| s.as_str())
    }
    
    /// Check if a plugin is loaded
    pub fn is_plugin_loaded(&self, plugin_name: &str) -> bool {
        self.plugins.contains_key(plugin_name)
    }
    
    /// Get the number of loaded plugins
    pub fn plugin_count(&self) -> usize {
        self.plugins.len()
    }
    
    /// Clear all loaded plugins
    pub fn clear(&mut self) -> Result<(), PluginError> {
        let plugin_names: Vec<String> = self.plugins.keys().cloned().collect();
        
        for plugin_name in plugin_names {
            self.unload_plugin(&plugin_name)?;
        }
        
        Ok(())
    }
}

impl Default for PluginManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Information about a plugin
#[derive(Debug, Clone)]
pub struct PluginInfo {
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub supported_languages: Vec<String>,
}

impl PluginInfo {
    /// Get a formatted string representation of the plugin info
    pub fn format(&self) -> String {
        format!(
            "{} v{} by {} - {} (supports: {})",
            self.name,
            self.version,
            self.author,
            self.description,
            self.supported_languages.join(", ")
        )
    }
}

// Tests are included inline in this file
#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    use crate::markdown::code_block::{ProcessingConfig, ProcessedCodeBlock, ProcessingError};

    // Mock plugin for testing
    #[derive(Debug)]
    struct MockPlugin {
        name: String,
        languages: Vec<String>,
    }

    impl MockPlugin {
        fn new(name: &str, languages: Vec<&str>) -> Self {
            Self {
                name: name.to_string(),
                languages: languages.iter().map(|s| s.to_string()).collect(),
            }
        }
    }

    impl CodeBlockPlugin for MockPlugin {
        fn name(&self) -> &str {
            &self.name
        }
        
        fn version(&self) -> &str {
            "1.0.0"
        }
        
        fn description(&self) -> &str {
            "Mock plugin for testing"
        }
        
        fn create_strategy(&self) -> Result<Box<dyn CodeBlockStrategy>, PluginError> {
            Ok(Box::new(MockStrategy::new(&self.name)))
        }
        
        fn supported_languages(&self) -> Vec<String> {
            self.languages.clone()
        }
    }

    // Mock strategy for testing
    #[derive(Debug)]
    struct MockStrategy {
        name: String,
    }

    impl MockStrategy {
        fn new(name: &str) -> Self {
            Self {
                name: name.to_string(),
            }
        }
    }

    impl CodeBlockStrategy for MockStrategy {
        fn process(&self, code: &str, _config: &ProcessingConfig) -> Result<ProcessedCodeBlock, ProcessingError> {
            Ok(ProcessedCodeBlock::new(code.to_string(), Some(self.name.clone())))
        }

        fn supports_language(&self, language: &str) -> bool {
            language == self.name
        }

        fn get_language_name(&self) -> &'static str {
            "mock"
        }
    }

    #[test]
    fn test_plugin_manager_creation() {
        let manager = PluginManager::new();
        assert_eq!(manager.plugin_count(), 0);
        assert!(manager.list_plugins().is_empty());
    }

    #[test]
    fn test_plugin_manager_with_registry() {
        let registry = Arc::new(Mutex::new(StrategyRegistry::new()));
        let manager = PluginManager::with_registry(registry);
        assert_eq!(manager.plugin_count(), 0);
    }

    #[test]
    fn test_load_plugin() {
        let mut manager = PluginManager::new();
        let registry = Arc::new(Mutex::new(StrategyRegistry::new()));
        manager.set_registry(registry.clone());
        
        let plugin = Box::new(MockPlugin::new("test-plugin", vec!["test"]));
        let result = manager.load_plugin(plugin);
        
        assert!(result.is_ok());
        assert_eq!(manager.plugin_count(), 1);
        assert!(manager.is_plugin_loaded("test-plugin"));
    }

    #[test]
    fn test_plugin_error_types() {
        let error = PluginError::loading_error("Test loading error");
        assert_eq!(error.error_type, PluginErrorType::LoadingError);
        assert_eq!(error.message, "Test loading error");
        
        let error = PluginError::configuration_error("Test config error");
        assert_eq!(error.error_type, PluginErrorType::ConfigurationError);
    }

    #[test]
    fn test_plugin_info_format() {
        let info = PluginInfo {
            name: "test-plugin".to_string(),
            version: "1.0.0".to_string(),
            description: "Test plugin".to_string(),
            author: "Test Author".to_string(),
            supported_languages: vec!["test".to_string(), "example".to_string()],
        };
        
        let formatted = info.format();
        assert!(formatted.contains("test-plugin"));
        assert!(formatted.contains("1.0.0"));
        assert!(formatted.contains("Test Author"));
        assert!(formatted.contains("test, example"));
    }
}