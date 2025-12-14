/**
 * Metadata types - todos, tags, backlinks, properties, tasks
 */

export interface TodoDto {
  id: number;
  note_id: number;
  line_number: number | null;
  description: string;
  completed: boolean;
  heading_path: string | null;
  /** GTD context (e.g., "home", "work", "phone", "computer"). */
  context: string | null;
  /** Priority level ("high", "medium", "low"). */
  priority: string | null;
  /** Due date as YYYY-MM-DD string. */
  due_date: string | null;
  created_at: string | null;
  completed_at: string | null;
}

/** A task (todo) with enriched context from its parent note. */
export interface TaskWithContext {
  todo: TodoDto;
  note_path: string;
  note_title: string | null;
  note_properties: PropertyDto[];
}

/** Query parameters for filtering tasks. */
export interface TaskQuery {
  /** Filter by completion status (null = all, true = completed, false = incomplete). */
  completed?: boolean | null;
  /** Filter by context (e.g., "home", "work"). */
  context?: string | null;
  /** Filter by priority ("high", "medium", "low"). */
  priority?: string | null;
  /** Filter by due date range start (inclusive, YYYY-MM-DD). */
  due_from?: string | null;
  /** Filter by due date range end (inclusive, YYYY-MM-DD). */
  due_to?: string | null;
  /** Filter by note property (key=value). */
  property_filter?: string | null;
  /** Maximum number of results. */
  limit?: number | null;
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

// ============================================================================
// Property Management Types
// ============================================================================

/** Request to rename a property key across all notes. */
export interface RenamePropertyKeyRequest {
  old_key: string;
  new_key: string;
}

/** Request to rename a property value across all notes with that key. */
export interface RenamePropertyValueRequest {
  key: string;
  old_value: string;
  new_value: string;
}

/** Request to merge two property keys. */
export interface MergePropertyKeysRequest {
  source_key: string;
  target_key: string;
}

/** Request to delete a property key from all notes. */
export interface DeletePropertyKeyRequest {
  key: string;
}

/** Response for bulk property operations. */
export interface PropertyOperationResult {
  affected_count: number;
  notes_affected: number;
}

/** Information about a property value used in the vault. */
export interface PropertyValueInfo {
  value: string;
  usage_count: number;
}

/** A note that uses a specific property. */
export interface NoteWithPropertyValue {
  note_id: number;
  path: string;
  title: string | null;
  value: string | null;
}

// ============================================================================
// Folder Property Types
// ============================================================================

/** A key-value property for a folder (inherited by notes in that folder tree). */
export interface FolderPropertyDto {
  id: number;
  folder_path: string;
  key: string;
  value: string | null;
  property_type: string | null;
}

/** Request to set a folder property value. */
export interface SetFolderPropertyRequest {
  folder_path: string;
  key: string;
  value: string | null;
  property_type: string | null;
}

/** A property with inheritance information. */
export interface PropertyWithInheritance {
  id: number;
  key: string;
  value: string | null;
  property_type: string | null;
  sort_order: number | null;
  /** True if this property is inherited from a folder (not directly set on the note). */
  inherited: boolean;
  /** The folder path this property is inherited from (if inherited). */
  inherited_from: string | null;
}
