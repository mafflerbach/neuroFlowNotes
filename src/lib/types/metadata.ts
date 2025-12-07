/**
 * Metadata types - todos, tags, backlinks, properties
 */

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

export interface TagDto {
  tag: string;
  count: number;
}

export interface BacklinkDto {
  from_note_id: number;
  from_note_path: string;
  from_note_title: string | null;
}

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
