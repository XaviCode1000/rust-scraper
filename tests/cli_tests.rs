//! CLI argument parsing tests
//!
//! Tests for `Args`, `Commands`, and `Shell` types from the CLI layer.
//!
//! Run with: cargo nextest run --test-threads 2 cli_tests

use clap::Parser;
use rust_scraper::{Args, Commands, ExportFormat, OutputFormat, Shell};

// ============================================================================
// Tests: Default values
// ============================================================================

#[test]
fn test_args_defaults() {
    let args = Args::parse_from(["rust-scraper"]);
    assert!(args.url.is_none());
    assert_eq!(args.selector, "body");
    assert_eq!(args.output, std::path::PathBuf::from("output"));
    assert_eq!(args.format, OutputFormat::Markdown);
    assert_eq!(args.export_format, ExportFormat::Jsonl);
    assert_eq!(args.delay_ms, 1000);
    assert_eq!(args.max_pages, 10);
    assert_eq!(args.verbose, 0);
    assert!(!args.download_images);
    assert!(!args.download_documents);
    assert!(!args.use_sitemap);
    assert!(!args.interactive);
    assert!(!args.resume);
    assert!(!args.clean_ai);
    assert!(!args.force_js_render);
    assert!(!args.dry_run);
    assert!(!args.quiet);
    assert!(!args.obsidian_wiki_links);
    assert!(args.obsidian_tags.is_none());
    assert!(!args.obsidian_relative_assets);
    assert!(args.vault.is_none());
    assert!(!args.quick_save);
    assert!(!args.one_file_per_url);
    assert!(!args.obsidian_rich_metadata);
    assert_eq!(args.max_depth, 2);
    assert_eq!(args.timeout_secs, 30);
    assert!(args.include_patterns.is_empty());
    assert!(args.exclude_patterns.is_empty());
    assert_eq!(args.max_retries, 3);
    assert_eq!(args.backoff_base_ms, 1000);
    assert_eq!(args.backoff_max_ms, 10000);
    assert_eq!(args.max_file_size, 52428800);
    assert_eq!(args.download_timeout, 30);
    assert_eq!(args.sitemap_depth, 3);
}

// ============================================================================
// Tests: URL argument parsing
// ============================================================================

#[test]
fn test_args_url_short_flag() {
    let args = Args::parse_from(["rust-scraper", "-u", "https://example.com"]);
    assert_eq!(args.url, Some("https://example.com".to_string()));
}

#[test]
fn test_args_url_long_flag() {
    let args = Args::parse_from(["rust-scraper", "--url", "https://example.com/page"]);
    assert_eq!(args.url, Some("https://example.com/page".to_string()));
}

#[test]
fn test_args_url_with_trailing_slash() {
    let args = Args::parse_from(["rust-scraper", "-u", "https://example.com/"]);
    assert_eq!(args.url, Some("https://example.com/".to_string()));
}

#[test]
fn test_args_url_with_query_params() {
    let args = Args::parse_from([
        "rust-scraper",
        "-u",
        "https://example.com/search?q=test&page=1",
    ]);
    assert_eq!(
        args.url,
        Some("https://example.com/search?q=test&page=1".to_string())
    );
}

// ============================================================================
// Tests: Selector argument
// ============================================================================

#[test]
fn test_args_selector_short_flag() {
    let args = Args::parse_from(["rust-scraper", "-s", "article.content"]);
    assert_eq!(args.selector, "article.content");
}

#[test]
fn test_args_selector_complex_css() {
    let args = Args::parse_from([
        "rust-scraper",
        "-s",
        "main > article.post h1, main > article.post h2",
    ]);
    assert_eq!(
        args.selector,
        "main > article.post h1, main > article.post h2"
    );
}

// ============================================================================
// Tests: Output format
// ============================================================================

#[test]
fn test_args_format_markdown() {
    let args = Args::parse_from(["rust-scraper", "-f", "markdown"]);
    assert_eq!(args.format, OutputFormat::Markdown);
}

#[test]
fn test_args_format_text() {
    let args = Args::parse_from(["rust-scraper", "-f", "text"]);
    assert_eq!(args.format, OutputFormat::Text);
}

