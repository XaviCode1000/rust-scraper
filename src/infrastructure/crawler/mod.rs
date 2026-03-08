//! Crawler infrastructure module
//!
//! Technical implementations for web crawling:
//! - HTTP client with rate limiting
//! - Link extraction from HTML
//! - Concurrent URL queue

pub mod http_client;
pub mod link_extractor;
pub mod url_queue;

pub use http_client::{create_rate_limited_client, fetch_url};
pub use link_extractor::{extract_links, is_internal_link, normalize_url};
pub use url_queue::UrlQueue;
