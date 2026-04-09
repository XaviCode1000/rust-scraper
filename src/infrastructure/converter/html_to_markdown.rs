//! HTML to Markdown conversion
//!
//! Uses html-to-markdown-rs crate for structure-preserving conversion.
//! HTML boilerplate (nav, sidebar, SVG, scripts) is stripped before
//! conversion for Obsidian-quality output.

use html_to_markdown_rs::{convert, CodeBlockStyle, ConversionOptions, HeadingStyle};
use tracing::warn;

/// Convert HTML to well-structured Markdown.
///
/// Pipeline:
/// 1. Remove boilerplate (scripts, nav, sidebar, SVG, page chrome)
/// 2. Convert clean HTML → Markdown with ATX headings and fenced code blocks
/// 3. Fall back to plain text if conversion fails
///
/// # Examples
///
/// ```
/// use rust_scraper::infrastructure::converter::html_to_markdown::convert_to_markdown;
///
/// let html = "<h1>Title</h1><p>Content</p>";
/// let md = convert_to_markdown(html);
/// assert!(md.contains("# Title"));
/// ```
pub fn convert_to_markdown(html: &str) -> String {
    // Step 1: Remove boilerplate (nav, sidebar, scripts, SVG, etc.)
    let cleaned = crate::infrastructure::converter::html_cleaner::clean_html(html);

    // Step 2: Convert clean HTML → Markdown
    let options = ConversionOptions {
        heading_style: HeadingStyle::Atx,
        code_block_style: CodeBlockStyle::Backticks,
        ..Default::default()
    };

    convert(&cleaned, Some(options)).unwrap_or_else(|e| {
        warn!("HTML to Markdown conversion failed: {}, falling back", e);
        crate::infrastructure::scraper::fallback::extract_text(html)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_convert_heading() {
        let html = "<h1>Main Title</h1>";
        let md = convert_to_markdown(html);
        assert!(md.contains("# Main Title"));
    }

    #[test]
    fn test_convert_paragraph() {
        let html = "<p>This is a paragraph.</p>";
        let md = convert_to_markdown(html);
        assert!(md.contains("This is a paragraph."));
    }

    #[test]
    fn test_convert_nested_structure() {
        let html = "<article><h1>Title</h1><p>Intro</p><h2>Section</h2><p>Content</p></article>";
        let md = convert_to_markdown(html);
        assert!(md.contains("# Title"));
        assert!(md.contains("## Section"));
        assert!(md.contains("Intro"));
        assert!(md.contains("Content"));
    }

    #[test]
    fn test_convert_empty_html() {
        let html = "";
        let md = convert_to_markdown(html);
        assert_eq!(md, "");
    }

    #[test]
    fn test_code_block_uses_backticks() {
        let html = "<pre><code>fn main() {}</code></pre>";
        let md = convert_to_markdown(html);
        assert!(md.contains("```"), "Expected fenced code blocks, got: {}", md);
    }

    #[test]
    fn test_boilerplate_removed() {
        let html = "<html><body><nav>Menu</nav><main><h1>Title</h1><p>Content</p></main><footer>Copyright</footer></body></html>";
        let md = convert_to_markdown(html);
        assert!(!md.contains("Menu"));
        assert!(!md.contains("Copyright"));
        assert!(md.contains("Title"));
        assert!(md.contains("Content"));
    }
}
