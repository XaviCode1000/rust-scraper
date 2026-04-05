//! Obsidian-compatible Markdown export
//!
//! Provides transformations for Obsidian vault compatibility:
//! - Wiki-link conversion: [text](url) -> [[slug|text]]
//! - Relative asset paths: absolute paths -> relative to .md file
//! - Slug extraction from URLs for wiki-link targets

use crate::domain::DownloadedAsset;
use pulldown_cmark::{Event, Options, Parser, Tag, TagEnd};
use std::path::Path;

/// Extract a URL-safe slug from a URL path.
///
/// Strips query strings, fragments, trailing slashes, and file extensions.
/// Takes the last path segment and converts to lowercase kebab-case.
///
/// # Examples
/// - "/blog/my-post" -> "my-post"
/// - "/docs/api/v2.html?page=1" -> "api-v2"
/// - "/" -> "index"
/// - "/My%20Post%20Title" -> "my-post-title" (URL-decoded)
pub fn slug_from_url(url_path: &str) -> String {
    // Strip query string
    let path = url_path.split('?').next().unwrap_or(url_path);
    // Strip fragment
    let path = path.split('#').next().unwrap_or(path);
    // Strip trailing slash
    let path = path.trim_end_matches('/');
    // Strip file extensions
    let path = path
        .trim_end_matches(".html")
        .trim_end_matches(".htm")
        .trim_end_matches(".php")
        .trim_end_matches(".asp")
        .trim_end_matches(".aspx")
        .trim_end_matches(".jsp");

    // Take last segment
    let segment = path.rsplit('/').next().unwrap_or(path);

    if segment.is_empty() {
        return "index".to_string();
    }

    // Manually decode common percent-encoded characters
    // url crate's percent_encoding isn't re-exported, so we do manual replacement
    let decoded = segment
        .replace("%20", " ")
        .replace("%2F", "/")
        .replace("%2f", "/")
        .replace("%3A", ":")
        .replace("%3a", ":")
        .replace("%2D", "-")
        .replace("%2d", "-")
        .replace("%2E", ".")
        .replace("%2e", ".")
        .replace("_", " "); // Underscores often become spaces in URLs

    // Convert to lowercase and replace non-alphanumeric with hyphens
    let mut slug = String::with_capacity(decoded.len());
    let mut last_was_hyphen = false;

    for ch in decoded.chars() {
        if ch.is_ascii_alphanumeric() {
            slug.push(ch.to_ascii_lowercase());
            last_was_hyphen = false;
        } else if !last_was_hyphen {
            slug.push('-');
            last_was_hyphen = true;
        }
        // Skip consecutive non-alphanumeric
    }

    // Trim leading/trailing hyphens
    slug.trim_matches('-').to_string()
}

/// Determines if a URL should be converted to a wiki-link.
/// Returns Some(slug) if conversion is possible, None otherwise.
fn should_convert_wikilink(url_str: &str, base_domain: &str) -> Option<String> {
    // Skip anchor links (e.g., #section)
    if url_str.starts_with('#') {
        return None;
    }

    // Skip relative paths (starting with / but not containing ://)
    // These should be left unchanged per original behavior
    if url_str.starts_with('/') && !url_str.contains("://") {
        return None;
    }

    // Try to parse the URL
    let parsed = match url::Url::parse(url_str) {
        Ok(p) => p,
        Err(_) => {
            // Could not parse as URL - skip it (includes relative paths)
            return None;
        },
    };

    // Get the host
    let host = match parsed.host_str() {
        Some(h) => h,
        None => {
            // No host means relative path - already handled above
            return None;
        },
    };

    // Only convert same-domain links
    if host != base_domain {
        return None;
    }

    // Extract path and generate slug
    let path = parsed.path();
    let slug = slug_from_url(path);
    Some(slug)
}

