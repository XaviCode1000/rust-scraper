//! CLI binary integration tests
//!
//! Tests the actual `rust-scraper` binary using `assert_cmd`.
//! These tests verify the binary behaves correctly for edge cases
//! without requiring network access.
//!
//! Run with: cargo nextest run --test-threads 2 cli_binary_test

use assert_cmd::Command;
use predicates::prelude::*;

fn cmd() -> Command {
    Command::cargo_bin("rust_scraper").expect("binary exists")
}

// ============================================================================
// Tests: Binary error handling
// ============================================================================

/// Test that running without --url shows an error message
#[test]
fn test_no_url_shows_error() {
    cmd()
        .assert()
        .failure()
        .stderr(predicate::str::contains("--url is required"));
}

/// Test that an invalid URL shows an error
#[test]
fn test_invalid_url_shows_error() {
    cmd()
        .arg("--url")
        .arg("not-a-url")
        .assert()
        .failure()
        .stderr(predicate::str::contains("URL inv\u{00e1}lida"));
}

// ============================================================================
// Tests: Binary help and version
// ============================================================================

/// Test that --help contains scraper description
///
/// Note: --help exits with code 64 because the app uses try_parse()
/// which treats DisplayHelp as an error. The help text is still printed.
#[test]
fn test_help_contains_scraper() {
    // The help text goes to stderr due to try_parse() behavior
    cmd()
        .arg("--help")
        .assert()
        .code(64)
        .stderr(predicate::str::contains("web scraper"));
}

/// Test that --version outputs something
#[test]
fn test_version() {
    cmd()
        .arg("--version")
        .assert()
        .code(64)
        .stderr(predicate::str::contains("1.1.0"));
}

// ============================================================================
// Tests: Dry-run mode
// ============================================================================

/// Test that --dry-run with a valid URL does not fail (but may fail on network)
#[test]
#[ignore = "requires network access"]
fn test_dry_run_with_url() {
    cmd()
        .arg("--url")
        .arg("https://example.com")
        .arg("--dry-run")
        .assert()
        .success();
}

// ============================================================================
// Tests: Feature flags
// ============================================================================

/// Test that --quiet flag is accepted
#[test]
fn test_quiet_flag_accepted() {
    // Should not fail at argument parsing (will fail at network without URL)
    cmd()
        .arg("--quiet")
        .assert()
        .failure()
        .stderr(predicate::str::contains("--url is required"));
}

/// Test that --dry-run flag is accepted
#[test]
fn test_dry_run_flag_accepted() {
    cmd()
        .arg("--dry-run")
        .assert()
        .failure()
        .stderr(predicate::str::contains("--url is required"));
}
