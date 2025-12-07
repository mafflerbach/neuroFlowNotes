/**
 * Search API - search and backlinks
 */

import { invoke } from "@tauri-apps/api/core";
import type { SearchResult, BacklinkDto } from "../../types";

export async function searchNotes(query: string, limit?: number): Promise<SearchResult[]> {
  return invoke<SearchResult[]>("search_notes", { query, limit });
}

export async function getBacklinks(noteId: number): Promise<BacklinkDto[]> {
  return invoke<BacklinkDto[]>("get_backlinks", { noteId });
}
