//! Configuration structures for code block processing

use std::collections::HashMap;
use std::time::Duration;
use std::path::Path;
use std::fs;
use serde::{Deserialize, Serialize};

use crate::markdown::code_block::error::ProcessingError;

/// Configuration for code block processing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessingConfig {
    pub enable_syntax_validation: bool,
    pub enable_formatting: bool,
    pub enable_optimization: bool,
    pub timeout_ms: u64,
    pub custom_options: HashMap<String, String>,
}

/// Global configuration for the code block system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeBlockConfig {
    pub global: GlobalConfig,
    pub languages: HashMap<String, LanguageConfig>,
}

/// Global configuration options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalConfig {
    pub enable_processing: bool,
    pub default_timeout_ms: u64,
    pub max_cache_size: usize,
    pub enable_parallel_processing: bool,
}

/// Language-specific configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanguageConfig {
    pub enable_syntax_validation: bool,
    pub enable_formatting: bool,
    pub formatter_options: HashMap<String, String>,
    pub custom_options: HashMap<String, String>,
}

impl Default for ProcessingConfig {
    fn default() -> Self {
        Self {
            enable_syntax_validation: true,
            enable_formatting: false,
            enable_optimization: false,
            timeout_ms: 5000, // 5 seconds default timeout
            custom_options: HashMap::new(),
        }
    }
}

impl Default for GlobalConfig {
    fn default() -> Self {
        Self {
            enable_processing: true,
            default_timeout_ms: 5000,
            max_cache_size: 1000,
            enable_parallel_processing: false,
        }
    }
}

impl Default for LanguageConfig {
    fn default() -> Self {
        Self {
            enable_syntax_validation: true,
            enable_formatting: false,
            formatter_options: HashMap::new(),
            custom_options: HashMap::new(),
        }
    }
}

impl Default for CodeBlockConfig {
    fn default() -> Self {
        Self {
            global: GlobalConfig::default(),
            languages: HashMap::new(),
        }
    }
}

impl ProcessingConfig {
    /// Create a new processing configuration
    pub fn new() -> Self {
        Self::default()
    }

    /// Enable syntax validation
    pub fn with_syntax_validation(mut self, enable: bool) -> Self {
        self.enable_syntax_validation = enable;
        self
    }

    /// Enable formatting
    pub fn with_formatting(mut self, enable: bool) -> Self {
        self.enable_formatting = enable;
        self
    }

    /// Set timeout in milliseconds
    pub fn with_timeout_ms(mut self, timeout_ms: u64) -> Self {
        self.timeout_ms = timeout_ms;
        self
    }

    /// Add a custom option
    pub fn with_custom_option(mut self, key: &str, value: &str) -> Self {
        self.custom_options.insert(key.to_string(), value.to_string());
        self
    }

    /// Get timeout as Duration
    pub fn timeout_duration(&self) -> Duration {
        Duration::from_millis(self.timeout_ms)
    }

    /// Check if a custom option exists
    pub fn has_custom_option(&self, key: &str) -> bool {
        self.custom_options.contains_key(key)
    }

    /// Get a custom option value
    pub fn get_custom_option(&self, key: &str) -> Option<&String> {
        self.custom_options.get(key)
    }
}

impl CodeBlockConfig {
    /// Create a new code block configuration
    pub fn new() -> Self {
        Self::default()
    }

    /// Load configuration from a YAML file
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, ProcessingError> {
        let content = fs::read_to_string(path.as_ref())
            .map_err(|e| ProcessingError::new(
                "configuration_error",
                &format!("Failed to read config file: {}", e),
            ))?;

        Self::from_yaml(&content)
    }

    /// Load configuration from YAML string
    pub fn from_yaml(yaml_content: &str) -> Result<Self, ProcessingError> {
        let mut config: CodeBlockConfig = serde_yaml::from_str(yaml_content)
            .map_err(|e| ProcessingError::new(
                "configuration_error",
                &format!("Failed to parse YAML config: {}", e),
            ))?;

        config.validate_and_normalize()?;
        Ok(config)
    }

    /// Load configuration from JSON string
    pub fn from_json(json_content: &str) -> Result<Self, ProcessingError> {
        let mut config: CodeBlockConfig = serde_json::from_str(json_content)
            .map_err(|e| ProcessingError::new(
                "configuration_error",
                &format!("Failed to parse JSON config: {}", e),
            ))?;

        config.validate_and_normalize()?;
        Ok(config)
    }

