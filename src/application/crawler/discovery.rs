//! URL Discovery module - Extracted from crawler_service.rs
//!
//! Handles:
//! - Sitemap parsing
//! - Link extraction from HTML
//! - URL normalization

use crate::domain::{ContentType, CrawlerConfig, DiscoveredUrl};
use crate::infrastructure::crawler::sitemap_parser::SitemapParser;
use anyhow::{Context, Result};
use scraper::{Html, Selector};
use std::sync::Arc;
use tracing::{debug, info};
use url::Url;

/// Discover URLs from a website using sitemap (preferred)
///
/// # Arguments
///
/// * `base_url` - Base URL of the website
/// * `sitemap_url` - Optional explicit sitemap URL
/// * `config` - Crawler configuration
///
/// # Returns
///
/// * `Ok(Vec<DiscoveredUrl>)` - URLs discovered from sitemap
pub async fn crawl_with_sitemap(
    base_url: &str,
    sitemap_url: Option<&str>,
    config: &CrawlerConfig,
) -> Result<Vec<DiscoveredUrl>> {
    info!("Crawling {} via sitemap", base_url);

    let sitemap_url = sitemap_url
        .map(|s| s.to_string())
        .unwrap_or_else(|| format!("{}/sitemap.xml", base_url.trim_end_matches('/')));

    let parser = SitemapParser::new();
    let urls = parser
        .parse(&sitemap_url)
        .await
        .context("Failed to parse sitemap")?;

    let base = Url::parse(base_url).context("Invalid base URL")?;
    let depth = 0u8;

    let mut discovered = Vec::with_capacity(urls.len());
    for url_str in urls {
        if let Ok(url) = Url::parse(&url_str) {
            // Filter by include/exclude patterns
            let url_str = url.to_string();
            let allowed = config.include_patterns.is_empty()
                || config.include_patterns.iter().any(|p| url_str.contains(p));

            let excluded = config.exclude_patterns.iter().any(|p| url_str.contains(p));

            if allowed && !excluded {
                discovered.push(DiscoveredUrl::xml(url, depth, base.clone()));
            }
        }
    }

    info!("Found {} URLs from sitemap", discovered.len());
    Ok(discovered)
}

/// Fetch and parse a sitemap
pub async fn fetch_sitemap(base_url: &str) -> Result<Vec<String>> {
    let sitemap_url = format!("{}/sitemap.xml", base_url.trim_end_matches('/'));
    let parser = SitemapParser::new();
    parser.parse(&sitemap_url).await
}

/// Extract links from HTML content
pub fn extract_links(html: &str, base_url: &str) -> Result<Vec<String>> {
    let document = Html::parse_document(html);
    let selector = Selector::parse("a[href]").map_err(|e| anyhow::anyhow!("Invalid selector: {}", e))?;

    let mut links = Vec::new();
    for element in document.select(&selector) {
        if let Some(href) = element.value().attr("href") {
            if !href.is_empty() && !href.starts_with('#') && !href.starts_with("javascript:") {
                links.push(href.to_string());
            }
        }
    }

    debug!("Extracted {} links from {}", links.len(), base_url);
    Ok(links)
}

/// Normalize a URL (resolve relative, remove fragments)
pub fn normalize_url(url: &str) -> String {
    // Remove fragments
    if let Some(pos) = url.find('#') {
        return url[..pos].to_string();
    }
    url.to_string()
}

/// Check if URL is internal to the seed domain
pub fn is_internal_link(url: &str, seed_domain: &str) -> bool {
    if let Ok(parsed) = Url::parse(url) {
        if let Some(domain) = parsed.host_str() {
            return domain == seed_domain || domain.ends_with(&format!(".{}", seed_domain));
        }
    }
    false
}

/// Check if URL is allowed by filters
pub fn is_allowed(url: &str, config: &CrawlerConfig) -> bool {
    let included = config.include_patterns.is_empty()
        || config.include_patterns.iter().any(|p| url.contains(p));

    let excluded = config.exclude_patterns.iter().any(|p| url.contains(p));

    included && !excluded
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_url() {
        assert_eq!(normalize_url("http://example.com/page#section"), "http://example.com/page");
        assert_eq!(normalize_url("http://example.com/page"), "http://example.com/page");
    }

    #[test]
    fn test_is_internal_link() {
        assert!(is_internal_link("https://example.com/page", "example.com"));
        assert!(is_internal_link("https://www.example.com/page", "example.com"));
        assert!(!is_internal_link("https://other.com/page", "example.com"));
    }
}