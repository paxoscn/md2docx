//! Web server binary

use md2docx_converter::{ConversionConfig, ConversionEngine};
use md2docx_converter::web::{WebServer, ResourceConfig};
use tracing_subscriber;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize structured logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info".into())
        )
        .init();

    // Create default configuration
    let config = ConversionConfig::default();
    
    // Create conversion engine
    let engine = ConversionEngine::new(config);
    
    // Create resource configuration from environment variables
    let resource_config = ResourceConfig {
        max_memory_mb: std::env::var("MAX_MEMORY_MB")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(1024),
        max_cpu_percent: std::env::var("MAX_CPU_PERCENT")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(80.0),
        request_timeout_seconds: std::env::var("REQUEST_TIMEOUT_SECONDS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(60),
        rate_limit_per_minute: std::env::var("RATE_LIMIT_PER_MINUTE")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(60),
    };
    
    // Get server port from environment
    let port = std::env::var("PORT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(3000);
    
    // Get worker count for async processing
    let worker_count = std::env::var("WORKER_COUNT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(4);
    
    // Create and configure web server
    let server = WebServer::with_resource_config(engine, port, resource_config)
        .with_task_queue(worker_count);
    
    tracing::info!("Starting Markdown to docx converter server...");
    tracing::info!("Configuration: Port={}, Workers={}, Memory={}MB, CPU={:.1}%, Timeout={}s, Rate={}req/min",
        port, worker_count, 
        std::env::var("MAX_MEMORY_MB").unwrap_or_else(|_| "1024".to_string()),
        std::env::var("MAX_CPU_PERCENT").unwrap_or_else(|_| "80.0".to_string()),
        std::env::var("REQUEST_TIMEOUT_SECONDS").unwrap_or_else(|_| "60".to_string()),
        std::env::var("RATE_LIMIT_PER_MINUTE").unwrap_or_else(|_| "60".to_string())
    );
    
    server.start().await?;

    Ok(())
}