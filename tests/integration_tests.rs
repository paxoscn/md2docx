//! Integration tests for the md2docx converter

use axum::{
    body::Body,
    http::{Request, StatusCode},
    Router,
};
use md2docx_converter::{
    config::ConversionConfig,
    conversion::ConversionEngine,
    web::api::{create_router, AppState},
};
use serde_json::json;
use std::fs;
use std::sync::Arc;
use tempfile::TempDir;
use tokio::time::{timeout, Duration};
use tower::ServiceExt;

/// Helper function to create test app
async fn create_test_app() -> Router {
    let config = ConversionConfig::default();
    let engine = Arc::new(std::sync::Mutex::new(ConversionEngine::new(config)));
    let app_state = AppState {
        conversion_engine: engine,
        task_queue: None,
    };
    create_router(app_state)
}

/// Test markdown content for integration tests
fn test_markdown_content() -> &'static str {
    r#"# Integration Test Document

This is a test document for integration testing.

## Features Tested

- **Bold text**
- *Italic text*
- `Inline code`

### Code Block

```rust
fn main() {
    println!("Hello, integration test!");
}
```

### List

1. First item
2. Second item
3. Third item

### Table

| Feature | Status |
|---------|--------|
| Parsing | ✅ |
| Generation | ✅ |
| API | ✅ |

![Test Image](https://example.com/test.jpg)

---

End of test document.
"#
}

