/**
 * Calendar configuration constants
 */

export const CALENDAR_CONFIG = {
  /** First hour displayed on the calendar grid */
  START_HOUR: 6,
  /** Last hour displayed on the calendar grid */
  END_HOUR: 22,
  /** Height of hour slots in weekly view (px) */
  HOUR_SLOT_HEIGHT_WEEKLY: 48,
  /** Height of hour slots in daily view (px) */
  HOUR_SLOT_HEIGHT_DAILY: 60,
  /** Padding between overlapping blocks (px) */
  BLOCK_OVERLAP_PADDING: 2,
} as const;

/** Generate array of hours for calendar grid */
export function getCalendarHours(
  startHour = CALENDAR_CONFIG.START_HOUR,
  endHour = CALENDAR_CONFIG.END_HOUR
): number[] {
  return Array.from({ length: endHour - startHour }, (_, i) => startHour + i);
}
