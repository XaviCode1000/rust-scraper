//! Observability Module
//!
//! Production-grade observability infrastructure:
//! - Structured JSON logging with file rotation
//! - Metrics collection (HTTP, crawler)
//! - OpenTelemetry tracing (stub)
//!
//! # Usage
//!
//! ```rust
//! use rust_scraper::infrastructure::observability::{init_json_logging, MetricsCollector};
//!
//! // Initialize logging
//! init_json_logging("info", Some(&log_dir), "rust-scraper")?;
//!
//! // Use metrics collector
//! let metrics = MetricsCollector::new();
//! metrics.record_request("example.com", 150.0, 200);
//! metrics.record_page_scraped("example.com");
//!
//! // Export on completion
//! let export = metrics.export();
//! println!("{}", serde_json::to_string_pretty(&export).unwrap());
//! ```

pub mod logging;
pub mod metrics;
pub mod async_logging;

pub use logging::{init_json_logging, init_json_logging_dual, init_otel_tracing, LogFormat};
pub use async_logging::{AsyncLogWriter, WriterConfig, init_async_logging};
pub use metrics::MetricsCollector;