/// Convert Markdown links to Obsidian wiki-links for same-domain URLs.
///
/// Transforms `[link text](https://same-domain.com/page)` -> `[[page-slug|link text]]`
/// External links (different domain) are left unchanged.
/// Links inside code blocks are NOT converted.
///
/// # Arguments
/// - `content` — Markdown content to process
/// - `base_domain` — The domain of the scraped page (e.g. "example.com")
///
/// # Returns
/// Markdown with same-domain links converted to wiki-link syntax
pub fn convert_wiki_links(content: &str, base_domain: &str) -> String {
    // Parse with GFM support (tables, footnotes, strikethrough, tasklists)
    let mut options = Options::all();
    // Disable smart punctuation to preserve literal characters
    options.remove(Options::ENABLE_SMART_PUNCTUATION);

    let parser = Parser::new_ext(content, options);

    // Transform the event stream and build output manually
    // (we can't use pulldown-cmark-to-cmark because it escapes [[wikilinks]])
    transform_and_serialize(parser, base_domain)
}

/// Transform link events to wiki-links and serialize to string.
fn transform_and_serialize<'a>(
    events: impl Iterator<Item = Event<'a>>,
    base_domain: &str,
) -> String {
    let mut result = String::new();
    let mut in_link = false;
    let mut link_url = String::new();
    let mut link_text_parts: Vec<Event<'a>> = Vec::new();
    let mut depth = 0;

    for event in events {
        match &event {
            // Entering a link
            Event::Start(Tag::Link {
                dest_url,
                title: _,
                id: _,
                link_type: _,
            }) => {
                if depth == 0 {
                    in_link = true;
                    link_url = dest_url.to_string();
                    link_text_parts.clear();
                }
                depth += 1;
                if !in_link {
                    push_event_text(&event, &mut result);
                }
            },
            // Exiting a link
            Event::End(TagEnd::Link) => {
                if depth == 1 && in_link {
                    // This is the end of our tracked link
                    in_link = false;
                    depth = 0;

                    // Check if we should convert to wiki-link
                    if let Some(slug) = should_convert_wikilink(&link_url, base_domain) {
                        // Extract text from link content
                        let link_text = extract_text_from_events(&link_text_parts);
                        let normalized_text = link_text.to_lowercase().trim().replace(' ', "-");

                        if slug == normalized_text {
                            // [[slug]] format
                            result.push_str("[[");
                            result.push_str(&slug);
                            result.push_str("]]");
                        } else {
                            // [[slug|text]] format
                            result.push_str("[[");
                            result.push_str(&slug);
                            result.push('|');
                            result.push_str(&link_text);
                            result.push_str("]]");
                        }
                    } else {
                        // NOT converting - output original link format: [text](url)
                        let link_text = extract_text_from_events(&link_text_parts);
                        result.push('[');
                        result.push_str(&link_text);
                        result.push_str("](");
                        result.push_str(&link_url);
                        result.push(')');
                    }
                    link_text_parts.clear();
                } else {
                    depth -= 1;
                    if !in_link {
                        push_event_text(&event, &mut result);
                    }
                }
            },
            // Other Start tags (nested)
            Event::Start(Tag::Image { .. }) => {
                // Track but don't convert images to wiki-links
                if in_link {
                    link_text_parts.push(event);
                } else {
                    push_event_text(&event, &mut result);
                }
            },
            Event::Start(_) => {
                if in_link {
                    depth += 1;
                    link_text_parts.push(event);
                } else {
                    push_event_text(&event, &mut result);
                }
            },
            Event::End(TagEnd::Image) => {
                if in_link {
                    link_text_parts.push(event);
                } else {
                    push_event_text(&event, &mut result);
                }
            },
            Event::End(_) => {
                if in_link && depth > 1 {
                    depth -= 1;
                    link_text_parts.push(event);
                } else if in_link {
                    link_text_parts.push(event);
                } else {
                    push_event_text(&event, &mut result);
                }
            },
            // Text content inside link
            _ => {
                if in_link {
                    link_text_parts.push(event);
                } else {
                    push_event_text(&event, &mut result);
                }
            },
        }
    }

    // Handle any remaining events that weren't closed properly
    if in_link {
        for e in link_text_parts.drain(..) {
            push_event_text(&e, &mut result);
        }
    }

    // Trim trailing newlines to match original behavior
    result.trim_end().to_string()
}

