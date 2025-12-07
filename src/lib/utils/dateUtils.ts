/**
 * Date utility functions for calendar and note management.
 * Centralizes all date formatting, parsing, and range calculations.
 */

// Day and month name constants
export const DAY_NAMES_SHORT = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"];
export const DAY_NAMES_FULL = ["Sunday", "Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday"];
export const MONTH_NAMES_SHORT = ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"];
export const MONTH_NAMES_FULL = ["January", "February", "March", "April", "May", "June", "July", "August", "September", "October", "November", "December"];

/**
 * Format a Date as YYYY-MM-DD string (ISO date without time).
 * This is the standard format used for date keys throughout the app.
 */
export function formatDateKey(date: Date): string {
  return date.toISOString().split("T")[0];
}

/**
 * Check if a date is today.
 */
export function isToday(date: Date): boolean {
  return formatDateKey(date) === formatDateKey(new Date());
}

/**
 * Check if two dates are the same day.
 */
export function isSameDay(a: Date, b: Date): boolean {
  return formatDateKey(a) === formatDateKey(b);
}

/**
 * Check if a date is in the same month as another date.
 */
export function isSameMonth(date: Date, reference: Date): boolean {
  return (
    date.getMonth() === reference.getMonth() &&
    date.getFullYear() === reference.getFullYear()
  );
}

/**
 * Format a date as "Monday, Dec 7" style.
 */
export function formatDateDisplay(date: Date): string {
  return `${DAY_NAMES_FULL[date.getDay()]}, ${MONTH_NAMES_SHORT[date.getMonth()]} ${date.getDate()}`;
}

/**
 * Format a date as "December 2025" style.
 */
export function formatMonthYear(date: Date): string {
  return `${MONTH_NAMES_FULL[date.getMonth()]} ${date.getFullYear()}`;
}

/**
 * Get the week range (Monday to Sunday) containing a date.
 * Returns ISO date strings for start and end.
 */
export function getWeekRange(date: Date): { start: string; end: string } {
  const d = new Date(date);
  const day = d.getDay();
  const diff = day === 0 ? -6 : 1 - day; // Adjust for Monday start
  const monday = new Date(d);
  monday.setDate(d.getDate() + diff);
  const sunday = new Date(monday);
  sunday.setDate(monday.getDate() + 6);
  return {
    start: formatDateKey(monday),
    end: formatDateKey(sunday),
  };
}

/**
 * Get all dates in the week containing a date (Monday to Sunday).
 */
export function getWeekDates(date: Date): Date[] {
  const d = new Date(date);
  const day = d.getDay();
  const diff = day === 0 ? -6 : 1 - day;
  const monday = new Date(d);
  monday.setDate(d.getDate() + diff);

  return Array.from({ length: 7 }, (_, i) => {
    const weekDay = new Date(monday);
    weekDay.setDate(monday.getDate() + i);
    return weekDay;
  });
}

/**
 * Format a week range as "Dec 2 - 8" or "Nov 28 - Dec 4" style.
 */
export function formatWeekRange(date: Date): string {
  const dates = getWeekDates(date);
  const start = dates[0];
  const end = dates[6];

  if (start.getMonth() === end.getMonth()) {
    return `${MONTH_NAMES_SHORT[start.getMonth()]} ${start.getDate()} - ${end.getDate()}`;
  }
  return `${MONTH_NAMES_SHORT[start.getMonth()]} ${start.getDate()} - ${MONTH_NAMES_SHORT[end.getMonth()]} ${end.getDate()}`;
}

/**
 * Format a week range with year as "Dec 2 - 8, 2025" style.
 */
export function formatWeekRangeWithYear(date: Date): string {
  const dates = getWeekDates(date);
  const start = dates[0];
  const end = dates[6];

  if (start.getMonth() === end.getMonth()) {
    return `${MONTH_NAMES_SHORT[start.getMonth()]} ${start.getDate()} - ${end.getDate()}, ${start.getFullYear()}`;
  } else if (start.getFullYear() === end.getFullYear()) {
    return `${MONTH_NAMES_SHORT[start.getMonth()]} ${start.getDate()} - ${MONTH_NAMES_SHORT[end.getMonth()]} ${end.getDate()}, ${start.getFullYear()}`;
  }
  return `${MONTH_NAMES_SHORT[start.getMonth()]} ${start.getDate()}, ${start.getFullYear()} - ${MONTH_NAMES_SHORT[end.getMonth()]} ${end.getDate()}, ${end.getFullYear()}`;
}

/**
 * Get the month range for a date.
 * Returns ISO date strings for first and last day of month.
 */
export function getMonthRange(date: Date): { start: string; end: string } {
  const start = new Date(date.getFullYear(), date.getMonth(), 1);
  const end = new Date(date.getFullYear(), date.getMonth() + 1, 0);
  return {
    start: formatDateKey(start),
    end: formatDateKey(end),
  };
}

/**
 * Generate a calendar grid for a month (6 weeks x 7 days).
 * Includes days from previous/next months to fill the grid.
 * Week starts on Monday.
 */
export function getMonthCalendarGrid(date: Date): Date[][] {
  const year = date.getFullYear();
  const month = date.getMonth();

  const firstDay = new Date(year, month, 1);
  const lastDay = new Date(year, month + 1, 0);

  // Adjust for Monday start (0 = Monday, 6 = Sunday)
  let startOffset = firstDay.getDay() - 1;
  if (startOffset < 0) startOffset = 6;

  const days: Date[] = [];

  // Add days from previous month
  for (let i = startOffset - 1; i >= 0; i--) {
    days.push(new Date(year, month, -i));
  }

  // Add days of current month
  for (let i = 1; i <= lastDay.getDate(); i++) {
    days.push(new Date(year, month, i));
  }

  // Add days from next month to fill the grid (6 rows = 42 cells)
  const remaining = 42 - days.length;
  for (let i = 1; i <= remaining; i++) {
    days.push(new Date(year, month + 1, i));
  }

  // Split into weeks
  const weeks: Date[][] = [];
  for (let i = 0; i < days.length; i += 7) {
    weeks.push(days.slice(i, i + 7));
  }

  return weeks;
}

/**
 * Navigate to previous day.
 */
export function getPreviousDay(date: Date): Date {
  const newDate = new Date(date);
  newDate.setDate(newDate.getDate() - 1);
  return newDate;
}

/**
 * Navigate to next day.
 */
export function getNextDay(date: Date): Date {
  const newDate = new Date(date);
  newDate.setDate(newDate.getDate() + 1);
  return newDate;
}

/**
 * Navigate to previous week.
 */
export function getPreviousWeek(date: Date): Date {
  const newDate = new Date(date);
  newDate.setDate(newDate.getDate() - 7);
  return newDate;
}

/**
 * Navigate to next week.
 */
export function getNextWeek(date: Date): Date {
  const newDate = new Date(date);
  newDate.setDate(newDate.getDate() + 7);
  return newDate;
}

/**
 * Navigate to previous month.
 */
export function getPreviousMonth(date: Date): Date {
  const newDate = new Date(date);
  newDate.setMonth(newDate.getMonth() - 1);
  return newDate;
}

/**
 * Navigate to next month.
 */
export function getNextMonth(date: Date): Date {
  const newDate = new Date(date);
  newDate.setMonth(newDate.getMonth() + 1);
  return newDate;
}
