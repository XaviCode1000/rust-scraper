//! Rust Scraper — Production-ready web scraper with Clean Architecture
//!
//! **Rust Scraper** is a high-performance, async web scraper designed for
//! building RAG (Retrieval-Augmented Generation) datasets. Built with Clean Architecture
//! principles for maintainability and production use.
//!
//! # Features
//!
//! - **Async Web Scraping**: Multi-threaded with Tokio runtime
//! - **Sitemap Support**: Zero-allocation streaming parser (quick-xml)
//!   - Gzip decompression (async-compression)
//!   - Sitemap index recursion (max depth 3)
//!   - Auto-discovery from `robots.txt`
//! - **TUI Interactivo**: Ratatui + crossterm URL selector
//!   - Interactive checkbox selection
//!   - Confirmation mode before download
//!   - Terminal restore on panic/exit
//! - **Clean Architecture**: Domain → Application → Infrastructure → Adapters
//! - **Error Handling**: `thiserror` for libraries, `anyhow` for applications
//! - **Performance**: True streaming (~8KB RAM), LazyLock cache, bounded concurrency
//! - **Security**: SSRF prevention, Windows-safe filenames, WAF bypass prevention
//!
//! # Architecture
//!
//! Following Clean Architecture with four layers:
//!
//! ```text
//! Domain (entities, errors)
//!     ↓
//! Application (services, use cases)
//!     ↓
//! Infrastructure (HTTP, parsers, converters)
//!     ↓
//! Adapters (TUI, CLI, detectors)
//! ```
//!
//! **Dependency Rule:** Dependencies point inward. Domain never imports frameworks.
//!
//! # Examples
//!
//! ## Basic Usage
//!
//! ```no_run
//! use rust_scraper::{create_http_client, scrape_with_readability, ScraperConfig};
//!
//! # #[tokio::main]
//! # async fn main() -> anyhow::Result<()> {
//! let client = create_http_client()?;
//! let url = url::Url::parse("https://example.com")?;
//! let config = ScraperConfig::default();
//! let results = scrape_with_readability(&client, &url).await?;
//! # Ok(())
//! # }
//! ```
//!
//! ## URL Discovery with Sitemap
//!
//! ```no_run
//! use rust_scraper::{discover_urls, CrawlerConfig};
//!
//! # #[tokio::main]
//! # async fn main() -> anyhow::Result<()> {
//! let config = CrawlerConfig::builder()
//!     .concurrency(5)
//!     .use_sitemap(true)
//!     .build();
//!
//! let urls = discover_urls("https://example.com", &config).await?;
//! println!("Found {} URLs", urls.len());
//! # Ok(())
//! # }
//! ```
//!
//! ## Custom Configuration
//!
//! ```
//! use rust_scraper::ScraperConfig;
//!
//! let config = ScraperConfig::default()
//!     .with_images()
//!     .with_documents()
//!     .with_output_dir("./output".into())
//!     .with_scraper_concurrency(5);
//!
//! assert!(config.download_images);
//! assert!(config.download_documents);
//! assert_eq!(config.scraper_concurrency, 5);
//! ```
//!
//! # Error Handling
//!
//! This library uses [`thiserror`](https://docs.rs/thiserror) for type-safe error handling.
//! All fallible functions return [`Result<T, ScraperError>`](Result).
//!
//! ```
//! use rust_scraper::{validate_and_parse_url, ScraperError};
//!
//! match validate_and_parse_url("https://example.com") {
//!     Ok(url) => println!("Valid URL: {}", url),
//!     Err(ScraperError::InvalidUrl(msg)) => eprintln!("Invalid URL: {}", msg),
//!     Err(e) => eprintln!("Error: {}", e),
//! }
//! ```
//!
//! # Performance
//!
//! - **Streaming**: Constant ~8KB RAM usage, no OOM risks
//! - **Zero-Allocation Parsing**: quick-xml for sitemaps
//! - **LazyLock Cache**: Syntax highlighting (2-10ms → ~0.01ms)
//! - **Bounded Concurrency**: Configurable parallel downloads
//!
//! # Security
//!
//! - **SSRF Prevention**: URL host comparison (not string contains)
//! - **Windows Safe**: Reserved names blocked (`CON` → `CON_safe`)
//! - **WAF Bypass Prevention**: Chrome 131+ UAs with TTL caching
//! - **Input Validation**: `url::Url::parse()` (RFC 3986 compliant)
//!
//! # Testing
//!
//! ```bash
//! # Run all tests
//! cargo test
//!
//! # Run with output
//! cargo test -- --nocapture
//!
//! # Run specific test
//! cargo test test_validate_and_parse_url
//! ```
//!
//! **Tests:** 198 passing ✅
//!
//! # MSRV
//!
//! Minimum Supported Rust Version: 1.75.0

