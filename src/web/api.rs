//! API route definitions

use crate::conversion::ConversionEngine;
use crate::web::{handlers, task_queue::TaskQueue};
use axum::{
    routing::{get, post},
    Router,
};
use std::sync::Arc;
use tokio::sync::Mutex;

/// Combined application state
#[derive(Clone)]
pub struct AppState {
    pub conversion_engine: Arc<Mutex<ConversionEngine>>,
    pub task_queue: Option<Arc<TaskQueue>>,
}

/// Create the main API router
pub fn create_router(app_state: AppState) -> Router {
    let mut router = Router::new()
        // Health check endpoint
        .route("/health", get(handlers::health_check))
        // Synchronous API routes
        .route("/api/convert", post(handlers::convert_markdown))
        .route("/api/convert/upload", post(handlers::upload_and_convert))
        .route("/api/convert/download", post(handlers::download_converted))
        // Configuration API routes
        .route("/api/config/update", post(handlers::update_config_natural))
        .route("/api/config/preview", post(handlers::preview_config_update))
        .route("/api/config/validate", post(handlers::validate_config));

    // Add async routes only if task queue is available
    if app_state.task_queue.is_some() {
        router = router
            .route("/api/convert/async", post(handlers::submit_async_conversion))
            .route("/api/tasks/:task_id/status", get(handlers::get_task_status))
            .route("/api/tasks/:task_id/download", get(handlers::download_task_result));
    }

    router.with_state(app_state)
}