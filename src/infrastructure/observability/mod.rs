//! Observability Module
//!
//! Production-grade observability infrastructure:
//! - Structured JSON logging with file rotation
//! - Metrics collection (HTTP, crawler)
//! - OpenTelemetry tracing (stub)
//! - Tokio console for runtime debugging
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
//!
//! # Tokio Console (Optional)
//!
//! For runtime observability, enable the `console` feature:
//! ```bash
//! RUSTFLAGS="--cfg tokio_unstable" cargo run --features console -- --url ...
//! ```
//!
//! Then in your code:
//! ```rust
//! #[cfg(feature = "console")]
//! rust_scraper::infrastructure::observability::init_console();
//! ```

pub mod logging;
pub mod metrics;
pub mod async_logging;

/// Initialize tokio-console for runtime debugging
/// 
/// # Requires
/// - RUSTFLAGS="--cfg tokio_unstable" at compile time
/// - Feature flag `console` enabled
/// 
/// # Note
/// Only available when compiled with `console` feature.
/// Without the feature, this function is a no-op.
#[cfg(feature = "console")]
pub fn init_console() {
    console_subscriber::init();
}

/// Placeholder when console feature is not enabled
#[cfg(not(feature = "console"))]
pub fn init_console() {
    // No-op - console not enabled
}

pub use logging::{init_json_logging, init_json_logging_dual, init_otel_tracing, LogFormat};
pub use async_logging::{AsyncLogWriter, WriterConfig, init_async_logging};
pub use metrics::MetricsCollector;