// ============================================================================
// Public API Exports
// ============================================================================

pub mod config;
pub mod error;

// Domain layer — Core business entities (pure, no dependencies)
pub mod domain;
pub use domain::{
    ContentType, CrawlError, CrawlResult, CrawlerConfig, CrawlerConfigBuilder, DiscoveredUrl,
    DownloadedAsset, ScrapedContent, ValidUrl,
};

// Application layer — Use cases (orchestration)
pub mod application;
pub use application::{
    crawl_site, create_http_client, discover_urls, discover_urls_for_tui, extract_domain,
    fetch_sitemap, is_allowed, is_excluded, is_internal_link, matches_pattern,
    scrape_multiple_with_limit, scrape_urls_for_tui, scrape_with_config, scrape_with_readability,
};

// Infrastructure layer — Implementations (technical details)
pub mod infrastructure;

// Adapters — External integrations (feature-gated)
pub mod adapters;

// Legacy re-exports for backward compatibility
pub mod extractor;
pub mod url_path;
pub mod user_agent;
pub use url_path::{Domain, OutputPath, UrlPath};
pub use user_agent::{get_random_user_agent_from_pool, UserAgentCache};

// CLI types
pub use clap::{Parser, ValueEnum};
pub use error::{Result, ScraperError};

// Re-export save_results for convenience
pub use infrastructure::output::file_saver::save_results;

// ============================================================================
// Public Types
// ============================================================================

/// Output format for scraped content.
///
/// # Examples
///
/// ```
/// use rust_scraper::OutputFormat;
///
/// let format = OutputFormat::Markdown;
/// assert_eq!(format, OutputFormat::Markdown);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum OutputFormat {
    /// Markdown format with YAML frontmatter (recommended for RAG)
    Markdown,
    /// Plain text without formatting
    Text,
    /// Structured JSON with metadata
    Json,
}

/// Configuration for web scraping and asset downloading.
///
/// Following **config-externalize** (rust-skills): All concurrency settings
/// are configurable for hardware-aware optimization.
///
/// # Examples
///
/// ```
/// use rust_scraper::ScraperConfig;
///
/// // Default configuration
/// let config = ScraperConfig::default();
///
/// // Custom configuration with builder pattern
/// let config = ScraperConfig::default()
///     .with_images()
///     .with_documents()
///     .with_output_dir("./output".into())
///     .with_scraper_concurrency(5);
/// ```
///
/// # Concurrency Recommendations
///
/// | Storage | Concurrency | Reason |
/// |---------|-------------|--------|
/// | HDD | 3 (default) | Avoids disk thrashing on mechanical drives |
/// | SSD | 5-8 | Faster random I/O |
/// | NVMe | 10+ | Very high IOPS |
#[derive(Debug, Clone)]
pub struct ScraperConfig {
    /// Enable image downloading (PNG, JPG, GIF, WEBP, SVG, BMP)
    pub download_images: bool,
    /// Enable document downloading (PDF, DOCX, XLSX, PPTX, etc.)
    pub download_documents: bool,
    /// Output directory for downloaded assets
    pub output_dir: std::path::PathBuf,
    /// Maximum file size in bytes (default: 50MB)
    pub max_file_size: Option<u64>,
    /// Maximum concurrent scrapers (default: 3 for HDD-aware on 4C CPU)
    pub scraper_concurrency: usize,
}

impl Default for ScraperConfig {
    fn default() -> Self {
        Self {
            download_images: false,
            download_documents: false,
            output_dir: std::path::PathBuf::from("output"),
            max_file_size: Some(50 * 1024 * 1024), // 50MB default
            scraper_concurrency: 3,                // HDD-aware: nproc - 1 for 4C CPU
        }
    }
}

impl ScraperConfig {
    /// Create a new config with default values.
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_scraper::ScraperConfig;
    ///
    /// let config = ScraperConfig::new();
    /// assert!(!config.download_images);
    /// ```
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Enable image downloading.
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_scraper::ScraperConfig;
    ///
    /// let config = ScraperConfig::default().with_images();
    /// assert!(config.download_images);
    /// ```
    #[must_use]
    pub fn with_images(mut self) -> Self {
        self.download_images = true;
        self
    }

    /// Enable document downloading.
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_scraper::ScraperConfig;
    ///
    /// let config = ScraperConfig::default().with_documents();
    /// assert!(config.download_documents);
    /// ```
    #[must_use]
    pub fn with_documents(mut self) -> Self {
        self.download_documents = true;
        self
    }

    /// Set custom output directory.
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_scraper::ScraperConfig;
    ///
    /// let config = ScraperConfig::default()
    ///     .with_output_dir("./my-output".into());
    /// assert_eq!(config.output_dir, std::path::PathBuf::from("./my-output"));
    /// ```
    #[must_use]
    pub fn with_output_dir(mut self, dir: std::path::PathBuf) -> Self {
        self.output_dir = dir;
        self
    }

