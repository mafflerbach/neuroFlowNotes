//! Note types.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// A note as exposed to the frontend.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct NoteDto {
    pub id: i64,
    pub path: String,
    pub title: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
    pub pinned: bool,
}

/// Minimal note info for lists/search results.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct NoteListItem {
    pub id: i64,
    pub path: String,
    pub title: Option<String>,
    pub pinned: bool,
}

/// Full note content for editing.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct NoteContent {
    pub id: i64,
    pub path: String,
    pub content: String,
}
