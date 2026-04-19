# SPEC — Async-Reactive TUI

**Change:** async-reactive-tui  
**Date:** 2026-04-19  
**Version:** 1.0.0  
**Status:** SPEC

---

## 1. Overview

### Summary

Extend the TUI from URL selection to real-time scraping progress display. Add progress widget and error logging widget that update dynamically during the scraping phase. Use `tokio::mpsc` channel for progress events streaming and `tokio::select!` for non-blocking event loop that handles both user input and scraping events concurrently.

### Motivation

Current TUI workflow:
1. Discover URLs (loading spinner)
2. User selects URLs via interactive TUI
3. Download starts silently in background
4. User sees no feedback until completion

Desired workflow:
1. Discover URLs (loading spinner)
2. User selects URLs via interactive TUI
3. Download starts with **real-time progress bar**
4. User sees **per-URL status**, **errors in real-time**, **completion summary**

### Scope

**In scope:**
- Add progress widget showing scraping state per URL
- Add error logging widget for real-time error display
- Add `ScrapeProgress` enum for progress state machine
- Add `AppEvent` enum for unified event handling
- Integrate `tokio::mpsc` channel for progress events
- Integrate `tokio::select!` for non-blocking event loop
- Real-time progress updates during scraping phase

**Out of scope:**
- Changes to URL selection TUI (existing code)
- Changes to scrape logic in application layer
- Changes to export pipeline

---

## 2. Functional Requirements

### FR-1: Progress Widget

The progress widget **MUST** display:

| Field | Type | Description |
|-------|------|--------------|
| `current` | `usize` | Number of URLs processed |
| `total` | `usize` | Total number of URLs to process |
| `percentage` | `f64` | Progress as percentage |
| `status` | `ScrapeStatus` | Current scraping status |

**States (ScrapeStatus):**

| State | Meaning |
|-------|---------|
| `Pending` | URL queued, not started |
| `Fetching` | Currently fetching HTTP response |
| `Extracting` | Processing with Readability |
| `Downloading` | Downloading assets (if enabled) |
| `Completed` | Successfully scraped |
| `Failed` | Error during scrape |

**Widget Position:** Top-center of the scraping panel  
**Widget Style:** Progress bar with percentage + per-URL state list

### FR-2: Error Logging Widget

The error logging widget **MUST** display:

| Field | Type | Description |
|-------|------|--------------|
| `errors` | `Vec<ErrorEntry>` | List of errors encountered |
| `max_display` | `usize` | Maximum entries to display (configurable, default: 10) |
| `auto_scroll` | `bool` | Auto-scroll to latest error |

**ErrorEntry structure:**

```rust
struct ErrorEntry {
    timestamp: DateTime<Utc>,
    url: String,
    error_type: ErrorType,
    message: String,
}
```

**ErrorType enum:**

```rust
enum ErrorType {
    Network,
    Http(u16),          // HTTP status code
    WafBlocked(String), // WAF provider name
    Parse(String),     // Parse error description
    Timeout,
    Connection,
    Other,
}
```

**Widget Position:** Bottom panel, below progress widget  
**Widget Style:** Scrollable list, newest first, color-coded by severity

### FR-3: Progress Event Channel

The system **MUST** use `tokio::mpsc` for progress events:

**Channel type:**
```rust
// Sender: Produced by scraper service
let (progress_tx, progress_rx) = tokio::mpsc::channel::<ScrapeProgress>(100);

// Producer calls:
progress_tx.send(ScrapeProgress::Started { url: url.clone() }).await?;
progress_tx.send(ScrapeProgress::StatusChanged { url: url.clone(), status: ScrapeStatus::Fetching }).await?;
progress_tx.send(ScrapeProgress::Completed { url: url.clone(), chars: 1234 }).await?;
// On error:
progress_tx.send(ScrapeProgress::Failed { url: url.clone(), error: "...".into() }).await?;
```

**Channel semantics:**
- Bounded channel (capacity: 100 events)
- Backpressure: Producer awaits if channel full
- Events are non-blocking for scraper (fire-and-forget style)
- Failed sends logged but don't fail the scrape

### FR-4: Unified Event Loop

The TUI event loop **MUST** use `tokio::select!` for concurrent handling:

