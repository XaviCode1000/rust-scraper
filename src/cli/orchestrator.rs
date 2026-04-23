//! CLI orchestrator — coordinates the main scraping pipeline.
//!
//! Orchestrates URL discovery, scraping, and export phases.

use tracing::info;

use crate::cli::error::CliExit;
use crate::cli::completions::generate_completions;
use crate::cli::export_flow::{run_export, save_files, ExportConfig};
use crate::cli::scrape_flow::scrape_urls;
use crate::cli::url_discovery::discover_urls;
use crate::Args;
use crate::ScraperConfig;
use crate::CrawlerConfig;

use crate::infrastructure::output::file_saver::ObsidianOptions;
use crate::Shell;
use crate::domain;

/// Handle shell completion generation.
pub fn handle_completions(shell: Shell) -> CliExit {
    let clap_shell = match shell {
        Shell::Bash => clap_complete::Shell::Bash,
        Shell::Elvish => clap_complete::Shell::Elvish,
        Shell::Fish => clap_complete::Shell::Fish,
        Shell::PowerShell => clap_complete::Shell::PowerShell,
        Shell::Zsh => clap_complete::Shell::Zsh,
    };
    generate_completions::<Args>(clap_shell)
        .map(|_| CliExit::Success)
        .unwrap_or_else(|_| CliExit::UsageError("completion generation failed".into()))
}

/// Main orchestration entry point.
///
/// Coordinates the full scraping pipeline:
/// 1. URL discovery
/// 2. Scraping with progress
/// 3. Export results
pub async fn run(args: Args) -> CliExit {
    let target_url_str = match args.url.as_ref() {
        Some(url) => url,
        None => return CliExit::UsageError("--url is required".into()),
    };

    let target_url = match url::Url::parse(target_url_str) {
        Ok(url) => url,
        Err(e) => return CliExit::UsageError(format!("Invalid URL: {}", e)),
    };

    // Create crawler config from args
    let crawler_config = CrawlerConfig::builder(target_url.clone())
        .max_pages(args.max_pages)
        .max_depth(args.max_depth)
        .include_patterns(args.include_patterns.clone())
        .exclude_patterns(args.exclude_patterns.clone())
        .build();

    // URL discovery phase
    let discovered_urls: Vec<url::Url> = discover_urls(&crawler_config, &args).await;
    if discovered_urls.is_empty() {
        info!("No URLs discovered");
        return CliExit::Success;
    }

    let urls_to_scrape = discovered_urls;

    // Create scraper config
    let scraper_config = ScraperConfig::default()
        .with_output_dir(args.output.clone())
        .with_scraper_concurrency(args.concurrency.resolve())
        .with_max_pages(args.max_pages);

    // Scraping phase


    let (results, failures): (Vec<domain::ScrapedContent>, Vec<(String, String)>) = scrape_urls(&urls_to_scrape, &scraper_config, &args, None).await;

    // Report failures
    for (url, error) in &failures {
        eprintln!("Failed to scrape {}: {}", url, error);
    }

    if results.is_empty() {
        eprintln!("No pages were successfully scraped");
        return CliExit::NetworkError("No pages were successfully scraped".into());
    }

    info!("Successfully scraped {} pages", results.len());

    // Obsidian options
    let obsidian_options = ObsidianOptions {
        wiki_links: args.obsidian_wiki_links,
        relative_assets: args.obsidian_relative_assets,
        tags: args.obsidian_tags.clone().unwrap_or_default(),
        rich_metadata: args.obsidian_rich_metadata,
        quick_save: args.quick_save,
        vault_path: args.vault.clone(),
    };

    // Determine output directory for individual files
    let output_dir = if args.quick_save && args.vault.is_some() {
        let vault = args.vault.as_ref().unwrap();
        let inbox = vault.join("_inbox");
        if !inbox.exists() {
            let _ = std::fs::create_dir_all(&inbox);
        }
        inbox
    } else {
        args.output.clone()
    };

    // Export phase
    let export_config = ExportConfig {
        results: &results,
        output_dir: args.output.clone(), // RAG export still goes to output_dir
        format: args.format,
        export_format: args.export_format,
        clean_ai: args.clean_ai,
        quick_save: args.quick_save,
        vault_path: args.vault.as_ref(),
        obsidian_options: obsidian_options.clone(),
        state_store: None, // TODO: Add state store
        resume: args.resume,
        ai_threshold: 0.3, // TODO: Add AI settings from args
        ai_max_tokens: 512,
        ai_offline: false,
    };

    // Save individual files (Markdown, etc.)
    save_files(&results, &output_dir, &args.format, &obsidian_options);

    match run_export(export_config).await {
        Ok(processed_urls) => {
            info!("Export completed for {} URLs", processed_urls.len());
            CliExit::Success
        }
        Err(e) => {
            eprintln!("Export failed: {:?}", e);
            e
        }
    }
}