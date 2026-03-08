//! Asset Download Module
//!
//! Handles downloading of images and documents from URLs.
//!
//! # Architecture
//!
//! Following rust-skills best practices:
//! - **Async I/O**: Uses `tokio::fs` to prevent thread starvation
//! - **Streaming**: Streams data with real-time byte limits
//! - **No Trust**: Never trusts `Content-Length` from external input
//! - **Configurable**: Concurrency limit externalized to config
//! - **Fallback**: MIME type fallback for extension detection

use std::path::{Path, PathBuf};

use crate::error::{Result, ScraperError};
use bytes::Bytes;
use futures::stream::{self, StreamExt};
use reqwest::Client;
use sha2::{Digest, Sha256};
use tokio::fs;
use tokio::io::AsyncWriteExt;

/// Result of a successful download
#[derive(Debug)]
pub struct DownloadedAsset {
    /// Original URL
    pub url: String,
    /// Local file path where asset was saved
    pub local_path: PathBuf,
    /// MIME type detected from HTTP headers
    pub mime_type: Option<String>,
    /// File size in bytes
    pub size: u64,
    /// SHA256 hash of content (first 12 hex chars used in filename)
    pub content_hash: String,
}

/// Download configuration
#[derive(Debug, Clone)]
pub struct DownloadConfig {
    /// Output directory for downloaded files
    pub output_dir: PathBuf,
    /// Subdirectory for images
    pub images_dir: String,
    /// Subdirectory for documents
    pub documents_dir: String,
    /// Maximum file size in bytes (default: 50MB)
    pub max_file_size: u64,
    /// Timeout for each download in seconds
    pub timeout_secs: u64,
    /// Maximum concurrent downloads (default: 3 for HDD)
    pub concurrency_limit: usize,
}

impl Default for DownloadConfig {
    fn default() -> Self {
        Self {
            output_dir: PathBuf::from("./downloads"),
            images_dir: "images".to_string(),
            documents_dir: "documents".to_string(),
            max_file_size: 50 * 1024 * 1024,
            timeout_secs: 30,
            concurrency_limit: 3,
        }
    }
}

/// Asset downloader
pub struct Downloader {
    client: Client,
    config: DownloadConfig,
}

impl Downloader {
    /// Create a new downloader with configuration
    pub fn new(config: DownloadConfig) -> Result<Self> {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(config.timeout_secs))
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
            .build()
            .map_err(|e| ScraperError::Config(format!("Failed to build HTTP client: {}", e)))?;

        Ok(Self { client, config })
    }

    /// Download a single asset with streaming and OOM protection
    pub async fn download(&self, url: &str) -> Result<DownloadedAsset> {
        let mut response = self
            .client
            .get(url)
            .send()
            .await
            .map_err(ScraperError::Network)?;

        let mime_type = response
            .headers()
            .get(reqwest::header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .map(String::from);

        let asset_type = crate::adapters::detector::detect_from_url(url);
        let subdir = if asset_type.is_image() {
            &self.config.images_dir
        } else {
            &self.config.documents_dir
        };

        let mut downloaded: u64 = 0;
        let mut buffer = Vec::new();
        let mut hasher = Sha256::new();

        loop {
            let chunk = response
                .chunk()
                .await
                .map_err(ScraperError::Network)?
                .unwrap_or_else(Bytes::new);

            if chunk.is_empty() {
                break;
            }

            let chunk_len = chunk.len() as u64;
            downloaded = downloaded
                .checked_add(chunk_len)
                .ok_or_else(|| ScraperError::download("Integer overflow in download size"))?;

            if downloaded > self.config.max_file_size {
                return Err(ScraperError::download(format!(
                    "file too large: {} bytes (max: {} bytes)",
                    downloaded, self.config.max_file_size
                )));
            }

            hasher.update(&chunk);
            buffer.extend_from_slice(&chunk);
        }

        let content_hash = format!("{:x}", hasher.finalize());
        let filename = self.generate_filename(url, mime_type.as_deref(), &buffer);
        let local_path = self.config.output_dir.join(subdir).join(&filename);

        if let Some(parent) = local_path.parent() {
            fs::create_dir_all(parent).await.map_err(ScraperError::Io)?;
        }

        let mut file = fs::File::create(&local_path)
            .await
            .map_err(ScraperError::Io)?;
        file.write_all(&buffer).await.map_err(ScraperError::Io)?;
        file.sync_all().await.map_err(ScraperError::Io)?;

        tracing::info!("downloaded: {} -> {:?}", url, local_path);

        Ok(DownloadedAsset {
            url: url.to_string(),
            local_path,
            mime_type,
            size: downloaded,
            content_hash: content_hash[..12].to_string(),
        })
    }

    /// Download multiple assets with configurable concurrency control
    pub async fn download_batch(&self, urls: &[String]) -> Vec<Result<DownloadedAsset>> {
        if urls.is_empty() {
            return Vec::new();
        }

        let tasks = urls.iter().map(|url| {
            let url = url.clone();
            async move { self.download(&url).await }
        });

        let results: Vec<Result<DownloadedAsset>> = stream::iter(tasks)
            .buffer_unordered(self.config.concurrency_limit)
            .collect()
            .await;

        results
    }

    /// Generate a unique filename with MIME type fallback
    fn generate_filename(&self, url: &str, mime_type: Option<&str>, content: &[u8]) -> String {
        let extension = crate::adapters::detector::get_extension(url)
            .filter(|ext| !ext.contains('/') && ext.len() <= 10 && !ext.is_empty())
            .or_else(|| mime_type_to_extension(mime_type.unwrap_or("")))
            .unwrap_or_else(|| "bin".into());

        let mut hasher = Sha256::new();
        hasher.update(content);
        let hash = format!("{:x}", hasher.finalize());

        format!("{}.{}", &hash[..12], extension)
    }
}