    /// Save configuration to a YAML file
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), ProcessingError> {
        let yaml_content = self.to_yaml()?;
        fs::write(path.as_ref(), yaml_content)
            .map_err(|e| ProcessingError::new(
                "configuration_error",
                &format!("Failed to write config file: {}", e),
            ))
    }

    /// Convert configuration to YAML string
    pub fn to_yaml(&self) -> Result<String, ProcessingError> {
        serde_yaml::to_string(self)
            .map_err(|e| ProcessingError::new(
                "configuration_error",
                &format!("Failed to serialize config to YAML: {}", e),
            ))
    }

    /// Convert configuration to JSON string
    pub fn to_json(&self) -> Result<String, ProcessingError> {
        serde_json::to_string_pretty(self)
            .map_err(|e| ProcessingError::new(
                "configuration_error",
                &format!("Failed to serialize config to JSON: {}", e),
            ))
    }

    /// Validate and normalize the configuration
    pub fn validate_and_normalize(&mut self) -> Result<(), ProcessingError> {
        // Validate global configuration
        self.validate_global_config()?;

        // Normalize language names to lowercase and validate language configs
        let mut normalized_languages = HashMap::new();
        let languages_to_process: Vec<_> = self.languages.drain().collect();
        for (lang, config) in languages_to_process {
            let normalized_lang = lang.to_lowercase();
            let mut validated_config = config;
            Self::validate_language_config_static(&normalized_lang, &mut validated_config)?;
            normalized_languages.insert(normalized_lang, validated_config);
        }
        self.languages = normalized_languages;

        Ok(())
    }

    /// Validate global configuration
    fn validate_global_config(&mut self) -> Result<(), ProcessingError> {
        // Ensure timeout is reasonable (between 100ms and 5 minutes)
        if self.global.default_timeout_ms < 100 {
            self.global.default_timeout_ms = 100;
        } else if self.global.default_timeout_ms > 300_000 {
            self.global.default_timeout_ms = 300_000;
        }

        // Ensure cache size is reasonable (between 10 and 100,000)
        if self.global.max_cache_size < 10 {
            self.global.max_cache_size = 10;
        } else if self.global.max_cache_size > 100_000 {
            self.global.max_cache_size = 100_000;
        }

        Ok(())
    }

    /// Validate language-specific configuration
    fn validate_language_config_static(language: &str, config: &mut LanguageConfig) -> Result<(), ProcessingError> {
        // Validate known languages
        let known_languages = [
            "rust", "rs", "javascript", "js", "typescript", "ts", "python", "py",
            "java", "c", "cpp", "c++", "csharp", "cs", "go", "php", "ruby", "rb",
            "swift", "kotlin", "scala", "clojure", "haskell", "erlang", "elixir",
            "sql", "json", "yaml", "yml", "xml", "html", "css", "scss", "sass",
            "markdown", "md", "bash", "sh", "powershell", "ps1", "dockerfile",
        ];

        if !known_languages.contains(&language) {
            // For unknown languages, disable advanced features by default
            if config.enable_syntax_validation {
                config.enable_syntax_validation = false;
            }
            if config.enable_formatting {
                config.enable_formatting = false;
            }
        }

        Ok(())
    }

    /// Merge with another configuration (other takes precedence)
    pub fn merge_with(&mut self, other: &CodeBlockConfig) {
        // Merge global config
        if other.global.enable_processing != GlobalConfig::default().enable_processing {
            self.global.enable_processing = other.global.enable_processing;
        }
        if other.global.default_timeout_ms != GlobalConfig::default().default_timeout_ms {
            self.global.default_timeout_ms = other.global.default_timeout_ms;
        }
        if other.global.max_cache_size != GlobalConfig::default().max_cache_size {
            self.global.max_cache_size = other.global.max_cache_size;
        }
        if other.global.enable_parallel_processing != GlobalConfig::default().enable_parallel_processing {
            self.global.enable_parallel_processing = other.global.enable_parallel_processing;
        }

        // Merge language configs
        for (lang, config) in &other.languages {
            self.languages.insert(lang.clone(), config.clone());
        }
    }

    /// Add language-specific configuration
    pub fn with_language_config(mut self, language: &str, config: LanguageConfig) -> Self {
        self.languages.insert(language.to_lowercase(), config);
        self
    }

    /// Get configuration for a specific language
    pub fn get_language_config(&self, language: &str) -> LanguageConfig {
        self.languages.get(&language.to_lowercase())
            .cloned()
            .unwrap_or_default()
    }

    /// Create processing config for a specific language
    pub fn create_processing_config(&self, language: Option<&str>) -> ProcessingConfig {
        let lang_config = if let Some(lang) = language {
            self.get_language_config(lang)
        } else {
            LanguageConfig::default()
        };

        ProcessingConfig {
            enable_syntax_validation: lang_config.enable_syntax_validation,
            enable_formatting: lang_config.enable_formatting,
            enable_optimization: false,
            timeout_ms: self.global.default_timeout_ms,
            custom_options: lang_config.custom_options.clone(),
        }
    }

    /// Check if processing is enabled globally
    pub fn is_processing_enabled(&self) -> bool {
        self.global.enable_processing
    }

    /// Get all configured languages
    pub fn get_configured_languages(&self) -> Vec<String> {
        self.languages.keys().cloned().collect()
    }

    /// Check if a language has specific configuration
    pub fn has_language_config(&self, language: &str) -> bool {
        self.languages.contains_key(&language.to_lowercase())
    }

    /// Remove language configuration
    pub fn remove_language_config(&mut self, language: &str) -> Option<LanguageConfig> {
        self.languages.remove(&language.to_lowercase())
    }

    /// Create a default configuration with common language settings
    pub fn with_common_languages() -> Self {
        let mut config = Self::default();

        // Rust configuration
        config = config.with_language_config("rust", 
            LanguageConfig::new()
                .with_syntax_validation(true)
                .with_formatting(true)
                .with_formatter_option("edition", "2021")
        );

        // JavaScript/TypeScript configuration
        config = config.with_language_config("javascript", 
            LanguageConfig::new()
                .with_syntax_validation(true)
                .with_formatting(true)
                .with_formatter_option("semicolons", "true")
        );

        config = config.with_language_config("typescript", 
            LanguageConfig::new()
                .with_syntax_validation(true)
                .with_formatting(true)
                .with_formatter_option("semicolons", "true")
        );

        // Python configuration
        config = config.with_language_config("python", 
            LanguageConfig::new()
                .with_syntax_validation(true)
                .with_formatting(true)
                .with_formatter_option("line_length", "88")
        );

        // JSON configuration
        config = config.with_language_config("json", 
            LanguageConfig::new()
                .with_syntax_validation(true)
                .with_formatting(true)
                .with_formatter_option("indent_size", "2")
        );

        config
    }
}

