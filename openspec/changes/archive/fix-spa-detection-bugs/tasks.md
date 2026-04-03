# Tasks: Fix SPA Detection Bugs

## 1. Refactor `detect_spa_content()` signature and logic

- [x] 1.1. Change function signature: `detect_spa_content(url: &str, text_content: &str, raw_html: &str) -> Option<SpaDetectionResult>`
- [x] 1.2. Move SPA marker detection (`<div id="root">`, `<div id="app">`) to search in `raw_html` instead of `text_content`
- [x] 1.3. Remove `has_empty_title` field from `SpaDetectionResult` struct
- [x] 1.4. Remove hostname-based title heuristic code
- [x] 1.5. Update existing unit tests for new signature

## 2. Update call sites in `scrape_with_config()`

- [x] 2.1. Update call at line ~177 (after Readability success) to pass `&html` as raw_html
- [x] 2.2. Update call at line ~201 (after fallback) to pass `&html` as raw_html

## 3. Differentiate warning messages

- [x] 3.1. If `has_spa_markers` is true: warn with "SPA markers detected" message
- [x] 3.2. If `has_spa_markers` is false: warn with "minimal content" message
- [x] 3.3. Add test for differentiated warnings

## 4. Verify

- [x] 4.1. `cargo nextest run --test-threads 2` passes
- [x] 4.2. `cargo clippy -- -D warnings` passes clean (NOTE: pre-existing error in vector_exporter.rs unrelated to changes)
- [x] 4.3. `cargo fmt --check` passes
