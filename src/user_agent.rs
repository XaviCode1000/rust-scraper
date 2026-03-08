//! User-Agent rotation module
//!
//! Provides pool of modern user agents to avoid detection by anti-bot systems.
//! Based on best practices: session-based consistency, popular browsers only.

use rand::Rng;

/// Pool of modern user agents - updated 2026
/// Sources: Chrome (~64% market), Firefox, Safari, Edge
const USER_AGENTS: &[&str] = &[
    // Chrome on Windows
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36",
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/121.0.0.0 Safari/537.36",
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/122.0.0.0 Safari/537.36",
    // Chrome on macOS
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36",
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/121.0.0.0 Safari/537.36",
    // Firefox on Windows
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:122.0) Gecko/20100101 Firefox/122.0",
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:123.0) Gecko/20100101 Firefox/123.0",
    // Firefox on macOS
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10.15; rv:122.0) Gecko/20100101 Firefox/122.0",
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10.15; rv:123.0) Gecko/20100101 Firefox/123.0",
    // Firefox on Linux
    "Mozilla/5.0 (X11; Linux x86_64; rv:122.0) Gecko/20100101 Firefox/122.0",
    "Mozilla/5.0 (X11; Linux x86_64; rv:123.0) Gecko/20100101 Firefox/123.0",
    // Safari on macOS
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.2 Safari/605.1.15",
    // Edge on Windows
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36 Edg/120.0.0.0",
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/121.0.0.0 Safari/537.36 Edg/121.0.0.0",
];

/// Get a random user agent from the pool
///
/// Uses random selection to avoid patterns.
#[must_use]
pub fn random_user_agent() -> &'static str {
    let mut rng = rand::thread_rng();
    let index = rng.gen_range(0..USER_AGENTS.len());
    USER_AGENTS[index]
}

/// Get a random user agent weighted towards Chrome (most common browser)
///
/// Weight: 40% Chrome, 20% Firefox, 20% Safari, 20% Edge
#[must_use]
pub fn random_user_agent_weighted() -> &'static str {
    let mut rng = rand::thread_rng();
    let r = rng.gen_range(0..100);
    if r < 40 {
        // Chrome (Windows + macOS)
        let chrome_agents: Vec<&str> = USER_AGENTS
            .iter()
            .filter(|ua| ua.contains("Chrome") && !ua.contains("Edg"))
            .copied()
            .collect();
        chrome_agents[rng.gen_range(0..chrome_agents.len())]
    } else if r < 60 {
        // Firefox
        let firefox_agents: Vec<&str> = USER_AGENTS
            .iter()
            .filter(|ua| ua.contains("Firefox"))
            .copied()
            .collect();
        firefox_agents[rng.gen_range(0..firefox_agents.len())]
    } else if r < 80 {
        // Safari
        USER_AGENTS
            .iter()
            .find(|ua| ua.contains("Safari"))
            .copied()
            .unwrap_or_else(random_user_agent)
    } else {
        // Edge
        USER_AGENTS
            .iter()
            .find(|ua| ua.contains("Edg"))
            .copied()
            .unwrap_or_else(random_user_agent)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_random_user_agent_returns_valid_ua() {
        let ua = random_user_agent();
        assert!(!ua.is_empty());
        assert!(ua.contains("Mozilla/5.0"));
    }

    #[test]
    fn test_random_user_agent_weighted_returns_valid_ua() {
        for _ in 0..100 {
            let ua = random_user_agent_weighted();
            assert!(!ua.is_empty());
            assert!(ua.contains("Mozilla/5.0"));
        }
    }

    #[test]
    fn test_user_agents_are_unique() {
        let mut agents: Vec<&str> = USER_AGENTS.to_vec();
        agents.sort();
        agents.dedup();
        assert_eq!(
            agents.len(),
            USER_AGENTS.len(),
            "User agents should be unique"
        );
    }
}