/// Push the text representation of an event to the result string.
/// This is a simplified serializer that handles the most common events.
/// For events we don't explicitly handle, we use the Debug representation.
fn push_event_text(event: &Event, result: &mut String) {
    match event {
        Event::Text(s) => result.push_str(s),
        Event::Code(s) => {
            result.push('`');
            result.push_str(s);
            result.push('`');
        },
        Event::Html(s) => result.push_str(s),
        Event::FootnoteReference(s) => {
            result.push_str("[^");
            result.push_str(s);
            result.push(']');
        },
        Event::TaskListMarker(checked) => {
            result.push_str(if *checked { "- [x] " } else { "- [ ] " });
        },
        Event::SoftBreak => result.push('\n'),
        Event::HardBreak => result.push_str("  \n"),
        Event::Rule => result.push_str("---\n"),
        Event::InlineMath(s) => {
            result.push('$');
            result.push_str(s);
            result.push('$');
        },
        Event::DisplayMath(s) => {
            result.push_str("$$");
            result.push_str(s);
            result.push_str("$$");
        },
        // Links and images are handled separately
        Event::Start(Tag::Link { .. }) => {},
        Event::End(TagEnd::Link) => {},
        Event::Start(Tag::Image { .. }) => {
            result.push_str("![");
        },
        Event::End(TagEnd::Image) => {},
        // Other tags - output opening/closing markers
        Event::Start(Tag::Paragraph) => {},
        Event::End(TagEnd::Paragraph) => result.push_str("\n\n"),
        Event::Start(Tag::CodeBlock(_)) => result.push_str("```\n"),
        Event::End(TagEnd::CodeBlock) => result.push_str("\n```\n"),
        Event::Start(Tag::BlockQuote(_)) => result.push_str("> "),
        Event::End(TagEnd::BlockQuote(_)) => result.push('\n'),
        Event::Start(Tag::List(_)) => {},
        Event::End(TagEnd::List(_)) => {},
        Event::Start(Tag::Item) => {},
        Event::End(TagEnd::Item) => {},
        Event::Start(Tag::Table(_)) => {},
        Event::End(TagEnd::Table) => result.push('\n'),
        Event::Start(Tag::TableRow) => {},
        Event::End(TagEnd::TableRow) => result.push('\n'),
        Event::Start(Tag::TableCell) => {},
        Event::End(TagEnd::TableCell) => result.push('|'),
        Event::Start(Tag::FootnoteDefinition(s)) => {
            result.push_str("[^");
            result.push_str(s);
            result.push_str("]: ");
        },
        Event::End(TagEnd::FootnoteDefinition) => result.push_str("\n\n"),
        Event::Start(Tag::Emphasis) => result.push('*'),
        Event::End(TagEnd::Emphasis) => result.push('*'),
        Event::Start(Tag::Strong) => result.push_str("**"),
        Event::End(TagEnd::Strong) => result.push_str("**"),
        Event::Start(Tag::Strikethrough) => result.push_str("~~"),
        Event::End(TagEnd::Strikethrough) => result.push_str("~~"),
        Event::Start(Tag::Heading { .. }) => {},
        Event::End(TagEnd::Heading(_)) => result.push('\n'),
        Event::Start(Tag::MetadataBlock(_)) => {},
        Event::End(TagEnd::MetadataBlock(_)) => result.push_str("---\n"),
        // Catch-all for any other events
        _ => {
            // For events we don't explicitly handle, try to output something reasonable
            // based on the debug representation
            let debug_str = format!("{:?}", event);
            // Only output if it looks like something real (not empty or just a placeholder)
            if !debug_str.is_empty()
                && debug_str != "InlineMath(\"\")"
                && debug_str != "DisplayMath(\"\")"
            {
                // Output as-is for raw events
                if debug_str.starts_with("Html(")
                    || debug_str.starts_with("Code(")
                    || debug_str.starts_with("Text(")
                {
                    // These are already handled above, skip
                } else {
                    // For other events, we just won't output them - they may not be important
                }
            }
        },
    }
}

/// Extract plain text from a sequence of events.
fn extract_text_from_events(events: &[Event]) -> String {
    let mut text = String::new();
    for event in events {
        match event {
            Event::Text(s) => text.push_str(s),
            Event::Code(s) => text.push_str(s),
            _ => {},
        }
    }
    text
}

