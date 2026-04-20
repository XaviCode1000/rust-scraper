//! Metrics Collection Module
//!
//! Provides in-memory metrics collection for HTTP requests and crawler operations.
//! Metrics are exported as JSON on application completion.

use dashmap::DashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

/// Metrics collector for HTTP and crawler operations.
///
/// Uses atomic counters for thread-safe incrementing across async tasks.
/// Bandwidth per-domain uses DashMap for concurrent access.
#[derive(Debug, Default)]
pub struct MetricsCollector {
    /// Total HTTP requests made
    http_requests: AtomicU64,
    /// Total HTTP errors (4xx/5xx responses)
    http_errors: AtomicU64,
    /// Total pages scraped successfully
    pages_scraped: AtomicU64,
    /// Total URLs discovered
    urls_discovered: AtomicU64,
    /// Total bytes downloaded per domain
    bandwidth_per_domain: DashMap<String, AtomicU64>,
    /// Request latency tracking (domain -> sum of latencies in ms)
    latency_sum_per_domain: DashMap<String, AtomicU64>,
    /// Request count per domain
    requests_per_domain: DashMap<String, AtomicU64>,
    /// Start time for duration calculation
    start_time: Option<Instant>,
}

impl MetricsCollector {
    /// Create a new metrics collector.
    pub fn new() -> Self {
        Self {
            start_time: Some(Instant::now()),
            ..Default::default()
        }
    }

    /// Record an HTTP request.
    ///
    /// * `domain` - The domain the request was made to
    /// * `latency_ms` - Request latency in milliseconds
    /// * `status` - HTTP status code
    #[inline]
    pub fn record_request(&self, domain: &str, latency_ms: f64, status: u16) {
        self.http_requests.fetch_add(1, Ordering::Relaxed);

        // Update per-domain latency tracking
        self.latency_sum_per_domain
            .entry(domain.to_string())
            .or_default()
            .fetch_add(latency_ms as u64, Ordering::Relaxed);
        self.requests_per_domain
            .entry(domain.to_string())
            .or_default()
            .fetch_add(1, Ordering::Relaxed);

        // Track errors (4xx/5xx)
        if status >= 400 {
            self.http_errors.fetch_add(1, Ordering::Relaxed);
        }
    }

    /// Record an HTTP error (connection failure, timeout, etc.).
    #[inline]
    pub fn record_error(&self, domain: &str, _status: u16) {
        self.http_errors.fetch_add(1, Ordering::Relaxed);
        self.http_requests.fetch_add(1, Ordering::Relaxed);

        // Still count as a request attempt for the domain
        self.requests_per_domain
            .entry(domain.to_string())
            .or_default()
            .fetch_add(1, Ordering::Relaxed);
    }

    /// Record a page scraped successfully.
    #[inline]
    pub fn record_page_scraped(&self, _domain: &str) {
        self.pages_scraped.fetch_add(1, Ordering::Relaxed);
    }

    /// Record a URL discovered.
    #[inline]
    pub fn record_url_discovered(&self, _domain: &str) {
        self.urls_discovered.fetch_add(1, Ordering::Relaxed);
    }

    /// Record bandwidth usage for a domain.
    #[inline]
    pub fn record_bandwidth(&self, domain: &str, bytes: u64) {
        self.bandwidth_per_domain
            .entry(domain.to_string())
            .or_default()
            .fetch_add(bytes, Ordering::Relaxed);
    }

    /// Get total HTTP requests.
    pub fn total_requests(&self) -> u64 {
        self.http_requests.load(Ordering::Relaxed)
    }

    /// Get total HTTP errors.
    pub fn total_errors(&self) -> u64 {
        self.http_errors.load(Ordering::Relaxed)
    }

    /// Get total pages scraped.
    pub fn total_pages(&self) -> u64 {
        self.pages_scraped.load(Ordering::Relaxed)
    }

    /// Get total URLs discovered.
    pub fn total_urls(&self) -> u64 {
        self.urls_discovered.load(Ordering::Relaxed)
    }

    /// Get runtime duration in seconds since collector creation.
    pub fn runtime_seconds(&self) -> f64 {
        self.start_time
            .map(|t| t.elapsed().as_secs_f64())
            .unwrap_or(0.0)
    }

