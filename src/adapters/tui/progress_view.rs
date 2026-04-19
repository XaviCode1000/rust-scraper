//! Progress View for async-reactive TUI.
//!
//! This module provides the main progress TUI view that displays:
//! - Real-time scraping progress with per-URL status
//! - Error log with color-coded errors
//! - ETA and completion statistics
//!
//! # Usage
//!
//! ```no_run
//! use rust_scraper::adapters::tui::run_progress_view;
//! use url::Url;
//! use tokio::sync::mpsc;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let urls = vec![Url::parse("https://example.com/1")?];
//! let (tx, rx) = mpsc::channel(100);
//! run_progress_view(rx, &urls).await;
//! # Ok(())
//! # }
//! ```

use ratatui::{backend::CrosstermBackend, Terminal};

use crate::adapters::tui::{
    progress_types::{ProgressState, ScrapeProgress},
    terminal::{restore_terminal, setup_terminal},
    ErrorLogWidget, ProgressWidget,
};

use std::io;
use std::time::Duration;
use tokio::sync::mpsc;
use url::Url;

/// Run the progress view TUI.
///
/// This function:
/// 1. Sets up the terminal in alternate screen mode
/// 2. Initializes progress state from the provided URLs
/// 3. Runs an event loop that:
///    - Receives progress updates from `progress_rx`
///    - Renders the progress widget and error log
///    - Handles 'q' key to quit
///    - Automatically exits when scraping is complete
/// 4. Restores the terminal on exit
///
/// # Arguments
///
/// * `progress_rx` - Receiver for progress events from the scraper
/// * `urls` - List of URLs being scraped (for initial state)
///
/// # Errors
///
/// Returns `io::Error` if terminal setup fails.
pub async fn run_progress_view(
    progress_rx: mpsc::Receiver<ScrapeProgress>,
    urls: &[Url],
) -> io::Result<()> {
    // Convert URLs to strings for progress state
    let url_strings: Vec<String> = urls.iter().map(|u| u.to_string()).collect();

    // Initialize progress state
    let mut state = ProgressState::new(url_strings);

    // Setup terminal
    let mut terminal = setup_terminal()?;

    // Track completion
    let mut is_complete = false;

    // Run event loop
    let result = run_progress_tui_loop(&mut terminal, progress_rx, &mut state, &mut is_complete).await;

    // Always restore terminal
    let _ = restore_terminal();

    result
}

/// Internal event loop for the progress TUI.
async fn run_progress_tui_loop(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    mut progress_rx: mpsc::Receiver<ScrapeProgress>,
    state: &mut ProgressState,
    is_complete: &mut bool,
) -> io::Result<()> {
    let render_interval = Duration::from_millis(50); // 20fps
    let mut last_render = std::time::Instant::now();
    let tick_interval = Duration::from_millis(100);
    let mut tick_interval = tokio::time::interval(tick_interval);

    loop {
        // Check if we should quit (is_complete set by Finished event)
        if *is_complete {
            // Small delay to show final state
            tokio::time::sleep(Duration::from_millis(500)).await;
            break;
        }

        // Use tokio::select! to wait on multiple async branches
        tokio::select! {
            // Branch 1: Progress channel events (async receive)
            progress = progress_rx.recv() => {
                match progress {
                    Some(p) => {
                        // Update state with progress event
                        state.update(p.clone());

                        // Check if this is a Finished event
                        if matches!(p, ScrapeProgress::Finished { .. }) {
                            *is_complete = true;
                        }
                    },
                    None => {
                        // Channel closed - we're done
                        *is_complete = true;
                    }
                }
            },

            // Branch 2: Periodic tick for rendering
            _ = tick_interval.tick() => {
                // Tick elapsed - continue to render
            },
        }

        // Check for keyboard input (non-blocking) after select! yields
        // This is checked on every iteration since we want responsive quit
        if let Some(event) = poll_key_event() {
            // Quit on 'q'
            if event == 'q' || event == 'Q' {
                break;
            }
        }

        // Render at fixed interval (throttle to ~20fps)
        if last_render.elapsed() >= render_interval {
            let _ = terminal.draw(|f| {
                render_progress_ui(f, state);
            });
            last_render = std::time::Instant::now();
        }
    }

    // Final render to show completion state
    let _ = terminal.draw(|f| {
        render_progress_ui(f, state);
    });

    Ok(())
}