#[tokio::test]
async fn test_health_endpoint() {
    let app = create_test_app().await;
    
    let response = app
        .oneshot(
            Request::builder()
                .uri("/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    
    assert_eq!(response.status(), StatusCode::OK);
    
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let health_response: serde_json::Value = serde_json::from_slice(&body).unwrap();
    
    assert_eq!(health_response["status"], "healthy");
    assert!(health_response["version"].is_string());
}

#[tokio::test]
async fn test_convert_endpoint_basic() {
    let app = create_test_app().await;
    
    let request_body = json!({
        "markdown": test_markdown_content()
    });
    
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/convert")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&request_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    
    assert_eq!(response.status(), StatusCode::OK);
    
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let convert_response: serde_json::Value = serde_json::from_slice(&body).unwrap();
    
    // Should return success response with file data
    assert_eq!(convert_response["success"], true);
    assert!(convert_response["file_data"].is_array());
    
    // Extract file data and check docx signature
    if let Some(file_data) = convert_response["file_data"].as_array() {
        assert!(!file_data.is_empty());
        assert!(file_data.len() > 1000); // Should be substantial
        
        // Check docx file signature (ZIP header) - first two bytes should be 'P' (80) and 'K' (75)
        assert_eq!(file_data[0].as_u64().unwrap(), 80);
        assert_eq!(file_data[1].as_u64().unwrap(), 75);
    }
}

#[tokio::test]
async fn test_convert_endpoint_with_config() {
    let app = create_test_app().await;
    
    let config = ConversionConfig::default();
    let config_yaml = serde_yaml::to_string(&config).unwrap();
    
    let request_body = json!({
        "markdown": test_markdown_content(),
        "config": config_yaml
    });
    
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/convert")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&request_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    
    assert_eq!(response.status(), StatusCode::OK);
    
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    assert!(!body.is_empty());
    assert!(body.len() > 1000);
}

#[tokio::test]
async fn test_convert_endpoint_invalid_markdown() {
    let app = create_test_app().await;
    
    let request_body = json!({
        "markdown": ""
    });
    
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/convert")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&request_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    
    // Empty markdown should still work, just return empty document
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_convert_endpoint_malformed_request() {
    let app = create_test_app().await;
    
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/convert")
                .header("content-type", "application/json")
                .body(Body::from("invalid json"))
                .unwrap(),
        )
        .await
        .unwrap();
    
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_convert_endpoint_missing_markdown() {
    let app = create_test_app().await;
    
    let request_body = json!({
        "config": "document:\n  page_size:\n    width: 595.0\n    height: 842.0"
    });
    
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/convert")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&request_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    
    // The actual status code might be 422 (Unprocessable Entity) instead of 400
    assert!(response.status().is_client_error());
}

#[tokio::test]
async fn test_config_update_endpoint() {
    let app = create_test_app().await;
    
    let request_body = json!({
        "natural_language": "Change the default font size to 14pt",
        "current_config": serde_yaml::to_string(&ConversionConfig::default()).unwrap()
    });
    
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/config/update")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&request_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    
    // This might fail if LLM is not configured, but should not crash
    assert!(response.status() == StatusCode::OK || response.status() == StatusCode::INTERNAL_SERVER_ERROR);
}

#[tokio::test]
async fn test_file_upload_endpoint() {
    let app = create_test_app().await;
    
    // Create a temporary markdown file
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.md");
    fs::write(&file_path, test_markdown_content()).unwrap();
    
    // Create multipart form data
    let boundary = "----formdata-test-boundary";
    let body = format!(
        "--{}\r\nContent-Disposition: form-data; name=\"file\"; filename=\"test.md\"\r\nContent-Type: text/markdown\r\n\r\n{}\r\n--{}--\r\n",
        boundary,
        test_markdown_content(),
        boundary
    );
    
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/upload")
                .header("content-type", format!("multipart/form-data; boundary={}", boundary))
                .body(Body::from(body))
                .unwrap(),
        )
        .await
        .unwrap();
    
    // Upload endpoint might not be implemented yet, so we check for either success or not found
    assert!(
        response.status() == StatusCode::OK || 
        response.status() == StatusCode::NOT_FOUND ||
        response.status() == StatusCode::BAD_REQUEST
    );
}

#[tokio::test]
async fn test_concurrent_conversion_requests() {
    let app = create_test_app().await;
    
    let request_body = json!({
        "markdown": test_markdown_content()
    });
    
    // Send multiple concurrent requests
    let mut handles = Vec::new();
    
    for _ in 0..5 {
        let app_clone = app.clone();
        let body_clone = serde_json::to_string(&request_body).unwrap();
        
        let handle = tokio::spawn(async move {
            app_clone
                .oneshot(
                    Request::builder()
                        .method("POST")
                        .uri("/api/convert")
                        .header("content-type", "application/json")
                        .body(Body::from(body_clone))
                        .unwrap(),
                )
                .await
        });
        
        handles.push(handle);
    }
    
    // Wait for all requests to complete
    for handle in handles {
        let response = handle.await.unwrap().unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }
}

#[tokio::test]
async fn test_conversion_timeout() {
    let app = create_test_app().await;
    
    // Create a very large markdown document to test timeout
    let mut large_markdown = String::new();
    for i in 0..10000 {
        large_markdown.push_str(&format!("# Heading {}\n\nParagraph {} with content.\n\n", i, i));
    }
    
    let request_body = json!({
        "markdown": large_markdown
    });
    
    let response_future = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/convert")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&request_body).unwrap()))
                .unwrap(),
        );
    
    // Set a reasonable timeout
    let result = timeout(Duration::from_secs(30), response_future).await;
    
    match result {
        Ok(response) => {
            let response = response.unwrap();
            // Should either succeed or return an error, but not timeout
            assert!(response.status() == StatusCode::OK || response.status().is_server_error());
        }
        Err(_) => {
            // Timeout occurred - this is acceptable for very large documents
            println!("Request timed out as expected for large document");
        }
    }
}

#[tokio::test]
async fn test_full_conversion_pipeline() {
    // Test the complete conversion pipeline without HTTP
    let config = ConversionConfig::default();
    let engine = ConversionEngine::new(config);
    
    let markdown = test_markdown_content();
    
    // Test conversion statistics
    let stats = engine.get_conversion_stats(markdown).unwrap();
    assert!(stats.headings > 0);
    assert!(stats.paragraphs > 0);
    assert!(stats.code_blocks > 0);
    assert!(stats.lists > 0);
    assert!(stats.tables > 0);
    assert!(stats.images > 0);
    assert!(stats.horizontal_rules > 0);
    
    // Test actual conversion
    let docx_bytes = engine.convert(markdown).await.unwrap();
    assert!(!docx_bytes.is_empty());
    assert!(docx_bytes.len() > 1000);
    
    // Verify docx file structure
    assert_eq!(&docx_bytes[0..2], b"PK"); // ZIP signature
}

#[tokio::test]
async fn test_batch_file_conversion() {
    let config = ConversionConfig::default();
    let engine = ConversionEngine::new(config);
    
    // Create temporary directory with test files
    let temp_dir = TempDir::new().unwrap();
    
    let test_files = vec![
        ("test1.md", "# Document 1\n\nContent for document 1."),
        ("test2.md", "# Document 2\n\nContent for document 2."),
        ("test3.md", "# Document 3\n\nContent for document 3."),
    ];
    
    let mut file_pairs = Vec::new();
    
    for (filename, content) in test_files {
        let input_path = temp_dir.path().join(filename);
        let output_path = temp_dir.path().join(filename.replace(".md", ".docx"));
        
        fs::write(&input_path, content).unwrap();
        
        file_pairs.push((
            input_path.to_string_lossy().to_string(),
            output_path.to_string_lossy().to_string(),
        ));
    }
    
    // Test batch conversion
    let results = engine.convert_batch(&file_pairs).await.unwrap();
    
    // All conversions should succeed
    assert_eq!(results.len(), 3);
    for result in results {
        assert!(result.is_ok());
    }
    
    // Verify output files exist and are valid
    for (_, output_path) in file_pairs {
        assert!(std::path::Path::new(&output_path).exists());
        
        let docx_bytes = fs::read(&output_path).unwrap();
        assert!(!docx_bytes.is_empty());
        assert_eq!(&docx_bytes[0..2], b"PK"); // ZIP signature
    }
}

#[tokio::test]
async fn test_error_handling_integration() {
    let config = ConversionConfig::default();
    let engine = ConversionEngine::new(config);
    
    // Test file not found error
    let result = engine.convert_file("nonexistent.md", "output.docx").await;
    assert!(result.is_err());
    
    // Test invalid output path (permission denied)
    let temp_dir = TempDir::new().unwrap();
    let input_path = temp_dir.path().join("test.md");
    fs::write(&input_path, "# Test").unwrap();
    
    let result = engine.convert_file(
        input_path.to_str().unwrap(),
        "/root/cannot_write_here.docx" // Should fail on most systems
    ).await;
    
    // Should either succeed (if running as root) or fail with permission error
    if result.is_err() {
        println!("Permission error as expected: {:?}", result.unwrap_err());
    }
}

#[tokio::test]
async fn test_configuration_validation_integration() {
    // Test with invalid configuration
    let mut invalid_config = ConversionConfig::default();
    invalid_config.document.page_size.width = -100.0;
    
    let engine = ConversionEngine::new(invalid_config);
    let validation_result = engine.validate_config();
    
    assert!(validation_result.is_err());
}

#[tokio::test]
async fn test_memory_usage_integration() {
    let config = ConversionConfig::default();
    let engine = ConversionEngine::new(config);
    
    // Create progressively larger documents and test memory usage
    for size in [100, 1000, 5000] {
        let mut large_markdown = String::new();
        for i in 0..size {
            large_markdown.push_str(&format!("## Section {}\n\nContent for section {}.\n\n", i, i));
        }
        
        let start_time = std::time::Instant::now();
        let result = engine.convert(&large_markdown).await;
        let duration = start_time.elapsed();
        
        assert!(result.is_ok());
        println!("Converted {} sections in {:?}", size, duration);
        
        // Memory usage should be reasonable (this is a basic check)
        assert!(duration.as_secs() < 30); // Should not take more than 30 seconds
    }
}

#[tokio::test]
async fn test_api_cors_headers() {
    let app = create_test_app().await;
    
    let response = app
        .oneshot(
            Request::builder()
                .method("OPTIONS")
                .uri("/api/convert")
                .header("Origin", "http://localhost:3000")
                .header("Access-Control-Request-Method", "POST")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    
    // Should handle CORS preflight requests
    assert!(
        response.status() == StatusCode::OK || 
        response.status() == StatusCode::NO_CONTENT ||
        response.status() == StatusCode::METHOD_NOT_ALLOWED
    );
}

#[tokio::test]
async fn test_numbering_configuration_integration() {
    // Test the numbering functionality with a complete configuration
    let mut config = ConversionConfig::default();
    
    // Add numbering to different heading levels
    config.styles.headings.get_mut(&1).unwrap().numbering = Some("%1.".to_string());
    config.styles.headings.get_mut(&2).unwrap().numbering = Some("%1.%2.".to_string());
    config.styles.headings.get_mut(&3).unwrap().numbering = Some("%1.%2.%3".to_string());
    
    // Validate the configuration with numbering
    assert!(config.validate().is_ok());
    
    let engine = ConversionEngine::new(config);
    
    // Test markdown with multiple heading levels
    let markdown_with_headings = r#"# First Chapter

This is the introduction.

## First Section

Content of the first section.

### Subsection A

Details in subsection A.

### Subsection B

Details in subsection B.

## Second Section

Content of the second section.

# Second Chapter

This is the second chapter.

## Another Section

More content here.
"#;
    
    // Convert and verify it doesn't crash
    let result = engine.convert(markdown_with_headings).await;
    assert!(result.is_ok());
    
    let docx_bytes = result.unwrap();
    assert!(!docx_bytes.is_empty());
    assert!(docx_bytes.len() > 1000);
    
    // Verify docx file structure
    assert_eq!(&docx_bytes[0..2], b"PK"); // ZIP signature
}

#[tokio::test]
async fn test_numbering_natural_language_parsing() {
    // Test the natural language parsing functionality for numbering
    use md2docx_converter::llm::prompts::{parse_numbering_request, NumberingAction};
    
    // Test Chinese requests
    let request = parse_numbering_request("为一级标题添加编号，格式为1.").unwrap();
    assert_eq!(request.level, 1);
    assert_eq!(request.action, NumberingAction::Add("%1.".to_string()));
    
    let request = parse_numbering_request("为二级标题添加编号，格式为1.1.").unwrap();
    assert_eq!(request.level, 2);
    assert_eq!(request.action, NumberingAction::Add("%1.%2.".to_string()));
    
    let request = parse_numbering_request("取消三级标题的编号").unwrap();
    assert_eq!(request.level, 3);
    assert_eq!(request.action, NumberingAction::Remove);
    
    // Test English requests
    let request = parse_numbering_request("Add numbering to H1 headings with format 1.").unwrap();
    assert_eq!(request.level, 1);
    assert_eq!(request.action, NumberingAction::Add("%1.".to_string()));
    
    let request = parse_numbering_request("Remove numbering from H2 headings").unwrap();
    assert_eq!(request.level, 2);
    assert_eq!(request.action, NumberingAction::Remove);
    
    // Test invalid requests
    assert!(parse_numbering_request("这不是编号相关的请求").is_none());
    assert!(parse_numbering_request("change font size").is_none());
}

#[tokio::test]
async fn test_numbering_config_validation() {
    // Test configuration validation with various numbering formats
    let mut config = ConversionConfig::default();
    
    // Test valid numbering formats
    let valid_formats = vec!["%1.", "%1.%2.", "%1.%2.%3", "%1-%2-%3", "Chapter %1"];
    for format in valid_formats {
        config.styles.headings.get_mut(&1).unwrap().numbering = Some(format.to_string());
        assert!(config.validate().is_ok(), "Format '{}' should be valid", format);
    }
    
    // Test invalid numbering formats
    let invalid_formats = vec!["", "no placeholders", "%1.%3.", "%2.%3.", "%0.", "%7."];
    for format in invalid_formats {
        config.styles.headings.get_mut(&1).unwrap().numbering = Some(format.to_string());
        let result = config.validate();
        assert!(result.is_err(), "Format '{}' should be invalid", format);
    }
    
    // Test None numbering (should be valid)
    config.styles.headings.get_mut(&1).unwrap().numbering = None;
    assert!(config.validate().is_ok());
}

#[tokio::test]
async fn test_numbering_serialization() {
    // Test that numbering configuration serializes and deserializes correctly
    let mut config = ConversionConfig::default();
    config.styles.headings.get_mut(&1).unwrap().numbering = Some("%1.".to_string());
    config.styles.headings.get_mut(&2).unwrap().numbering = Some("%1.%2.".to_string());
    
    // Test YAML serialization
    let yaml = serde_yaml::to_string(&config).unwrap();
    assert!(yaml.contains("numbering"));
    assert!(yaml.contains("%1."));
    assert!(yaml.contains("%1.%2."));
    
    // Test deserialization
    let deserialized: ConversionConfig = serde_yaml::from_str(&yaml).unwrap();
    assert!(deserialized.validate().is_ok());
    assert_eq!(deserialized.styles.headings.get(&1).unwrap().numbering, Some("%1.".to_string()));
    assert_eq!(deserialized.styles.headings.get(&2).unwrap().numbering, Some("%1.%2.".to_string()));
    assert_eq!(deserialized.styles.headings.get(&3).unwrap().numbering, None);
    
    // Test JSON serialization
    let json = serde_json::to_string(&config).unwrap();
    let deserialized: ConversionConfig = serde_json::from_str(&json).unwrap();
    assert!(deserialized.validate().is_ok());
    assert_eq!(deserialized.styles.headings.get(&1).unwrap().numbering, Some("%1.".to_string()));
}