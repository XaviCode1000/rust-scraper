//! Crawler domain entities
//!
//! Core business entities for web crawling functionality.
//! Following Clean Architecture: pure domain logic, no framework dependencies.
//!
//! # Rules Applied
//!
//! - **err-thiserror-for-libraries**: `CrawlError` uses thiserror
//! - **api-builder**: `CrawlerConfig` with builder pattern
//! - **api-must-use**: `#[must_use]` on result structs
//! - **api-non-exhaustive**: `#[non_exhaustive]` for future evolution

use thiserror::Error;
use url::Url;

/// Content type discovered during crawling
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContentType {
    /// HTML page
    Html,
    /// XML document (including sitemaps)
    Xml,
    /// Plain text
    Text,
    /// Unknown or other content type
    Other,
}

impl Default for ContentType {
    fn default() -> Self {
        Self::Html
    }
}

/// A discovered URL during crawling
///
/// Note: Cannot derive `Copy` because `Url` is not `Copy`.
/// Following **own-borrow-over-clone**: We'll pass references where possible.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiscoveredUrl {
    /// The discovered URL
    pub url: Url,
    /// Depth in the crawl tree (0 = seed)
    pub depth: u8,
    /// Parent URL that led to this discovery
    pub parent_url: Url,
    /// Content type if known
    pub content_type: ContentType,
}

impl DiscoveredUrl {
    /// Create a new discovered URL
    #[must_use]
    pub fn new(url: Url, depth: u8, parent_url: Url, content_type: ContentType) -> Self {
        Self {
            url,
            depth,
            parent_url,
            content_type,
        }
    }

    /// Create a new discovered URL with default HTML content type
    #[must_use]
    pub fn html(url: Url, depth: u8, parent_url: Url) -> Self {
        Self {
            url,
            depth,
            parent_url,
            content_type: ContentType::Html,
        }
    }
}

/// Crawler configuration with builder pattern
///
/// Following **api-builder**: Provides fluent builder API.
/// Following **api-non-exhaustive**: Can evolve without breaking changes.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct CrawlerConfig {
    /// Seed URL to start crawling from
    pub seed_url: Url,
    /// Maximum depth to crawl (0 = only seed)
    pub max_depth: u8,
    /// Maximum number of pages to crawl
    pub max_pages: usize,
    /// URL patterns to include (glob-style)
    pub include_patterns: Vec<String>,
    /// URL patterns to exclude (glob-style)
    pub exclude_patterns: Vec<String>,
    /// Concurrency level (number of parallel requests)
    pub concurrency: usize,
    /// Delay between requests in milliseconds (rate limiting)
    pub delay_ms: u64,
    /// User agent string
    pub user_agent: String,
    /// Timeout for each request in seconds
    pub timeout_secs: u64,
}

impl CrawlerConfig {
    /// Create a new config with seed URL
    ///
    /// Following **api-builder**: Returns builder for fluent configuration.
    #[must_use]
    pub fn builder(seed_url: Url) -> CrawlerConfigBuilder {
        CrawlerConfigBuilder::new(seed_url)
    }

    /// Create a new config with default values
    #[must_use]
    pub fn new(seed_url: Url) -> Self {
        Self {
            seed_url,
            max_depth: 3,
            max_pages: 100,
            include_patterns: Vec::new(),
            exclude_patterns: Vec::new(),
            concurrency: 3, // Hardware-aware: nproc - 1 for 4C CPU
            delay_ms: 500,  // Hardware-aware: 500ms for HDD
            user_agent: "rust-scraper/0.3.0 (Web Crawler)".to_string(),
            timeout_secs: 30,
        }
    }

    /// Check if a URL matches the include patterns
    #[inline]
    #[must_use]
    pub fn matches_include(&self, url: &str) -> bool {
        if self.include_patterns.is_empty() {
            return true;
        }
        self.include_patterns
            .iter()
            .any(|pattern| matches_pattern(url, pattern))
    }

