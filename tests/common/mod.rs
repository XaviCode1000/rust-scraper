//! Shared test fixtures and helpers
//!
//! Common utilities for integration tests:
//! - HTML fixture loading
//! - WireMock server setup
//! - Test content generators
//!
//! # Usage
//!
//! ```ignore
//! mod common;
//! use common::{load_fixture, mock_server};
//! ```

use std::path::Path;

/// Load an HTML fixture from the tests/fixtures/ directory.
///
/// # Panics
///
/// Panics if the fixture file cannot be read.
pub fn load_fixture(name: &str) -> String {
    let fixture_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(name);
    std::fs::read_to_string(&fixture_path).unwrap_or_else(|e| {
        panic!(
            "Failed to load fixture {}: {}",
            fixture_path.display(),
            e
        )
    })
}

/// Get the path to the fixtures directory.
pub fn fixtures_dir() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
}

/// Create a minimal ScrapedContent for testing.
pub fn mock_scraped_content(url: &str, title: &str, content: &str) -> rust_scraper::ScrapedContent {
    rust_scraper::ScrapedContent {
        title: title.to_string(),
        content: content.to_string(),
        url: rust_scraper::ValidUrl::parse(url).expect("valid test URL"),
        excerpt: None,
        author: None,
        date: None,
        html: None,
        assets: Vec::new(),
    }
}

/// Create a ScrapedContent with raw HTML included.
pub fn mock_scraped_content_with_html(
    url: &str,
    title: &str,
    content: &str,
    html: &str,
) -> rust_scraper::ScrapedContent {
    rust_scraper::ScrapedContent {
        title: title.to_string(),
        content: content.to_string(),
        url: rust_scraper::ValidUrl::parse(url).expect("valid test URL"),
        excerpt: None,
        author: None,
        date: None,
        html: Some(html.to_string()),
        assets: Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_fixture_static_page() {
        let html = load_fixture("static_page.html");
        assert!(html.contains("<title>Test Page"));
        assert!(html.contains("Sample Article Title"));
    }

    #[test]
    fn test_fixtures_dir_exists() {
        let dir = fixtures_dir();
        assert!(dir.exists());
        assert!(dir.is_dir());
    }

    #[test]
    fn test_mock_scraped_content() {
        let content = mock_scraped_content(
            "https://example.com/test",
            "Test Title",
            "Test content",
        );
        assert_eq!(content.title, "Test Title");
        assert_eq!(content.content, "Test content");
        assert_eq!(content.url.as_str(), "https://example.com/test");
    }
}
