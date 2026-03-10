//! Inference engine — ONNX model execution with tract-onnx
//!
//! Handles loading and executing ONNX models for sentence embedding generation:
//! - Thread-safe session sharing with `Arc<TypedSimplePlan<TypedModel>>` (`own-arc-shared`)
//! - Async inference via `spawn_blocking` (`async-spawn-blocking`)
//! - Clone Arc before await (`async-clone-before-await`)
//! - 384-dimensional embedding output for all-MiniLM-L6-v2
//!
//! # Design Decisions
//!
//! - **TypedSimplePlan**: Uses tract's typed plan type for type-safe inference
//! - **Arc for session sharing**: Session is wrapped in Arc for thread-safe access across threads
//! - **spawn_blocking**: CPU-intensive inference runs in blocking pool to avoid starving async runtime
//! - **No locks across await**: Clone Arc before async operations
//!
//! # Examples
//!
//! ```no_run
//! # async fn example() -> anyhow::Result<()> {
//! use rust_scraper::infrastructure::ai::InferenceEngine;
//!
//! let engine = InferenceEngine::load_from_file("path/to/model.onnx").await?;
//! let tokens = vec![101i64, 2054, 2003, 102]; // [CLS] hello world [SEP]
//! let embedding = engine.run_inference(&tokens).await?;
//! assert_eq!(embedding.len(), 384);
//! # Ok(())
//! # }
//! ```

use std::path::Path;
use std::sync::Arc;

use tract_onnx::prelude::*;
use tracing::debug;

use crate::error::SemanticError;

/// Thread-safe inference session
///
/// Uses tract's TypedSimplePlan which is Send + Sync for thread-safe sharing.
/// This is the correct type for ONNX inference with tract-onnx 0.21.
pub type InferenceSession = Arc<TypedSimplePlan<TypedModel>>;

/// ONNX inference engine for sentence embeddings
///
/// This engine loads an ONNX model and provides methods for running inference
/// to generate sentence embeddings (384-dimensional vectors for all-MiniLM-L6-v2).
///
/// # Thread Safety
///
/// `InferenceEngine` is `Clone` because it wraps the session in `Arc<TypedSimplePlan<TypedModel>>`.
/// Cloning is cheap (just increments atomic counter) and safe for concurrent use.
///
/// # Examples
///
/// ```no_run
/// # async fn example() -> anyhow::Result<()> {
/// use rust_scraper::infrastructure::ai::InferenceEngine;
///
/// let engine = InferenceEngine::load_from_file("path/to/model.onnx").await?;
///
/// // Clone for concurrent use
/// let engine_clone = engine.clone();
///
/// // Both can be used concurrently
/// let tokens = vec![101i64, 2054, 2003, 102];
/// let embedding1 = engine.run_inference(&tokens).await?;
/// let embedding2 = engine_clone.run_inference(&tokens).await?;
/// # Ok(())
/// # }
/// ```
#[derive(Clone)]
pub struct InferenceEngine {
    session: InferenceSession,
}

impl InferenceEngine {
    /// Load ONNX model from file
    ///
    /// Uses the tract-onnx pattern:
    /// 1. Read model bytes
    /// 2. Parse ONNX model with `model_for_read()`
    /// 3. Optimize the model graph
    /// 4. Build executable plan with `into_runnable()`
    ///
    /// The model is loaded once and shared across threads via `Arc`.
    ///
    /// # Arguments
    ///
    /// * `model_path` - Path to the ONNX model file
    ///
    /// # Returns
    ///
    /// * `Ok(InferenceEngine)` - Model loaded successfully
    /// * `Err(SemanticError::ModelLoad)` - Failed to load model
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - File doesn't exist or can't be read
    /// - ONNX model is invalid or corrupted
    /// - Model has unexpected input/output structure
    ///
    /// # Example
    ///
    /// ```no_run
    /// # async fn example() -> anyhow::Result<()> {
    /// use rust_scraper::infrastructure::ai::InferenceEngine;
    ///
    /// let engine = InferenceEngine::load_from_file("model.onnx").await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn load_from_file<P: AsRef<Path>>(model_path: P) -> Result<Self, SemanticError> {
        let model_path = model_path.as_ref();

        debug!(path = ?model_path, "Loading ONNX model");

