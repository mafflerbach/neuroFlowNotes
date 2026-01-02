/**
 * Search API - search, hybrid search, and embeddings
 */

import { invoke } from "@tauri-apps/api/core";
import type {
  SearchResult,
  BacklinkDto,
  HybridSearchResult,
  EmbeddingSettings,
  EmbeddingStatus,
} from "../../types";

/** Search notes using FTS5. */
export async function searchNotes(query: string, limit?: number): Promise<SearchResult[]> {
  return invoke<SearchResult[]>("search_notes", { query, limit });
}

/** Search notes using hybrid FTS5 + vector search. */
export async function hybridSearchNotes(
  query: string,
  options?: {
    limit?: number;
    useSemantic?: boolean;
    settings?: EmbeddingSettings;
  }
): Promise<HybridSearchResult[]> {
  return invoke<HybridSearchResult[]>("hybrid_search_notes", {
    query,
    limit: options?.limit,
    useSemantic: options?.useSemantic,
    settings: options?.settings,
  });
}

/** Get embedding service status. */
export async function getEmbeddingStatus(
  settings: EmbeddingSettings
): Promise<EmbeddingStatus> {
  return invoke<EmbeddingStatus>("get_embedding_status", { settings });
}

/** Test embedding service connection. */
export async function testEmbeddingConnection(
  settings: EmbeddingSettings
): Promise<boolean> {
  return invoke<boolean>("test_embedding_connection", { settings });
}

export async function getBacklinks(noteId: number): Promise<BacklinkDto[]> {
  return invoke<BacklinkDto[]>("get_backlinks", { noteId });
}

/** Generate embedding for a single note. */
export async function generateNoteEmbedding(
  noteId: number,
  settings: EmbeddingSettings
): Promise<boolean> {
  return invoke<boolean>("generate_note_embedding", { noteId, settings });
}

/** Get list of notes needing embeddings. */
export async function getNotesNeedingEmbeddings(
  limit: number
): Promise<[number, string][]> {
  return invoke<[number, string][]>("get_notes_needing_embeddings", { limit });
}
