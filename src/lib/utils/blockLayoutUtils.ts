/**
 * Utilities for schedule block layout calculations.
 * Handles time parsing, overlap detection, and column-based layout
 * for displaying overlapping blocks side-by-side (like Outlook/Google Calendar).
 */

import type { ScheduleBlockDto } from "../types";

/**
 * Parse a time string "HH:MM" or "HH:MM:SS" to decimal hours.
 * e.g., "14:30" -> 14.5
 */
export function parseTime(timeStr: string): number {
  const [hours, minutes] = timeStr.split(":").map(Number);
  return hours + minutes / 60;
}

/**
 * Format decimal hours to "HH:MM" string.
 * e.g., 14.5 -> "14:30"
 */
export function formatTime(decimalHours: number): string {
  const hours = Math.floor(decimalHours);
  const minutes = Math.round((decimalHours - hours) * 60);
  return `${hours.toString().padStart(2, "0")}:${minutes.toString().padStart(2, "0")}`;
}

/**
 * Format a time string to short format "HH:MM".
 * e.g., "14:30:00" -> "14:30"
 */
export function formatTimeShort(timeStr: string): string {
  return timeStr.slice(0, 5);
}

/**
 * Check if two schedule blocks overlap in time.
 */
export function blocksOverlap(a: ScheduleBlockDto, b: ScheduleBlockDto): boolean {
  const aStart = parseTime(a.start_time);
  const aEnd = parseTime(a.end_time);
  const bStart = parseTime(b.start_time);
  const bEnd = parseTime(b.end_time);
  return aStart < bEnd && bStart < aEnd;
}

export interface BlockWithLayout {
  block: ScheduleBlockDto;
  column: number;
  totalColumns: number;
}

/**
 * Calculate layout for overlapping blocks.
 * Assigns each block a column position so overlapping blocks display side-by-side.
 * Returns blocks with their column assignment and total columns in their group.
 */
export function getBlocksWithLayout(blocks: ScheduleBlockDto[]): BlockWithLayout[] {
  if (blocks.length === 0) return [];

  // Sort by start time, then by duration (longer blocks first)
  const sorted = [...blocks].sort((a, b) => {
    const startDiff = parseTime(a.start_time) - parseTime(b.start_time);
    if (startDiff !== 0) return startDiff;
    return parseTime(b.end_time) - parseTime(a.end_time);
  });

  // Group overlapping blocks together
  const groups: ScheduleBlockDto[][] = [];
  for (const block of sorted) {
    let added = false;
    for (const group of groups) {
      if (group.some((b) => blocksOverlap(b, block))) {
        group.push(block);
        added = true;
        break;
      }
    }
    if (!added) {
      groups.push([block]);
    }
  }

  // Merge groups that overlap with each other
  const mergedGroups: ScheduleBlockDto[][] = [];
  for (const group of groups) {
    let merged = false;
    for (const mGroup of mergedGroups) {
      if (group.some((b1) => mGroup.some((b2) => blocksOverlap(b1, b2)))) {
        mGroup.push(...group);
        merged = true;
        break;
      }
    }
    if (!merged) {
      mergedGroups.push([...group]);
    }
  }

  // Assign columns within each group
  const result: BlockWithLayout[] = [];
  for (const group of mergedGroups) {
    const columns: ScheduleBlockDto[][] = [];
    for (const block of group) {
      let placed = false;
      for (let col = 0; col < columns.length; col++) {
        if (!columns[col].some((b) => blocksOverlap(b, block))) {
          columns[col].push(block);
          result.push({ block, column: col, totalColumns: 0 });
          placed = true;
          break;
        }
      }
      if (!placed) {
        columns.push([block]);
        result.push({ block, column: columns.length - 1, totalColumns: 0 });
      }
    }
    // Update totalColumns for this group
    const totalCols = columns.length;
    for (const item of result) {
      if (group.includes(item.block)) {
        item.totalColumns = totalCols;
      }
    }
  }

  return result;
}

/**
 * Calculate CSS style for positioning a schedule block in a time grid.
 *
 * @param block - The schedule block
 * @param column - Column index (0-based) for overlapping blocks
 * @param totalColumns - Total number of columns in the overlap group
 * @param startHour - First hour displayed in the grid (e.g., 6 for 6am)
 * @param endHour - Last hour displayed in the grid (e.g., 22 for 10pm)
 * @returns CSS style string for positioning
 */
export function getBlockStyle(
  block: ScheduleBlockDto,
  column: number = 0,
  totalColumns: number = 1,
  startHour: number = 6,
  endHour: number = 22
): string {
  const startTime = parseTime(block.start_time);
  const endTime = parseTime(block.end_time);
  const top = ((startTime - startHour) / (endHour - startHour)) * 100;
  const height = ((endTime - startTime) / (endHour - startHour)) * 100;
  const color = block.color || "#4f6bed";

  const width = 100 / totalColumns;
  const left = column * width;

  return `top: ${top}%; height: ${height}%; left: ${left}%; width: ${width}%; background-color: ${color};`;
}

/**
 * Calculate the current time position as a percentage of the day grid.
 *
 * @param startHour - First hour displayed in the grid
 * @param endHour - Last hour displayed in the grid
 * @returns Percentage position, or null if current time is outside the grid
 */
export function getCurrentTimePosition(startHour: number = 6, endHour: number = 22): number | null {
  const now = new Date();
  const currentHour = now.getHours() + now.getMinutes() / 60;

  if (currentHour < startHour || currentHour > endHour) {
    return null;
  }

  return ((currentHour - startHour) / (endHour - startHour)) * 100;
}
