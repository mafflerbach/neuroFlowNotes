/**
 * Habit tracker types
 */

// ============================================================================
// Habit Types
// ============================================================================

/** Type of habit value. */
export type HabitType = "boolean" | "number" | "text" | "rating";

/** A habit definition. */
export interface HabitDto {
  id: number;
  name: string;
  description: string | null;
  habit_type: HabitType;
  /** Unit for number habits (e.g., "minutes", "glasses", "km"). */
  unit: string | null;
  /** Color for visual display (hex or named color). */
  color: string | null;
  /** Target value for number habits (e.g., 8 glasses). */
  target_value: number | null;
  /** Whether the habit is archived (soft deleted). */
  archived: boolean;
  /** Sort order for display. */
  sort_order: number;
}

/** Request to create a new habit. */
export interface CreateHabitRequest {
  name: string;
  description?: string | null;
  habit_type: HabitType;
  unit?: string | null;
  color?: string | null;
  target_value?: number | null;
}

/** Request to update a habit. */
export interface UpdateHabitRequest {
  id: number;
  name?: string | null;
  description?: string | null;
  habit_type?: HabitType | null;
  unit?: string | null;
  color?: string | null;
  target_value?: number | null;
  archived?: boolean | null;
  sort_order?: number | null;
}

// ============================================================================
// Habit Entry Types
// ============================================================================

/** A single habit entry (log). */
export interface HabitEntryDto {
  id: number;
  habit_id: number;
  /** Date as YYYY-MM-DD string. */
  date: string;
  /** Optional time as HH:MM:SS string (for multiple entries per day). */
  time: string | null;
  /** Value as string (interpreted based on habit_type). */
  value: string;
  /** Optional notes for this entry. */
  notes: string | null;
}

/** Request to log a habit entry. */
export interface LogHabitEntryRequest {
  habit_id: number;
  /** Date as YYYY-MM-DD string. */
  date: string;
  /** Optional time as HH:MM:SS string. */
  time?: string | null;
  /** Value to log. */
  value: string;
  /** Optional notes. */
  notes?: string | null;
}

/** Request to update a habit entry. */
export interface UpdateHabitEntryRequest {
  id: number;
  value?: string | null;
  notes?: string | null;
  time?: string | null;
}

// ============================================================================
// Habit Tracker Embed Types
// ============================================================================

/** View type for habit tracker embeds. */
export type HabitViewType = "table" | "calendar" | "streak" | "list";

/** Orientation for table view. */
export type HabitTableOrientation = "horizontal" | "vertical";

/** Date range preset for habit tracker. */
export type HabitDateRange =
  | "single_day"
  | "last7_days"
  | "last30_days"
  | "this_week"
  | "this_month"
  | "custom";

/** Query for habit tracker embed. */
export interface HabitTrackerQuery {
  /** Habit names or IDs to include (empty = all non-archived). */
  habits: string[];
  /** View type. */
  view: HabitViewType;
  /** Table orientation (only applies to table view). */
  orientation: HabitTableOrientation;
  /** Date range preset. */
  date_range: HabitDateRange;
  /** Reference date for computing date ranges (YYYY-MM-DD). Defaults to today.
   * Useful for templates where you want to center the view on a specific date.
   * Can use template variables like {{date}} in the YAML.
   */
  date?: string | null;
  /** Custom start date (YYYY-MM-DD) when date_range is custom. */
  start_date?: string | null;
  /** Custom end date (YYYY-MM-DD) when date_range is custom. */
  end_date?: string | null;
  /** Whether cells are editable in the embed. */
  editable: boolean;
  /** Whether to show summary row (totals, streaks). */
  show_summary: boolean;
}

/** Habit with its entries for a date range. */
export interface HabitWithEntries {
  habit: HabitDto;
  /** Entries organized by date (YYYY-MM-DD -> entries). */
  entries_by_date: [string, HabitEntryDto[]][];
}

/** Response for habit tracker embed. */
export interface HabitTrackerResponse {
  /** The parsed query configuration. */
  query: HabitTrackerQuery;
  /** Habits with their entries. */
  habits: HabitWithEntries[];
  /** Actual date range start (YYYY-MM-DD). */
  date_range_start: string;
  /** Actual date range end (YYYY-MM-DD). */
  date_range_end: string;
  /** Error message if query failed. */
  error: string | null;
}
