//! LLM client for natural language configuration updates

use crate::error::ConfigError;
use reqwest::{Client, header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE}};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tracing::{debug, error, info, warn};

/// Supported LLM providers
#[derive(Debug, Clone)]
pub enum LlmProvider {
    OpenAI,
    Claude,
}

/// Configuration for LLM client
#[derive(Debug, Clone)]
pub struct LlmConfig {
    pub provider: LlmProvider,
    pub api_key: String,
    pub model: String,
    pub base_url: Option<String>,
    pub max_retries: u32,
    pub timeout_seconds: u64,
}

/// Client for interacting with LLM APIs
pub struct LlmClient {
    client: Client,
    config: LlmConfig,
}

/// Request structure for OpenAI API calls
#[derive(Debug, Serialize)]
struct OpenAIRequest {
    model: String,
    messages: Vec<Message>,
    temperature: f32,
    max_tokens: Option<u32>,
}

/// Request structure for Claude API calls
#[derive(Debug, Serialize)]
struct ClaudeRequest {
    model: String,
    messages: Vec<Message>,
    temperature: f32,
    max_tokens: u32,
}

/// Message structure for LLM conversations
#[derive(Debug, Clone, Serialize, Deserialize)]
struct Message {
    role: String,
    content: String,
}

/// Response structure from OpenAI API
#[derive(Debug, Deserialize)]
struct OpenAIResponse {
    choices: Vec<OpenAIChoice>,
}

/// Choice structure in OpenAI response
#[derive(Debug, Deserialize)]
struct OpenAIChoice {
    message: ResponseMessage,
}

/// Response structure from Claude API
#[derive(Debug, Deserialize)]
struct ClaudeResponse {
    content: Vec<ClaudeContent>,
}

/// Content structure in Claude response
#[derive(Debug, Deserialize)]
struct ClaudeContent {
    text: String,
}

/// Response message structure
#[derive(Debug, Deserialize)]
struct ResponseMessage {
    content: String,
}

impl Default for LlmConfig {
    fn default() -> Self {
        Self {
            provider: LlmProvider::OpenAI,
            api_key: String::new(),
            model: "gpt-3.5-turbo".to_string(),
            base_url: None,
            max_retries: 3,
            timeout_seconds: 30,
        }
    }
}

impl Default for LlmClient {
    fn default() -> Self {
        // Create a default client with environment-based configuration
        let api_key = std::env::var("OPENAI_API_KEY")
            .or_else(|_| std::env::var("ANTHROPIC_API_KEY"))
            .unwrap_or_else(|_| "dummy-key".to_string());
        
        let config = LlmConfig {
            api_key,
            ..Default::default()
        };
        
        // Use a dummy client if we can't create a real one
        Self::new(config).unwrap_or_else(|_| {
            let dummy_config = LlmConfig {
                api_key: "dummy-key".to_string(),
                ..Default::default()
            };
            
            let client = Client::builder()
                .timeout(Duration::from_secs(dummy_config.timeout_seconds))
                .build()
                .expect("Failed to create HTTP client");
            
            Self {
                client,
                config: dummy_config,
            }
        })
    }
}

impl LlmClient {
    /// Create a new LLM client with configuration
    pub fn new(config: LlmConfig) -> Result<Self, ConfigError> {
        if config.api_key.is_empty() {
            return Err(ConfigError::Validation("API key cannot be empty".to_string()));
        }

        let client = Client::builder()
            .timeout(Duration::from_secs(config.timeout_seconds))
            .build()
            .map_err(|e| ConfigError::LlmApi(format!("Failed to create HTTP client: {}", e)))?;

        Ok(Self { client, config })
    }

    /// Create a new LLM client with simple parameters (for backward compatibility)
    pub fn new_simple(api_key: String, base_url: String) -> Result<Self, ConfigError> {
        let config = LlmConfig {
            api_key,
            base_url: Some(base_url),
            ..Default::default()
        };
        Self::new(config)
    }

    /// Send a prompt to the LLM and get a response
    pub async fn send_prompt(&self, prompt: &str) -> Result<String, ConfigError> {
        let messages = vec![Message {
            role: "user".to_string(),
            content: prompt.to_string(),
        }];

        self.send_messages(&messages).await
    }

