/**
 * Habit Tracker Plugin Types
 */

export interface HabitTrackerSettings {
  /** Default view type for embeds (table, calendar, streak, list). */
  defaultView: "table" | "calendar" | "streak" | "list";
  /** Default date range for embeds. */
  defaultDateRange: "last7_days" | "last30_days" | "this_week" | "this_month";
  /** Whether to show habits in calendar view. */
  showInCalendar: boolean;
  /** Default color for new habits. */
  defaultColor: string;
}
