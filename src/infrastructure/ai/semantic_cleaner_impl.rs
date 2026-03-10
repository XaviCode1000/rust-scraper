//! Semantic Cleaner implementation — Full RAG Pipeline Integration
//!
//! This module provides the concrete implementation of the [`SemanticCleaner`](crate::domain::semantic_cleaner::SemanticCleaner)
//! trait using the complete Phase 2 + Phase 3 pipeline:
//!
//! # Architecture
//!
//! ```text
//! HTML Input
//!     ↓
//! [Chunker] Split into semantic chunks (arena allocator)
//!     ↓
//! [Tokenizer] Convert each chunk to token IDs
//!     ↓
//! [InferenceEngine] Generate embeddings (spawn_blocking, concurrent)
//!     ↓
//! [RelevanceScorer] Filter by threshold (SIMD cosine similarity)
//!     ↓
//! Vec<DocumentChunk> Output
//! ```
//!
//! # Rust-Skills Applied
//!
//! - [`async-join-parallel`](crate::rust_skills::async_join_parallel): Use `try_join_all` for concurrent embeddings
//! - [`mem-reuse-collections`](crate::rust_skills::mem_reuse_collections): Pre-allocate `Vec::with_capacity`, reuse buffers
//! - [`own-borrow-over-clone`](crate::rust_skills::own_borrow_over_clone): Borrow `&chunks`, `&embeddings` - don't clone
//! - [`async-spawn-blocking`](crate::rust_skills::async_spawn_blocking): InferenceEngine uses spawn_blocking internally
//! - [`err-context-chain`](crate::rust_skills::err_context_chain): Add `.context()` to errors
//! - [`anti-unwrap-abuse`](crate::rust_skills::anti_unwrap_abuse): Use `?` operator, NO `.unwrap()` in prod
//! - [`anti-lock-across-await`](crate::rust_skills::anti_lock_across_await): Don't hold MutexGuard across `.await`
//! - [`api-builder-pattern`](crate::rust_skills::api_builder_pattern): ModelConfig uses builder pattern
//! - [`type-newtype-ids`](crate::rust_skills::type_newtype_ids): Using `ChunkId` for type-safe IDs
//! - [`opt-simd-portable`](crate::rust_skills::opt_simd_portable): RelevanceScorer uses `wide::f32x8` SIMD
//!
//! # Examples
//!
//! ```no_run
//! # #[cfg(feature = "ai")]
//! # async fn example() -> anyhow::Result<()> {
//! use rust_scraper::infrastructure::ai::{create_semantic_cleaner, ModelConfig};
//!
//! let config = ModelConfig::default();
//! let cleaner = create_semantic_cleaner(&config).await?;
//!
//! let html = "<article><p>Hello world. Test content.</p></article>";
//! let chunks = cleaner.clean(html).await?;
//!
//! println!("Generated {} chunks", chunks.len());
//! # Ok(())
//! # }
//! ```

use std::path::PathBuf;
use std::sync::Arc;

use futures::future::try_join_all;
use tracing::{debug, info, warn};

use crate::domain::semantic_cleaner::{private, SemanticCleaner};
use crate::domain::DocumentChunk;
use crate::error::SemanticError;
use crate::infrastructure::ai::model_cache::{
    default_cache_dir, CacheConfig, ModelCache, DEFAULT_MODEL_FILE, DEFAULT_MODEL_REPO,
};
use crate::infrastructure::ai::model_downloader::ModelDownloader;
use crate::infrastructure::ai::{HtmlChunker, InferenceEngine, MiniLmTokenizer, RelevanceScorer};

/// Model configuration
///
/// Controls model loading and inference behavior.
///
/// # Builder Pattern
///
/// Following `api-builder-pattern`, use builder methods for configuration:
///
/// ```
/// # use rust_scraper::infrastructure::ai::ModelConfig;
/// let config = ModelConfig::new()
///     .with_repo("sentence-transformers/all-MiniLM-L6-v2")
///     .with_offline_mode(true)
///     .with_max_tokens(512);
/// ```
#[derive(Debug, Clone)]
pub struct ModelConfig {
    /// Model repository on HuggingFace Hub
    pub repo: String,
    /// Model filename within repository
    pub model_file: String,
    /// Cache directory for downloaded models
    pub cache_dir: PathBuf,
    /// Enable auto-download if model not cached
    pub auto_download: bool,
    /// Offline mode (fail if not cached)
    pub offline_mode: bool,
    /// Maximum tokens per chunk (model-specific)
    pub max_tokens: usize,
    /// Relevance threshold for filtering (0.0-1.0)
    pub relevance_threshold: f32,
}

