//! HTTP request handlers

use crate::web::{api::AppState, task_queue::{create_conversion_task, TaskStatus}};
use axum::{
    extract::{Multipart, State, Path},
    http::{header, StatusCode},
    response::{Json, Response},
    Json as JsonExtractor,
};
use serde::{Deserialize, Serialize};

/// Health check response
#[derive(Serialize)]
pub struct HealthResponse {
    status: String,
    version: String,
}

/// Conversion request matching API specification
#[derive(Deserialize)]
pub struct ConvertRequest {
    pub markdown: String,
    pub config: Option<String>,
    pub natural_language: Option<String>,
}

/// Conversion response matching API specification
#[derive(Serialize)]
pub struct ConvertResponse {
    pub success: bool,
    pub file_data: Option<Vec<u8>>,
    pub error: Option<String>,
}

/// Configuration update request
#[derive(Deserialize)]
pub struct ConfigUpdateRequest {
    pub natural_language: String,
    pub current_config: Option<String>,
}

/// Configuration update response
#[derive(Serialize)]
pub struct ConfigUpdateResponse {
    pub success: bool,
    pub updated_config: Option<String>,
    pub error: Option<String>,
}

/// File upload response
#[derive(Serialize)]
pub struct FileUploadResponse {
    pub success: bool,
    pub message: String,
    pub file_id: Option<String>,
}

/// Async conversion request
#[derive(Deserialize)]
pub struct AsyncConvertRequest {
    pub markdown: String,
    pub config: Option<String>,
    pub filename: Option<String>,
}

/// Async conversion response
#[derive(Serialize)]
pub struct AsyncConvertResponse {
    pub success: bool,
    pub task_id: Option<String>,
    pub error: Option<String>,
}

/// Task status response
#[derive(Serialize)]
pub struct TaskStatusResponse {
    pub task_id: String,
    pub status: TaskStatus,
    pub completed: bool,
    pub error: Option<String>,
}

/// Health check handler
pub async fn health_check() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "healthy".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}

/// Convert Markdown to docx handler
pub async fn convert_markdown(
    State(app_state): State<AppState>,
    JsonExtractor(request): JsonExtractor<ConvertRequest>,
) -> Result<Json<ConvertResponse>, StatusCode> {
    let engine = &app_state.conversion_engine;
    tracing::info!("Received conversion request");
    
    // Handle natural language config update if provided
    if let Some(natural_language) = &request.natural_language {
        tracing::info!("Processing natural language config update: {}", natural_language);
        
        // For now, we'll use the existing engine - natural language processing will be implemented in task 7.3
        // This is a placeholder that acknowledges the parameter but doesn't process it yet
        tracing::warn!("Natural language config processing not yet implemented, using default config");
    }
    
    // Apply custom config if provided
    if let Some(config_yaml) = &request.config {
        tracing::info!("Applying custom YAML configuration");
        
        // Parse the YAML config
        match serde_yaml::from_str::<serde_json::Value>(config_yaml) {
            Ok(config_value) => {
                // For now, we'll log the config but not apply it
                // Full config application will be enhanced in future tasks
                tracing::info!("Received custom config: {:?}", config_value);
                tracing::warn!("Custom config application not fully implemented, using default config");
            }
            Err(e) => {
                tracing::error!("Failed to parse YAML config: {}", e);
                return Ok(Json(ConvertResponse {
                    success: false,
                    file_data: None,
                    error: Some(format!("Invalid YAML configuration: {}", e)),
                }));
            }
        }
    }
    
    // Perform the conversion
    match engine.convert(&request.markdown).await {
        Ok(docx_bytes) => {
            tracing::info!("Conversion successful, generated {} bytes", docx_bytes.len());
            Ok(Json(ConvertResponse {
                success: true,
                file_data: Some(docx_bytes),
                error: None,
            }))
        }
        Err(e) => {
            tracing::error!("Conversion failed: {}", e);
            Ok(Json(ConvertResponse {
                success: false,
                file_data: None,
                error: Some(format!("Conversion failed: {}", e)),
            }))
        }
    }
}

