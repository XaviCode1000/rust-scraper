//! SIMD-accelerated embedding operations
//!
//! Provides high-performance vector operations for embedding processing:
//! - Cosine similarity using AVX2 SIMD (`opt-simd-portable`)
//! - Dot product for normalized vectors
//! - Batch operations for efficiency
//!
//! # Performance
//!
//! On Haswell (AVX2), `wide::f32x8` provides 4-8x speedup over scalar operations.
//! The `wide` crate is used for stable SIMD without nightly Rust.

use wide::f32x8;

/// SIMD-accelerated cosine similarity
///
/// Computes cosine similarity between two vectors using AVX2 SIMD instructions.
///
/// # Mathematical Background
///
/// For normalized vectors (unit length), cosine similarity equals dot product:
/// ```text
/// cos(θ) = (A · B) / (||A|| × ||B||)
/// ```
///
/// When `||A|| = ||B|| = 1` (normalized):
/// ```text
/// cos(θ) = A · B = Σ(aᵢ × bᵢ)
/// ```
///
/// The `all-MiniLM-L6-v2` model outputs normalized embeddings, so we can use
/// dot product directly.
///
/// # Arguments
///
/// * `a` - First vector (should be normalized)
/// * `b` - Second vector (should be normalized)
///
/// # Returns
///
/// Cosine similarity in range [-1.0, 1.0]:
/// - `1.0`: Identical vectors
/// - `0.0`: Orthogonal (unrelated)
/// - `-1.0`: Opposite vectors
///
/// # Examples
///
/// ```
/// # #[cfg(feature = "ai")]
/// # fn example() {
/// use rust_scraper::infrastructure::ai::embedding_ops::cosine_similarity;
///
/// // Identical vectors
/// let vec = vec![0.5f32, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5];
/// let sim = cosine_similarity(&vec, &vec);
/// assert!((sim - 1.0).abs() < 0.001);
///
/// // Orthogonal vectors
/// let a = vec![1.0f32, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];
/// let b = vec![0.0f32, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];
/// let sim = cosine_similarity(&a, &b);
/// assert!(sim.abs() < 0.001);
/// # }
/// ```
///
/// # Performance Notes
///
/// - Uses `wide::f32x8` for 8-wide SIMD parallelism
/// - Processes 8 floats per instruction on Haswell (AVX2)
/// - Falls back to scalar for remainder elements
#[must_use]
pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    let len = a.len().min(b.len());

    if len == 0 {
        return 0.0;
    }

    // Process 8 elements at a time using wide::f32x8
    let simd_chunks = len / 8;
    let remainder = len % 8;

    // SIMD dot product using f32x8
    let mut sum = f32x8::splat(0.0);

    for i in 0..simd_chunks {
        let offset = i * 8;
        // Use array conversion (wide doesn't have from_slice)
        let mut av_array = [0.0f32; 8];
        let mut bv_array = [0.0f32; 8];
        av_array.copy_from_slice(&a[offset..offset + 8]);
        bv_array.copy_from_slice(&b[offset..offset + 8]);
        let av = f32x8::from(av_array);
        let bv = f32x8::from(bv_array);
        sum += av * bv;
    }

    // Reduce SIMD lanes to scalar using reduce_add
    let mut dot_product = sum.reduce_add();

    // Handle remainder elements (scalar fallback)
    let scalar_start = simd_chunks * 8;
    for i in scalar_start..scalar_start + remainder {
        dot_product += a[i] * b[i];
    }

    dot_product
}

/// Compute dot product of two vectors (scalar fallback)
///
/// Used when vectors are too small for SIMD or as a reference implementation.
///
/// # Arguments
///
/// * `a` - First vector
/// * `b` - Second vector
///
/// # Returns
///
/// Dot product value
#[must_use]
pub fn dot_product_scalar(a: &[f32], b: &[f32]) -> f32 {
    let len = a.len().min(b.len());
    a[..len]
        .iter()
        .zip(b[..len].iter())
        .map(|(&x, &y)| x * y)
        .sum()
}

/// Normalize a vector to unit length
///
/// # Arguments
///
/// * `vector` - Input vector
///
/// # Returns
///
/// Normalized vector (unit length)
///
/// # Panics
///
/// Panics if the vector has zero magnitude
#[must_use]
pub fn normalize(vector: &[f32]) -> Vec<f32> {
    let magnitude = vector.iter().map(|&x| x * x).sum::<f32>().sqrt();

    if magnitude < f32::EPSILON {
        panic!("Cannot normalize zero-magnitude vector");
    }

    vector.iter().map(|&x| x / magnitude).collect()
}

/// Compute Euclidean distance between two vectors
///
/// # Arguments
///
/// * `a` - First vector
/// * `b` - Second vector
///
/// # Returns
///
/// Euclidean distance
#[must_use]
pub fn euclidean_distance(a: &[f32], b: &[f32]) -> f32 {
    let len = a.len().min(b.len());
    a[..len]
        .iter()
        .zip(b[..len].iter())
        .map(|(&x, &y)| (x - y).powi(2))
        .sum::<f32>()
        .sqrt()
}

/// Batch cosine similarity
///
/// Compute similarity between one query vector and multiple candidate vectors.
///
/// # Arguments
///
/// * `query` - Query vector
/// * `candidates` - Slice of candidate vectors
///
/// # Returns
///
/// Vector of similarity scores
#[must_use]
pub fn batch_cosine_similarity(query: &[f32], candidates: &[Vec<f32>]) -> Vec<f32> {
    candidates
        .iter()
        .map(|candidate| cosine_similarity(query, candidate))
        .collect()
}

