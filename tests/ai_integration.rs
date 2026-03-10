//! AI Integration Tests
//!
//! Integration tests for AI-powered semantic cleaning features.
//! These tests are feature-gated behind the `ai` feature flag.
//!
//! # Running Tests
//!
//! ```bash
//! # Run all AI tests
//! cargo test --features ai --test ai_integration -- --nocapture
//!
//! # Run specific test
//! cargo test --features ai --test ai_integration test_semantic_cleaner_trait_defined
//! ```

#![cfg(feature = "ai")]

use rust_scraper::domain::DocumentChunk;
use rust_scraper::infrastructure::ai::model_cache::{
    default_cache_dir, CacheConfig, ModelCache, DEFAULT_MODEL_FILE, DEFAULT_MODEL_REPO,
};
use rust_scraper::infrastructure::ai::model_downloader::ModelDownloader;
use rust_scraper::infrastructure::ai::ModelConfig;
use rust_scraper::SemanticCleaner;
use rust_scraper::SemanticError;

/// Test that the SemanticCleaner trait is defined and accessible
#[test]
fn test_semantic_cleaner_trait_defined() {
    // This test verifies that the trait exists and can be referenced
    // It's a compile-time check more than a runtime test

    // If this compiles, the trait exists in the domain layer
    fn _assert_trait_exists<T: SemanticCleaner>(_cleaner: T) {}

    // We can't create a real instance without loading a model,
    // but we can verify the trait is accessible
}

/// Test that the model cache directory logic works correctly
#[tokio::test]
async fn test_model_cache_directory_created() {
    let temp_dir = tempfile::tempdir().unwrap();
    let cache_dir = temp_dir.path().join("test_ai_cache");

    let config = CacheConfig::new().with_cache_dir(cache_dir.clone());
    let cache = ModelCache::new(config);

    // Directory shouldn't exist yet
    assert!(!cache_dir.exists());

    // Create it
    cache.ensure_cache_dir().await.unwrap();

    // Now it should exist
    assert!(cache_dir.exists());
    assert!(cache_dir.is_dir());

    // Verify it's the right directory
    assert_eq!(cache.cache_dir(), &cache_dir);
}

/// Test that the model download structure is correct
#[tokio::test]
async fn test_model_download_structure() {
    // Test that ModelDownloader can be constructed with the right API
    let downloader = ModelDownloader::new()
        .with_repo(DEFAULT_MODEL_REPO)
        .with_file(DEFAULT_MODEL_FILE);

    assert_eq!(downloader.repo(), DEFAULT_MODEL_REPO);
    assert_eq!(downloader.file(), DEFAULT_MODEL_FILE);

    // Test that download_to method exists and has the right signature
    // (We don't actually download in this test to avoid network dependency)
    let temp_dir = tempfile::tempdir().unwrap();
    let result = downloader.download_to(temp_dir.path()).await;

    // This will fail because we're not actually downloading,
    // but it should fail with a proper error, not a compilation error
    assert!(result.is_err());

    // Verify the error type is correct
    if let Err(SemanticError::Download { repo, cause }) = result {
        assert_eq!(repo, DEFAULT_MODEL_REPO);
        assert!(!cause.is_empty());
    } else {
        panic!("Expected SemanticError::Download");
    }
}

/// Test that ModelConfig has the correct default values
#[test]
fn test_model_config_defaults() {
    let config = ModelConfig::default();
    
    assert_eq!(config.repo, DEFAULT_MODEL_REPO);
    assert_eq!(config.model_file, DEFAULT_MODEL_FILE);
    assert!(config.auto_download);
    assert!(!config.offline_mode);
    assert_eq!(config.max_tokens, 512);
    
    // Verify cache_dir ends with ai_models
    assert!(config.cache_dir.to_string_lossy().contains("ai_models"));
}

/// Test that ModelConfig builder pattern works
#[test]
fn test_model_config_builder() {
    let temp_dir = tempfile::tempdir().unwrap();
    
    let config = ModelConfig::new()
        .with_repo("test/repo")
        .with_file("test.onnx")
        .with_cache_dir(temp_dir.path().to_path_buf())
        .with_auto_download(false)
        .with_offline_mode(true)
        .with_max_tokens(256);
    
    assert_eq!(config.repo, "test/repo");
    assert_eq!(config.model_file, "test.onnx");
    assert_eq!(config.cache_dir, temp_dir.path());
    assert!(!config.auto_download);
    assert!(config.offline_mode);
    assert_eq!(config.max_tokens, 256);
}

/// Test that ModelConfig offline mode is configured correctly
#[test]
fn test_semantic_cleaner_offline_mode_config() {
    let temp_dir = tempfile::tempdir().unwrap();

    let config = ModelConfig::new()
        .with_cache_dir(temp_dir.path().to_path_buf())
        .with_auto_download(false)
        .with_offline_mode(true);

    // Verify configuration
    assert!(!config.auto_download);
    assert!(config.offline_mode);
    assert_eq!(config.cache_dir, temp_dir.path());
}

