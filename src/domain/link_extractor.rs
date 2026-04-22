//! Link extraction domain interface
//!
//! Defines the contract for extracting links from HTML content.
//! Infrastructure layer implements this trait.

use crate::domain::CrawlError;

/// Domain interface for link extraction
///
/// This trait defines the contract for extracting and normalizing
/// links from HTML content. The infrastructure layer provides
/// the implementation using external libraries like scraper.
pub trait LinkExtractor {
    /// Extract all links from HTML content
    ///
    /// # Arguments
    ///
    /// * `html` - HTML content to parse
    /// * `base_url` - Base URL for resolving relative links
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<String>)` - List of extracted, normalized URLs
    /// * `Err(CrawlError)` - Parse or processing error
    fn extract_links(&self, html: &str, base_url: &str) -> Result<Vec<String>, CrawlError>;
}

/// Domain service for link processing logic
///
/// Contains pure functions for link normalization and validation
/// that don't depend on external libraries.
pub struct LinkProcessor;

impl LinkProcessor {
    /// Check if a URL is internal (same domain)
    ///
    /// Pure function for domain checking logic.
    pub fn is_internal_link(url: &str, domain: &str) -> bool {
        Self::extract_domain(url)
            .map(|url_domain| url_domain == domain || url_domain.ends_with(&format!(".{}", domain)))
            .unwrap_or(false)
    }

    /// Normalize a URL (remove fragments, trailing slashes, etc.)
    ///
    /// Pure function for URL normalization.
    pub fn normalize_url(url: &str) -> String {
        // Remove fragment
        let without_fragment = url.split('#').next().unwrap_or(url);

        // Parse and rebuild URL for consistent normalization
        if let Ok(parsed) = url::Url::parse(without_fragment) {
            // Keep trailing slash if present, remove query params for deduplication
            let mut normalized = parsed[..url::Position::AfterPath].to_string();

            // Preserve trailing slash
            if without_fragment.ends_with('/') && !normalized.ends_with('/') {
                normalized.push('/');
            }

            normalized
        } else {
            without_fragment.to_string()
        }
    }

    /// Extract domain from URL
    ///
    /// Pure function for domain extraction.
    fn extract_domain(url: &str) -> Option<&str> {
        url.split("://")
            .nth(1)
            .and_then(|rest| rest.split('/').next())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_link_processor_is_internal_link() {
        assert!(LinkProcessor::is_internal_link(
            "https://example.com/page",
            "example.com"
        ));
        assert!(LinkProcessor::is_internal_link(
            "https://www.example.com/page",
            "example.com"
        ));
        assert!(LinkProcessor::is_internal_link(
            "https://blog.example.com/post",
            "example.com"
        ));
        assert!(!LinkProcessor::is_internal_link(
            "https://other.com/page",
            "example.com"
        ));
        assert!(!LinkProcessor::is_internal_link(
            "invalid-url",
            "example.com"
        ));
    }

    #[test]
    fn test_link_processor_normalize_url_remove_fragment() {
        assert_eq!(
            LinkProcessor::normalize_url("https://example.com/page#section"),
            "https://example.com/page"
        );
        assert_eq!(
            LinkProcessor::normalize_url("https://example.com/page#top"),
            "https://example.com/page"
        );
    }

    #[test]
    fn test_link_processor_normalize_url_preserve_trailing_slash() {
        assert_eq!(
            LinkProcessor::normalize_url("https://example.com/page/"),
            "https://example.com/page/"
        );
        assert_eq!(
            LinkProcessor::normalize_url("https://example.com/page/#section"),
            "https://example.com/page/"
        );
    }

    #[test]
    fn test_link_processor_normalize_url_no_change() {
        assert_eq!(
            LinkProcessor::normalize_url("https://example.com/page"),
            "https://example.com/page"
        );
    }

    #[test]
    fn test_link_processor_normalize_url_invalid() {
        let result = LinkProcessor::normalize_url("not-a-valid-url");
        assert_eq!(result, "not-a-valid-url");
    }
}
