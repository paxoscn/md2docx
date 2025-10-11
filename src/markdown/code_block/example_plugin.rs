//! Example plugin implementation
//! 
//! This module provides example implementations of plugins to demonstrate
//! how to create and use plugins with the code block processing system.

use std::collections::HashMap;
use std::time::Instant;
use crate::markdown::code_block::{
    CodeBlockPlugin, CodeBlockStrategy, PluginError, PluginConfigSchema, ConfigOption, 
    ConfigOptionType, ProcessedCodeBlock, ProcessingConfig, ProcessingError, ProcessingMetadata
};

/// Example plugin that provides basic JSON processing
#[derive(Debug)]
pub struct JsonPlugin {
    config: JsonPluginConfig,
}

#[derive(Debug, Clone)]
struct JsonPluginConfig {
    enable_formatting: bool,
    enable_validation: bool,
    indent_size: usize,
}

impl Default for JsonPluginConfig {
    fn default() -> Self {
        Self {
            enable_formatting: true,
            enable_validation: true,
            indent_size: 2,
        }
    }
}

impl JsonPlugin {
    /// Create a new JSON plugin with default configuration
    pub fn new() -> Self {
        Self {
            config: JsonPluginConfig::default(),
        }
    }
    
    /// Create a new JSON plugin with custom configuration
    pub fn with_config(config: JsonPluginConfig) -> Self {
        Self { config }
    }
}

impl CodeBlockPlugin for JsonPlugin {
    fn name(&self) -> &str {
        "json-processor"
    }
    
    fn version(&self) -> &str {
        "1.0.0"
    }
    
    fn description(&self) -> &str {
        "JSON code block processor with validation and formatting"
    }
    
    fn author(&self) -> &str {
        "Code Block Plugin System"
    }
    
    fn create_strategy(&self) -> Result<Box<dyn CodeBlockStrategy>, PluginError> {
        Ok(Box::new(JsonStrategy::new(self.config.clone())))
    }
    
    fn supported_languages(&self) -> Vec<String> {
        vec!["json".to_string(), "jsonc".to_string()]
    }
    
    fn initialize(&self) -> Result<(), PluginError> {
        // Perform any initialization logic here
        // For example, check if required dependencies are available
        Ok(())
    }
    
    fn get_config_schema(&self) -> Option<PluginConfigSchema> {
        Some(PluginConfigSchema {
            options: vec![
                ConfigOption {
                    name: "enable_formatting".to_string(),
                    description: "Enable JSON formatting".to_string(),
                    option_type: ConfigOptionType::Boolean,
                    default_value: Some("true".to_string()),
                    required: false,
                },
                ConfigOption {
                    name: "enable_validation".to_string(),
                    description: "Enable JSON validation".to_string(),
                    option_type: ConfigOptionType::Boolean,
                    default_value: Some("true".to_string()),
                    required: false,
                },
                ConfigOption {
                    name: "indent_size".to_string(),
                    description: "Number of spaces for indentation".to_string(),
                    option_type: ConfigOptionType::Integer,
                    default_value: Some("2".to_string()),
                    required: false,
                },
            ],
            required_options: vec![],
        })
    }
    
    fn configure(&mut self, config: &HashMap<String, String>) -> Result<(), PluginError> {
        if let Some(enable_formatting) = config.get("enable_formatting") {
            self.config.enable_formatting = enable_formatting.parse().map_err(|_| {
                PluginError::configuration_error("Invalid value for enable_formatting")
            })?;
        }
        
        if let Some(enable_validation) = config.get("enable_validation") {
            self.config.enable_validation = enable_validation.parse().map_err(|_| {
                PluginError::configuration_error("Invalid value for enable_validation")
            })?;
        }
        
        if let Some(indent_size) = config.get("indent_size") {
            self.config.indent_size = indent_size.parse().map_err(|_| {
                PluginError::configuration_error("Invalid value for indent_size")
            })?;
        }
        
        Ok(())
    }
}

/// JSON processing strategy implementation
#[derive(Debug, Clone)]
struct JsonStrategy {
    config: JsonPluginConfig,
}

