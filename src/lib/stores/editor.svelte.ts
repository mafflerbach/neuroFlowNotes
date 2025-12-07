/**
 * Editor store - manages the currently open note and its content.
 */

import type { NoteContent, TodoDto } from "../types";
import * as api from "../services/api";

class EditorStore {
  currentNote = $state<NoteContent | null>(null);
  todos = $state<TodoDto[]>([]);
  isLoading = $state(false);
  isDirty = $state(false);
  error = $state<string | null>(null);

  get isOpen() {
    return this.currentNote !== null;
  }

  get currentPath() {
    return this.currentNote?.path ?? null;
  }

  get currentId() {
    return this.currentNote?.id ?? null;
  }

  async openNote(path: string) {
    // Don't reload if already open
    if (this.currentPath === path) return;

    // Save current note if dirty
    if (this.isDirty && this.currentNote) {
      await this.save();
    }

    this.isLoading = true;
    this.error = null;

    try {
      this.currentNote = await api.getNoteContent(path);
      this.isDirty = false;
      await this.refreshTodos();
    } catch (e) {
      this.error = e instanceof Error ? e.message : String(e);
      throw e;
    } finally {
      this.isLoading = false;
    }
  }

  async refreshTodos() {
    if (!this.currentId) return;

    try {
      this.todos = await api.getTodosForNote(this.currentId);
    } catch (e) {
      console.error("Failed to refresh todos:", e);
    }
  }

  updateContent(content: string) {
    if (this.currentNote) {
      this.currentNote.content = content;
      this.isDirty = true;
    }
  }

  async save() {
    if (!this.currentNote || !this.isDirty) return;

    this.isLoading = true;
    this.error = null;

    try {
      await api.saveNote(this.currentNote.path, this.currentNote.content);
      this.isDirty = false;
      await this.refreshTodos();
    } catch (e) {
      this.error = e instanceof Error ? e.message : String(e);
      throw e;
    } finally {
      this.isLoading = false;
    }
  }

  async toggleTodo(todoId: number, completed: boolean) {
    try {
      await api.toggleTodo(todoId, completed);
      // Refresh the note content and todos after toggling
      if (this.currentPath) {
        this.currentNote = await api.getNoteContent(this.currentPath);
        await this.refreshTodos();
      }
    } catch (e) {
      console.error("Failed to toggle todo:", e);
    }
  }

  close() {
    this.currentNote = null;
    this.todos = [];
    this.isDirty = false;
    this.error = null;
  }
}

export const editorStore = new EditorStore();
