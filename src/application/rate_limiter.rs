//! Rate Limiter module — Token Bucket implementation using governor
//!
//! Extracts the rate limiting logic from crawler_service.rs to allow
//! for independent testing and potential future swapping (e.g., Redis-backed).
//!
//! # Design Decisions
//!
//! - Uses `governor` crate with Token Bucket algorithm
//! - Thread-safe via Arc (shares across async tasks)
//! - Configurable delay and burst parameters
//! - No Mutex needed - governor handles internal synchronization

use std::num::NonZeroU32;
use std::sync::Arc;
use std::time::Duration;

use governor::{
    clock::QuantaClock,
    state::{InMemoryState, NotKeyed},
    Quota, RateLimiter,
};

use crate::error::ScraperError;

/// Type alias for the rate limiter - allows swapping implementations
pub type CrawlRateLimiter = RateLimiter<NotKeyed, InMemoryState, QuantaClock>;

/// Rate limiter configuration
#[derive(Debug, Clone)]
pub struct RateLimiterConfig {
    /// Delay between requests in milliseconds
    pub delay_ms: u64,
    /// Maximum concurrent requests (burst)
    pub concurrency: u32,
}

impl RateLimiterConfig {
    /// Create new configuration
    pub fn new(delay_ms: u64, concurrency: u32) -> Self {
        Self {
            delay_ms,
            concurrency,
        }
    }
}

/// Shared rate limiter for crawl operations
#[derive(Clone)]
pub struct SharedRateLimiter(Arc<CrawlRateLimiter>);

impl SharedRateLimiter {
    /// Create a new shared rate limiter from config
    pub fn new(config: &RateLimiterConfig) -> Result<Self, ScraperError> {
        let quota = Quota::with_period(Duration::from_millis(config.delay_ms))
            .ok_or_else(|| ScraperError::Config("Invalid period".into()))?;

        let quota = quota.allow_burst(
            NonZeroU32::new(config.concurrency)
                .ok_or_else(|| ScraperError::Config("Concurrency must be > 0".into()))?,
        );

        let limiter = RateLimiter::direct(quota);
        Ok(Self(Arc::new(limiter)))
    }

    /// Wait until a permit is available
    pub async fn until_ready(&self) {
        self.0.until_ready().await;
    }
}

