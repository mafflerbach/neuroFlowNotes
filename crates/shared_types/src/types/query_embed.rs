//! Query embed types (for inline ```query``` blocks).

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::query::{FilterMatchMode, PropertyFilter, QueryResultItem, QueryResultType};

/// View type for displaying query results.
#[derive(Debug, Clone, Default, Serialize, Deserialize, TS)]
#[ts(export)]
pub enum QueryViewType {
    /// Display as a table with columns.
    #[default]
    Table,
    /// Display as a simple list.
    List,
    /// Display as a Kanban board grouped by a property.
    Kanban,
    /// Display as cards in a grid layout.
    Card,
}

/// Sort direction for query results.
#[derive(Debug, Clone, Default, Serialize, Deserialize, TS)]
#[ts(export)]
pub enum SortDirection {
    #[default]
    Asc,
    Desc,
}

/// Sort configuration for query results.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct QuerySort {
    /// Property to sort by (e.g., "due_date", "priority", "note_title").
    pub property: String,
    /// Sort direction.
    pub direction: SortDirection,
}

/// Kanban-specific configuration.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct KanbanConfig {
    /// Property to group cards into columns (e.g., "priority", "status", "context").
    pub group_by: String,
    /// Fields to display on each card.
    #[serde(default)]
    pub card_fields: Vec<String>,
    /// Whether to show cards without a value in an "Uncategorized" column.
    #[serde(default = "default_true")]
    pub show_uncategorized: bool,
}

fn default_true() -> bool {
    true
}

impl Default for KanbanConfig {
    fn default() -> Self {
        Self {
            group_by: "priority".to_string(),
            card_fields: vec!["description".to_string(), "due_date".to_string()],
            show_uncategorized: true,
        }
    }
}

/// Card-specific configuration.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct CardConfig {
    /// Property to use as cover image (expects a URL or path to an image).
    #[serde(default)]
    pub cover_property: Option<String>,
    /// Fields to display on each card.
    #[serde(default)]
    pub display_fields: Vec<String>,
    /// Number of columns in the grid (0 = auto).
    #[serde(default = "default_card_columns")]
    pub columns: u8,
}

fn default_card_columns() -> u8 {
    0 // Auto
}

impl Default for CardConfig {
    fn default() -> Self {
        Self {
            cover_property: None,
            display_fields: vec!["description".to_string()],
            columns: 0,
        }
    }
}

/// View configuration for query embed.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct QueryViewConfig {
    /// View type (table, list, kanban, or card).
    #[serde(default)]
    pub view_type: QueryViewType,
    /// Columns to display (for table view). If empty, use defaults.
    #[serde(default)]
    pub columns: Vec<String>,
    /// Sort configuration.
    pub sort: Option<QuerySort>,
    /// Kanban-specific configuration (only used when view_type is "Kanban").
    pub kanban: Option<KanbanConfig>,
    /// Card-specific configuration (only used when view_type is "Card").
    pub card: Option<CardConfig>,
}

impl Default for QueryViewConfig {
    fn default() -> Self {
        Self {
            view_type: QueryViewType::Table,
            columns: vec![],
            sort: None,
            kanban: None,
            card: None,
        }
    }
}

/// A single query tab definition.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct QueryTab {
    /// Display name for this tab.
    pub name: String,
    /// Property filters to apply for this tab.
    #[serde(default)]
    pub filters: Vec<PropertyFilter>,
    /// How to match filters (All = AND, Any = OR). Defaults to All.
    #[serde(default = "default_match_mode")]
    pub match_mode: FilterMatchMode,
    /// What type of results to return. Defaults to Tasks.
    #[serde(default = "default_result_type")]
    pub result_type: QueryResultType,
    /// Include completed tasks. Defaults to false.
    #[serde(default)]
    pub include_completed: bool,
    /// Maximum number of results. Defaults to 50.
    #[serde(default = "default_limit")]
    pub limit: i32,
    /// View configuration for this tab.
    #[serde(default)]
    pub view: QueryViewConfig,
}

/// A complete query embed definition (parsed from YAML in ```query``` blocks).
/// Supports both single-query mode and multi-tab mode.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct QueryEmbed {
    /// Property filters to apply (for single-query mode).
    #[serde(default)]
    pub filters: Vec<PropertyFilter>,
    /// How to match filters (All = AND, Any = OR). Defaults to All.
    #[serde(default = "default_match_mode")]
    pub match_mode: FilterMatchMode,
    /// What type of results to return. Defaults to Tasks.
    #[serde(default = "default_result_type")]
    pub result_type: QueryResultType,
    /// Include completed tasks. Defaults to false.
    #[serde(default)]
    pub include_completed: bool,
    /// Maximum number of results. Defaults to 50.
    #[serde(default = "default_limit")]
    pub limit: i32,
    /// View configuration.
    #[serde(default)]
    pub view: QueryViewConfig,
    /// Optional tabs for multi-query mode. If present, overrides single-query fields.
    #[serde(default)]
    pub tabs: Vec<QueryTab>,
}

fn default_match_mode() -> FilterMatchMode {
    FilterMatchMode::All
}

fn default_result_type() -> QueryResultType {
    QueryResultType::Tasks
}

fn default_limit() -> i32 {
    50
}

impl Default for QueryEmbed {
    fn default() -> Self {
        Self {
            filters: vec![],
            match_mode: FilterMatchMode::All,
            result_type: QueryResultType::Tasks,
            include_completed: false,
            limit: 50,
            view: QueryViewConfig::default(),
            tabs: vec![],
        }
    }
}

/// Request to execute a query embed (YAML string).
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ExecuteQueryEmbedRequest {
    /// The YAML content of the query block.
    pub yaml_content: String,
}

/// Results for a single tab.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct TabResult {
    /// Tab name.
    pub name: String,
    /// The results for this tab.
    pub results: Vec<QueryResultItem>,
    /// Total count of matching items for this tab.
    pub total_count: i64,
    /// View configuration for this tab.
    pub view: QueryViewConfig,
}

/// Response from executing a query embed.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct QueryEmbedResponse {
    /// The parsed query configuration.
    pub query: QueryEmbed,
    /// The results matching the query (for single-query mode).
    pub results: Vec<QueryResultItem>,
    /// Total count of matching items (for single-query mode).
    pub total_count: i64,
    /// Results per tab (for multi-tab mode). Empty if not using tabs.
    #[serde(default)]
    pub tab_results: Vec<TabResult>,
    /// Error message if parsing or execution failed.
    pub error: Option<String>,
}