    /// Check if a URL matches the exclude patterns
    #[inline]
    #[must_use]
    pub fn matches_exclude(&self, url: &str) -> bool {
        self.exclude_patterns
            .iter()
            .any(|pattern| matches_pattern(url, pattern))
    }
}

/// Builder for CrawlerConfig
///
/// Following **api-builder** and **api-must-use**.
#[derive(Debug)]
#[must_use]
pub struct CrawlerConfigBuilder {
    seed_url: Url,
    max_depth: u8,
    max_pages: usize,
    include_patterns: Vec<String>,
    exclude_patterns: Vec<String>,
    concurrency: usize,
    delay_ms: u64,
    user_agent: String,
    timeout_secs: u64,
}

impl CrawlerConfigBuilder {
    /// Create a new builder with seed URL
    #[must_use]
    pub fn new(seed_url: Url) -> Self {
        Self {
            seed_url,
            max_depth: 3,
            max_pages: 100,
            include_patterns: Vec::new(),
            exclude_patterns: Vec::new(),
            concurrency: 3,
            delay_ms: 500,
            user_agent: "rust-scraper/0.3.0 (Web Crawler)".to_string(),
            timeout_secs: 30,
        }
    }

    /// Set maximum crawl depth
    #[must_use]
    pub fn max_depth(mut self, depth: u8) -> Self {
        self.max_depth = depth;
        self
    }

    /// Set maximum number of pages
    #[must_use]
    pub fn max_pages(mut self, pages: usize) -> Self {
        self.max_pages = pages;
        self
    }

    /// Add an include pattern
    #[must_use]
    pub fn include_pattern(mut self, pattern: impl Into<String>) -> Self {
        self.include_patterns.push(pattern.into());
        self
    }

    /// Add multiple include patterns
    #[must_use]
    pub fn include_patterns(mut self, patterns: Vec<String>) -> Self {
        self.include_patterns.extend(patterns);
        self
    }

    /// Add an exclude pattern
    #[must_use]
    pub fn exclude_pattern(mut self, pattern: impl Into<String>) -> Self {
        self.exclude_patterns.push(pattern.into());
        self
    }

    /// Add multiple exclude patterns
    #[must_use]
    pub fn exclude_patterns(mut self, patterns: Vec<String>) -> Self {
        self.exclude_patterns.extend(patterns);
        self
    }

    /// Set concurrency level
    #[must_use]
    pub fn concurrency(mut self, level: usize) -> Self {
        self.concurrency = level;
        self
    }

    /// Set delay between requests in milliseconds
    #[must_use]
    pub fn delay_ms(mut self, ms: u64) -> Self {
        self.delay_ms = ms;
        self
    }

    /// Set user agent string
    #[must_use]
    pub fn user_agent(mut self, ua: impl Into<String>) -> Self {
        self.user_agent = ua.into();
        self
    }

    /// Set request timeout in seconds
    #[must_use]
    pub fn timeout_secs(mut self, secs: u64) -> Self {
        self.timeout_secs = secs;
        self
    }

    /// Build the configuration
    #[must_use]
    pub fn build(self) -> CrawlerConfig {
        CrawlerConfig {
            seed_url: self.seed_url,
            max_depth: self.max_depth,
            max_pages: self.max_pages,
            include_patterns: self.include_patterns,
            exclude_patterns: self.exclude_patterns,
            concurrency: self.concurrency,
            delay_ms: self.delay_ms,
            user_agent: self.user_agent,
            timeout_secs: self.timeout_secs,
        }
    }
}

/// Crawl result containing discovered URLs
///
/// Following **api-must-use** and **api-non-exhaustive**.
#[derive(Debug, Clone, Default)]
#[must_use]
#[non_exhaustive]
pub struct CrawlResult {
    /// All discovered URLs
    pub urls: Vec<DiscoveredUrl>,
    /// Total number of pages crawled
    pub total_pages: usize,
    /// Number of errors encountered
    pub errors: usize,
}