impl JsonStrategy {
    fn new(config: JsonPluginConfig) -> Self {
        Self { config }
    }
    
    fn validate_json(&self, code: &str) -> Result<bool, ProcessingError> {
        // Simple JSON validation using serde_json
        match serde_json::from_str::<serde_json::Value>(code) {
            Ok(_) => Ok(true),
            Err(e) => Err(ProcessingError::new(
                "syntax_error",
                &format!("Invalid JSON: {}", e),
            )),
        }
    }
    
    fn format_json(&self, code: &str) -> Result<String, ProcessingError> {
        let value: serde_json::Value = serde_json::from_str(code).map_err(|e| {
            ProcessingError::new(
                "parse_error",
                &format!("Failed to parse JSON: {}", e),
            )
        })?;
        
        let formatted = if self.config.indent_size > 0 {
            serde_json::to_string_pretty(&value)
        } else {
            serde_json::to_string(&value)
        }.map_err(|e| {
            ProcessingError::new(
                "format_error",
                &format!("Failed to format JSON: {}", e),
            )
        })?;
        
        Ok(formatted)
    }
}

impl CodeBlockStrategy for JsonStrategy {
    fn process(&self, code: &str, config: &ProcessingConfig) -> Result<ProcessedCodeBlock, ProcessingError> {
        let start_time = Instant::now();
        let mut errors = Vec::new();
        let mut warnings = Vec::new();
        
        // Validate JSON if enabled
        let syntax_valid = if self.config.enable_validation && config.enable_syntax_validation {
            match self.validate_json(code) {
                Ok(valid) => valid,
                Err(e) => {
                    errors.push(e);
                    false
                }
            }
        } else {
            true // Skip validation
        };
        
        // Format JSON if enabled and valid
        let formatted_code = if self.config.enable_formatting && config.enable_formatting && syntax_valid {
            match self.format_json(code) {
                Ok(formatted) => Some(formatted),
                Err(e) => {
                    warnings.push(crate::markdown::code_block::ProcessingWarning::new(
                        "formatting_failed",
                        &e.message,
                    ));
                    None
                }
            }
        } else {
            None
        };
        
        let mut metadata = ProcessingMetadata::new(self.get_version())
            .with_processing_time(start_time.elapsed());
        
        metadata.is_validated = self.config.enable_validation && config.enable_syntax_validation;
        metadata.is_formatted = formatted_code.is_some();
        metadata.syntax_valid = syntax_valid;
        
        let mut processed = ProcessedCodeBlock::new(code.to_string(), Some("json".to_string()))
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
        matches!(language.to_lowercase().as_str(), "json" | "jsonc")
    }
    
    fn get_language_name(&self) -> &'static str {
        "json"
    }
    
    fn get_priority(&self) -> u8 {
        100
    }
    
    fn get_version(&self) -> &'static str {
        "1.0.0"
    }
    
    fn get_description(&self) -> &'static str {
        "JSON processing strategy with validation and formatting"
    }
}

/// Example plugin that provides basic XML processing
#[derive(Debug)]
pub struct XmlPlugin;

impl XmlPlugin {
    pub fn new() -> Self {
        Self
    }
}

impl CodeBlockPlugin for XmlPlugin {
    fn name(&self) -> &str {
        "xml-processor"
    }
    
    fn version(&self) -> &str {
        "1.0.0"
    }
    
    fn description(&self) -> &str {
        "Basic XML code block processor"
    }
    
    fn author(&self) -> &str {
        "Code Block Plugin System"
    }
    
    fn create_strategy(&self) -> Result<Box<dyn CodeBlockStrategy>, PluginError> {
        Ok(Box::new(XmlStrategy::new()))
    }
    
    fn supported_languages(&self) -> Vec<String> {
        vec!["xml".to_string(), "html".to_string(), "xhtml".to_string()]
    }
}

/// Simple XML processing strategy
#[derive(Debug, Clone)]
struct XmlStrategy;

impl XmlStrategy {
    fn new() -> Self {
        Self
    }
}

