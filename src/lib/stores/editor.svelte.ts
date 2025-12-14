/**
 * Editor store - manages the currently open note and its content.
 */

import type { NoteContent, TodoDto } from "../types";
import * as api from "../services/api";
import { extractH1Title, generatePathFromTitle, titleToFilename } from "../utils/docListUtils";
import { workspaceStore } from "./workspace.svelte";
import { vaultStore } from "./vault.svelte";
import { logger } from "../utils/logger";

// Autosave delay in milliseconds
const AUTOSAVE_DELAY = 1500;

class EditorStore {
  currentNote = $state<NoteContent | null>(null);
  todos = $state<TodoDto[]>([]);
  isLoading = $state(false);
  isDirty = $state(false);
  error = $state<string | null>(null);

  // Flag to prevent reload during save/rename operations
  private isSaving = false;

  // Autosave timer
  private autosaveTimer: ReturnType<typeof setTimeout> | null = null;

  // Callback to notify App.svelte to refresh calendar data
  onScheduleBlocksUpdated: (() => void) | null = null;

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
    // Don't reload if we're in the middle of a save/rename
    if (this.isSaving) return;

    // Don't reload if already open (check by path)
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
      logger.error("EditorStore", "Failed to refresh todos:", e);
    }
  }

  updateContent(content: string) {
    if (this.currentNote) {
      this.currentNote.content = content;
      this.isDirty = true;
      this.scheduleAutosave();
    }
  }

  /**
   * Schedule an autosave after a delay.
   * Resets the timer if called again before the delay expires.
   */
  private scheduleAutosave() {
    // Clear any existing timer
    if (this.autosaveTimer) {
      clearTimeout(this.autosaveTimer);
    }

    // Schedule a new save
    this.autosaveTimer = setTimeout(async () => {
      this.autosaveTimer = null;
      if (this.isDirty) {
        try {
          await this.save();
        } catch (e) {
          logger.error("EditorStore", "Autosave failed:", e);
        }
      }
    }, AUTOSAVE_DELAY);
  }

  /**
   * Cancel any pending autosave.
   */
  private cancelAutosave() {
    if (this.autosaveTimer) {
      clearTimeout(this.autosaveTimer);
      this.autosaveTimer = null;
    }
  }

  async save() {
    if (!this.currentNote || !this.isDirty) return;

    // Cancel any pending autosave since we're saving now
    this.cancelAutosave();

    // Note: Don't set isLoading = true here - that would hide the editor and destroy the container
    this.isSaving = true;
    this.error = null;

    try {
      await api.saveNote(this.currentNote.path, this.currentNote.content);
      this.isDirty = false;
      await this.refreshTodos();

      // Sync H1 title to linked schedule blocks and rename file if needed
      await this.syncTitleAndRename();
    } catch (e) {
      this.error = e instanceof Error ? e.message : String(e);
      throw e;
    } finally {
      this.isSaving = false;
    }
  }

  /**
   * Sync the current note's H1 title to schedule blocks and rename file if needed.
   */
  private async syncTitleAndRename() {
    if (!this.currentNote) return;

    const h1Title = extractH1Title(this.currentNote.content);
    if (!h1Title) return;

    try {
      // Check if filename needs to be updated based on H1
      const expectedFilename = titleToFilename(h1Title);
      const currentFilename = this.currentNote.path.split("/").pop() || "";

      if (currentFilename !== expectedFilename) {
        const oldPath = this.currentNote.path;
        const newPath = generatePathFromTitle(oldPath, h1Title);

        try {
          await api.renameNote(oldPath, newPath);
          // Update the current note's path in memory
          this.currentNote.path = newPath;
          // Update workspace store's breadcrumb
          workspaceStore.updateDocPath(oldPath, newPath, h1Title);
          // Refresh folder tree to show the renamed file
          await vaultStore.refreshFolderTree();
        } catch (e) {
          // File might already exist with that name - continue silently
        }
      }

      // Get schedule blocks linked to this note
      const blocks = await api.getScheduleBlocksForNote(this.currentNote.id);

      // Update any blocks where label differs from H1
      let blocksUpdated = false;
      for (const block of blocks) {
        if (block.label !== h1Title) {
          await api.updateScheduleBlock({
            id: block.id,
            note_id: block.note_id,
            date: null,
            start_time: null,
            end_time: null,
            label: h1Title,
            color: null,
            context: null,
            rrule: null,
          });
          blocksUpdated = true;
        }
      }

      // Update title property
      await api.setProperty({
        note_id: this.currentNote.id,
        key: "title",
        value: h1Title,
        property_type: "text",
      });

      // Notify App.svelte to refresh calendar data if blocks were updated
      if (blocksUpdated && this.onScheduleBlocksUpdated) {
        this.onScheduleBlocksUpdated();
      }
    } catch (e) {
      logger.error("EditorStore", "Failed to sync title:", e);
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
      logger.error("EditorStore", "Failed to toggle todo:", e);
    }
  }

  close() {
    this.cancelAutosave();
    this.currentNote = null;
    this.todos = [];
    this.isDirty = false;
    this.error = null;
  }
}

export const editorStore = new EditorStore();
