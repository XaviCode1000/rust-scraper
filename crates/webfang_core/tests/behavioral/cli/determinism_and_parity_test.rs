use crate::BehavioralTest;
use std::collections::HashSet;
use wiremock::matchers::{method, path};
use wiremock::{Mock, ResponseTemplate};

/// Sets up a simple two-level fixture for testing discovery behavior.
/// Structure:
///   / (seed) -> /a, /b (depth 1)
///   /a -> /a1, /a2 (depth 2)
///   /b -> /b1, /b2 (depth 2)
/// Each page has sufficient content to pass the minimum-content guard.
async fn setup_two_level_fixture() -> BehavioralTest {
    let t = BehavioralTest::new().await;
    
    // Seed page links to two depth-1 pages
    Mock::given(method("GET"))
        .and(path("/"))
        .respond_with(ResponseTemplate::new(200).set_body_string(
            r#"<html><body>
                <h1>Seed Page</h1>
                <p>This is the seed page with sufficient content to pass the minimum-content guard. It contains enough text to be considered useful for extraction.</p>
                <a href="/a">a</a>
                <a href="/b">b</a>
            </body></html>"#
        ))
        .mount(&t.server)
        .await;
    
    // Depth-1 page "a" links to two depth-2 pages
    Mock::given(method("GET"))
        .and(path("/a"))
        .respond_with(ResponseTemplate::new(200).set_body_string(
            r#"<html><body>
                <h1>Page A</h1>
                <p>This is the first depth-1 page with sufficient content to pass the minimum-content guard. It contains enough text to be considered useful for extraction.</p>
                <a href="/a1">a1</a>
                <a href="/a2">a2</a>
            </body></html>"#
        ))
        .mount(&t.server)
        .await;
        
    // Depth-1 page "b" links to two depth-2 pages
    Mock::given(method("GET"))
        .and(path("/b"))
        .respond_with(ResponseTemplate::new(200).set_body_string(
            r#"<html><body>
                <h1>Page B</h1>
                <p>This is the second depth-1 page with sufficient content to pass the minimum-content guard. It contains enough text to be considered useful for extraction.</p>
                <a href="/b1">b1</a>
                <a href="/b2">b2</a>
            </body></html>"#
        ))
        .mount(&t.server)
        .await;
    
    // Depth-2 pages (leaves) with sufficient content
    for (page, name) in &[("a1", "Leaf A1"), ("a2", "Leaf A2"), ("b1", "Leaf B1"), ("b2", "Leaf B2")] {
        Mock::given(method("GET"))
            .and(path(format!("/{page}")))
            .respond_with(ResponseTemplate::new(200).set_body_string(
                format!(
                    r#"<html><body>
                        <h1>{}</h1>
                        <p>This is a leaf page with sufficient content to pass the minimum-content guard. It contains enough text to be considered useful for extraction.</p>
                    </body></html>"#,
                    name
                )
            ))
            .mount(&t.server)
            .await;
    }
    
    t
}

/// Collect all markdown files from output and return their contents as a sorted set.
fn collect_output_contents(test: &BehavioralTest) -> HashSet<String> {
    let md_files = test.find_files("md");
    let mut contents = HashSet::new();
    
    for file in md_files {
        if let Ok(content) = std::fs::read_to_string(file) {
            contents.insert(content);
        }
    }
    
    contents
}

/// Collect all URLs discovered during a crawl by checking the manifest or similar.
/// For this test, we'll look for a manifest file that lists discovered URLs.
fn collect_discovered_urls(test: &BehavioralTest) -> HashSet<String> {
    // Look for manifest files that might contain URL information
    let manifest_files = test.find_files("manifest");
    let mut urls = HashSet::new();
    
    for file in manifest_files {
        if let Ok(content) = std::fs::read_to_string(&file) {
            for line in content.lines() {
                // Skip comments and empty lines
                let line = line.trim();
                if line.is_empty() || line.starts_with('#') {
                    continue;
                }
                urls.insert(line.to_string());
            }
        }
    }
    
    urls
}

#[tokio::test]
async fn repeated_no_checkpoint_crawls_are_deterministic() {
    // Issue #1237: F-13 nondeterministic output set
    // Repeated --no-checkpoint crawls should yield identical output sets
    
    let t = setup_two_level_fixture().await;
    
    // First crawl
    t.scraper_cmd()
        .arg("--no-checkpoint")
        .arg("--max-depth=2")
        .arg("--max-pages=10")
        .arg("--quiet")
        .assert()
        .success();
    
    let first_run_output = collect_output_contents(&t);
    let first_run_urls = collect_discovered_urls(&t);
    
    // Second crawl with same parameters
    t.scraper_cmd()
        .arg("--no-checkpoint")
        .arg("--max-depth=2")
        .arg("--max-pages=10")
        .arg("--quiet")
        .assert()
        .success();
    
    let second_run_output = collect_output_contents(&t);
    let second_run_urls = collect_discovered_urls(&t);
    
    // Output contents should be identical
    assert_eq!(
        first_run_output, second_run_output,
        "Repeated --no-checkpoint crawls should produce identical output content\n\
         First run: {:?}\n\
         Second run: {:?}",
        first_run_output, second_run_output
    );
    
    // Discovered URL sets should be identical
    assert_eq!(
        first_run_urls, second_run_urls,
        "Repeated --no-checkpoint crawls should discover identical URL sets\n\
         First run URLs: {:?}\n\
         Second run URLs: {:?}",
        first_run_urls, second_run_urls
    );
}

#[tokio::test]
async fn dry_run_matches_real_discovery() {
    // Issue #1232: F-14 dry-run vs real discovery divergence
    // --dry-run should produce the same URL set as a real run
    
    let t = setup_two_level_fixture().await;
    
    // Dry run
    t.scraper_cmd()
        .arg("--dry-run")
        .arg("--max-depth=2")
        .arg("--max-pages=10")
        .arg("--quiet")
        .assert()
        .success();
    
    let dry_run_urls = collect_discovered_urls(&t);
    
    // Real run (use scraper_cmd as-is, which already includes --output)
    t.scraper_cmd()
        .arg("--max-depth=2")
        .arg("--max-pages=10")
        .arg("--quiet")
        .assert()
        .success();
    
    let real_run_urls = collect_discovered_urls(&t);
    
    // URL sets should be identical
    assert_eq!(
        dry_run_urls, real_run_urls,
        "--dry-run and real crawl should discover identical URL sets\n\
         Dry run URLs: {:?}\n\
         Real run URLs: {:?}",
        dry_run_urls, real_run_urls
    );
    
    // Neither should be empty (we expect to find URLs)
    assert!(!dry_run_urls.is_empty(), "Dry run should discover some URLs");
    assert!(!real_run_urls.is_empty(), "Real run should discover some URLs");
}

#[tokio::test]
async fn dry_run_produces_no_output_files() {
    // Additional check: dry-run should not produce any output files
    
    let t = setup_two_level_fixture().await;
    
    t.scraper_cmd()
        .arg("--dry-run")
        .arg("--max-depth=2")
        .arg("--max-pages=10")
        .arg("--quiet")
        .assert()
        .success();
    
    let output_files: Vec<_> = std::fs::read_dir(t.out.path())
        .unwrap()
        .filter_map(|e| e.ok())
        .collect();
    
    assert!(
        output_files.is_empty(),
        "Dry-run should not create any output files, found {}",
        output_files.len()
    );
}
