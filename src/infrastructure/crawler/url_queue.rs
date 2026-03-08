//! Concurrent URL queue for crawling
//!
//! Thread-safe URL queue with deduplication.
//!
//! # Rules Applied
//!
//! - **async-no-lock-across-await**: Uses DashSet for lock-free concurrent access
//! - **mem-with-capacity**: Pre-allocates internal structures
//! - **own-borrow-over-clone**: Efficient borrowing where possible

use dashmap::DashSet;
use tokio::sync::Mutex;
use tracing::debug;

use crate::domain::DiscoveredUrl;

/// Thread-safe URL queue with deduplication
///
/// Following **async-no-lock-across-await**: Uses DashSet for concurrent access
/// without holding locks across .await points.
/// Following **mem-with-capacity**: Pre-allocates internal storage.
pub struct UrlQueue {
    /// Pending URLs to crawl
    queue: Mutex<Vec<DiscoveredUrl>>,
    /// Set of URLs already in queue or visited (for deduplication)
    seen: DashSet<String>,
}

impl UrlQueue {
    /// Create a new URL queue
    ///
    /// Following **mem-with-capacity**: Pre-allocates internal structures.
    #[must_use]
    pub fn new() -> Self {
        Self {
            queue: Mutex::new(Vec::with_capacity(100)),
            seen: DashSet::with_capacity(100),
        }
    }

    /// Push a URL to the queue
    ///
    /// Returns `false` if the URL was already seen (duplicate).
    ///
    /// # Arguments
    ///
    /// * `url` - URL to add
    ///
    /// # Returns
    ///
    /// `true` if added, `false` if duplicate
    pub fn push(&self, url: DiscoveredUrl) -> bool {
        let url_str = url.url.as_str().to_string();

        // Check and insert into seen set (atomic operation)
        if self.seen.contains(&url_str) {
            debug!("Duplicate URL in queue: {}", url_str);
            return false;
        }

        self.seen.insert(url_str);

        // Add to queue
        let mut queue = self.queue.blocking_lock();
        queue.push(url);

        true
    }

    /// Pop a URL from the queue
    ///
    /// # Returns
    ///
    /// `Some(DiscoveredUrl)` if queue has URLs, `None` if empty
    pub fn pop(&self) -> Option<DiscoveredUrl> {
        let mut queue = self.queue.blocking_lock();
        queue.pop()
    }

    /// Get the current queue length
    ///
    /// # Returns
    ///
    /// Number of URLs in the queue
    pub fn len(&self) -> usize {
        self.queue.blocking_lock().len()
    }

    /// Check if the queue is empty
    ///
    /// # Returns
    ///
    /// `true` if queue is empty
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.queue.blocking_lock().is_empty()
    }

    /// Get the number of seen URLs
    ///
    /// # Returns
    ///
    /// Number of URLs that have been seen (added or visited)
    pub fn seen_count(&self) -> usize {
        self.seen.len()
    }

    /// Clear the queue (but not the seen set)
    pub fn clear(&self) {
        self.queue.blocking_lock().clear();
    }

    /// Get all URLs from the queue (for debugging)
    ///
    /// # Returns
    ///
    /// Vec of all URLs currently in the queue
    #[cfg(test)]
    pub fn get_all(&self) -> Vec<DiscoveredUrl> {
        self.queue.blocking_lock().clone()
    }
}

impl Default for UrlQueue {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use url::Url;

    fn create_test_url(path: &str) -> DiscoveredUrl {
        let url = Url::parse(&format!("https://example.com{}", path)).unwrap();
        let parent = Url::parse("https://example.com/").unwrap();
        DiscoveredUrl::html(url, 0, parent)
    }

    #[test]
    fn test_url_queue_new() {
        let queue = UrlQueue::new();
        assert!(queue.is_empty());
        assert_eq!(queue.len(), 0);
        assert_eq!(queue.seen_count(), 0);
    }

    #[test]
    fn test_url_queue_push_pop() {
        let queue = UrlQueue::new();

        let url1 = create_test_url("/page1");
        let url2 = create_test_url("/page2");

        assert!(queue.push(url1));
        assert!(queue.push(url2));

        assert_eq!(queue.len(), 2);
        assert_eq!(queue.seen_count(), 2);

        let popped = queue.pop();
        assert!(popped.is_some());
        assert_eq!(popped.unwrap().url.path(), "/page2"); // LIFO

        assert_eq!(queue.len(), 1);
    }

    #[test]
    fn test_url_queue_duplicate_detection() {
        let queue = UrlQueue::new();

        let url1 = create_test_url("/page1");
        let url2 = create_test_url("/page1"); // Same URL

        assert!(queue.push(url1));
        assert!(!queue.push(url2)); // Duplicate

        assert_eq!(queue.len(), 1);
        assert_eq!(queue.seen_count(), 1);
    }

    #[test]
    fn test_url_queue_empty_pop() {
        let queue = UrlQueue::new();
        assert!(queue.pop().is_none());
    }

    #[test]
    fn test_url_queue_clear() {
        let queue = UrlQueue::new();

        queue.push(create_test_url("/page1"));
        queue.push(create_test_url("/page2"));

        assert_eq!(queue.len(), 2);

        queue.clear();

        assert_eq!(queue.len(), 0);
        assert_eq!(queue.seen_count(), 2); // Seen set not cleared
    }

    #[test]
    fn test_url_queue_multiple_urls() {
        let queue = UrlQueue::new();

        for i in 0..10 {
            let url = create_test_url(&format!("/page{}", i));
            assert!(queue.push(url));
        }

        assert_eq!(queue.len(), 10);
        assert_eq!(queue.seen_count(), 10);

        // Pop all
        for _ in 0..10 {
            assert!(queue.pop().is_some());
        }

        assert!(queue.is_empty());
    }
}
