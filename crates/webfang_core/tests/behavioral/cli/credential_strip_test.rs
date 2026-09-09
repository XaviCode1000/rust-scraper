//! Credential-strip regression tests (MISSION F-31).
//!
//! Observable behavior: userinfo embedded in the seed `--url` must never
//! persist in any file written under `--output` (markdown, exports, trace,
//! checkpoint/state), and non-HTTP(S) schemes must be rejected uniformly by
//! `ValidUrl`.

use crate::cmd;
use crate::BehavioralTest;
use walkdir::WalkDir;
use wiremock::matchers::{method, path};
use wiremock::{Mock, ResponseTemplate};

const SEED_HTML: &str = r#"
<html><head><title>Credential Strip Test</title></head>
<body><main><article>
<h1>Hello World</h1>
<p>This is meaningful content for the extractor to process.</p>
<p>More paragraphs ensure readability can extract a proper document.</p>
</article></main></body></html>
"#;

/// A seed URL carrying `user:SECRETPASS@` userinfo must scrape successfully
/// (wiremock matches on path only) while the secret never lands in any file
/// under the output directory — markdown, exports, trace JSONL, or state.
#[tokio::test]
async fn credential_userinfo_never_persists_in_outputs() {
    // --- Arrange ---
    let t = BehavioralTest::new().await;

    Mock::given(method("GET"))
        .and(path("/page"))
        .respond_with(ResponseTemplate::new(200).set_body_string(SEED_HTML))
        .mount(&t.server)
        .await;

    // Inject userinfo into the mock-server URI. Wiremock routes on path, so
    // the mock keeps responding while the binary receives a credentialed URL.
    let host_port = t
        .server
        .uri()
        .strip_prefix("http://")
        .expect("mock server uri is http")
        .to_owned();
    let cred_url = format!("http://user:SECRETPASS@{host_port}/page");
    let trace_path = t.out.path().join("trace.jsonl");

    // --- Act ---
    cmd()
        .arg("--url")
        .arg(&cred_url)
        .arg("--output")
        .arg(t.out.path())
        .arg("--single-page")
        .arg("--trace-file")
        .arg(&trace_path)
        .arg("--quiet")
        .assert()
        .success();

    // --- Assert ---
    assert!(
        trace_path.is_file(),
        "trace file must exist at {}",
        trace_path.display()
    );

    let mut checked = 0usize;
    for entry in WalkDir::new(t.out.path())
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| e.file_type().is_file())
    {
        let bytes = std::fs::read(entry.path())
            .unwrap_or_else(|e| panic!("read {}: {e}", entry.path().display()));
        let contents = String::from_utf8_lossy(&bytes);
        assert!(
            !contents.contains("SECRETPASS"),
            "file {} leaks credentials",
            entry.path().display()
        );
        checked += 1;
    }
    assert!(
        checked >= 2,
        "expected at least the scraped .md plus trace.jsonl under output, got {checked}"
    );
}

/// `ValidUrl::parse` and `ValidUrl::try_from_url` must reject non-HTTP(S)
/// schemes uniformly — `data:`, `file:`, and `ftp:` are WHATWG-valid URLs
/// but nothing the crawler can fetch.
#[test]
fn non_http_schemes_rejected_uniformly() {
    use webfang_core::domain::ValidUrl;

    for raw in [
        "data:text/html,<h1>hi</h1>",
        "file:///etc/passwd",
        "ftp://example.com/file",
    ] {
        assert!(
            ValidUrl::parse(raw).is_err(),
            "ValidUrl::parse must reject {raw}"
        );
        let url = url::Url::parse(raw).unwrap();
        assert!(
            ValidUrl::try_from_url(url).is_err(),
            "ValidUrl::try_from_url must reject {raw}"
        );
    }
}
