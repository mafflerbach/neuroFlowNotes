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

// ============================================================================
// Embed Types
// ============================================================================

/// Request to resolve an embed (![[target]] or ![[target#section]]).
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ResolveEmbedRequest {
    /// The target note name or path (without .md extension).
    pub target: String,
    /// Optional section slug to extract (e.g., "my-section" from "## My Section").
    pub section: Option<String>,
    /// Current embedding depth (starts at 0, max 3).
    pub depth: u8,
}

/// Result of resolving an embed.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct EmbedContent {
    /// The note ID if found in database.
    pub note_id: Option<i64>,
    /// The resolved path to the note or image.
    pub path: String,
    /// The markdown content to embed (for notes).
    pub content: Option<String>,
    /// Whether this is an image embed.
    pub is_image: bool,
    /// Asset URL for images (using Tauri asset protocol).
    pub asset_url: Option<String>,
    /// Error message if resolution failed.
    pub error: Option<String>,
}

/// Information about a heading in a note (for section autocomplete).
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct HeadingInfo {
    /// Heading level (1-6).
    pub level: u8,
    /// The heading text as displayed.
    pub text: String,
    /// URL-safe slug for linking (e.g., "my-section").
    pub slug: String,
}

// ============================================================================
// Query Builder Types
// ============================================================================

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

/// Information about a property key used in the vault.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct PropertyKeyInfo {
    /// The property key name.
    pub key: String,
    /// Number of notes using this property.
    pub usage_count: i64,
    /// Sample values for this property (up to 10).
    pub sample_values: Vec<String>,
    /// Most common property type for this key (text, date, number, boolean, list).
    pub property_type: Option<String>,
}

// ============================================================================
// Property Management Types (bulk operations)
// ============================================================================

/// Request to rename a property key across all notes.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct RenamePropertyKeyRequest {
    /// The current key name.
    pub old_key: String,
    /// The new key name.
    pub new_key: String,
}

/// Request to rename a property value across all notes with that key.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct RenamePropertyValueRequest {
    /// The property key.
    pub key: String,
    /// The current value.
    pub old_value: String,
    /// The new value.
    pub new_value: String,
}

/// Request to merge two property keys (rename source to target, merging if target exists).
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct MergePropertyKeysRequest {
    /// The property key to merge from (will be deleted).
    pub source_key: String,
    /// The property key to merge into (will keep all values).
    pub target_key: String,
}

/// Request to delete a property key from all notes.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct DeletePropertyKeyRequest {
    /// The property key to delete.
    pub key: String,
}

/// Response for bulk property operations.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct PropertyOperationResult {
    /// Number of properties affected.
    pub affected_count: i64,
    /// Number of notes affected.
    pub notes_affected: i64,
}

/// Information about a property value used in the vault.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct PropertyValueInfo {
    /// The property value.
    pub value: String,
    /// Number of notes using this value.
    pub usage_count: i64,
}

/// A note that uses a specific property, including the value.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct NoteWithPropertyValue {
    /// The note ID.
    pub note_id: i64,
    /// The note path.
    pub path: String,
    /// The note title (if any).
    pub title: Option<String>,
    /// The property value in this note.
    pub value: Option<String>,
}

// ============================================================================
// Folder Property Types
// ============================================================================

/// A key-value property for a folder (inherited by notes in that folder tree).
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct FolderPropertyDto {
    pub id: i64,
    pub folder_path: String,
    pub key: String,
    pub value: Option<String>,
    /// Type hint: "text", "date", "number", "boolean", "list"
    pub property_type: Option<String>,
}

/// Request to set a folder property value.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct SetFolderPropertyRequest {
    pub folder_path: String,
    pub key: String,
    pub value: Option<String>,
    pub property_type: Option<String>,
}

/// A property with inheritance information.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct PropertyWithInheritance {
    pub id: i64,
    pub key: String,
    pub value: Option<String>,
    pub property_type: Option<String>,
    pub sort_order: Option<i32>,
    /// True if this property is inherited from a folder (not directly set on the note).
    pub inherited: bool,
    /// The folder path this property is inherited from (if inherited).
    pub inherited_from: Option<String>,
}

// ============================================================================
// Query Embed Types (for inline ```query``` blocks)
// ============================================================================

/// View type for displaying query results.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub enum QueryViewType {
    /// Display as a table with columns.
    Table,
    /// Display as a simple list.
    List,
    /// Display as a Kanban board grouped by a property.
    Kanban,
}

impl Default for QueryViewType {
    fn default() -> Self {
        QueryViewType::Table
    }
}

/// Sort direction for query results.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub enum SortDirection {
    Asc,
    Desc,
}

impl Default for SortDirection {
    fn default() -> Self {
        SortDirection::Asc
    }
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

/// View configuration for query embed.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct QueryViewConfig {
    /// View type (table, list, or kanban).
    #[serde(default)]
    pub view_type: QueryViewType,
    /// Columns to display (for table view). If empty, use defaults.
    #[serde(default)]
    pub columns: Vec<String>,
    /// Sort configuration.
    pub sort: Option<QuerySort>,
    /// Kanban-specific configuration (only used when view_type is "Kanban").
    pub kanban: Option<KanbanConfig>,
}

impl Default for QueryViewConfig {
    fn default() -> Self {
        Self {
            view_type: QueryViewType::Table,
            columns: vec![],
            sort: None,
            kanban: None,
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

// ============================================================================
// Import Types (for Obsidian vault import)
// ============================================================================

/// Request to import an Obsidian vault.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ImportVaultRequest {
    /// Path to the source Obsidian vault.
    pub source_path: String,
    /// Optional subfolder within the target vault to import into.
    pub target_subfolder: Option<String>,
}

/// Progress update during vault import.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ImportProgress {
    /// Current file being processed.
    pub current_file: String,
    /// Number of files processed so far.
    pub files_processed: i64,
    /// Total number of files to process.
    pub total_files: i64,
    /// Number of properties imported.
    pub properties_imported: i64,
    /// Number of tags imported.
    pub tags_imported: i64,
}

/// Result of vault import.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ImportResult {
    /// Number of notes imported.
    pub notes_imported: i64,
    /// Number of files copied (includes non-markdown assets).
    pub files_copied: i64,
    /// Number of properties imported from frontmatter.
    pub properties_imported: i64,
    /// Number of tags imported (from frontmatter).
    pub tags_imported: i64,
    /// Duration of import in milliseconds.
    pub duration_ms: u64,
    /// Any warnings or skipped files.
    pub warnings: Vec<String>,
}
