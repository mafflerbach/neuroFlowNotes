/**
 * Import types - for Obsidian vault import
 */

/** Request to import an Obsidian vault. */
export interface ImportVaultRequest {
  /** Path to the source Obsidian vault. */
  source_path: string;
  /** Optional subfolder within the target vault to import into. */
  target_subfolder: string | null;
}

/** Progress update during vault import. */
export interface ImportProgress {
  /** Current file being processed. */
  current_file: string;
  /** Number of files processed so far. */
  files_processed: number;
  /** Total number of files to process. */
  total_files: number;
  /** Number of properties imported. */
  properties_imported: number;
  /** Number of tags imported. */
  tags_imported: number;
}

/** Result of vault import. */
export interface ImportResult {
  /** Number of notes imported. */
  notes_imported: number;
  /** Number of files copied (includes non-markdown assets). */
  files_copied: number;
  /** Number of properties imported from frontmatter. */
  properties_imported: number;
  /** Number of tags imported (from frontmatter). */
  tags_imported: number;
  /** Duration of import in milliseconds. */
  duration_ms: number;
  /** Any warnings or skipped files. */
  warnings: string[];
}