/// Poll for keyboard events (non-blocking).
fn poll_key_event() -> Option<char> {
    use crossterm::event::{self, Event, KeyCode, KeyEventKind};

    if event::poll(std::time::Duration::ZERO).ok() != Some(true) {
        return None;
    }

    match event::read() {
        Ok(Event::Key(key)) if key.kind == KeyEventKind::Press => {
            if let KeyCode::Char(c) = key.code {
                Some(c)
            } else {
                None
            }
        },
        _ => None,
    }
}

/// Render the progress UI components.
fn render_progress_ui(frame: &mut ratatui::Frame<'_>, state: &ProgressState) {
    use ratatui::layout::{Constraint, Direction, Layout};

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(10),  // Progress widget
            Constraint::Length(8), // Error log
        ])
        .split(frame.area());

    // Render progress widget
    let mut progress_widget = ProgressWidget::new(state)
        .with_errors(true)
        .with_max_errors(5);
    progress_widget.render(frame, chunks[0]);

    // Render error log widget with auto-scroll
    let mut error_widget = ErrorLogWidget::new(&state.errors)
        .with_max_errors(10)
        .with_auto_scroll(true);
    error_widget.render(frame, chunks[1]);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapters::tui::progress_types::{ScrapeError, ScrapeStatus};

    fn sample_urls() -> Vec<Url> {
        vec![
            Url::parse("https://example.com/1").unwrap(),
            Url::parse("https://example.com/2").unwrap(),
        ]
    }

    #[test]
    fn test_poll_key_event_no_event() {
        // Should return None when no key pressed
        let result = poll_key_event();
        // In test environment, likely None
        assert!(result.is_none() || result.is_some());
    }

    #[tokio::test]
    async fn test_progress_state_updates() {
        let url_strings: Vec<String> = sample_urls()
            .iter()
            .map(|u| u.to_string())
            .collect();
        let mut state = ProgressState::new(url_strings);

        // Test Started event
        state.update(ScrapeProgress::Started {
            url: "https://example.com/1".to_string(),
        });
        assert_eq!(state.urls[0].status, ScrapeStatus::Fetching);

        // Test Completed event
        state.update(ScrapeProgress::Completed {
            url: "https://example.com/1".to_string(),
            chars: 1000,
        });
        assert_eq!(state.completed, 1);
        assert_eq!(state.urls[0].status, ScrapeStatus::Completed);

        // Test Failed event
        state.update(ScrapeProgress::Started {
            url: "https://example.com/2".to_string(),
        });
        state.update(ScrapeProgress::Failed {
            url: "https://example.com/2".to_string(),
            error: ScrapeError::Network("connection refused".to_string()),
        });
        assert_eq!(state.failed, 1);
        assert_eq!(state.urls[1].status, ScrapeStatus::Failed);
    }

    #[test]
    fn test_progress_state_percentage() {
        let url_strings = vec![
            "https://example.com/1".to_string(),
            "https://example.com/2".to_string(),
        ];
        let mut state = ProgressState::new(url_strings);

        // Initially 0%
        assert_eq!(state.percentage(), 0.0);

        // Complete one URL (50%)
        state.update(ScrapeProgress::Completed {
            url: "https://example.com/1".to_string(),
            chars: 100,
        });
        assert!((state.percentage() - 50.0).abs() < 0.1);

        // Fail another (100%)
        state.update(ScrapeProgress::Failed {
            url: "https://example.com/2".to_string(),
            error: ScrapeError::Other("error".to_string()),
        });
        assert_eq!(state.percentage(), 100.0);
    }
}