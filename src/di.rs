//! Dependency Injection Container
//!
//! Provides centralized dependency injection for the application.
//! Following Clean Architecture: container lives in application layer.

use std::sync::Arc;
use crate::config::Config;
use crate::application::http_client::HttpClient;
use crate::infrastructure::config::ScraperConfig;

/// Dependency Injection Container
///
/// Holds all application services and their dependencies.
/// Services are created once and reused throughout the application lifetime.
#[derive(Clone)]
pub struct Container {
    config: Arc<Config>,
    http_client: Arc<HttpClient>,
}

impl Container {
    /// Create a new container with the given configuration
    pub async fn new(config: Config) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        // Validate config
        config.validate()?;

        // Create HTTP client
        let http_client = Arc::new(HttpClient::new(config.http.clone())?);

        Ok(Self {
            config: Arc::new(config),
            http_client,
        })
    }

    /// Get the application configuration
    pub fn config(&self) -> &Config {
        &self.config
    }

    /// Get the HTTP client
    pub fn http_client(&self) -> &HttpClient {
        &self.http_client
    }

    /// Get the scraper configuration
    pub fn scraper_config(&self) -> &ScraperConfig {
        &self.config.scraper
    }
}