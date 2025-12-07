//! Filesystem operations for NeuroFlow Notes.
//!
//! This crate handles all direct filesystem interactions:
//! - Reading and writing markdown files
//! - Scanning directories for markdown files
//! - Computing file hashes for change detection

use std::path::{Path, PathBuf};
use thiserror::Error;
use tokio::fs;
use tracing::{debug, instrument};
use xxhash_rust::xxh3::xxh3_64;

/// Errors that can occur during filesystem operations.
#[derive(Error, Debug)]
pub enum FsError {
    #[error("File not found: {0}")]
    NotFound(PathBuf),

    #[error("Path is not within vault: {0}")]
    OutsideVault(PathBuf),

    #[error("Invalid path: {0}")]
    InvalidPath(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, FsError>;

/// A handle to a vault's filesystem.
#[derive(Debug, Clone)]
pub struct VaultFs {
    /// Root path of the vault.
    root: PathBuf,
}

impl VaultFs {
    /// Create a new VaultFs for the given root path.
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    /// Get the root path of the vault.
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Ensure the .neuroflow directory exists.
    pub async fn ensure_neuroflow_dir(&self) -> Result<PathBuf> {
        let neuroflow_dir = self.root.join(".neuroflow");
        fs::create_dir_all(&neuroflow_dir).await?;
        Ok(neuroflow_dir)
    }

    /// Get the path to the vault database.
    pub fn db_path(&self) -> PathBuf {
        self.root.join(".neuroflow").join("neuroflow.db")
    }

    /// Get the path to the vault config.
    pub fn config_path(&self) -> PathBuf {
        self.root.join(".neuroflow").join("config.json")
    }

    /// Check if a path is within the vault.
    pub fn is_within_vault(&self, path: &Path) -> bool {
        path.starts_with(&self.root)
    }

    /// Convert an absolute path to a vault-relative path.
    pub fn to_relative(&self, path: &Path) -> Result<PathBuf> {
        path.strip_prefix(&self.root)
            .map(|p| p.to_path_buf())
            .map_err(|_| FsError::OutsideVault(path.to_path_buf()))
    }

    /// Convert a vault-relative path to an absolute path.
    pub fn to_absolute(&self, relative: &Path) -> PathBuf {
        self.root.join(relative)
    }

    /// Read a markdown file's content.
    #[instrument(skip(self), fields(vault = %self.root.display()))]
    pub async fn read_file(&self, relative_path: &Path) -> Result<String> {
        let absolute = self.to_absolute(relative_path);
        debug!("Reading file: {}", absolute.display());

        if !absolute.exists() {
            return Err(FsError::NotFound(absolute));
        }

        let content = fs::read_to_string(&absolute).await?;
        Ok(content)
    }

    /// Write content to a markdown file.
    #[instrument(skip(self, content), fields(vault = %self.root.display()))]
    pub async fn write_file(&self, relative_path: &Path, content: &str) -> Result<()> {
        let absolute = self.to_absolute(relative_path);
        debug!("Writing file: {}", absolute.display());

        // Ensure parent directory exists
        if let Some(parent) = absolute.parent() {
            fs::create_dir_all(parent).await?;
        }

        fs::write(&absolute, content).await?;
        Ok(())
    }

    /// Delete a file.
    #[instrument(skip(self), fields(vault = %self.root.display()))]
    pub async fn delete_file(&self, relative_path: &Path) -> Result<()> {
        let absolute = self.to_absolute(relative_path);
        debug!("Deleting file: {}", absolute.display());

        if absolute.exists() {
            fs::remove_file(&absolute).await?;
        }
        Ok(())
    }

    /// Rename/move a file within the vault.
    #[instrument(skip(self), fields(vault = %self.root.display()))]
    pub async fn rename_file(&self, from_path: &Path, to_path: &Path) -> Result<()> {
        let from_absolute = self.to_absolute(from_path);
        let to_absolute = self.to_absolute(to_path);
        debug!("Renaming file: {} -> {}", from_absolute.display(), to_absolute.display());

        if !from_absolute.exists() {
            return Err(FsError::NotFound(from_absolute));
        }

        // Ensure target directory exists
        if let Some(parent) = to_absolute.parent() {
            fs::create_dir_all(parent).await?;
        }

        fs::rename(&from_absolute, &to_absolute).await?;
        Ok(())
    }

    /// Check if a file exists.
    pub async fn exists(&self, relative_path: &Path) -> bool {
        let absolute = self.to_absolute(relative_path);
        absolute.exists()
    }

    /// Get file metadata (modification time).
    pub async fn modified_time(&self, relative_path: &Path) -> Result<std::time::SystemTime> {
        let absolute = self.to_absolute(relative_path);
        let metadata = fs::metadata(&absolute).await?;
        Ok(metadata.modified()?)
    }

    /// Scan the vault for all markdown files.
    #[instrument(skip(self), fields(vault = %self.root.display()))]
    pub async fn scan_markdown_files(&self) -> Result<Vec<PathBuf>> {
        let mut files = Vec::new();
        self.scan_dir_recursive(&self.root, &mut files).await?;
        debug!("Found {} markdown files", files.len());
        Ok(files)
    }

    /// Recursively scan a directory for markdown files.
    #[async_recursion::async_recursion]
    async fn scan_dir_recursive(&self, dir: &Path, files: &mut Vec<PathBuf>) -> Result<()> {
        let mut entries = fs::read_dir(dir).await?;

        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

            // Skip hidden files/directories and .neuroflow
            if file_name.starts_with('.') {
                continue;
            }

            if path.is_dir() {
                self.scan_dir_recursive(&path, files).await?;
            } else if path.extension().and_then(|e| e.to_str()) == Some("md") {
                // Store as relative path
                if let Ok(relative) = self.to_relative(&path) {
                    files.push(relative);
                }
            }
        }

        Ok(())
    }
}

/// Compute a hash of file content for change detection.
pub fn hash_content(content: &str) -> String {
    let hash = xxh3_64(content.as_bytes());
    format!("{:016x}", hash)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_content() {
        let hash1 = hash_content("Hello, world!");
        let hash2 = hash_content("Hello, world!");
        let hash3 = hash_content("Different content");

        assert_eq!(hash1, hash2);
        assert_ne!(hash1, hash3);
    }
}
