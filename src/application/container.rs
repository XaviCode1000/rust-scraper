//! Dependency Injection Container
//!
//! Provides a centralized way to wire up all services and their dependencies.
//! Following Clean Architecture, the container lives in the application layer
//! and creates instances of infrastructure implementations.

use std::sync::Arc;

use crate::domain::{
    repositories::CrawlResultRepository,
    CrawlerConfig,
};
use crate::infrastructure::config::ScraperConfig;
use crate::application::http_client::{HttpClient, HttpClientConfig};
use crate::infrastructure::export::state_store::StateStore;

/// Dependency Injection Container
///
/// Holds all service instances and their configurations.
/// Services are created once and reused throughout the application.
#[derive(Clone)]
pub struct Container {
    pub crawler_config: CrawlerConfig,
    pub scraper_config: ScraperConfig,
    pub http_client: Arc<HttpClient>,
    pub state_store: Option<Arc<StateStore>>,
}

impl Container {
    /// Create a new container with the given configurations.
    ///
    /// # Arguments
    ///
    /// * `crawler_config` - Configuration for crawling behavior
    /// * `scraper_config` - Configuration for scraping behavior
    ///
    /// # Returns
    ///
    /// A new container instance with all services initialized
    pub async fn new(
        crawler_config: CrawlerConfig,
        scraper_config: ScraperConfig,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        // Initialize infrastructure services
        let http_client = Arc::new(HttpClient::new(HttpClientConfig::default())?);

        // State store is optional (for resume mode)
        let state_store = None;

        Ok(Self {
            crawler_config,
            scraper_config,
            http_client,
            state_store,
        })
    }

    /// Set the state store for resume functionality.
    pub fn with_state_store(mut self, state_store: StateStore) -> Self {
        self.state_store = Some(Arc::new(state_store));
        self
    }

    /// Get a repository for crawl results (backed by state store if available).
    pub fn crawl_result_repository(&self) -> Option<Arc<dyn CrawlResultRepository>> {
        // TODO: Implement CrawlResultRepository for StateStore
        None
    }
}