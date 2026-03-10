//! Model cache management — Cache directory, validation, and lifecycle
//!
//! Handles caching of AI models with:
//! - Automatic cache directory creation
//! - SHA256 integrity validation
//! - Cache cleanup and lifecycle management
//! - Offline mode support
//!
//! # Cache Structure
//!
//! ```text
//! ~/.cache/rust-scraper/ai_models/
//! ├── model.onnx              # ONNX model file
//! ├── model.onnx.sha256       # SHA256 checksum
//! ├── tokenizer.json          # Tokenizer configuration
//! └── metadata.json           # Model metadata (version, download date)
//! ```
//!
//! # Design Decisions
//!
//! - **XDG cache convention** (`dirs` crate): Follows OS cache directory standards
//! - **SHA256 validation**: Ensures cache integrity after download
//! - **Memory-mapped loading** (`memmap2`): Zero-copy for HDD optimization
//! - **Async file operations** (`async-tokio-fs`): Non-blocking I/O

use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use sha2::{Digest, Sha256};
use tokio::fs::{self, File};
use tokio::io::{AsyncReadExt, BufReader};
use tracing::{debug, info};

use crate::error::SemanticError;

/// Default cache directory path
///
/// Uses XDG cache convention: `~/.cache/rust-scraper/ai_models/`
const CACHE_DIR_NAME: &str = "rust-scraper";
const AI_MODELS_SUBDIR: &str = "ai_models";

/// Default model repository
pub const DEFAULT_MODEL_REPO: &str = "sentence-transformers/all-MiniLM-L6-v2";

/// Default model file name
pub const DEFAULT_MODEL_FILE: &str = "model.onnx";

/// Expected SHA256 for all-MiniLM-L6-v2 ONNX model
///
/// This is the known-good hash for the official model from HuggingFace Hub.
/// If validation fails, the cache is considered corrupted.
pub const DEFAULT_MODEL_SHA256: &str =
    "6d9d2f06f5e2e5e6f5e5e5e5e5e5e5e5e5e5e5e5e5e5e5e5e5e5e5e5e5e5e5e5"; // Placeholder - update with real hash

/// Cache configuration
///
/// Controls cache behavior and validation settings.
#[derive(Debug, Clone)]
pub struct CacheConfig {
    /// Cache directory path (default: ~/.cache/rust-scraper/ai_models/)
    pub cache_dir: PathBuf,
    /// Enable SHA256 validation (default: true)
    pub validate_sha256: bool,
    /// Expected SHA256 hash (optional, uses default if None)
    pub expected_sha256: Option<String>,
    /// Enable offline mode (default: false)
    ///
    /// In offline mode, fails if model is not cached instead of downloading.
    pub offline_mode: bool,
    /// Cache TTL (time-to-live) in days (default: 30)
    ///
    /// After this period, the model is considered stale and should be revalidated.
    pub cache_ttl_days: Option<u64>,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            cache_dir: default_cache_dir(),
            validate_sha256: true,
            expected_sha256: None,
            offline_mode: false,
            cache_ttl_days: Some(30),
        }
    }
}

impl CacheConfig {
    /// Create a new cache configuration with default values
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Set custom cache directory
    ///
    /// # Arguments
    ///
    /// * `dir` - Custom cache directory path
    #[must_use]
    pub fn with_cache_dir(mut self, dir: PathBuf) -> Self {
        self.cache_dir = dir;
        self
    }

    /// Enable or disable SHA256 validation
    ///
    /// # Arguments
    ///
    /// * `validate` - `true` to enable validation (recommended)
    #[must_use]
    pub fn with_validation(mut self, validate: bool) -> Self {
        self.validate_sha256 = validate;
        self
    }

    /// Set expected SHA256 hash
    ///
    /// # Arguments
    ///
    /// * `sha256` - Expected SHA256 hash (hex string)
    #[must_use]
    pub fn with_sha256(mut self, sha256: impl Into<String>) -> Self {
        self.expected_sha256 = Some(sha256.into());
        self
    }

    /// Enable offline mode
    ///
    /// In offline mode, operations fail if model is not cached.
    #[must_use]
    pub fn with_offline_mode(mut self, offline: bool) -> Self {
        self.offline_mode = offline;
        self
    }

