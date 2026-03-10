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
use rust_scraper::infrastructure::ai::{InferenceEngine, ModelConfig};
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

// ============================================================================
// InferenceEngine Tests (NEW - Phase 2)
// ============================================================================

/// Test that InferenceEngine type exists and compiles
///
/// This is a compile-time check - if this compiles, the type exists
/// with the correct structure and API.
#[test]
fn test_inference_engine_type_exists() {
    // This is a compile-time check
    // If this compiles, the type exists with the correct structure
    fn _assert_type_exists(_engine: InferenceEngine) {}
}

/// Test that InferenceEngine is Send + Sync (thread-safe)
///
/// This is critical for using InferenceEngine in async contexts
/// with tokio::spawn and across thread boundaries.
///
/// Following `own-arc-shared` and `async-spawn-blocking` rules,
/// InferenceEngine must be Send + Sync to work with Arc and spawn_blocking.
#[test]
fn test_inference_engine_is_send_sync() {
    fn assert_send<T: Send>() {}
    fn assert_sync<T: Sync>() {}

    assert_send::<InferenceEngine>();
    assert_sync::<InferenceEngine>();
}

/// Test that InferenceEngine is Clone (cheap Arc clone)
///
/// InferenceEngine wraps Arc<RunnableModel>, so cloning is cheap
/// (just increments atomic counter) and safe for concurrent use.
#[test]
fn test_inference_engine_is_clone() {
    fn assert_clone<T: Clone>() {}
    assert_clone::<InferenceEngine>();
}

/// Test that TokenBatch can be created
///
/// Verifies the token batch structure for batch inference.
#[test]
fn test_token_batch_creation() {
    use rust_scraper::infrastructure::ai::tokenizer::TokenBatch;

    let batch = TokenBatch::new(
        vec![vec![1, 2, 3], vec![4, 5, 6]],
        vec![vec![1, 1, 1], vec![1, 1, 1]],
        vec![vec![0, 0, 0], vec![0, 0, 0]],
    );

    assert_eq!(batch.len(), 2);
    assert_eq!(batch.sequence_length(), 3);
    assert!(!batch.is_empty());
}

/// Test tokenizer type traits
///
/// Verifies that MiniLmTokenizer has the correct Send/Sync properties.
#[test]
fn test_tokenizer_type_traits() {
    use rust_scraper::infrastructure::ai::tokenizer::MiniLmTokenizer;

    fn assert_send<T: Send>() {}

    // MiniLmTokenizer should be Send (can be moved between threads)
    // but not necessarily Sync (internal state may not be thread-safe)
    assert_send::<MiniLmTokenizer>();
}

// ============================================================================
// Module 3 Tests: Semantic Chunking (ChunkId, Sentence, Chunker)
// ============================================================================

/// Test that ChunkId type exists and compiles
#[test]
fn test_chunk_id_type_exists() {
    use rust_scraper::infrastructure::ai::ChunkId;

    fn _assert_type_exists(_id: ChunkId) {}
}

/// Test ChunkId creation and display
#[test]
fn test_chunk_id_display() {
    use rust_scraper::infrastructure::ai::ChunkId;

    let id = ChunkId(42);
    assert_eq!(format!("{}", id), "chunk-42");
}

/// Test ChunkId inner value access
#[test]
fn test_chunk_id_inner() {
    use rust_scraper::infrastructure::ai::ChunkId;

    let id = ChunkId::new(123);
    assert_eq!(id.inner(), 123);
}

/// Test ChunkId equality
#[test]
fn test_chunk_id_equality() {
    use rust_scraper::infrastructure::ai::ChunkId;

    let id1 = ChunkId(42);
    let id2 = ChunkId(42);
    let id3 = ChunkId(43);

    assert_eq!(id1, id2);
    assert_ne!(id1, id3);
}

/// Test that SentenceSplitter type exists
#[test]
fn test_sentence_splitter_type_exists() {
    use rust_scraper::infrastructure::ai::SentenceSplitter;

    fn _assert_type_exists(_splitter: SentenceSplitter) {}
}

/// Test sentence splitter basic functionality
#[test]
fn test_sentence_splitter_basic() {
    use rust_scraper::infrastructure::ai::SentenceSplitter;

    let splitter = SentenceSplitter;
    let sentences = splitter.split("Hello world. How are you?");
    assert!(sentences.len() >= 2);
}

/// Test sentence splitter count
#[test]
fn test_sentence_splitter_count() {
    use rust_scraper::infrastructure::ai::SentenceSplitter;

    let splitter = SentenceSplitter;
    let count = splitter.count("One. Two. Three.");
    assert_eq!(count, 3);
}

/// Test sentence splitter trimmed output
#[test]
fn test_sentence_splitter_trimmed() {
    use rust_scraper::infrastructure::ai::SentenceSplitter;

    let splitter = SentenceSplitter;
    let sentences = splitter.split_trimmed("  First.  Second.  Third.  ");
    assert_eq!(sentences.len(), 3);
    assert_eq!(sentences[0], "First.");
}

