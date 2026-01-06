/**
 * Query builder types - for property-based filtering.
 */

import type { TaskWithContext, PropertyDto } from "./metadata";
import type { NoteListItem } from "./note";

/** Operator for property filters. */
export type PropertyOperator =
  | "Exists"
  | "NotExists"
  | "Equals"
  | "NotEquals"
  | "Contains"
  | "StartsWith"
  | "EndsWith"
  | "ContainsAll"
  | "ContainsAny"
  | "DateOn"
  | "DateBefore"
  | "DateAfter"
  | "DateOnOrBefore"
  | "DateOnOrAfter";

/** A single property filter condition. */
export interface PropertyFilter {
  /** The property key to filter on. */
  key: string;
  /** The comparison operator. */
  operator: PropertyOperator;
  /** The value to compare against (not used for Exists/NotExists). */
  value: string | null;
}

/** How to match multiple filter conditions. */
export type FilterMatchMode = "All" | "Any";

/** What type of results to return. */
export type QueryResultType = "Tasks" | "Notes" | "Both";

/** Request to run a query. */
export interface QueryRequest {
  /** Property filters to apply. */
  filters: PropertyFilter[];
  /** How to match filters (All = AND, Any = OR). */
  match_mode: FilterMatchMode;
  /** What type of results to return. */
  result_type: QueryResultType;
  /** Include completed tasks (only for Tasks/Both result types). */
  include_completed: boolean;
  /** Maximum number of results. */
  limit: number | null;
}

/** A single query result item (can be a task or a note). */
export interface QueryResultItem {
  /** The type of result ("task" or "note"). */
  item_type: "task" | "note";
  /** Task data (if item_type is "task"). */
  task: TaskWithContext | null;
  /** Note data (if item_type is "note"). */
  note: NoteListItem | null;
  /** Properties of the note (for display in results). */
  properties: PropertyDto[];
}

/** Response from running a query. */
export interface QueryResponse {
  /** The results matching the query. */
  results: QueryResultItem[];
  /** Total count of matching items (may be > results.len() if limited). */
  total_count: number;
}

/** Information about a property key used in the vault. */
export interface PropertyKeyInfo {
  /** The property key name. */
  key: string;
  /** Number of notes using this property. */
  usage_count: number;
  /** Sample values for this property (up to 10). */
  sample_values: string[];
  /** Most common property type for this key (text, date, number, boolean, list). */
  property_type: string | null;
}

/** User-friendly labels for operators. */
export const OPERATOR_LABELS: Record<PropertyOperator, string> = {
  Exists: "exists",
  NotExists: "does not exist",
  Equals: "equals",
  NotEquals: "does not equal",
  Contains: "contains",
  StartsWith: "starts with",
  EndsWith: "ends with",
  ContainsAll: "contains all",
  ContainsAny: "contains any",
  DateOn: "on date",
  DateBefore: "before",
  DateAfter: "after",
  DateOnOrBefore: "on or before",
  DateOnOrAfter: "on or after",
};

/** Operators that don't require a value. */
export const VALUELESS_OPERATORS: PropertyOperator[] = ["Exists", "NotExists"];

// ============================================================================
// Query Embed Types (for inline ```query``` blocks)
// ============================================================================

/** View type for displaying query results. */
export type QueryViewType = "Table" | "List" | "Kanban" | "Card";

/** Sort direction for query results. */
export type SortDirection = "Asc" | "Desc";

/** Sort configuration for query results. */
export interface QuerySort {
  /** Property to sort by (e.g., "due_date", "priority", "note_title"). */
  property: string;
  /** Sort direction. */
  direction: SortDirection;
}

/** Kanban-specific configuration. */
export interface KanbanConfig {
  /** Property to group cards into columns (e.g., "priority", "status", "context"). */
  group_by: string;
  /** Fields to display on each card. */
  card_fields: string[];
  /** Whether to show cards without a value in an "Uncategorized" column. */
  show_uncategorized: boolean;
}

