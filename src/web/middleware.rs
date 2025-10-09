//! Middleware for request handling, rate limiting, and resource management

use axum::{
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::Response,
};
use governor::{Quota, RateLimiter};
use std::{
    num::NonZeroU32,
    sync::Arc,
    time::Duration,
};
use sysinfo::System;
use tokio::time::timeout;
use tracing::{error, info, warn};

/// Rate limiter for API requests
pub type ApiRateLimiter = Arc<RateLimiter<governor::state::direct::NotKeyed, governor::state::InMemoryState, governor::clock::DefaultClock>>;

/// Resource monitor for tracking system usage
pub struct ResourceMonitor {
    system: Arc<tokio::sync::Mutex<System>>,
    max_memory_mb: u64,
    max_cpu_percent: f32,
}

/// Configuration for resource management
#[derive(Debug, Clone)]
pub struct ResourceConfig {
    pub max_memory_mb: u64,
    pub max_cpu_percent: f32,
    pub request_timeout_seconds: u64,
    pub rate_limit_per_minute: u32,
}

impl Default for ResourceConfig {
    fn default() -> Self {
        Self {
            max_memory_mb: 102400, // 100GB
            max_cpu_percent: 80.0,
            request_timeout_seconds: 60,
            rate_limit_per_minute: 60,
        }
    }
}

impl ResourceMonitor {
    /// Create a new resource monitor
    pub fn new(config: ResourceConfig) -> Self {
        let mut system = System::new_all();
        system.refresh_all();
        
        Self {
            system: Arc::new(tokio::sync::Mutex::new(system)),
            max_memory_mb: config.max_memory_mb,
            max_cpu_percent: config.max_cpu_percent,
        }
    }

    /// Check if system resources are within acceptable limits
    pub async fn check_resources(&self) -> Result<(), String> {
        let mut system = self.system.lock().await;
        system.refresh_all();

        // Check memory usage
        let used_memory = system.used_memory() / 1024 / 1024; // Convert to MB
        if used_memory > self.max_memory_mb {
            return Err(format!(
                "Memory usage too high: {}MB > {}MB",
                used_memory, self.max_memory_mb
            ));
        }

        // Check CPU usage
        let cpu_usage = system.global_cpu_info().cpu_usage();
        if cpu_usage > self.max_cpu_percent {
            return Err(format!(
                "CPU usage too high: {:.1}% > {:.1}%",
                cpu_usage, self.max_cpu_percent
            ));
        }

        Ok(())
    }

    /// Get current resource usage statistics
    pub async fn get_stats(&self) -> ResourceStats {
        let mut system = self.system.lock().await;
        system.refresh_all();

        ResourceStats {
            memory_used_mb: system.used_memory() / 1024 / 1024,
            memory_total_mb: system.total_memory() / 1024 / 1024,
            cpu_usage_percent: system.global_cpu_info().cpu_usage(),
            process_count: system.processes().len(),
        }
    }
}

/// Resource usage statistics
#[derive(Debug, Clone)]
pub struct ResourceStats {
    pub memory_used_mb: u64,
    pub memory_total_mb: u64,
    pub cpu_usage_percent: f32,
    pub process_count: usize,
}

/// Create a rate limiter with the specified requests per minute
pub fn create_rate_limiter(requests_per_minute: u32) -> ApiRateLimiter {
    let quota = Quota::per_minute(NonZeroU32::new(requests_per_minute).unwrap());
    Arc::new(RateLimiter::direct(quota))
}

/// Rate limiting middleware
pub async fn rate_limit_middleware(
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Extract rate limiter from request extensions
    let rate_limiter = request
        .extensions()
        .get::<ApiRateLimiter>()
        .cloned();

    if let Some(limiter) = rate_limiter {
        // Check rate limit
        match limiter.check() {
            Ok(_) => {
                // Request allowed, proceed
                Ok(next.run(request).await)
            }
            Err(_) => {
                warn!("Rate limit exceeded for request");
                Err(StatusCode::TOO_MANY_REQUESTS)
            }
        }
    } else {
        // No rate limiter configured, proceed without limiting
        Ok(next.run(request).await)
    }
}

