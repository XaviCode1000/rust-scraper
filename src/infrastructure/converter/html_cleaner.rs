//! HTML boilerplate removal before Markdown conversion.
//!
//! Uses `html-cleaning` to strip navigation, sidebars, scripts,
//! SVGs, and other non-content elements before converting to Markdown.

/// Clean HTML by removing boilerplate (nav, sidebar, scripts, SVGs).
///
/// Removes:
/// - `script`, `style`, `noscript` (code and styles)
/// - `form`, `iframe`, `object`, `embed` (interactive)
/// - `svg`, `canvas`, `video`, `audio` (media)
/// - `nav`, `header`, `footer`, `aside` (page chrome)
/// - Prunes empty elements, normalizes whitespace, strips attributes
///
/// Returns the cleaned HTML as a string.
pub fn clean_html(html: &str) -> String {
    use html_cleaning::HtmlCleaner;

    let options = html_cleaning::CleaningOptions {
        tags_to_remove: vec![
            // Scripts and styles
            "script".into(),
            "style".into(),
            "noscript".into(),
            // Interactive/embedded
            "form".into(),
            "iframe".into(),
            "object".into(),
            "embed".into(),
            // Media (SVGs, canvas, video, audio)
            "svg".into(),
            "canvas".into(),
            "video".into(),
            "audio".into(),
            // Page chrome (navigation, header, footer, sidebar)
            "nav".into(),
            "header".into(),
            "footer".into(),
            "aside".into(),
        ],
        selectors_to_remove: vec![
            // Starlight/Astro navigation and sidebar
            ".site-title".into(),
            ".global-nav".into(),
            ".global-nav-list".into(),
            ".mobile-menu-wrapper".into(),
            ".right-sidebar".into(),
            ".right-sidebar-container".into(),
            ".mobile-toc".into(),
            ".sl-sidebar".into(),
            ".sl-mobile-toc".into(),
            // Search and feedback
            ".search".into(),
            ".site-search".into(),
            ".social-icons".into(),
            ".page-feedback".into(),
            ".feedback".into(),
            // Breadcrumb and pagination
            ".sl-breadcrumbs".into(),
            ".pagination".into(),
            // Meta tags and hidden elements
            "[class*='sr-only']".into(),
            "[aria-hidden='true']".into(),
            "[hidden]".into(),
            // Copy-to-clipboard and utility buttons
            ".copy-markdown-btn".into(),
            ".copy-code-button".into(),
            // Skip links
            ".skip-link".into(),
        ],
        prune_empty: true,
        normalize_whitespace: true,
        strip_attributes: true,
        preserved_attributes: vec![
            "href".into(),
            "src".into(),
            "alt".into(),
            "id".into(),
            "class".into(),
            "dir".into(),
            "code".into(),
        ],
        ..Default::default()
    };

    let cleaner = HtmlCleaner::with_options(options);
    let doc = dom_query::Document::from(html);
    cleaner.clean(&doc);
    doc.html().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_removes_scripts() {
        let html = "<html><body><script>alert(1)</script><p>Hello</p></body></html>";
        let cleaned = clean_html(html);
        assert!(!cleaned.contains("<script>"));
        assert!(cleaned.contains("Hello"));
    }

    #[test]
    fn test_clean_removes_svg() {
        let html = "<html><body><nav><svg>icon</svg></nav><article><h1>Title</h1></article></body></html>";
        let cleaned = clean_html(html);
        assert!(!cleaned.contains("<svg>"));
        assert!(!cleaned.contains("<nav>"));
    }

    #[test]
    fn test_clean_preserves_content() {
        let html = "<html><body><nav>Menu</nav><main><h1>Article</h1><p>Content here</p></main></body></html>";
        let cleaned = clean_html(html);
        assert!(cleaned.contains("Article"));
        assert!(cleaned.contains("Content here"));
        assert!(!cleaned.contains("Menu"));
    }

    #[test]
    fn test_clean_empty_html() {
        let html = "";
        let cleaned = clean_html(html);
        // Should not panic
        assert!(cleaned.is_empty() || cleaned.contains("<html>"));
    }
}
