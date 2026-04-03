# Spec: Fix SPA Detection Bugs

## Requirement 1: SPA markers detected in raw HTML

The function `detect_spa_content()` SHALL search for SPA mount point markers (`<div id="root">`, `<div id="app">`) in the raw HTML source, NOT in extracted text content.

### Scenarios

**Given** a page with raw HTML containing `<div id="root"></div>`
**When** `detect_spa_content()` is called with that raw HTML
**Then** `has_spa_markers` SHALL be `true`

**Given** a page with raw HTML containing `<div id="app"></div>`
**When** `detect_spa_content()` is called with that raw HTML
**Then** `has_spa_markers` SHALL be `true`

**Given** a page with raw HTML that has no SPA markers
**When** `detect_spa_content()` is called
**Then** `has_spa_markers` SHALL be `false`

**Given** extracted text content below 50 chars
**When** raw HTML has no SPA markers
**Then** the page is flagged for minimal content but `has_spa_markers` is `false`

## Requirement 2: has_empty_title removed as dead code

The `has_empty_title` field in `SpaDetectionResult` SHALL be removed. It currently analyzes the hostname instead of the HTML `<title>` tag, producing meaningless results, and the field is never read by any caller.

### Scenarios

**Given** the `SpaDetectionResult` struct
**Then** it SHALL NOT contain a `has_empty_title` field

**Given** the `detect_spa_content()` function
**Then** it SHALL NOT compute any hostname-based title heuristic

## Requirement 3: Differentiated warning messages

The warning emitted by `scrape_with_config()` SHALL distinguish between pages with minimal content and pages with confirmed SPA markers.

### Scenarios

**Given** a page with content below 50 chars AND SPA markers detected
**When** `scrape_with_config()` processes the page
**Then** the warning SHALL mention "SPA markers detected" and list the markers found

**Given** a page with content below 50 chars but NO SPA markers
**When** `scrape_with_config()` processes the page
**Then** the warning SHALL mention "minimal content" without claiming SPA markers

**Given** a page with content above 50 chars
**When** `scrape_with_config()` processes the page
**Then** NO SPA warning SHALL be emitted

## Requirement 4: Tests updated for new signature

All existing tests for `detect_spa_content()` SHALL pass with the new function signature that accepts both raw HTML and text content.

### Scenarios

**Given** the test `test_detect_spa_content_spa_markers`
**When** called with raw HTML containing `<div id="root">`
**Then** `has_spa_markers` is `true`

**Given** the test `test_detect_spa_content_below_threshold`
**When** called with empty text and empty HTML
**Then** detection returns `Some` with `char_count` of 0

**Given** boundary tests at 49, 50, and 51 chars
**When** raw HTML has no SPA markers
**Then** only the 49-char case triggers detection
