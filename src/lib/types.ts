// Types matching the Rust shared_types crate

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

export interface TodoDto {
  id: number;
  note_id: number;
  line_number: number | null;
  description: string;
  completed: boolean;
  heading_path: string | null;
  created_at: string | null;
  completed_at: string | null;
}

export interface ScheduleBlockDto {
  id: number;
  note_id: number | null;
  date: string;
  start_time: string;
  end_time: string;
  label: string | null;
  color: string | null;
  context: string | null;
}

export interface TagDto {
  tag: string;
  count: number;
}

export interface BacklinkDto {
  from_note_id: number;
  from_note_path: string;
  from_note_title: string | null;
}

export interface VaultInfo {
  path: string;
  name: string;
  note_count: number;
}

export interface SearchResult {
  note_id: number;
  path: string;
  title: string | null;
  snippet: string | null;
  score: number;
}

export interface FolderNode {
  name: string;
  path: string;
  is_dir: boolean;
  children: FolderNode[];
}

// Properties
export interface PropertyDto {
  id: number;
  note_id: number;
  key: string;
  value: string | null;
  property_type: string | null;
  sort_order: number | null;
}

export interface SetPropertyRequest {
  note_id: number;
  key: string;
  value: string | null;
  property_type: string | null;
}

// Schedule Block Requests
export interface CreateScheduleBlockRequest {
  note_id: number | null;
  date: string;
  start_time: string;
  end_time: string;
  label: string | null;
  color: string | null;
  context: string | null;
}

export interface UpdateScheduleBlockRequest {
  id: number;
  note_id: number | null;
  date: string | null;
  start_time: string | null;
  end_time: string | null;
  label: string | null;
  color: string | null;
  context: string | null;
}

// Notes by Date
export interface NoteForDate {
  note: NoteListItem;
  source: "scheduled" | "journal" | "created";
  schedule_block: ScheduleBlockDto | null;
}

// Event payloads
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
