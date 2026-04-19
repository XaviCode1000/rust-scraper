//! Async event loop for reactive TUI.
//!
//! This module implements an async event loop that listens to multiple input sources:
//! - crossterm terminal events (keyboard input)
//! - MPSC channel for application events
//! - Periodic tick for UI updates
//!
//! Uses tokio::select! for non-blocking concurrent event handling.

use crate::adapters::tui::progress_types::AppEvent;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::time::interval;

/// Receiver for application-sourced events.
///
/// This channel allows other async tasks to push events into the TUI event loop.
/// Non-blocking — events are queued and processed asynchronously.
pub type AppEventReceiver = mpsc::Receiver<AppEvent>;

/// Sender for application-sourced events.
///
/// Clonable so multiple tasks can send events to the TUI.
/// Non-blocking — send() returns immediately if channel has capacity.
pub type AppEventSender = mpsc::Sender<AppEvent>;

/// Create a new MPSC channel for AppEvent.
///
/// # Returns
///
/// (sender, receiver) pair. The sender can be cloned and shared across tasks.
/// Channel buffer size is 16 by default.
pub fn app_event_channel() -> (AppEventSender, AppEventReceiver) {
    mpsc::channel(16)
}

/// TUI event loop configuration.
#[derive(Debug, Clone)]
pub struct EventLoopConfig {
    /// Tick interval for periodic UI updates (default: 100ms)
    pub tick_interval: Duration,
    /// Whether quiet mode is active (skip TUI rendering)
    pub quiet: bool,
}

impl EventLoopConfig {
    /// Create new config with defaults.
    #[must_use]
    pub fn new() -> Self {
        Self {
            tick_interval: Duration::from_millis(100),
            quiet: false,
        }
    }

    /// Set quiet mode.
    #[must_use]
    pub fn with_quiet(mut self, quiet: bool) -> Self {
        self.quiet = quiet;
        self
    }

    /// Set tick interval.
    #[must_use]
    pub fn with_tick_interval(mut self, interval: Duration) -> Self {
        self.tick_interval = interval;
        self
    }
}

impl Default for EventLoopConfig {
    fn default() -> Self {
        Self::new()
    }
}

/// Event loop state passed to handlers.
#[derive(Debug)]
pub struct EventLoopState {
    /// Current tick count
    pub tick_count: u64,
    /// Whether to exit the loop
    pub should_quit: bool,
}

impl EventLoopState {
    /// Create new state.
    #[must_use]
    pub fn new() -> Self {
        Self {
            tick_count: 0,
            should_quit: false,
        }
    }
}

impl Default for EventLoopState {
    fn default() -> Self {
        Self::new()
    }
}

/// Poll for crossterm events (non-blocking).
///
/// # Returns
///
/// Some(AppEvent) if event available, None otherwise.
#[inline]
pub fn poll_crossterm_event() -> Option<AppEvent> {
    // Use try_read for non-blocking behavior
    // Note: crossterm's poll is already non-blocking, but we wrap to convert to Option
    event::poll(Duration::ZERO).ok().and_then(|available| {
        if available {
            match event::read() {
                Ok(Event::Key(key)) => {
                    if key.kind == KeyEventKind::Press {
                        Some(handle_key_event(key.code))
                    } else {
                        Some(AppEvent::None)
                    }
                },
                Ok(Event::Resize(_, _)) => Some(AppEvent::None),
                Ok(Event::FocusGained) | Ok(Event::FocusLost) => Some(AppEvent::None),
                Ok(Event::Mouse(_)) | Ok(Event::Paste(_)) => Some(AppEvent::None),
                Err(_) => Some(AppEvent::None),
            }
        } else {
            None
        }
    })
}

/// Convert crossterm KeyCode to AppEvent.
fn handle_key_event(code: KeyCode) -> AppEvent {
    match code {
        KeyCode::Char(c) => AppEvent::UserInput(c.to_string()),
        KeyCode::Esc => AppEvent::UserInput("Escape".to_string()),
        KeyCode::Up => AppEvent::UserInput("Up".to_string()),
        KeyCode::Down => AppEvent::UserInput("Down".to_string()),
        KeyCode::Enter => AppEvent::UserInput("Enter".to_string()),
        _ => AppEvent::None,
    }
}

