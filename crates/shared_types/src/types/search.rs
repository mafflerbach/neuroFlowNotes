//! Search types.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// A search result.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct SearchResult {
    pub note_id: i64,
    pub path: String,
    pub title: Option<String>,
    pub snippet: Option<String>,
    pub score: f64,
}

/// Search query parameters.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct SearchQuery {
    pub query: String,
    pub limit: Option<i32>,
    pub offset: Option<i32>,
}

/// How a result was matched in hybrid search.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub enum MatchType {
    /// Matched via keyword search (FTS5) only.
    Keyword,
    /// Matched via semantic search (vector) only.
    Semantic,
    /// Matched in both keyword and semantic search.
    Both,
}

/// A hybrid search result combining FTS5 and vector search.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct HybridSearchResult {
    pub note_id: i64,
    pub path: String,
    pub title: Option<String>,
    pub snippet: Option<String>,
    /// BM25 score from FTS5 search (if matched).
    pub fts_score: Option<f64>,
    /// Cosine similarity from vector search (if matched).
    pub vector_score: Option<f64>,
    /// Combined score using Reciprocal Rank Fusion.
    pub combined_score: f64,
    /// How this result was matched.
    pub match_type: MatchType,
}

/// Options for hybrid search.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct HybridSearchOptions {
    /// Whether to use semantic search (requires embeddings enabled).
    pub use_semantic: bool,
    /// Weight for FTS results in combined score (0.0 - 1.0).
    pub fts_weight: f64,
    /// Maximum results to return.
    pub limit: i32,
}