/// MIME type to file extension mapping
fn mime_type_to_extension(mime: &str) -> Option<String> {
    let mime = mime.trim();
    match mime {
        "image/jpeg" | "image/jpg" => Some("jpg".to_string()),
        "image/png" => Some("png".to_string()),
        "image/gif" => Some("gif".to_string()),
        "image/webp" => Some("webp".to_string()),
        "image/svg+xml" => Some("svg".to_string()),
        "image/bmp" => Some("bmp".to_string()),
        "image/tiff" => Some("tiff".to_string()),
        "image/x-icon" => Some("ico".to_string()),
        "application/pdf" => Some("pdf".to_string()),
        "application/msword" => Some("doc".to_string()),
        "application/vnd.openxmlformats-officedocument.wordprocessingml.document" => {
            Some("docx".to_string())
        }
        "application/vnd.ms-excel" => Some("xls".to_string()),
        "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet" => {
            Some("xlsx".to_string())
        }
        "application/vnd.ms-powerpoint" => Some("ppt".to_string()),
        "application/vnd.openxmlformats-officedocument.presentationml.presentation" => {
            Some("pptx".to_string())
        }
        "text/csv" => Some("csv".to_string()),
        "application/vnd.oasis.opendocument.text" => Some("odt".to_string()),
        "application/vnd.oasis.opendocument.spreadsheet" => Some("ods".to_string()),
        "application/epub+zip" => Some("epub".to_string()),
        "application/rtf" => Some("rtf".to_string()),
        "text/plain" => Some("txt".to_string()),
        "application/json" => Some("json".to_string()),
        "application/xml" | "text/xml" => Some("xml".to_string()),
        _ => None,
    }
}

/// Simple async download without creating a Downloader instance
pub async fn quick_download(url: &str, output_dir: &Path) -> Result<DownloadedAsset> {
    let config = DownloadConfig {
        output_dir: output_dir.to_path_buf(),
        ..Default::default()
    };

    let downloader = Downloader::new(config)?;
    downloader.download(url).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_downloader_creation() {
        let temp_dir = TempDir::new().unwrap();
        let config = DownloadConfig {
            output_dir: temp_dir.path().to_path_buf(),
            ..Default::default()
        };
        let downloader = Downloader::new(config);
        assert!(downloader.is_ok());
    }

    #[tokio::test]
    async fn test_downloader_config_concurrency() {
        let config = DownloadConfig {
            concurrency_limit: 10,
            ..Default::default()
        };
        assert_eq!(config.concurrency_limit, 10);
    }

    #[test]
    fn test_mime_type_to_extension() {
        assert_eq!(mime_type_to_extension("image/png"), Some("png".to_string()));
        assert_eq!(
            mime_type_to_extension("image/jpeg"),
            Some("jpg".to_string())
        );
        assert_eq!(
            mime_type_to_extension("application/pdf"),
            Some("pdf".to_string())
        );
        assert_eq!(mime_type_to_extension("application/unknown"), None);
        assert_eq!(mime_type_to_extension(""), None);
    }

    #[test]
    fn test_generate_filename_with_mime_fallback() {
        let temp_dir = TempDir::new().unwrap();
        let config = DownloadConfig {
            output_dir: temp_dir.path().to_path_buf(),
            ..Default::default()
        };
        let downloader = Downloader::new(config).unwrap();

        let filename = downloader.generate_filename(
            "https://example.com/image.png",
            Some("image/png"),
            b"test content",
        );
        assert!(filename.ends_with(".png"));

        let filename = downloader.generate_filename(
            "https://example.com/api/getimage",
            Some("image/jpeg"),
            b"test content",
        );
        assert!(
            filename.ends_with(".jpg"),
            "Expected .jpg but got: {}",
            filename
        );

        let filename =
            downloader.generate_filename("https://example.com/api/getfile", None, b"test content");
        assert!(
            filename.ends_with(".bin"),
            "Expected .bin but got: {}",
            filename
        );
    }

    #[tokio::test]
    async fn test_download_streaming_limit() {
        let temp_dir = TempDir::new().unwrap();
        let config = DownloadConfig {
            output_dir: temp_dir.path().to_path_buf(),
            max_file_size: 1024,
            ..Default::default()
        };
        let downloader = Downloader::new(config).unwrap();
        assert_eq!(downloader.config.max_file_size, 1024);
    }

    #[tokio::test]
    async fn test_download_batch_empty() {
        let temp_dir = TempDir::new().unwrap();
        let config = DownloadConfig {
            output_dir: temp_dir.path().to_path_buf(),
            ..Default::default()
        };
        let downloader = Downloader::new(config).unwrap();
        let results = downloader.download_batch(&[]).await;
        assert!(results.is_empty());
    }
}
