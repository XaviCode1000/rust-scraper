//! URL discovery logic extracted from orchestrator.

use std::sync::Arc;

use indicatif::{ProgressBar, ProgressDrawTarget, ProgressStyle};
use tracing::warn;
use url::Url;

use crate::application::crawl_options::CrawlOptions;
use crate::application::crawler::content_sink::{CapturedPage, CrawlContentSink};
use crate::application::crawler::crawl_site;
use crate::application::crawler::engine::{crawl_site_with_options, EngineOptions};
use crate::application::discover_urls_single_fetch;
use crate::domain::persistence::PersistenceMode;
use crate::error::Result as ScraperResult;
use crate::CrawlerConfig;

/// Build the discovery progress spinner, or `None` when quiet is enabled.
fn build_discovery_progress_bar(opts: &CrawlOptions, message: &str) -> Option<ProgressBar> {
    if opts.export.quiet {
        return None;
    }
    let pb = ProgressBar::new_spinner();
    pb.set_draw_target(ProgressDrawTarget::stderr());
    pb.enable_steady_tick(std::time::Duration::from_millis(100));
    // The spinner template is a hardcoded constant; parsing cannot fail.
    #[allow(clippy::expect_used)]
    let style = ProgressStyle::default_spinner()
        .template("{spinner} {msg}")
        .expect("valid spinner template");
    pb.set_style(style);
    pb.set_message(message.to_owned());
    Some(pb)
}

/// Discover URLs with progress bar.
///
/// Returns `Err` on network/timeout errors instead of silently swallowing them.
pub async fn discover_urls(
    crawler_config: &CrawlerConfig,
    opts: &CrawlOptions,
) -> ScraperResult<Vec<Url>> {
    let discovery_pb = build_discovery_progress_bar(opts, "Discovering URLs...");

    let discovered_urls = match discover_urls_single_fetch(opts.url.as_str(), crawler_config).await
    {
        Ok(urls) => urls,
        Err(e) => {
            // Treat an empty sitemap as empty discovery (technical success),
            // not as a network error. Typed match on `SitemapEmpty` — string
            // matching on the display message coupled exit codes to wording
            // (stabilization-sitemap-regression). Only propagate real errors.
            if matches!(e, crate::error::ScraperError::SitemapEmpty) {
                if let Some(pb) = discovery_pb.as_ref() {
                    pb.finish_with_message("No URLs found");
                }
                Vec::new()
            } else {
                if let Some(pb) = discovery_pb.as_ref() {
                    pb.finish_with_message("Discovery failed");
                }
                return Err(e);
            }
        },
    };

    if let Some(pb) = discovery_pb {
        pb.finish_with_message(format!("Found {} URLs", discovered_urls.len()).to_owned());
    }

    Ok(discovered_urls)
}

/// Unified discovery output (F-14, #1232 slice 1).
///
/// `urls` is the ordered discovery set consumed by dry-run previews and
/// the `plan_urls` boundary. `pages` carries captured bodies once a sink is
/// wired (slice 2); in slice 1 the sink is always `None`, so it stays empty.
#[derive(Debug, Clone, Default)]
pub struct DiscoveryOutput {
    /// Ordered discovered URLs (seed handling stays in `plan_urls`).
    pub urls: Vec<Url>,
    /// Captured page bodies (empty while no sink is wired).
    pub pages: Vec<CapturedPage>,
}