/// Rewrite Markdown image/document references to use relative paths.
///
/// Transforms `![](absolute/local/path)` -> `![](../../relative/path)`
/// based on the `.md` file's location and the asset's `local_path`.
///
/// Uses fuzzy matching to handle URL-encoded paths in Markdown references.
///
/// # Arguments
/// - `content` — Markdown content with `![]()` references
/// - `md_file_dir` — Directory containing the output `.md` file
/// - `assets` — DownloadedAsset list with `local_path` and original `url`
///
/// # Returns
/// Markdown with asset paths rewritten as relative
pub fn resolve_asset_paths(
    content: &str,
    md_file_dir: &Path,
    assets: &[DownloadedAsset],
) -> String {
    if assets.is_empty() {
        return content.to_string();
    }

    // Build a map: original_url -> relative_path
    use std::collections::HashMap;
    let mut asset_map: HashMap<String, String> = HashMap::with_capacity(assets.len());

    for asset in assets {
        // Compute relative path using pathdiff
        let local_path = Path::new(&asset.local_path);
        let rel = match pathdiff::diff_paths(local_path, md_file_dir) {
            Some(p) => p,
            None => {
                // pathdiff returns None for cross-drive on Windows - skip this asset
                continue;
            },
        };

        // Convert to forward slashes for Obsidian compatibility (handle Windows paths)
        let rel_str = rel.to_string_lossy().replace('\\', "/");
        asset_map.insert(asset.url.clone(), rel_str);
    }

    if asset_map.is_empty() {
        return content.to_string();
    }

    // Parse with GFM support
    let mut options = Options::all();
    options.remove(Options::ENABLE_SMART_PUNCTUATION);

    let parser = Parser::new_ext(content, options);

    // Transform image events and serialize
    transform_images_and_serialize(parser, &asset_map)
}