    /// Send messages to the LLM with system prompt support
    pub async fn send_messages(&self, messages: &[Message]) -> Result<String, ConfigError> {
        let mut attempt = 0;
        let mut last_error = None;

        while attempt < self.config.max_retries {
            attempt += 1;
            
            debug!("LLM API call attempt {} of {}", attempt, self.config.max_retries);

            match self.make_api_call(messages).await {
                Ok(response) => {
                    info!("LLM API call successful on attempt {}", attempt);
                    return Ok(response);
                }
                Err(e) => {
                    warn!("LLM API call failed on attempt {}: {}", attempt, e);
                    last_error = Some(e);
                    
                    if attempt < self.config.max_retries {
                        let delay = Duration::from_millis(1000 * attempt as u64);
                        debug!("Retrying in {:?}", delay);
                        tokio::time::sleep(delay).await;
                    }
                }
            }
        }

        Err(last_error.unwrap_or_else(|| {
            ConfigError::LlmApi("All retry attempts failed".to_string())
        }))
    }

    /// Make the actual API call based on provider
    async fn make_api_call(&self, messages: &[Message]) -> Result<String, ConfigError> {
        match self.config.provider {
            LlmProvider::OpenAI => self.call_openai_api(messages).await,
            LlmProvider::Claude => self.call_claude_api(messages).await,
        }
    }

    /// Call OpenAI API
    async fn call_openai_api(&self, messages: &[Message]) -> Result<String, ConfigError> {
        let url = self.config.base_url
            .as_deref()
            .unwrap_or("https://api.openai.com/v1/chat/completions");

        let request = OpenAIRequest {
            model: self.config.model.clone(),
            messages: messages.to_vec(),
            temperature: 0.1,
            max_tokens: Some(4000),
        };

        let mut headers = HeaderMap::new();
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", self.config.api_key))
                .map_err(|e| ConfigError::LlmApi(format!("Invalid API key format: {}", e)))?,
        );
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

        debug!("Sending request to OpenAI API: {}", url);

