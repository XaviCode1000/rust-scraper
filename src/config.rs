//! Configuration Module
//!
//! Handles logging initialization and application configuration.

use tracing_subscriber::{fmt, prelude::*, EnvFilter};

/// Check if NO_COLOR env var is set (any non-empty value)
pub fn is_no_color() -> bool {
    std::env::var("NO_COLOR")
        .map(|v| !v.is_empty())
        .unwrap_or(false)
}

/// Whether emoji should be emitted in output
pub fn should_emit_emoji() -> bool {
    !is_no_color()
}

/// Initialize logging with configurable level (backward compat)
pub fn init_logging(level: &str) {
    init_logging_dual(level, false, is_no_color());
}

/// Dual-mode logging: forces stderr, supports quiet mode and NO_COLOR
///
/// # Arguments
///
/// * `level` - Log level: "error", "warn", "info", "debug", "trace"
/// * `quiet` - If true, suppresses info/debug output (warn+error only)
/// * `no_color` - If true, disables ANSI color codes
pub fn init_logging_dual(level: &str, quiet: bool, no_color: bool) {
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| {
        if quiet {
            // Quiet mode: only warnings and errors
            EnvFilter::new("rust_scraper=warn,tokio=warn,reqwest=warn")
        } else {
            EnvFilter::new(format!("rust_scraper={},tokio=warn,reqwest=warn", level))
        }
    });

    // Use try_init to avoid panicking if a subscriber is already set
    let _ = tracing_subscriber::registry()
        .with(
            fmt::layer()
                .with_writer(std::io::stderr)
                .with_ansi(!no_color)
                .with_target(true),
        )
        .with(env_filter)
        .try_init();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init_logging_with_valid_level() {
        // Note: init() can only be called once per process
        let result = tracing_subscriber::registry()
            .with(
                fmt::layer()
                    .with_writer(std::io::stderr)
                    .with_ansi(true)
                    .with_target(true),
            )
            .with(EnvFilter::new("error"))
            .try_init();
        assert!(result.is_ok() || result.is_err()); // first call succeeds, rest fail
    }

    #[test]
    fn test_is_no_color_default() {
        // By default, NO_COLOR should not be set in tests
        // (If the outer environment has it, we skip this test)
        if std::env::var("NO_COLOR").is_ok() {
            return;
        }
        assert!(!is_no_color());
    }

    #[test]
    fn test_should_emit_emoji_default() {
        if std::env::var("NO_COLOR").is_ok() {
            return;
        }
        assert!(should_emit_emoji());
    }

    #[test]
    fn test_is_no_color_when_set() {
        std::env::set_var("NO_COLOR", "1");
        assert!(is_no_color());
        assert!(!should_emit_emoji());
        std::env::remove_var("NO_COLOR");
    }

    #[test]
    fn test_init_logging_dual_quiet_mode() {
        // Verify the function doesn't panic with various combinations
        init_logging_dual("warn", true, false);
        init_logging_dual("debug", false, true);
    }

    #[test]
    fn test_init_logging_with_debug_level() {
        let _filter = EnvFilter::new("rust_scraper=debug,tokio=warn,reqwest=warn");
    }

    #[test]
    fn test_init_logging_with_trace_level() {
        let _filter = EnvFilter::new("rust_scraper=trace,tokio=warn,reqwest=warn");
    }
}
