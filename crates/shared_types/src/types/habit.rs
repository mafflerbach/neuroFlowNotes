//! Habit tracker types.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

// ============================================================================
// Habit Types
// ============================================================================

/// Type of habit value.
#[derive(Debug, Clone, Default, Serialize, Deserialize, TS, PartialEq)]
#[ts(export)]
#[serde(rename_all = "lowercase")]
pub enum HabitType {
    /// Simple yes/no checkbox.
    #[default]
    Boolean,
    /// Numeric value with optional unit.
    Number,
    /// Free-form text.
    Text,
    /// 1-5 star rating.
    Rating,
}

impl HabitType {
    pub fn as_str(&self) -> &'static str {
        match self {
            HabitType::Boolean => "boolean",
            HabitType::Number => "number",
            HabitType::Text => "text",
            HabitType::Rating => "rating",
        }
    }

    pub fn from_string(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "number" => HabitType::Number,
            "text" => HabitType::Text,
            "rating" => HabitType::Rating,
            _ => HabitType::Boolean,
        }
    }
}

/// A habit definition.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct HabitDto {
    pub id: i64,
    pub name: String,
    pub description: Option<String>,
    pub habit_type: HabitType,
    /// Unit for number habits (e.g., "minutes", "glasses", "km").
    pub unit: Option<String>,
    /// Color for visual display (hex or named color).
    pub color: Option<String>,
    /// Target value for number habits (e.g., 8 glasses).
    pub target_value: Option<f64>,
    /// Whether the habit is archived (soft deleted).
    pub archived: bool,
    /// Sort order for display.
    pub sort_order: i32,
}

/// Request to create a new habit.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct CreateHabitRequest {
    pub name: String,
    pub description: Option<String>,
    pub habit_type: HabitType,
    pub unit: Option<String>,
    pub color: Option<String>,
    pub target_value: Option<f64>,
}

/// Request to update a habit.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct UpdateHabitRequest {
    pub id: i64,
    pub name: Option<String>,
    pub description: Option<String>,
    pub habit_type: Option<HabitType>,
    pub unit: Option<String>,
    pub color: Option<String>,
    pub target_value: Option<f64>,
    pub archived: Option<bool>,
    pub sort_order: Option<i32>,
}

// ============================================================================
// Habit Entry Types
// ============================================================================

/// A single habit entry (log).
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct HabitEntryDto {
    pub id: i64,
    pub habit_id: i64,
    /// Date as YYYY-MM-DD string.
    pub date: String,
    /// Optional time as HH:MM:SS string (for multiple entries per day).
    pub time: Option<String>,
    /// Value as string (interpreted based on habit_type).
    pub value: String,
    /// Optional notes for this entry.
    pub notes: Option<String>,
}

/// Request to log a habit entry.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct LogHabitEntryRequest {
    pub habit_id: i64,
    /// Date as YYYY-MM-DD string.
    pub date: String,
    /// Optional time as HH:MM:SS string.
    pub time: Option<String>,
    /// Value to log.
    pub value: String,
    /// Optional notes.
    pub notes: Option<String>,
}

/// Request to update a habit entry.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct UpdateHabitEntryRequest {
    pub id: i64,
    pub value: Option<String>,
    pub notes: Option<String>,
    pub time: Option<String>,
}

// ============================================================================
// Habit Tracker Embed Types
// ============================================================================

/// View type for habit tracker embeds.
#[derive(Debug, Clone, Default, Serialize, Deserialize, TS, PartialEq)]
#[ts(export)]
#[serde(rename_all = "lowercase")]
pub enum HabitViewType {
    /// Grid of habits x dates.
    #[default]
    Table,
    /// Calendar heatmap view.
    Calendar,
    /// Streak visualization.
    Streak,
    /// Simple list view.
    List,
}

/// Orientation for table view.
#[derive(Debug, Clone, Default, Serialize, Deserialize, TS, PartialEq)]
#[ts(export)]
#[serde(rename_all = "lowercase")]
pub enum HabitTableOrientation {
    /// Habits as rows, dates as columns (default).
    #[default]
    Horizontal,
    /// Dates as rows, habits as columns.
    Vertical,
}

/// Date range preset for habit tracker.
#[derive(Debug, Clone, Default, Serialize, Deserialize, TS, PartialEq)]
#[ts(export)]
#[serde(rename_all = "snake_case")]
pub enum HabitDateRange {
    /// Single day (uses the reference date or today).
    SingleDay,
    #[default]
    Last7Days,
    Last30Days,
    ThisWeek,
    ThisMonth,
    Custom,
}

/// Query for habit tracker embed.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct HabitTrackerQuery {
    /// Habit names or IDs to include (empty = all non-archived).
    #[serde(default)]
    pub habits: Vec<String>,
    /// View type.
    #[serde(default)]
    pub view: HabitViewType,
    /// Table orientation (only applies to table view).
    #[serde(default)]
    pub orientation: HabitTableOrientation,
    /// Date range preset.
    #[serde(default)]
    pub date_range: HabitDateRange,
    /// Reference date for computing date ranges (YYYY-MM-DD). Defaults to today.
    /// Useful for templates where you want to center the view on a specific date.
    pub date: Option<String>,
    /// Custom start date (YYYY-MM-DD) when date_range is Custom.
    pub start_date: Option<String>,
    /// Custom end date (YYYY-MM-DD) when date_range is Custom.
    pub end_date: Option<String>,
    /// Whether cells are editable in the embed.
    #[serde(default = "default_true")]
    pub editable: bool,
    /// Whether to show summary row (totals, streaks).
    #[serde(default = "default_true")]
    pub show_summary: bool,
}

fn default_true() -> bool {
    true
}

impl Default for HabitTrackerQuery {
    fn default() -> Self {
        Self {
            habits: vec![],
            view: HabitViewType::default(),
            orientation: HabitTableOrientation::default(),
            date_range: HabitDateRange::default(),
            date: None,
            start_date: None,
            end_date: None,
            editable: true,
            show_summary: true,
        }
    }
}

/// Habit with its entries for a date range.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct HabitWithEntries {
    pub habit: HabitDto,
    /// Entries organized by date (YYYY-MM-DD -> entries).
    pub entries_by_date: Vec<(String, Vec<HabitEntryDto>)>,
}

/// Response for habit tracker embed.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct HabitTrackerResponse {
    /// The parsed query configuration.
    pub query: HabitTrackerQuery,
    /// Habits with their entries.
    pub habits: Vec<HabitWithEntries>,
    /// Actual date range start (YYYY-MM-DD).
    pub date_range_start: String,
    /// Actual date range end (YYYY-MM-DD).
    pub date_range_end: String,
    /// Error message if query failed.
    pub error: Option<String>,
}
