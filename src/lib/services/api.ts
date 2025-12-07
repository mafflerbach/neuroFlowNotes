/**
 * API service layer - wraps Tauri commands for the frontend.
 * Note: Tauri 2 automatically converts Rust snake_case params to camelCase for frontend.
 */

import { invoke } from "@tauri-apps/api/core";
import type {
  VaultInfo,
  NoteListItem,
  NoteDto,
  NoteContent,
  TodoDto,
  TagDto,
  BacklinkDto,
  SearchResult,
  FolderNode,
  PropertyDto,
  SetPropertyRequest,
  ScheduleBlockDto,
  CreateScheduleBlockRequest,
  UpdateScheduleBlockRequest,
  NoteForDate,
} from "../types";

// ============================================================================
// Vault
// ============================================================================

export async function openVault(path: string): Promise<VaultInfo> {
  return invoke<VaultInfo>("open_vault", { path });
}

export async function closeVault(): Promise<void> {
  return invoke("close_vault");
}

export async function getVaultInfo(): Promise<VaultInfo | null> {
  return invoke<VaultInfo | null>("get_vault_info");
}

// ============================================================================
// Notes
// ============================================================================

export async function listNotes(): Promise<NoteListItem[]> {
  return invoke<NoteListItem[]>("list_notes");
}

export async function getNote(noteId: number): Promise<NoteDto> {
  return invoke<NoteDto>("get_note", { noteId });
}

export async function getNoteContent(path: string): Promise<NoteContent> {
  return invoke<NoteContent>("get_note_content", { path });
}

export async function saveNote(path: string, content: string): Promise<number> {
  return invoke<number>("save_note", { path, content });
}

export async function renameNote(oldPath: string, newPath: string): Promise<number> {
  return invoke<number>("rename_note", { oldPath, newPath });
}

// ============================================================================
// Todos
// ============================================================================

export async function getTodosForNote(noteId: number): Promise<TodoDto[]> {
  return invoke<TodoDto[]>("get_todos_for_note", { noteId });
}

export async function toggleTodo(todoId: number, completed: boolean): Promise<void> {
  return invoke("toggle_todo", { todoId, completed });
}

export async function getIncompleteTodos(): Promise<TodoDto[]> {
  return invoke<TodoDto[]>("get_incomplete_todos");
}

// ============================================================================
// Tags
// ============================================================================

export async function listTags(): Promise<TagDto[]> {
  return invoke<TagDto[]>("list_tags");
}

// ============================================================================
// Backlinks
// ============================================================================

export async function getBacklinks(noteId: number): Promise<BacklinkDto[]> {
  return invoke<BacklinkDto[]>("get_backlinks", { noteId });
}

// ============================================================================
// Search
// ============================================================================

export async function searchNotes(query: string, limit?: number): Promise<SearchResult[]> {
  return invoke<SearchResult[]>("search_notes", { query, limit });
}

// ============================================================================
// Folder Tree
// ============================================================================

export async function getFolderTree(): Promise<FolderNode> {
  return invoke<FolderNode>("get_folder_tree");
}

// ============================================================================
// Properties
// ============================================================================

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
// Schedule Blocks
// ============================================================================

export async function createScheduleBlock(request: CreateScheduleBlockRequest): Promise<number> {
  return invoke<number>("create_schedule_block", { request });
}

export async function getScheduleBlocks(startDate: string, endDate: string): Promise<ScheduleBlockDto[]> {
  return invoke<ScheduleBlockDto[]>("get_schedule_blocks", { startDate, endDate });
}

export async function getScheduleBlocksForDate(date: string): Promise<ScheduleBlockDto[]> {
  return invoke<ScheduleBlockDto[]>("get_schedule_blocks_for_date", { date });
}

export async function updateScheduleBlock(request: UpdateScheduleBlockRequest): Promise<void> {
  return invoke("update_schedule_block", { request });
}

export async function getScheduleBlocksForNote(noteId: number): Promise<ScheduleBlockDto[]> {
  return invoke<ScheduleBlockDto[]>("get_schedule_blocks_for_note", { noteId });
}

export async function deleteScheduleBlock(id: number): Promise<void> {
  return invoke("delete_schedule_block", { id });
}

// ============================================================================
// Notes by Date
// ============================================================================

export async function getNotesForDate(date: string): Promise<NoteForDate[]> {
  return invoke<NoteForDate[]>("get_notes_for_date", { date });
}

export async function getNotesForDateRange(
  startDate: string,
  endDate: string
): Promise<[string, NoteForDate[]][]> {
  return invoke<[string, NoteForDate[]][]>("get_notes_for_date_range", { startDate, endDate });
}
