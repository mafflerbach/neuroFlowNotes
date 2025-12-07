//! Shared types (DTOs) for NeuroFlow Notes.
//!
//! These types are used for communication between Rust backend and TypeScript frontend.
//! They are designed to be serializable and have TypeScript bindings generated via ts-rs.

use chrono::{DateTime, NaiveDate, NaiveTime, Utc};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

// ============================================================================
// Note Types
// ============================================================================

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

// ============================================================================
// Todo Types
// ============================================================================

/// A todo item extracted from a note.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct TodoDto {
    pub id: i64,
    pub note_id: i64,
    pub line_number: Option<i32>,
    pub description: String,
    pub completed: bool,
    pub heading_path: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
}

/// Request to toggle a todo's completion status.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct TodoToggleRequest {
    pub todo_id: i64,
    pub completed: bool,
}

// ============================================================================
// Schedule Block Types
// ============================================================================

/// A scheduled time block (optionally linked to a note).
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ScheduleBlockDto {
    pub id: i64,
    pub note_id: Option<i64>,
    pub date: NaiveDate,
    pub start_time: NaiveTime,
    pub end_time: NaiveTime,
    pub label: Option<String>,
    pub color: Option<String>,
    pub context: Option<String>,
}

/// Request to create a new schedule block.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct CreateScheduleBlockRequest {
    pub note_id: Option<i64>,
    pub date: NaiveDate,
    pub start_time: NaiveTime,
    pub end_time: NaiveTime,
    pub label: Option<String>,
    pub color: Option<String>,
    pub context: Option<String>,
}

/// Request to update an existing schedule block.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct UpdateScheduleBlockRequest {
    pub id: i64,
    pub note_id: Option<i64>,
    pub date: Option<NaiveDate>,
    pub start_time: Option<NaiveTime>,
    pub end_time: Option<NaiveTime>,
    pub label: Option<String>,
    pub color: Option<String>,
    pub context: Option<String>,
}

// ============================================================================
// Tag Types
// ============================================================================

/// A tag with usage count.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct TagDto {
    pub tag: String,
    pub count: i64,
}

// ============================================================================
// Backlink Types
// ============================================================================

/// A backlink (note that links to another note).
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct BacklinkDto {
    pub from_note_id: i64,
    pub from_note_path: String,
    pub from_note_title: Option<String>,
}

// ============================================================================
// Vault Types
// ============================================================================

/// Information about an open vault.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct VaultInfo {
    pub path: String,
    pub name: String,
    pub note_count: i64,
}

/// Entry in the recent vaults list.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct RecentVault {
    pub path: String,
    pub name: String,
    pub last_opened: DateTime<Utc>,
}

// ============================================================================
// Search Types
// ============================================================================

/// A search result.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct SearchResult {
    pub note_id: i64,
    pub path: String,
    pub title: Option<String>,
    pub snippet: Option<String>,
    pub score: f64,
}

/// Search query parameters.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct SearchQuery {
    pub query: String,
    pub limit: Option<i32>,
    pub offset: Option<i32>,
}

// ============================================================================
// Folder Tree Types
// ============================================================================

/// A node in the folder tree.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct FolderNode {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub children: Vec<FolderNode>,
}

// ============================================================================
// Property Types
// ============================================================================

/// A key-value property for a note (stored in SQLite, not frontmatter).
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct PropertyDto {
    pub id: i64,
    pub note_id: i64,
    pub key: String,
    pub value: Option<String>,
    /// Type hint: "text", "date", "number", "boolean", "list"
    pub property_type: Option<String>,
    pub sort_order: Option<i32>,
}

/// Request to set a property value.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct SetPropertyRequest {
    pub note_id: i64,
    pub key: String,
    pub value: Option<String>,
    pub property_type: Option<String>,
}

// ============================================================================
// Notes by Date Types
// ============================================================================

/// A note with its association type to a date.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct NoteForDate {
    pub note: NoteListItem,
    /// "scheduled", "journal", or "created"
    pub source: String,
    /// If source is "scheduled", the schedule block info
    pub schedule_block: Option<ScheduleBlockDto>,
}

// ============================================================================
// Event Payloads (for Tauri events)
// ============================================================================

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
