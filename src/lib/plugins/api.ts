/**
 * Plugin API - Frontend wrappers for plugin backend commands.
 */

import { invoke } from "@tauri-apps/api/core";
import type { BackendHooks, HttpRequestOptions, HttpResponse, ScheduleBlock, NoteContent, NoteListItem } from "./types";

// =============================================================================
// Plugin Config API
// =============================================================================

export async function readPluginConfig<T>(pluginId: string): Promise<T | null> {
  const result = await invoke<T | null>("read_plugin_config", { pluginId });
  return result;
}

export async function writePluginConfig<T>(pluginId: string, config: T): Promise<void> {
  await invoke("write_plugin_config", { pluginId, config });
}

export async function listPluginConfigs(): Promise<string[]> {
  return invoke<string[]>("list_plugin_configs");
}

// =============================================================================
// HTTP Client API
// =============================================================================

export async function pluginHttpRequest(options: HttpRequestOptions): Promise<HttpResponse> {
  return invoke<HttpResponse>("plugin_http_request", { options });
}

// =============================================================================
// Note & Schedule API (existing commands wrapped for plugins)
// =============================================================================

async function getScheduleBlocks(startDate: string, endDate: string): Promise<ScheduleBlock[]> {
  return invoke<ScheduleBlock[]>("get_schedule_blocks", { startDate, endDate });
}

async function getNoteContent(noteId: number): Promise<NoteContent | null> {
  try {
    // First get the note to get its path
    const note = await invoke<{ id: number; path: string; title: string | null }>(
      "get_note",
      { noteId }
    );

    // Then get the content using the path
    const content = await invoke<{ id: number; path: string; title: string | null; content: string }>(
      "get_note_content",
      { path: note.path }
    );

    return {
      id: content.id,
      path: content.path,
      title: content.title,
      content: content.content,
    };
  } catch {
    return null;
  }
}

async function getNoteByPath(path: string): Promise<NoteContent | null> {
  try {
    const result = await invoke<{ id: number; path: string; title: string | null; content: string }>(
      "get_note_content",
      { path }
    );
    return {
      id: result.id,
      path: result.path,
      title: result.title,
      content: result.content,
    };
  } catch {
    return null;
  }
}

async function listNotes(): Promise<NoteListItem[]> {
  return invoke<NoteListItem[]>("list_notes");
}

// =============================================================================
// Backend Hooks Factory
// =============================================================================

/**
 * Create backend hooks object for the plugin registry.
 * This provides a unified interface for plugins to access backend functionality.
 */
export function createBackendHooks(): BackendHooks {
  return {
    getScheduleBlocks,
    getNoteContent,
    getNoteByPath,
    listNotes,
    httpRequest: pluginHttpRequest,
    readPluginConfig,
    writePluginConfig,
  };
}
