//! Sitemap configuration with builder pattern
//!
//! Following api-builder-pattern: clear, self-documenting API

/// Sitemap parser configuration
///
/// Following api-builder-pattern: clear, self-documenting API
#[derive(Debug, Clone)]
pub struct SitemapConfig {
    /// Enable gzip decompression (default: true)
    pub gzip_enabled: bool,
    /// Maximum recursion depth for sitemap indexes (default: 3)
    pub max_depth: u8,
    /// Concurrent requests for sitemap indexes (default: 5)
    pub concurrency: usize,
    /// Maximum HTTP response size in bytes (default: 50MB)
    pub max_response_size: usize,
    /// Maximum decompressed gzip size in bytes (default: 100MB)
    pub max_decompressed_size: usize,
}

impl Default for SitemapConfig {
    fn default() -> Self {
        Self {
            gzip_enabled: true,
            max_depth: 3,
            concurrency: 5,
            max_response_size: 52_428_800,      // 50MB
            max_decompressed_size: 104_857_600, // 100MB
        }
    }
}

impl SitemapConfig {
    /// Create new config builder
    pub fn builder() -> SitemapConfigBuilder {
        SitemapConfigBuilder::default()
    }
}

/// Builder for SitemapConfig
///
/// Following api-builder-must-use: #[must_use] attribute
#[derive(Default)]
#[must_use = "builders do nothing unless you call build()"]
pub struct SitemapConfigBuilder {
    gzip_enabled: bool,
    max_depth: u8,
    concurrency: usize,
    max_response_size: usize,
    max_decompressed_size: usize,
}

impl SitemapConfigBuilder {
    /// Enable or disable gzip decompression
    pub fn gzip_enabled(mut self, enabled: bool) -> Self {
        self.gzip_enabled = enabled;
        self
    }

    /// Set maximum recursion depth for sitemap indexes
    pub fn max_depth(mut self, depth: u8) -> Self {
        self.max_depth = depth;
        self
    }

    /// Set concurrency level for parallel sitemap parsing
    pub fn concurrency(mut self, count: usize) -> Self {
        self.concurrency = count;
        self
    }

    /// Set maximum HTTP response size in bytes
    pub fn max_response_size(mut self, size: usize) -> Self {
        self.max_response_size = size;
        self
    }

    /// Set maximum decompressed gzip size in bytes
    pub fn max_decompressed_size(mut self, size: usize) -> Self {
        self.max_decompressed_size = size;
        self
    }

    /// Build the configuration
    #[must_use]
    pub fn build(self) -> SitemapConfig {
        let defaults = SitemapConfig::default();
        SitemapConfig {
            gzip_enabled: self.gzip_enabled,
            max_depth: self.max_depth,
            concurrency: self.concurrency,
            max_response_size: if self.max_response_size == 0 {
                defaults.max_response_size
            } else {
                self.max_response_size
            },
            max_decompressed_size: if self.max_decompressed_size == 0 {
                defaults.max_decompressed_size
            } else {
                self.max_decompressed_size
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = SitemapConfig::default();
        assert!(config.gzip_enabled);
        assert_eq!(config.max_depth, 3);
        assert_eq!(config.concurrency, 5);
        assert_eq!(config.max_response_size, 52_428_800);
        assert_eq!(config.max_decompressed_size, 104_857_600);
    }

    #[test]
    fn test_builder_custom_values() {
        let config = SitemapConfig::builder()
            .gzip_enabled(false)
            .max_depth(1)
            .concurrency(10)
            .max_response_size(1_000_000)
            .max_decompressed_size(2_000_000)
            .build();

        assert!(!config.gzip_enabled);
        assert_eq!(config.max_depth, 1);
        assert_eq!(config.concurrency, 10);
        assert_eq!(config.max_response_size, 1_000_000);
        assert_eq!(config.max_decompressed_size, 2_000_000);
    }

    #[test]
    fn test_builder_zero_values_use_defaults() {
        let config = SitemapConfig::builder()
            .max_response_size(0)
            .max_decompressed_size(0)
            .build();

        assert_eq!(config.max_response_size, 52_428_800);
        assert_eq!(config.max_decompressed_size, 104_857_600);
    }
}
