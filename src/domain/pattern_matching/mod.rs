//! URL pattern matching
//!
//! SSRF-safe pattern matching that compares HOSTS only, not raw URL strings.
//!
//! Following **own-borrow-over-clone**: Accepts `&str` not `&String`.
//! Following **opt-inline**: Inlined for hot path performance.
//! Following **security-ssrf-prevention**: Parses URL before comparison (no `.contains()` on raw string)
//!
//! # Security
//!
//! This function parses the URL using `url::Url` and compares HOSTS only,
//! NOT raw string substrings. This prevents SSRF attacks where malicious
//! URLs like `https://evil.com/?q=example.com/path` could bypass filters.

use url::Url;

/// Check if a URL matches a glob-style pattern
///
/// Following **own-borrow-over-clone**: Accepts `&str` not `&String`.
/// Following **opt-inline**: Inlined for hot path performance.
/// Following **security-ssrf-prevention**: Parses URL before comparison (no `.contains()` on raw string)
///
/// # Security
///
/// This function parses the URL using `url::Url` and compares HOSTS only,
/// NOT raw string substrings. This prevents SSRF attacks where malicious
/// URLs like `https://evil.com/?q=example.com/path` could bypass filters.
///
/// # Examples
///
/// ```
/// use rust_scraper::domain::pattern_matching::matches_pattern;
///
/// // Valid subdomain match
/// assert!(matches_pattern("https://blog.example.com/post", "*.example.com/*"));
///
/// // SSRF bypass attempt (should NOT match)
/// assert!(!matches_pattern("https://evil.com/?q=example.com/path", "*.example.com/*"));
/// ```
#[inline]
#[must_use]
pub fn matches_pattern(url_str: &str, pattern: &str) -> bool {
    // Parse URL FIRST (extract real host)
    let url = match Url::parse(url_str) {
        Ok(u) => u,
        Err(_) => return false, // Invalid URL → no match
    };

    let host = match url.host_str() {
        Some(h) => h,
        None => return false, // No host → no match
    };

    // Handle empty pattern
    if pattern.is_empty() {
        return true;
    }

    // Handle wildcard
    if pattern == "*" {
        return true;
    }

    // Compare HOSTS only (NOT raw URL strings)
    match pattern {
        // *.example.com/* → match subdomain ONLY (not root domain)
        p if p.starts_with("*.") && p.ends_with("*") => {
            let domain = &p[2..p.len() - 1]; // "example.com/"
            let domain = domain.trim_end_matches('/');
            // Must be a subdomain, NOT the root domain itself
            host.ends_with(&format!(".{}", domain))
        },
        // *.example.com → match subdomain ONLY (not root domain)
        p if p.starts_with("*.") => {
            let domain = &p[2..];
            // Must be a subdomain, NOT the root domain itself
            host.ends_with(&format!(".{}", domain))
        },
        // Exact host match (no wildcard)
        p => host == p,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ========== SSRF PREVENTION TESTS ==========

    #[test]
    fn test_matches_pattern_ssrf_bypass_attempt() {
        assert!(!matches_pattern(
            "https://evil.com/?q=example.com/path",
            "*.example.com/*"
        ));

        assert!(!matches_pattern(
            "https://attacker.com/?redirect=example.com/admin",
            "*.example.com/*"
        ));

        assert!(!matches_pattern(
            "https://malicious.com/redirect?url=example.com/secret",
            "*.example.com/*"
        ));
    }

    #[test]
    fn test_matches_pattern_real_subdomain() {
        assert!(matches_pattern(
            "https://blog.example.com/post",
            "*.example.com/*"
        ));

        assert!(matches_pattern(
            "https://sub.example.com/page",
            "*.example.com"
        ));

        assert!(matches_pattern(
            "https://deep.sub.example.com/page",
            "*.example.com/*"
        ));
    }

    #[test]
    fn test_matches_pattern_with_port() {
        assert!(matches_pattern(
            "https://blog.example.com:8080/path",
            "*.example.com/*"
        ));

        assert!(matches_pattern(
            "https://blog.example.com:443/post",
            "*.example.com/*"
        ));
    }

    #[test]
    fn test_matches_pattern_ipv4() {
        assert!(matches_pattern(
            "http://192.168.1.1:8080/path",
            "192.168.1.1"
        ));
    }

    #[test]
    fn test_matches_pattern_ipv6() {
        assert!(matches_pattern("http://[::1]:8080/path", "[::1]"));
    }

    #[test]
    fn test_matches_pattern_invalid_url() {
        assert!(!matches_pattern("not-a-url", "*.example.com/*"));
        assert!(!matches_pattern("://missing-scheme.com", "*"));
        assert!(!matches_pattern("", "*"));
    }

    #[test]
    fn test_matches_pattern_wildcard() {
        assert!(matches_pattern("https://example.com/page", "*"));
        assert!(matches_pattern("https://any.domain.com/page", "*"));
    }

    #[test]
    fn test_matches_pattern_empty() {
        assert!(matches_pattern("https://example.com", ""));
    }

    #[test]
    fn test_matches_pattern_no_match() {
        assert!(!matches_pattern("https://other.com/page", "example.com"));
        assert!(!matches_pattern("https://evil.com/page", "*.example.com/*"));
    }

    #[test]
    fn test_matches_pattern_exact_host() {
        assert!(matches_pattern("https://example.com/page", "example.com"));
        assert!(!matches_pattern(
            "https://sub.example.com/page",
            "example.com"
        ));
    }

    #[test]
    fn test_matches_pattern_prefix_wildcard() {
        assert!(matches_pattern(
            "https://blog.example.com/admin/users",
            "*.example.com/*"
        ));
        assert!(matches_pattern(
            "https://admin.example.com/users",
            "*.example.com/*"
        ));
        // Root domain does NOT match *.example.com/*
        assert!(!matches_pattern(
            "https://example.com/admin/users",
            "*.example.com/*"
        ));
    }

    #[test]
    fn test_matches_pattern_slash_wildcard() {
        assert!(matches_pattern(
            "https://blog.example.com/admin/users",
            "*.example.com/*"
        ));
        assert!(matches_pattern(
            "https://admin.example.com/users",
            "*.example.com/*"
        ));
        assert!(!matches_pattern(
            "https://example.com/admin/users",
            "*.example.com/*"
        ));
    }
}