/// Single discovery entry behind both dry-run and the real DOM path.
///
/// Survivor is the recursive Engine path (`crawl_site` /
/// `crawl_site_with_options`), so `max_depth`, `max_pages`, robots, and
/// include/exclude patterns are honored identically in previews and crawls.
/// The sitemap branch keeps using [`discover_urls`] (source of truth from
/// XML); this function covers DOM mode only.
///
/// `persistence_mode` selects the checkpointing Engine entry exactly as the
/// legacy recursive path did. `sink` is reserved for slice 2 content
/// capture and is always `None` in slice 1; a `Some` value is ignored with
/// a warning so no engine change is needed (lead ruling b: no new engine
/// entry variant, the future sink travels as a field, single path).
///
/// Returns [`DiscoveryOutput`] with `pages` empty while no sink is wired.
pub async fn discover_urls_unified(
    crawler_config: CrawlerConfig,
    opts: &CrawlOptions,
    persistence_mode: &PersistenceMode,
    sink: Option<Arc<dyn CrawlContentSink>>,
) -> ScraperResult<DiscoveryOutput> {
    if sink.is_some() {
        warn!("capture sink ignored in slice 1; continuing without content capture");
    }
    let discovery_pb = build_discovery_progress_bar(opts, "Discovering URLs (recursive)...");

    let result = if let Some(checkpoint) = persistence_mode.checkpoint_cfg() {
        let options = EngineOptions {
            checkpoint_path: Some(checkpoint.dir.clone()),
            checkpoint_interval: checkpoint.interval,
            downloader_factory: Some(
                crate::application::container::Container::downloader_factory(),
            ),
            ..EngineOptions::default()
        };
        crawl_site_with_options(crawler_config, options).await?
    } else {
        crawl_site(crawler_config).await?
    };

    let urls: Vec<Url> = result.urls.into_iter().map(|d| d.url).collect();
    let count = urls.len();

    if let Some(pb) = discovery_pb {
        pb.finish_with_message(format!("Found {count} URLs").to_owned());
    }

    Ok(DiscoveryOutput {
        urls,
        pages: Vec::new(),
    })
}

/// Recursively discover URLs by running the real crawl Engine (BFS).
///
/// The default (non-interactive, non-sitemap) DOM crawl path previously called
/// `discover_urls_single_fetch`, which performs a SINGLE fetch and one round of link
/// extraction — so `--max-depth` was silently ignored and every crawl behaved
/// like depth 1 (bug #651). This routes discovery through [`crawl_site`], the
/// same recursive engine the batch and MCP paths use, so `max_depth`,
/// `max_pages`, robots, and include/exclude patterns are all honored.
///
/// The Engine returns a metadata-only `CrawlResult` (the set of fetched URLs);
/// the rich content extraction and on-disk export stay in the CLI's existing
/// `scrape_phase` / `export_phase`, which consume this URL list exactly as the
/// old single-level discovery produced it — so output location and format are
/// unchanged.
///
/// `persistence_mode` is the unified control-plane from slice 5c:
/// when the mode enables checkpointing (`Checkpoint` or `Full` — only via
/// an explicit `--resume`/`--state-dir` opt-in, F-01), the
/// engine is wired with `crawl_site_with_options` so the scoped
/// `crawl_checkpoint_<seed-hash>.json`
/// is created and the interval flows from the mode (not hardcoded).
/// `Disabled` and `Resume` fall back to `crawl_site` — the no-checkpoint path.
///
/// Compatibility shim over [`discover_urls_unified`] (F-14, #1232): keeps the
/// `Vec<Url>` call shape while the orchestrator migrates to the unified output.
pub async fn discover_urls_recursive(
    crawler_config: CrawlerConfig,
    opts: &CrawlOptions,
    persistence_mode: &PersistenceMode,
) -> ScraperResult<Vec<Url>> {
    let output = discover_urls_unified(crawler_config, opts, persistence_mode, None).await?;
    Ok(output.urls)
}

#[cfg(test)]
mod tests {
    use super::*;

