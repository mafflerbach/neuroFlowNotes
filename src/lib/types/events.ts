/**
 * Event payload types for Tauri events
 */

export interface NotesUpdatedPayload {
  note_ids: number[];
}

export interface NotesDeletedPayload {
  note_ids: number[];
}

export interface IndexCompletePayload {
  notes_indexed: number;
  duration_ms: number;
}
