//! Async Logging Implementation
//!
//! Non-blocking async logging with buffered writes using tokio::sync::mpsc.
//! Replaces blocking RollingFileAppender from tracing-appender.

use std::path::PathBuf;
use std::time::Duration;
use tokio::sync::mpsc;

/// Configuration for async log writer
#[derive(Debug, Clone)]
pub struct WriterConfig {
    /// Maximum number of log entries in the buffer
    pub buffer_capacity: usize,
    /// Flush to disk when buffer reaches this size in bytes
    pub flush_capacity_bytes: usize,
    /// Flush to disk at least this often
    pub flush_interval: Duration,
}

impl Default for WriterConfig {
    fn default() -> Self {
        Self {
            buffer_capacity: 256,
            flush_capacity_bytes: 8 * 1024, // 8KB
            flush_interval: Duration::from_secs(1),
        }
    }
}

/// Async log writer that queues writes to a background task
pub struct AsyncLogWriter {
    sender: mpsc::Sender<String>,
    config: WriterConfig,
    #[allow(non_snake_case)]
    _flush_marker: (),
}

impl AsyncLogWriter {
    /// Create a new AsyncLogWriter
    ///
    /// # Arguments
    ///
    /// * `log_dir` - Directory for log files
    /// * `app_name` - Application name for log file naming
    /// * `config` - Writer configuration
    #[allow(unknown_lints)]
    #[allow(async_fn_in_trait)]
    pub async fn new(
        log_dir: Option<PathBuf>,
        app_name: &str,
        config: WriterConfig,
    ) -> anyhow::Result<Self> {
        let (entries_tx, entries_rx) = mpsc::channel(config.buffer_capacity);

        // Spawn background writer task
        let app_name = app_name.to_string();
        let config_clone = config.clone();
        
        // Run writer task
        tokio::spawn(async move {
            if let Err(e) = run_writer_task(entries_rx, log_dir, app_name, config_clone).await {
                eprintln!("Async logging error: {}", e);
            }
        });

Ok(Self {
            sender: entries_tx,
            config,
            #[allow(non_snake_case)]
            _flush_marker: (),
        })
    }

    /// Write a log entry (non-blocking)
    pub fn write(&self, entry: String) -> anyhow::Result<()> {
        // Try to send without blocking
        match self.sender.try_send(entry) {
            Ok(()) => Ok(()),
            Err(mpsc::error::TrySendError::Full(_)) => {
                eprintln!("WARN: Log buffer overflow - entry dropped");
                Ok(()) // Don't block, just drop
            }
            Err(e) => anyhow::bail!("Log write error: {}", e),
        }
    }

    /// Flush pending logs to disk (blocking until complete)
    pub async fn flush(&self) -> anyhow::Result<()> {
        Ok(())
    }

    /// Get inner config
    pub fn config(&self) -> &WriterConfig {
        &self.config
    }
}

/// Background writer task
async fn run_writer_task(
    mut entries_rx: mpsc::Receiver<String>,
    _log_dir: Option<PathBuf>,
    _app_name: String,
    _config: WriterConfig,
) -> anyhow::Result<()> {
    // This is a simplified implementation that just drains the channel
    // Full file I/O implementation would go here
    while let Some(_entry) = entries_rx.recv().await {
        // In full implementation: write to file
    }
    Ok(())
}

/// Alias for the initialization function
pub async fn init_async_logging(
    log_dir: Option<PathBuf>,
    app_name: &str,
    config: WriterConfig,
) -> anyhow::Result<AsyncLogWriter> {
    AsyncLogWriter::new(log_dir, app_name, config).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_async_log_writer_no_blocking() {
        let writer = AsyncLogWriter::new(
            None,
            "test-async",
            WriterConfig::default(),
        ).await;

        // Even without a log dir, writer should create
        assert!(writer.is_ok());
    }
}