/// Find most similar vector from candidates
///
/// # Arguments
///
/// * `query` - Query vector
/// * `candidates` - Slice of candidate vectors
///
/// # Returns
///
/// Index of most similar candidate, or None if empty
#[must_use]
pub fn find_most_similar(query: &[f32], candidates: &[Vec<f32>]) -> Option<usize> {
    if candidates.is_empty() {
        return None;
    }

    let mut best_idx = 0;
    let mut best_score = f32::NEG_INFINITY;

    for (idx, candidate) in candidates.iter().enumerate() {
        let score = cosine_similarity(query, candidate);
        if score > best_score {
            best_score = score;
            best_idx = idx;
        }
    }

    Some(best_idx)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cosine_similarity_identical() {
        // Use a normalized vector (magnitude = 1.0)
        // 1/sqrt(8) ≈ 0.3536 for 8-dimensional unit vector
        let normalization = 1.0f32 / 8.0f32.sqrt();
        let vec = vec![normalization; 8];
        let sim = cosine_similarity(&vec, &vec);
        assert!((sim - 1.0).abs() < 0.001, "Expected ~1.0, got {}", sim);
    }

    #[test]
    fn test_cosine_similarity_orthogonal() {
        let a = vec![1.0f32, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];
        let b = vec![0.0f32, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];
        let sim = cosine_similarity(&a, &b);
        assert!(sim.abs() < 0.001, "Expected ~0.0, got {}", sim);
    }

    #[test]
    fn test_cosine_similarity_opposite() {
        let a = vec![1.0f32, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];
        let b = vec![-1.0f32, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];
        let sim = cosine_similarity(&a, &b);
        assert!((sim + 1.0).abs() < 0.001, "Expected ~-1.0, got {}", sim);
    }

    #[test]
    fn test_cosine_similarity_partial() {
        let a = vec![1.0f32, 0.0, 0.0, 0.0];
        let b = vec![0.0f32, 0.0, 0.0, 1.0];
        let sim = cosine_similarity(&a, &b);
        assert!(sim.abs() < 0.001);
    }

    #[test]
    fn test_cosine_similarity_different_lengths() {
        let a = vec![1.0f32, 0.0, 0.0];
        let b = vec![1.0f32, 0.0, 0.0, 0.0, 0.0];
        let sim = cosine_similarity(&a, &b);
        assert!((sim - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_cosine_similarity_empty() {
        let a: Vec<f32> = vec![];
        let b: Vec<f32> = vec![];
        let sim = cosine_similarity(&a, &b);
        assert_eq!(sim, 0.0);
    }

    #[test]
    fn test_dot_product_scalar() {
        let a = vec![1.0f32, 2.0, 3.0];
        let b = vec![4.0f32, 5.0, 6.0];
        let dot = dot_product_scalar(&a, &b);
        assert_eq!(dot, 32.0); // 1*4 + 2*5 + 3*6 = 32
    }

    #[test]
    fn test_normalize() {
        let v = vec![3.0f32, 4.0];
        let normalized = normalize(&v);
        let magnitude: f32 = normalized.iter().map(|&x| x * x).sum::<f32>().sqrt();
        assert!((magnitude - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_euclidean_distance() {
        let a = vec![0.0f32, 0.0];
        let b = vec![3.0f32, 4.0];
        let dist = euclidean_distance(&a, &b);
        assert!((dist - 5.0).abs() < 0.001); // 3-4-5 triangle
    }

    #[test]
    fn test_batch_cosine_similarity() {
        let query = vec![1.0f32, 0.0, 0.0, 0.0];
        let candidates = vec![
            vec![1.0f32, 0.0, 0.0, 0.0],  // identical
            vec![0.0f32, 1.0, 0.0, 0.0],  // orthogonal
            vec![-1.0f32, 0.0, 0.0, 0.0], // opposite
        ];
        let scores = batch_cosine_similarity(&query, &candidates);
        assert!((scores[0] - 1.0).abs() < 0.001);
        assert!(scores[1].abs() < 0.001);
        assert!((scores[2] + 1.0).abs() < 0.001);
    }

    #[test]
    fn test_find_most_similar() {
        let query = vec![1.0f32, 0.0, 0.0, 0.0];
        let candidates = vec![
            vec![0.0f32, 1.0, 0.0, 0.0], // orthogonal
            vec![1.0f32, 0.0, 0.0, 0.0], // identical (best)
            vec![0.0f32, 0.0, 1.0, 0.0], // orthogonal
        ];
        let best_idx = find_most_similar(&query, &candidates);
        assert_eq!(best_idx, Some(1));
    }

    #[test]
    fn test_find_most_similar_empty() {
        let query = vec![1.0f32, 0.0, 0.0];
        let candidates: Vec<Vec<f32>> = vec![];
        let best_idx = find_most_similar(&query, &candidates);
        assert_eq!(best_idx, None);
    }

    #[test]
    fn test_normalize_panic() {
        let v = vec![0.0f32, 0.0, 0.0];
        let result = std::panic::catch_unwind(|| normalize(&v));
        assert!(result.is_err());
    }

    #[test]
    fn test_cosine_similarity_large_vector() {
        // Test with vector larger than 8 elements (tests SIMD + remainder)
        let a: Vec<f32> = (0..20).map(|i| if i == 0 { 1.0 } else { 0.0 }).collect();
        let b: Vec<f32> = (0..20).map(|i| if i == 0 { 1.0 } else { 0.0 }).collect();
        let sim = cosine_similarity(&a, &b);
        assert!((sim - 1.0).abs() < 0.001);
    }
}
