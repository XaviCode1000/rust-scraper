//! TUI Adapter Module
//!
//! Interactive terminal UI for URL selection.
//! This is a Delivery Mechanism (Adapter layer).
//!
//! # Architecture
//!
//! The TUI is an adapter that:
//! 1. Receives discovered URLs from Application layer
//! 2. Renders interactive UI for user selection
//! 3. Returns selected URLs back to orchestrator
//!
//! # Examples
//!
//! ```no_run
//! use rust_scraper::adapters::tui;
//! use url::Url;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let urls = vec![
//!     Url::parse("https://example.com/1")?,
//!     Url::parse("https://example.com/2")?,
//! ];
//! let selected = tui::run_selector(&urls).await?;
//! # Ok(())
//! # }
//! ```

mod terminal;
mod url_selector;

pub use terminal::{restore_terminal, setup_terminal};
pub use url_selector::{run_selector, UrlSelector, UrlSelectorState};

use thiserror::Error;

/// TUI adapter errors
///
/// Follows err-thiserror-lib rule for library error types.
#[derive(Debug, Error)]
pub enum TuiError {
    #[error("Terminal setup failed: {0}")]
    TerminalSetup(#[from] std::io::Error),

    #[error("User interrupted")]
    Interrupted,
}

/// Result type for TUI operations
pub type Result<T> = std::result::Result<T, TuiError>;