        let response = self
            .client
            .post(url)
            .headers(headers)
            .json(&request)
            .send()
            .await
            .map_err(|e| ConfigError::LlmApi(format!("HTTP request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            
            error!("OpenAI API error {}: {}", status, error_text);
            return Err(ConfigError::LlmApi(format!(
                "API request failed with status {}: {}",
                status, error_text
            )));
        }

        let api_response: OpenAIResponse = response
            .json()
            .await
            .map_err(|e| ConfigError::LlmApi(format!("Failed to parse response: {}", e)))?;

        api_response
            .choices
            .first()
            .map(|choice| choice.message.content.clone())
            .ok_or_else(|| ConfigError::LlmApi("No response choices available".to_string()))
    }

    /// Call Claude API
    async fn call_claude_api(&self, messages: &[Message]) -> Result<String, ConfigError> {
        let url = self.config.base_url
            .as_deref()
            .unwrap_or("https://api.anthropic.com/v1/messages");

        let request = ClaudeRequest {
            model: self.config.model.clone(),
            messages: messages.to_vec(),
            temperature: 0.1,
            max_tokens: 4000,
        };

        let mut headers = HeaderMap::new();
        headers.insert(
            "x-api-key",
            HeaderValue::from_str(&self.config.api_key)
                .map_err(|e| ConfigError::LlmApi(format!("Invalid API key format: {}", e)))?,
        );
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        headers.insert("anthropic-version", HeaderValue::from_static("2023-06-01"));

        debug!("Sending request to Claude API: {}", url);

        let response = self
            .client
            .post(url)
            .headers(headers)
            .json(&request)
            .send()
            .await
            .map_err(|e| ConfigError::LlmApi(format!("HTTP request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            
            error!("Claude API error {}: {}", status, error_text);
            return Err(ConfigError::LlmApi(format!(
                "API request failed with status {}: {}",
                status, error_text
            )));
        }

        let api_response: ClaudeResponse = response
            .json()
            .await
            .map_err(|e| ConfigError::LlmApi(format!("Failed to parse response: {}", e)))?;

        api_response
            .content
            .first()
            .map(|content| content.text.clone())
            .ok_or_else(|| ConfigError::LlmApi("No response content available".to_string()))
    }

    /// Generate configuration update based on natural language description
    pub async fn generate_config_update(
        &self,
        current_config: &str,
        update_description: &str,
    ) -> Result<String, ConfigError> {
        use crate::llm::prompts::{CONFIG_UPDATE_SYSTEM_PROMPT, create_config_update_prompt, extract_yaml_from_response};
        
        debug!("Generating config update for description: {}", update_description);
        
        // Create messages with system prompt and user request
        let messages = vec![
            Message {
                role: "system".to_string(),
                content: CONFIG_UPDATE_SYSTEM_PROMPT.to_string(),
            },
            Message {
                role: "user".to_string(),
                content: create_config_update_prompt(current_config, update_description),
            },
        ];

        // Send to LLM and get response
        let response = self.send_messages(&messages).await?;
        
        // Extract YAML from response (remove markdown code blocks if present)
        let updated_config = extract_yaml_from_response(&response);
        
        debug!("Generated updated config: {}", updated_config);
        
        Ok(updated_config)
    }

    /// Validate configuration using LLM
    pub async fn validate_config(&self, config: &str) -> Result<bool, ConfigError> {
        use crate::llm::prompts::{CONFIG_VALIDATION_PROMPT, create_config_validation_prompt};
        
        debug!("Validating configuration with LLM");
        
        let messages = vec![
            Message {
                role: "system".to_string(),
                content: CONFIG_VALIDATION_PROMPT.to_string(),
            },
            Message {
                role: "user".to_string(),
                content: create_config_validation_prompt(config),
            },
        ];

        let response = self.send_messages(&messages).await?;
        let is_valid = response.trim().to_uppercase().starts_with("VALID");
        
        if !is_valid {
            debug!("Configuration validation failed: {}", response);
        }
        
        Ok(is_valid)
    }

    /// Generate configuration update with validation
    pub async fn generate_and_validate_config_update(
        &self,
        current_config: &str,
        update_description: &str,
    ) -> Result<String, ConfigError> {
        // Generate the updated configuration
        let updated_config = self.generate_config_update(current_config, update_description).await?;
        
        // Validate the updated configuration
        let is_valid = self.validate_config(&updated_config).await?;
        
        if !is_valid {
            return Err(ConfigError::Validation(
                "Generated configuration failed validation".to_string()
            ));
        }
        
        Ok(updated_config)
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_llm_config_default() {
        let config = LlmConfig::default();
        assert_eq!(config.max_retries, 3);
        assert_eq!(config.timeout_seconds, 30);
        assert_eq!(config.model, "gpt-3.5-turbo");
        assert!(matches!(config.provider, LlmProvider::OpenAI));
    }

    #[test]
    fn test_llm_client_creation_with_empty_api_key() {
        let config = LlmConfig {
            api_key: String::new(),
            ..Default::default()
        };
        
        let result = LlmClient::new(config);
        assert!(result.is_err());
        
        if let Err(ConfigError::Validation(msg)) = result {
            assert!(msg.contains("API key cannot be empty"));
        } else {
            panic!("Expected validation error for empty API key");
        }
    }

    #[test]
    fn test_llm_client_creation_with_valid_config() {
        let config = LlmConfig {
            api_key: "test-key".to_string(),
            ..Default::default()
        };
        
        let result = LlmClient::new(config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_backward_compatibility_constructor() {
        let result = LlmClient::new_simple(
            "test-key".to_string(),
            "https://api.openai.com".to_string(),
        );
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_send_prompt_with_invalid_client() {
        // This test would require mocking the HTTP client
        // For now, we'll just test the basic structure
        let config = LlmConfig {
            api_key: "invalid-key".to_string(),
            base_url: Some("http://localhost:9999".to_string()), // Non-existent server
            max_retries: 1,
            timeout_seconds: 1,
            ..Default::default()
        };
        
        let client = LlmClient::new(config).unwrap();
        let result = client.send_prompt("test prompt").await;
        
        // Should fail due to connection error
        assert!(result.is_err());
    }
}