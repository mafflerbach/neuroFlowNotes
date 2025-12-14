//! Event payload types (for Tauri events).

use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// Payload for notes:updated event.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct NotesUpdatedPayload {
    pub note_ids: Vec<i64>,
}

/// Payload for notes:deleted event.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct NotesDeletedPayload {
    pub note_ids: Vec<i64>,
}

/// Payload for index:complete event.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct IndexCompletePayload {
    pub notes_indexed: i64,
    pub duration_ms: u64,
}
