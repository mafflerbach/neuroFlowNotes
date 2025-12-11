/**
 * Calendar Drag & Drop / Resize Utilities
 * Shared logic for CalendarWeekly and CalendarDaily components.
 */

import type { ScheduleBlockDto } from "../types";
import { formatDateKey } from "./dateUtils";

/**
 * Time calculation configuration
 */
export interface TimeConfig {
  startHour: number;
  endHour: number;
  hourSlotHeight: number;
}

/**
 * Result of a time calculation from Y position
 */
export interface TimeFromPosition {
  hour: number;
  minute: number;
  totalMinutes: number;
  timeString: string;
}

/**
 * Calculate time from a Y position in a calendar column.
 * Snaps to 15-minute intervals.
 */
export function calculateTimeFromY(
  y: number,
  config: TimeConfig
): TimeFromPosition {
  const { startHour, endHour, hourSlotHeight } = config;

  const rawHour = y / hourSlotHeight + startHour;
  const hour = Math.floor(rawHour);
  const minuteFraction = rawHour - hour;
  const minute = Math.floor(minuteFraction * 4) * 15; // 0, 15, 30, 45

  const totalMinutes = hour * 60 + minute;
  const clampedMinutes = Math.max(
    startHour * 60,
    Math.min(endHour * 60, totalMinutes)
  );

  const clampedHour = Math.floor(clampedMinutes / 60);
  const clampedMin = clampedMinutes % 60;
  const timeString = `${clampedHour.toString().padStart(2, "0")}:${clampedMin.toString().padStart(2, "0")}:00`;

  return {
    hour: clampedHour,
    minute: clampedMin,
    totalMinutes: clampedMinutes,
    timeString,
  };
}

/**
 * Calculate new start/end times when dropping a block.
 * Preserves the block's duration while moving it to a new time.
 */
export function calculateDropTimes(
  block: ScheduleBlockDto,
  dropY: number,
  config: TimeConfig
): { newStartTime: string; newEndTime: string } {
  const { startHour, endHour, hourSlotHeight } = config;

  // Calculate target time from drop position
  const rawHour = dropY / hourSlotHeight + startHour;
  const hour = Math.floor(rawHour);
  const minuteFraction = rawHour - hour;
  const minute = Math.floor(minuteFraction * 4) * 15;

  // Calculate block duration
  const [origStartH, origStartM] = block.start_time.split(":").map(Number);
  const [origEndH, origEndM] = block.end_time.split(":").map(Number);
  const durationMinutes = origEndH * 60 + origEndM - (origStartH * 60 + origStartM);

  // Calculate new times and clamp to valid range
  const newStartMinutes = hour * 60 + minute;
  const clampedStart = Math.max(
    startHour * 60,
    Math.min((endHour - 1) * 60, newStartMinutes)
  );
  const clampedEnd = Math.min(endHour * 60, clampedStart + durationMinutes);

  const newStartHour = Math.floor(clampedStart / 60);
  const newStartMin = clampedStart % 60;
  const newEndHour = Math.floor(clampedEnd / 60);
  const newEndMin = clampedEnd % 60;

  const newStartTime = `${newStartHour.toString().padStart(2, "0")}:${newStartMin.toString().padStart(2, "0")}:00`;
  const newEndTime = `${newEndHour.toString().padStart(2, "0")}:${newEndMin.toString().padStart(2, "0")}:00`;

  return { newStartTime, newEndTime };
}

/**
 * Calculate new times when resizing a block.
 * Returns null if the resize would result in invalid times (start >= end).
 */