impl LanguageConfig {
    /// Create a new language configuration
    pub fn new() -> Self {
        Self::default()
    }

    /// Enable syntax validation for this language
    pub fn with_syntax_validation(mut self, enable: bool) -> Self {
        self.enable_syntax_validation = enable;
        self
    }

    /// Enable formatting for this language
    pub fn with_formatting(mut self, enable: bool) -> Self {
        self.enable_formatting = enable;
        self
    }

    /// Add a formatter option
    pub fn with_formatter_option(mut self, key: &str, value: &str) -> Self {
        self.formatter_options.insert(key.to_string(), value.to_string());
        self
    }

    /// Add a custom option
    pub fn with_custom_option(mut self, key: &str, value: &str) -> Self {
        self.custom_options.insert(key.to_string(), value.to_string());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_processing_config_default() {
        let config = ProcessingConfig::default();
        assert!(config.enable_syntax_validation);
        assert!(!config.enable_formatting);
        assert_eq!(config.timeout_ms, 5000);
        assert!(config.custom_options.is_empty());
    }

    #[test]
    fn test_processing_config_builder() {
        let config = ProcessingConfig::new()
            .with_syntax_validation(false)
            .with_formatting(true)
            .with_timeout_ms(10000)
            .with_custom_option("indent_size", "4");

        assert!(!config.enable_syntax_validation);
        assert!(config.enable_formatting);
        assert_eq!(config.timeout_ms, 10000);
        assert_eq!(config.get_custom_option("indent_size"), Some(&"4".to_string()));
    }

    #[test]
    fn test_timeout_duration() {
        let config = ProcessingConfig::new().with_timeout_ms(3000);
        assert_eq!(config.timeout_duration(), Duration::from_millis(3000));
    }

    #[test]
    fn test_code_block_config() {
        let rust_config = LanguageConfig::new()
            .with_syntax_validation(true)
            .with_formatting(true)
            .with_formatter_option("edition", "2021");

        let config = CodeBlockConfig::new()
            .with_language_config("rust", rust_config);

        let rust_lang_config = config.get_language_config("rust");
        assert!(rust_lang_config.enable_syntax_validation);
        assert!(rust_lang_config.enable_formatting);
        assert_eq!(rust_lang_config.formatter_options.get("edition"), Some(&"2021".to_string()));
    }

    #[test]
    fn test_create_processing_config() {
        let rust_config = LanguageConfig::new()
            .with_syntax_validation(false)
            .with_formatting(true);

        let config = CodeBlockConfig::new()
            .with_language_config("rust", rust_config);

        let processing_config = config.create_processing_config(Some("rust"));
        assert!(!processing_config.enable_syntax_validation);
        assert!(processing_config.enable_formatting);
    }

    #[test]
    fn test_language_config_case_insensitive() {
        let config = CodeBlockConfig::new()
            .with_language_config("RUST", LanguageConfig::new().with_formatting(true));

        let rust_config = config.get_language_config("rust");
        assert!(rust_config.enable_formatting);

        let rust_upper_config = config.get_language_config("RUST");
        assert!(rust_upper_config.enable_formatting);
    }

    #[test]
    fn test_global_config() {
        let mut config = CodeBlockConfig::new();
        config.global.enable_processing = false;
        
        assert!(!config.is_processing_enabled());
    }

    #[test]
    fn test_yaml_serialization() {
        let config = CodeBlockConfig::with_common_languages();
        let yaml = config.to_yaml().expect("Should serialize to YAML");
        
        assert!(yaml.contains("global:"));
        assert!(yaml.contains("languages:"));
        assert!(yaml.contains("rust:"));
        assert!(yaml.contains("javascript:"));
    }

    #[test]
    fn test_json_serialization() {
        let config = CodeBlockConfig::with_common_languages();
        let json = config.to_json().expect("Should serialize to JSON");
        
        assert!(json.contains("\"global\""));
        assert!(json.contains("\"languages\""));
        assert!(json.contains("\"rust\""));
        assert!(json.contains("\"javascript\""));
    }

    #[test]
    fn test_yaml_deserialization() {
        let yaml_content = r#"
global:
  enable_processing: true
  default_timeout_ms: 3000
  max_cache_size: 500
  enable_parallel_processing: false
languages:
  rust:
    enable_syntax_validation: true
    enable_formatting: true
    formatter_options:
      edition: "2021"
    custom_options: {}
  python:
    enable_syntax_validation: false
    enable_formatting: true
    formatter_options:
      line_length: "100"
    custom_options: {}
"#;

        let config = CodeBlockConfig::from_yaml(yaml_content).expect("Should parse YAML");
        
        assert!(config.global.enable_processing);
        assert_eq!(config.global.default_timeout_ms, 3000);
        assert_eq!(config.global.max_cache_size, 500);
        
        let rust_config = config.get_language_config("rust");
        assert!(rust_config.enable_syntax_validation);
        assert!(rust_config.enable_formatting);
        assert_eq!(rust_config.formatter_options.get("edition"), Some(&"2021".to_string()));
        
        let python_config = config.get_language_config("python");
        assert!(!python_config.enable_syntax_validation);
        assert!(python_config.enable_formatting);
        assert_eq!(python_config.formatter_options.get("line_length"), Some(&"100".to_string()));
    }

    #[test]
    fn test_json_deserialization() {
        let json_content = r#"
{
  "global": {
    "enable_processing": true,
    "default_timeout_ms": 2000,
    "max_cache_size": 200,
    "enable_parallel_processing": true
  },
  "languages": {
    "javascript": {
      "enable_syntax_validation": true,
      "enable_formatting": false,
      "formatter_options": {
        "semicolons": "false"
      },
      "custom_options": {}
    }
  }
}
"#;

        let config = CodeBlockConfig::from_json(json_content).expect("Should parse JSON");
        
        assert!(config.global.enable_processing);
        assert_eq!(config.global.default_timeout_ms, 2000);
        assert_eq!(config.global.max_cache_size, 200);
        assert!(config.global.enable_parallel_processing);
        
        let js_config = config.get_language_config("javascript");
        assert!(js_config.enable_syntax_validation);
        assert!(!js_config.enable_formatting);
        assert_eq!(js_config.formatter_options.get("semicolons"), Some(&"false".to_string()));
    }

    #[test]
    fn test_file_operations() {
        let config = CodeBlockConfig::with_common_languages();
        
        // Test saving to file
        let temp_file = NamedTempFile::new().expect("Should create temp file");
        config.save_to_file(temp_file.path()).expect("Should save to file");
        
        // Test loading from file
        let loaded_config = CodeBlockConfig::from_file(temp_file.path()).expect("Should load from file");
        
        assert_eq!(config.global.enable_processing, loaded_config.global.enable_processing);
        assert_eq!(config.global.default_timeout_ms, loaded_config.global.default_timeout_ms);
        assert_eq!(config.languages.len(), loaded_config.languages.len());
    }

    #[test]
    fn test_validation_and_normalization() {
        let yaml_content = r#"
global:
  enable_processing: true
  default_timeout_ms: 50  # Too low, should be normalized to 100
  max_cache_size: 5       # Too low, should be normalized to 10
  enable_parallel_processing: false
languages:
  RUST:  # Should be normalized to lowercase
    enable_syntax_validation: true
    enable_formatting: true
    formatter_options: {}
    custom_options: {}
  unknown_language:  # Should have advanced features disabled
    enable_syntax_validation: true
    enable_formatting: true
    formatter_options: {}
    custom_options: {}
"#;

        let config = CodeBlockConfig::from_yaml(yaml_content).expect("Should parse and validate YAML");
        
        // Check global config normalization
        assert_eq!(config.global.default_timeout_ms, 100);
        assert_eq!(config.global.max_cache_size, 10);
        
        // Check language normalization - both should work since we normalize to lowercase
        assert!(config.has_language_config("rust"));
        assert!(config.has_language_config("RUST")); // This should also work due to case-insensitive lookup
        
        // Check unknown language validation
        let unknown_config = config.get_language_config("unknown_language");
        assert!(!unknown_config.enable_syntax_validation);
        assert!(!unknown_config.enable_formatting);
    }

    #[test]
    fn test_config_merging() {
        let mut base_config = CodeBlockConfig::new();
        base_config.global.default_timeout_ms = 1000;
        base_config = base_config.with_language_config("rust", 
            LanguageConfig::new().with_syntax_validation(false)
        );

        let override_config = CodeBlockConfig::new()
            .with_language_config("rust", 
                LanguageConfig::new().with_formatting(true)
            )
            .with_language_config("python", 
                LanguageConfig::new().with_syntax_validation(true)
            );

        base_config.merge_with(&override_config);

        // Rust config should be completely overridden
        let rust_config = base_config.get_language_config("rust");
        assert!(rust_config.enable_formatting);

        // Python config should be added
        assert!(base_config.has_language_config("python"));
        let python_config = base_config.get_language_config("python");
        assert!(python_config.enable_syntax_validation);
    }

    #[test]
    fn test_common_languages_config() {
        let config = CodeBlockConfig::with_common_languages();
        
        // Check that common languages are configured
        assert!(config.has_language_config("rust"));
        assert!(config.has_language_config("javascript"));
        assert!(config.has_language_config("typescript"));
        assert!(config.has_language_config("python"));
        assert!(config.has_language_config("json"));
        
        // Check specific configurations
        let rust_config = config.get_language_config("rust");
        assert!(rust_config.enable_syntax_validation);
        assert!(rust_config.enable_formatting);
        assert_eq!(rust_config.formatter_options.get("edition"), Some(&"2021".to_string()));
        
        let python_config = config.get_language_config("python");
        assert!(python_config.enable_syntax_validation);
        assert!(python_config.enable_formatting);
        assert_eq!(python_config.formatter_options.get("line_length"), Some(&"88".to_string()));
    }

    #[test]
    fn test_config_utilities() {
        let mut config = CodeBlockConfig::with_common_languages();
        
        // Test getting configured languages
        let languages = config.get_configured_languages();
        assert!(languages.contains(&"rust".to_string()));
        assert!(languages.contains(&"javascript".to_string()));
        
        // Test removing language config
        let removed = config.remove_language_config("rust");
        assert!(removed.is_some());
        assert!(!config.has_language_config("rust"));
        
        // Test that removed language falls back to default
        let default_rust_config = config.get_language_config("rust");
        assert_eq!(default_rust_config.enable_syntax_validation, LanguageConfig::default().enable_syntax_validation);
    }

    #[test]
    fn test_invalid_yaml_handling() {
        let invalid_yaml = "invalid: yaml: content: [";
        let result = CodeBlockConfig::from_yaml(invalid_yaml);
        assert!(result.is_err());
        
        if let Err(error) = result {
            assert!(error.message.contains("Failed to parse YAML config"));
        }
    }

    #[test]
    fn test_invalid_json_handling() {
        let invalid_json = r#"{"invalid": json content"#;
        let result = CodeBlockConfig::from_json(invalid_json);
        assert!(result.is_err());
        
        if let Err(error) = result {
            assert!(error.message.contains("Failed to parse JSON config"));
        }
    }

    #[test]
    fn test_extreme_timeout_values() {
        let yaml_with_extreme_values = r#"
global:
  enable_processing: true
  default_timeout_ms: 500000  # Too high, should be capped at 300000
  max_cache_size: 200000      # Too high, should be capped at 100000
  enable_parallel_processing: false
languages: {}
"#;

        let config = CodeBlockConfig::from_yaml(yaml_with_extreme_values).expect("Should parse YAML");
        
        assert_eq!(config.global.default_timeout_ms, 300_000);
        assert_eq!(config.global.max_cache_size, 100_000);
    }
}