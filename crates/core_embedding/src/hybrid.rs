//! Hybrid search combining FTS5 and vector search with Reciprocal Rank Fusion.

use core_storage::{VaultRepository, VectorSearchResult};
use shared_types::{HybridSearchResult, MatchType, SearchResult};
use std::collections::HashMap;
use tracing::debug;

use crate::EmbeddingClient;

/// RRF smoothing constant (commonly 60).
const RRF_K: f64 = 60.0;

/// Perform hybrid search combining FTS5 and vector similarity.
///
/// If vector search is unavailable (embeddings disabled or LM Studio down),
/// falls back to FTS5 only.
pub async fn hybrid_search(
    client: &EmbeddingClient,
    repo: &VaultRepository,
    query: &str,
    limit: i32,
    use_semantic: bool,
) -> Result<Vec<HybridSearchResult>, crate::EmbeddingError> {
    // Get FTS5 results
    let fts_results = repo
        .search(query, limit * 2)
        .await
        .map_err(|e| crate::EmbeddingError::Api {
            message: e.to_string(),
        })?;

    debug!("FTS5 search returned {} results", fts_results.len());

    // If semantic search is disabled or no FTS results, return FTS-only
    if !use_semantic || !client.settings().enabled {
        return Ok(fts_results
            .into_iter()
            .map(|r| HybridSearchResult {
                note_id: r.note_id,
                path: r.path,
                title: r.title,
                snippet: r.snippet,
                fts_score: Some(r.score),
                vector_score: None,
                combined_score: r.score,
                match_type: MatchType::Keyword,
            })
            .take(limit as usize)
            .collect());
    }

    // Generate query embedding
    let query_embedding = match client.embed(query).await {
        Ok(emb) => emb,
        Err(e) => {
            debug!(
                "Failed to generate query embedding, falling back to FTS: {}",
                e
            );
            return Ok(fts_results
                .into_iter()
                .map(|r| HybridSearchResult {
                    note_id: r.note_id,
                    path: r.path,
                    title: r.title,
                    snippet: r.snippet,
                    fts_score: Some(r.score),
                    vector_score: None,
                    combined_score: r.score,
                    match_type: MatchType::Keyword,
                })
                .take(limit as usize)
                .collect());
        }
    };

    // Get vector search results
    let vector_results = repo
        .vector_search(&query_embedding, limit * 2)
        .await
        .map_err(|e| crate::EmbeddingError::Api {
            message: e.to_string(),
        })?;

    debug!("Vector search returned {} results", vector_results.len());

    // Combine with RRF
    let combined = reciprocal_rank_fusion(fts_results, vector_results, limit);

    Ok(combined)
}

/// Combine FTS and vector results using Reciprocal Rank Fusion.
///
/// RRF score = 1/(k + rank_fts) + 1/(k + rank_vector)
/// where k is a smoothing constant (typically 60).
fn reciprocal_rank_fusion(
    fts_results: Vec<SearchResult>,
    vector_results: Vec<VectorSearchResult>,
    limit: i32,
) -> Vec<HybridSearchResult> {
    // Create maps for looking up results by note_id
    let mut result_map: HashMap<i64, HybridSearchResult> = HashMap::new();

    // Process FTS results with their ranks
    for (rank, result) in fts_results.into_iter().enumerate() {
        let rrf_score = 1.0 / (RRF_K + (rank + 1) as f64);

        result_map.insert(
            result.note_id,
            HybridSearchResult {
                note_id: result.note_id,
                path: result.path,
                title: result.title,
                snippet: result.snippet,
                fts_score: Some(result.score),
                vector_score: None,
                combined_score: rrf_score,
                match_type: MatchType::Keyword,
            },
        );
    }

    // Process vector results with their ranks
    for (rank, result) in vector_results.into_iter().enumerate() {
        let rrf_score = 1.0 / (RRF_K + (rank + 1) as f64);

        match result_map.get_mut(&result.note_id) {
            Some(existing) => {
                // Note exists in FTS results - add vector score
                existing.vector_score = Some(result.score);
                existing.combined_score += rrf_score;
                existing.match_type = MatchType::Both;
            }
            None => {
                // Note only in vector results - use content preview as snippet
                result_map.insert(
                    result.note_id,
                    HybridSearchResult {
                        note_id: result.note_id,
                        path: result.path,
                        title: result.title,
                        snippet: result.content_preview,
                        fts_score: None,
                        vector_score: Some(result.score),
                        combined_score: rrf_score,
                        match_type: MatchType::Semantic,
                    },
                );
            }
        }
    }

    // Sort by combined score (descending)
    let mut results: Vec<HybridSearchResult> = result_map.into_values().collect();
    results.sort_by(|a, b| {
        b.combined_score
            .partial_cmp(&a.combined_score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    // Take top N
    results.truncate(limit as usize);

    debug!("Hybrid search returned {} combined results", results.len());
    results
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rrf_scoring() {
        // Test RRF score calculation
        let fts_results = vec![
            SearchResult {
                note_id: 1,
                path: "a.md".to_string(),
                title: Some("A".to_string()),
                snippet: None,
                score: 10.0,
            },
            SearchResult {
                note_id: 2,
                path: "b.md".to_string(),
                title: Some("B".to_string()),
                snippet: None,
                score: 8.0,
            },
        ];

        let vector_results = vec![
            VectorSearchResult {
                note_id: 2,
                path: "b.md".to_string(),
                title: Some("B".to_string()),
                content_preview: Some("Preview of B".to_string()),
                score: 0.95,
            },
            VectorSearchResult {
                note_id: 3,
                path: "c.md".to_string(),
                title: Some("C".to_string()),
                content_preview: Some("Preview of C".to_string()),
                score: 0.90,
            },
        ];

        let combined = reciprocal_rank_fusion(fts_results, vector_results, 10);

        // Note 2 should be top (matched in both)
        assert_eq!(combined[0].note_id, 2);
        assert_eq!(combined[0].match_type, MatchType::Both);
        assert!(combined[0].fts_score.is_some());
        assert!(combined[0].vector_score.is_some());

        // Note 1 and 3 should follow
        assert!(combined.len() >= 3);
    }

    #[test]
    fn test_rrf_single_source() {
        // Test with only FTS results
        let fts_results = vec![SearchResult {
            note_id: 1,
            path: "a.md".to_string(),
            title: Some("A".to_string()),
            snippet: None,
            score: 10.0,
        }];

        let combined = reciprocal_rank_fusion(fts_results, vec![], 10);

        assert_eq!(combined.len(), 1);
        assert_eq!(combined[0].match_type, MatchType::Keyword);
    }
}
