use anyhow::{Result, Context};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;
use sha2::{Sha256, Digest};
use tracing::{info, warn, debug};

use crate::security_layer::SecurityLayer;

#[derive(thiserror::Error, Debug)]
pub enum IPFSError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Storage error: {0}")]
    Storage(String),
    #[error("Content not found: {0}")]
    NotFound(String),
    #[error("Content too large: {0} bytes (max {1})")]
    TooLarge(usize, usize),
}

/// Content-addressed storage node.
///
/// Stores blobs on disk under `<cas_dir>/<first-2-hex>/<full-sha256-hex>`.
/// The hash returned by [`store_content`] is the hex-encoded SHA-256 digest
/// of the raw bytes that were stored, so retrieval is deterministic and
/// verifiable.
#[derive(Clone)]
pub struct IPFSNode {
    cas_dir: PathBuf,
    max_content_size: usize,
    max_storage_bytes: u64,
    security_layer: Arc<SecurityLayer>,
    /// In-memory count cache (refreshed on startup, updated on writes).
    count: Arc<RwLock<usize>>,
}

impl IPFSNode {
    pub async fn new(security_layer: Arc<SecurityLayer>) -> Result<Self> {
        Self::with_dir("./data/cas", security_layer).await
    }

    pub async fn with_dir(cas_dir: impl AsRef<Path>, security_layer: Arc<SecurityLayer>) -> Result<Self> {
        let cas_dir = cas_dir.as_ref().to_path_buf();
        tokio::fs::create_dir_all(&cas_dir).await
            .context("Failed to create CAS directory")?;

        let count = count_files_in_dir(&cas_dir).await;

        info!("📦 Content-addressed store ready at {:?} ({} objects)", cas_dir, count);

        Ok(Self {
            cas_dir,
            max_content_size: 10 * 1024 * 1024, // 10 MB default
            max_storage_bytes: 5 * 1024 * 1024 * 1024, // 5 GB default
            security_layer,
            count: Arc::new(RwLock::new(count)),
        })
    }

    /// Store content and return its SHA-256 hex hash.
    pub async fn store_content(&self, content: &[u8]) -> Result<String> {
        if content.is_empty() {
            return Err(IPFSError::Storage("Cannot store empty content".into()).into());
        }
        if content.len() > self.max_content_size {
            return Err(IPFSError::TooLarge(content.len(), self.max_content_size).into());
        }

        let hash = sha256_hex(content);
        let blob_path = self.blob_path(&hash);

        // If already stored, return immediately (idempotent).
        if blob_path.exists() {
            debug!("📦 Content already exists: {}", hash);
            return Ok(hash);
        }

        // Ensure shard directory exists
        if let Some(parent) = blob_path.parent() {
            tokio::fs::create_dir_all(parent).await
                .context("Failed to create shard directory")?;
        }

        // Atomic write: write to a temp file then rename.
        let tmp_path = blob_path.with_extension("tmp");
        tokio::fs::write(&tmp_path, content).await
            .context("Failed to write content blob")?;
        tokio::fs::rename(&tmp_path, &blob_path).await
            .context("Failed to finalize content blob")?;

        *self.count.write().await += 1;

        info!("📦 Content stored: {} ({} bytes)", hash, content.len());
        Ok(hash)
    }

    /// Retrieve content by its SHA-256 hex hash. Verifies integrity on read.
    pub async fn retrieve_content(&self, hash: &str) -> Result<Vec<u8>> {
        // Basic hex validation
        if hash.len() != 64 || !hash.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(IPFSError::NotFound(hash.to_string()).into());
        }

        let blob_path = self.blob_path(hash);
        if !blob_path.exists() {
            return Err(IPFSError::NotFound(hash.to_string()).into());
        }

        let data = tokio::fs::read(&blob_path).await
            .context("Failed to read content blob")?;

        // Verify integrity
        let actual_hash = sha256_hex(&data);
        if actual_hash != hash {
            warn!("⚠️ Integrity mismatch for {}: got {}", hash, actual_hash);
            return Err(IPFSError::Storage(format!(
                "Integrity check failed for {}", hash
            )).into());
        }

