/**
 * Plugin System
 *
 * Main entry point for the NeuroFlow plugin system.
 */

// Types
export type {
  Plugin,
  PluginMeta,
  PluginConfig,
  RegisteredPlugin,
  SettingField,
  SettingsSection,
  SettingType,
  SettingOption,
  CalendarHook,
  SidebarHook,
  EditorHook,
  CommandHook,
  BackendHooks,
  HttpRequestOptions,
  HttpResponse,
  ScheduleBlock,
  NoteContent,
  NoteListItem,
} from "./types";

// Registry
import { pluginRegistry, useBackendHooks } from "./registry.svelte";
export { pluginRegistry, useBackendHooks };

// API
export { createBackendHooks, readPluginConfig, writePluginConfig, pluginHttpRequest } from "./api";

// Built-in plugins
import { llmFileSummarizerPlugin } from "./builtin/llm-file-summarizer";
import { llmDailySummarizerPlugin } from "./builtin/llm-daily-summarizer";
// Register built-in plugins
pluginRegistry.register(llmFileSummarizerPlugin);
pluginRegistry.register(llmDailySummarizerPlugin);
