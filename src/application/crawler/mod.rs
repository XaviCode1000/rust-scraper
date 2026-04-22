//! Crawler module - Split from crawler_service.rs
//!
//! Modules:
//! - discovery: URL discovery and sitemap parsing (was 300+ lines)
//! - engine: Async crawl engine and orchestration (was 400+ lines)
//! - state: Progress and state types (kept here for now)

pub mod discovery;
pub mod engine;

/// Re-export for backwards compatibility
pub use discovery::{discover_urls, discover_urls_for_tui, fetch_sitemap};
pub use engine::{crawl_site, crawl_with_sitemap};

/// Progress tracking for the TUI
pub use crate::application::crawler_service::CrawlProgress;
/// State tracking during crawl
pub use crate::application::crawler_service::CrawlState;