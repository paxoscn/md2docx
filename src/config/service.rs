//! Configuration service for managing YAML configs and LLM integration

use crate::config::{ConversionConfig, YamlProcessor};
use crate::error::ConfigError;
use crate::llm::LlmClient;

/// Service for managing configuration operations
pub struct ConfigurationService {
    llm_client: LlmClient,
    yaml_processor: YamlProcessor,
}

impl ConfigurationService {
    /// Create a new configuration service with default LLM client
    pub fn new() -> Self {
        // Create a default LLM client - this will be configured from environment variables
        let llm_client = LlmClient::default();
        
        Self {
            llm_client,
            yaml_processor: YamlProcessor::new(),
        }
    }
    
    /// Create a new configuration service with custom LLM client
    pub fn with_llm_client(llm_client: LlmClient) -> Self {
        Self {
            llm_client,
            yaml_processor: YamlProcessor::new(),
        }
    }

    /// Parse YAML configuration from string
    pub async fn parse_config(&self, yaml: &str) -> Result<ConversionConfig, ConfigError> {
        self.yaml_processor.parse(yaml).await
    }

    /// Update configuration using natural language description
    pub async fn update_with_natural_language(
        &self,
        config: &ConversionConfig,
        prompt: &str,
    ) -> Result<ConversionConfig, ConfigError> {
        // Convert current config to YAML string
        let current_yaml = self.yaml_processor.serialize(config)?;
        
        // Use LLM to generate updated configuration
        let updated_yaml = self.llm_client
            .generate_and_validate_config_update(&current_yaml, prompt)
            .await?;
        
        // Parse the updated YAML back to ConversionConfig
        let updated_config = self.yaml_processor.parse(&updated_yaml).await?;
        
        // Validate the updated configuration
        self.validate_config(&updated_config)?;
        
        Ok(updated_config)
    }

    /// Preview configuration changes without applying them
    pub async fn preview_config_update(
        &self,
        config: &ConversionConfig,
        prompt: &str,
    ) -> Result<String, ConfigError> {
        let current_yaml = self.yaml_processor.serialize(config)?;
        
        // Generate updated configuration YAML
        let updated_yaml = self.llm_client
            .generate_config_update(&current_yaml, prompt)
            .await?;
        
        Ok(updated_yaml)
    }

    /// Apply configuration update from YAML string
    pub async fn apply_config_from_yaml(&self, yaml: &str) -> Result<ConversionConfig, ConfigError> {
        // Parse and validate the YAML
        let config = self.yaml_processor.parse(yaml).await?;
        self.validate_config(&config)?;
        
        Ok(config)
    }

    /// Validate configuration structure and values
    pub fn validate_config(&self, config: &ConversionConfig) -> Result<(), ConfigError> {
        self.yaml_processor.validate(config)
    }

    /// Get default configuration
    pub fn default_config(&self) -> ConversionConfig {
        ConversionConfig::default()
    }
}

/// Trait defining configuration management interface
pub trait ConfigManager {
    async fn parse_config(&self, yaml: &str) -> Result<ConversionConfig, ConfigError>;
    async fn update_with_natural_language(
        &self,
        config: &ConversionConfig,
        prompt: &str,
    ) -> Result<ConversionConfig, ConfigError>;
    fn validate_config(&self, config: &ConversionConfig) -> Result<(), ConfigError>;
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::llm::{LlmConfig, LlmProvider};

    fn create_test_service() -> ConfigurationService {
        ConfigurationService::new()
    }

    #[tokio::test]
    async fn test_parse_config() {
        let service = create_test_service();
        let default_config = ConversionConfig::default();
        let yaml = service.yaml_processor.serialize(&default_config).unwrap();
        
        let result = service.parse_config(&yaml).await;
        assert!(result.is_ok());
        
        let parsed_config = result.unwrap();
        assert_eq!(parsed_config.document.default_font.family, default_config.document.default_font.family);
    }

    #[test]
    fn test_validate_config() {
        let service = create_test_service();
        let config = ConversionConfig::default();
        
        let result = service.validate_config(&config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_default_config() {
        let service = create_test_service();
        let config = service.default_config();
        
        // Should be able to validate the default config
        assert!(service.validate_config(&config).is_ok());
    }

    #[tokio::test]
    async fn test_apply_config_from_yaml() {
        let service = create_test_service();
        let default_config = ConversionConfig::default();
        let yaml = service.yaml_processor.serialize(&default_config).unwrap();
        
        let result = service.apply_config_from_yaml(&yaml).await;
        assert!(result.is_ok());
        
        let applied_config = result.unwrap();
        assert_eq!(applied_config.document.default_font.family, default_config.document.default_font.family);
    }

    #[tokio::test]
    async fn test_apply_invalid_yaml() {
        let service = create_test_service();
        let invalid_yaml = "invalid: yaml: content: [";
        
        let result = service.apply_config_from_yaml(invalid_yaml).await;
        assert!(result.is_err());
    }

    // Note: Tests for update_with_natural_language and preview_config_update
    // would require a working LLM endpoint, so they're not included here.
    // In a real testing environment, you would mock the LLM client.
}