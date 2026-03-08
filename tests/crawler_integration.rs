//! Crawler integration tests
//!
//! Integration tests for the web crawler functionality.
//! These tests require network access and test against real websites.
//!
//! Run with: `cargo test --test crawler_integration -- --ignored`
//! (Tests are ignored by default to avoid network calls in CI)

use rust_scraper::{
    crawl_site, discover_urls, fetch_sitemap, is_allowed, is_excluded, is_internal_link,
    matches_pattern, CrawlerConfig,
};
use url::Url;

/// Test pattern matching functionality
#[test]
fn test_matches_pattern_wildcard() {
    assert!(matches_pattern("https://example.com/page", "*"));
    assert!(matches_pattern("https://any.domain/any/path", "*"));
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
    assert!(!matches_pattern("https://other.com/page", "*.example.com"));
}

#[test]
fn test_matches_pattern_prefix_wildcard() {
    assert!(matches_pattern(
        "https://example.com/admin/users",
        "https://example.com/admin/*"
    ));
    assert!(matches_pattern(
        "https://example.com/admin/settings",
        "https://example.com/admin/*"
    ));
    assert!(!matches_pattern(
        "https://example.com/public/page",
        "https://example.com/admin/*"
    ));
}

#[test]
fn test_is_excluded() {
    let patterns = vec![
        "*/admin/*".to_string(),
        "*/private/*".to_string(),
        "*.example.com/login".to_string(),
    ];

    assert!(is_excluded("https://example.com/admin/users", &patterns));
    assert!(is_excluded("https://example.com/private/data", &patterns));
    assert!(is_excluded("https://blog.example.com/login", &patterns));
    assert!(!is_excluded("https://example.com/public/page", &patterns));
}

#[test]
fn test_is_internal_link() {
    assert!(is_internal_link("https://example.com/page", "example.com"));
    assert!(is_internal_link(
        "https://www.example.com/page",
        "example.com"
    ));
    assert!(is_internal_link(
        "https://blog.example.com/post",
        "example.com"
    ));
    assert!(!is_internal_link("https://other.com/page", "example.com"));
}

/// Test crawl with a real small website
///
/// This test is ignored by default to avoid network calls.
/// Run with: `cargo test --test crawler_integration -- --ignored`
#[tokio::test]
#[ignore]
async fn test_crawl_site_small() {
    let seed = Url::parse("https://example.com").unwrap();
    let config = CrawlerConfig::builder(seed)
        .max_depth(1)
        .max_pages(5)
        .delay_ms(500)
        .build();

    let result = crawl_site(config).await.unwrap();

    assert!(result.total_pages >= 1);
    assert!(!result.urls.is_empty());
    println!("Crawled {} pages", result.total_pages);
}

/// Test URL discovery
///
/// This test is ignored by default to avoid network calls.
#[tokio::test]
#[ignore]
async fn test_discover_urls() {
    let seed = Url::parse("https://example.com").unwrap();
    let config = CrawlerConfig::new(seed);

    let urls = discover_urls("https://example.com", 0, &config)
        .await
        .unwrap();

    // example.com should have at least some links
    assert!(!urls.is_empty());
    println!("Discovered {} URLs", urls.len());
}

/// Test sitemap fetching
///
/// This test is ignored by default to avoid network calls.
#[tokio::test]
#[ignore]
async fn test_fetch_sitemap() {
    // Try with a site known to have a sitemap
    let urls = fetch_sitemap("https://example.com").await.unwrap();

    // May be empty if no sitemap exists
    println!("Found {} URLs in sitemap", urls.len());
}

/// Test URL filtering with config
#[test]
fn test_is_allowed_complex() {
    let seed = Url::parse("https://example.com").unwrap();

    // Config with include and exclude patterns
    let config = CrawlerConfig::builder(seed)
        .include_pattern("*.example.com/blog/*".to_string())
        .include_pattern("*.example.com/docs/*".to_string())
        .exclude_pattern("*/draft/*".to_string())
        .build();

    // Allowed: matches blog include
    assert!(is_allowed("https://example.com/blog/post-1", &config));
    assert!(is_allowed("https://blog.example.com/blog/post-1", &config));

    // Allowed: matches docs include
    assert!(is_allowed("https://example.com/docs/guide", &config));

    // Denied: matches draft exclude
    assert!(!is_allowed("https://example.com/blog/draft/post", &config));
    assert!(!is_allowed("https://example.com/docs/draft/guide", &config));

    // Denied: doesn't match any include
    assert!(!is_allowed("https://example.com/shop/products", &config));
    assert!(!is_allowed("https://example.com/admin/users", &config));
}

/// Test crawler config builder
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
        .user_agent("test-crawler/1.0")
        .timeout_secs(60)
        .build();

    assert_eq!(config.max_depth, 5);
    assert_eq!(config.max_pages, 500);
    assert_eq!(config.concurrency, 5);
    assert_eq!(config.delay_ms, 1000);
    assert_eq!(config.include_patterns.len(), 1);
    assert_eq!(config.exclude_patterns.len(), 1);
    assert_eq!(config.user_agent, "test-crawler/1.0");
    assert_eq!(config.timeout_secs, 60);
}

/// Test crawler config default values
#[test]
fn test_crawler_config_defaults() {
    let seed = Url::parse("https://example.com").unwrap();
    let config = CrawlerConfig::new(seed);

    assert_eq!(config.max_depth, 3);
    assert_eq!(config.max_pages, 100);
    assert_eq!(config.concurrency, 3); // Hardware-aware default
    assert_eq!(config.delay_ms, 500); // Hardware-aware default
}
