//! Embedding types for semantic search.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// Settings for embedding generation via LM Studio.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct EmbeddingSettings {
    /// Whether semantic search is enabled.
    pub enabled: bool,
    /// LM Studio API endpoint URL (e.g., "http://localhost:1234/v1").
    pub endpoint_url: String,
    /// Model name for embeddings (e.g., "nomic-ai/nomic-embed-text-v1.5-GGUF").
    pub model: String,
    /// Embedding vector dimensions (e.g., 768 for nomic-embed-text).
    pub dimensions: i32,
    /// Number of texts to process in a single batch.
    pub batch_size: i32,
}

impl Default for EmbeddingSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            endpoint_url: "http://localhost:1234/v1".to_string(),
            model: "nomic-ai/nomic-embed-text-v1.5-GGUF".to_string(),
            dimensions: 768,
            batch_size: 10,
        }
    }
}

/// Status of the embedding service connection.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct EmbeddingStatus {
    /// Whether the embedding service is reachable.
    pub connected: bool,
    /// Error message if not connected.
    pub error: Option<String>,
    /// Number of notes with embeddings.
    pub indexed_count: i64,
    /// Total number of notes.
    pub total_count: i64,
}

/// Progress of embedding rebuild operation.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct EmbeddingProgress {
    /// Current number of notes processed.
    pub processed: i64,
    /// Total number of notes to process.
    pub total: i64,
    /// Whether the operation is complete.
    pub complete: bool,
    /// Error message if any.
    pub error: Option<String>,
}
