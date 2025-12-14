//! Backlink types.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// A backlink (note that links to another note).
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct BacklinkDto {
    pub from_note_id: i64,
    pub from_note_path: String,
    pub from_note_title: Option<String>,
}
