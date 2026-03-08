//! Rust Scraper - Modern web scraper for RAG datasets
//!
//! Extracts clean, structured content from web pages using readability algorithm.

use anyhow::Context;
use rust_scraper::{
    create_http_client, save_results, scrape_with_config, validate_and_parse_url, Args, Parser,
    ScraperConfig, UserAgentCache,
};
use tracing::{info, warn};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 1. Parse CLI arguments - Fail fast if URL is missing
    let args = Args::parse();

    // 2. Initialize logging with configurable level
    let log_level = match args.verbose {
        0 => "info",
        1 => "debug",
        _ => "trace",
    };
    rust_scraper::config::init_logging(log_level);

    info!("🚀 Rust Scraper v0.4.0 - Clean Architecture");
    info!("📌 Target: {}", args.url);
    info!("📁 Output: {}", args.output.display());

    // 3. Load user agents with TTL-based caching (TASK-001)
    info!("🔄 Loading user agents (cache check)...");
    let user_agents = UserAgentCache::load().await;
    info!(
        "✅ User agent loaded: {} agents available",
        user_agents.len()
    );

    // 4. Validate URL - parse with url crate (TASK-003: RFC 3986 compliant)
    let parsed_url = validate_and_parse_url(&args.url).context("Invalid URL provided")?;

    info!("✅ URL validated: {}", parsed_url);

    // 5. Create configured HTTP client (with retry + user-agent rotation)
    // Note: Uses deprecated get_random_user_agent() for backward compatibility
    let client = create_http_client()?;

    // 6. Configure scraping with download options
    let config = ScraperConfig {
        download_images: args.download_images,
        download_documents: args.download_documents,
        output_dir: args.output.clone(),
        max_file_size: Some(50 * 1024 * 1024), // 50MB default
        scraper_concurrency: 3,                // HDD-aware: nproc - 1 for 4C CPU
    };

    if config.download_images {
        info!("🖼️  Image download: ENABLED");
    }
    if config.download_documents {
        info!("📄 Document download: ENABLED");
    }

    // 7. Execute scraping
    info!("📡 Starting scraping...");

    let results = scrape_with_config(&client, &parsed_url, &config)
        .await
        .context("Scraping failed")?;

    if results.is_empty() {
        warn!("⚠️  No content extracted from page");
        return Ok(());
    }

    info!(
        "✅ Scraping completed: {} elements extracted",
        results.len()
    );

    // 8. Save results
    info!("💾 Saving results...");
    save_results(&results, &args.output, &args.format)?;

    // Summary of downloaded assets
    let total_assets: usize = results.iter().map(|r| r.assets.len()).sum();
    if total_assets > 0 {
        info!(
            "📦 Total assets downloaded: {} (images and documents)",
            total_assets
        );
    }

    info!("🎉 Pipeline completed successfully!");
    info!("📊 Files generated: {}", args.output.display());

    Ok(())
}