**Pattern:**
```rust
loop {
    tokio::select! {
        // Priority 1: User input (lowest latency)
        Some(event) = user_input.next() => {
            handle_user_event(event);
        }

        // Priority 2: Progress events (immediate update)
        Some(progress) = progress_rx.recv() => {
            update_progress_state(progress);
        }

        // Priority 3: Tick for animations/blink
        _ = tick_interval.tick() => {
            render_tick();
        }
    }
}
```

**Constraints:**
- `tokio::select!` (macro, not `select!` builtin) for async/await
- Non-blocking: if no event ready, render current state and loop
- User input priority: keyboard events processed first
- Tick: 100ms interval for progress bar animations

---

## 3. User Scenarios

### UC-1: Scraping with Progress

**Scenario:** User selects 5 URLs and starts scraping

**Steps:**

```
1. User runs: rust-scraper --url https://example.com --interactive
2. TUI discovers URLs (spinner)
3. TUI shows URL list, user selects 5 URLs
4. User presses Enter → Y to confirm
5. TUI switches to PROGRESS view:
   ┌────────────────────────────────────────────────────────┐
   │ 🕷️ Scraping Progress - q: Quit, Ctrl+C: Force Stop    │
   └────────────────────────────────────────────────────────┘
   ┌────────────────────────────────────────────────────────┐
   │ ████████████░░░░░░░░ 40% (2/5) | Fetching: example.com/3│
   └────────────────────────────────────────────────────────┘
   ┌────────────────────────────────────────────────────────┐
   │ URLs:                                                 │
   │ ✅ https://example.com/              (1234 chars)      │
   │ ✅ https://example.com/about       (5678 chars)      │
   │ 🔄 https://example.com/contact    Fetching...       │
   │ ⏳ https://example.com/blog/...  Pending            │
   │ ⏳ https://example.com/docs/...  Pending            │
   └────────────────────────────────────────────────────────┘
   ┌────────────────────────────────────────────────────────┐
   │ Errors (0)                                            │
   └────────────────────────────────────────────────────────┘
   ┌────────────────────────────────────────────────────────┐
   │ 📊 2 completed | 3 remaining | ⏱ Est: 15s           │
   └────────────────────────────────────────────────────────┘
6. Scrape completes → Shows completion summary
```

**Expected:**
- Progress bar updates within 100ms of each status change
- Per-URL state updates as scrape progresses
- Estimated time updates after first few URLs

### UC-2: Errors During Scrape

**Scenario:** One URL returns HTTP 404

**Steps:**

```
1. Same as UC-1, step 1-5
2. When scraping https://example.com/contact:
   - HTTP response: 404 Not Found
   - Progress event: ScrapeProgress::Failed
3. Error widget shows:
   ┌────────────────────────────────────────────────────────┐
   │ Errors (1)                                            │
   │ ⚠️ 12:34:56 https://example.com/contact → HTTP 404    │
   └────────────────────────────────────────────────────────┘
4. Scraping continues to remaining URLs
5. Final summary shows 4 succeeded, 1 failed
```

**Expected:**
- Error appears in widget within 200ms of failure
- Error includes: timestamp, URL, error type, message
- Failed URLs show ❌ in URL list, not removed

### UC-3: WAF Detection During Scrape

**Scenario:** Site returns WAF challenge (Cloudflare)

**Steps:**

```
1. Same as UC-1, step 1-5
2. When scraping https://example.com/api:
   - HTML contains Cloudflare challenge
   - Detected as WAF challenge
   - Progress event: ScrapeProgress::Failed
3. Error widget shows:
   ┌────────────────────────────────────────────────────────┐
   │ Errors (1)                                            │
   │ 🛡️ 12:34:56 https://example.com/api → WAF blocked    │
   │       (Cloudflare)                                     │
   └────────────────────────────────────────────────────────┘
```

**Expected:**
- Error type shows WAF provider name
- Specific icon for WAF (🛡️)

---

## 4. Data Structures

### ScrapeProgress Enum

```rust
/// Progress event emitted during scraping
///
/// This enum is the payload sent through tokio::mpsc channel
/// from the scraper service to the TUI.
#[derive(Debug, Clone)]
pub enum ScrapeProgress {
    /// Scraping started for a URL
    Started {
        /// URL being scraped
        url: ValidUrl,
    },

    /// Status changed for a URL
    StatusChanged {
        /// URL being scraped
        url: ValidUrl,
        /// New status
        status: ScrapeStatus,
    },

    /// Successfully completed scraping
    Completed {
        /// URL that was scraped
        url: ValidUrl,
        /// Character count of extracted content
        chars: usize,
    },

    /// Failed to scrape URL
    Failed {
        /// URL that failed
        url: ValidUrl,
        /// Error details
        error: ScrapeError,
    },

    /// All URLs processed (final event)
    Finished {
        /// Total URLs processed
        total: usize,
        /// Successful count
        successful: usize,
        /// Failed count
        failed: usize,
    },
}
```

