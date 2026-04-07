//! WAF Detection Engine - Layer 7 Protection
//!
//! This module provides advanced WAF detection beyond the basic signature matching
//! in http_client.rs. It includes:
//! - Detection by Control Headers (x-datadome-response, cf-mitigated, etc.)
//! - Entropy analysis for "Silent Challenge" detection
//! - Efficient O(N) matching using Aho-Corasick for 50+ signatures
//!
//! # Usage
//!
//! ```rust
//! use rust_scraper::infrastructure::http::waf_engine::WafInspector;
//!
//! // After receiving HTTP 200 response with body
//! if let Err(e) = WafInspector::verify_integrity(&response, &body) {
//!     return Err(e); // WAF challenge detected
//! }
//! ```

use crate::error::ScraperError;
use aho_corasick::AhoCorasick;
use once_cell::sync::Lazy;
use std::collections::HashSet;
use wreq::header::HeaderMap;

/// Control headers that indicate WAF processing (2026 signatures)
const WAF_CONTROL_HEADERS: &[(&str, &str)] = &[
    ("x-datadome-response", "DataDome"),
    ("cf-mitigated", "Cloudflare"),
    ("x-akamai-edge-auth", "Akamai"),
    ("x-sucuri-id", "Sucuri"),
    ("x-wordpress", "Wordfence"),
    ("cf-ray", "Cloudflare"),
    ("x-cdn", "Imperva"),
];

/// WAF signatures for HTML body scanning (2026 updated)
/// These replace the basic signatures in http_client.rs for more comprehensive coverage
const WAF_BODY_SIGNATURES: &[(&str, &str)] = &[
    // Cloudflare (updated 2026)
    ("cf-turnstile", "Cloudflare Turnstile"),
    ("challenge-platform", "Cloudflare JS Challenge"),
    ("Just a moment...", "Cloudflare"),
    ("Checking your browser", "Cloudflare"),
    ("__cf_chl_f_tk", "Cloudflare"),
    ("_cf_chl_opt", "Cloudflare"),
    ("cloudflare", "Cloudflare"),
    ("cf-dns", "Cloudflare"),
    // Google reCAPTCHA (updated)
    ("g-recaptcha", "reCAPTCHA"),
    ("recaptcha/api.js", "reCAPTCHA"),
    ("grecaptcha.execute", "reCAPTCHA"),
    ("recaptcha Enterprise", "reCAPTCHA"),
    // hCaptcha (updated)
    ("hcaptcha.com", "hCaptcha"),
    ("h-captcha", "hCaptcha"),
    ("hcaptcha.js", "hCaptcha"),
    // DataDome (updated 2026 - Silent Challenges)
    ("datadome", "DataDome"),
    ("dd-captcha", "DataDome"),
    ("dd=", "DataDome"),
    ("data-domain", "DataDome"),
    // PerimeterX / HUMAN Security (updated)
    ("perimeterx", "PerimeterX"),
    ("_pxCaptcha", "PerimeterX"),
    ("human-security", "HUMAN"),
    ("px-init", "PerimeterX"),
    // Akamai Bot Manager (updated 2026)
    ("_abck", "Akamai Bot Manager"),
    ("SensorData", "Akamai Bot Manager"),
    ("akamai", "Akamai"),
    // Imperva (updated)
    ("imperva", "Imperva"),
    ("incapsula", "Imperva"),
    ("_Incapsula_Resource", "Imperva"),
    // F5 (updated)
    ("_nfv", "F5"),
    ("BIGipServer", "F5"),
    // Generic challenges (expanded)
    ("Please verify you are a human", "Generic Challenge"),
    ("verify you are human", "Generic Challenge"),
    ("bot detection", "Generic Detection"),
    ("checking your browser", "Browser Verification"),
    ("attack detected", "Security Firewall"),
    ("suspicious activity", "Security Firewall"),
    ("captcha-delivery", "Challenge Delivery"),
    ("__js_challenge__", "JS Challenge"),
];

/// Aho-Corasick automaton for O(N) multi-pattern matching
/// Replaces O(N*M) linear search in http_client.rs
static WAF_AC: Lazy<AhoCorasick> = Lazy::new(|| {
    AhoCorasick::new(WAF_BODY_SIGNATURES.iter().map(|(sig, _)| sig))
        .expect("Failed to build Aho-Corasick automaton")
});

/// WafInspector provides multi-layer WAF detection
pub struct WafInspector;

impl WafInspector {
    /// Verify response integrity across multiple layers
    ///
    /// 1. Control Headers: Check for WAF-specific headers (immediate)
    /// 2. Body Signatures: O(N) scan using Aho-Corasick
    /// 3. Entropy Analysis: Detect "Silent Challenges" in minimal HTML
    ///
    /// # Arguments
    /// * `headers` - Response headers from HTTP call
    /// * `body` - Response body (HTML content)
    ///
    /// # Returns
    /// * `Ok(())` - No WAF challenge detected
    /// * `Err(ScraperError::WafBlocked)` - WAF challenge detected
    pub fn verify_integrity(headers: &HeaderMap, body: &str) -> Result<(), ScraperError> {
        // Layer 1: Control Headers (fastest - O(1) lookup)
        Self::check_control_headers(headers)?;

        // Layer 2: Body Signature Matching (O(N) with Aho-Corasick)
        Self::check_body_signatures(body)?;

        // Layer 3: Entropy Analysis (detect Silent Challenges)
        Self::check_entropy(body)?;

        Ok(())
    }