impl From<RateLimiter<NotKeyed, InMemoryState, QuantaClock>> for SharedRateLimiter {
    fn from(limiter: RateLimiter<NotKeyed, InMemoryState, QuantaClock>) -> Self {
        Self(Arc::new(limiter))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rate_limiter_config_default() {
        let config = RateLimiterConfig::new(100, 5);
        assert_eq!(config.delay_ms, 100);
        assert_eq!(config.concurrency, 5);
    }

    #[tokio::test]
    async fn test_rate_limiter_creation() {
        let config = RateLimiterConfig::new(50, 2);
        let limiter = SharedRateLimiter::new(&config);
        assert!(limiter.is_ok());
    }

    #[tokio::test]
    async fn test_rate_limiter_until_ready() {
        let config = RateLimiterConfig::new(10, 1);
        let limiter = SharedRateLimiter::new(&config).unwrap();
        
        // Wait for permit availability
        limiter.until_ready().await;
        
        // If we got here, the limiter worked
    }

    #[test]
    fn test_rate_limiter_clone_ischeap() {
        let config = RateLimiterConfig::new(100, 5);
        let limiter = SharedRateLimiter::new(&config).unwrap();

        // Cloning should be cheap (just Arc increment)
        let _clone = limiter.clone();
    }

    // ============================================================================
    // Behavioral Rate Limiting Tests
    // ============================================================================

    #[tokio::test]
    async fn test_rate_limiter_until_ready_spreads_over_time() {
        // Test que N tasks concurrentes llamando until_ready() son espaciadas
        // Config: delay_ms=50ms, concurrency=1
        // 5 tasks → mínimo ~200ms de spread total
        // Mide elapsed y verifica >= (N-1) * delay

        let config = RateLimiterConfig::new(50, 1); // 50ms entre requests, burst=1
        let limiter = SharedRateLimiter::new(&config).unwrap();

        let num_tasks = 5;
        let start = std::time::Instant::now();

        let mut handles = Vec::new();
        for _ in 0..num_tasks {
            let limiter = limiter.clone();
            let handle = tokio::spawn(async move {
                limiter.until_ready().await;
            });
            handles.push(handle);
        }

        futures::future::join_all(handles).await;
        let elapsed = start.elapsed();

        // 5 tasks con delay de 50ms → mínimo ~200ms
        // Con algo de jitter, verificamos al menos 150ms (75% de teórico)
        let min_expected_ms = 150;
        assert!(
            elapsed.as_millis() >= min_expected_ms,
            "Tiempo transcurrido {}ms < {}ms mínimo — rate limiter no está espaciando",
            elapsed.as_millis(),
            min_expected_ms
        );
    }

    #[tokio::test]
    async fn test_rate_limiter_burst_allows_parallel_requests() {
        // Test que burst de N requests ocurren en paralelo
        // Config: delay_ms=100ms, concurrency=5
        // 5 tasks simultáneas → todas deben pasar rápido (dentro del burst)
        use tokio::time::Instant;

        let config = RateLimiterConfig::new(100, 5); // 100ms delay, burst=5
        let limiter = SharedRateLimiter::new(&config).unwrap();

        let num_tasks = 5;
        let start = Instant::now();

        let mut handles = Vec::new();
        for _ in 0..num_tasks {
            let limiter = limiter.clone();
            let handle = tokio::spawn(async move {
                limiter.until_ready().await;
            });
            handles.push(handle);
        }

        futures::future::join_all(handles).await;
        let elapsed = start.elapsed();

        // 5 tasks con burst=5 → todas deberían pasar casi instantáneo (< 50ms)
        assert!(
            elapsed.as_millis() < 50,
            "Tiempo {}ms > 50ms — burst no está funcionando",
            elapsed.as_millis()
        );
    }

    #[tokio::test]
    async fn test_rate_limiter_concurrent_backpressure() {
        // Test que 20 tasks concurrentes no colapsan — se encolan correctamente
        let config = RateLimiterConfig::new(10, 1); // 10ms, burst=1
        let limiter = SharedRateLimiter::new(&config).unwrap();

        let num_tasks = 20;
        let start = std::time::Instant::now();

        let mut handles = Vec::new();
        for _ in 0..num_tasks {
            let limiter = limiter.clone();
            let handle = tokio::spawn(async move {
                limiter.until_ready().await;
            });
            handles.push(handle);
        }

        futures::future::join_all(handles).await;
        let elapsed = start.elapsed();

        // 20 tasks × 10ms delay = 190ms mínimo
        // Verificamos que tomó al menos 100ms (rate limiting activo)
        assert!(
            elapsed.as_millis() >= 100,
            "20 tasks completaron en {}ms — rate limiting no está regulando",
            elapsed.as_millis()
        );
    }

    #[test]
    fn test_rate_limiter_config_zero_delay_returns_error() {
        // delay_ms=0 → debe retornar error, no panic
        let config = RateLimiterConfig::new(0, 1);
        let result = SharedRateLimiter::new(&config);
        assert!(
            result.is_err(),
            "delay_ms=0 debería retornar error"
        );
    }

    #[test]
    fn test_rate_limiter_config_zero_concurrency_returns_error() {
        // concurrency=0 → debe retornar error, no panic
        let config = RateLimiterConfig::new(100, 0);
        let result = SharedRateLimiter::new(&config);
        assert!(
            result.is_err(),
            "concurrency=0 debería retornar error"
        );
    }

    #[test]
    fn test_rate_limiter_config_default_values() {
        // Verifica valores por defecto
        let config = RateLimiterConfig::new(100, 5);
        assert_eq!(config.delay_ms, 100);
        assert_eq!(config.concurrency, 5);
    }

    #[test]
    fn test_rate_limiter_cheap_clone() {
        // Verifica que SharedRateLimiter sea Arc barato (no falla al clonar)
        let config = RateLimiterConfig::new(100, 5);
        let limiter = SharedRateLimiter::new(&config).unwrap();
        let _clone = limiter.clone();
        let _clone2 = limiter.clone();
        // Solo verificamos que compile y no falle
        assert!(true);
    }
}