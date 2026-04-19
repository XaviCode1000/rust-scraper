//! Rust Scraper - Production-ready web scraper with Clean Architecture
//!
//! Extracts clean, structured content from web pages using readability algorithm.
//!
//! # Architecture
//!
//! Following Clean Architecture with TUI support:
//!
//! ```text
//! main.rs (thin entry point) -> orchestrator::run()
//!     │
//!     ├─→ Args::try_parse()           ← CLI parsing
//!     ├─→ handle_completions()        ← Subcommand handling
//!     ├─→ ConfigDefaults::load()      ← TOML config
//!     ├─→ preflight::apply_config_defaults() ← Config merge
//!     ├─→ init_logging_dual()         ← stderr-only tracing
//!     └─→ orchestrator::run()         ← Full pipeline
//! ```
//!
//! **Golden Rule:** Application layer NEVER imports ratatui/crossterm/indicatif.

mod export_flow;
mod orchestrator;
mod preflight;

use std::env;
use std::io::{self, IsTerminal};

use clap::Parser;
use inquire::Text;
use rust_scraper::cli::config::ConfigDefaults;
use rust_scraper::cli::error::CliExit;
use rust_scraper::{init_logging_dual, is_no_color, Args, Commands};

/// Check if running in CI environment.
fn is_ci() -> bool {
    env::var("CI").is_ok()
}

/// Check if stdin is a terminal.
fn stdin_is_tty() -> bool {
    io::stdin().is_terminal()
}

/// Prompt for URL using inquire (interactive mode).
fn prompt_for_url() -> Result<String, CliExit> {
    use inquire::validator::Validation;
    
    Text::new("Enter the URL to scrape:")
        .with_help_message("Example: https://example.com")
        .with_validator(|input: &str| {
            if input.is_empty() {
                Err("URL cannot be empty".into())
            } else if !input.starts_with("http://") && !input.starts_with("https://") {
                Err("URL must start with http:// or https://".into())
            } else {
                Ok(Validation::Valid)
            }
        })
        .prompt()
        .map_err(|e| {
            eprintln!("Error prompting for URL: {}", e);
            CliExit::UsageError("interactive prompt failed".into())
        })
}

#[tokio::main]
async fn main() -> CliExit {
    // =========================================================================
    // 1. Parse CLI arguments
    // =========================================================================
    let args = match Args::try_parse() {
        Ok(args) => args,
        Err(e) => {
            eprintln!("{}", e);
            return CliExit::UsageError("invalid arguments".into());
        },
    };

    // =========================================================================
    // 2. Handle subcommands (completions)
    // =========================================================================
    if let Some(Commands::Completions { shell }) = args.subcommand {
        return orchestrator::handle_completions(shell);
    }

    // =========================================================================
    // 3. URL handling with interactive wizard
    // =========================================================================
    let mut args = args;
    
    // If no URL provided, check for interactive mode
    if args.url.is_none() {
        // CI environment always requires --url
        if is_ci() {
            eprintln!("Error: --url is required for scraping (CI mode)");
            return CliExit::UsageError("--url is required".into());
        }
        
        // Try interactive prompt only if stdin is a TTY
        if stdin_is_tty() {
            match prompt_for_url() {
                Ok(url) => {
                    args.url = Some(url);
                }
Err(_e) => {
                    // Prompt failed (e.g., non-interactive), fall through to error
                    eprintln!("Error: --url is required for scraping");
                    return CliExit::UsageError("--url is required".into());
                }
            }
        } else {
            // Not a TTY and no URL provided
            eprintln!("Error: --url is required for scraping");
            return CliExit::UsageError("--url is required".into());
        }
    }

    // =========================================================================
    // 4. Load config file (graceful: missing file = defaults)
    // =========================================================================
    let config_path = dirs::config_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("rust-scraper")
        .join("config.toml");
    let config_defaults = ConfigDefaults::load(&config_path);

    // =========================================================================
    // 5. Apply config file defaults where CLI args are at default values
    // =========================================================================
    let args = preflight::apply_config_defaults(args, &config_defaults);

    // =========================================================================
    // 6. Initialize logging (stderr-only, respects quiet + NO_COLOR)
    // =========================================================================
    let no_color = is_no_color();
    let log_level = match args.verbose {
        0 => "info",
        1 => "debug",
        _ => "trace",
    };
    init_logging_dual(log_level, args.quiet, no_color);

    // =========================================================================
    // 7. Delegate to orchestrator
    // =========================================================================
    orchestrator::run(args).await
}
