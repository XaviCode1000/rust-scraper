//! Repository interfaces for domain data persistence
//!
//! Defines contracts for storing and retrieving domain entities.
//! Infrastructure layer implements these traits.

use crate::domain::{CrawlError, CrawlResult, ScrapedContent};

/// Repository interface for crawl results
///
/// Defines the contract for persisting and retrieving crawl data.
/// Implementations can use files, databases, or other storage backends.
pub trait CrawlResultRepository {
    /// Save a crawl result
    ///
    /// # Arguments
    ///
    /// * `result` - The crawl result to persist
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Success
    /// * `Err(CrawlError)` - Persistence error
    fn save(&self, result: &CrawlResult) -> Result<(), CrawlError>;

    /// Find scraped content by URL
    ///
    /// # Arguments
    ///
    /// * `url` - The URL to search for
    ///
    /// # Returns
    ///
    /// * `Ok(Some(content))` - Found content
    /// * `Ok(None)` - Not found
    /// * `Err(CrawlError)` - Query error
    fn find_by_url(&self, url: &str) -> Result<Option<ScrapedContent>, CrawlError>;

    /// Get all crawled URLs
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<String>)` - List of crawled URLs
    /// * `Err(CrawlError)` - Query error
    fn get_all_urls(&self) -> Result<Vec<String>, CrawlError>;
}
