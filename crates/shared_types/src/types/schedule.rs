//! Schedule block types.

use chrono::{NaiveDate, NaiveTime};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::note::NoteListItem;

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
    /// RFC 5545 recurrence rule (e.g., "FREQ=WEEKLY;BYDAY=MO,WE,FR").
    pub rrule: Option<String>,
    /// True if this is an occurrence of a recurring block (not the master).
    /// Occurrences have the same id as their master but different dates.
    #[serde(default)]
    pub is_occurrence: bool,
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
    /// RFC 5545 recurrence rule (e.g., "FREQ=WEEKLY;BYDAY=MO,WE,FR").
    pub rrule: Option<String>,
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
    /// RFC 5545 recurrence rule (e.g., "FREQ=WEEKLY;BYDAY=MO,WE,FR").
    /// Set to empty string to clear recurrence.
    pub rrule: Option<String>,
}

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