    // T-2.1: discover_urls returns Result (compile-time + runtime verification)
    #[cfg_attr(
        miri,
        ignore = "btls/wreq FFI (BoringSSL TLS_method) not supported by Miri"
    )]
    #[tokio::test]
    async fn discover_urls_returns_result_type() {
        let seed_url = url::Url::parse("https://localhost:1").unwrap();
        let config = CrawlerConfig::builder(seed_url).build();
        let opts = CrawlOptions {
            url: crate::domain::ValidUrl::parse("https://localhost:1").unwrap(),
            ..Default::default()
        };

        let result = discover_urls(&config, &opts).await;
        // Should return Err for unreachable host, proving Result return type
        assert!(result.is_err(), "Expected Err for unreachable host");
    }

    /// Shared in-flight gauge + six-node star topology (seed + 5 leaves)
    /// for the R2-1 diagnostic: counts every request and the high-water
    /// mark of concurrent responses so the scheduler bound derived from
    /// the operator override is observable end to end.
    struct SixNodeGauge {
        inflight: std::sync::Arc<std::sync::atomic::AtomicUsize>,
        max_inflight: std::sync::Arc<std::sync::atomic::AtomicUsize>,
        total_requests: std::sync::Arc<std::sync::atomic::AtomicUsize>,
        seed_uri: String,
    }

    impl Clone for SixNodeGauge {
        fn clone(&self) -> Self {
            use std::sync::Arc;
            Self {
                inflight: Arc::clone(&self.inflight),
                max_inflight: Arc::clone(&self.max_inflight),
                total_requests: Arc::clone(&self.total_requests),
                seed_uri: self.seed_uri.clone(),
            }
        }
    }

    impl wiremock::Respond for SixNodeGauge {
        fn respond(&self, request: &wiremock::Request) -> wiremock::ResponseTemplate {
            use std::sync::atomic::Ordering as AtomicOrdering;
            let current = self.inflight.fetch_add(1, AtomicOrdering::SeqCst) + 1;
            self.max_inflight.fetch_max(current, AtomicOrdering::SeqCst);
            self.total_requests.fetch_add(1, AtomicOrdering::SeqCst);
            // Force overlap when the scheduler bound allows parallel
            // fetches, so an over-broad bound is observed by the gauge.
            std::thread::sleep(std::time::Duration::from_millis(30));
            self.inflight.fetch_sub(1, AtomicOrdering::SeqCst);
            if request.url.path() == "/" {
                let links: String = (0..5)
                    .map(|i| format!(r#"<a href="{}/p{i}">n{i}</a>"#, self.seed_uri))
                    .collect();
                wiremock::ResponseTemplate::new(200)
                    .set_body_string(format!("<html><body>{links}</body></html>"))
            } else {
                wiremock::ResponseTemplate::new(200)
                    .set_body_string("<html><body>leaf</body></html>")
            }
        }
    }

    /// Six-node diagnostic (bug R2-1): recursive URL discovery runs the
    /// real crawl Engine via `crawl_site`, so an operator `crawl = 1`
    /// override carried on the discovery config must reach it — six nodes
    /// fetched strictly one at a time instead of the auto tier table.
    #[cfg(not(miri))] // wiremock + wreq use boring-sys2 FFI (unsupported by Miri)
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn recursive_discovery_enforces_concurrency_override_six_node_diagnostic() {
        use crate::domain::budget::tiers::{BurstPermits, CrawlConcurrency};
        use crate::domain::budget::BudgetOverrides;
        use std::sync::atomic::{AtomicUsize, Ordering as AtomicOrdering};

        let server = wiremock::MockServer::start().await;
        let gauge = SixNodeGauge {
            inflight: std::sync::Arc::new(AtomicUsize::new(0)),
            max_inflight: std::sync::Arc::new(AtomicUsize::new(0)),
            total_requests: std::sync::Arc::new(AtomicUsize::new(0)),
            seed_uri: server.uri(),
        };
        wiremock::Mock::given(wiremock::matchers::any())
            .respond_with(gauge.clone())
            .mount(&server)
            .await;

        let seed_url = crate::domain::ValidUrl::parse(&format!("{}/", server.uri())).unwrap();
        let config = CrawlerConfig::builder(seed_url.as_url().clone())
            .max_depth(1)
            .max_pages(10)
            .concurrency(std::num::NonZeroUsize::new(16).expect("16 is non-zero")) // configured value must be beaten by the override
            .timeout_secs(5)
            .ignore_robots(true)
            .budget_overrides(BudgetOverrides {
                crawl: CrawlConcurrency::new(1).ok(),
                rate_burst: BurstPermits::new(4).ok(),
                ..BudgetOverrides::default()
            })
            .build();
        let mut opts = CrawlOptions {
            url: seed_url,
            ..Default::default()
        };
        opts.export.quiet = true;
        let discovered = discover_urls_recursive(config, &opts, &PersistenceMode::Disabled)
            .await
            .expect("six-node discovery must succeed");

        assert_eq!(
            discovered.len(),
            6,
            "seed + 5 discovered leaves must all be found"
        );
        assert_eq!(
            gauge.max_inflight.load(AtomicOrdering::SeqCst),
            1,
            "override crawl=1 must cap concurrent fetches at 1 through recursive discovery"
        );
    }
}
