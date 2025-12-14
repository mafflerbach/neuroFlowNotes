//! Import types (for Obsidian vault import).

use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// Request to import an Obsidian vault.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ImportVaultRequest {
    /// Path to the source Obsidian vault.
    pub source_path: String,
    /// Optional subfolder within the target vault to import into.
    pub target_subfolder: Option<String>,
}

/// Progress update during vault import.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ImportProgress {
    /// Current file being processed.
    pub current_file: String,
    /// Number of files processed so far.
    pub files_processed: i64,
    /// Total number of files to process.
    pub total_files: i64,
    /// Number of properties imported.
    pub properties_imported: i64,
    /// Number of tags imported.
    pub tags_imported: i64,
}

/// Result of vault import.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ImportResult {
    /// Number of notes imported.
    pub notes_imported: i64,
    /// Number of files copied (includes non-markdown assets).
    pub files_copied: i64,
    /// Number of properties imported from frontmatter.
    pub properties_imported: i64,
    /// Number of tags imported (from frontmatter).
    pub tags_imported: i64,
    /// Duration of import in milliseconds.
    pub duration_ms: u64,
    /// Any warnings or skipped files.
    pub warnings: Vec<String>,
}