impl CrawlResult {
    /// Create a new crawl result
    #[must_use]
    pub fn new(urls: Vec<DiscoveredUrl>, total_pages: usize, errors: usize) -> Self {
        Self {
            urls,
            total_pages,
            errors,
        }
    }

    /// Create an empty crawl result
    #[must_use]
    pub fn empty() -> Self {
        Self::default()
    }

    /// Check if the result is empty
    #[inline]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.urls.is_empty()
    }
}

/// Crawl errors
///
/// Following **err-thiserror-for-libraries**: Uses thiserror for library error types.
/// Following **api-non-exhaustive**: Can add variants without breaking changes.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum CrawlError {
    /// Network error during HTTP request
    #[error("network error: {0}")]
    Network(#[from] reqwest::Error),

    /// URL parsing error
    #[error("invalid URL: {0}")]
    InvalidUrl(String),

    /// HTML parsing error
    #[error("parse error: {0}")]
    Parse(String),

    /// Rate limit exceeded
    #[error("rate limit exceeded")]
    RateLimit,

    /// Maximum depth exceeded
    #[error("maximum depth {max} exceeded at depth {current}")]
    MaxDepthExceeded { current: u8, max: u8 },

    /// Maximum pages exceeded
    #[error("maximum pages {max} exceeded")]
    MaxPagesExceeded { max: usize },

    /// URL excluded by pattern
    #[error("URL excluded: {0}")]
    UrlExcluded(String),

    /// Invalid content type
    #[error("invalid content type: {0}")]
    InvalidContentType(String),

    /// General anyhow error for wrapped errors
    #[error("internal error: {0}")]
    Internal(#[from] anyhow::Error),
}

/// Pattern matching helper function
///
/// Following **own-borrow-over-clone**: Accepts &str not &String.
/// Following **opt-inline**: Inlined for hot path performance.
#[inline]
#[must_use]
pub fn matches_pattern(url: &str, pattern: &str) -> bool {
    // Simple glob-style matching
    // TODO: Consider using globset crate for more complex patterns
    if pattern.is_empty() {
        return true;
    }

    // Handle wildcard patterns
    if pattern == "*" {
        return true;
    }

    // Handle patterns with both prefix and suffix wildcards: *.example.com/*
    if pattern.starts_with("*.") && pattern.ends_with("*") {
        // Extract middle part: *.example.com/* -> example.com/
        let middle = &pattern[2..pattern.len() - 1];
        return url.contains(middle);
    }

    // Handle patterns starting with */ : */admin/*
    if pattern.starts_with("*/") && pattern.ends_with("*") {
        // Extract middle part: */admin/* -> admin/
        let middle = &pattern[2..pattern.len() - 1];
        return url.contains(middle);
    }

    // Handle prefix wildcards: *.example.com
    if pattern.starts_with("*.") {
        let domain = &pattern[2..];
        return url.contains(domain);
    }

    // Handle suffix wildcards: https://example.com/admin/*
    if pattern.ends_with("*") {
        let prefix = &pattern[..pattern.len() - 1];
        return url.starts_with(prefix);
    }

    // Handle */prefix patterns: */admin/*
    if pattern.starts_with("*/") {
        let suffix = &pattern[1..];
        return url.ends_with(suffix);
    }

    // Exact match or contains
    url.contains(pattern)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_discovered_url_new() {
        let url = Url::parse("https://example.com/page").unwrap();
        let parent = Url::parse("https://example.com/").unwrap();
        let discovered = DiscoveredUrl::new(url, 1, parent, ContentType::Html);

        assert_eq!(discovered.depth, 1);
        assert_eq!(discovered.content_type, ContentType::Html);
    }

    #[test]
    fn test_discovered_url_html() {
        let url = Url::parse("https://example.com/page").unwrap();
        let parent = Url::parse("https://example.com/").unwrap();
        let discovered = DiscoveredUrl::html(url, 0, parent);

        assert_eq!(discovered.depth, 0);
        assert_eq!(discovered.content_type, ContentType::Html);
    }

    #[test]
    fn test_crawler_config_builder() {
        let seed = Url::parse("https://example.com").unwrap();
        let config = CrawlerConfig::builder(seed)
            .max_depth(5)
            .max_pages(500)
            .concurrency(5)
            .delay_ms(1000)
            .include_pattern("*.example.com/*".to_string())
            .exclude_pattern("*/admin/*".to_string())
            .build();

        assert_eq!(config.max_depth, 5);
        assert_eq!(config.max_pages, 500);
        assert_eq!(config.concurrency, 5);
        assert_eq!(config.delay_ms, 1000);
        assert_eq!(config.include_patterns.len(), 1);
        assert_eq!(config.exclude_patterns.len(), 1);
    }

    #[test]
    fn test_crawler_config_default() {
        let seed = Url::parse("https://example.com").unwrap();
        let config = CrawlerConfig::new(seed);

        assert_eq!(config.max_depth, 3);
        assert_eq!(config.max_pages, 100);
        assert_eq!(config.concurrency, 3);
        assert_eq!(config.delay_ms, 500);
    }

    #[test]
    fn test_crawl_result_empty() {
        let result = CrawlResult::empty();
        assert!(result.is_empty());
        assert_eq!(result.total_pages, 0);
        assert_eq!(result.errors, 0);
    }

    #[test]
    fn test_crawl_result_new() {
        let url = Url::parse("https://example.com").unwrap();
        let parent = Url::parse("https://example.com/").unwrap();
        let discovered = DiscoveredUrl::html(url, 0, parent);
        let result = CrawlResult::new(vec![discovered], 1, 0);

        assert!(!result.is_empty());
        assert_eq!(result.total_pages, 1);
        assert_eq!(result.errors, 0);
        assert_eq!(result.urls.len(), 1);
    }

    #[test]
    fn test_matches_pattern_wildcard() {
        assert!(matches_pattern("https://example.com/page", "*"));
    }

    #[test]
    fn test_matches_pattern_domain_wildcard() {
        assert!(matches_pattern(
            "https://blog.example.com/post",
            "*.example.com/*"
        ));
        assert!(matches_pattern(
            "https://sub.example.com/page",
            "*.example.com"
        ));
    }

    #[test]
    fn test_matches_pattern_prefix_wildcard() {
        assert!(matches_pattern(
            "https://example.com/admin/users",
            "https://example.com/admin/*"
        ));
    }

    #[test]
    fn test_matches_pattern_slash_wildcard() {
        // */admin/* should match any URL containing /admin/
        assert!(matches_pattern(
            "https://example.com/admin/users",
            "*/admin/*"
        ));
        assert!(matches_pattern(
            "https://example.com/admin/settings",
            "*/admin/*"
        ));
    }

    #[test]
    fn test_matches_pattern_exact() {
        assert!(matches_pattern(
            "https://example.com/page",
            "example.com/page"
        ));
    }

    #[test]
    fn test_matches_pattern_empty() {
        assert!(matches_pattern("https://example.com", ""));
    }

    #[test]
    fn test_matches_pattern_no_match() {
        assert!(!matches_pattern("https://other.com/page", "example.com"));
    }

    #[test]
    fn test_crawl_error_display() {
        let error = CrawlError::MaxDepthExceeded { current: 5, max: 3 };
        assert_eq!(format!("{}", error), "maximum depth 3 exceeded at depth 5");

        let error = CrawlError::MaxPagesExceeded { max: 100 };
        assert_eq!(format!("{}", error), "maximum pages 100 exceeded");
    }

    #[test]
    fn test_crawl_error_display_internal() {
        // Test that CrawlError::Internal variant works
        let error = CrawlError::Internal(anyhow::anyhow!("test error"));
        assert!(error.to_string().contains("test error"));
    }
}