### ScrapeStatus Enum

```rust
/// Current status of scraping for a URL
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScrapeStatus {
    /// URL queued, not started
    Pending,
    /// Currently fetching HTTP response
    Fetching,
    /// Processing with Readability algorithm
    Extracting,
    /// Downloading assets (if enabled)
    Downloading,
    /// Successfully scraped
    Completed,
    /// Error during scrape
    Failed,
}

impl ScrapeStatus {
    /// Get display icon for status
    pub fn icon(&self) -> &'static str {
        match self {
            ScrapeStatus::Pending => "⏳",
            ScrapeStatus::Fetching => "🌐",
            ScrapeStatus::Extracting => "📄",
            ScrapeStatus::Downloading => "📥",
            ScrapeStatus::Completed => "✅",
            ScrapeStatus::Failed => "❌",
        }
    }
}
```

### ScrapeError Enum

```rust
/// Error details for failed scrape
#[derive(Debug, Clone)]
pub enum ScrapeError {
    /// Network error (connection failed)
    Network(String),
    /// HTTP error with status code
    Http(u16, String),
    /// WAF/CAPTCHA challenge detected
    WafBlocked {
        /// WAF provider name
        provider: String,
        /// URL that triggered
        url: String,
    },
    /// Parse error
    Parse(String),
    /// Request timeout
    Timeout,
    /// Connection error
    Connection(String),
    /// Other/unclassified error
    Other(String),
}

impl ScrapeError {
    /// Get error type for display
    pub fn error_type(&self) -> ErrorType {
        match self {
            ScrapeError::Network(_) => ErrorType::Network,
            ScrapeError::Http(code, _) => ErrorType::Http(*code),
            ScrapeError::WafBlocked { provider, .. } => ErrorType::WafBlocked(provider.clone()),
            ScrapeError::Parse(_) => ErrorType::Parse("Parse error".to_string()),
            ScrapeError::Timeout => ErrorType::Timeout,
            ScrapeError::Connection(_) => ErrorType::Connection,
            ScrapeError::Other(_) => ErrorType::Other,
        }
    }

    /// Get user-friendly message
    pub fn message(&self) -> String {
        match self {
            ScrapeError::Network(s) => format!("Network error: {}", s),
            ScrapeError::Http(code, msg) => format!("HTTP {} - {}", code, msg),
            ScrapeError::WafBlocked { provider, url } => format!("WAF blocked ({}): {}", provider, url),
            ScrapeError::Parse(s) => format!("Parse error: {}", s),
            ScrapeError::Timeout => "Request timeout".to_string(),
            ScrapeError::Connection(s) => format!("Connection error: {}", s),
            ScrapeError::Other(s) => s.clone(),
        }
    }
}
```

### AppEvent Enum

```rust
/// Unified event type for TUI event loop
///
/// This enum represents all possible events in the TUI event loop:
/// - User input events (keyboard)
/// - Progress events from scraping
/// - System ticks for animations
pub enum AppEvent {
    /// User pressed a key
    UserInput(crossterm::event::KeyEvent),
    /// Progress update from scraper
    Progress(ScrapeProgress),
    /// Tick for animations/render
    Tick,
    /// Force quit (Ctrl+C)
    Quit,
    /// No event (render and continue)
    None,
}
```

### ErrorType Enum

```rust
/// Type of error for classification
#[derive(Debug, Clone)]
pub enum ErrorType {
    /// Network-level error
    Network,
    /// HTTP error with status code
    Http(u16),
    /// WAF/CAPTCHA blocked
    WafBlocked(String),
    /// Parse error
    Parse(String),
    /// Request timeout
    Timeout,
    /// Connection error
    Connection,
    /// Other/unknown
    Other,
}
```

### ErrorEntry Struct

```rust
/// Entry in error log widget
#[derive(Debug, Clone)]
pub struct ErrorEntry {
    /// When the error occurred
    pub timestamp: DateTime<Utc>,
    /// URL that caused the error
    pub url: String,
    /// Classification of error
    pub error_type: ErrorType,
    /// Human-readable message
    pub message: String,
}
```

