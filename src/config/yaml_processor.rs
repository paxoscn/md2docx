//! YAML processing utilities for configuration

use crate::config::ConversionConfig;
use crate::error::ConfigError;
use std::path::Path;
use tokio::fs;

/// Processor for YAML configuration files
pub struct YamlProcessor;

impl YamlProcessor {
    /// Create a new YAML processor
    pub fn new() -> Self {
        Self
    }

    /// Parse YAML string into configuration
    pub async fn parse(&self, yaml: &str) -> Result<ConversionConfig, ConfigError> {
        let config: ConversionConfig = serde_yaml::from_str(yaml)
            .map_err(|e| ConfigError::InvalidYaml(e.to_string()))?;
        
        // Validate the parsed configuration
        config.validate()
            .map_err(|e| ConfigError::Validation(e.to_string()))?;
        
        Ok(config)
    }

    /// Parse YAML file into configuration
    pub async fn parse_file<P: AsRef<Path>>(&self, path: P) -> Result<ConversionConfig, ConfigError> {
        let content = fs::read_to_string(path).await
            .map_err(|e| ConfigError::Io(e))?;
        
        self.parse(&content).await
    }

    /// Serialize configuration to YAML string
    pub fn serialize(&self, config: &ConversionConfig) -> Result<String, ConfigError> {
        // Validate before serializing
        config.validate()
            .map_err(|e| ConfigError::Validation(e.to_string()))?;
        
        serde_yaml::to_string(config)
            .map_err(|e| ConfigError::InvalidYaml(e.to_string()))
    }

    /// Write configuration to YAML file
    pub async fn write_file<P: AsRef<Path>>(&self, config: &ConversionConfig, path: P) -> Result<(), ConfigError> {
        let yaml_content = self.serialize(config)?;
        
        fs::write(path, yaml_content).await
            .map_err(|e| ConfigError::Io(e))?;
        
        Ok(())
    }

    /// Validate configuration structure and values
    pub fn validate(&self, config: &ConversionConfig) -> Result<(), ConfigError> {
        config.validate()
            .map_err(|e| ConfigError::Validation(e.to_string()))
    }

    /// Create a default configuration and save it to a file
    pub async fn create_default_config<P: AsRef<Path>>(&self, path: P) -> Result<ConversionConfig, ConfigError> {
        let config = ConversionConfig::default();
        self.write_file(&config, path).await?;
        Ok(config)
    }

    /// Merge two configurations, with the second one taking precedence
    pub fn merge_configs(&self, base: &ConversionConfig, override_config: &ConversionConfig) -> Result<ConversionConfig, ConfigError> {
        // For now, we'll do a simple override. In a more sophisticated implementation,
        // we could merge individual fields selectively
        let mut merged = base.clone();
        
        // Override document settings
        merged.document = override_config.document.clone();
        merged.styles = override_config.styles.clone();
        merged.elements = override_config.elements.clone();
        
        // Validate the merged configuration
        merged.validate()
            .map_err(|e| ConfigError::Validation(e.to_string()))?;
        
        Ok(merged)
    }
}

