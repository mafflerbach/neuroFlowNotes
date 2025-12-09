<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { ChevronLeft, ChevronRight, Pencil } from "lucide-svelte";
  import { workspaceStore } from "../stores/workspace.svelte";
  import type { ScheduleBlockDto } from "../types";
  import {
    formatDateKey,
    isToday,
    getPreviousDay,
    getNextDay,
    DAY_NAMES_FULL,
    MONTH_NAMES_FULL,
  } from "../utils/dateUtils";
  import {
    getBlocksWithLayout,
    getBlockStyle,
    formatTimeShort,
  } from "../utils/blockLayoutUtils";
  import { CALENDAR_CONFIG, getCalendarHours } from "../constants/calendar";

  interface Props {
    scheduleBlocks?: ScheduleBlockDto[];
    onBlockClick?: (block: ScheduleBlockDto) => void;
    onBlockEdit?: (block: ScheduleBlockDto) => void;
    onEmptySlotClick?: (hour: number) => void;
    onBlockMove?: (block: ScheduleBlockDto, newDate: string, newStartTime: string, newEndTime: string) => void;
  }

  let {
    scheduleBlocks = [],
    onBlockClick,
    onBlockEdit,
    onEmptySlotClick,
    onBlockMove,
  }: Props = $props();

  // Configuration from constants
  const { START_HOUR: startHour, END_HOUR: endHour, HOUR_SLOT_HEIGHT_DAILY: hourSlotHeight } = CALENDAR_CONFIG;
  const hours = getCalendarHours();

  const selectedDate = $derived(workspaceStore.selectedDate);

  const blocksForDay = $derived(() => {
    const dateKey = formatDateKey(selectedDate);
    return scheduleBlocks.filter((b) => b.date === dateKey);
  });

  function computeBlockStyle(block: ScheduleBlockDto, column: number, totalColumns: number): string {
    return getBlockStyle(block, column, totalColumns, startHour, endHour, hourSlotHeight);
  }

  function goToPreviousDay() {
    workspaceStore.selectDate(getPreviousDay(selectedDate));
  }

  function goToNextDay() {
    workspaceStore.selectDate(getNextDay(selectedDate));
  }

  // Current time for red line indicator (updates every minute)
  let currentTime = $state(new Date());
  let timeUpdateInterval: ReturnType<typeof setInterval> | null = null;

  onMount(() => {
    // Update current time every minute
    timeUpdateInterval = setInterval(() => {
      currentTime = new Date();
    }, 60000); // 60 seconds
  });

  onDestroy(() => {
    if (timeUpdateInterval) {
      clearInterval(timeUpdateInterval);
    }
  });

  // Drag & Drop state
  let draggedBlock = $state<ScheduleBlockDto | null>(null);
  let isDropTarget = $state(false);

  // Resize state
  let resizingBlock = $state<ScheduleBlockDto | null>(null);
  let resizeEdge = $state<"top" | "bottom" | null>(null);

  function handleBlockDragStart(e: DragEvent, block: ScheduleBlockDto) {
    if (!e.dataTransfer) return;
    draggedBlock = block;
    e.dataTransfer.effectAllowed = "move";
    e.dataTransfer.setData("text/plain", block.id.toString());
  }

  function handleBlockDragEnd() {
    draggedBlock = null;
    isDropTarget = false;
  }

  function handleColumnDragOver(e: DragEvent) {
    if (!draggedBlock) return;
    e.preventDefault();

    if (e.dataTransfer) {
      e.dataTransfer.dropEffect = "move";
    }

    isDropTarget = true;
  }

  function handleColumnDragLeave(e: DragEvent) {
    const relatedTarget = e.relatedTarget as HTMLElement | null;
    if (relatedTarget && (e.currentTarget as HTMLElement).contains(relatedTarget)) {
      return;
    }
    isDropTarget = false;
  }

  function handleColumnDrop(e: DragEvent) {
    e.preventDefault();

    if (!draggedBlock || !onBlockMove) {
      handleBlockDragEnd();
      return;
    }

    // Calculate new time from mouse position
    const target = e.currentTarget as HTMLElement;
    const rect = target.getBoundingClientRect();
    const y = e.clientY - rect.top;

    // Calculate hour and minute (snap to 15 min intervals)
    const rawHour = y / hourSlotHeight + startHour;
    const hour = Math.floor(rawHour);
    const minuteFraction = rawHour - hour;
    const minute = Math.floor(minuteFraction * 4) * 15; // 0, 15, 30, 45

    // Calculate block duration
    const [origStartH, origStartM] = draggedBlock.start_time.split(":").map(Number);
    const [origEndH, origEndM] = draggedBlock.end_time.split(":").map(Number);
    const durationMinutes = (origEndH * 60 + origEndM) - (origStartH * 60 + origStartM);

    // Calculate new times and clamp to valid range
    const newStartMinutes = hour * 60 + minute;
    const clampedStart = Math.max(startHour * 60, Math.min((endHour - 1) * 60, newStartMinutes));
    const clampedEnd = Math.min(endHour * 60, clampedStart + durationMinutes);

    const newStartHour = Math.floor(clampedStart / 60);
    const newStartMin = clampedStart % 60;
    const newEndHour = Math.floor(clampedEnd / 60);
    const newEndMin = clampedEnd % 60;

    const newStartTime = `${newStartHour.toString().padStart(2, "0")}:${newStartMin.toString().padStart(2, "0")}:00`;
    const newEndTime = `${newEndHour.toString().padStart(2, "0")}:${newEndMin.toString().padStart(2, "0")}:00`;
    const newDate = formatDateKey(selectedDate);

    onBlockMove(draggedBlock, newDate, newStartTime, newEndTime);

    handleBlockDragEnd();
  }

  // Resize handlers
  function handleResizeStart(e: MouseEvent, block: ScheduleBlockDto, edge: "top" | "bottom") {
    e.preventDefault();
    e.stopPropagation();

    resizingBlock = block;
    resizeEdge = edge;

    // Add global listeners
    window.addEventListener("mousemove", handleResizeMove);
    window.addEventListener("mouseup", handleResizeEnd);
  }

  function handleResizeMove(_e: MouseEvent) {
    // Visual feedback during resize (optional)
  }

  function handleResizeEnd(e: MouseEvent) {
    if (!resizingBlock || !resizeEdge || !onBlockMove) {
      cleanupResize();
      return;
    }

    // Find the day column to get the Y offset
    const dayColumn = document.querySelector(".day-column");
    if (!dayColumn) {
      cleanupResize();
      return;
    }

    const columnRect = dayColumn.getBoundingClientRect();
    const y = e.clientY - columnRect.top;

    // Calculate new time (snap to 15 min intervals)
    const rawHour = y / hourSlotHeight + startHour;
    const hour = Math.floor(rawHour);
    const minuteFraction = rawHour - hour;
    const minute = Math.floor(minuteFraction * 4) * 15;

    const newMinutes = hour * 60 + minute;
    const clampedMinutes = Math.max(startHour * 60, Math.min(endHour * 60, newMinutes));

    const newHour = Math.floor(clampedMinutes / 60);
    const newMin = clampedMinutes % 60;
    const newTime = `${newHour.toString().padStart(2, "0")}:${newMin.toString().padStart(2, "0")}:00`;

    let newStartTime: string;
    let newEndTime: string;

    if (resizeEdge === "top") {
      newStartTime = newTime;
      newEndTime = resizingBlock.end_time;
      const [endH, endM] = newEndTime.split(":").map(Number);
      if (clampedMinutes >= endH * 60 + endM) {
        cleanupResize();
        return;
      }
    } else {
      newStartTime = resizingBlock.start_time;
      newEndTime = newTime;
      const [startH, startM] = newStartTime.split(":").map(Number);
      if (clampedMinutes <= startH * 60 + startM) {
        cleanupResize();
        return;
      }
    }

    onBlockMove(resizingBlock, resizingBlock.date, newStartTime, newEndTime);
    cleanupResize();
  }

  function cleanupResize() {
    resizingBlock = null;
    resizeEdge = null;
    window.removeEventListener("mousemove", handleResizeMove);
    window.removeEventListener("mouseup", handleResizeEnd);
  }