### ProgressState Struct

```rust
/// Aggregated progress state for the entire batch
#[derive(Debug, Clone)]
pub struct ProgressState {
    /// All URLs being scraped
    pub urls: Vec<UrlState>,
    /// Total count
    pub total: usize,
    /// Completed count
    pub completed: usize,
    /// Failed count
    pub failed: usize,
    /// Errors encountered
    pub errors: Vec<ErrorEntry>,
    /// Start time (for ETA calculation)
    pub start_time: Option<DateTime<Utc>>,
    /// Estimated time remaining
    pub eta_seconds: Option<u64>,
}

impl ProgressState {
    /// Calculate progress percentage
    pub fn percentage(&self) -> f64 {
        if self.total == 0 {
            return 0.0;
        }
        let completed = self.completed + self.failed;
        (completed as f64 / self.total as f64) * 100.0
    }

    /// Update ETA based on current progress
    pub fn update_eta(&mut self) {
        let now = Utc::now();
        if let Some(start) = self.start_time {
            let elapsed = (now - start).num_seconds() as f64;
            let completed = self.completed as f64;
            if completed > 0.0 && self.total > self.completed {
                let per_url = elapsed / completed;
                let remaining = (self.total - self.completed) as f64;
                self.eta_seconds = Some((per_url * remaining) as u64);
            }
        } else {
            self.start_time = Some(now);
        }
    }
}
```

### UrlState Struct

```rust
/// State for a single URL in the batch
#[derive(Debug, Clone)]
pub struct UrlState {
    /// The URL
    pub url: ValidUrl,
    /// Current scraping status
    pub status: ScrapeStatus,
    /// Character count (if completed)
    pub chars: Option<usize>,
    /// Error (if failed)
    pub error: Option<ScrapeError>,
}
```

---

## 5. Architecture

### Module Structure

```
src/adapters/tui/
├── mod.rs                    # Exports + AppState
├── terminal.rs              # Terminal setup (unchanged)
├── url_selector.rs          # URL selection (unchanged)
├── progress_state.rs        # NEW: Progress state management
├── progress_widget.rs       # NEW: Progress bar widget
├── error_log_widget.rs      # NEW: Error log widget
├── scrape_progress.rs       # NEW: ScrapeProgress, ScrapeStatus, ScrapeError
├── app_event.rs            # NEW: AppEvent enum
└── event_loop.rs           # NEW: Tokio select! event loop
```

### Channel Integration

```rust
// In: src/adapters/tui/mod.rs

mod event_loop;
mod scrape_progress;
mod progress_state;
mod progress_widget;
mod error_log_widget;
mod app_event;

pub use scrape_progress::{ScrapeProgress, ScrapeStatus, ScrapeError};
pub use app_event::AppEvent;
pub use progress_state::ProgressState;
pub use error_log_widget::{ErrorEntry, ErrorType};
```

### Service Integration

The scraper service calls progress events. Two approaches:

**Option A: Pass sender to service (recommended)**
```rust
// Caller creates channel
let (progress_tx, progress_rx) = tokio::mpsc::channel(100);

// Pass sender to scraping function
let results = scrape_urls_for_tui(&urls, &config, progress_tx).await?;

// Pass receiver to TUI for progress view
let app_state = AppState::new(progress_rx, urls);
run_scraping(app_state).await?;
```

**Option B: Callback/hook**
```rust
// Config has progress callback
let config = ScraperConfig::default()
    .with_progress_hook(|progress| {
        // Send to channel
    });
```

---

## 6. Acceptance Criteria

### AC-1: Progress Widget Displays

| ID | Criterion | Verification |
|----|-----------|---------------|
| AC-1.1 | Progress widget shows current/total count | Run scrape, verify "2/5" format |
| AC-1.2 | Progress bar shows percentage | Verify "40%" in widget |
| AC-1.3 | Progress updates within 200ms of status change | Timestamp comparison |
| AC-1.4 | Per-URL state shows correct icon | Visual inspection |

### AC-2: Error Logging Widget

| ID | Criterion | Verification |
|----|-----------|---------------|
| AC-2.1 | Errors appear in widget within 200ms of failure | Timestamp comparison |
| AC-2.2 | Error shows timestamp, URL, error type | Parse error entry fields |
| AC-2.3 | WAF errors show provider name | Trigger Cloudflare, verify message |
| AC-2.4 | Maximum 10 errors displayed (default) | Scrape 15 failing URLs, count displayed |