impl Default for YamlProcessor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[tokio::test]
    async fn test_parse_valid_yaml() {
        let processor = YamlProcessor::new();
        let default_config = ConversionConfig::default();
        
        // Use the serialized default config as our test YAML
        let yaml = processor.serialize(&default_config).unwrap();
        
        let result = processor.parse(&yaml).await;
        assert!(result.is_ok());
        
        let config = result.unwrap();
        assert_eq!(config.document.default_font.family, default_config.document.default_font.family);
        assert_eq!(config.document.default_font.size, default_config.document.default_font.size);
    }

    #[tokio::test]
    async fn test_parse_invalid_yaml() {
        let processor = YamlProcessor::new();
        // Create a YAML with invalid configuration (negative page size)
        let invalid_yaml = "document:\n  page_size:\n    width: -100.0\n    height: 842.0\n  margins:\n    top: 72.0\n    bottom: 72.0\n    left: 72.0\n    right: 72.0\n  default_font:\n    family: \"Arial\"\n    size: 12.0\n    bold: false\n    italic: false\nstyles:\n  headings: {}\n  paragraph:\n    font:\n      family: \"Arial\"\n      size: 12.0\n      bold: false\n      italic: false\n    line_spacing: 1.0\n    spacing_after: 0.0\n  code_block:\n    font:\n      family: \"Courier\"\n      size: 10.0\n      bold: false\n      italic: false\n    border_width: 0.0\n    preserve_line_breaks: true\n    line_spacing: 1.0\n    paragraph_spacing: 6.0\n  table:\n    header_font:\n      family: \"Arial\"\n      size: 12.0\n      bold: true\n      italic: false\n    cell_font:\n      family: \"Arial\"\n      size: 12.0\n      bold: false\n      italic: false\n    border_width: 1.0\nelements:\n  image:\n    max_width: 500.0\n    max_height: 400.0\n  list:\n    indent: 36.0\n    spacing: 6.0\n  link:\n    color: \"#0066cc\"\n    underline: true";

        let result = processor.parse(invalid_yaml).await;
        assert!(result.is_err());
        
        // Check the specific error type
        match result.unwrap_err() {
            ConfigError::Validation(_) => {}, // This is what we expect
            other => panic!("Expected ValidationError, got: {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_serialize_and_deserialize() {
        let processor = YamlProcessor::new();
        let config = ConversionConfig::default();
        
        let yaml = processor.serialize(&config).unwrap();
        let parsed_config = processor.parse(&yaml).await.unwrap();
        
        // Verify that serialization and deserialization preserve the data
        assert_eq!(config.document.default_font.family, parsed_config.document.default_font.family);
        assert_eq!(config.document.default_font.size, parsed_config.document.default_font.size);
    }

    #[tokio::test]
    async fn test_file_operations() {
        let processor = YamlProcessor::new();
        let config = ConversionConfig::default();
        
        // Create a temporary file
        let mut temp_file = NamedTempFile::new().unwrap();
        let temp_path = temp_file.path();
        
        // Write config to file
        processor.write_file(&config, temp_path).await.unwrap();
        
        // Read config from file
        let loaded_config = processor.parse_file(temp_path).await.unwrap();
        
        // Verify the loaded config matches the original
        assert_eq!(config.document.default_font.family, loaded_config.document.default_font.family);
        assert_eq!(config.document.default_font.size, loaded_config.document.default_font.size);
    }

    #[tokio::test]
    async fn test_create_default_config() {
        let processor = YamlProcessor::new();
        let temp_file = NamedTempFile::new().unwrap();
        let temp_path = temp_file.path();
        
        let config = processor.create_default_config(temp_path).await.unwrap();
        
        // Verify the file was created and contains valid config
        let loaded_config = processor.parse_file(temp_path).await.unwrap();
        assert_eq!(config.document.default_font.family, loaded_config.document.default_font.family);
    }

    #[tokio::test]
    async fn test_merge_configs() {
        let processor = YamlProcessor::new();
        let base_config = ConversionConfig::default();
        let mut override_config = ConversionConfig::default();
        
        // Modify the override config
        override_config.document.default_font.size = 14.0;
        override_config.document.default_font.family = "Arial".to_string();
        
        let merged = processor.merge_configs(&base_config, &override_config).unwrap();
        
        // Verify the override values are used
        assert_eq!(merged.document.default_font.size, 14.0);
        assert_eq!(merged.document.default_font.family, "Arial");
    }

    #[tokio::test]
    async fn test_validation() {
        let processor = YamlProcessor::new();
        let mut config = ConversionConfig::default();
        
        // Valid config should pass
        assert!(processor.validate(&config).is_ok());
        
        // Invalid config should fail
        config.document.default_font.size = -1.0;
        assert!(processor.validate(&config).is_err());
    }

    #[tokio::test]
    async fn test_malformed_yaml() {
        let processor = YamlProcessor::new();
        let malformed_yaml = "document:\n  page_size:\n    width: not_a_number";

        let result = processor.parse(malformed_yaml).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ConfigError::InvalidYaml(_)));
    }
}