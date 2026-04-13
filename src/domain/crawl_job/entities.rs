//! Crawl job entities
//!
//! Core entities representing URLs discovered during crawling.

use url::Url;

/// Content type discovered during crawling
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ContentType {
    /// HTML page
    #[default]
    Html,
    /// XML document (including sitemaps)
    Xml,
    /// Plain text
    Text,
    /// Unknown or other content type
    Other,
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
}
