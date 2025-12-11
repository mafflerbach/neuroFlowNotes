//! File watcher for detecting changes to markdown files.

use crate::vault::VaultEvent;
use core_fs::{hash_content, VaultFs};
use core_index::markdown::parse;
use core_storage::VaultRepository;
use notify::{RecommendedWatcher, RecursiveMode};
use notify_debouncer_mini::{new_debouncer, DebouncedEventKind, Debouncer};
use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{broadcast, mpsc, Mutex};
use tracing::{debug, error, info, warn};

/// File watcher that monitors the vault for changes.
pub struct FileWatcher {
    /// The vault root path.
    root: PathBuf,
    /// Repository for database operations.
    repo: VaultRepository,
    /// Filesystem handle.
    fs: VaultFs,
    /// Event sender for vault events.
    event_tx: broadcast::Sender<VaultEvent>,
    /// Channel to stop the watcher.
    stop_tx: Option<mpsc::Sender<()>>,
    /// The debouncer (holds the watcher).
    debouncer: Arc<Mutex<Option<Debouncer<RecommendedWatcher>>>>,
}

impl FileWatcher {
    /// Create a new file watcher.
    pub fn new(
        root: PathBuf,
        repo: VaultRepository,
        fs: VaultFs,
        event_tx: broadcast::Sender<VaultEvent>,
    ) -> Result<Self, notify::Error> {
        Ok(Self {
            root,
            repo,
            fs,
            event_tx,
            stop_tx: None,
            debouncer: Arc::new(Mutex::new(None)),
        })
    }

    /// Start watching for file changes.
    pub async fn start(&self) {
        let (_stop_tx, mut stop_rx) = mpsc::channel::<()>(1);
        let (event_tx, mut event_rx) = mpsc::channel::<Vec<notify_debouncer_mini::DebouncedEvent>>(100);

        // Create the debouncer
        let debouncer_result = new_debouncer(
            Duration::from_millis(500),
            move |result: Result<Vec<notify_debouncer_mini::DebouncedEvent>, notify::Error>| {
                match result {
                    Ok(events) => {
                        let _ = event_tx.blocking_send(events);
                    }
                    Err(e) => {
                        error!("Watcher error: {}", e);
                    }
                }
            },
        );

        let mut debouncer = match debouncer_result {
            Ok(d) => d,
            Err(e) => {
                error!("Failed to create debouncer: {}", e);
                return;
            }
        };

        // Start watching the root directory
        if let Err(e) = debouncer.watcher().watch(&self.root, RecursiveMode::Recursive) {
            error!("Failed to watch directory: {}", e);
            return;
        }

        info!("File watcher started for {}", self.root.display());

        // Store the debouncer
        *self.debouncer.lock().await = Some(debouncer);

        // Clone what we need for the async task
        let repo = self.repo.clone();
        let fs = self.fs.clone();
        let vault_event_tx = self.event_tx.clone();
        let root = self.root.clone();

        // Spawn the event processing task
        tokio::spawn(async move {
            loop {
                tokio::select! {
                    Some(events) = event_rx.recv() => {
                        process_events(&root, &repo, &fs, &vault_event_tx, events).await;
                    }
                    _ = stop_rx.recv() => {
                        info!("File watcher stopping");
                        break;
                    }
                }
            }
        });
    }

    /// Stop the watcher.
    pub async fn stop(&self) {
        if let Some(tx) = &self.stop_tx {
            let _ = tx.send(()).await;
        }

        // Drop the debouncer to stop watching
        *self.debouncer.lock().await = None;
    }
}

/// Process a batch of file system events.
async fn process_events(
    _root: &PathBuf,
    repo: &VaultRepository,
    fs: &VaultFs,
    event_tx: &broadcast::Sender<VaultEvent>,
    events: Vec<notify_debouncer_mini::DebouncedEvent>,
) {
    let mut to_index: HashSet<PathBuf> = HashSet::new();
    let mut to_remove: HashSet<PathBuf> = HashSet::new();

    for event in events {
        let path = &event.path;

        // Skip non-markdown files
        if path.extension().and_then(|e| e.to_str()) != Some("md") {
            continue;
        }

        // Skip hidden files and .neuroflow directory
        let path_str = path.to_string_lossy();
        if path_str.contains("/.") || path_str.contains("\\.") {
            continue;
        }

        match event.kind {
            DebouncedEventKind::Any => {
                if path.exists() {
                    to_index.insert(path.clone());
                } else {
                    to_remove.insert(path.clone());
                }
            }
            DebouncedEventKind::AnyContinuous => {
                // Continuous events during debounce, ignore
            }
            _ => {
                // Handle any future variants
            }
        }
    }

    // Process removals
    let mut deleted_ids = Vec::new();
    for path in to_remove {
        if let Ok(relative) = fs.to_relative(&path) {
            let path_str = relative.to_string_lossy().to_string();
            match repo.delete_note(&path_str).await {
                Ok(Some(id)) => {
                    debug!("Removed from index: {}", path_str);
                    deleted_ids.push(id);
                }
                Ok(None) => {}
                Err(e) => {
                    warn!("Failed to remove {}: {}", path_str, e);
                }
            }
        }
    }

    if !deleted_ids.is_empty() {
        let _ = event_tx.send(VaultEvent::NotesDeleted(deleted_ids));
    }

    // Process additions/modifications
    let mut updated_ids = Vec::new();
    for path in to_index {
        if let Ok(relative) = fs.to_relative(&path) {
            let path_str = relative.to_string_lossy().to_string();

            // Read and check hash
            match fs.read_file(&relative).await {
                Ok(content) => {
                    let hash = hash_content(&content);

                    // Check if changed
                    let existing_hash = repo.get_note_hash(&path_str).await.ok().flatten();
                    if existing_hash.as_ref() == Some(&hash) {
                        continue;
                    }

                    // Parse and index
                    let analysis = parse(&content);
                    match repo.index_note(&path_str, &content, &hash, &analysis).await {
                        Ok(id) => {
                            debug!("Indexed: {}", path_str);
                            updated_ids.push(id);
                        }
                        Err(e) => {
                            warn!("Failed to index {}: {}", path_str, e);
                        }
                    }
                }
                Err(e) => {
                    warn!("Failed to read {}: {}", path_str, e);
                }
            }
        }
    }

    if !updated_ids.is_empty() {
        let _ = event_tx.send(VaultEvent::NotesUpdated(updated_ids));
    }
}
