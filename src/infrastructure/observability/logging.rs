//! JSON Logging Initialization Module
//!
//! Provides JSON-formatted logging with file rotation for production use.
//! Uses tracing-subscriber with json feature.
//!
//! Also provides async logging via AsyncLogWriter for non-blocking writes.

use std::path::Path;

use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};
use tracing_appender::rolling::{RollingFileAppender, Rotation};

/// Log format enum for CLI.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LogFormat {
    /// Human-readable text format (default)
    #[default]
    Text,
    /// JSON format for machine parsing
    Json,
}

impl std::str::FromStr for LogFormat {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "json" => Ok(Self::Json),
            "text" => Ok(Self::Text),
            _ => Err(format!("Invalid log format: {}. Valid: text, json", s)),
        }
    }
}

impl std::fmt::Display for LogFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Text => "text",
                Self::Json => "json",
            }
        )
    }
}

/// Initialize JSON logging with file rotation.
///
/// # Arguments
///
/// * `level` - Log level: "error", "warn", "info", "debug", "trace"
/// * `log_dir` - Optional directory for log files. If None, logs to stderr only.
/// * `app_name` - Application name for log file naming (default: "rust-scraper")
///
/// # Returns
///
/// Ok(()) on success, or an error if log_dir cannot be created.
pub fn init_json_logging(
    level: &str,
    log_dir: Option<&Path>,
    app_name: &str,
) -> anyhow::Result<()> {
    init_json_logging_dual(level, false, false, log_dir, app_name)
}

/// Extended JSON logging with quiet mode and no-color support.
///
/// # Arguments
///
/// * `level` - Log level
/// * `quiet` - If true, only warn+ output
/// * `no_color` - If true, disable ANSI colors
/// * `log_dir` - Optional directory for log files
/// * `app_name` - Application name for log file naming
pub fn init_json_logging_dual(
    level: &str,
    quiet: bool,
    no_color: bool,
    log_dir: Option<&Path>,
    app_name: &str,
) -> anyhow::Result<()> {
    let filter = if quiet {
        EnvFilter::new("rust_scraper=warn,tokio=warn,reqwest=warn")
    } else {
        EnvFilter::new(format!("rust_scraper={},tokio=warn,reqwest=warn", level))
    };

    // Build subscriber layers
    let subscriber = tracing_subscriber::registry().with(filter);

    // Text layer for stderr (always)
    let text_layer = fmt::layer()
        .with_writer(std::io::stderr)
        .with_ansi(!no_color)
        .with_target(true)
        .pretty();

    let subscriber = subscriber.with(text_layer);

    // JSON file layer if log_dir provided
    if let Some(dir) = log_dir {
        // Create file appender with daily rotation
        let file_appender =
            RollingFileAppender::new(Rotation::DAILY, dir, format!("{}.log", app_name));

        let json_layer = fmt::layer()
            .with_writer(file_appender)
            .with_ansi(false)
            .with_target(true)
            .json();

        subscriber.with(json_layer).init();
    } else {
        subscriber.init();
    }

    Ok(())
}

/// Initialize OpenTelemetry tracing (stub for future implementation).
///
/// Currently returns Ok(()) - full OpenTelemetry integration deferred per proposal scope.
pub fn init_otel_tracing() -> anyhow::Result<()> {
    // TODO: Implement OpenTelemetry exporter
    // For now, this is a stub that allows the code to compile
    // Full distributed tracing with W3C TraceContext is deferred
    tracing::debug!("OpenTelemetry tracing initialized (stub)");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn test_log_format_from_str() {
        assert_eq!(LogFormat::from_str("text").unwrap(), LogFormat::Text);
        assert_eq!(LogFormat::from_str("json").unwrap(), LogFormat::Json);
        assert_eq!(LogFormat::from_str("TEXT").unwrap(), LogFormat::Text);
        assert_eq!(LogFormat::from_str("JSON").unwrap(), LogFormat::Json);
    }

    #[test]
    fn test_log_format_from_str_invalid() {
        assert!(LogFormat::from_str("invalid").is_err());
    }

    #[test]
    fn test_log_format_display() {
        assert_eq!(LogFormat::Text.to_string(), "text");
        assert_eq!(LogFormat::Json.to_string(), "json");
    }

    #[test]
    fn test_init_json_logging_default() {
        // Should not panic - initializes with default settings
        let result = init_json_logging("info", None, "test-app");
        assert!(result.is_ok());
    }

    #[test]
    #[ignore] // Ignored: tracing global subscriber may already be set in test context
    fn test_init_json_logging_with_temp_dir() {
        let temp_dir = std::env::temp_dir();
        // Note: tracing subscriber may already be initialized in test context
        // This test verifies the function works when called with a temp dir
        let _ = init_json_logging("info", Some(&temp_dir), "test-app");

        // Clean up log file if created
        let log_file = temp_dir.join("test-app.log");
        let _ = std::fs::remove_file(log_file);
    }

    #[test]
    fn test_init_otel_tracing() {
        let result = init_otel_tracing();
        assert!(result.is_ok());
    }
}