/// Test that HtmlChunker type exists
#[test]
fn test_chunker_type_exists() {
    use rust_scraper::infrastructure::ai::HtmlChunker;

    fn _assert_type_exists(_chunker: HtmlChunker) {}
}

/// Test chunker creation with defaults
#[test]
fn test_chunker_creation() {
    use rust_scraper::infrastructure::ai::HtmlChunker;

    let chunker = HtmlChunker::new();
    assert!(chunker.min_chunk_size() > 0);
    assert!(chunker.max_chunk_size() > 0);
    assert!(chunker.similarity_threshold() > 0.0);
    assert!(chunker.similarity_threshold() <= 1.0);
}

/// Test chunker builder pattern
#[test]
fn test_chunker_builder_pattern() {
    use rust_scraper::infrastructure::ai::HtmlChunker;

    let chunker = HtmlChunker::new()
        .with_min_chunk_size(80)
        .with_max_chunk_size(400)
        .with_similarity_threshold(0.6);

    assert_eq!(chunker.min_chunk_size(), 80);
    assert_eq!(chunker.max_chunk_size(), 400);
    assert_eq!(chunker.similarity_threshold(), 0.6);
}

/// Test chunker with custom config
#[test]
fn test_chunker_with_config() {
    use rust_scraper::infrastructure::ai::HtmlChunker;

    let chunker = HtmlChunker::with_config(50, 300, 0.7);
    assert_eq!(chunker.min_chunk_size(), 50);
    assert_eq!(chunker.max_chunk_size(), 300);
    assert_eq!(chunker.similarity_threshold(), 0.7);
}

/// Test chunker basic HTML processing
#[test]
fn test_chunker_basic_html() {
    use rust_scraper::infrastructure::ai::HtmlChunker;

    let chunker = HtmlChunker::new();
    let html = "<p>This is a paragraph with enough text to meet the minimum chunk size requirement for testing purposes.</p>";
    let result = chunker.chunk(html);
    assert!(result.is_ok());
}

/// Test chunker empty HTML
#[test]
fn test_chunker_empty_html() {
    use rust_scraper::infrastructure::ai::HtmlChunker;

    let chunker = HtmlChunker::new();
    let html = "";
    let result = chunker.chunk(html);
    assert!(result.is_ok());
    assert!(result.unwrap().is_empty());
}

// ============================================================================
// Module 4 Tests: Embedding Operations, Relevance Scorer, Threshold Config
// ============================================================================

/// Test cosine similarity with identical vectors
#[test]
fn test_cosine_similarity_identical() {
    use rust_scraper::infrastructure::ai::embedding_ops::cosine_similarity;

    // Use a normalized vector (magnitude = 1.0)
    // 1/sqrt(8) ≈ 0.3536 for 8-dimensional unit vector
    let normalization = 1.0f32 / 8.0f32.sqrt();
    let vec = vec![normalization; 8];
    let sim = cosine_similarity(&vec, &vec);
    assert!((sim - 1.0).abs() < 0.001, "Expected ~1.0, got {}", sim);
}

/// Test cosine similarity with orthogonal vectors
#[test]
fn test_cosine_similarity_orthogonal() {
    use rust_scraper::infrastructure::ai::embedding_ops::cosine_similarity;

    let a = vec![1.0f32, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];
    let b = vec![0.0f32, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];
    let sim = cosine_similarity(&a, &b);
    assert!(sim.abs() < 0.001, "Expected ~0.0, got {}", sim);
}

/// Test cosine similarity with opposite vectors
#[test]
fn test_cosine_similarity_opposite() {
    use rust_scraper::infrastructure::ai::embedding_ops::cosine_similarity;

    let a = vec![1.0f32, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];
    let b = vec![-1.0f32, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];
    let sim = cosine_similarity(&a, &b);
    assert!((sim + 1.0).abs() < 0.001, "Expected ~-1.0, got {}", sim);
}

/// Test cosine similarity with empty vectors
#[test]
fn test_cosine_similarity_empty() {
    use rust_scraper::infrastructure::ai::embedding_ops::cosine_similarity;

    let a: Vec<f32> = vec![];
    let b: Vec<f32> = vec![];
    let sim = cosine_similarity(&a, &b);
    assert_eq!(sim, 0.0);
}

/// Test dot product scalar fallback
#[test]
fn test_dot_product_scalar() {
    use rust_scraper::infrastructure::ai::embedding_ops::dot_product_scalar;

    let a = vec![1.0f32, 2.0, 3.0];
    let b = vec![4.0f32, 5.0, 6.0];
    let dot = dot_product_scalar(&a, &b);
    assert_eq!(dot, 32.0); // 1*4 + 2*5 + 3*6 = 32
}

/// Test vector normalization
#[test]
fn test_normalize() {
    use rust_scraper::infrastructure::ai::embedding_ops::normalize;

    let v = vec![3.0f32, 4.0];
    let normalized = normalize(&v);
    let magnitude: f32 = normalized.iter().map(|&x| x * x).sum::<f32>().sqrt();
    assert!((magnitude - 1.0).abs() < 0.001);
}