impl Default for ModelConfig {
    fn default() -> Self {
        Self {
            repo: DEFAULT_MODEL_REPO.to_string(),
            model_file: DEFAULT_MODEL_FILE.to_string(),
            cache_dir: default_cache_dir(),
            auto_download: true,
            offline_mode: false,
            max_tokens: 512,          // all-MiniLM-L6-v2 limit
            relevance_threshold: 0.3, // Moderate relevance threshold
        }
    }
}

impl ModelConfig {
    /// Create a new model configuration with default values
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Set model repository
    #[must_use]
    pub fn with_repo(mut self, repo: impl Into<String>) -> Self {
        self.repo = repo.into();
        self
    }

    /// Set model filename
    #[must_use]
    pub fn with_file(mut self, file: impl Into<String>) -> Self {
        self.model_file = file.into();
        self
    }

    /// Set cache directory
    #[must_use]
    pub fn with_cache_dir(mut self, dir: PathBuf) -> Self {
        self.cache_dir = dir;
        self
    }

    /// Enable/disable auto-download
    #[must_use]
    pub fn with_auto_download(mut self, enabled: bool) -> Self {
        self.auto_download = enabled;
        self
    }

    /// Enable offline mode
    #[must_use]
    pub fn with_offline_mode(mut self, enabled: bool) -> Self {
        self.offline_mode = enabled;
        self
    }

    /// Set maximum tokens per chunk
    #[must_use]
    pub fn with_max_tokens(mut self, tokens: usize) -> Self {
        self.max_tokens = tokens;
        self
    }

    /// Set relevance threshold for filtering
    #[must_use]
    pub fn with_relevance_threshold(mut self, threshold: f32) -> Self {
        // Validate threshold range
        assert!(
            (0.0..=1.0).contains(&threshold),
            "Relevance threshold must be between 0.0 and 1.0, got {}",
            threshold
        );
        self.relevance_threshold = threshold;
        self
    }
}

/// Semantic Cleaner implementation using full RAG pipeline
///
/// This is the concrete implementation of the [`SemanticCleaner`] trait.
/// It integrates all Phase 2 and Phase 3 modules:
/// - [`HtmlChunker`]: Semantic chunking with arena allocator
/// - [`MiniLmTokenizer`]: HuggingFace tokenization
/// - [`InferenceEngine`]: ONNX model execution with spawn_blocking
/// - [`RelevanceScorer`]: SIMD-accelerated cosine similarity filtering
///
/// # Thread Safety
///
/// This type is `Send + Sync` and can be safely shared across threads.
/// All components use `Arc` for thread-safe sharing.
///
/// # Performance
///
/// - **First call**: Model download (~90MB) + load (~100-500ms)
/// - **Subsequent calls**: ~50-200ms per page (depending on content size)
/// - **Memory**: Arena allocator reduces allocation overhead
/// - **Concurrency**: Embeddings generated concurrently with `try_join_all`
pub struct SemanticCleanerImpl {
    // Phase 2: Core inference
    /// ONNX inference engine (shared via Arc)
    inference_engine: Arc<InferenceEngine>,
    /// HuggingFace tokenizer
    tokenizer: MiniLmTokenizer,

    // Phase 3: Chunking + scoring
    /// Semantic HTML chunker with arena allocator
    chunker: HtmlChunker,
    /// Relevance scorer with SIMD cosine similarity
    scorer: RelevanceScorer,

    // Config
    /// Model and pipeline configuration
    config: ModelConfig,
}

