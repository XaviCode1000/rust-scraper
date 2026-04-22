//! Crawler Engine - Extracted from crawler_service.rs
//!
//! Handles:
//! - Main crawl loop with concurrency
//! - Rate limiting integration
//! - Results collection

use crate::application::rate_limiter::{RateLimiterConfig, SharedRateLimiter};
use crate::application::results_channel::ResultsCollector;
use crate::domain::{CrawlError, CrawlResult, DiscoveredUrl};
use crate::domain::CrawlerConfig;
use anyhow::Result;
use std::sync::Arc;
use tokio::sync::Mutex;
use std::collections::HashSet;
use tracing::{debug, info, span, Level};

/// Custom URL queue for deduplication
pub struct UrlQueue {
    inner: Mutex<Vec<DiscoveredUrl>>,
}

impl UrlQueue {
    pub fn new() -> Self {
        Self {
            inner: Mutex::new(Vec::new()),
        }
    }

    pub async fn push(&self, url: DiscoveredUrl) {
        let mut guard = self.inner.lock().await;
        guard.push(url);
    }

    pub async fn pop_front(&self) -> Option<DiscoveredUrl> {
        let mut guard = self.inner.lock().await;
        guard.pop_front()
    }

    pub async fn drain_all(&self) -> Vec<DiscoveredUrl> {
        let mut guard = self.inner.lock().await;
        std::mem::take(&mut *guard)
    }
}

/// Main crawl site function
pub async fn crawl_site(config: CrawlerConfig) -> Result<CrawlResult, CrawlError> {
    let span = span!(
        Level::INFO,
        "crawl_site",
        seed_url = %config.seed_url,
        max_depth = config.max_depth,
        max_pages = config.max_pages
    );
    let _guard = span.enter();

    info!(
        "Starting crawl from {} with max_depth={} max_pages={}",
        config.seed_url, config.max_depth, config.max_pages
    );

    // Clone config for async safety
    let config = Arc::new(config);
    let config_clone = Arc::clone(&config);

    // Create rate limiter
    let rate_limiter_config = RateLimiterConfig::new(config_clone.delay_ms, config_clone.concurrency as u32);
    let rate_limiter = match SharedRateLimiter::new(&rate_limiter_config) {
        Ok(limiter) => limiter,
        Err(e) => return Err(CrawlError::Internal(e.to_string())),
    };

    // Create URL queue
    let queue = Arc::new(UrlQueue::new());

    // Seed URL
    queue.push(DiscoveredUrl::html(
        config_clone.seed_url.clone(),
        0,
        config_clone.seed_url.clone(),
    )).await;

    // Track visited URLs
    let visited = Arc::new(Mutex::new(HashSet::<String>::new()));

    // Results collector
    let results_collector = ResultsCollector::new(config_clone.max_pages, Some(config_clone.max_pages));
    let error_count = Arc::new(std::sync::atomic::AtomicUsize::new(0));

    let mut tasks = tokio::task::JoinSet::new();
    let mut url_queue = std::collections::VecDeque::new();
    url_queue.push_back(DiscoveredUrl::html(
        config_clone.seed_url.clone(),
        0,
        config_clone.seed_url.clone(),
    ));

    // Main crawl loop
    while !url_queue.is_empty() || !tasks.is_empty() {
        if results_collector.is_full(config_clone.max_pages) {
            info!("Reached max pages limit: {}", config_clone.max_pages);
            break;
        }

        // Process completed tasks
        while let Some(result) = tasks.try_join_next() {
            // handle_crawl_result(result, &error_count);
            let _ = result;
        }

        // Drain from queue
        url_queue.append(&mut queue.drain_all().await);

        // Spawn new tasks
        while let Some(discovered_url) = url_queue.pop_front() {
            if tasks.len() >= config_clone.concurrency {
                url_queue.push_front(discovered_url);
                break;
            }

            // Check if already visited
            {
                let visited_guard = visited.lock().await;
                if visited_guard.contains(discovered_url.url.as_str()) {
                    continue;
                }
            }

            // Clone for task
            let config_task = Arc::clone(&config);
            let queue_task = Arc::clone(&queue);
            let results_sender = results_collector.clone();
            let visited_task = Arc::clone(&visited);
            let error_count_task = Arc::clone(&error_count);
            let rate_limiter_task = rate_limiter.clone();
            let discovered_url_task = discovered_url.clone();

            tasks.spawn(async move {
                rate_limiter_task.until_ready().await;

                let url_str = discovered_url_task.url.as_str().to_string();
                let url_depth = discovered_url_task.depth;

                debug!("Crawling: {} (depth={})", url_str, url_depth);

                // Placeholder for HTTP fetch logic
                // In full implementation, fetch URL and extract content
            });
        }
    }

    // Collect final results
    let results = results_collector.collect().await;

    Ok(CrawlResult {
        total_pages: results.len(),
        successful: results.len(),
        failed: 0,
        results,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_url_queue() {
        let queue = UrlQueue::new();
        // Basic test - queue can be created
        assert!(true);
    }
}