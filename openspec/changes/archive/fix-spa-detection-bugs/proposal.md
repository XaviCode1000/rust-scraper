# Proposal: Fix SPA Detection Bugs

## Intent

The SPA detection feature (`detect_spa_content`) has 3 bugs that render it partially non-functional. SPA markers (`<div id="root">`) are searched in extracted **text** instead of raw HTML, so they never match. The `has_empty_title` heuristic checks the hostname, not the actual `<title>` tag. Warning messages don't differentiate between short content and confirmed SPA markers.

These bugs were found during a GitNexus code audit (Issue #20). The detection fires but produces misleading diagnostics.

## Scope

### In Scope
- **Bug 1 (HIGH):** Pass raw HTML to `detect_spa_content()` for marker detection while keeping extracted text for char count threshold
- **Bug 2 (MEDIUM):** Remove dead `has_empty_title` code that analyzes hostname instead of HTML title tag, or implement proper title parsing
- **Bug 3 (LOW):** Differentiate warning messages between "short content" and "SPA markers detected"
- Update tests to match new function signature and behavior

### Out of Scope
- Actual JS rendering implementation (tracked in Issue #16)
- `JsRenderer` trait implementation (forward-compatible stub, remains unchanged)
- Headless browser integration

## Capabilities

### New Capabilities
None.

### Modified Capabilities
- `spa-detection`: Signature change to accept both raw HTML and extracted text; improved diagnostic accuracy and warning specificity

## Approach

1. Change `detect_spa_content()` signature to accept both `raw_html: &str` (for marker detection) and `text_content: &str` (for char count threshold)
2. SPA markers (`<div id="root">`, `<div id="app">`) searched in `raw_html`
3. Remove or fix `has_empty_title` — if kept, parse `<title>` from raw HTML; if unused, delete the field
4. Update call sites in `scrape_with_config()` (lines 177, 201) to pass both `html` and extracted text
5. Differentiate warning messages based on `has_spa_markers` vs just low char count
6. Update existing tests and add cases for raw HTML marker detection

## Affected Areas

| Area | Impact | Description |
|------|--------|-------------|
| `src/domain/js_renderer.rs` | Unchanged | Trait stub remains as-is; no implementation needed |
| `src/application/scraper_service.rs` | Modified | `detect_spa_content()` signature, `SpaDetectionResult` struct, call sites, warning messages, tests |

## Risks

| Risk | Likelihood | Mitigation |
|------|------------|------------|
| Signature change breaks external callers | Low | Only called internally within `scraper_service.rs` |
| Raw HTML parsing for title adds complexity | Low | Use existing `scraper` crate already in deps |
| False positives increase with raw HTML markers | Medium | Keep conservative — markers are strong signals, not sole criteria |

## Rollback Plan

Revert the commit. The change is isolated to `scraper_service.rs` — no data migration, no public API change, no config changes. Previous behavior (searching markers in text, which never matched) is restored.

## Dependencies

- None beyond existing project dependencies (`scraper` crate already used for HTML parsing)

## Success Criteria

- [ ] SPA markers `<div id="root">` and `<div id="app">` are detected in raw HTML, not stripped text
- [ ] `has_empty_title` either correctly parses `<title>` tag or is removed as dead code
- [ ] Warning messages differentiate between "minimal content" and "SPA markers found"
- [ ] All existing tests pass with `cargo nextest run --test-threads 2`
- [ ] `cargo clippy -- -D warnings` passes clean