/// Update configuration with natural language handler
pub async fn update_config_natural(
    State(app_state): State<AppState>,
    JsonExtractor(request): JsonExtractor<ConfigUpdateRequest>,
) -> Result<Json<ConfigUpdateResponse>, StatusCode> {
    let engine = &app_state.conversion_engine;
    tracing::info!("Received natural language config update request");
    
    // Get current configuration
    let current_config = engine.config();
    
    // Create LLM client for configuration updates
    // For now, we'll use environment variables for API configuration
    let llm_config = match create_llm_config_from_env() {
        Ok(config) => config,
        Err(e) => {
            tracing::error!("Failed to create LLM config: {}", e);
            return Ok(Json(ConfigUpdateResponse {
                success: false,
                updated_config: None,
                error: Some(format!("LLM configuration error: {}", e)),
            }));
        }
    };
    
    let llm_client = match crate::llm::LlmClient::new(llm_config) {
        Ok(client) => client,
        Err(e) => {
            tracing::error!("Failed to create LLM client: {}", e);
            return Ok(Json(ConfigUpdateResponse {
                success: false,
                updated_config: None,
                error: Some(format!("Failed to initialize LLM client: {}", e)),
            }));
        }
    };
    
    // Create configuration service
    let config_service = crate::config::ConfigurationService::with_llm_client(llm_client);
    
    // Use current config or provided config as base
    let base_config = if let Some(current_yaml) = &request.current_config {
        match config_service.parse_config(current_yaml).await {
            Ok(config) => config,
            Err(e) => {
                tracing::error!("Failed to parse provided config: {}", e);
                return Ok(Json(ConfigUpdateResponse {
                    success: false,
                    updated_config: None,
                    error: Some(format!("Invalid current config: {}", e)),
                }));
            }
        }
    } else {
        current_config.clone()
    };
    
    // Generate updated configuration using natural language
    match config_service.update_with_natural_language(&base_config, &request.natural_language).await {
        Ok(updated_config) => {
            // Serialize the updated config back to YAML
            let yaml_processor = crate::config::YamlProcessor::new();
            match yaml_processor.serialize(&updated_config) {
                Ok(updated_yaml) => {
                    tracing::info!("Successfully generated updated configuration");
                    Ok(Json(ConfigUpdateResponse {
                        success: true,
                        updated_config: Some(updated_yaml),
                        error: None,
                    }))
                }
                Err(e) => {
                    tracing::error!("Failed to serialize updated config: {}", e);
                    Ok(Json(ConfigUpdateResponse {
                        success: false,
                        updated_config: None,
                        error: Some(format!("Failed to serialize config: {}", e)),
                    }))
                }
            }
        }
        Err(e) => {
            tracing::error!("Failed to update configuration: {}", e);
            Ok(Json(ConfigUpdateResponse {
                success: false,
                updated_config: None,
                error: Some(format!("Configuration update failed: {}", e)),
            }))
        }
    }
}

/// Preview configuration update without applying changes
pub async fn preview_config_update(
    State(app_state): State<AppState>,
    JsonExtractor(request): JsonExtractor<ConfigUpdateRequest>,
) -> Result<Json<ConfigUpdateResponse>, StatusCode> {
    let engine = &app_state.conversion_engine;
    tracing::info!("Received config preview request");
    
    // Get current configuration
    let current_config = engine.config();
    
    // Create LLM client for configuration updates
    let llm_config = match create_llm_config_from_env() {
        Ok(config) => config,
        Err(e) => {
            tracing::error!("Failed to create LLM config: {}", e);
            return Ok(Json(ConfigUpdateResponse {
                success: false,
                updated_config: None,
                error: Some(format!("LLM configuration error: {}", e)),
            }));
        }
    };
    
    let llm_client = match crate::llm::LlmClient::new(llm_config) {
        Ok(client) => client,
        Err(e) => {
            tracing::error!("Failed to create LLM client: {}", e);
            return Ok(Json(ConfigUpdateResponse {
                success: false,
                updated_config: None,
                error: Some(format!("Failed to initialize LLM client: {}", e)),
            }));
        }
    };
    
    // Create configuration service
    let config_service = crate::config::ConfigurationService::with_llm_client(llm_client);
    
    // Use current config or provided config as base
    let base_config = if let Some(current_yaml) = &request.current_config {
        match config_service.parse_config(current_yaml).await {
            Ok(config) => config,
            Err(e) => {
                tracing::error!("Failed to parse provided config: {}", e);
                return Ok(Json(ConfigUpdateResponse {
                    success: false,
                    updated_config: None,
                    error: Some(format!("Invalid current config: {}", e)),
                }));
            }
        }
    } else {
        current_config.clone()
    };
    
    // Preview configuration changes
    match config_service.preview_config_update(&base_config, &request.natural_language).await {
        Ok(preview_yaml) => {
            tracing::info!("Successfully generated config preview");
            Ok(Json(ConfigUpdateResponse {
                success: true,
                updated_config: Some(preview_yaml),
                error: None,
            }))
        }
        Err(e) => {
            tracing::error!("Failed to preview configuration: {}", e);
            Ok(Json(ConfigUpdateResponse {
                success: false,
                updated_config: None,
                error: Some(format!("Configuration preview failed: {}", e)),
            }))
        }
    }
}

