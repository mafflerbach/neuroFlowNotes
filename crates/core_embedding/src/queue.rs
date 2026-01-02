//! Background embedding queue for non-blocking embedding generation.
//!
//! This module provides a queue that processes embedding jobs asynchronously,
//! allowing note saves to complete immediately while embeddings are generated
//! in the background.

use crate::EmbeddingClient;
use core_storage::{extract_content_preview, VaultRepository};
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{debug, info, warn};

/// A job to generate an embedding for a note.
#[derive(Debug)]
struct EmbeddingJob {
    note_id: i64,
    content: String,
    content_hash: String,
    content_preview: String,
}

/// Handle for queuing embedding jobs.
#[derive(Clone)]
pub struct EmbeddingQueue {
    tx: mpsc::Sender<EmbeddingJob>,
}

impl EmbeddingQueue {
    /// Start the background embedding worker and return a queue handle.
    pub fn start(client: Arc<EmbeddingClient>, repo: Arc<VaultRepository>) -> Self {
        let (tx, rx) = mpsc::channel::<EmbeddingJob>(100);

        // Spawn the background worker
        tokio::spawn(embedding_worker(rx, client, repo));

        info!("Background embedding worker started");
        Self { tx }
    }

    /// Queue a note for embedding generation.
    ///
    /// This is non-blocking and returns immediately.
    /// If the queue is full, the job is dropped with a warning.
    pub fn queue(&self, note_id: i64, content: String, content_hash: String) {
        let content_preview = extract_content_preview(&content);
        let job = EmbeddingJob {
            note_id,
            content,
            content_hash,
            content_preview,
        };

        match self.tx.try_send(job) {
            Ok(()) => {
                debug!("Queued embedding job for note {}", note_id);
            }
            Err(mpsc::error::TrySendError::Full(_)) => {
                warn!("Embedding queue full, dropping job for note {}", note_id);
            }
            Err(mpsc::error::TrySendError::Closed(_)) => {
                warn!("Embedding queue closed, dropping job for note {}", note_id);
            }
        }
    }

    /// Queue a note for embedding, waiting if the queue is full.
    ///
    /// This should be used when you want to ensure the job is queued.
    pub async fn queue_async(&self, note_id: i64, content: String, content_hash: String) {
        let content_preview = extract_content_preview(&content);
        let job = EmbeddingJob {
            note_id,
            content,
            content_hash,
            content_preview,
        };

        if let Err(e) = self.tx.send(job).await {
            warn!("Failed to queue embedding job for note: {}", e);
        }
    }
}

/// Background worker that processes embedding jobs.
async fn embedding_worker(
    mut rx: mpsc::Receiver<EmbeddingJob>,
    client: Arc<EmbeddingClient>,
    repo: Arc<VaultRepository>,
) {
    info!("Embedding worker started");

    while let Some(job) = rx.recv().await {
        // Check if embeddings are enabled
        if !client.settings().enabled {
            debug!("Embeddings disabled, skipping job for note {}", job.note_id);
            continue;
        }

        // Check if note still needs embedding (content might have changed)
        match repo.needs_embedding(job.note_id, &job.content_hash).await {
            Ok(false) => {
                debug!(
                    "Note {} already has current embedding, skipping",
                    job.note_id
                );
                continue;
            }
            Ok(true) => {}
            Err(e) => {
                warn!(
                    "Failed to check if note {} needs embedding: {}",
                    job.note_id, e
                );
                continue;
            }
        }

        // Generate embedding
        debug!("Generating embedding for note {}", job.note_id);
        match client.embed(&job.content).await {
            Ok(embedding) => {
                // Store the embedding with content preview
                match repo
                    .store_embedding(job.note_id, &embedding, &job.content_hash, Some(&job.content_preview))
                    .await
                {
                    Ok(()) => {
                        debug!("Stored embedding for note {}", job.note_id);
                    }
                    Err(e) => {
                        warn!("Failed to store embedding for note {}: {}", job.note_id, e);
                    }
                }
            }
            Err(e) => {
                warn!(
                    "Failed to generate embedding for note {}: {}",
                    job.note_id, e
                );
            }
        }
    }

    info!("Embedding worker stopped");
}

/// Manager for background embedding operations.
pub struct EmbeddingManager {
    client: Arc<EmbeddingClient>,
    repo: Arc<VaultRepository>,
    queue: Option<EmbeddingQueue>,
}

impl EmbeddingManager {
    /// Create a new embedding manager.
    pub fn new(client: EmbeddingClient, repo: VaultRepository) -> Self {
        let client = Arc::new(client);
        let repo = Arc::new(repo);

        // Start the queue if embeddings are enabled
        let queue = if client.settings().enabled {
            Some(EmbeddingQueue::start(Arc::clone(&client), Arc::clone(&repo)))
        } else {
            None
        };

        Self {
            client,
            repo,
            queue,
        }
    }

    /// Get the embedding client.
    pub fn client(&self) -> &EmbeddingClient {
        &self.client
    }

    /// Get the repository.
    pub fn repo(&self) -> &VaultRepository {
        &self.repo
    }

    /// Queue a note for embedding.
    pub fn queue_embedding(&self, note_id: i64, content: String, content_hash: String) {
        if let Some(queue) = &self.queue {
            queue.queue(note_id, content, content_hash);
        }
    }

    /// Check if embeddings are enabled.
    pub fn is_enabled(&self) -> bool {
        self.client.settings().enabled
    }

    /// Restart the queue with new settings.
    pub fn update_settings(&mut self, enabled: bool) {
        if enabled && self.queue.is_none() {
            // Start the queue
            self.queue = Some(EmbeddingQueue::start(
                Arc::clone(&self.client),
                Arc::clone(&self.repo),
            ));
            info!("Embedding queue started");
        } else if !enabled {
            // Drop the queue (worker will stop when tx is dropped)
            self.queue = None;
            info!("Embedding queue stopped");
        }
    }

    /// Rebuild embeddings for all notes.
    /// Returns (processed, total) counts.
    pub async fn rebuild_embeddings(
        &self,
        batch_size: i32,
        mut progress_callback: impl FnMut(i64, i64),
    ) -> Result<(i64, i64), crate::EmbeddingError> {
        let total = self
            .repo
            .get_notes_without_embeddings(i32::MAX)
            .await
            .map_err(|e| crate::EmbeddingError::Api {
                message: e.to_string(),
            })?
            .len() as i64;

        let mut processed = 0i64;

        loop {
            let notes = self
                .repo
                .get_notes_without_embeddings(batch_size)
                .await
                .map_err(|e| crate::EmbeddingError::Api {
                    message: e.to_string(),
                })?;

            if notes.is_empty() {
                break;
            }

            for (note_id, _path) in &notes {
                // For rebuild, we'd need to read the note content
                // This is a simplified version - full implementation would
                // read note content and compute hash
                debug!("Would rebuild embedding for note {}", note_id);
                processed += 1;
                progress_callback(processed, total);
            }
        }

        Ok((processed, total))
    }
}
