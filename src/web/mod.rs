//! Web API and interface module

pub mod api;
pub mod handlers;
pub mod middleware;
pub mod server;
pub mod task_queue;

pub use server::WebServer;
pub use middleware::{ResourceConfig, ResourceMonitor};
pub use task_queue::{TaskQueue, TaskQueueManager, ConversionTask, TaskResult, TaskStatus};