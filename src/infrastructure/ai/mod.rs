//! AI module — Model download, caching, and inference
//!
//! This module provides AI-powered semantic cleaning capabilities:
//! - Automatic model download from HuggingFace Hub
//! - Cache management with SHA256 validation
//! - Memory-mapped model loading (zero-copy for HDD optimization)
//! - ONNX inference for embedding generation
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

#[cfg(feature = "ai")]
pub mod model_cache;

#[cfg(feature = "ai")]
pub mod model_downloader;

#[cfg(feature = "ai")]
pub mod semantic_cleaner_impl;

// Re-exports for convenience
#[cfg(feature = "ai")]
pub use model_cache::{CacheConfig, ModelCache};

#[cfg(feature = "ai")]
pub use model_downloader::{DownloadProgress, ModelDownloader};

#[cfg(feature = "ai")]
pub use semantic_cleaner_impl::{ModelConfig, SemanticCleanerImpl};
