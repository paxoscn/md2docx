//! Web server implementation

use crate::conversion::ConversionEngine;
use crate::error::ConversionError;
use crate::web::api::{create_router, AppState};
use crate::web::middleware::{
    resource_management_middleware, rate_limit_middleware, resource_monitor_middleware,
    timeout_middleware, resource_logging_middleware, ResourceConfig,
};
use crate::web::task_queue::TaskQueueManager;
use axum::{middleware, Router};
use std::{sync::Arc, time::Duration};
use tokio::net::TcpListener;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use tracing::{info, warn};

/// Web server for the conversion API
pub struct WebServer {
    conversion_engine: Arc<ConversionEngine>,
    port: u16,
    resource_config: ResourceConfig,
    task_queue_manager: Option<Arc<TaskQueueManager>>,
}

impl WebServer {
    /// Create a new web server
    pub fn new(conversion_engine: ConversionEngine, port: u16) -> Self {
        Self {
            conversion_engine: Arc::new(conversion_engine),
            port,
            resource_config: ResourceConfig::default(),
            task_queue_manager: None,
        }
    }

    /// Create a new web server with custom resource configuration
    pub fn with_resource_config(
        conversion_engine: ConversionEngine,
        port: u16,
        resource_config: ResourceConfig,
    ) -> Self {
        Self {
            conversion_engine: Arc::new(conversion_engine),
            port,
            resource_config,
            task_queue_manager: None,
        }
    }

    /// Enable async task queue for large file processing
    pub fn with_task_queue(mut self, worker_count: usize) -> Self {
        let task_queue_manager = Arc::new(TaskQueueManager::new(
            self.conversion_engine.clone(),
            worker_count,
            Duration::from_secs(300), // Cleanup every 5 minutes
            Duration::from_secs(3600), // Keep tasks for 1 hour
        ));
        
        self.task_queue_manager = Some(task_queue_manager);
        self
    }

    /// Start the web server
    pub async fn start(&self) -> Result<(), ConversionError> {
        // Start task queue cleanup if enabled
        if let Some(task_queue_manager) = &self.task_queue_manager {
            task_queue_manager.start_cleanup_task();
            info!("Task queue manager started with cleanup enabled");
        }

        let app = self.create_app();
        let listener = TcpListener::bind(format!("0.0.0.0:{}", self.port))
            .await
            .map_err(|e| ConversionError::Io(e))?;

        info!("Server starting on port {} with resource management enabled", self.port);
        info!("Resource limits: Memory: {}MB, CPU: {:.1}%, Timeout: {}s, Rate limit: {}/min",
            self.resource_config.max_memory_mb,
            self.resource_config.max_cpu_percent,
            self.resource_config.request_timeout_seconds,
            self.resource_config.rate_limit_per_minute
        );
        
        axum::serve(listener, app)
            .await
            .map_err(|e| ConversionError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;

        Ok(())
    }

    /// Create the Axum application
    fn create_app(&self) -> Router {
        // Create app state
        let app_state = AppState {
            conversion_engine: self.conversion_engine.clone(),
            task_queue: self.task_queue_manager.as_ref().map(|tm| tm.queue()),
        };

        create_router(app_state)
            // CORS layer
            .layer(
                CorsLayer::new()
                    .allow_origin(Any)
                    .allow_methods(Any)
                    .allow_headers(Any)
            )
            // Tracing layer for HTTP requests
            .layer(TraceLayer::new_for_http())
            // Resource management middleware layers (order matters!)
            .layer(middleware::from_fn(resource_logging_middleware))
            .layer(middleware::from_fn(timeout_middleware))
            .layer(middleware::from_fn(resource_monitor_middleware))
            .layer(middleware::from_fn(rate_limit_middleware))
            .layer(middleware::from_fn(resource_management_middleware))
            // Custom logging middleware
            .layer(middleware::from_fn(logging_middleware))
    }
}

/// Logging middleware for request/response logging
async fn logging_middleware(
    request: axum::extract::Request,
    next: axum::middleware::Next,
) -> axum::response::Response {
    let method = request.method().clone();
    let uri = request.uri().clone();
    let start = std::time::Instant::now();
    
    info!("Request: {} {}", method, uri);
    
    let response = next.run(request).await;
    let duration = start.elapsed();
    
    let status = response.status();
    if status.is_success() {
        info!("Response: {} {} - {} in {:?}", method, uri, status, duration);
    } else {
        warn!("Response: {} {} - {} in {:?}", method, uri, status, duration);
    }
    
    response
}