        debug!("📦 Content retrieved: {} ({} bytes)", hash, data.len());
        Ok(data)
    }

    /// Delete content by hash. Returns true if it existed.
    pub async fn delete_content(&self, hash: &str) -> Result<bool> {
        let blob_path = self.blob_path(hash);
        if blob_path.exists() {
            tokio::fs::remove_file(&blob_path).await
                .context("Failed to delete content blob")?;
            let mut c = self.count.write().await;
            *c = c.saturating_sub(1);
            info!("🗑️ Content deleted: {}", hash);
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Check if a hash exists in the store.
    pub async fn has_content(&self, hash: &str) -> bool {
        self.blob_path(hash).exists()
    }

    // Compatibility methods -----------------------------------------------

    pub async fn get_peer_id(&self) -> String {
        // No real peer networking yet; return a stable identifier based on the CAS dir.
        format!("local-cas:{}", self.cas_dir.display())
    }

    pub async fn get_addresses(&self) -> Vec<String> {
        vec![format!("file://{}", self.cas_dir.display())]
    }

    pub async fn get_content_count(&self) -> usize {
        *self.count.read().await
    }

    pub async fn start(&self) -> Result<()> {
        info!("📦 Content-addressed storage service running");
        // No background loop needed — storage is on-demand.
        // Just park so the spawned task doesn't immediately exit.
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(3600)).await;
        }
    }

    // Internal helpers ----------------------------------------------------

    fn blob_path(&self, hash: &str) -> PathBuf {
        // Shard by first 2 hex chars to avoid huge flat directories.
        let shard = &hash[..2.min(hash.len())];
        self.cas_dir.join(shard).join(hash)
    }
}

/// SHA-256 hex digest of bytes.
fn sha256_hex(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hex::encode(hasher.finalize())
}

/// Count existing blob files in the CAS directory tree.
async fn count_files_in_dir(dir: &Path) -> usize {
    let mut count = 0;
    if let Ok(mut entries) = tokio::fs::read_dir(dir).await {
        while let Ok(Some(entry)) = entries.next_entry().await {
            let path = entry.path();
            if path.is_dir() {
                if let Ok(mut sub) = tokio::fs::read_dir(&path).await {
                    while let Ok(Some(_)) = sub.next_entry().await {
                        count += 1;
                    }
                }
            }
        }
    }
    count
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_store_and_retrieve() {
        let dir = tempfile::tempdir().unwrap();
        let security = Arc::new(SecurityLayer::new().await.unwrap());
        let node = IPFSNode::with_dir(dir.path(), security).await.unwrap();

        let data = b"hello world";
        let hash = node.store_content(data).await.unwrap();
        assert_eq!(hash.len(), 64); // SHA-256 hex

        let retrieved = node.retrieve_content(&hash).await.unwrap();
        assert_eq!(retrieved, data);
    }

    #[tokio::test]
    async fn test_idempotent_store() {
        let dir = tempfile::tempdir().unwrap();
        let security = Arc::new(SecurityLayer::new().await.unwrap());
        let node = IPFSNode::with_dir(dir.path(), security).await.unwrap();

        let data = b"duplicate test";
        let hash1 = node.store_content(data).await.unwrap();
        let hash2 = node.store_content(data).await.unwrap();
        assert_eq!(hash1, hash2);
        assert_eq!(node.get_content_count().await, 1);
    }

    #[tokio::test]
    async fn test_not_found() {
        let dir = tempfile::tempdir().unwrap();
        let security = Arc::new(SecurityLayer::new().await.unwrap());
        let node = IPFSNode::with_dir(dir.path(), security).await.unwrap();

        let result = node.retrieve_content(
            "0000000000000000000000000000000000000000000000000000000000000000"
        ).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_empty_content_rejected() {
        let dir = tempfile::tempdir().unwrap();
        let security = Arc::new(SecurityLayer::new().await.unwrap());
        let node = IPFSNode::with_dir(dir.path(), security).await.unwrap();

        let result = node.store_content(b"").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_delete_content() {
        let dir = tempfile::tempdir().unwrap();
        let security = Arc::new(SecurityLayer::new().await.unwrap());
        let node = IPFSNode::with_dir(dir.path(), security).await.unwrap();

        let hash = node.store_content(b"to be deleted").await.unwrap();
        assert!(node.has_content(&hash).await);
        assert!(node.delete_content(&hash).await.unwrap());
        assert!(!node.has_content(&hash).await);
    }
}