#[test]
fn test_args_format_json() {
    let args = Args::parse_from(["rust-scraper", "-f", "json"]);
    assert_eq!(args.format, OutputFormat::Json);
}

#[test]
fn test_args_format_json_lower() {
    let args = Args::parse_from(["rust-scraper", "-f", "json"]);
    assert_eq!(args.format, OutputFormat::Json);
}

// ============================================================================
// Tests: Export format
// ============================================================================

#[test]
fn test_args_export_format_jsonl() {
    let args = Args::parse_from(["rust-scraper", "--export-format", "jsonl"]);
    assert_eq!(args.export_format, ExportFormat::Jsonl);
}

#[test]
fn test_args_export_format_vector() {
    let args = Args::parse_from(["rust-scraper", "--export-format", "vector"]);
    assert_eq!(args.export_format, ExportFormat::Vector);
}

// ============================================================================
// Tests: Output directory
// ============================================================================

#[test]
fn test_args_output_nested_path() {
    let args = Args::parse_from(["rust-scraper", "-o", "data/scraped/2026"]);
    assert_eq!(args.output, std::path::PathBuf::from("data/scraped/2026"));
}

// ============================================================================
// Tests: Concurrency config
// ============================================================================

#[test]
fn test_args_concurrency_numeric() {
    let args = Args::parse_from(["rust-scraper", "--concurrency", "8"]);
    assert!(!args.concurrency.is_auto());
}

#[test]
fn test_args_concurrency_auto() {
    let args = Args::parse_from(["rust-scraper", "--concurrency", "auto"]);
    assert!(args.concurrency.is_auto());
}

// ============================================================================
// Tests: Verbose flag
// ============================================================================

#[test]
fn test_args_verbose_single() {
    let args = Args::parse_from(["rust-scraper", "-v"]);
    assert_eq!(args.verbose, 1);
}

#[test]
fn test_args_verbose_double() {
    let args = Args::parse_from(["rust-scraper", "-vv"]);
    assert_eq!(args.verbose, 2);
}

#[test]
fn test_args_verbose_triple() {
    let args = Args::parse_from(["rust-scraper", "-vvv"]);
    assert_eq!(args.verbose, 3);
}

// ============================================================================
// Tests: Crawler settings
// ============================================================================

#[test]
fn test_args_max_depth() {
    let args = Args::parse_from(["rust-scraper", "--max-depth", "5"]);
    assert_eq!(args.max_depth, 5);
}

#[test]
fn test_args_max_pages() {
    let args = Args::parse_from(["rust-scraper", "--max-pages", "500"]);
    assert_eq!(args.max_pages, 500);
}

#[test]
fn test_args_include_patterns() {
    let args = Args::parse_from([
        "rust-scraper",
        "--include-pattern",
        "*.example.com",
        "--include-pattern",
        "docs.example.com",
    ]);
    assert_eq!(args.include_patterns.len(), 2);
    assert!(args.include_patterns.contains(&"*.example.com".to_string()));
    assert!(args
        .include_patterns
        .contains(&"docs.example.com".to_string()));
}

#[test]
fn test_args_exclude_patterns() {
    let args = Args::parse_from(["rust-scraper", "--exclude-pattern", "*.admin.com"]);
    assert_eq!(args.exclude_patterns.len(), 1);
    assert!(args.exclude_patterns.contains(&"*.admin.com".to_string()));
}

// ============================================================================
// Tests: HTTP client settings
// ============================================================================

#[test]
fn test_args_max_retries() {
    let args = Args::parse_from(["rust-scraper", "--max-retries", "5"]);
    assert_eq!(args.max_retries, 5);
}

#[test]
fn test_args_accept_language() {
    let args = Args::parse_from(["rust-scraper", "--accept-language", "es-AR,es;q=0.9"]);
    assert_eq!(args.accept_language, "es-AR,es;q=0.9");
}

// ============================================================================
// Tests: Subcommands
// ============================================================================

#[test]
fn test_completions_subcommand_fish() {
    let args = Args::parse_from(["rust-scraper", "completions", "fish"]);
    assert!(matches!(
        args.subcommand,
        Some(Commands::Completions { shell: Shell::Fish })
    ));
}

