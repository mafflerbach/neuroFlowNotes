/**
 * Note-related types
 */

export interface NoteDto {
  id: number;
  path: string;
  title: string | null;
  created_at: string | null;
  updated_at: string | null;
  pinned: boolean;
}

export interface NoteListItem {
  id: number;
  path: string;
  title: string | null;
  pinned: boolean;
}

export interface NoteContent {
  id: number;
  path: string;
  content: string;
}