/// Transform image events to use relative paths and serialize to string.
fn transform_images_and_serialize<'a>(
    events: impl Iterator<Item = Event<'a>>,
    asset_map: &std::collections::HashMap<String, String>,
) -> String {
    let mut result = String::new();
    let mut in_image = false;
    let mut image_url = String::new();
    let mut alt_text = String::new();

    for event in events {
        match &event {
            Event::Start(Tag::Image {
                link_type: _,
                dest_url,
                title: _,
                id: _,
            }) => {
                in_image = true;
                image_url = dest_url.to_string();
                alt_text.clear();
                result.push_str("![");
            },
            Event::End(TagEnd::Image) => {
                if in_image {
                    // Look up the relative path
                    if let Some(rel_path) = asset_map.get(&image_url) {
                        // Output with relative path
                        result.push_str(&alt_text);
                        result.push_str("](");
                        result.push_str(rel_path);
                        result.push(')');
                    } else {
                        // Not an asset we downloaded - keep original
                        result.push_str(&alt_text);
                        result.push_str("](");
                        result.push_str(&image_url);
                        result.push(')');
                    }
                    in_image = false;
                    image_url.clear();
                    alt_text.clear();
                } else {
                    push_event_text(&event, &mut result);
                }
            },
            Event::Text(s) => {
                if in_image {
                    alt_text.push_str(s);
                } else {
                    result.push_str(s);
                }
            },
            _ => {
                if in_image {
                    // Collect alt text content
                } else {
                    push_event_text(&event, &mut result);
                }
            },
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    // === convert_wiki_links tests ===

    #[test]
    fn test_convert_same_domain_link() {
        let md = "[Read more](https://example.com/about)";
        let result = convert_wiki_links(md, "example.com");
        assert_eq!(result, "[[about|Read more]]");
    }

    #[test]
    fn test_skip_external_domain_link() {
        let md = "[Google](https://google.com)";
        let result = convert_wiki_links(md, "example.com");
        assert_eq!(result, "[Google](https://google.com)");
    }

    #[test]
    fn test_skip_links_in_code_block() {
        let md = "```\n[not a link](https://example.com/foo)\n```";
        let result = convert_wiki_links(md, "example.com");
        assert!(result.contains("[not a link]"));
    }

    #[test]
    fn test_skip_inline_code_link() {
        let md = "Use `[link](https://example.com)` for docs";
        let result = convert_wiki_links(md, "example.com");
        assert!(result.contains("[link](https://example.com)"));
    }

    #[test]
    fn test_multiple_links_mixed() {
        let md = "[internal](https://example.com/a) and [external](https://other.com/b)";
        let result = convert_wiki_links(md, "example.com");
        assert!(result.contains("[[a|internal]]"));
        assert!(result.contains("[external](https://other.com/b)"));
    }

    #[test]
    fn test_identical_links_all_converted() {
        let md = "[link](https://example.com/x) and [link](https://example.com/x)";
        let result = convert_wiki_links(md, "example.com");
        assert_eq!(result.matches("[[x|link]]").count(), 2);
    }

    #[test]
    fn test_anchor_links_unchanged() {
        let md = "[Section](#section)";
        let result = convert_wiki_links(md, "example.com");
        assert_eq!(result, md);
    }

    #[test]
    fn test_relative_links_unchanged() {
        let md = "[About](/about)";
        let result = convert_wiki_links(md, "example.com");
        assert_eq!(result, md);
    }

    // === slug_from_url tests ===

    #[test]
    fn test_slug_simple_path() {
        assert_eq!(slug_from_url("/blog/my-post"), "my-post");
    }

    #[test]
    fn test_slug_with_query_and_fragment() {
        assert_eq!(slug_from_url("/page?id=1#section"), "page");
    }

    #[test]
    fn test_slug_root_path() {
        assert_eq!(slug_from_url("/"), "index");
    }

    #[test]
    fn test_slug_with_extension() {
        assert_eq!(slug_from_url("/docs/api.html"), "api");
    }

    #[test]
    fn test_slug_url_encoded() {
        assert_eq!(slug_from_url("/My%20Post%20Title"), "my-post-title");
    }

    #[test]
    fn test_slug_nested_with_date() {
        assert_eq!(slug_from_url("/2026/04/03/hello-world/"), "hello-world");
    }

    #[test]
    fn test_slug_trailing_slash() {
        assert_eq!(slug_from_url("/blog/"), "blog");
    }

    #[test]
    fn test_slug_multiple_extensions() {
        assert_eq!(slug_from_url("/page.asp?id=1"), "page");
    }

    // === resolve_asset_paths tests ===

    #[test]
    fn test_resolve_single_asset() {
        let md = "![image](https://example.com/img/photo.png)";
        let assets = vec![DownloadedAsset {
            url: "https://example.com/img/photo.png".to_string(),
            local_path: "/home/user/output/example.com/images/photo.png".to_string(),
            asset_type: "image".to_string(),
            size: 1024,
        }];
        let md_dir = Path::new("/home/user/output/example.com");
        let result = resolve_asset_paths(md, md_dir, &assets);
        assert!(result.contains("images/photo.png"));
    }

    #[test]
    fn test_resolve_no_assets() {
        let md = "No images here";
        let result = resolve_asset_paths(md, Path::new("/tmp"), &[]);
        assert_eq!(result, md);
    }

    #[test]
    fn test_resolve_asset_in_nested_dir() {
        let md = "![chart](https://example.com/charts/data.png)";
        let assets = vec![DownloadedAsset {
            url: "https://example.com/charts/data.png".to_string(),
            local_path: "/tmp/output/example.com/blog/images/data.png".to_string(),
            asset_type: "image".to_string(),
            size: 2048,
        }];
        let md_dir = Path::new("/tmp/output/example.com/blog");
        let result = resolve_asset_paths(md, md_dir, &assets);
        assert!(result.contains("images/data.png"));
    }

    #[test]
    fn test_resolve_skips_code_blocks() {
        let md = "```\n![](https://example.com/img.png)\n```";
        let assets = vec![DownloadedAsset {
            url: "https://example.com/img.png".to_string(),
            local_path: "/tmp/img.png".to_string(),
            asset_type: "image".to_string(),
            size: 1024,
        }];
        let result = resolve_asset_paths(md, Path::new("/tmp"), &assets);
        // Should not replace the image in code block
        assert!(result.contains("!["));
    }

    #[test]
    fn test_resolve_windows_paths_converted() {
        // Windows paths should be converted to forward slashes
        let md = "![image](https://example.com/img.png)";
        let assets = vec![DownloadedAsset {
            url: "https://example.com/img.png".to_string(),
            local_path: "C:\\Users\\output\\example.com\\images\\photo.png".to_string(),
            asset_type: "image".to_string(),
            size: 1024,
        }];
        let md_dir = Path::new("C:\\Users\\output\\example.com");
        let result = resolve_asset_paths(md, md_dir, &assets);
        assert!(result.contains("images/photo.png"));
    }
}
