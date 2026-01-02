/**
 * Template types - daily note creation and template settings
 */

/**
 * Settings for template system (stored in vault config).
 */
export interface TemplateSettings {
  /** Path to the daily note template file (relative to vault root, e.g., "templates/daily.md"). */
  daily_template_path: string | null;
  /** Pattern for daily note file paths (e.g., "journal/{{year}}/{{month}}/{{date}}.md"). */
  daily_note_pattern: string;
}

/**
 * Result of creating a daily note.
 */
export interface DailyNoteResult {
  /** The note ID. */
  id: number;
  /** The note path. */
  path: string;
  /** The note title. */
  title: string | null;
  /** Whether the note was newly created (true) or already existed (false). */
  created: boolean;
}

/**
 * Default template settings.
 */
export const DEFAULT_TEMPLATE_SETTINGS: TemplateSettings = {
  daily_template_path: null,
  daily_note_pattern: "journal/{{year}}/{{month}}/{{date}}.md",
};
