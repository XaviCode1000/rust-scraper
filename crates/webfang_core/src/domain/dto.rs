use serde::{Deserialize, Serialize};
use std::time::SystemTime;

/// Canonical export record shared between CLI and MCP.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ExportRecord {
    /// Original URL provided by the user.
    pub url: String,
    /// HTTP status code of the final response.
    pub status_code: u16,
    /// Final URL after any redirects.
    pub final_url: String,
    /// SHA-256 hash of the content (if any).
    pub content_sha256: Option<String>,
    /// Timestamp of when the export was created.
    pub timestamp: SystemTime,
    /// Number of words in the content (if text).
    pub word_count: Option<usize>,
    /// Version of the metadata format.
    pub metadata_version: String,
    /// Title extracted from the content (if any).
    pub title: Option<String>,
    /// Description extracted from the content (if any).
    pub description: Option<String>,
}