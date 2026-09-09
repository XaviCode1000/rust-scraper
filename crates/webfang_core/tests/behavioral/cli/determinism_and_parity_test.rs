use crate::BehavioralTest;
use std::collections::BTreeSet;
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
            </body></html>"#,
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
            </body></html>"#,
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
            </body></html>"#,
        ))
        .mount(&t.server)
        .await;

    // Depth-2 pages (leaves) with sufficient content
    for (page, name) in &[
        ("a1", "Leaf A1"),
        ("a2", "Leaf A2"),
        ("b1", "Leaf B1"),
        ("b2", "Leaf B2"),
    ] {
        Mock::given(method("GET"))
            .and(path(format!("/{page}")))
            .respond_with(ResponseTemplate::new(200).set_body_string(format!(
                r#"<html><body>
                    <h1>{name}</h1>
                    <p>This is a leaf page with sufficient content to pass the minimum-content guard. It contains enough text to be considered useful for extraction.</p>
                </body></html>"#
            )))
            .mount(&t.server)
            .await;
    }

    t
}

/// Paths this fixture serves. Compared as sets so discovery+scrape double
/// fetch (F-05, narrowed) cannot fail the test — only the URL SET matters.
const FIXTURE_PATHS: &[&str] = &["/", "/a", "/b", "/a1", "/a2", "/b1", "/b2"];

/// Parse the `  {url}` lines `--dry-run` prints to stdout into a path set.
fn dry_run_paths(stdout: &str, server_base: &str) -> BTreeSet<String> {
    stdout
        .lines()
        .filter_map(|line| {
            let trimmed = line.trim();
            trimmed
                .strip_prefix(server_base)
                .map(|rest| {
                    if rest.is_empty() {
                        "/".to_string()
                    } else {
                        rest.to_string()
                    }
                })
                .filter(|p| FIXTURE_PATHS.contains(&p.as_str()))
        })
        .collect()
}

#[tokio::test]
async fn dry_run_matches_real_discovery() {
    // Issue #1232 (F-14): `--dry-run` must print the same URL set the real
    // crawl fetches. Fixed by #1248 (`discover_urls_unified`); this test pins
    // the contract. Existing `dry_run_test.rs` covers zero-files and
    // discovery-request-only, but not URL-set parity — that gap is this test.
    let t = setup_two_level_fixture().await;
    let server_base = t.server.uri();

    // Dry run → stdout URL set (no output files expected).
    let dry_out = t
        .scraper_cmd()
        .arg("--dry-run")
        .arg("--max-depth=2")
        .arg("--max-pages=10")
        .arg("--quiet")
        .output()
        .expect("dry-run executes");
    assert!(
        dry_out.status.success(),
        "dry-run exits 0, stderr: {}",
        String::from_utf8_lossy(&dry_out.stderr)
    );
    let dry_paths = dry_run_paths(&String::from_utf8_lossy(&dry_out.stdout), &server_base);
    assert_eq!(
        dry_paths,
        FIXTURE_PATHS.iter().map(|s| s.to_string()).collect(),
        "dry-run discovers the whole two-level fixture"
    );

    let requests_before = t.server.received_requests().await.map(|r| r.len());

    // Real run.
    t.scraper_cmd()
        .arg("--no-checkpoint")
        .arg("--max-depth=2")
        .arg("--max-pages=10")
        .arg("--quiet")
        .assert()
        .success();

    // Real-run fetched set, from the fixture server's received requests after
    // the dry run (robots.txt misses are filtered by FIXTURE_PATHS).
    let after = requests_before.unwrap_or_default();
    let fetched: BTreeSet<String> = t
        .server
        .received_requests()
        .await
        .unwrap_or_default()
        .into_iter()
        .skip(after)
        .map(|req| req.url.path().to_string())
        .filter(|p| FIXTURE_PATHS.contains(&p.as_str()))
        .collect();

    assert_eq!(
        dry_paths, fetched,
        "--dry-run and real crawl discover identical URL sets"
    );
}
