//! Crawler service module
//!
//! Main crawling orchestration logic.
//!
//! # Rules Applied
//!
//! - **async-no-lock-across-await**: Uses Semaphore for concurrency control
//!   instead of Mutex to avoid holding locks across .await.
//! - **async-clone-before-await**: Clones config before async operations.
//! - **err-anyhow-for-apps**: Result with anyhow for application layer
//! - **own-borrow-over-clone**: Accept `&str` not `&String`
//! - **mem-with-capacity**: Vec::with_capacity when size is known

use std::collections::HashSet;
use std::num::NonZeroU32;
use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use governor::{Quota, RateLimiter};
use tokio::sync::Semaphore;
use tracing::{debug, error, info, warn};
use url::Url;

use crate::domain::{CrawlError, CrawlResult, CrawlerConfig, DiscoveredUrl};

use super::url_filter::is_allowed;
use crate::infrastructure::crawler::{
    extract_links, fetch_url, is_internal_link, normalize_url, UrlQueue,
};

/// Crawl a website starting from the seed URL
///
/// Following **async-no-lock-across-await**: Uses Semaphore for concurrency control
/// instead of Mutex to avoid holding locks across .await.
/// Following **async-clone-before-await**: Clones config before async operations.
///
/// # Arguments
///
/// * `config` - Crawler configuration
///
/// # Returns
///
/// * `Ok(CrawlResult)` - Crawl result with discovered URLs
/// * `Err(CrawlError)` - Error during crawling
///
/// # Examples
///
/// ```no_run
/// use rust_scraper::{domain::CrawlerConfig, application::crawl_site};
/// use url::Url;
///
/// # #[tokio::main]
/// # async fn main() -> anyhow::Result<()> {
/// let seed = Url::parse("https://example.com")?;
/// let config = CrawlerConfig::builder(seed)
///     .max_depth(2)
///     .max_pages(50)
///     .build();
///
/// let result = crawl_site(config).await?;
/// println!("Crawled {} pages", result.total_pages);
/// # Ok(())
/// # }
/// ```
pub async fn crawl_site(config: CrawlerConfig) -> Result<CrawlResult, CrawlError> {
    info!(
        "Starting crawl from {} with max_depth={} max_pages={}",
        config.seed_url, config.max_depth, config.max_pages
    );

    // Clone config for async safety (following async-clone-before-await)
    let config = Arc::new(config);
    let config_clone = Arc::clone(&config);

    // Create rate limiter (governor) - shared across tasks
    let quota = Quota::with_period(Duration::from_millis(config_clone.delay_ms))
        .unwrap()
        .allow_burst(NonZeroU32::new(config_clone.concurrency as u32).unwrap());
    let rate_limiter = Arc::new(RateLimiter::direct(quota));

    // Create semaphore for concurrency control (avoiding Mutex across await)
    let semaphore = Arc::new(Semaphore::new(config_clone.concurrency));

    // Create URL queue
    let queue = Arc::new(UrlQueue::new());

    // Add seed URL to queue
    let seed_discovered = DiscoveredUrl::html(
        config_clone.seed_url.clone(),
        0,
        config_clone.seed_url.clone(),
    );
    queue.push(seed_discovered);

    // Track visited URLs
    let visited = Arc::new(tokio::sync::Mutex::new(HashSet::<String>::new()));

    // Results collector
    let results = Arc::new(tokio::sync::Mutex::new(Vec::new()));
    let error_count = Arc::new(std::sync::atomic::AtomicUsize::new(0));

    let mut tasks = tokio::task::JoinSet::new();

    // Main crawl loop
    loop {
        // Check if we've reached max pages
        {
            let results_guard = results.lock().await;
            if results_guard.len() >= config_clone.max_pages {
                info!("Reached max pages limit: {}", config_clone.max_pages);
                break;
            }
        }

        // Get next URL from queue
        let Some(discovered_url) = queue.pop() else {
            // Queue is empty, wait for tasks to complete
            debug!("Queue empty, waiting for tasks");
            if tasks.is_empty() {
                break;
            }
            // Wait for at least one task to complete
            if let Some(task_result) = tasks.join_next().await {
                match task_result {
                    Ok(Ok(())) => {}
                    Ok(Err(e)) => {
                        warn!("Task error: {}", e);
                        error_count.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                    }
                    Err(e) => {
                        warn!("Task panicked: {}", e);
                        error_count.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                    }
                }
            }
            continue;
        };

        // Check depth limit
        if discovered_url.depth > config_clone.max_depth {
            debug!(
                "Skipping URL at depth {} (max: {})",
                discovered_url.depth, config_clone.max_depth
            );
            continue;
        }

        // Clone URL string before moving discovered_url
        let url_str = discovered_url.url.as_str().to_string();
        let url_depth = discovered_url.depth;

        // Check if already visited
        {
            let visited_guard = visited.lock().await;
            if visited_guard.contains(&url_str) {
                debug!("Already visited: {}", url_str);
                continue;
            }
            drop(visited_guard);
        }

        // Clone data for task (async-clone-before-await)
        let config_task = Arc::clone(&config);
        let queue_task = Arc::clone(&queue);
        let results_task = Arc::clone(&results);
        let visited_task = Arc::clone(&visited);
        let error_count_task = Arc::clone(&error_count);
        let semaphore_task = Arc::clone(&semaphore);
        let rate_limiter_task = Arc::clone(&rate_limiter);
        let discovered_url_task = discovered_url.clone();
        let parent_url = discovered_url_task.url.clone();

        // Spawn crawl task
        tasks.spawn(async move {
            // Acquire semaphore permit (concurrency control)
            let _permit = semaphore_task
                .acquire()
                .await
                .map_err(|e| CrawlError::Internal(anyhow::anyhow!("Semaphore error: {}", e)))?;

            // Rate limiting
            rate_limiter_task.until_ready().await;

            debug!("Crawling: {} (depth={})", url_str, url_depth);

            // Fetch URL
            match fetch_url(&url_str, &config_task).await {
                Ok(response) => {
                    // Add to results
                    {
                        let mut results_guard = results_task.lock().await;
                        results_guard.push(discovered_url_task);
                    }

                    // Extract links and add to queue
                    if url_depth < config_task.max_depth {
                        match extract_links(&response, &url_str) {
                            Ok(links) => {
                                for link in links {
                                    let normalized = normalize_url(&link);
                                    if let Ok(parsed_url) = Url::parse(&normalized) {
                                        // Check if internal link
                                        if let Some(seed_domain) = config_task.seed_url.host_str() {
                                            if is_internal_link(&normalized, seed_domain) {
                                                // Check if allowed by filters
                                                if is_allowed(&normalized, &config_task) {
                                                    // Check if not visited
                                                    let visited_guard = visited_task.lock().await;
                                                    if !visited_guard.contains(&normalized) {
                                                        drop(visited_guard);

                                                        let new_discovered = DiscoveredUrl::html(
                                                            parsed_url,
                                                            url_depth + 1,
                                                            parent_url.clone(),
                                                        );
                                                        queue_task.push(new_discovered);
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                            Err(e) => {
                                warn!("Failed to extract links from {}: {}", url_str, e);
                                error_count_task.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                            }
                        }
                    }
                }
                Err(e) => {
                    error!("Failed to fetch {}: {}", url_str, e);
                    error_count_task.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                    return Err(e);
                }
            }

            Ok(())
        });

        // Limit concurrent tasks
        if tasks.len() >= config_clone.concurrency {
            if let Some(task_result) = tasks.join_next().await {
                match task_result {
                    Ok(Ok(())) => {}
                    Ok(Err(e)) => {
                        warn!("Task error: {}", e);
                        error_count.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                    }
                    Err(e) => {
                        warn!("Task panicked: {}", e);
                        error_count.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                    }
                }
            }
        }
    }

    // Wait for remaining tasks
    while let Some(task_result) = tasks.join_next().await {
        match task_result {
            Ok(Err(e)) => {
                warn!("Task error: {}", e);
                error_count.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            }
            Err(e) => {
                warn!("Task panicked: {}", e);
                error_count.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            }
            _ => {}
        }
    }

    // Collect results
    let results_guard = results.lock().await;
    let total_pages = results_guard.len();
    let errors = error_count.load(std::sync::atomic::Ordering::SeqCst);

    info!("Crawl complete: {} pages, {} errors", total_pages, errors);

    Ok(CrawlResult::new(results_guard.clone(), total_pages, errors))
}

/// Discover URLs from a single page
///
/// Following **own-borrow-over-clone**: Accepts `&str` not `&String`.
///
/// # Arguments
///
/// * `url` - URL to discover links from
/// * `depth` - Current depth in crawl tree
/// * `config` - Crawler configuration
///
/// # Returns
///
/// * `Ok(Vec<DiscoveredUrl>)` - Discovered URLs
/// * `Err(CrawlError)` - Error during discovery
pub async fn discover_urls(
    url: &str,
    depth: usize,
    config: &CrawlerConfig,
) -> Result<Vec<DiscoveredUrl>, CrawlError> {
    debug!("Discovering URLs from {} at depth {}", url, depth);

    // Clone config for async safety
    let config = Arc::new(config.clone());
    let config_clone = Arc::clone(&config);

    // Fetch URL
    let response = fetch_url(url, &config_clone).await?;

    // Extract links
    let links = extract_links(&response, url)?;

    // Parse and filter URLs
    let base_url = Url::parse(url).map_err(|e| CrawlError::InvalidUrl(e.to_string()))?;
    let mut discovered = Vec::with_capacity(links.len());

    for link in links {
        let normalized = normalize_url(&link);
        if let Ok(parsed_url) = Url::parse(&normalized) {
            // Check if internal link
            if let Some(seed_domain) = config.seed_url.host_str() {
                if is_internal_link(&normalized, seed_domain) {
                    // Check if allowed
                    if is_allowed(&normalized, &config) {
                        discovered.push(DiscoveredUrl::html(
                            parsed_url,
                            depth as u8,
                            base_url.clone(),
                        ));
                    }
                }
            }
        }
    }

    Ok(discovered)
}

/// Fetch and parse a sitemap.xml file
///
/// Following **own-borrow-over-clone**: Accepts `&str`.
///
/// # Arguments
///
/// * `base_url` - Base URL of the website
///
/// # Returns
///
/// * `Ok(Vec<String>)` - List of URLs from sitemap
/// * `Err(CrawlError)` - Error during fetch or parse
///
/// # Examples
///
/// ```no_run
/// use rust_scraper::application::fetch_sitemap;
///
/// # #[tokio::main]
/// # async fn main() -> anyhow::Result<()> {
/// let urls = fetch_sitemap("https://example.com").await?;
/// println!("Found {} URLs in sitemap", urls.len());
/// # Ok(())
/// # }
/// ```
pub async fn fetch_sitemap(base_url: &str) -> Result<Vec<String>, CrawlError> {
    info!("Fetching sitemap from {}", base_url);

    // Try common sitemap locations
    let sitemap_urls = [
        format!("{}/sitemap.xml", base_url.trim_end_matches('/')),
        format!("{}/sitemap_index.xml", base_url.trim_end_matches('/')),
        format!("{}/sitemap.xml.gz", base_url.trim_end_matches('/')),
    ];

    let mut all_urls = Vec::new();

    for sitemap_url in &sitemap_urls {
        debug!("Trying sitemap: {}", sitemap_url);

        // Create minimal config for sitemap fetch
        let seed = Url::parse(base_url).map_err(|e| CrawlError::InvalidUrl(e.to_string()))?;
        let config = Arc::new(CrawlerConfig::new(seed));
        let config_clone = Arc::clone(&config);

        match fetch_url(sitemap_url, &config_clone).await {
            Ok(response) => {
                // Parse sitemap XML using regex as fallback
                match parse_sitemap_simple(&response) {
                    Ok(urls) => {
                        info!("Found {} URLs in {}", urls.len(), sitemap_url);
                        all_urls.extend(urls);
                    }
                    Err(e) => {
                        warn!("Failed to parse sitemap {}: {}", sitemap_url, e);
                    }
                }
            }
            Err(e) => {
                debug!("Sitemap not found at {}: {}", sitemap_url, e);
            }
        }
    }

    if all_urls.is_empty() {
        warn!("No sitemap found for {}", base_url);
    }

    Ok(all_urls)
}

/// Parse sitemap XML content using simple regex
///
/// # Arguments
///
/// * `xml_content` - XML content of the sitemap
///
/// # Returns
///
/// * `Ok(Vec<String>)` - List of URLs
/// * `Err(CrawlError)` - Parse error
fn parse_sitemap_simple(xml_content: &str) -> Result<Vec<String>, CrawlError> {
    // Simple regex to extract <loc>...</loc> content
    let re = regex::Regex::new(r"<loc>\s*([^<]+)\s*</loc>")
        .map_err(|e| CrawlError::Parse(format!("Failed to compile regex: {}", e)))?;

    let urls: Vec<String> = re
        .captures_iter(xml_content)
        .filter_map(|cap| cap.get(1).map(|m| m.as_str().to_string()))
        .collect();

    Ok(urls)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_sitemap_xml() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">
    <url>
        <loc>https://example.com/page1</loc>
    </url>
    <url>
        <loc>https://example.com/page2</loc>
    </url>
    <url>
        <loc>https://example.com/page3</loc>
    </url>
</urlset>"#;

        let urls = parse_sitemap_simple(xml).unwrap();
        assert_eq!(urls.len(), 3);
        assert_eq!(urls[0], "https://example.com/page1");
        assert_eq!(urls[1], "https://example.com/page2");
        assert_eq!(urls[2], "https://example.com/page3");
    }

    #[test]
    fn test_parse_sitemap_xml_empty() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">
</urlset>"#;

        let urls = parse_sitemap_simple(xml).unwrap();
        assert!(urls.is_empty());
    }

    #[tokio::test]
    async fn test_discover_urls_invalid_url() {
        let seed = Url::parse("https://example.com").unwrap();
        let config = CrawlerConfig::new(seed);

        let result = discover_urls("not-a-valid-url", 0, &config).await;
        assert!(result.is_err());
    }
}