</script>

<div class="calendar-daily">
  <!-- Header with date navigation -->
  <div class="day-header">
    <button class="nav-arrow" onclick={goToPreviousDay} aria-label="Previous day">
      <ChevronLeft size={20} />
    </button>

    <div class="date-display" class:is-today={isToday(selectedDate)}>
      <span class="day-name">{DAY_NAMES_FULL[selectedDate.getDay()]}</span>
      <span class="date-full">
        {MONTH_NAMES_FULL[selectedDate.getMonth()]} {selectedDate.getDate()}, {selectedDate.getFullYear()}
      </span>
    </div>

    <button class="nav-arrow" onclick={goToNextDay} aria-label="Next day">
      <ChevronRight size={20} />
    </button>
  </div>

  <!-- Time grid -->
  <div class="day-grid">
    <!-- Time labels -->
    <div class="time-gutter">
      {#each hours as hour}
        <div class="time-label">
          {hour.toString().padStart(2, "0")}:00
        </div>
      {/each}
    </div>

    <!-- Main column -->
    <div
      class="day-column"
      class:is-drop-target={isDropTarget}
      ondragover={handleColumnDragOver}
      ondragleave={handleColumnDragLeave}
      ondrop={handleColumnDrop}
      role="region"
    >
      <!-- Hour slots -->
      {#each hours as hour}
        <button
          class="hour-slot"
          onclick={() => onEmptySlotClick?.(hour)}
        >
          <span class="slot-time">{hour}:00</span>
        </button>
      {/each}

      <!-- Schedule blocks overlay -->
      <div class="blocks-container">
        {#each getBlocksWithLayout(blocksForDay()) as { block, column, totalColumns } (block.id)}
          <div
            class="schedule-block"
            class:is-dragging={draggedBlock?.id === block.id}
            class:is-resizing={resizingBlock?.id === block.id}
            style={computeBlockStyle(block, column, totalColumns)}
            draggable="true"
            onclick={() => onBlockClick?.(block)}
            onkeydown={(e) => e.key === "Enter" && onBlockClick?.(block)}
            ondragstart={(e) => handleBlockDragStart(e, block)}
            ondragend={handleBlockDragEnd}
            role="button"
            tabindex="0"
          >
            <!-- Top resize handle -->
            <!-- svelte-ignore a11y_no_static_element_interactions -->
            <div
              class="resize-handle resize-handle-top"
              onmousedown={(e) => handleResizeStart(e, block, "top")}
            ></div>
            <div class="block-content">
              <span class="block-time">
                {formatTimeShort(block.start_time)} - {formatTimeShort(block.end_time)}
              </span>
              {#if block.label}
                <span class="block-label">{block.label}</span>
              {/if}
              {#if block.context}
                <span class="block-context">{block.context}</span>
              {/if}
            </div>
            <button
              class="block-edit-btn"
              onclick={(e) => {
                e.stopPropagation();
                onBlockEdit?.(block);
              }}
              title="Edit block"
            >
              <Pencil size={14} />
            </button>
            <!-- Bottom resize handle -->
            <!-- svelte-ignore a11y_no_static_element_interactions -->
            <div
              class="resize-handle resize-handle-bottom"
              onmousedown={(e) => handleResizeStart(e, block, "bottom")}
            ></div>
          </div>
        {/each}
      </div>

      <!-- Current time indicator (updates every minute) -->
      {#if isToday(selectedDate)}
        {@const currentHour = currentTime.getHours() + currentTime.getMinutes() / 60}
        {#if currentHour >= startHour && currentHour <= endHour}
          <div
            class="current-time-line"
            style="top: {(currentHour - startHour) * hourSlotHeight}px"
          >
            <div class="current-time-dot"></div>
          </div>
        {/if}
      {/if}
    </div>
  </div>
</div>

<style>
  .calendar-daily {
    display: flex;
    flex-direction: column;
    height: 100%;
    background: var(--calendar-bg);
  }

  .day-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: var(--spacing-3) var(--spacing-4);
    border-bottom: 1px solid var(--calendar-border);
  }

  .nav-arrow {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 36px;
    height: 36px;
    border: none;
    background: transparent;
    border-radius: var(--radius-lg);
    color: var(--text-primary);
    cursor: pointer;
  }

  .nav-arrow:hover {
    background: var(--bg-hover);
  }

  .date-display {
    text-align: center;
  }

  .date-display.is-today .day-name {
    color: var(--color-primary);
  }

  .day-name {
    display: block;
    font-size: var(--font-size-md);
    font-weight: var(--font-weight-semibold);
    color: var(--text-primary);
  }

  .date-full {
    font-size: var(--font-size-sm);
    color: var(--text-muted);
  }

  .day-grid {
    display: flex;
    flex: 1;
    overflow-y: auto;
  }

  .time-gutter {
    width: 60px;
    flex-shrink: 0;
    display: flex;
    flex-direction: column;
  }

  .time-label {
    height: 60px;
    display: flex;
    align-items: flex-start;
    justify-content: flex-end;
    padding-right: var(--spacing-3);
    font-size: var(--font-size-sm);
    color: var(--text-muted);
    transform: translateY(-8px);
  }

  .day-column {
    flex: 1;
    position: relative;
    border-left: 1px solid var(--calendar-border);
  }

  .day-column.is-drop-target {
    background: var(--color-primary-light);
    outline: 2px dashed var(--color-primary);
    outline-offset: -2px;
  }

  .hour-slot {
    height: 60px;
    width: 100%;
    border: none;
    background: transparent;
    border-bottom: 1px solid var(--border-light);
    cursor: pointer;
    text-align: left;
    padding: var(--spacing-1) var(--spacing-2);
  }

  .hour-slot:hover {
    background: var(--bg-hover);
  }

  .slot-time {
    font-size: var(--font-size-xs);
    color: var(--text-muted);
    opacity: 0;
  }

  .hour-slot:hover .slot-time {
    opacity: 1;
  }

  .blocks-container {
    position: absolute;
    top: 0;
    left: var(--spacing-2);
    right: var(--spacing-2);
    bottom: 0;
    pointer-events: none;
  }

  .schedule-block {
    position: absolute;
    /* left and width set dynamically via style for overlapping blocks */
    border: none;
    border-radius: var(--radius-md);
    padding: var(--spacing-2) var(--spacing-3);
    color: var(--block-default-text);
    cursor: pointer;
    text-align: left;
    pointer-events: auto;
    opacity: 0.95;
    overflow: hidden;
    box-sizing: border-box;
  }

  .schedule-block:hover {
    opacity: 1;
    box-shadow: var(--shadow-lg);
  }

  .schedule-block.is-dragging {
    opacity: 0.5;
    cursor: grabbing;
  }

  .schedule-block.is-resizing {
    opacity: 0.8;
    box-shadow: var(--shadow-lg);
  }

  .resize-handle {
    position: absolute;
    left: 0;
    right: 0;
    height: 8px;
    cursor: ns-resize;
    opacity: 0;
    transition: opacity var(--transition-normal);
  }

  .resize-handle-top {
    top: 0;
    border-radius: var(--radius-md) var(--radius-md) 0 0;
  }

  .resize-handle-bottom {
    bottom: 0;
    border-radius: 0 0 var(--radius-md) var(--radius-md);
  }

  .schedule-block:hover .resize-handle {
    opacity: 1;
    background: rgba(0, 0, 0, 0.2);
  }

  .resize-handle:hover {
    background: rgba(0, 0, 0, 0.4) !important;
  }

  .schedule-block:hover .block-edit-btn {
    opacity: 1;
  }

  .block-edit-btn {
    position: absolute;
    top: var(--spacing-2);
    right: var(--spacing-2);
    width: 24px;
    height: 24px;
    display: flex;
    align-items: center;
    justify-content: center;
    background: rgba(0, 0, 0, 0.2);
    border: none;
    border-radius: var(--radius-sm);
    cursor: pointer;
    opacity: 0;
    transition: opacity var(--transition-normal), background var(--transition-normal);
    color: inherit;
  }

  .block-edit-btn:hover {
    background: rgba(0, 0, 0, 0.4);
  }

  .block-content {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .block-time {
    font-size: var(--font-size-sm);
    font-weight: var(--font-weight-semibold);
  }

  .block-label {
    font-size: var(--font-size-md);
    font-weight: var(--font-weight-medium);
  }

  .block-context {
    font-size: var(--font-size-xs);
    opacity: 0.9;
  }

  .current-time-line {
    position: absolute;
    left: 0;
    right: 0;
    height: 2px;
    background: var(--color-error);
    z-index: 10;
  }

  .current-time-dot {
    position: absolute;
    left: -5px;
    top: -4px;
    width: 10px;
    height: 10px;
    background: var(--color-error);
    border-radius: var(--radius-full);
  }
</style>
