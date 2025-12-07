<script lang="ts">
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

  interface Props {
    scheduleBlocks?: ScheduleBlockDto[];
    onBlockClick?: (block: ScheduleBlockDto) => void;
    onBlockEdit?: (block: ScheduleBlockDto) => void;
    onEmptySlotClick?: (hour: number) => void;
  }

  let {
    scheduleBlocks = [],
    onBlockClick,
    onBlockEdit,
    onEmptySlotClick,
  }: Props = $props();

  // Configuration
  const startHour = 6;
  const endHour = 22;
  const hours = Array.from({ length: endHour - startHour }, (_, i) => startHour + i);

  const selectedDate = $derived(workspaceStore.selectedDate);

  const blocksForDay = $derived(() => {
    const dateKey = formatDateKey(selectedDate);
    return scheduleBlocks.filter((b) => b.date === dateKey);
  });

  function computeBlockStyle(block: ScheduleBlockDto, column: number, totalColumns: number): string {
    return getBlockStyle(block, column, totalColumns, startHour, endHour);
  }

  function goToPreviousDay() {
    workspaceStore.selectDate(getPreviousDay(selectedDate));
  }

  function goToNextDay() {
    workspaceStore.selectDate(getNextDay(selectedDate));
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
    <div class="day-column">
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
            style={computeBlockStyle(block, column, totalColumns)}
            onclick={() => onBlockClick?.(block)}
            onkeydown={(e) => e.key === "Enter" && onBlockClick?.(block)}
            role="button"
            tabindex="0"
          >
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
          </div>
        {/each}
      </div>

      <!-- Current time indicator -->
      {#if isToday(selectedDate)}
        {@const now = new Date()}
        {@const currentHour = now.getHours() + now.getMinutes() / 60}
        {#if currentHour >= startHour && currentHour <= endHour}
          <div
            class="current-time-line"
            style="top: {((currentHour - startHour) / (endHour - startHour)) * 100}%"
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