### AC-3: Event Loop

| ID | Criterion | Verification |
|----|-----------|---------------|
| AC-3.1 | User input processed without blocking | Press key during scrape, verify immediate response |
| AC-3.2 | tokio::select! used for event handling | Code inspection |
| AC-3.3 | Event loop terminates cleanly on 'q' | Press q, verify exit |
| AC-3.4 | Ctrl+C forces stop | Press Ctrl+C, verify interrupt |

### AC-4: Data Structures

| ID | Criterion | Verification |
|----|-----------|---------------|
| AC-4.1 | ScrapeProgress enum has all variants | Match enum definition |
| AC-4.2 | ScrapeStatus has all states | Match enum definition |
| AC-4.3 | AppEvent aggregates all events | Match enum definition |
| AC-4.4 | ErrorType classifies all errors | Match enum definition |

### AC-5: End-to-End Flow

| ID | Criterion | Verification |
|----|-----------|---------------|
| AC-5.1 | TUI discovers URLs → Select → Scrape → Progress displays | Full integration test |
| AC-5.2 | Scrape with errors shows progress + errors + summary | Fail 2 URLs, verify both widgets |
| AC-5.3 | Scrape completion shows final summary | Verify "X succeeded, Y failed" |
| AC-5.4 | Terminal restores on exit | Run, exit, verify terminal state |

---

## 7. Visual Layout

### Scrape Panel Layout

```
┌─────────────────────────────────────────────────────────────────────────┐
│ 🕷️ Scraping Progress                         q: Quit     │
├─────────────────────────────────────────────────────────────────────────┤
│ ████████████████░░░░░░░░░░░░░  60% (3/5)    Est: 12s   │
├─────────────────────────────────────────────────────────────────────────┤
│ URL Status                            Chars    │
│ ✅ https://example.com/           Completed  1234     │
│ ✅ https://example.com/about      Completed  5678     │
│ 🔄 https://example.com/contact   Fetching   -        │
│ ⏳ https://example.com/blog/...  Pending    -        │
│ ⏳ https://example.com/docs/...  Pending    -        │
├─────────────────────────────────────────────────────────────────────────┤
│ Errors (1)                                                │
│ ⚠️ 12:34:56 https://example.com/contact → HTTP 404    │
├─────────────────────────────────────────────────────────────────────────┤
│ 📊 2 completed | 3 remaining | 1 failed | ⏱ 12s   │
└─────────────────────────────────────────────────────────────────────────┘
```

### Color Scheme

| Element | Color | Hex |
|---------|-------|-----|
| Progress bar fill | Green | `#4CAF50` |
| Progress bar empty | Gray | `#616161` |
| Pending icon | Gray | `#9E9E9E` |
| Fetching icon | Blue | `#2196F3` |
| Completed icon | Green | `#4CAF50` |
| Failed icon | Red | `#F44336` |
| Error warning | Orange | `#FF9800` |
| WAF error | Purple | `#9C27B0` |

---

## 8. Dependencies

### New Dependencies

| Crate | Version | Reason |
|-------|---------|--------|
| `tokio` | existing | mpsc channel, select! macro |
| `chrono` | `0.4` | Timestamp for error log |
| `crossterm` | existing | Key events |

### No New Dependencies

- `ratatui` — already used
- `thiserror` — already used

---

## 9. Testing Strategy

### Unit Tests

| Module | Tests |
|--------|-------|
| `ScrapeProgress` | All variants serialize correctly |
| `ScrapeStatus` | Icon returns correct value |
| `ProgressState` | Percentage calculates correctly |
| `AppEvent` | All variants handled |

### Integration Tests

| Test | Scenario |
|------|----------|
| `test_scrape_with_progress` | Scrape 3 URLs, verify progress events |
| `test_scrape_with_errors` | Scrape with 404, verify error displayed |
| `test_event_loop_quits` | Press 'q', verify exits cleanly |
| `test_ctrl_c_stops` | Press Ctrl+C, verify force stop |

---

## 10. Out of Scope (Deferred)

- Animated progress bar (moving segments)
- Per-URL timing (time per URL)
- Pause/resume scraping
- Retry failed URLs
- Export during scrape
- Multi-select for retry