/// Create LLM configuration from environment variables
fn create_llm_config_from_env() -> Result<crate::llm::LlmConfig, crate::error::ConfigError> {
    use crate::llm::{LlmConfig, LlmProvider};
    
    // Try to get API key from environment
    let api_key = std::env::var("OPENAI_API_KEY")
        .or_else(|_| std::env::var("CLAUDE_API_KEY"))
        .or_else(|_| std::env::var("LLM_API_KEY"))
        .map_err(|_| crate::error::ConfigError::Validation(
            "No LLM API key found in environment variables (OPENAI_API_KEY, CLAUDE_API_KEY, or LLM_API_KEY)".to_string()
        ))?;
    
    // Determine provider based on which key was found
    let (provider, model, base_url) = if std::env::var("OPENAI_API_KEY").is_ok() || std::env::var("LLM_API_KEY").is_ok() {
        (
            LlmProvider::OpenAI,
            std::env::var("OPENAI_MODEL").unwrap_or_else(|_| "gpt-3.5-turbo".to_string()),
            std::env::var("OPENAI_BASE_URL").ok(),
        )
    } else {
        (
            LlmProvider::Claude,
            std::env::var("CLAUDE_MODEL").unwrap_or_else(|_| "claude-3-sonnet-20240229".to_string()),
            std::env::var("CLAUDE_BASE_URL").ok(),
        )
    };
    
    Ok(LlmConfig {
        provider,
        api_key,
        model,
        base_url,
        max_retries: std::env::var("LLM_MAX_RETRIES")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(3),
        timeout_seconds: std::env::var("LLM_TIMEOUT_SECONDS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(30),
    })
}

/// Submit conversion task for async processing
pub async fn submit_async_conversion(
    State(app_state): State<AppState>,
    JsonExtractor(request): JsonExtractor<AsyncConvertRequest>,
) -> Result<Json<AsyncConvertResponse>, StatusCode> {
    let task_queue = match &app_state.task_queue {
        Some(queue) => queue,
        None => {
            tracing::error!("Task queue not available");
            return Ok(Json(AsyncConvertResponse {
                success: false,
                task_id: None,
                error: Some("Async processing not available".to_string()),
            }));
        }
    };
    tracing::info!("Received async conversion request");
    
    // Create conversion task
    let task = create_conversion_task(
        request.markdown,
        request.config,
        request.filename,
    );
    
    // Submit to task queue
    match task_queue.submit_task(task).await {
        Ok(task_id) => {
            tracing::info!("Submitted async conversion task: {}", task_id);
            Ok(Json(AsyncConvertResponse {
                success: true,
                task_id: Some(task_id),
                error: None,
            }))
        }
        Err(e) => {
            tracing::error!("Failed to submit async conversion task: {}", e);
            Ok(Json(AsyncConvertResponse {
                success: false,
                task_id: None,
                error: Some(format!("Failed to submit task: {}", e)),
            }))
        }
    }
}

/// Get status of an async conversion task
pub async fn get_task_status(
    State(app_state): State<AppState>,
    Path(task_id): Path<String>,
) -> Result<Json<TaskStatusResponse>, StatusCode> {
    let task_queue = match &app_state.task_queue {
        Some(queue) => queue,
        None => return Err(StatusCode::NOT_FOUND),
    };
    tracing::debug!("Checking status for task: {}", task_id);
    
    match task_queue.get_task_result(&task_id).await {
        Some(result) => {
            let completed = matches!(result.status, TaskStatus::Completed | TaskStatus::Failed);
            
            Ok(Json(TaskStatusResponse {
                task_id: result.task_id,
                status: result.status,
                completed,
                error: result.error,
            }))
        }
        None => {
            tracing::warn!("Task not found: {}", task_id);
            Err(StatusCode::NOT_FOUND)
        }
    }
}

/// Download result of completed async conversion task
pub async fn download_task_result(
    State(app_state): State<AppState>,
    Path(task_id): Path<String>,
) -> Result<Response, StatusCode> {
    let task_queue = match &app_state.task_queue {
        Some(queue) => queue,
        None => return Err(StatusCode::NOT_FOUND),
    };
    tracing::info!("Download request for task: {}", task_id);
    
    match task_queue.get_task_result(&task_id).await {
        Some(result) => {
            match result.status {
                TaskStatus::Completed => {
                    if let Some(docx_bytes) = result.result {
                        let filename = format!("converted_{}.docx", task_id);
                        
                        let response = Response::builder()
                            .status(StatusCode::OK)
                            .header(header::CONTENT_TYPE, "application/vnd.openxmlformats-officedocument.wordprocessingml.document")
                            .header(header::CONTENT_DISPOSITION, format!("attachment; filename=\"{}\"", filename))
                            .header(header::CONTENT_LENGTH, docx_bytes.len())
                            .body(axum::body::Body::from(docx_bytes))
                            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
                        
                        Ok(response)
                    } else {
                        tracing::error!("Task {} completed but no result data available", task_id);
                        Err(StatusCode::INTERNAL_SERVER_ERROR)
                    }
                }
                TaskStatus::Failed => {
                    tracing::warn!("Attempted to download failed task: {}", task_id);
                    Err(StatusCode::BAD_REQUEST)
                }
                TaskStatus::Processing | TaskStatus::Pending => {
                    tracing::info!("Task {} not yet completed", task_id);
                    Err(StatusCode::ACCEPTED) // 202 - task still processing
                }
            }
        }
        None => {
            tracing::warn!("Task not found for download: {}", task_id);
            Err(StatusCode::NOT_FOUND)
        }
    }
}

/// Handle file upload for conversion
pub async fn upload_and_convert(
    State(app_state): State<AppState>,
    mut multipart: Multipart,
) -> Result<Response, StatusCode> {
    let engine = &app_state.conversion_engine;
    tracing::info!("Received file upload request");
    
    let mut markdown_content = String::new();
    let mut config_yaml: Option<String> = None;
    let mut filename = String::from("converted.docx");
    
    // Process multipart form data
    while let Some(field) = multipart.next_field().await.map_err(|_| StatusCode::BAD_REQUEST)? {
        let field_name = field.name().unwrap_or("").to_string();
        
        match field_name.as_str() {
            "file" => {
                // Extract filename if available
                if let Some(file_name) = field.file_name() {
                    filename = file_name.replace(".md", ".docx");
                }
                
                // Read file content
                let data = field.bytes().await.map_err(|_| StatusCode::BAD_REQUEST)?;
                markdown_content = String::from_utf8(data.to_vec())
                    .map_err(|_| StatusCode::BAD_REQUEST)?;
                
                tracing::info!("Uploaded file content: {} characters", markdown_content.len());
            }
            "config" => {
                let data = field.bytes().await.map_err(|_| StatusCode::BAD_REQUEST)?;
                config_yaml = Some(String::from_utf8(data.to_vec())
                    .map_err(|_| StatusCode::BAD_REQUEST)?);
                
                tracing::info!("Received custom config with upload");
            }
            _ => {
                tracing::warn!("Unknown field in multipart upload: {}", field_name);
            }
        }
    }
    
    if markdown_content.is_empty() {
        tracing::error!("No markdown content received in upload");
        return Err(StatusCode::BAD_REQUEST);
    }
    
    // Apply custom config if provided (placeholder for future enhancement)
    if let Some(config_yaml) = config_yaml {
        tracing::info!("Applying custom YAML configuration from upload");
        
        match serde_yaml::from_str::<serde_json::Value>(&config_yaml) {
            Ok(config_value) => {
                tracing::info!("Parsed custom config: {:?}", config_value);
                // Config application will be enhanced in future tasks
                tracing::warn!("Custom config application not fully implemented, using default config");
            }
            Err(e) => {
                tracing::error!("Failed to parse uploaded YAML config: {}", e);
                return Err(StatusCode::BAD_REQUEST);
            }
        }
    }
    
    // Perform conversion
    match engine.convert(&markdown_content).await {
        Ok(docx_bytes) => {
            tracing::info!("File conversion successful, generated {} bytes", docx_bytes.len());
            
            // Return docx file as download
            let response = Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, "application/vnd.openxmlformats-officedocument.wordprocessingml.document")
                .header(header::CONTENT_DISPOSITION, format!("attachment; filename=\"{}\"", filename))
                .header(header::CONTENT_LENGTH, docx_bytes.len())
                .body(axum::body::Body::from(docx_bytes))
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            
            Ok(response)
        }
        Err(e) => {
            tracing::error!("File conversion failed: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Download converted file (for cases where conversion was done via JSON API)
pub async fn download_converted(
    State(app_state): State<AppState>,
    JsonExtractor(request): JsonExtractor<ConvertRequest>,
) -> Result<Response, StatusCode> {
    let engine = &app_state.conversion_engine;
    tracing::info!("Received download request for converted file");
    
    // Perform conversion
    match engine.convert(&request.markdown).await {
        Ok(docx_bytes) => {
            tracing::info!("Conversion for download successful, generated {} bytes", docx_bytes.len());
            
            let filename = "converted.docx";
            let response = Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, "application/vnd.openxmlformats-officedocument.wordprocessingml.document")
                .header(header::CONTENT_DISPOSITION, format!("attachment; filename=\"{}\"", filename))
                .header(header::CONTENT_LENGTH, docx_bytes.len())
                .body(axum::body::Body::from(docx_bytes))
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            
            Ok(response)
        }
        Err(e) => {
            tracing::error!("Conversion for download failed: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}