/// Run the async event loop.
///
/// This function uses tokio::select! to concurrently listen to:
/// 1. Terminal input events (non-blocking)
/// 2. Application event channel
/// 3. Periodic tick events
///
/// # Arguments
///
/// * `config` - Event loop configuration
/// * `receiver` - MPSC receiver for application events
/// * `handler` - Async function to handle each event
///
/// # Notes
///
/// - Does NOT render UI — that's the caller's responsibility
/// - Skips rendering in quiet mode but still processes events
/// - Handler receives AppEvent and should return bool:
///   - true = continue running
///   - false = exit loop
pub async fn run_event_loop<F>(config: EventLoopConfig, receiver: AppEventReceiver, mut handler: F)
where
    F: FnMut(AppEvent, &mut EventLoopState) -> bool,
{
    let mut state = EventLoopState::new();
    let mut tick_interval = interval(config.tick_interval);
    let mut receiver = receiver;

    // Main event loop with tokio::select!
    loop {
        tokio::select! {
            // Priority 1: User input (crossterm) - non-blocking
            // In quiet mode, skip crossterm polling
            _ = async {
                poll_crossterm_event()
            }, if !config.quiet => {
                let event = poll_crossterm_event();
                if let Some(ev) = event {
                    if !handler(ev, &mut state) {
                        break;
                    }
                }
            },

            // Priority 2: Application events via MPSC channel
            app_event = receiver.recv() => {
                match app_event {
                    Some(event) => {
                        if !handler(event, &mut state) {
                            break;
                        }
                    },
                    None => {
                        // Channel closed - exit gracefully
                        break;
                    }
                }
            },

            // Priority 3: Periodic tick
            _ = tick_interval.tick() => {
                state.tick_count += 1;
                if !handler(AppEvent::Tick, &mut state) {
                    break;
                }
            },
        }

        // Check quit flag
        if state.should_quit {
            break;
        }
    }
}

/// Convenience: run event loop until Quit event.
pub async fn run_event_loop_until_quit(config: EventLoopConfig, receiver: AppEventReceiver) {
    run_event_loop(config, receiver, |event, state| {
        if matches!(event, AppEvent::Quit) {
            state.should_quit = true;
            return false;
        }
        true
    })
    .await;
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[tokio::test]
    async fn test_channel_creation() {
        let (tx, mut rx) = app_event_channel();
        assert!(tx.send(AppEvent::Tick).await.is_ok());
        assert!(rx.recv().await.is_some());
    }

    #[test]
    fn test_config_defaults() {
        let config = EventLoopConfig::new();

        assert_eq!(config.tick_interval, Duration::from_millis(100));
        assert!(!config.quiet);
    }

    #[test]
    fn test_config_builder() {
        let config = EventLoopConfig::new()
            .with_quiet(true)
            .with_tick_interval(Duration::from_millis(500));

        assert!(config.quiet);
        assert_eq!(config.tick_interval, Duration::from_millis(500));
    }

    #[test]
    fn test_event_loop_state_defaults() {
        let state = EventLoopState::new();
        assert_eq!(state.tick_count, 0);
        assert!(!state.should_quit);
    }

    #[test]
    fn test_poll_crossterm_event_no_event() {
        // When no event available, should return None immediately
        // This tests non-blocking behavior
        let result = poll_crossterm_event();
        // Result depends on whether terminal is available
        // In test environment, likely None (no terminal)
        // But the function should not block
        assert!(result.is_none() || matches!(result, Some(AppEvent::None)));
    }

    #[tokio::test]
    async fn test_event_loop_exits_on_quit() {
        let (tx, rx) = app_event_channel();
        let config = EventLoopConfig::new();

        // Send Quit event - loop should exit
        tx.send(AppEvent::Quit).await.unwrap();

        run_event_loop(config, rx, |event, state| {
            if matches!(event, AppEvent::Quit) {
                state.should_quit = true;
                return false;
            }
            true
        })
        .await;
    }

    #[tokio::test]
    async fn test_quiet_mode_skips_crossterm() {
        let (_tx, rx) = app_event_channel();
        let config = EventLoopConfig::new().with_quiet(true);

        let tick_count = std::sync::Arc::new(std::sync::atomic::AtomicU64::new(0));
        let tick_count_clone = tick_count.clone();

        // In quiet mode, crossterm polling is skipped
        // But ticks should still work
        run_event_loop(config, rx, move |event, state| {
            if matches!(event, AppEvent::Tick) {
                tick_count_clone.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            }
            // Exit after a few ticks
            state.tick_count < 3
        })
        .await;

        assert!(tick_count.load(std::sync::atomic::Ordering::Relaxed) > 0);
    }
}
