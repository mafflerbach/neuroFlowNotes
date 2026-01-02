//! API types for LM Studio embedding requests.

use serde::{Deserialize, Serialize};

/// Request to the embeddings API.
#[derive(Debug, Serialize)]
pub struct EmbeddingRequest {
    /// Model name for embeddings.
    pub model: String,
    /// Input text(s) to embed.
    pub input: Vec<String>,
    /// Encoding format (always "float").
    pub encoding_format: String,
}

/// Response from the embeddings API.
#[derive(Debug, Deserialize)]
pub struct EmbeddingResponse {
    /// List of embedding data.
    pub data: Vec<EmbeddingData>,
    /// Model used for embeddings.
    pub model: String,
    /// Usage statistics (optional).
    pub usage: Option<EmbeddingUsage>,
}

/// Individual embedding in the response.
#[derive(Debug, Deserialize)]
pub struct EmbeddingData {
    /// The embedding vector.
    pub embedding: Vec<f32>,
    /// Index in the input array.
    pub index: usize,
    /// Object type (always "embedding").
    pub object: String,
}

/// Token usage statistics.
#[derive(Debug, Deserialize)]
pub struct EmbeddingUsage {
    /// Number of prompt tokens.
    pub prompt_tokens: i32,
    /// Total tokens used.
    pub total_tokens: i32,
}

/// Error from embedding operations.
#[derive(Debug, thiserror::Error)]
pub enum EmbeddingError {
    #[error("HTTP request failed: {0}")]
    Request(#[from] reqwest::Error),

    #[error("API error: {message}")]
    Api { message: String },

    #[error("Service unavailable: {0}")]
    Unavailable(String),

    #[error("Invalid response: {0}")]
    InvalidResponse(String),

    #[error("Dimension mismatch: expected {expected}, got {actual}")]
    DimensionMismatch { expected: usize, actual: usize },
}