/// Resource monitoring middleware
pub async fn resource_monitor_middleware(
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Extract resource monitor from request extensions
    let monitor = request
        .extensions()
        .get::<Arc<ResourceMonitor>>()
        .cloned();

    if let Some(monitor) = monitor {
        // Check system resources before processing request
        match monitor.check_resources().await {
            Ok(_) => {
                // Resources OK, proceed with request
                Ok(next.run(request).await)
            }
            Err(error_msg) => {
                error!("Resource limit exceeded: {}", error_msg);
                Err(StatusCode::SERVICE_UNAVAILABLE)
            }
        }
    } else {
        // No resource monitor configured, proceed without checking
        Ok(next.run(request).await)
    }
}

/// Request timeout middleware
pub async fn timeout_middleware(
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Get timeout duration from request extensions or use default
    let timeout_duration = request
        .extensions()
        .get::<Duration>()
        .cloned()
        .unwrap_or(Duration::from_secs(60));

    // Apply timeout to request processing
    match timeout(timeout_duration, next.run(request)).await {
        Ok(response) => Ok(response),
        Err(_) => {
            warn!("Request timed out after {:?}", timeout_duration);
            Err(StatusCode::REQUEST_TIMEOUT)
        }
    }
}

/// Combined middleware that adds resource management extensions to requests
pub async fn resource_management_middleware(
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Add default resource management components if not already present
    if !request.extensions().get::<ApiRateLimiter>().is_some() {
        let rate_limiter = create_rate_limiter(60); // Default 60 requests per minute
        request.extensions_mut().insert(rate_limiter);
    }

    if !request.extensions().get::<Arc<ResourceMonitor>>().is_some() {
        let monitor = Arc::new(ResourceMonitor::new(ResourceConfig::default()));
        request.extensions_mut().insert(monitor);
    }

    if !request.extensions().get::<Duration>().is_some() {
        request.extensions_mut().insert(Duration::from_secs(60));
    }

    Ok(next.run(request).await)
}

/// Middleware to log resource usage for monitoring
pub async fn resource_logging_middleware(
    request: Request,
    next: Next,
) -> Response {
    let start_time = std::time::Instant::now();
    
    // Get resource monitor if available
    let monitor = request
        .extensions()
        .get::<Arc<ResourceMonitor>>()
        .cloned();

    let response = next.run(request).await;
    
    let duration = start_time.elapsed();
    
    // Log resource usage if monitor is available
    if let Some(monitor) = monitor {
        let stats = monitor.get_stats().await;
        info!(
            "Request completed in {:?} - Memory: {}MB/{} MB ({:.1}%), CPU: {:.1}%",
            duration,
            stats.memory_used_mb,
            stats.memory_total_mb,
            (stats.memory_used_mb as f32 / stats.memory_total_mb as f32) * 100.0,
            stats.cpu_usage_percent
        );
    }

    response
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resource_config_default() {
        let config = ResourceConfig::default();
        assert_eq!(config.max_memory_mb, 1024);
        assert_eq!(config.max_cpu_percent, 80.0);
        assert_eq!(config.request_timeout_seconds, 60);
        assert_eq!(config.rate_limit_per_minute, 60);
    }

    #[test]
    fn test_create_rate_limiter() {
        let limiter = create_rate_limiter(100);
        
        // Should allow first request
        assert!(limiter.check().is_ok());
    }

    #[tokio::test]
    async fn test_resource_monitor_creation() {
        let config = ResourceConfig::default();
        let monitor = ResourceMonitor::new(config);
        
        // Should be able to get stats
        let stats = monitor.get_stats().await;
        assert!(stats.memory_total_mb > 0);
    }

    #[tokio::test]
    async fn test_resource_monitor_check() {
        let config = ResourceConfig {
            max_memory_mb: u64::MAX, // Very high limit
            max_cpu_percent: 100.0,  // Very high limit
            ..Default::default()
    };
        let monitor = ResourceMonitor::new(config);
        
        // Should pass with very high limits
        assert!(monitor.check_resources().await.is_ok());
    }

    #[tokio::test]
    async fn test_resource_monitor_check_failure() {
        let config = ResourceConfig {
            max_memory_mb: 1, // Very low limit
            max_cpu_percent: 0.1, // Very low limit
            ..Default::default()
        };
        let monitor = ResourceMonitor::new(config);
        
        // Should fail with very low limits
        assert!(monitor.check_resources().await.is_err());
    }
}