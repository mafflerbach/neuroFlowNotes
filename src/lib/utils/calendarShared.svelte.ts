/**
 * Shared calendar state management utilities.
 * Uses Svelte 5 runes for reactive state.
 */

import type { ScheduleBlockDto } from "../types";
import {
  calculateDropTimes,
  calculateResizeTimes,
  getYFromElement,
  getColumnY,
  type TimeConfig,
} from "./calendarDragDrop";
import { formatDateKey } from "./dateUtils";

/**
 * Creates drag & drop state management for calendar components.
 * Returns reactive state and handlers for block dragging.
 */
export function createDragState() {
  let draggedBlock = $state<ScheduleBlockDto | null>(null);
  let dropTargetDate = $state<Date | null>(null);

  function startDrag(e: DragEvent, block: ScheduleBlockDto) {
    if (!e.dataTransfer) return;
    draggedBlock = block;
    e.dataTransfer.effectAllowed = "move";
    e.dataTransfer.setData("text/plain", block.id.toString());
  }

  function endDrag() {
    draggedBlock = null;
    dropTargetDate = null;
  }

  function setDropTarget(date: Date | null) {
    dropTargetDate = date;
  }

  function handleDragOver(e: DragEvent, date?: Date) {
    if (!draggedBlock) return;
    e.preventDefault();
    if (e.dataTransfer) {
      e.dataTransfer.dropEffect = "move";
    }
    dropTargetDate = date ?? null;
  }

  function handleDragLeave(e: DragEvent) {
    const relatedTarget = e.relatedTarget as HTMLElement | null;
    if (relatedTarget && (e.currentTarget as HTMLElement).contains(relatedTarget)) {
      return;
    }
    dropTargetDate = null;
  }

  function handleDrop(
    e: DragEvent,
    date: Date,
    timeConfig: TimeConfig,
    onMove?: (block: ScheduleBlockDto, newDate: string, newStartTime: string, newEndTime: string) => void
  ) {
    e.preventDefault();

    if (!draggedBlock || !onMove) {
      endDrag();
      return;
    }

    const y = getYFromElement(e, e.currentTarget as HTMLElement);
    const { newStartTime, newEndTime } = calculateDropTimes(draggedBlock, y, timeConfig);
    const newDate = formatDateKey(date);

    onMove(draggedBlock, newDate, newStartTime, newEndTime);
    endDrag();
  }

  return {
    get draggedBlock() { return draggedBlock; },
    get dropTargetDate() { return dropTargetDate; },
    startDrag,
    endDrag,
    setDropTarget,
    handleDragOver,
    handleDragLeave,
    handleDrop,
    isBlockDragging: (blockId: number) => draggedBlock?.id === blockId,
    isDropTarget: (date: Date) => dropTargetDate !== null && formatDateKey(date) === formatDateKey(dropTargetDate),
  };
}

/**
 * Creates resize state management for calendar components.
 * Returns reactive state and handlers for block resizing.
 */
export function createResizeState() {
  let resizingBlock = $state<ScheduleBlockDto | null>(null);
  let resizeEdge = $state<"top" | "bottom" | null>(null);

  let moveHandler: ((e: MouseEvent) => void) | null = null;
  let endHandler: ((e: MouseEvent) => void) | null = null;

  function startResize(
    e: MouseEvent,
    block: ScheduleBlockDto,
    edge: "top" | "bottom",
    columnSelector: string,
    timeConfig: TimeConfig,
    onMove?: (block: ScheduleBlockDto, newDate: string, newStartTime: string, newEndTime: string) => void
  ) {
    e.preventDefault();
    e.stopPropagation();

    resizingBlock = block;
    resizeEdge = edge;

    moveHandler = (_moveEvent: MouseEvent) => {
      // Visual feedback during resize (optional)
    };

    endHandler = (endEvent: MouseEvent) => {
      if (!resizingBlock || !resizeEdge || !onMove) {
        cleanup();
        return;
      }

      const y = getColumnY(endEvent, columnSelector);
      if (y === null) {
        cleanup();
        return;
      }

      const result = calculateResizeTimes(resizingBlock, y, resizeEdge, timeConfig);
      if (result) {
        onMove(resizingBlock, resizingBlock.date, result.newStartTime, result.newEndTime);
      }

      cleanup();
    };

    window.addEventListener("mousemove", moveHandler);
    window.addEventListener("mouseup", endHandler);
  }

  function cleanup() {
    if (moveHandler) {
      window.removeEventListener("mousemove", moveHandler);
    }
    if (endHandler) {
      window.removeEventListener("mouseup", endHandler);
    }
    resizingBlock = null;
    resizeEdge = null;
    moveHandler = null;
    endHandler = null;
  }

  return {
    get resizingBlock() { return resizingBlock; },
    get resizeEdge() { return resizeEdge; },
    startResize,
    cleanup,
    isBlockResizing: (blockId: number) => resizingBlock?.id === blockId,
  };
}

/**
 * Configuration for calendar block appearance.
 */
export interface BlockConfig {
  /** Size variant for different calendar views */
  size: "small" | "medium" | "large";
  /** Whether to show the context field */
  showContext?: boolean;
  /** Whether to show the time range */
  showTime?: boolean;
}

/**
 * Default configurations for different calendar views.
 */
export const BLOCK_CONFIGS = {
  daily: {
    size: "large",
    showContext: true,
    showTime: true,
  } as BlockConfig,
  weekly: {
    size: "medium",
    showContext: false,
    showTime: true,
  } as BlockConfig,
  monthly: {
    size: "small",
    showContext: false,
    showTime: false,
  } as BlockConfig,
};