/// Test Euclidean distance
#[test]
fn test_euclidean_distance() {
    use rust_scraper::infrastructure::ai::embedding_ops::euclidean_distance;

    let a = vec![0.0f32, 0.0];
    let b = vec![3.0f32, 4.0];
    let dist = euclidean_distance(&a, &b);
    assert!((dist - 5.0).abs() < 0.001); // 3-4-5 triangle
}

/// Test that RelevanceScorer type exists
#[test]
fn test_relevance_scorer_type_exists() {
    use rust_scraper::infrastructure::ai::RelevanceScorer;

    fn _assert_type_exists(_scorer: RelevanceScorer) {}
}

/// Test relevance scorer creation
#[test]
fn test_relevance_scorer_creation() {
    use rust_scraper::infrastructure::ai::RelevanceScorer;

    let scorer = RelevanceScorer::new(0.3);
    assert_eq!(scorer.threshold(), 0.3);
}

/// Test relevance scorer with reference
#[test]
fn test_relevance_scorer_with_reference() {
    use rust_scraper::infrastructure::ai::RelevanceScorer;

    let reference = vec![0.5f32; 8];
    let scorer = RelevanceScorer::with_reference(0.5, reference.clone());
    assert_eq!(scorer.threshold(), 0.5);
    assert_eq!(scorer.reference(), Some(reference.as_slice()));
}

/// Test relevance scorer threshold validation
#[test]
#[should_panic(expected = "Threshold must be between")]
fn test_relevance_scorer_invalid_threshold() {
    use rust_scraper::infrastructure::ai::RelevanceScorer;

    let _ = RelevanceScorer::new(1.5);
}

/// Test relevance scorer meets_threshold
#[test]
fn test_relevance_scorer_meets_threshold() {
    use rust_scraper::infrastructure::ai::RelevanceScorer;

    let scorer = RelevanceScorer::new(0.5);
    assert!(scorer.meets_threshold(0.6));
    assert!(scorer.meets_threshold(0.5));
    assert!(!scorer.meets_threshold(0.4));
}

/// Test that ThresholdConfig type exists
#[test]
fn test_threshold_config_type_exists() {
    use rust_scraper::infrastructure::ai::ThresholdConfig;

    fn _assert_type_exists(_config: ThresholdConfig) {}
}

/// Test threshold config default values
#[test]
fn test_threshold_config_defaults() {
    use rust_scraper::infrastructure::ai::ThresholdConfig;

    let config = ThresholdConfig::new();
    assert_eq!(config.min_threshold(), 0.0);
    assert_eq!(config.max_threshold(), 1.0);
    assert_eq!(config.default_threshold(), 0.3);
}

/// Test threshold config builder pattern
#[test]
fn test_threshold_config_builder() {
    use rust_scraper::infrastructure::ai::ThresholdConfig;

    let config = ThresholdConfig::new()
        .with_min_threshold(0.2)
        .with_max_threshold(0.8)
        .with_default_threshold(0.5)
        .build();

    assert_eq!(config.min_threshold(), 0.2);
    assert_eq!(config.max_threshold(), 0.8);
    assert_eq!(config.default_threshold(), 0.5);
}

/// Test threshold config is_valid
#[test]
fn test_threshold_config_is_valid() {
    use rust_scraper::infrastructure::ai::ThresholdConfig;

    let config = ThresholdConfig::new()
        .with_min_threshold(0.2)
        .with_max_threshold(0.8)
        .build();

    assert!(config.is_valid(0.5));
    assert!(!config.is_valid(0.1));
}

/// Test threshold config clamp
#[test]
fn test_threshold_config_clamp() {
    use rust_scraper::infrastructure::ai::ThresholdConfig;

    let config = ThresholdConfig::new()
        .with_min_threshold(0.2)
        .with_max_threshold(0.8)
        .build();

    assert_eq!(config.clamp(0.1), 0.2);
    assert_eq!(config.clamp(0.5), 0.5);
    assert_eq!(config.clamp(0.9), 0.8);
}

/// Test threshold config strict preset
#[test]
fn test_threshold_config_strict() {
    use rust_scraper::infrastructure::ai::ThresholdConfig;

    let config = ThresholdConfig::strict();
    assert_eq!(config.min_threshold(), 0.5);
    assert_eq!(config.max_threshold(), 1.0);
    assert_eq!(config.default_threshold(), 0.7);
}

/// Test threshold config lenient preset
#[test]
fn test_threshold_config_lenient() {
    use rust_scraper::infrastructure::ai::ThresholdConfig;

    let config = ThresholdConfig::lenient();
    assert_eq!(config.min_threshold(), 0.0);
    assert_eq!(config.max_threshold(), 0.5);
    assert_eq!(config.default_threshold(), 0.2);
}

/// Test threshold config balanced preset
#[test]
fn test_threshold_config_balanced() {
    use rust_scraper::infrastructure::ai::ThresholdConfig;

    let config = ThresholdConfig::balanced();
    assert_eq!(config.min_threshold(), 0.1);
    assert_eq!(config.max_threshold(), 0.9);
    assert_eq!(config.default_threshold(), 0.4);
}
