<script lang="ts">
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
    <button class="nav-arrow" onclick={goToPreviousDay}>
      <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M15 18l-6-6 6-6" />
      </svg>
    </button>

    <div class="date-display" class:is-today={isToday(selectedDate)}>
      <span class="day-name">{DAY_NAMES_FULL[selectedDate.getDay()]}</span>
      <span class="date-full">
        {MONTH_NAMES_FULL[selectedDate.getMonth()]} {selectedDate.getDate()}, {selectedDate.getFullYear()}
      </span>
    </div>

    <button class="nav-arrow" onclick={goToNextDay}>
      <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M9 18l6-6-6-6" />
      </svg>
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
              <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7" />
                <path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z" />
              </svg>
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
    background: var(--calendar-bg, #fff);
  }

  .day-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 12px 16px;
    border-bottom: 1px solid var(--border-color, #e0e0e0);
  }

  .nav-arrow {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 36px;
    height: 36px;
    border: none;
    background: transparent;
    border-radius: 8px;
    color: var(--text-color, #333);
    cursor: pointer;
  }

  .nav-arrow:hover {
    background: var(--hover-bg, #f0f0f0);
  }

  .date-display {
    text-align: center;
  }

  .date-display.is-today .day-name {
    color: var(--primary-color, #4f6bed);
  }

  .day-name {
    display: block;
    font-size: 14px;
    font-weight: 600;
    color: var(--text-color, #333);
  }

  .date-full {
    font-size: 12px;
    color: var(--text-muted, #666);
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
    padding-right: 12px;
    font-size: 12px;
    color: var(--text-muted, #666);
    transform: translateY(-8px);
  }

  .day-column {
    flex: 1;
    position: relative;
    border-left: 1px solid var(--border-color, #e0e0e0);
  }

  .hour-slot {
    height: 60px;
    width: 100%;
    border: none;
    background: transparent;
    border-bottom: 1px solid var(--border-light, #f0f0f0);
    cursor: pointer;
    text-align: left;
    padding: 4px 8px;
  }

  .hour-slot:hover {
    background: var(--hover-bg, #f5f5f5);
  }

  .slot-time {
    font-size: 10px;
    color: var(--text-muted, #999);
    opacity: 0;
  }

  .hour-slot:hover .slot-time {
    opacity: 1;
  }

  .blocks-container {
    position: absolute;
    top: 0;
    left: 8px;
    right: 8px;
    bottom: 0;
    pointer-events: none;
  }

  .schedule-block {
    position: absolute;
    /* left and width set dynamically via style for overlapping blocks */
    border: none;
    border-radius: 6px;
    padding: 8px 12px;
    color: white;
    cursor: pointer;
    text-align: left;
    pointer-events: auto;
    opacity: 0.95;
    overflow: hidden;
    box-sizing: border-box;
  }

  .schedule-block:hover {
    opacity: 1;
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
  }

  .schedule-block:hover .block-edit-btn {
    opacity: 1;
  }

  .block-edit-btn {
    position: absolute;
    top: 6px;
    right: 6px;
    width: 24px;
    height: 24px;
    display: flex;
    align-items: center;
    justify-content: center;
    background: rgba(0, 0, 0, 0.2);
    border: none;
    border-radius: 4px;
    cursor: pointer;
    opacity: 0;
    transition: opacity 0.15s, background 0.15s;
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
    font-size: 12px;
    font-weight: 600;
  }

  .block-label {
    font-size: 14px;
    font-weight: 500;
  }

  .block-context {
    font-size: 11px;
    opacity: 0.9;
  }

  .current-time-line {
    position: absolute;
    left: 0;
    right: 0;
    height: 2px;
    background: var(--error-color, #d32f2f);
    z-index: 10;
  }

  .current-time-dot {
    position: absolute;
    left: -5px;
    top: -4px;
    width: 10px;
    height: 10px;
    background: var(--error-color, #d32f2f);
    border-radius: 50%;
  }
</style>
