/**
 * Embedding types for semantic search
 */

/**
 * Settings for embedding generation via LM Studio.
 */
export interface EmbeddingSettings {
  /** Whether semantic search is enabled. */
  enabled: boolean;
  /** LM Studio API endpoint URL (e.g., "http://localhost:1234/v1"). */
  endpoint_url: string;
  /** Model name for embeddings (e.g., "nomic-ai/nomic-embed-text-v1.5-GGUF"). */
  model: string;
  /** Embedding vector dimensions (e.g., 768 for nomic-embed-text). */
  dimensions: number;
  /** Number of texts to process in a single batch. */
  batch_size: number;
}

/**
 * Status of the embedding service connection.
 */
export interface EmbeddingStatus {
  /** Whether the embedding service is reachable. */
  connected: boolean;
  /** Error message if not connected. */
  error: string | null;
  /** Number of notes with embeddings. */
  indexed_count: number;
  /** Total number of notes. */
  total_count: number;
}

/**
 * Progress of embedding rebuild operation.
 */
export interface EmbeddingProgress {
  /** Current number of notes processed. */
  processed: number;
  /** Total number of notes to process. */
  total: number;
  /** Whether the operation is complete. */
  complete: boolean;
  /** Error message if any. */
  error: string | null;
}

/**
 * Default embedding settings.
 */
export const DEFAULT_EMBEDDING_SETTINGS: EmbeddingSettings = {
  enabled: false,
  endpoint_url: "http://localhost:1234/v1",
  model: "nomic-ai/nomic-embed-text-v1.5-GGUF",
  dimensions: 768,
  batch_size: 10,
};