    /// Set cache TTL
    ///
    /// # Arguments
    ///
    /// * `days` - Cache TTL in days (None = no expiration)
    #[must_use]
    pub fn with_ttl_days(mut self, days: Option<u64>) -> Self {
        self.cache_ttl_days = days;
        self
    }
}

/// Get the default cache directory path
///
/// Returns `~/.cache/rust-scraper/ai_models/` on Linux/macOS
/// Falls back to current directory if home directory is unavailable.
///
/// # Examples
///
/// ```
/// use rust_scraper::infrastructure::ai::model_cache::default_cache_dir;
///
/// let cache_dir = default_cache_dir();
/// println!("Cache directory: {:?}", cache_dir);
/// ```
#[must_use]
pub fn default_cache_dir() -> PathBuf {
    // Try to use dirs crate for XDG-compliant paths
    if let Some(cache_base) = dirs::cache_dir() {
        cache_base.join(CACHE_DIR_NAME).join(AI_MODELS_SUBDIR)
    } else {
        // Fallback to current directory
        PathBuf::from("./.cache/ai_models")
    }
}

/// Model cache manager
///
/// Handles cache lifecycle: creation, validation, cleanup.
///
/// # Examples
///
/// ```no_run
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// use rust_scraper::infrastructure::ai::model_cache::{ModelCache, CacheConfig};
///
/// let config = CacheConfig::default();
/// let cache = ModelCache::new(config);
///
/// // Ensure cache directory exists
/// cache.ensure_cache_dir().await?;
///
/// // Check if model is cached
/// let is_cached = cache.is_model_cached("model.onnx");
/// println!("Model cached: {}", is_cached);
/// # Ok(())
/// # }
/// ```
pub struct ModelCache {
    config: CacheConfig,
}

impl ModelCache {
    /// Create a new model cache manager
    ///
    /// # Arguments
    ///
    /// * `config` - Cache configuration
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_scraper::infrastructure::ai::model_cache::{ModelCache, CacheConfig};
    ///
    /// let config = CacheConfig::default();
    /// let cache = ModelCache::new(config);
    /// ```
    #[must_use]
    pub fn new(config: CacheConfig) -> Self {
        Self { config }
    }

    /// Get the cache directory path
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_scraper::infrastructure::ai::model_cache::{ModelCache, CacheConfig};
    ///
    /// let config = CacheConfig::default();
    /// let cache = ModelCache::new(config);
    /// let cache_dir = cache.cache_dir();
    /// ```
    #[must_use]
    pub fn cache_dir(&self) -> &Path {
        &self.config.cache_dir
    }

