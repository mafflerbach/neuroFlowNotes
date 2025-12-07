/**
 * Notes API - note CRUD operations
 */

import { invoke } from "@tauri-apps/api/core";
import type { NoteListItem, NoteDto, NoteContent } from "../../types";

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

export async function deleteNote(path: string): Promise<number | null> {
  return invoke<number | null>("delete_note", { path });
}