    /// Calculate average latency for a domain in milliseconds.
    pub fn avg_latency_ms(&self, domain: &str) -> f64 {
        let requests = self
            .requests_per_domain
            .get(domain)
            .map(|r| r.load(Ordering::Relaxed))
            .unwrap_or(0);
        let latency_sum = self
            .latency_sum_per_domain
            .get(domain)
            .map(|l| l.load(Ordering::Relaxed))
            .unwrap_or(0);

        if requests == 0 {
            return 0.0;
        }
        latency_sum as f64 / requests as f64
    }

    /// Export all metrics as a JSON value.
    pub fn export(&self) -> serde_json::Value {
        let mut domains = Vec::new();
        for entry in self.bandwidth_per_domain.iter() {
            let domain = entry.key().clone();
            let bandwidth = entry.value().load(Ordering::Relaxed);
            let requests = self
                .requests_per_domain
                .get(&domain)
                .map(|r| r.load(Ordering::Relaxed))
                .unwrap_or(0);
            let latency_sum = self
                .latency_sum_per_domain
                .get(&domain)
                .map(|l| l.load(Ordering::Relaxed))
                .unwrap_or(0);

            let avg_latency = if requests > 0 {
                latency_sum as f64 / requests as f64
            } else {
                0.0
            };

            domains.push(serde_json::json!({
                "domain": domain,
                "requests": requests,
                "bandwidth_bytes": bandwidth,
                "avg_latency_ms": avg_latency,
            }));
        }

        serde_json::json!({
            "runtime_seconds": self.runtime_seconds(),
            "http": {
                "total_requests": self.total_requests(),
                "total_errors": self.total_errors(),
                "error_rate": if self.total_requests() > 0 {
                    self.total_errors() as f64 / self.total_requests() as f64
                } else {
                    0.0
                },
            },
            "crawler": {
                "pages_scraped": self.total_pages(),
                "urls_discovered": self.total_urls(),
            },
            "domains": domains,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_collector_new() {
        let metrics = MetricsCollector::new();
        assert_eq!(metrics.total_requests(), 0);
        assert_eq!(metrics.total_errors(), 0);
        assert_eq!(metrics.total_pages(), 0);
    }

    #[test]
    fn test_record_request() {
        let metrics = MetricsCollector::new();
        metrics.record_request("example.com", 150.0, 200);

        assert_eq!(metrics.total_requests(), 1);
        assert_eq!(metrics.total_errors(), 0);
        assert_eq!(metrics.avg_latency_ms("example.com"), 150.0);
    }

    #[test]
    fn test_record_error() {
        let metrics = MetricsCollector::new();
        metrics.record_request("example.com", 50.0, 500);

        assert_eq!(metrics.total_requests(), 1);
        assert_eq!(metrics.total_errors(), 1);
    }

    #[test]
    fn test_record_page_scraped() {
        let metrics = MetricsCollector::new();
        metrics.record_page_scraped("example.com");

        assert_eq!(metrics.total_pages(), 1);
    }

    #[test]
    fn test_record_url_discovered() {
        let metrics = MetricsCollector::new();
        metrics.record_url_discovered("example.com");

        assert_eq!(metrics.total_urls(), 1);
    }

    #[test]
    fn test_record_bandwidth() {
        let metrics = MetricsCollector::new();
        metrics.record_bandwidth("example.com", 50000);

        let export = metrics.export();
        let domains = export["domains"].as_array().unwrap();
        assert_eq!(domains[0]["bandwidth_bytes"], 50000);
    }

    #[test]
    fn test_export_includes_runtime() {
        let metrics = MetricsCollector::new();
        let export = metrics.export();

        assert!(export["runtime_seconds"].is_number());
    }

    #[test]
    fn test_concurrent_increment_arc() {
        use std::sync::Arc;

        let metrics = Arc::new(MetricsCollector::new());

        let mut handles = vec![];
        for _ in 0..10 {
            let metrics = Arc::clone(&metrics);
            handles.push(std::thread::spawn(move || {
                metrics.record_request("example.com", 100.0, 200);
                metrics.record_page_scraped("example.com");
            }));
        }

        for handle in handles {
            let _ = handle.join();
        }

        assert!(metrics.total_requests() >= 10);
    }
}