#[test]
fn test_completions_subcommand_zsh() {
    let args = Args::parse_from(["rust-scraper", "completions", "zsh"]);
    assert!(matches!(
        args.subcommand,
        Some(Commands::Completions { shell: Shell::Zsh })
    ));
}

#[test]
fn test_completions_subcommand_bash() {
    let args = Args::parse_from(["rust-scraper", "completions", "bash"]);
    assert!(matches!(
        args.subcommand,
        Some(Commands::Completions { shell: Shell::Bash })
    ));
}

#[test]
fn test_completions_subcommand_powershell() {
    let args = Args::parse_from(["rust-scraper", "completions", "power-shell"]);
    assert!(matches!(
        args.subcommand,
        Some(Commands::Completions {
            shell: Shell::PowerShell
        })
    ));
}

// ============================================================================
// Tests: Boolean flags
// ============================================================================

#[test]
fn test_args_boolean_flags_default_false() {
    let args = Args::parse_from(["rust-scraper"]);
    assert!(!args.download_images);
    assert!(!args.download_documents);
    assert!(!args.use_sitemap);
    assert!(!args.interactive);
    assert!(!args.resume);
    assert!(!args.clean_ai);
    assert!(!args.force_js_render);
    assert!(!args.dry_run);
    assert!(!args.quiet);
    assert!(!args.quick_save);
    assert!(!args.one_file_per_url);
}

#[test]
fn test_args_dry_run_flag() {
    let args = Args::parse_from(["rust-scraper", "--dry-run"]);
    assert!(args.dry_run);
}

#[test]
fn test_args_quiet_flag() {
    let args = Args::parse_from(["rust-scraper", "--quiet"]);
    assert!(args.quiet);
}

// ============================================================================
// Tests: Obsidian options
// ============================================================================

#[test]
fn test_args_obsidian_wiki_links() {
    let args = Args::parse_from(["rust-scraper", "--obsidian-wiki-links"]);
    assert!(args.obsidian_wiki_links);
}

#[test]
fn test_args_obsidian_tags_parsing() {
    let args = Args::parse_from(["rust-scraper", "--obsidian-tags", "scraped,rust,web"]);
    let tags = args.obsidian_tags.expect("Tags should be set");
    assert_eq!(tags.len(), 3);
    assert_eq!(tags[0], "scraped");
    assert_eq!(tags[1], "rust");
    assert_eq!(tags[2], "web");
}

// ============================================================================
// Tests: Shell enum conversion
// ============================================================================

#[test]
fn test_shell_to_clap_complete() {
    use clap_complete::Shell as ClapCompleteShell;

    assert_eq!(
        clap_complete::Shell::from(Shell::Bash),
        ClapCompleteShell::Bash
    );
    assert_eq!(
        clap_complete::Shell::from(Shell::Fish),
        ClapCompleteShell::Fish
    );
    assert_eq!(
        clap_complete::Shell::from(Shell::Zsh),
        ClapCompleteShell::Zsh
    );
    assert_eq!(
        clap_complete::Shell::from(Shell::PowerShell),
        ClapCompleteShell::PowerShell
    );
    assert_eq!(
        clap_complete::Shell::from(Shell::Elvish),
        ClapCompleteShell::Elvish
    );
}

// ============================================================================
// Tests: Combined arguments
// ============================================================================

#[test]
fn test_args_combined_flags() {
    let args = Args::parse_from([
        "rust-scraper",
        "-u",
        "https://example.com",
        "-s",
        "article",
        "-o",
        "output",
        "-f",
        "markdown",
        "-vv",
        "--max-pages",
        "50",
        "--delay-ms",
        "1000",
        "--obsidian-wiki-links",
        "--dry-run",
    ]);

    assert_eq!(args.url, Some("https://example.com".to_string()));
    assert_eq!(args.selector, "article");
    assert_eq!(args.output, std::path::PathBuf::from("output"));
    assert_eq!(args.format, OutputFormat::Markdown);
    assert_eq!(args.verbose, 2);
    assert_eq!(args.max_pages, 50);
    assert_eq!(args.delay_ms, 1000);
    assert!(args.obsidian_wiki_links);
    assert!(args.dry_run);
}
