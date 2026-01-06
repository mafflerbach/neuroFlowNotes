/**
 * Templates API - daily note creation and template settings
 */

import { invoke } from "@tauri-apps/api/core";
import type { TemplateSettings, DailyNoteResult } from "../../types";

// ============================================================================
// Template Settings
// ============================================================================

/**
 * Get template settings from vault config.
 */
export async function getTemplateSettings(): Promise<TemplateSettings> {
  return invoke<TemplateSettings>("get_template_settings");
}

/**
 * Save template settings to vault config.
 */
export async function saveTemplateSettings(settings: TemplateSettings): Promise<void> {
  return invoke("save_template_settings", { settings });
}

// ============================================================================
// Template Files
// ============================================================================

/**
 * List all template files in the templates/ folder.
 * Returns paths relative to vault root (e.g., "templates/daily.md").
 */
export async function listTemplates(): Promise<string[]> {
  return invoke<string[]>("list_templates");
}

/**
 * Create a new note from a template file.
 * Template variables ({{date}}, {{week}}, etc.) will be automatically substituted.
 *
 * @param targetPath - Path where the note should be created (e.g., "work/tickets/PROJ-123.md")
 * @param templatePath - Path to template file (e.g., "templates/ticket.md")
 * @returns The ID of the newly created note
 */
export async function createNoteFromTemplate(
  targetPath: string,
  templatePath: string
): Promise<number> {
  return invoke<number>("create_note_from_template", { targetPath, templatePath });
}

// ============================================================================
// Daily Notes
// ============================================================================

/**
 * Create or open a daily note for the given date.
 * If the note already exists, returns its info without modifying it.
 *
 * @param date - Date in YYYY-MM-DD format
 * @returns The daily note result
 */
export async function createDailyNote(date: string): Promise<DailyNoteResult> {
  return invoke<DailyNoteResult>("create_daily_note", { date });
}

/**
 * Preview the daily note path for a given date pattern.
 * Useful for settings UI to show what path will be generated.
 *
 * @param pattern - The path pattern (e.g., "journal/{{year}}/{{date}}.md")
 * @param date - Date in YYYY-MM-DD format
 * @returns The rendered path
 */
export async function previewDailyNotePath(pattern: string, date: string): Promise<string> {
  return invoke<string>("preview_daily_note_path", { pattern, date });
}
