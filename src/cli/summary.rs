//! Scrape Summary
//!
//! T-060: ScrapeSummary struct with emoji/ASCII display.

use std::time::Duration;

/// Summary of a scraping run.
pub struct ScrapeSummary {
    /// Total URLs discovered during crawling
    pub urls_discovered: usize,
    /// URLs successfully scraped
    pub urls_scraped: usize,
    /// URLs that failed to scrape
    pub urls_failed: usize,
    /// URLs skipped (e.g., resume mode duplicates)
    pub urls_skipped: usize,
    /// Total content elements extracted
    pub elements_extracted: usize,
    /// Total assets (images, documents) downloaded
    pub assets_downloaded: usize,
    /// Total duration of the scraping run
    pub duration: Duration,
}

impl ScrapeSummary {
    /// Create a new ScrapeSummary
    pub fn new(
        urls_discovered: usize,
        urls_scraped: usize,
        urls_failed: usize,
        urls_skipped: usize,
        elements_extracted: usize,
        assets_downloaded: usize,
        duration: Duration,
    ) -> Self {
        Self {
            urls_discovered,
            urls_scraped,
            urls_failed,
            urls_skipped,
            elements_extracted,
            assets_downloaded,
            duration,
        }
    }

    /// Format the summary for display.
    ///
    /// When `no_color` is true, uses ASCII markers (`[OK]`, `[FAIL]`, etc.)
    /// instead of emojis.
    pub fn display(&self, no_color: bool) -> String {
        if no_color {
            self.display_ascii()
        } else {
            self.display_emoji()
        }
    }

    fn display_emoji(&self) -> String {
        let secs = self.duration.as_secs();
        let mins = secs / 60;
        let remain_secs = secs % 60;
        let dur = if mins > 0 {
            format!("{mins}m {remain_secs}s")
        } else {
            format!("{remain_secs}s")
        };

        format!(
            "\nScrape Complete! ✅\n\
             ✅ scrapados: {scraped}\n\
             ❌ falhos: {failed}\n\
             ⏭️  pulados: {skipped}\n\
             📊 resumo: {discovered} descobertos, {elements} elementos extraídos, \
             {assets} assets baixados, {dur} de duração",
            scraped = self.urls_scraped,
            failed = self.urls_failed,
            skipped = self.urls_skipped,
            discovered = self.urls_discovered,
            elements = self.elements_extracted,
            assets = self.assets_downloaded,
        )
    }

    fn display_ascii(&self) -> String {
        let secs = self.duration.as_secs();
        let mins = secs / 60;
        let remain_secs = secs % 60;
        let dur = if mins > 0 {
            format!("{mins}m {remain_secs}s")
        } else {
            format!("{remain_secs}s")
        };

        format!(
            "\n[OK] scraped: {scraped}\n\
             [FAIL] failed: {failed}\n\
             [SKIP] skipped: {skipped}\n\
             [SUMMARY] {discovered} discovered, {elements} elements extracted, \
             {assets} assets downloaded, {dur} duration",
            scraped = self.urls_scraped,
            failed = self.urls_failed,
            skipped = self.urls_skipped,
            discovered = self.urls_discovered,
            elements = self.elements_extracted,
            assets = self.assets_downloaded,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_summary() -> ScrapeSummary {
        ScrapeSummary {
            urls_discovered: 10,
            urls_scraped: 7,
            urls_failed: 2,
            urls_skipped: 1,
            elements_extracted: 42,
            assets_downloaded: 5,
            duration: Duration::from_secs(45),
        }
    }

    #[test]
    fn test_summary_display_emoji() {
        let s = make_summary();
        let out = s.display(false);
        assert!(out.contains("Scrape Complete!"));
        assert!(out.contains("✅"));
        assert!(out.contains("❌ falhos: 2"));
        assert!(out.contains("⏭️  pulados: 1"));
        assert!(out.contains("📊 resumo"));
        assert!(out.contains("45s"));
    }

    #[test]
    fn test_summary_display_no_color() {
        let s = make_summary();
        let out = s.display(true);
        assert!(!out.contains("✅"));
        assert!(!out.contains("❌"));
        assert!(!out.contains("⏭️"));
        assert!(!out.contains("📊"));
        assert!(out.contains("[OK] scraped: 7"));
        assert!(out.contains("[FAIL] failed: 2"));
        assert!(out.contains("[SKIP] skipped: 1"));
        assert!(out.contains("[SUMMARY]"));
        assert!(out.contains("45s"));
    }

    #[test]
    fn test_summary_display_dur_minutes() {
        let s = ScrapeSummary {
            urls_discovered: 1,
            urls_scraped: 1,
            urls_failed: 0,
            urls_skipped: 0,
            elements_extracted: 1,
            assets_downloaded: 0,
            duration: Duration::from_secs(90),
        };
        let out = s.display(false);
        assert!(out.contains("1m 30s"));
    }
}
