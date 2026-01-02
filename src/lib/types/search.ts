/**
 * Search-related types
 */

export interface SearchResult {
  note_id: number;
  path: string;
  title: string | null;
  snippet: string | null;
  score: number;
}

/**
 * How a result was matched in hybrid search.
 */
export type MatchType = "Keyword" | "Semantic" | "Both";

/**
 * A hybrid search result combining FTS5 and vector search.
 */
export interface HybridSearchResult {
  note_id: number;
  path: string;
  title: string | null;
  snippet: string | null;
  /** BM25 score from FTS5 search (if matched). */
  fts_score: number | null;
  /** Cosine similarity from vector search (if matched). */
  vector_score: number | null;
  /** Combined score using Reciprocal Rank Fusion. */
  combined_score: number;
  /** How this result was matched. */
  match_type: MatchType;
}

/**
 * Options for hybrid search.
 */
export interface HybridSearchOptions {
  /** Whether to use semantic search (requires embeddings enabled). */
  use_semantic: boolean;
  /** Weight for FTS results in combined score (0.0 - 1.0). */
  fts_weight: number;
  /** Maximum results to return. */
  limit: number;
}