/** Card-specific configuration. */
export interface CardConfig {
  /** Property to use as cover image (expects a URL or path to an image). */
  cover_property: string | null;
  /** Fields to display on each card. */
  display_fields: string[];
  /** Number of columns in the grid (0 = auto). */
  columns: number;
  /** Property to toggle on card click (e.g., "read" for reading lists). */
  toggle_property?: string;
  /** Position of toggle button on card. */
  toggle_position?: "top-right" | "top-left" | "inline";
  /** Dim card when toggle property is true. */
  dim_when_true?: boolean;
}

/** Interactive filter configuration. */
export interface InteractiveFilter {
  /** Property key to filter on. */
  key: string;
  /** Filter UI style. */
  style: "chips" | "buttons" | "dropdown";
  /** Show "All" option to clear filter. */
  show_all: boolean;
  /** Allow selecting multiple values. */
  multi_select: boolean;
  /** Optional display label for the filter. */
  label?: string;
}

/** Stats bar configuration. */
export interface StatsConfig {
  /** Whether to show the stats bar. */
  show: boolean;
  /** Show total count. */
  total: boolean;
  /** Property to group stats by (e.g., "read" to show "5 read, 10 unread"). */
  group_by?: string;
}

/** View configuration for query embed. */
export interface QueryViewConfig {
  /** View type (table, list, kanban, or card). */
  view_type: QueryViewType;
  /** Columns to display (for table view). If empty, use defaults. */
  columns: string[];
  /** Sort configuration. */
  sort: QuerySort | null;
  /** Kanban-specific configuration (only used when view_type is "Kanban"). */
  kanban: KanbanConfig | null;
  /** Card-specific configuration (only used when view_type is "Card"). */
  card: CardConfig | null;
  /** Interactive filter configurations. */
  interactive_filters?: InteractiveFilter[];
  /** Stats bar configuration. */
  stats?: StatsConfig;
}

/** A single query tab definition. */
export interface QueryTab {
  /** Display name for this tab. */
  name: string;
  /** Property filters to apply for this tab. */
  filters: PropertyFilter[];
  /** How to match filters (All = AND, Any = OR). Defaults to All. */
  match_mode: FilterMatchMode;
  /** What type of results to return. Defaults to Tasks. */
  result_type: QueryResultType;
  /** Include completed tasks. Defaults to false. */
  include_completed: boolean;
  /** Maximum number of results. Defaults to 50. */
  limit: number;
  /** View configuration for this tab. */
  view: QueryViewConfig;
}

/** A complete query embed definition (parsed from YAML in ```query``` blocks).
 * Supports both single-query mode and multi-tab mode. */
export interface QueryEmbed {
  /** Property filters to apply (for single-query mode). */
  filters: PropertyFilter[];
  /** How to match filters (All = AND, Any = OR). Defaults to All. */
  match_mode: FilterMatchMode;
  /** What type of results to return. Defaults to Tasks. */
  result_type: QueryResultType;
  /** Include completed tasks. Defaults to false. */
  include_completed: boolean;
  /** Maximum number of results. Defaults to 50. */
  limit: number;
  /** View configuration. */
  view: QueryViewConfig;
  /** Optional tabs for multi-query mode. If present, overrides single-query fields. */
  tabs: QueryTab[];
}

/** Results for a single tab. */
export interface TabResult {
  /** Tab name. */
  name: string;
  /** The results for this tab. */
  results: QueryResultItem[];
  /** Total count of matching items for this tab. */
  total_count: number;
  /** View configuration for this tab. */
  view: QueryViewConfig;
}

/** Response from executing a query embed. */
export interface QueryEmbedResponse {
  /** The parsed query configuration. */
  query: QueryEmbed;
  /** The results matching the query (for single-query mode). */
  results: QueryResultItem[];
  /** Total count of matching items (for single-query mode). */
  total_count: number;
  /** Results per tab (for multi-tab mode). Empty if not using tabs. */
  tab_results: TabResult[];
  /** Error message if parsing or execution failed. */
  error: string | null;
}
