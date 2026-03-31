//! Terminal setup and teardown
//!
//! Handles terminal initialization and panic hooks.
//! Follows security-no-unwrap-prod rule - no unwrap() in production code.

use crossterm::{
    cursor::Show,
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;

/// Setup terminal for TUI mode
///
/// # Returns
///
/// Configured terminal in alternate screen with raw mode enabled
///
/// # Errors
///
/// Returns `io::Error` if terminal setup fails
#[inline]
pub fn setup_terminal() -> io::Result<Terminal<CrosstermBackend<io::Stdout>>> {
    enable_raw_mode()?;
    execute!(io::stdout(), EnterAlternateScreen)?;

    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::new(backend)?;
    terminal.hide_cursor()?;

    // Setup panic hook to restore terminal on panic
    setup_panic_hook();

    Ok(terminal)
}

/// Restore terminal to normal state
///
/// # Errors
///
/// Returns `io::Error` if terminal restoration fails
#[inline]
pub fn restore_terminal() -> io::Result<()> {
    disable_raw_mode()?;
    execute!(io::stdout(), LeaveAlternateScreen)?;
    execute!(io::stdout(), Show)?;
    Ok(())
}

/// Setup panic hook to restore terminal on panic
///
/// This ensures the terminal is not left in a corrupted state
/// if the application panics during TUI mode.
///
/// Each restoration step runs independently so that a partial
/// failure (e.g., broken stdout) doesn't prevent other steps.
fn setup_panic_hook() {
    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        // Each step runs independently — failure in one doesn't block others
        // Following **err-result-over-panic**: ignore errors in cleanup path
        let _ = disable_raw_mode();
        let _ = execute!(io::stdout(), LeaveAlternateScreen);
        let _ = execute!(io::stdout(), Show);
        eprintln!("Application panicked. Terminal restored.");
        original_hook(panic_info);
    }));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore = "Requires interactive terminal"]
    fn test_terminal_setup_teardown() {
        let terminal = setup_terminal();
        assert!(terminal.is_ok());

        let _ = restore_terminal();
    }
}