impl CodeBlockStrategy for XmlStrategy {
    fn process(&self, code: &str, _config: &ProcessingConfig) -> Result<ProcessedCodeBlock, ProcessingError> {
        let start_time = Instant::now();
        
        // Basic XML validation - just check for balanced tags
        let syntax_valid = self.basic_xml_validation(code);
        
        let mut metadata = ProcessingMetadata::new(self.get_version())
            .with_processing_time(start_time.elapsed());
        
        metadata.syntax_valid = syntax_valid;
        
        let processed = ProcessedCodeBlock::new(code.to_string(), Some("xml".to_string()))
            .with_metadata(metadata);
        
        Ok(processed)
    }
    
    fn supports_language(&self, language: &str) -> bool {
        matches!(language.to_lowercase().as_str(), "xml" | "html" | "xhtml")
    }
    
    fn get_language_name(&self) -> &'static str {
        "xml"
    }
    
    fn get_description(&self) -> &'static str {
        "Basic XML processing strategy"
    }
}

impl XmlStrategy {
    fn basic_xml_validation(&self, code: &str) -> bool {
        // Very basic validation - just check that opening and closing tags match
        let mut tag_stack = Vec::new();
        let mut in_tag = false;
        let mut current_tag = String::new();
        
        for ch in code.chars() {
            match ch {
                '<' => {
                    in_tag = true;
                    current_tag.clear();
                }
                '>' => {
                    if in_tag {
                        in_tag = false;
                        if !current_tag.is_empty() {
                            if current_tag.starts_with('/') {
                                // Closing tag
                                let tag_name = &current_tag[1..];
                                if let Some(last_tag) = tag_stack.pop() {
                                    if last_tag != tag_name {
                                        return false; // Mismatched tags
                                    }
                                } else {
                                    return false; // Closing tag without opening
                                }
                            } else if !current_tag.ends_with('/') {
                                // Opening tag (not self-closing)
                                let tag_name = current_tag.split_whitespace().next().unwrap_or("");
                                if !tag_name.is_empty() {
                                    tag_stack.push(tag_name.to_string());
                                }
                            }
                        }
                    }
                }
                _ => {
                    if in_tag {
                        current_tag.push(ch);
                    }
                }
            }
        }
        
        tag_stack.is_empty() // All tags should be closed
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_json_plugin_creation() {
        let plugin = JsonPlugin::new();
        assert_eq!(plugin.name(), "json-processor");
        assert_eq!(plugin.version(), "1.0.0");
        assert!(!plugin.supported_languages().is_empty());
    }

    #[test]
    fn test_json_plugin_strategy_creation() {
        let plugin = JsonPlugin::new();
        let strategy = plugin.create_strategy().unwrap();
        assert_eq!(strategy.get_language_name(), "json");
        assert!(strategy.supports_language("json"));
        assert!(strategy.supports_language("jsonc"));
    }

    #[test]
    fn test_json_plugin_configuration() {
        let mut plugin = JsonPlugin::new();
        let mut config = HashMap::new();
        config.insert("enable_formatting".to_string(), "false".to_string());
        config.insert("indent_size".to_string(), "4".to_string());
        
        plugin.configure(&config).unwrap();
        assert!(!plugin.config.enable_formatting);
        assert_eq!(plugin.config.indent_size, 4);
    }

    #[test]
    fn test_xml_plugin_creation() {
        let plugin = XmlPlugin::new();
        assert_eq!(plugin.name(), "xml-processor");
        assert!(plugin.supported_languages().contains(&"xml".to_string()));
        assert!(plugin.supported_languages().contains(&"html".to_string()));
    }

    #[test]
    fn test_xml_strategy_basic_validation() {
        let strategy = XmlStrategy::new();
        
        // Valid XML
        assert!(strategy.basic_xml_validation("<root><child>content</child></root>"));
        assert!(strategy.basic_xml_validation("<self-closing/>"));
        
        // Invalid XML
        assert!(!strategy.basic_xml_validation("<root><child>content</root>"));
        assert!(!strategy.basic_xml_validation("<root><child>content</child>"));
    }
}