/// Test that DocumentChunk can be created (verifies domain integration)
#[test]
fn test_document_chunk_creation() {
    let chunk = DocumentChunk {
        id: uuid::Uuid::new_v4(),
        url: "https://example.com".to_string(),
        title: "Test Page".to_string(),
        content: "Test content".to_string(),
        metadata: std::collections::HashMap::new(),
        timestamp: chrono::Utc::now(),
        embeddings: None,
    };
    
    assert_eq!(chunk.url, "https://example.com");
    assert_eq!(chunk.title, "Test Page");
    assert_eq!(chunk.content, "Test content");
    assert!(!chunk.has_embeddings());
}

/// Test that default_cache_dir returns a valid path
#[test]
fn test_default_cache_dir() {
    let cache_dir = default_cache_dir();
    
    // Should end with ai_models
    assert!(cache_dir.to_string_lossy().ends_with("ai_models"));
    
    // Should contain rust-scraper
    assert!(cache_dir.to_string_lossy().contains("rust-scraper"));
}

/// Test that ModelCache can check if a model is cached
#[tokio::test]
async fn test_model_cache_is_cached() {
    let temp_dir = tempfile::tempdir().unwrap();
    let cache_dir = temp_dir.path().join("test_cache");
    
    let config = CacheConfig::new().with_cache_dir(cache_dir.clone());
    let cache = ModelCache::new(config);
    
    // Should return false for non-existent file
    assert!(!cache.is_model_cached("model.onnx"));
    
    // Create a dummy file
    tokio::fs::create_dir_all(&cache_dir).await.unwrap();
    tokio::fs::File::create(cache_dir.join("model.onnx"))
        .await
        .unwrap();
    
    // Should return true now
    assert!(cache.is_model_cached("model.onnx"));
}

/// Test that ModelCache can get model path
#[test]
fn test_model_cache_model_path() {
    let temp_dir = tempfile::tempdir().unwrap();
    let cache_dir = temp_dir.path().join("test_cache");
    
    let config = CacheConfig::new().with_cache_dir(cache_dir.clone());
    let cache = ModelCache::new(config);
    
    let model_path = cache.model_path("model.onnx");
    assert_eq!(model_path, cache_dir.join("model.onnx"));
}

/// Test that DownloadProgress calculations work correctly
#[test]
fn test_download_progress_calculations() {
    use rust_scraper::infrastructure::ai::DownloadProgress;
    
    // Test percentage calculation
    let progress = DownloadProgress {
        downloaded: 50,
        total: Some(100),
        speed: None,
        eta_seconds: None,
    };
    
    assert_eq!(progress.percentage(), Some(50.0));
    assert!(!progress.is_complete());
    
    // Test complete download
    let progress = DownloadProgress {
        downloaded: 100,
        total: Some(100),
        speed: None,
        eta_seconds: None,
    };
    
    assert_eq!(progress.percentage(), Some(100.0));
    assert!(progress.is_complete());
    
    // Test no total
    let progress = DownloadProgress {
        downloaded: 50,
        total: None,
        speed: None,
        eta_seconds: None,
    };
    
    assert!(progress.percentage().is_none());
    assert!(!progress.is_complete());
}

/// Test that SemanticError variants are properly defined
#[test]
fn test_semantic_error_variants() {
    // Test ModelLoad error
    let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
    let err = SemanticError::ModelLoad(io_err);
    assert!(err.to_string().contains("cargando modelo"));
    
    // Test ChunkTooLarge error
    let err = SemanticError::ChunkTooLarge {
        chunk_id: "chunk-1".to_string(),
        tokens: 600,
        max: 512,
    };
    assert!(err.to_string().contains("chunk-1"));
    assert!(err.to_string().contains("600 > 512"));
    
    // Test Download error
    let err = SemanticError::Download {
        repo: "test/repo".to_string(),
        cause: "network error".to_string(),
    };
    assert!(err.to_string().contains("test/repo"));
    assert!(err.to_string().contains("network error"));
    
    // Test CacheValidation error
    let err = SemanticError::CacheValidation {
        repo: "test/repo".to_string(),
        expected: "abc123".to_string(),
        actual: "def456".to_string(),
    };
    assert!(err.to_string().contains("abc123"));
    assert!(err.to_string().contains("def456"));
    
    // Test OfflineMode error
    let err = SemanticError::OfflineMode {
        repo: "test/repo".to_string(),
    };
    assert!(err.to_string().contains("test/repo"));
    assert!(err.to_string().contains("offline"));
}

/// Test that ScraperError can be created from SemanticError
#[test]
fn test_scraper_error_from_semantic_error() {
    use rust_scraper::ScraperError;
    
    let semantic_err = SemanticError::ModelLoad(
        std::io::Error::new(std::io::ErrorKind::NotFound, "model missing")
    );
    
    let scraper_err: ScraperError = semantic_err.into();
    assert!(scraper_err.to_string().contains("limpieza semántica"));
}