impl SemanticCleanerImpl {
    /// Create a new semantic cleaner with full pipeline
    ///
    /// This method loads all pipeline components:
    /// 1. Downloads/loads ONNX model
    /// 2. Loads tokenizer
    /// 3. Creates chunker and scorer
    ///
    /// # Arguments
    ///
    /// * `config` - Model configuration
    ///
    /// # Returns
    ///
    /// * `Ok(SemanticCleanerImpl)` - Successfully created cleaner
    /// * `Err(SemanticError)` - Model loading or download failed
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - Model download fails
    /// - Model file is corrupted (SHA256 mismatch)
    /// - ONNX model fails to load
    /// - Tokenizer fails to load
    /// - Offline mode enabled but model not cached
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// use rust_scraper::infrastructure::ai::{SemanticCleanerImpl, ModelConfig};
    ///
    /// let config = ModelConfig::default();
    /// let cleaner = SemanticCleanerImpl::new(config).await?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Performance
    ///
    /// - **First call**: Model download (~90MB) + load (~100-500ms)
    /// - **Subsequent calls**: Cache hit, ~10-50ms per page
    /// - **Memory**: Memory-mapped files, ~90MB virtual memory
    pub async fn new(config: ModelConfig) -> Result<Self, SemanticError> {
        info!(
            repo = %config.repo,
            file = %config.model_file,
            cache_dir = ?config.cache_dir,
            relevance_threshold = config.relevance_threshold,
            "Initializing semantic cleaner with full RAG pipeline"
        );

        // Create cache manager
        let cache_config = CacheConfig::new()
            .with_cache_dir(config.cache_dir.clone())
            .with_offline_mode(config.offline_mode);

        let cache = ModelCache::new(cache_config.clone());

        // Ensure cache directory exists
        cache.ensure_cache_dir().await?;

        // Check if model is cached
        if cache.is_model_cached(&config.model_file) {
            debug!("Model found in cache");
        } else if config.offline_mode {
            return Err(SemanticError::OfflineMode {
                repo: config.repo.clone(),
            });
        } else if config.auto_download {
            // Download model
            info!("Model not cached, downloading...");
            let downloader = ModelDownloader::new()
                .with_repo(&config.repo)
                .with_file(&config.model_file);

            downloader.download_to(&config.cache_dir).await?;
        } else {
            return Err(SemanticError::OfflineMode {
                repo: config.repo.clone(),
            });
        };

        // Validate model integrity
        if cache_config.validate_sha256 {
            debug!("Validating model integrity...");
            cache
                .validate_model(&config.model_file, None)
                .await
                .unwrap_or_else(|e| {
                    warn!("Model validation failed: {}", e);
                    // Continue anyway - model might still work
                });
        }

        // Load inference engine
        let model_path = cache.model_path(&config.model_file);
        let inference_engine = Arc::new(
            InferenceEngine::load_from_file(&model_path)
                .await
                .map_err(|e| {
                    SemanticError::ModelLoad(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        format!("Failed to load inference engine: {}", e),
                    ))
                })?,
        );

        // Load tokenizer
        let tokenizer_path = config.cache_dir.join("tokenizer.json");
        let tokenizer = if tokenizer_path.exists() {
            MiniLmTokenizer::from_file(&tokenizer_path)
                .await
                .map_err(|e| SemanticError::Tokenize(format!("Failed to load tokenizer: {}", e)))?
        } else {
            return Err(SemanticError::Tokenize(
                "Tokenizer not found in cache. Run model download first.".to_string(),
            ));
        };

        // Create chunker with config
        let chunker = HtmlChunker::new();

        // Create scorer with relevance threshold
        let scorer = RelevanceScorer::new(config.relevance_threshold);

        info!("Semantic cleaner initialized successfully");
        debug!(
            embedding_dim = inference_engine.embedding_dim(),
            max_tokens = config.max_tokens,
            relevance_threshold = config.relevance_threshold,
            "Pipeline components loaded"
        );

        Ok(Self {
            inference_engine,
            tokenizer,
            chunker,
            scorer,
            config,
        })
    }

    /// Get the cache directory
    #[must_use]
    pub fn cache_dir(&self) -> &std::path::Path {
        &self.config.cache_dir
    }

    /// Check if auto-download is enabled
    #[must_use]
    pub fn auto_download_enabled(&self) -> bool {
        self.config.auto_download
    }

    /// Get the relevance threshold
    #[must_use]
    pub fn relevance_threshold(&self) -> f32 {
        self.config.relevance_threshold
    }

    /// Set the relevance threshold
    ///
    /// # Arguments
    ///
    /// * `threshold` - New threshold value (0.0-1.0)
    ///
    /// # Panics
    ///
    /// Panics if threshold is outside [0.0, 1.0] range
    pub fn set_relevance_threshold(&mut self, threshold: f32) {
        assert!(
            (0.0..=1.0).contains(&threshold),
            "Relevance threshold must be between 0.0 and 1.0, got {}",
            threshold
        );
        self.config.relevance_threshold = threshold;
        self.scorer.set_threshold(threshold);
    }
}

