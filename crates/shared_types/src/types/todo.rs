//! Todo/Task types.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::property::PropertyDto;

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
    /// GTD context (e.g., "home", "work", "phone", "computer").
    pub context: Option<String>,
    /// Priority level ("high", "medium", "low").
    pub priority: Option<String>,
    /// Due date as YYYY-MM-DD string.
    pub due_date: Option<String>,
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

/// A task (todo) with enriched context from its parent note.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct TaskWithContext {
    /// The todo item.
    pub todo: TodoDto,
    /// Path to the note containing this task.
    pub note_path: String,
    /// Title of the note containing this task.
    pub note_title: Option<String>,
    /// Properties inherited from the parent note (e.g., project, area).
    pub note_properties: Vec<PropertyDto>,
}

/// Query parameters for filtering tasks.
#[derive(Debug, Clone, Default, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct TaskQuery {
    /// Filter by completion status (None = all, Some(true) = completed, Some(false) = incomplete).
    pub completed: Option<bool>,
    /// Filter by context (e.g., "home", "work").
    pub context: Option<String>,
    /// Filter by priority ("high", "medium", "low").
    pub priority: Option<String>,
    /// Filter by due date range start (inclusive, YYYY-MM-DD).
    pub due_from: Option<String>,
    /// Filter by due date range end (inclusive, YYYY-MM-DD).
    pub due_to: Option<String>,
    /// Filter by note property (key=value).
    pub property_filter: Option<String>,
    /// Maximum number of results.
    pub limit: Option<i32>,
}
