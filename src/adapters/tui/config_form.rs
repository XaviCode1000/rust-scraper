//! Config Form Module
//!
//! Interactive configuration form using ratatui-form.

use crossterm::event::KeyEvent;
use ratatui::{layout::Rect, Frame};
use ratatui_form::{Form, FormResult};
use serde_json::Value;

/// State for the configuration form
pub struct ConfigFormState {
    /// The underlying form
    pub form: Form,
    /// Whether the form has been submitted
    pub submitted: bool,
    /// Whether the form was cancelled
    pub cancelled: bool,
}

impl ConfigFormState {
    /// Create a new ConfigFormState from a form
    pub fn new(form: Form) -> Self {
        Self {
            form,
            submitted: false,
            cancelled: false,
        }
    }

    /// Create a new config form with default values.
    pub fn new_default() -> Self {
        Self::new(Self::build_form())
    }

    /// Build the configuration form with all sections
    fn build_form() -> Form {
        let mut builder = Form::builder().title("Scraper Configuration");

        // ========================================
        // Output Section (format, export_format, output)
        // ========================================
        builder = builder
            .text("output", "Output Directory")
            .initial_value("output")
            .done()
            .select("format", "Output Format")
            .option("markdown", "Markdown")
            .option("json", "JSON")
            .option("text", "Plain Text")
            .initial_value("markdown")
            .done()
            .select("export_format", "Export Format")
            .option("jsonl", "JSONL")
            .option("vector", "Vector")
            .option("auto", "Auto")
            .initial_value("jsonl")
            .done();

        // ========================================
        // Discovery Section (use_sitemap, max_pages, max_depth)
        // ========================================
        builder = builder
            .checkbox("use_sitemap", "Use Sitemap")
            .checked(false)
            .done()
            .text("max_pages", "Max Pages")
            .initial_value("10")
            .done()
            .text("max_depth", "Max Depth")
            .initial_value("2")
            .done();

        // ========================================
        // Download Section (download_images, download_documents)
        // ========================================
        builder = builder
            .checkbox("download_images", "Download Images")
            .checked(false)
            .done()
            .checkbox("download_documents", "Download Documents")
            .checked(false)
            .done();

        // ========================================
        // Obsidian Section (obsidian_wiki_links, vault, quick_save)
        // ========================================
        builder = builder
            .checkbox("obsidian_wiki_links", "Obsidian Wiki Links")
            .checked(false)
            .done()
            .text("vault", "Vault Path")
            .initial_value("")
            .done()
            .checkbox("quick_save", "Quick Save")
            .checked(false)
            .done();

        // ========================================
        // AI Section (clean_ai) — feature-gated
        // ========================================
        #[cfg(feature = "ai")]
        {
            builder = builder
                .checkbox("clean_ai", "AI Cleaning")
                .checked(false)
                .done();
        }

        builder.build()
    }

    /// Handle a keyboard event, updating the form state.
    pub fn handle_input(&mut self, key: KeyEvent) {
        self.form.handle_input(key);
        match self.form.result() {
            FormResult::Submitted => self.submitted = true,
            FormResult::Cancelled => self.cancelled = true,
            FormResult::Active => {},
        }
    }

    /// Render the form to the given frame area.
    pub fn render(&self, frame: &mut Frame, area: Rect) {
        self.form.render(area, frame.buffer_mut());
    }

    /// Get the form data as JSON.
    pub fn data(&self) -> Value {
        self.form.to_json()
    }

    /// Mark the form as submitted
    pub fn mark_submitted(&mut self) {
        self.submitted = true;
    }

    /// Mark the form as cancelled
    pub fn mark_cancelled(&mut self) {
        self.cancelled = true;
    }

    /// Check if the form interaction is complete
    pub fn is_done(&self) -> bool {
        self.submitted || self.cancelled
    }
}