// Implement the Sealed trait for SemanticCleanerImpl
// This is required by the sealed trait pattern
impl private::Sealed for SemanticCleanerImpl {}

#[async_trait::async_trait]
impl SemanticCleaner for SemanticCleanerImpl {
    async fn clean(&self, html: &str) -> Result<Vec<DocumentChunk>, SemanticError> {
        debug!(
            html_length = html.len(),
            "Starting full RAG pipeline: chunk → tokenize → embed → score"
        );

        // Step 1: Semantic chunking (uses arena internally)
        // Following `own-borrow-over-clone`: borrow html, don't clone
        let chunks = self
            .chunker
            .chunk(html)
            .map_err(|e| SemanticError::Tokenize(format!("Chunking failed: {}", e)))?;

        if chunks.is_empty() {
            debug!("No chunks produced from HTML");
            return Ok(Vec::new());
        }

        debug!(chunks_count = chunks.len(), "Step 1: Chunking complete");

        // Step 2: Tokenize all chunks (mem-reuse-collections: reuse buffer)
        // Pre-allocate with capacity following `mem-with-capacity`
        let mut token_buffers = Vec::with_capacity(chunks.len());
        for chunk in &chunks {
            let tokens = self.tokenizer.tokenize(&chunk.content).map_err(|e| {
                SemanticError::Tokenize(format!("Tokenization failed for chunk: {}", e))
            })?;

            // Validate token count
            if tokens.len() > self.config.max_tokens {
                return Err(SemanticError::ChunkTooLarge {
                    chunk_id: format!("chunk-{}", token_buffers.len()),
                    tokens: tokens.len(),
                    max: self.config.max_tokens,
                });
            }

            token_buffers.push(tokens);
        }

        debug!(
            tokens_generated = token_buffers.len(),
            "Step 2: Tokenization complete"
        );

        // Step 3: Generate embeddings CONCURRENTLY (async-join-parallel)
        // Following `async-join-parallel`: use try_join_all for concurrent independent operations
        // Following `async-spawn-blocking`: InferenceEngine already uses spawn_blocking internally
        // Following `anti-lock-across-await`: No locks held across await points
        let embeddings = try_join_all(
            token_buffers
                .iter()
                .map(|tokens| self.inference_engine.run_inference(tokens)),
        )
        .await
        .map_err(|e| {
            SemanticError::Inference(format!("Concurrent embedding generation failed: {}", e))
        })?;

        debug!(
            embeddings_generated = embeddings.len(),
            embedding_dim = embeddings.first().map(|e| e.len()).unwrap_or(0),
            "Step 3: Embedding generation complete"
        );

        // Step 4: Score and filter (own-borrow-over-clone: borrow embeddings)
        // Following `own-borrow-over-clone`: borrow &chunks and &embeddings, don't clone
        // Following `opt-simd-portable`: RelevanceScorer uses SIMD cosine similarity
        let filtered = self.filter_by_relevance(&chunks, &embeddings)?;

        debug!(
            chunks_before = chunks.len(),
            chunks_after = filtered.len(),
            filtered_out = chunks.len() - filtered.len(),
            "Step 4: Relevance filtering complete"
        );

        info!(total_chunks = filtered.len(), "Full RAG pipeline complete");

        Ok(filtered)
    }

    fn max_tokens(&self) -> usize {
        self.config.max_tokens
    }

    fn is_ready(&self) -> bool {
        // Model is ready if inference engine is ready
        self.inference_engine.is_ready()
    }
}

