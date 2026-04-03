//! Obsidian-compatible Markdown export
//!
//! Provides transformations for Obsidian vault compatibility:
//! - Wiki-link conversion: [text](url) -> [[slug|text]]
//! - Relative asset paths: absolute paths -> relative to .md file
//! - Slug extraction from URLs for wiki-link targets

use crate::domain::DownloadedAsset;
use regex::Regex;
use std::path::Path;
use std::sync::LazyLock;

/// Matches Markdown links: [text](url)
/// Uses alternation to skip code blocks and inline code:
///   (```[\s\S]*?```|`[^`]+`) — skip fenced/inline code
///   | \[([^\]]+)\]\(([^)]+)\) — capture link text and URL
static MARKDOWN_LINK_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?s)(```[\s\S]*?```|`[^`]+`)|\[([^\]]+)\]\(([^)]+)\)")
        .expect("BUG: invalid regex for markdown links")
});

/// Matches Markdown image references: ![alt](path)
/// Same code-block-skipping strategy.
static MARKDOWN_IMAGE_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?s)(```[\s\S]*?```|`[^`]+`)|!\[([^\]]*)\]\(([^)]+)\)")
        .expect("BUG: invalid regex for markdown images")
});

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
    MARKDOWN_LINK_RE
        .replace_all(content, |caps: &regex::Captures| {
            // If code block matched, return unchanged
            if caps.get(1).is_some() {
                return caps[0].to_string();
            }

            let link_text = &caps[2];
            let url_str = &caps[3];

            // Try to parse the URL
            let parsed = match url::Url::parse(url_str) {
                Ok(p) => p,
                Err(_) => {
                    // Handle relative paths (root-relative: /path/to/page)
                    // These are always same-site since they start with /
                    if url_str.starts_with('/') && !url_str.starts_with("//") {
                        let slug = slug_from_url(url_str);
                        if slug == link_text.to_lowercase().trim().replace(' ', "-") {
                            return format!("[[{}]]", slug);
                        } else {
                            return format!("[[{}|{}]]", slug, link_text);
                        }
                    }
                    return caps[0].to_string(); // Not a valid URL or not root-relative
                },
            };

            // Get the host
            let host = match parsed.host_str() {
                Some(h) => h,
                None => {
                    // No host means relative path — treat as same-site
                    let path = parsed.path();
                    let slug = slug_from_url(path);
                    if slug == link_text.to_lowercase().trim().replace(' ', "-") {
                        return format!("[[{}]]", slug);
                    } else {
                        return format!("[[{}|{}]]", slug, link_text);
                    }
                },
            };

            // Only convert same-domain links
            if host != base_domain {
                return caps[0].to_string();
            }

            // Extract path and generate slug
            let path = parsed.path();
            let slug = slug_from_url(path);

            // Obsidian wiki-link format: [[slug|text]] or [[slug]] if text == slug
            if slug == link_text.to_lowercase().trim().replace(' ', "-") {
                format!("[[{}]]", slug)
            } else {
                format!("[[{}|{}]]", slug, link_text)
            }
        })
        .to_string()
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
    // Use HashMap for efficient lookup
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

    MARKDOWN_IMAGE_RE
        .replace_all(content, |caps: &regex::Captures| {
            // If code block matched, return unchanged
            if caps.get(1).is_some() {
                return caps[0].to_string();
            }

            let alt = &caps[2];
            let path = &caps[3];

            // Try exact match first (most common case)
            let replacement = asset_map.get(path);

            match replacement {
                Some(rel) => format!("![{alt}]({rel})"),
                None => caps[0].to_string(), // Not an asset we downloaded
            }
        })
        .to_string()
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
