//! Vault management - opening, indexing, and coordinating vault operations.

use crate::watcher::FileWatcher;
use core_fs::{hash_content, VaultFs};
use core_index::markdown::{parse, update_wiki_links};
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

    /// Get the vault root path.
    pub fn root_path(&self) -> &Path {
        self.fs.root()
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
            debug!("File unchanged, returning existing note ID: {}", path_str);
            // Return existing note ID even though content unchanged
            let existing_note = self.repo.get_note_by_path(&path_str).await?;
            return Ok(Some(existing_note.id));
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

    /// Rename a note (file and database path), updating all references across the vault.
    #[instrument(skip(self))]
    pub async fn rename_note(&self, old_path: &str, new_path: &str) -> Result<i64> {
        // Check if target already exists
        if self.fs.exists(Path::new(new_path)).await {
            return Err(VaultError::FileAlreadyExists(new_path.to_string()));
        }

        // Get old and new note names for reference updating
        let old_name = Path::new(old_path)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or(old_path);
        let new_name = Path::new(new_path)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or(new_path);

        // Get the note ID before renaming (for finding backlinks)
        let note = self.repo.get_note_by_path(old_path).await?;
        let note_id = note.id;

        // Find all notes that link to this note
        let linking_notes = self.repo.get_notes_linking_to(note_id).await?;
        let mut updated_ids = vec![note_id];

        // Update references in all linking notes
        for linking_note in linking_notes {
            // Read the linking note's content
            let content = self.fs.read_file(Path::new(&linking_note.path)).await?;

            // Update wiki links
            let updated_content = update_wiki_links(&content, old_name, new_name);

            // Only write if content changed
            if updated_content != content {
                debug!(
                    "Updating references in {} ({} -> {})",
                    linking_note.path, old_name, new_name
                );

                // Write updated content
                self.fs
                    .write_file(Path::new(&linking_note.path), &updated_content)
                    .await?;

                // Reindex the updated note
                if let Ok(Some(_)) = self.index_file(Path::new(&linking_note.path)).await {
                    updated_ids.push(linking_note.id);
                }
            }
        }

        // Rename the file on disk
        self.fs
            .rename_file(Path::new(old_path), Path::new(new_path))
            .await?;

        // Update the database path
        self.repo.rename_note(old_path, new_path).await?;

        // Emit event for all updated notes
        let _ = self.event_tx.send(VaultEvent::NotesUpdated(updated_ids.clone()));

        info!(
            "Renamed note {} -> {} (id={}), updated {} references",
            old_path,
            new_path,
            note_id,
            updated_ids.len() - 1
        );
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

    /// Resolve a note name/path to its full path and ID.
    /// Supports fuzzy matching by title or exact path matching.
    pub async fn resolve_note(&self, target: &str) -> Option<(i64, String)> {
        let notes = self.repo.list_notes().await.ok()?;

        // Try exact path match first (with or without .md)
        let target_path = if target.ends_with(".md") {
            target.to_string()
        } else {
            format!("{}.md", target)
        };

        if let Some(note) = notes.iter().find(|n| n.path == target_path) {
            return Some((note.id, note.path.clone()));
        }

        // Also try matching by just the filename (for notes in subdirectories)
        if let Some(note) = notes.iter().find(|n| {
            n.path.ends_with(&format!("/{}", target_path)) ||
            n.path == target_path
        }) {
            return Some((note.id, note.path.clone()));
        }

        // Try title match (case-insensitive)
        let target_lower = target.to_lowercase();
        if let Some(note) = notes.iter().find(|n| {
            n.title.as_ref().map(|t| t.to_lowercase() == target_lower).unwrap_or(false)
        }) {
            return Some((note.id, note.path.clone()));
        }

        // Try filename without extension match
        let target_name = target.strip_suffix(".md").unwrap_or(target);
        if let Some(note) = notes.iter().find(|n| {
            let note_name = n.path
                .rsplit('/')
                .next()
                .unwrap_or(&n.path)
                .strip_suffix(".md")
                .unwrap_or(&n.path);
            note_name.eq_ignore_ascii_case(target_name)
        }) {
            return Some((note.id, note.path.clone()));
        }

        None
    }

    /// Resolve an asset path (image, etc.) to its full filesystem path.
    /// Searches the vault directory for the file.
    pub async fn resolve_asset_path(&self, target: &str) -> Option<PathBuf> {
        let target_path = Path::new(target);

        // If target is an absolute path within the vault, use it directly
        let direct_path = self.fs.to_absolute(target_path);
        if direct_path.exists() {
            return Some(direct_path);
        }

        // Search for the file in the vault
        // For now, just do a simple recursive search
        if let Ok(found) = self.find_asset_recursive(self.fs.root(), target).await {
            return found;
        }

        None
    }

    /// Recursively search for an asset file.
    async fn find_asset_recursive(&self, dir: &Path, target: &str) -> std::io::Result<Option<PathBuf>> {
        let target_name = Path::new(target)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or(target);

        let mut entries = tokio::fs::read_dir(dir).await?;

        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

            // Skip hidden files/directories
            if file_name.starts_with('.') {
                continue;
            }

            if path.is_dir() {
                // Recurse into subdirectory
                if let Ok(Some(found)) = Box::pin(self.find_asset_recursive(&path, target)).await {
                    return Ok(Some(found));
                }
            } else if file_name == target_name || path.ends_with(target) {
                return Ok(Some(path));
            }
        }

        Ok(None)
    }
}