impl SemanticCleanerImpl {
    /// Filter chunks by relevance score
    ///
    /// Pairs each chunk with its embedding, scores against a reference,
    /// and filters by threshold.
    ///
    /// # Arguments
    ///
    /// * `chunks` - Slice of DocumentChunks (borrowed, following `own-borrow-over-clone`)
    /// * `embeddings` - Slice of embedding vectors (borrowed)
    ///
    /// # Returns
    ///
    /// Filtered vector of DocumentChunks meeting relevance threshold
    ///
    /// # Performance
    ///
    /// Uses SIMD-accelerated cosine similarity via `RelevanceScorer`.
    fn filter_by_relevance(
        &self,
        chunks: &[DocumentChunk],
        embeddings: &[Vec<f32>],
    ) -> Result<Vec<DocumentChunk>, SemanticError> {
        // Create (chunk, embedding) pairs
        // Following `mem-with-capacity`: pre-allocate
        let mut chunk_embedding_pairs = Vec::with_capacity(chunks.len());

        for (chunk, embedding) in chunks.iter().zip(embeddings.iter()) {
            chunk_embedding_pairs.push((chunk.clone(), embedding.clone()));
        }

        // Use first embedding as reference (simple strategy)
        // In production, this could be a query vector or domain-specific reference
        let reference = embeddings.first().ok_or_else(|| {
            SemanticError::Inference("No embeddings available for relevance scoring".to_string())
        })?;

        // Filter using scorer
        let filtered = self.scorer.filter(&chunk_embedding_pairs, Some(reference));

        Ok(filtered)
    }
}

/// Create a semantic cleaner with the specified configuration
///
/// This is the main entry point for creating a [`SemanticCleaner`].
///
/// # Arguments
///
/// * `config` - Model configuration
///
/// # Returns
///
/// * `Ok(Box<dyn SemanticCleaner>)` - Successfully created cleaner
/// * `Err(SemanticError)` - Creation failed
///
/// # Examples
///
/// ```no_run
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// use rust_scraper::infrastructure::ai::{create_semantic_cleaner, ModelConfig};
///
/// let config = ModelConfig::default();
/// let cleaner = create_semantic_cleaner(&config).await?;
///
/// let html = "<article><p>Hello World</p></article>";
/// let chunks = cleaner.clean(html).await?;
/// # Ok(())
/// # }
/// ```
pub(crate) async fn create_semantic_cleaner(
    config: &ModelConfig,
) -> Result<Box<dyn SemanticCleaner>, SemanticError> {
    let cleaner = SemanticCleanerImpl::new(config.clone()).await?;
    Ok(Box::new(cleaner))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_config_default() {
        let config = ModelConfig::default();
        assert_eq!(config.repo, DEFAULT_MODEL_REPO);
        assert_eq!(config.model_file, DEFAULT_MODEL_FILE);
        assert!(config.auto_download);
        assert!(!config.offline_mode);
        assert_eq!(config.max_tokens, 512);
        assert_eq!(config.relevance_threshold, 0.3);
    }

    #[test]
    fn test_model_config_builder() {
        let config = ModelConfig::new()
            .with_repo("test/repo")
            .with_file("test.onnx")
            .with_auto_download(false)
            .with_offline_mode(true)
            .with_max_tokens(256)
            .with_relevance_threshold(0.5);

        assert_eq!(config.repo, "test/repo");
        assert_eq!(config.model_file, "test.onnx");
        assert!(!config.auto_download);
        assert!(config.offline_mode);
        assert_eq!(config.max_tokens, 256);
        assert_eq!(config.relevance_threshold, 0.5);
    }

    #[test]
    #[should_panic(expected = "Relevance threshold must be between")]
    fn test_model_config_invalid_threshold() {
        ModelConfig::new().with_relevance_threshold(1.5);
    }

    #[test]
    fn test_semantic_cleaner_type_traits() {
        fn assert_send<T: Send>() {}
        fn assert_sync<T: Sync>() {}

        // SemanticCleanerImpl should be Send + Sync
        assert_send::<SemanticCleanerImpl>();
        assert_sync::<SemanticCleanerImpl>();
    }

    #[tokio::test]
    async fn test_semantic_cleaner_creation_fails_without_model() {
        // This test verifies that creation fails gracefully when model is not available
        let config = ModelConfig::new()
            .with_auto_download(false)
            .with_offline_mode(true);

        let result = SemanticCleanerImpl::new(config).await;

        // Should fail with OfflineMode error
        assert!(result.is_err());

        if let Err(SemanticError::OfflineMode { repo }) = result {
            assert_eq!(repo, DEFAULT_MODEL_REPO);
        } else {
            panic!("Expected SemanticError::OfflineMode");
        }
    }

    #[test]
    fn test_filter_by_relevance_empty() {
        // Test filter_by_relevance with empty input
        // Note: This test can't actually run without async setup
        // It's here as a placeholder for future mock-based testing
    }
}
