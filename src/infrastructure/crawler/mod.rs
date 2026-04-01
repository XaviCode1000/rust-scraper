//! Crawler infrastructure module
//!
//! Technical implementations for web crawling:
//! - HTTP client with rate limiting
//! - Link extraction from HTML
//! - Concurrent URL queue
//! - Sitemap parsing (FASE 3)

pub mod http_client;
pub mod link_extractor;
pub mod sitemap_parser;
pub mod url_queue;

pub use http_client::{create_rate_limited_client, fetch_url};
pub use link_extractor::{extract_links, is_internal_link, normalize_url};
pub use sitemap_parser::{resolve_url, SitemapConfig, SitemapError, SitemapParser};
pub use url_queue::UrlQueue;
