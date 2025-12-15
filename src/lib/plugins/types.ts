/**
 * Plugin System Types
 *
 * Defines the interfaces and types for the NeuroFlow plugin system.
 * Plugins are frontend-only but can use backend hooks for data access.
 */

import type { Component } from "svelte";

// =============================================================================
// Settings Schema Types
// =============================================================================

export type SettingType = "string" | "number" | "boolean" | "select" | "textarea" | "password";

export interface SettingOption {
  value: string;
  label: string;
}

export interface SettingField {
  key: string;
  label: string;
  description?: string;
  type: SettingType;
  default: unknown;
  required?: boolean;
  options?: SettingOption[]; // For 'select' type
  placeholder?: string;
  min?: number; // For 'number' type
  max?: number;
}

export interface SettingsSection {
  title: string;
  description?: string;
  fields: SettingField[];
}

// =============================================================================
// Plugin Hooks - Integration Points
// =============================================================================

export interface CalendarHook {
  /** Button to show in calendar toolbar */
  toolbarAction?: {
    label: string;
    icon?: string;
    onClick: (selectedDate: string) => void | Promise<void>;
  };
  /** Context menu items for schedule blocks */
  blockContextMenu?: {
    label: string;
    onClick: (blockId: number, noteId: number | null) => void | Promise<void>;
  }[];
}

export interface SidebarHook {
  /** Panel to add to sidebar */
  panel?: {
    id: string;
    label: string;
    icon?: string;
    component: Component;
  };
}

export interface EditorHook {
  /** Context menu items for editor */
  contextMenu?: {
    label: string;
    onClick: (selectedText: string, notePath: string) => void | Promise<void>;
  }[];
}

export interface CommandHook {
  /** Commands that can be triggered via command palette or shortcuts */
  commands?: {
    id: string;
    label: string;
    shortcut?: string;
    execute: () => void | Promise<void>;
  }[];
}

// =============================================================================
// Plugin Definition
// =============================================================================

export interface PluginMeta {
  id: string;
  name: string;
  description: string;
  version: string;
  author?: string;
}

export interface PluginConfig<T = Record<string, unknown>> {
  enabled: boolean;
  settings: T;
}

export interface Plugin<TSettings = Record<string, unknown>> {
  /** Plugin metadata */
  meta: PluginMeta;

  /** Settings schema for auto-generating settings UI */
  settingsSchema: SettingsSection[];

  /** Default settings values */
  defaultSettings: TSettings;

  /** Lifecycle: Called when plugin is enabled */
  onEnable?: (settings: TSettings) => Promise<void>;

  /** Lifecycle: Called when plugin is disabled */
  onDisable?: () => Promise<void>;

  /** Lifecycle: Called when settings change */
  onSettingsChange?: (settings: TSettings) => Promise<void>;

  /** Integration hooks */
  hooks?: {
    calendar?: CalendarHook;
    sidebar?: SidebarHook;
    editor?: EditorHook;
    commands?: CommandHook;
  };

  /** Custom settings panel component (optional, overrides auto-generated) */
  SettingsPanel?: Component<{ settings: TSettings; onUpdate: (settings: TSettings) => void }>;
}

// =============================================================================
// Plugin Registry Types
// =============================================================================

export interface RegisteredPlugin<TSettings = Record<string, unknown>> {
  plugin: Plugin<TSettings>;
  config: PluginConfig<TSettings>;
}

export interface PluginRegistry {
  plugins: Map<string, RegisteredPlugin>;
  register: <T>(plugin: Plugin<T>) => void;
  unregister: (pluginId: string) => void;
  get: <T>(pluginId: string) => RegisteredPlugin<T> | undefined;
  getAll: () => RegisteredPlugin[];
  getEnabled: () => RegisteredPlugin[];
  enable: (pluginId: string) => Promise<void>;
  disable: (pluginId: string) => Promise<void>;
  updateSettings: <T>(pluginId: string, settings: T) => Promise<void>;
}

// =============================================================================
// Backend Hook Types (what plugins can access)
// =============================================================================

export interface ScheduleBlock {
  id: number;
  note_id: number | null;
  date: string;
  start_time: string;
  end_time: string;
  label: string | null;
  color: string | null;
  context: string | null;
}

export interface NoteContent {
  id: number;
  path: string;
  title: string | null;
  content: string;
}

/** Note list item (for file picker) */
export interface NoteListItem {
  id: number;
  path: string;
  title: string | null;
  pinned: boolean;
}

/** Backend hooks available to plugins */
export interface BackendHooks {
  /** Get schedule blocks for a date range */
  getScheduleBlocks: (startDate: string, endDate: string) => Promise<ScheduleBlock[]>;

  /** Get note content by ID */
  getNoteContent: (noteId: number) => Promise<NoteContent | null>;

  /** Get note content by path */
  getNoteByPath: (path: string) => Promise<NoteContent | null>;

  /** List all notes in the vault */
  listNotes: () => Promise<NoteListItem[]>;

  /** Make an HTTP request (for LLM APIs, etc.) */
  httpRequest: (options: HttpRequestOptions) => Promise<HttpResponse>;

  /** Read plugin config file */
  readPluginConfig: <T>(pluginId: string) => Promise<T | null>;

  /** Write plugin config file */
  writePluginConfig: <T>(pluginId: string, config: T) => Promise<void>;
}

export interface HttpRequestOptions {
  url: string;
  method: "GET" | "POST" | "PUT" | "DELETE";
  headers?: Record<string, string>;
  body?: unknown;
  timeout?: number;
}

export interface HttpResponse {
  status: number;
  headers: Record<string, string>;
  body: unknown;
}