export function calculateResizeTimes(
  block: ScheduleBlockDto,
  resizeY: number,
  edge: "top" | "bottom",
  config: TimeConfig
): { newStartTime: string; newEndTime: string } | null {
  const { startHour, endHour, hourSlotHeight } = config;

  // Calculate new time from Y position
  const rawHour = resizeY / hourSlotHeight + startHour;
  const hour = Math.floor(rawHour);
  const minuteFraction = rawHour - hour;
  const minute = Math.floor(minuteFraction * 4) * 15;

  const newMinutes = hour * 60 + minute;
  const clampedMinutes = Math.max(
    startHour * 60,
    Math.min(endHour * 60, newMinutes)
  );

  const newHour = Math.floor(clampedMinutes / 60);
  const newMin = clampedMinutes % 60;
  const newTime = `${newHour.toString().padStart(2, "0")}:${newMin.toString().padStart(2, "0")}:00`;

  if (edge === "top") {
    // Resizing top edge - changing start time
    const [endH, endM] = block.end_time.split(":").map(Number);
    if (clampedMinutes >= endH * 60 + endM) {
      return null; // Start would be >= end
    }
    return { newStartTime: newTime, newEndTime: block.end_time };
  } else {
    // Resizing bottom edge - changing end time
    const [startH, startM] = block.start_time.split(":").map(Number);
    if (clampedMinutes <= startH * 60 + startM) {
      return null; // End would be <= start
    }
    return { newStartTime: block.start_time, newEndTime: newTime };
  }
}

/**
 * State manager for calendar drag & drop operations.
 * Use this class to manage drag state in calendar components.
 */
export class CalendarDragState {
  draggedBlock: ScheduleBlockDto | null = null;
  dropTargetDate: Date | null = null;

  startDrag(e: DragEvent, block: ScheduleBlockDto): void {
    if (!e.dataTransfer) return;
    this.draggedBlock = block;
    e.dataTransfer.effectAllowed = "move";
    e.dataTransfer.setData("text/plain", block.id.toString());
  }

  endDrag(): void {
    this.draggedBlock = null;
    this.dropTargetDate = null;
  }

  setDropTarget(date: Date | null): void {
    this.dropTargetDate = date;
  }

  isDragging(): boolean {
    return this.draggedBlock !== null;
  }

  isBlockBeingDragged(blockId: number): boolean {
    return this.draggedBlock?.id === blockId;
  }

  isDropTargetDate(date: Date): boolean {
    if (!this.dropTargetDate) return false;
    return formatDateKey(date) === formatDateKey(this.dropTargetDate);
  }
}

/**
 * State manager for calendar resize operations.
 * Use this class to manage resize state in calendar components.
 */
export class CalendarResizeState {
  resizingBlock: ScheduleBlockDto | null = null;
  resizeEdge: "top" | "bottom" | null = null;

  private moveHandler: ((e: MouseEvent) => void) | null = null;
  private endHandler: ((e: MouseEvent) => void) | null = null;

  startResize(
    e: MouseEvent,
    block: ScheduleBlockDto,
    edge: "top" | "bottom",
    onMove?: (e: MouseEvent) => void,
    onEnd?: (e: MouseEvent) => void
  ): void {
    e.preventDefault();
    e.stopPropagation();

    this.resizingBlock = block;
    this.resizeEdge = edge;

    // Store handlers for cleanup
    this.moveHandler = onMove || (() => {});
    this.endHandler = (e: MouseEvent) => {
      onEnd?.(e);
      this.cleanup();
    };

    window.addEventListener("mousemove", this.moveHandler);
    window.addEventListener("mouseup", this.endHandler);
  }

  cleanup(): void {
    if (this.moveHandler) {
      window.removeEventListener("mousemove", this.moveHandler);
    }
    if (this.endHandler) {
      window.removeEventListener("mouseup", this.endHandler);
    }
    this.resizingBlock = null;
    this.resizeEdge = null;
    this.moveHandler = null;
    this.endHandler = null;
  }

  isResizing(): boolean {
    return this.resizingBlock !== null;
  }

  isBlockBeingResized(blockId: number): boolean {
    return this.resizingBlock?.id === blockId;
  }
}

/**
 * Get the Y offset relative to a column element from a mouse event.
 */
export function getColumnY(e: MouseEvent, columnSelector: string): number | null {
  const columns = document.querySelectorAll(columnSelector);

  for (const col of columns) {
    const rect = col.getBoundingClientRect();
    if (e.clientX >= rect.left && e.clientX <= rect.right) {
      return e.clientY - rect.top;
    }
  }

  return null;
}

/**
 * Get the Y offset from a specific element's bounding rect.
 */
export function getYFromElement(e: MouseEvent | DragEvent, element: HTMLElement): number {
  const rect = element.getBoundingClientRect();
  return e.clientY - rect.top;
}
