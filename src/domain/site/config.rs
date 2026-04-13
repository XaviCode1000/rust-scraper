//! Site configuration with builder pattern
//!
//! Configuration for crawling a specific site.

use url::Url;

use crate::domain::pattern_matching::matches_pattern;

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
    /// Use sitemap for URL discovery (FASE 3)
    pub use_sitemap: bool,
    /// Explicit sitemap URL (auto-discovers if None)
    pub sitemap_url: Option<String>,
}

impl CrawlerConfig {
    /// Create a new config with seed URL
    ///
    /// Following **api-builder**: Returns builder for fluent configuration.
    pub fn builder(seed_url: Url) -> CrawlerConfigBuilder {
        CrawlerConfigBuilder::new(seed_url)
    }

    /// Create a new config with default values
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
            use_sitemap: false,
            sitemap_url: None,
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
    use_sitemap: bool,
    sitemap_url: Option<String>,
}

impl CrawlerConfigBuilder {
    /// Create a new builder with seed URL
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
            use_sitemap: false,
            sitemap_url: None,
        }
    }

    /// Set maximum crawl depth
    pub fn max_depth(mut self, depth: u8) -> Self {
        self.max_depth = depth;
        self
    }

    /// Set maximum number of pages
    pub fn max_pages(mut self, pages: usize) -> Self {
        self.max_pages = pages;
        self
    }

    /// Add an include pattern
    pub fn include_pattern(mut self, pattern: impl Into<String>) -> Self {
        self.include_patterns.push(pattern.into());
        self
    }

    /// Add multiple include patterns
    pub fn include_patterns(mut self, patterns: Vec<String>) -> Self {
        self.include_patterns.extend(patterns);
        self
    }

    /// Add an exclude pattern
    pub fn exclude_pattern(mut self, pattern: impl Into<String>) -> Self {
        self.exclude_patterns.push(pattern.into());
        self
    }

    /// Add multiple exclude patterns
    pub fn exclude_patterns(mut self, patterns: Vec<String>) -> Self {
        self.exclude_patterns.extend(patterns);
        self
    }

    /// Set concurrency level
    pub fn concurrency(mut self, level: usize) -> Self {
        self.concurrency = level;
        self
    }

    /// Set delay between requests in milliseconds
    pub fn delay_ms(mut self, ms: u64) -> Self {
        self.delay_ms = ms;
        self
    }

    /// Set user agent string
    pub fn user_agent(mut self, ua: impl Into<String>) -> Self {
        self.user_agent = ua.into();
        self
    }

    /// Set request timeout in seconds
    pub fn timeout_secs(mut self, secs: u64) -> Self {
        self.timeout_secs = secs;
        self
    }

    /// Set use_sitemap flag (FASE 3)
    pub fn use_sitemap(mut self, use_sitemap: bool) -> Self {
        self.use_sitemap = use_sitemap;
        self
    }

    /// Set explicit sitemap URL (FASE 3)
    pub fn sitemap_url(mut self, url: impl Into<String>) -> Self {
        self.sitemap_url = Some(url.into());
        self
    }

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
            use_sitemap: self.use_sitemap,
            sitemap_url: self.sitemap_url,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