    /// Set scraper concurrency limit.
    ///
    /// # Arguments
    ///
    /// * `concurrency` - Maximum concurrent scrapers
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_scraper::ScraperConfig;
    ///
    /// let config = ScraperConfig::default()
    ///     .with_scraper_concurrency(5);
    /// assert_eq!(config.scraper_concurrency, 5);
    /// ```
    ///
    /// # Recommendations
    ///
    /// - **HDD**: 3 (default) — avoids disk thrashing
    /// - **SSD**: 5-8 — faster random I/O
    /// - **NVMe**: 10+ — very high IOPS
    #[must_use]
    pub fn with_scraper_concurrency(mut self, concurrency: usize) -> Self {
        self.scraper_concurrency = concurrency;
        self
    }

    /// Check if any download is enabled.
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_scraper::ScraperConfig;
    ///
    /// let config = ScraperConfig::default();
    /// assert!(!config.has_downloads());
    ///
    /// let config = config.with_images();
    /// assert!(config.has_downloads());
    /// ```
    pub fn has_downloads(&self) -> bool {
        self.download_images || self.download_documents
    }
}

/// CLI Arguments for the rust-scraper binary.
///
/// Parsed using `clap` with derive macros.
///
/// # Examples
///
/// ```no_run
/// use rust_scraper::Args;
/// use clap::Parser;
///
/// let args = Args::parse_from([
///     "rust-scraper",
///     "--url", "https://example.com",
///     "--output", "./output",
///     "--format", "markdown",
/// ]);
///
/// assert_eq!(args.url, "https://example.com");
/// ```
#[derive(Parser, Debug)]
#[command(name = "rust-scraper")]
#[command(about = "Production-ready web scraper with Clean Architecture", long_about = None)]
pub struct Args {
    /// URL to scrape (required)
    #[arg(short, long, required = true)]
    pub url: String,

    /// CSS selector for content extraction
    #[arg(short, long, default_value = "body")]
    pub selector: String,

    /// Output directory for scraped content
    #[arg(short, long, default_value = "output")]
    pub output: std::path::PathBuf,

    /// Output format (markdown, text, json)
    #[arg(short, long, default_value = "markdown", value_enum)]
    pub format: OutputFormat,

    /// Delay between requests in milliseconds
    #[arg(long, default_value = "1000")]
    pub delay_ms: u64,

    /// Maximum pages to scrape
    #[arg(long, default_value = "10")]
    pub max_pages: usize,

    /// Download images from the page
    #[arg(long, default_value = "false")]
    pub download_images: bool,

    /// Download documents from the page (PDF, DOCX, XLSX, etc.)
    #[arg(long, default_value = "false")]
    pub download_documents: bool,

    /// Verbosity level (use multiple times for more detail: -v, -vv, -vvv)
    #[arg(short, long, action = clap::ArgAction::Count)]
    pub verbose: u8,

    // ========== Sitemap Support ==========
    /// Use sitemap for URL discovery (auto-discovers from robots.txt if URL not provided)
    #[arg(long)]
    pub use_sitemap: bool,

    /// Explicit sitemap URL (optional, auto-discovers if not provided)
    #[arg(long, requires = "use_sitemap")]
    pub sitemap_url: Option<String>,

    // ========== TUI Interactive Mode ==========
    /// Interactive mode with TUI URL selector
    #[arg(long)]
    pub interactive: bool,
}

// ============================================================================
// Public Functions
// ============================================================================