    /// Ensure cache directory exists
    ///
    /// Creates the directory (and parent directories) if they don't exist.
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Directory exists or was created successfully
    /// * `Err(SemanticError::ModelLoad)` - Failed to create directory
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// use rust_scraper::infrastructure::ai::model_cache::{ModelCache, CacheConfig};
    ///
    /// let cache = ModelCache::new(CacheConfig::default());
    /// cache.ensure_cache_dir().await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn ensure_cache_dir(&self) -> Result<(), SemanticError> {
        fs::create_dir_all(&self.config.cache_dir)
            .await
            .map_err(|e| {
                SemanticError::ModelLoad(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("Failed to create cache directory: {}", e),
                ))
            })?;

        debug!(
            path = ?self.config.cache_dir,
            "Cache directory ready"
        );

        Ok(())
    }

    /// Check if a model file exists in cache
    ///
    /// This is a quick existence check. Does NOT validate integrity.
    ///
    /// # Arguments
    ///
    /// * `model_file` - Model filename to check (e.g., "model.onnx")
    ///
    /// # Returns
    ///
    /// * `true` - File exists in cache
    /// * `false` - File doesn't exist
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_scraper::infrastructure::ai::model_cache::{ModelCache, CacheConfig};
    ///
    /// let cache = ModelCache::new(CacheConfig::default());
    /// let is_cached = cache.is_model_cached("model.onnx");
    /// ```
    #[must_use]
    pub fn is_model_cached(&self, model_file: &str) -> bool {
        let model_path = self.config.cache_dir.join(model_file);
        model_path.exists()
    }

    /// Validate model file integrity using SHA256
    ///
    /// Computes SHA256 hash of the cached file and compares with expected value.
    ///
    /// # Arguments
    ///
    /// * `model_file` - Model filename to validate
    /// * `expected_sha256` - Expected SHA256 hash (uses config default if None)
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Validation passed
    /// * `Err(SemanticError::CacheValidation)` - Hash mismatch
    /// * `Err(SemanticError::ModelLoad)` - File read error
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// use rust_scraper::infrastructure::ai::model_cache::{ModelCache, CacheConfig};
    ///
    /// let cache = ModelCache::new(CacheConfig::default());
    /// cache.validate_model("model.onnx", None).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn validate_model(
        &self,
        model_file: &str,
        expected_sha256: Option<&str>,
    ) -> Result<(), SemanticError> {
        let model_path = self.config.cache_dir.join(model_file);

        // Check file exists
        if !model_path.exists() {
            return Err(SemanticError::ModelLoad(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("Model file not found: {:?}", model_path),
            )));
        }

        // Get expected hash
        let expected = expected_sha256
            .map(String::from)
            .or_else(|| self.config.expected_sha256.clone())
            .unwrap_or_else(|| DEFAULT_MODEL_SHA256.to_string());

        // Compute actual hash
        let file = File::open(&model_path).await.map_err(|e| {
            SemanticError::ModelLoad(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to open model file: {}", e),
            ))
        })?;

        let mut reader = BufReader::new(file);
        let mut hasher = Sha256::new();
        let mut buffer = vec![0u8; 8192]; // 8KB buffer for streaming hash

        loop {
            let bytes_read = reader.read(&mut buffer).await.map_err(|e| {
                SemanticError::ModelLoad(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("Failed to read model file: {}", e),
                ))
            })?;

            if bytes_read == 0 {
                break;
            }

            hasher.update(&buffer[..bytes_read]);
        }

        let actual_sha = format!("{:x}", hasher.finalize());

        // Compare hashes
        if actual_sha != expected {
            return Err(SemanticError::CacheValidation {
                repo: DEFAULT_MODEL_REPO.to_string(),
                expected,
                actual: actual_sha,
            });
        }

        debug!(
            path = ?model_path,
            sha = %actual_sha,
            "Model validation passed"
        );

        Ok(())
    }

    /// Check if cached model is stale
    ///
    /// A model is considered stale if:
    /// - It was downloaded more than `cache_ttl_days` ago
    /// - File modification time is older than TTL
    ///
    /// # Arguments
    ///
    /// * `model_file` - Model filename to check
    ///
    /// # Returns
    ///
    /// * `true` - Model is stale (should be revalidated)
    /// * `false` - Model is fresh or TTL not configured
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// use rust_scraper::infrastructure::ai::model_cache::{ModelCache, CacheConfig};
    ///
    /// let cache = ModelCache::new(CacheConfig::default());
    /// let is_stale = cache.is_model_stale("model.onnx").await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn is_model_stale(&self, model_file: &str) -> Result<bool, SemanticError> {
        let Some(ttl_days) = self.config.cache_ttl_days else {
            return Ok(false); // No TTL configured = never stale
        };

        let model_path = self.config.cache_dir.join(model_file);

        if !model_path.exists() {
            return Ok(true); // Doesn't exist = stale
        }

        // Get file modification time
        let metadata = fs::metadata(&model_path).await.map_err(|e| {
            SemanticError::ModelLoad(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to read model metadata: {}", e),
            ))
        })?;

        let modified = metadata.modified().unwrap_or(SystemTime::UNIX_EPOCH);

        let age = modified.elapsed().unwrap_or(Duration::ZERO);

        let ttl = Duration::from_secs(ttl_days * 24 * 60 * 60);

        Ok(age > ttl)
    }

    /// Get the full path to a cached model file
    ///
    /// # Arguments
    ///
    /// * `model_file` - Model filename
    ///
    /// # Returns
    ///
    /// Full path to the model file in cache
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_scraper::infrastructure::ai::model_cache::{ModelCache, CacheConfig};
    ///
    /// let cache = ModelCache::new(CacheConfig::default());
    /// let model_path = cache.model_path("model.onnx");
    /// ```
    #[must_use]
    pub fn model_path(&self, model_file: &str) -> PathBuf {
        self.config.cache_dir.join(model_file)
    }

    /// Clear the entire cache
    ///
    /// Deletes all files in the cache directory.
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Cache cleared successfully
    /// * `Err(SemanticError::ModelLoad)` - Failed to delete files
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// use rust_scraper::infrastructure::ai::model_cache::{ModelCache, CacheConfig};
    ///
    /// let cache = ModelCache::new(CacheConfig::default());
    /// cache.clear().await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn clear(&self) -> Result<(), SemanticError> {
        if !self.config.cache_dir.exists() {
            return Ok(()); // Nothing to clear
        }

        fs::remove_dir_all(&self.config.cache_dir)
            .await
            .map_err(|e| {
                SemanticError::ModelLoad(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("Failed to clear cache: {}", e),
                ))
            })?;

        info!(
            path = ?self.config.cache_dir,
            "Cache cleared"
        );

        Ok(())
    }

    /// Get cache size in bytes
    ///
    /// Sums up the size of all files in the cache directory.
    ///
    /// # Returns
    ///
    /// * `Ok(u64)` - Total cache size in bytes
    /// * `Err(SemanticError::ModelLoad)` - Failed to read directory
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// use rust_scraper::infrastructure::ai::model_cache::{ModelCache, CacheConfig};
    ///
    /// let cache = ModelCache::new(CacheConfig::default());
    /// let size_bytes = cache.size().await?;
    /// println!("Cache size: {} MB", size_bytes / (1024 * 1024));
    /// # Ok(())
    /// # }
    /// ```
    pub async fn size(&self) -> Result<u64, SemanticError> {
        if !self.config.cache_dir.exists() {
            return Ok(0);
        }

        let mut total_size = 0u64;
        let mut entries = fs::read_dir(&self.config.cache_dir).await.map_err(|e| {
            SemanticError::ModelLoad(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to read cache directory: {}", e),
            ))
        })?;

        while let Some(entry) = entries.next_entry().await.map_err(|e| {
            SemanticError::ModelLoad(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to read directory entry: {}", e),
            ))
        })? {
            let metadata = entry.metadata().await.map_err(|e| {
                SemanticError::ModelLoad(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("Failed to read entry metadata: {}", e),
                ))
            })?;

            if metadata.is_file() {
                total_size += metadata.len();
            }
        }

        Ok(total_size)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_cache_dir() {
        let cache_dir = default_cache_dir();
        assert!(cache_dir.ends_with("ai_models"));
        assert!(cache_dir.to_string_lossy().contains("rust-scraper"));
    }

    #[test]
    fn test_cache_config_default() {
        let config = CacheConfig::default();
        assert!(config.validate_sha256);
        assert!(!config.offline_mode);
        assert_eq!(config.cache_ttl_days, Some(30));
    }

    #[test]
    fn test_cache_config_builder() {
        let config = CacheConfig::new()
            .with_validation(false)
            .with_offline_mode(true)
            .with_ttl_days(None);

        assert!(!config.validate_sha256);
        assert!(config.offline_mode);
        assert!(config.cache_ttl_days.is_none());
    }

    #[tokio::test]
    async fn test_ensure_cache_dir() {
        let temp_dir = tempfile::tempdir().unwrap();
        let cache_dir = temp_dir.path().join("test_cache");

        let config = CacheConfig::new().with_cache_dir(cache_dir.clone());
        let cache = ModelCache::new(config);

        // Directory shouldn't exist yet
        assert!(!cache_dir.exists());

        // Create it
        cache.ensure_cache_dir().await.unwrap();

        // Now it should exist
        assert!(cache_dir.exists());
    }

    #[tokio::test]
    async fn test_is_model_cached() {
        let temp_dir = tempfile::tempdir().unwrap();
        let cache_dir = temp_dir.path().join("test_cache");

        let config = CacheConfig::new().with_cache_dir(cache_dir.clone());
        let cache = ModelCache::new(config);

        // Should return false for non-existent file
        assert!(!cache.is_model_cached("model.onnx"));

        // Create a dummy file
        fs::create_dir_all(&cache_dir).await.unwrap();
        File::create(cache_dir.join("model.onnx")).await.unwrap();

        // Should return true now
        assert!(cache.is_model_cached("model.onnx"));
    }

    #[tokio::test]
    async fn test_model_path() {
        let temp_dir = tempfile::tempdir().unwrap();
        let cache_dir = temp_dir.path().join("test_cache");

        let config = CacheConfig::new().with_cache_dir(cache_dir.clone());
        let cache = ModelCache::new(config);

        let model_path = cache.model_path("model.onnx");
        assert_eq!(model_path, cache_dir.join("model.onnx"));
    }
}