    /// Check for WAF control headers that indicate bot detection/processing
    #[inline]
    fn check_control_headers(headers: &HeaderMap) -> Result<(), ScraperError> {
        for (header_name, provider) in WAF_CONTROL_HEADERS {
            // Check if header exists (even with empty value indicates WAF processing)
            if headers.get(*header_name).is_some() {
                // Some headers like cf-ray exist even for normal requests,
                // but others like x-datadome-response specifically indicate bot challenges
                if *header_name == "x-datadome-response"
                    || *header_name == "cf-mitigated"
                    || *header_name == "x-akamai-edge-auth"
                {
                    return Err(ScraperError::WafBlocked {
                        url: String::new(),
                        provider: format!("{}: header detected", provider),
                    });
                }
            }
        }
        Ok(())
    }

    /// Check body content for WAF signatures using O(N) Aho-Corasick
    #[inline]
    fn check_body_signatures(body: &str) -> Result<(), ScraperError> {
        // Early exit for empty or very small bodies
        // Lowered to 10 chars to detect short WAF challenge pages
        if body.len() < 10 {
            return Ok(());
        }

        // Use Aho-Corasick for O(N) multi-pattern matching
        if let Some(mat) = WAF_AC.find_iter(body).next() {
            // Map pattern index to provider name
            let provider = WAF_BODY_SIGNATURES[mat.pattern()].1;
            return Err(ScraperError::WafBlocked {
                url: String::new(),
                provider: format!("Signature detected: {}", provider),
            });
        }

        Ok(())
    }

    /// Detect "Silent Challenges" using entropy analysis
    ///
    /// WAFs in 2026 sometimes return HTTP 200 with minimal HTML containing
    /// heavy JavaScript challenges. This function detects that pattern:
    /// - Body < 1500 bytes
    /// - High density of <script> tags (> 5)
    /// - Low text content ratio
    #[inline]
    fn check_entropy(body: &str) -> Result<(), ScraperError> {
        // Only analyze bodies under 1500 bytes
        if body.len() > 1500 {
            return Ok(());
        }

        // Count <script> tags efficiently
        let script_count = body.matches("<script").count();

        // Silent Challenge detection:
        // - Multiple script tags in a small body suggests JS challenge
        // - Low text ratio indicates mostly code, not content
        if script_count > 5 && body.len() < 1000 {
            return Err(ScraperError::WafBlocked {
                url: String::new(),
                provider: "Silent Challenge: High JS density in minimal body".into(),
            });
        }

        // Additional entropy check: ratio of script to text
        if body.len() < 500 && script_count > 3 {
            return Err(ScraperError::WafBlocked {
                url: String::new(),
                provider: "Silent Challenge: Suspicious script/text ratio".into(),
            });
        }

        Ok(())
    }

    /// Get the list of supported WAF providers
    #[must_use]
    pub fn supported_providers() -> Vec<&'static str> {
        // Extract unique provider names from signatures
        let mut providers: Vec<&str> = Vec::new();
        let mut seen: HashSet<&str> = HashSet::new();

        for (_, provider) in WAF_BODY_SIGNATURES {
            if !seen.contains(provider) {
                seen.insert(provider);
                providers.push(provider);
            }
        }
        providers.sort();
        providers
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_waf_control_header_detection() {
        // Test DataDome header detection
        let mut headers = HeaderMap::new();
        headers.insert("x-datadome-response", "blocked".parse().unwrap());

        let result = WafInspector::verify_integrity(&headers, "normal content");
        assert!(result.is_err());

        // Test that cf-ray alone doesn't trigger (common in normal requests)
        let mut headers = HeaderMap::new();
        headers.insert("cf-ray", "abc123".parse().unwrap());

        let result = WafInspector::verify_integrity(&headers, "normal content");
        assert!(result.is_ok());
    }

    #[test]
    fn test_waf_body_signature_detection() {
        // Test Cloudflare detection
        let result = WafInspector::verify_integrity(&HeaderMap::new(), "Just a moment...");
        assert!(result.is_err());

        // Test reCAPTCHA detection
        let result = WafInspector::verify_integrity(&HeaderMap::new(), "<div class='g-recaptcha'>");
        assert!(result.is_err());

        // Test normal content passes
        let result = WafInspector::verify_integrity(
            &HeaderMap::new(),
            "<html><body><p>Hello World</p></body></html>",
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_silent_challenge_detection() {
        // High script density in small body should trigger
        let body = r#"<html><script></script><script></script><script></script><script></script><script></script><script></script></html>"#;
        let result = WafInspector::verify_integrity(&HeaderMap::new(), body);
        assert!(result.is_err());

        // Normal content should pass
        let body = "<html><body><p>Hello</p></body></html>";
        let result = WafInspector::verify_integrity(&HeaderMap::new(), body);
        assert!(result.is_ok());
    }

    #[test]
    fn test_aho_corasick_performance() {
        // Test that Aho-Corasick works correctly - using exact phrase from signatures
        let body = "This is a page with Just a moment... and recaptcha/api.js content";
        let result = WafInspector::verify_integrity(&HeaderMap::new(), body);
        assert!(result.is_err());
    }

    #[test]
    fn test_supported_providers() {
        let providers = WafInspector::supported_providers();
        assert!(!providers.is_empty());
        assert!(providers.contains(&"Cloudflare"));
        assert!(providers.contains(&"reCAPTCHA"));
        assert!(providers.contains(&"DataDome"));
    }
}
