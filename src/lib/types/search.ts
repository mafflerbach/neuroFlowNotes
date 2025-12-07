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
