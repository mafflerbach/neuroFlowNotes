/**
 * Properties API - note property operations
 */

import { invoke } from "@tauri-apps/api/core";
import type {
  PropertyDto,
  SetPropertyRequest,
  RenamePropertyKeyRequest,
  RenamePropertyValueRequest,
  MergePropertyKeysRequest,
  DeletePropertyKeyRequest,
  PropertyOperationResult,
  PropertyValueInfo,
  NoteWithPropertyValue,
} from "../../types";

export async function getProperties(noteId: number): Promise<PropertyDto[]> {
  return invoke<PropertyDto[]>("get_properties", { noteId });
}

export async function setProperty(request: SetPropertyRequest): Promise<number> {
  return invoke<number>("set_property", { request });
}

export async function deleteProperty(noteId: number, key: string): Promise<void> {
  return invoke("delete_property", { noteId, key });
}

// ============================================================================
// Property Management (Bulk Operations)
// ============================================================================

/** Rename a property key across all notes. */
export async function renamePropertyKey(
  request: RenamePropertyKeyRequest
): Promise<PropertyOperationResult> {
  return invoke<PropertyOperationResult>("rename_property_key", { request });
}

/** Rename a property value across all notes with that key. */
export async function renamePropertyValue(
  request: RenamePropertyValueRequest
): Promise<PropertyOperationResult> {
  return invoke<PropertyOperationResult>("rename_property_value", { request });
}

/** Merge two property keys (rename source to target). */
export async function mergePropertyKeys(
  request: MergePropertyKeysRequest
): Promise<PropertyOperationResult> {
  return invoke<PropertyOperationResult>("merge_property_keys", { request });
}

/** Delete a property key from all notes. */
export async function deletePropertyKey(
  request: DeletePropertyKeyRequest
): Promise<PropertyOperationResult> {
  return invoke<PropertyOperationResult>("delete_property_key", { request });
}

/** Get all distinct values for a property key with usage counts. */
export async function getPropertyValuesWithCounts(
  key: string
): Promise<PropertyValueInfo[]> {
  return invoke<PropertyValueInfo[]>("get_property_values_with_counts", { key });
}

/** Get all notes that have a specific property key. */
export async function getNotesWithProperty(
  key: string
): Promise<NoteWithPropertyValue[]> {
  return invoke<NoteWithPropertyValue[]>("get_notes_with_property", { key });
}

/** Get all notes that have a specific property key and value. */
export async function getNotesWithPropertyValue(
  key: string,
  value: string
): Promise<NoteWithPropertyValue[]> {
  return invoke<NoteWithPropertyValue[]>("get_notes_with_property_value", { key, value });
}