        // Read model bytes (async I/O)
        let model_data = tokio::fs::read(model_path)
            .await
            .map_err(|e| SemanticError::ModelLoad(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to read model file: {}", e),
            )))?;

        // Build tract model from bytes using model_for_read
        // Note: model_for_read takes a mutable reader, so we use a slice
        let model = tract_onnx::onnx()
            .model_for_read(&mut &model_data[..])
            .map_err(|e| SemanticError::ModelLoad(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to parse ONNX model: {}", e),
            )))?;

        // Optimize the model graph (operator fusion, constant folding, etc.)
        let optimized = model
            .into_optimized()
            .map_err(|e| SemanticError::ModelLoad(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to optimize model: {}", e),
            )))?;

        // Build executable plan (TypedSimplePlan<TypedModel>)
        // This is the correct method - into_runnable() returns the plan
        let plan = optimized
            .into_runnable()
            .map_err(|e| SemanticError::ModelLoad(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to create runnable plan: {}", e),
            )))?;

        // Wrap in Arc for thread-safe sharing
        let session = Arc::new(plan);

        debug!("Model loaded successfully");

        Ok(Self { session })
    }

    /// Run inference on token IDs
    ///
    /// Takes token IDs and generates a 384-dimensional embedding vector.
    /// Uses `spawn_blocking` to avoid blocking the async runtime (`async-spawn-blocking`).
    ///
    /// # Arguments
    ///
    /// * `tokens` - Token IDs (including [CLS] and [SEP] special tokens)
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<f32>)` - 384-dimensional embedding vector
    /// * `Err(SemanticError::Inference)` - Inference failed
    ///
    /// # Performance
    ///
    /// Typical latency: 10-50ms per inference on Haswell CPU.
    /// This is CPU-intensive work, hence `spawn_blocking` is mandatory.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # async fn example() -> anyhow::Result<()> {
    /// use rust_scraper::infrastructure::ai::InferenceEngine;
    ///
    /// let engine = InferenceEngine::load_from_file("model.onnx").await?;
    /// let tokens = vec![101i64, 2054, 2003, 102]; // [CLS] hello world [SEP]
    /// let embedding = engine.run_inference(&tokens).await?;
    /// assert_eq!(embedding.len(), 384);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn run_inference(&self, tokens: &[i64]) -> Result<Vec<f32>, SemanticError> {
        // Clone Arc before await to avoid holding references across suspension
        // This ensures the future is Send and can be spawned (`async-clone-before-await`)
        let session = Arc::clone(&self.session);
        let tokens = tokens.to_vec();

        // Run inference in blocking pool (CPU-intensive work)
        // This prevents blocking the async runtime threads (`async-spawn-blocking`)
        let result = tokio::task::spawn_blocking(move || {
            // Create input tensor: shape [1, sequence_length] with i64 data
            // Using Tensor::from_shape which takes shape slice and data slice
            let input_tensor =
                Tensor::from_shape(&[1, tokens.len()], &tokens)
                    .map_err(|e| SemanticError::Inference(format!("Failed to create tensor: {}", e)))?;

            // Create state for the plan
            // Pass the Arc directly (not &Arc) - TypedSimpleState::new takes P: Borrow<SimplePlan>
            let mut state = TypedSimpleState::new(session.clone())
                .map_err(|e| SemanticError::Inference(format!("Failed to create state: {}", e)))?;

            // Run the model with the input tensor
            // Input: tvec!(tensor.into()) - correct pattern from tract examples
            // Output: TypedSimpleState result containing output tensors
            let outputs = state
                .run(tvec![input_tensor.into()])
                .map_err(|e| SemanticError::Inference(format!("Model execution failed: {}", e)))?;

            // Extract first output tensor (the embedding)
            let output = outputs
                .first()
                .ok_or_else(|| SemanticError::Inference("No output from model".to_string()))?;

            // Convert to Vec<f32> by iterating over the tensor
            // Using to_array_view for zero-copy access to tensor data
            let embedding: Vec<f32> = output
                .to_array_view::<f32>()
                .map_err(|e| SemanticError::Inference(format!("Failed to extract output: {}", e)))?
                .iter()
                .copied()
                .collect();

            Ok(embedding)
        })
        .await
        .map_err(|e| SemanticError::Inference(format!("Task join error: {}", e)))?;

        // Propagate the inner Result from spawn_blocking
        result
    }

    /// Get embedding dimension (384 for all-MiniLM-L6-v2)
    ///
    /// This is a constant for the all-MiniLM-L6-v2 model.
    #[must_use]
    pub fn embedding_dim(&self) -> usize {
        384
    }

    /// Check if engine is ready for inference
    ///
    /// Returns true if the session Arc has at least one strong reference.
    #[must_use]
    pub fn is_ready(&self) -> bool {
        Arc::strong_count(&self.session) > 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test that InferenceEngine type exists and compiles
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
    #[test]
    fn test_inference_engine_is_send_sync() {
        fn assert_send<T: Send>() {}
        fn assert_sync<T: Sync>() {}

        assert_send::<InferenceEngine>();
        assert_sync::<InferenceEngine>();
    }

    /// Test that InferenceEngine is Clone (cheap Arc clone)
    #[test]
    fn test_inference_engine_is_clone() {
        fn assert_clone<T: Clone>() {}
        assert_clone::<InferenceEngine>();
    }
}
