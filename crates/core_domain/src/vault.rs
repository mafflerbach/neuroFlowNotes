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

    #[error("File already exists: {0}")]
    FileAlreadyExists(String),

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

    /// Rename a note (file and database path).
    #[instrument(skip(self))]
    pub async fn rename_note(&self, old_path: &str, new_path: &str) -> Result<i64> {
        // Check if target already exists
        if self.fs.exists(Path::new(new_path)).await {
            return Err(VaultError::FileAlreadyExists(new_path.to_string()));
        }

        // Rename the file on disk
        self.fs
            .rename_file(Path::new(old_path), Path::new(new_path))
            .await?;

        // Update the database path
        let note_id = self.repo.rename_note(old_path, new_path).await?;

        // Emit event
        let _ = self.event_tx.send(VaultEvent::NotesUpdated(vec![note_id]));

        info!("Renamed note {} -> {} (id={})", old_path, new_path, note_id);
        Ok(note_id)
    }

    /// Delete a note (file and database record).
    #[instrument(skip(self))]
    pub async fn delete_note(&self, path: &str) -> Result<Option<i64>> {
        // Delete file from disk
        self.fs.delete_file(Path::new(path)).await?;

        // Remove from database
        let deleted_id = self.repo.delete_note(path).await?;

        // Emit event
        if let Some(id) = deleted_id {
            let _ = self.event_tx.send(VaultEvent::NotesDeleted(vec![id]));
            info!("Deleted note: {} (id={})", path, id);
        }

        Ok(deleted_id)
    }

    /// Create a folder in the vault.
    #[instrument(skip(self))]
    pub async fn create_folder(&self, path: &str) -> Result<()> {
        let absolute = self.fs.to_absolute(Path::new(path));
        tokio::fs::create_dir_all(&absolute)
            .await
            .map_err(core_fs::FsError::from)?;
        info!("Created folder: {}", path);
        Ok(())
    }

    /// Rename/move a folder and update all note paths within it.
    #[instrument(skip(self))]
    pub async fn rename_folder(&self, old_path: &str, new_path: &str) -> Result<Vec<i64>> {
        let old_absolute = self.fs.to_absolute(Path::new(old_path));
        let new_absolute = self.fs.to_absolute(Path::new(new_path));

        // Check if target already exists
        if new_absolute.exists() {
            return Err(VaultError::FileAlreadyExists(new_path.to_string()));
        }

        // Ensure target parent directory exists
        if let Some(parent) = new_absolute.parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .map_err(core_fs::FsError::from)?;
        }

        // Find all notes in this folder and update their paths
        let notes = self.repo.list_notes().await?;
        let old_prefix = if old_path.is_empty() {
            String::new()
        } else {
            format!("{}/", old_path)
        };

        let mut updated_ids = Vec::new();
        for note in notes {
            if note.path.starts_with(&old_prefix) {
                // Calculate new path for this note
                let relative_path = note.path.strip_prefix(&old_prefix).unwrap_or(&note.path);
                let note_new_path = if new_path.is_empty() {
                    relative_path.to_string()
                } else {
                    format!("{}/{}", new_path, relative_path)
                };

                // Update database path
                let note_id = self.repo.rename_note(&note.path, &note_new_path).await?;
                updated_ids.push(note_id);
            }
        }

        // Move the folder on disk
        tokio::fs::rename(&old_absolute, &new_absolute)
            .await
            .map_err(core_fs::FsError::from)?;

        // Emit event for updated notes
        if !updated_ids.is_empty() {
            let _ = self.event_tx.send(VaultEvent::NotesUpdated(updated_ids.clone()));
        }

        info!(
            "Renamed folder {} -> {} ({} notes updated)",
            old_path,
            new_path,
            updated_ids.len()
        );
        Ok(updated_ids)
    }

    /// Delete a folder and all its contents.
    #[instrument(skip(self))]
    pub async fn delete_folder(&self, path: &str) -> Result<Vec<i64>> {
        let absolute = self.fs.to_absolute(Path::new(path));

        // First, find all notes in this folder and delete them from the database
        let notes = self.repo.list_notes().await?;
        let folder_prefix = if path.is_empty() { String::new() } else { format!("{}/", path) };
        let mut deleted_ids = Vec::new();

        for note in notes {
            if note.path.starts_with(&folder_prefix) || note.path == path {
                if let Some(id) = self.repo.delete_note(&note.path).await? {
                    deleted_ids.push(id);
                }
            }
        }

        // Then delete the folder from disk
        if absolute.exists() {
            tokio::fs::remove_dir_all(&absolute)
                .await
                .map_err(core_fs::FsError::from)?;
        }

        // Emit event for deleted notes
        if !deleted_ids.is_empty() {
            let _ = self.event_tx.send(VaultEvent::NotesDeleted(deleted_ids.clone()));
        }

        info!("Deleted folder: {} ({} notes removed)", path, deleted_ids.len());
        Ok(deleted_ids)
    }
}
