//! Template settings for daily notes and other templated content.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// Settings for template system (stored in vault config).
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct TemplateSettings {
    /// Path to the daily note template file (relative to vault root, e.g., "templates/daily.md").
    pub daily_template_path: Option<String>,

    /// Pattern for daily note file paths (e.g., "journal/{{year}}/{{month}}/{{date}}.md").
    pub daily_note_pattern: String,
}

impl Default for TemplateSettings {
    fn default() -> Self {
        Self {
            daily_template_path: None,
            daily_note_pattern: "journal/{{year}}/{{month}}/{{date}}.md".to_string(),
        }
    }
}

/// Result of creating a daily note.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct DailyNoteResult {
    /// The note ID.
    pub id: i64,
    /// The note path.
    pub path: String,
    /// The note title.
    pub title: Option<String>,
    /// Whether the note was newly created (true) or already existed (false).
    pub created: bool,
}
