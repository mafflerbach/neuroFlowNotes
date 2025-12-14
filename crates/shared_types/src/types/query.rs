//! Query builder types.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::note::NoteListItem;
use super::property::PropertyDto;
use super::todo::TaskWithContext;

/// Operator for property filters.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub enum PropertyOperator {
    /// Property exists (any value)
    Exists,
    /// Property does not exist
    NotExists,
    /// Property equals exact value
    Equals,
    /// Property does not equal value
    NotEquals,
    /// Property contains substring
    Contains,
    /// Property starts with prefix
    StartsWith,
    /// Property ends with suffix
    EndsWith,
    /// Property list contains ALL of the specified values (comma-separated)
    ContainsAll,
    /// Property list contains ANY of the specified values (comma-separated)
    ContainsAny,
    /// Date property equals the specified date (YYYY-MM-DD)
    DateOn,
    /// Date property is before the specified date
    DateBefore,
    /// Date property is after the specified date
    DateAfter,
    /// Date property is on or before the specified date
    DateOnOrBefore,
    /// Date property is on or after the specified date
    DateOnOrAfter,
}

/// A single property filter condition.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct PropertyFilter {
    /// The property key to filter on.
    pub key: String,
    /// The comparison operator.
    pub operator: PropertyOperator,
    /// The value to compare against (not used for Exists/NotExists).
    pub value: Option<String>,
}

/// How to match multiple filter conditions.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub enum FilterMatchMode {
    /// All conditions must match (AND).
    All,
    /// Any condition can match (OR).
    Any,
}

/// What type of results to return.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub enum QueryResultType {
    /// Return tasks from matching notes.
    Tasks,
    /// Return matching notes.
    Notes,
    /// Return both tasks and notes.
    Both,
}

/// Request to run a query.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct QueryRequest {
    /// Property filters to apply.
    pub filters: Vec<PropertyFilter>,
    /// How to match filters (All = AND, Any = OR).
    pub match_mode: FilterMatchMode,
    /// What type of results to return.
    pub result_type: QueryResultType,
    /// Include completed tasks (only for Tasks/Both result types).
    pub include_completed: bool,
    /// Maximum number of results.
    pub limit: Option<i32>,
}

/// A single query result item (can be a task or a note).
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct QueryResultItem {
    /// The type of result ("task" or "note").
    pub item_type: String,
    /// Task data (if item_type is "task").
    pub task: Option<TaskWithContext>,
    /// Note data (if item_type is "note").
    pub note: Option<NoteListItem>,
    /// Properties of the note (for display in results).
    pub properties: Vec<PropertyDto>,
}

/// Response from running a query.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct QueryResponse {
    /// The results matching the query.
    pub results: Vec<QueryResultItem>,
    /// Total count of matching items (may be > results.len() if limited).
    pub total_count: i64,
}