/// Validate and parse a URL string using the `url` crate (RFC 3986 compliant).
///
/// This function performs strict URL validation:
/// - Trims whitespace automatically
/// - Requires http or https scheme (case-insensitive)
/// - Requires a valid host
/// - Rejects malformed URLs
///
/// # Arguments
///
/// * `url` - URL string to validate and parse
///
/// # Returns
///
/// * `Ok(url::Url)` - Validated and parsed URL
/// * `Err(ScraperError::InvalidUrl)` - Invalid URL with error message
///
/// # Errors
///
/// Returns an error if:
/// - URL is empty
/// - URL has invalid format
/// - URL scheme is not http or https
/// - URL has no host
///
/// # Examples
///
/// ```
/// use rust_scraper::validate_and_parse_url;
///
/// // Valid URLs
/// let url = validate_and_parse_url("https://example.com").unwrap();
/// assert_eq!(url.host_str(), Some("example.com"));
///
/// let url = validate_and_parse_url("HTTP://EXAMPLE.COM").unwrap();
/// assert_eq!(url.scheme(), "http");
///
/// // Invalid URLs
/// assert!(validate_and_parse_url("").is_err());
/// assert!(validate_and_parse_url("ftp://example.com").is_err());
/// assert!(validate_and_parse_url("not-a-url").is_err());
/// ```
///
/// # Whitespace Handling
///
/// Leading and trailing whitespace is automatically trimmed:
///
/// ```
/// use rust_scraper::validate_and_parse_url;
///
/// let url = validate_and_parse_url("  https://example.com  ").unwrap();
/// assert_eq!(url.host_str(), Some("example.com"));
/// ```
pub fn validate_and_parse_url(url: &str) -> Result<url::Url> {
    if url.is_empty() {
        return Err(ScraperError::invalid_url("URL cannot be empty"));
    }

    // Url::parse automatically trims whitespace and handles case-insensitive schemes
    // Following rust-skills: url-no-string-split (don't use starts_with for URLs)
    let parsed = url::Url::parse(url.trim())
        .map_err(|e| ScraperError::invalid_url(format!("Failed to parse URL '{}': {}", url, e)))?;

    // Check scheme (case-insensitive, already lowercased by Url::parse)
    match parsed.scheme() {
        "http" | "https" => {}
        scheme => {
            return Err(ScraperError::invalid_url(format!(
                "URL must use http or https scheme, got '{}'",
                scheme
            )))
        }
    }

    if parsed.host_str().is_none() {
        return Err(ScraperError::invalid_url("URL must have a valid host"));
    }

    Ok(parsed)
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scraper_config_default() {
        let config = ScraperConfig::default();
        assert!(!config.download_images);
        assert!(!config.download_documents);
        assert!(!config.has_downloads());
        assert_eq!(config.scraper_concurrency, 3);
    }

    #[test]
    fn test_scraper_config_with_images() {
        let config = ScraperConfig::default().with_images();
        assert!(config.download_images);
        assert!(config.has_downloads());
    }

    #[test]
    fn test_scraper_config_with_documents() {
        let config = ScraperConfig::default().with_documents();
        assert!(config.download_documents);
        assert!(config.has_downloads());
    }

    #[test]
    fn test_scraper_config_with_concurrency() {
        let config = ScraperConfig::default().with_scraper_concurrency(5);
        assert_eq!(config.scraper_concurrency, 5);
    }

    #[test]
    fn test_validate_and_parse_url_success() {
        let url = validate_and_parse_url("https://example.com");
        assert!(url.is_ok());
    }

    #[test]
    fn test_validate_and_parse_url_empty() {
        let result = validate_and_parse_url("");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_and_parse_url_invalid_scheme() {
        let result = validate_and_parse_url("ftp://example.com");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("http or https"));
    }

    // ========================================================================
    // TASK-003: URL Validation Robusta Tests
    // ========================================================================

    #[test]
    fn test_url_validation_with_spaces() {
        // Url::parse trims whitespace automatically
        let url = validate_and_parse_url(" https://example.com ").unwrap();
        assert_eq!(url.host_str(), Some("example.com"));
    }

    #[test]
    fn test_url_validation_case_insensitive() {
        // Scheme is case-insensitive
        let url = validate_and_parse_url("HTTP://EXAMPLE.COM").unwrap();
        assert_eq!(url.scheme(), "http");
        assert_eq!(url.host_str(), Some("example.com"));
    }

    #[test]
    fn test_url_validation_https_uppercase() {
        let url = validate_and_parse_url("HTTPS://example.com").unwrap();
        assert_eq!(url.scheme(), "https");
    }

    #[test]
    fn test_url_validation_mixed_case() {
        let url = validate_and_parse_url("HtTpS://Example.COM").unwrap();
        assert_eq!(url.scheme(), "https");
        assert_eq!(url.host_str(), Some("example.com"));
    }

    #[test]
    fn test_url_validation_with_leading_spaces() {
        let url = validate_and_parse_url("   https://example.com").unwrap();
        assert_eq!(url.scheme(), "https");
    }

    #[test]
    fn test_url_validation_with_trailing_spaces() {
        let url = validate_and_parse_url("https://example.com   ").unwrap();
        assert_eq!(url.scheme(), "https");
    }

    #[test]
    fn test_url_validation_invalid_scheme_ftp() {
        let result = validate_and_parse_url("ftp://example.com");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("http or https"));
    }

    #[test]
    fn test_url_validation_invalid_scheme_file() {
        let result = validate_and_parse_url("file:///etc/passwd");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("http or https"));
    }

    #[test]
    fn test_url_validation_no_scheme() {
        let result = validate_and_parse_url("example.com");
        assert!(result.is_err());
    }

    #[test]
    fn test_url_validation_malformed() {
        let result = validate_and_parse_url("not-a-valid-url-at-all");
        assert!(result.is_err());
    }
}
