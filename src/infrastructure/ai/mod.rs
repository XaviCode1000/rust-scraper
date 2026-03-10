//! AI module — Model download, caching, inference, and semantic chunking
//!
//! This module provides AI-powered semantic cleaning capabilities:
//! - Automatic model download from HuggingFace Hub
//! - Cache management with SHA256 validation
//! - Memory-mapped model loading (zero-copy for HDD optimization)
//! - ONNX inference for embedding generation
//! - Semantic chunking with SIMD-accelerated cosine similarity
//!
//! # Architecture
//!
//! Following Clean Architecture, this module implements the [`SemanticCleaner`](crate::domain::semantic_cleaner::SemanticCleaner)
//! trait defined in the domain layer.
//!
//! ```text
//! domain::semantic_cleaner::SemanticCleaner (trait)
//!     ↑ (implemented by)
//! infrastructure::ai::SemanticCleanerImpl (concrete implementation)
//! ```
//!
//! # Features
//!
//! This module is feature-gated behind the `ai` feature flag:
//!
//! ```toml
//! [dependencies]
//! rust_scraper = { version = "1.0", features = ["ai"] }
//! ```
//!
//! # Model Information
//!
//! - **Model**: `sentence-transformers/all-MiniLM-L6-v2`
//! - **Format**: ONNX (optimized for inference)
//! - **Size**: ~90MB
//! - **Max Tokens**: 512 per chunk
//! - **Cache Location**: `~/.cache/rust-scraper/ai_models/`
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
//! let html = "<article><p>Hello World</p></article>";
//! let chunks = cleaner.clean(html).await?;
//!
//! println!("Generated {} chunks", chunks.len());
//! # Ok(())
//! # }
//! ```

// Core AI infrastructure (Modules 1-2)
#[cfg(feature = "ai")]
pub mod model_cache;

#[cfg(feature = "ai")]
pub mod model_downloader;

#[cfg(feature = "ai")]
pub mod semantic_cleaner_impl;

#[cfg(feature = "ai")]
pub mod inference_engine;

#[cfg(feature = "ai")]
pub mod tokenizer;

// Semantic Chunking (Modules 3-4)
#[cfg(feature = "ai")]
pub mod chunk_id;

#[cfg(feature = "ai")]
pub mod sentence;

#[cfg(feature = "ai")]
pub mod chunker;

#[cfg(feature = "ai")]
pub mod embedding_ops;

#[cfg(feature = "ai")]
pub mod relevance_scorer;

#[cfg(feature = "ai")]
pub mod threshold_config;

// Re-exports for convenience (Modules 1-2)
#[cfg(feature = "ai")]
pub use model_cache::{CacheConfig, ModelCache};

#[cfg(feature = "ai")]
pub use model_downloader::{DownloadProgress, ModelDownloader};

#[cfg(feature = "ai")]
pub use semantic_cleaner_impl::{ModelConfig, SemanticCleanerImpl};

#[cfg(feature = "ai")]
pub use inference_engine::InferenceEngine;

#[cfg(feature = "ai")]
pub use tokenizer::{MiniLmTokenizer, TokenBatch, DEFAULT_MAX_LENGTH};

// Re-exports for Semantic Chunking (Modules 3-4)
#[cfg(feature = "ai")]
pub use chunk_id::ChunkId;

#[cfg(feature = "ai")]
pub use sentence::SentenceSplitter;

#[cfg(feature = "ai")]
pub use chunker::HtmlChunker;

#[cfg(feature = "ai")]
pub use relevance_scorer::RelevanceScorer;

#[cfg(feature = "ai")]
pub use threshold_config::ThresholdConfig;
