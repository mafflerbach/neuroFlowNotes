//! Vault management - opening, indexing, and coordinating vault operations.

use crate::watcher::FileWatcher;
use core_fs::{hash_content, VaultFs};
use core_index::markdown::parse;
use core_storage::{init_database, VaultRepository};
use shared_types::{IndexCompletePayload, NoteListItem, VaultInfo};
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Instant;
use thiserror::Error;
use tokio::sync::{broadcast, RwLock};
use tracing::{debug, error, info, instrument, warn};

#[derive(Error, Debug)]
pub enum VaultError {
    #[error("Vault path does not exist: {0}")]
    PathNotFound(PathBuf),

    #[error("Vault path is not a directory: {0}")]
    NotADirectory(PathBuf),

    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Storage error: {0}")]
    Storage(#[from] core_storage::StorageError),

    #[error("Filesystem error: {0}")]
    Fs(#[from] core_fs::FsError),

    #[error("Watcher error: {0}")]
    Watcher(#[from] notify::Error),
}

pub type Result<T> = std::result::Result<T, VaultError>;

/// Events emitted by the vault.
#[derive(Debug, Clone)]
pub enum VaultEvent {
    /// Notes were updated (reindexed).
    NotesUpdated(Vec<i64>),
    /// Notes were deleted.
    NotesDeleted(Vec<i64>),
    /// Full index complete.
    IndexComplete(IndexCompletePayload),
}

/// An open vault.
pub struct Vault {
    /// Filesystem handle.
    fs: VaultFs,
    /// Database repository.
    repo: VaultRepository,
    /// File watcher (optional, can be disabled).
    watcher: Option<FileWatcher>,
    /// Event sender for vault events.
    event_tx: broadcast::Sender<VaultEvent>,
    /// Track if initial index is complete.
    indexed: Arc<RwLock<bool>>,
}

impl Vault {
    /// Open a vault at the given path.
    #[instrument(skip_all, fields(path = %path.as_ref().display()))]
    pub async fn open(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();

        // Validate path
        if !path.exists() {
            return Err(VaultError::PathNotFound(path.to_path_buf()));
        }
        if !path.is_dir() {
            return Err(VaultError::NotADirectory(path.to_path_buf()));
        }

        info!("Opening vault at {}", path.display());

        // Create filesystem handle
        let fs = VaultFs::new(path);

        // Ensure .neuroflow directory exists
        fs.ensure_neuroflow_dir().await?;

        // Open database
        let db_path = fs.db_path();
        info!("Database path: {}", db_path.display());

        let options = SqliteConnectOptions::new()
            .filename(&db_path)
            .create_if_missing(true);

        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect_with(options)
            .await?;

        // Initialize schema
        init_database(&pool).await?;

        let repo = VaultRepository::new(pool);

        // Create event channel
        let (event_tx, _) = broadcast::channel(100);

        let vault = Self {
            fs,
            repo,
            watcher: None,
            event_tx,
            indexed: Arc::new(RwLock::new(false)),
        };

        Ok(vault)
    }

    /// Get vault info.
    pub async fn info(&self) -> Result<VaultInfo> {
        let note_count = self.repo.count_notes().await?;
        let name = self
            .fs
            .root()
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("Vault")
            .to_string();

        Ok(VaultInfo {
            path: self.fs.root().to_string_lossy().to_string(),
            name,
            note_count,
        })
    }

    /// Get the repository for direct access.
    pub fn repo(&self) -> &VaultRepository {
        &self.repo
    }

    /// Get the filesystem handle.
    pub fn fs(&self) -> &VaultFs {
        &self.fs
    }

    /// Subscribe to vault events.
    pub fn subscribe(&self) -> broadcast::Receiver<VaultEvent> {
        self.event_tx.subscribe()
    }

    /// Emit a vault event.
    pub fn emit(&self, event: VaultEvent) {
        let _ = self.event_tx.send(event);
    }

    /// Perform initial full index of the vault.
    #[instrument(skip(self))]
    pub async fn full_index(&self) -> Result<IndexCompletePayload> {
        let start = Instant::now();
        info!("Starting full vault index");

        let files = self.fs.scan_markdown_files().await?;
        info!("Found {} markdown files", files.len());

        // Build a set of file paths that exist on disk
        let mut existing_paths: std::collections::HashSet<String> = std::collections::HashSet::new();
        for file_path in &files {
            existing_paths.insert(file_path.to_string_lossy().to_string());
        }

        // Cleanup: Remove notes from database that no longer exist on disk
        let db_notes = self.repo.list_notes().await?;
        let mut deleted_ids = Vec::new();
        for note in db_notes {
            if !existing_paths.contains(&note.path) {
                info!("Removing orphaned note from database: {}", note.path);
                if let Ok(Some(id)) = self.repo.delete_note(&note.path).await {
                    deleted_ids.push(id);
                }
            }
        }

        if !deleted_ids.is_empty() {
            info!("Removed {} orphaned notes from database", deleted_ids.len());
            let _ = self.event_tx.send(VaultEvent::NotesDeleted(deleted_ids));
        }

        // Index existing files
        let mut indexed_count = 0;
        let mut updated_ids = Vec::new();

        for file_path in files {
            match self.index_file(&file_path).await {
                Ok(Some(id)) => {
                    updated_ids.push(id);
                    indexed_count += 1;
                }
                Ok(None) => {
                    // File unchanged, skip
                }
                Err(e) => {
                    warn!("Failed to index {}: {}", file_path.display(), e);
                }
            }
        }

        let duration = start.elapsed();
        info!(
            "Full index complete: {} notes in {:?}",
            indexed_count, duration
        );

        // Mark as indexed
        *self.indexed.write().await = true;

        let payload = IndexCompletePayload {
            notes_indexed: indexed_count,
            duration_ms: duration.as_millis() as u64,
        };

        // Emit event
        let _ = self.event_tx.send(VaultEvent::IndexComplete(payload.clone()));

        if !updated_ids.is_empty() {
            let _ = self.event_tx.send(VaultEvent::NotesUpdated(updated_ids));
        }

        Ok(payload)
    }

    /// Index a single file. Returns the note ID if the file was indexed (changed).
    #[instrument(skip(self), fields(path = %path.as_ref().display()))]
    pub async fn index_file(&self, path: impl AsRef<Path>) -> Result<Option<i64>> {
        let path = path.as_ref();
        let path_str = path.to_string_lossy().to_string();

        // Read file content
        let content = self.fs.read_file(path).await?;
        let hash = hash_content(&content);

        // Check if file changed
        let existing_hash = self.repo.get_note_hash(&path_str).await?;
        if existing_hash.as_ref() == Some(&hash) {
            debug!("File unchanged, skipping: {}", path_str);
            return Ok(None);
        }

        // Parse markdown
        let analysis = parse(&content);

        // Index to database
        let note_id = self.repo.index_note(&path_str, &content, &hash, &analysis).await?;

        debug!("Indexed file: {} (id={})", path_str, note_id);
        Ok(Some(note_id))
    }

    /// Remove a file from the index.
    #[instrument(skip(self), fields(path = %path.as_ref().display()))]
    pub async fn remove_file(&self, path: impl AsRef<Path>) -> Result<Option<i64>> {
        let path_str = path.as_ref().to_string_lossy().to_string();
        let deleted_id = self.repo.delete_note(&path_str).await?;

        if let Some(id) = deleted_id {
            let _ = self.event_tx.send(VaultEvent::NotesDeleted(vec![id]));
        }

        Ok(deleted_id)
    }

    /// Start the file watcher.
    pub async fn start_watcher(&mut self) -> Result<()> {
        if self.watcher.is_some() {
            return Ok(());
        }

        let watcher = FileWatcher::new(
            self.fs.root().to_path_buf(),
            self.repo.clone(),
            self.fs.clone(),
            self.event_tx.clone(),
        )?;

        watcher.start().await;
        self.watcher = Some(watcher);

        info!("File watcher started");
        Ok(())
    }

    /// Stop the file watcher.
    pub async fn stop_watcher(&mut self) {
        if let Some(watcher) = self.watcher.take() {
            watcher.stop().await;
            info!("File watcher stopped");
        }
    }

    /// List all notes.
    pub async fn list_notes(&self) -> Result<Vec<NoteListItem>> {
        Ok(self.repo.list_notes().await?)
    }

    /// Read a note's content.
    pub async fn read_note(&self, path: &str) -> Result<String> {
        Ok(self.fs.read_file(Path::new(path)).await?)
    }

    /// Write a note's content.
    #[instrument(skip(self, content))]
    pub async fn write_note(&self, path: &str, content: &str) -> Result<i64> {
        // Write to filesystem
        self.fs.write_file(Path::new(path), content).await?;

        // Reindex (same flow as external change)
        let note_id = self.index_file(Path::new(path)).await?.unwrap_or_else(|| {
            // This shouldn't happen since we just wrote the file
            error!("Failed to get note ID after write: {}", path);
            0
        });

        // Emit event
        if note_id > 0 {
            let _ = self.event_tx.send(VaultEvent::NotesUpdated(vec![note_id]));
        }

        Ok(note_id)
    }
}
