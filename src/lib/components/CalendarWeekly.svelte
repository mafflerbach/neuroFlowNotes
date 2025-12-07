<script lang="ts">
  import { ChevronLeft, ChevronRight, Pencil } from "lucide-svelte";
  import { workspaceStore } from "../stores/workspace.svelte";
  import type { ScheduleBlockDto, NoteListItem } from "../types";
  import {
    formatDateKey,
    isToday,
    isSameDay,
    getWeekDates,
    formatWeekRangeWithYear,
    getPreviousWeek,
    getNextWeek,
    DAY_NAMES_SHORT,
  } from "../utils/dateUtils";
  import {
    getBlocksWithLayout,
    getBlockStyle,
    formatTimeShort,
  } from "../utils/blockLayoutUtils";

  interface Props {
    scheduleBlocks?: ScheduleBlockDto[];
    notesForWeek?: Map<string, NoteListItem[]>;
    onBlockClick?: (block: ScheduleBlockDto) => void;
    onBlockEdit?: (block: ScheduleBlockDto) => void;
    onNoteClick?: (note: NoteListItem, date: Date) => void;
    onEmptySlotClick?: (date: Date, hour: number) => void;
  }

  let {
    scheduleBlocks = [],
    notesForWeek = new Map(),
    onBlockClick,
    onBlockEdit,
    onNoteClick,
    onEmptySlotClick,
  }: Props = $props();

  // Configuration
  const startHour = 6;
  const endHour = 22;
  const hours = Array.from({ length: endHour - startHour }, (_, i) => startHour + i);

  const selectedDate = $derived(workspaceStore.selectedDate);
  const weekDates = $derived(() => getWeekDates(selectedDate));
  const weekRange = $derived(() => formatWeekRangeWithYear(selectedDate));

  function goToPreviousWeek() {
    workspaceStore.selectDate(getPreviousWeek(selectedDate));
  }

  function goToNextWeek() {
    workspaceStore.selectDate(getNextWeek(selectedDate));
  }

  function isSelected(date: Date): boolean {
    return isSameDay(date, selectedDate);
  }

  function getBlocksForDate(date: Date): ScheduleBlockDto[] {
    const dateKey = formatDateKey(date);
    return scheduleBlocks.filter((b) => b.date === dateKey);
  }

  function getNotesForDate(date: Date): NoteListItem[] {
    const dateKey = formatDateKey(date);
    return notesForWeek.get(dateKey) || [];
  }

  function computeBlockStyle(block: ScheduleBlockDto, column: number, totalColumns: number): string {
    return getBlockStyle(block, column, totalColumns, startHour, endHour);
  }

  function handleDayClick(date: Date) {
    workspaceStore.selectDate(date);
  }

  function handleSlotClick(date: Date, hour: number) {
    onEmptySlotClick?.(date, hour);
  }
</script>

<div class="calendar-weekly">
  <!-- Navigation header -->
  <div class="week-nav">
    <button class="nav-arrow" onclick={goToPreviousWeek} aria-label="Previous week">
      <ChevronLeft size={20} />
    </button>
    <span class="week-range">{weekRange()}</span>
    <button class="nav-arrow" onclick={goToNextWeek} aria-label="Next week">
      <ChevronRight size={20} />
    </button>
  </div>

  <!-- Header row with day names and dates -->
  <div class="week-header">
    <div class="time-gutter"></div>
    {#each weekDates() as date, i}
      <button
        class="day-header"
        class:is-today={isToday(date)}
        class:is-selected={isSelected(date)}
        onclick={() => handleDayClick(date)}
      >
        <span class="day-name">{DAY_NAMES_SHORT[i]}</span>
        <span class="day-number">{date.getDate()}</span>
      </button>
    {/each}
  </div>

  <!-- Time grid -->
  <div class="week-grid">
    <!-- Time labels column -->
    <div class="time-gutter">
      {#each hours as hour}
        <div class="time-label">
          {hour.toString().padStart(2, "0")}:00
        </div>
      {/each}
    </div>

    <!-- Day columns -->
    {#each weekDates() as date, dayIndex}
      <div
        class="day-column"
        class:is-today={isToday(date)}
        class:is-weekend={dayIndex >= 5}
      >
        <!-- Hour slots -->
        {#each hours as hour}
          <button
            class="hour-slot"
            onclick={() => handleSlotClick(date, hour)}
            aria-label="Create event at {hour}:00"
          ></button>
        {/each}

        <!-- Schedule blocks overlay -->
        <div class="blocks-container">
          {#each getBlocksWithLayout(getBlocksForDate(date)) as { block, column, totalColumns } (block.id)}
            <div
              class="schedule-block"
              style={computeBlockStyle(block, column, totalColumns)}
              onclick={() => onBlockClick?.(block)}
              onkeydown={(e) => e.key === "Enter" && onBlockClick?.(block)}
              role="button"
              tabindex="0"
              title={block.label || ""}
            >
              <span class="block-time">
                {formatTimeShort(block.start_time)} - {formatTimeShort(block.end_time)}
              </span>
              {#if block.label}
                <span class="block-label">{block.label}</span>
              {/if}
              <button
                class="block-edit-btn"
                onclick={(e) => {
                  e.stopPropagation();
                  onBlockEdit?.(block);
                }}
                title="Edit block"
              >
                <Pencil size={12} />
              </button>
            </div>
          {/each}
        </div>

        <!-- Notes dots at the bottom -->
        {#if getNotesForDate(date).length > 0}
          <div class="notes-indicator">
            {#each getNotesForDate(date).slice(0, 3) as note (note.id)}
              <button
                class="note-dot"
                onclick={() => onNoteClick?.(note, date)}
                title={note.title || note.path}
              ></button>
            {/each}
            {#if getNotesForDate(date).length > 3}
              <span class="more-notes">+{getNotesForDate(date).length - 3}</span>
            {/if}
          </div>
        {/if}
      </div>
    {/each}
  </div>
</div>

<style>
  .calendar-weekly {
    display: flex;
    flex-direction: column;
    height: 100%;
    min-height: 400px;
    background: var(--calendar-bg);
  }

  .week-nav {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: var(--spacing-4);
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

  .week-range {
    font-size: var(--font-size-lg);
    font-weight: var(--font-weight-semibold);
    color: var(--text-primary);
    min-width: 200px;
    text-align: center;
  }

  .week-header {
    display: flex;
    border-bottom: 1px solid var(--calendar-border);
    flex-shrink: 0;
  }

  .time-gutter {
    width: 60px;
    flex-shrink: 0;
  }

  .day-header {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    padding: var(--spacing-2) var(--spacing-1);
    border: none;
    background: transparent;
    cursor: pointer;
    border-left: 1px solid var(--calendar-border);
  }

  .day-header:hover {
    background: var(--bg-hover);
  }

  .day-header.is-today .day-number {
    background: var(--calendar-selected-bg);
    color: var(--calendar-selected-text);
    border-radius: var(--radius-full);
    width: 28px;
    height: 28px;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .day-header.is-selected {
    background: var(--bg-active);
  }

  .day-name {
    font-size: var(--font-size-xs);
    color: var(--text-muted);
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }

  .day-number {
    font-size: var(--font-size-xl);
    font-weight: var(--font-weight-medium);
    margin-top: 2px;
  }

  .week-grid {
    display: flex;
    flex: 1;
    overflow-y: auto;
  }

  .week-grid .time-gutter {
    display: flex;
    flex-direction: column;
  }

  .time-label {
    height: 48px;
    display: flex;
    align-items: flex-start;
    justify-content: flex-end;
    padding-right: var(--spacing-2);
    font-size: var(--font-size-xs);
    color: var(--text-muted);
    transform: translateY(-6px);
  }

  .day-column {
    flex: 1;
    position: relative;
    border-left: 1px solid var(--calendar-border);
    display: flex;
    flex-direction: column;
  }

  .day-column.is-weekend {
    background: var(--calendar-weekend-bg);
  }

  .day-column.is-today {
    background: var(--calendar-today-bg);
  }

  .hour-slot {
    height: 48px;
    border: none;
    background: transparent;
    border-bottom: 1px solid var(--border-light);
    cursor: pointer;
  }

  .hour-slot:hover {
    background: var(--bg-hover);
  }

  .blocks-container {
    position: absolute;
    top: 0;
    left: var(--spacing-1);
    right: var(--spacing-1);
    bottom: 0;
    pointer-events: none;
  }

  .schedule-block {
    position: absolute;
    /* left and width set dynamically via style for overlapping blocks */
    border: none;
    border-radius: var(--radius-sm);
    padding: var(--spacing-1) var(--spacing-2);
    color: var(--block-default-text);
    font-size: var(--font-size-xs);
    cursor: pointer;
    overflow: hidden;
    text-align: left;
    pointer-events: auto;
    opacity: 0.9;
    box-sizing: border-box;
  }

  .schedule-block:hover {
    opacity: 1;
    box-shadow: var(--shadow-md);
  }

  .schedule-block:hover .block-edit-btn {
    opacity: 1;
  }

  .block-edit-btn {
    position: absolute;
    top: 2px;
    right: 2px;
    width: 20px;
    height: 20px;
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

  .block-time {
    display: block;
    font-weight: var(--font-weight-medium);
    white-space: nowrap;
  }

  .block-label {
    display: block;
    margin-top: 2px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .notes-indicator {
    position: absolute;
    bottom: var(--spacing-1);
    left: var(--spacing-1);
    right: var(--spacing-1);
    display: flex;
    gap: var(--spacing-1);
    align-items: center;
  }

  .note-dot {
    width: 8px;
    height: 8px;
    border-radius: var(--radius-full);
    background: var(--text-muted);
    border: none;
    cursor: pointer;
    padding: 0;
  }

  .note-dot:hover {
    background: var(--color-primary);
    transform: scale(1.2);
  }

  .more-notes {
    font-size: var(--font-size-xs);
    color: var(--text-muted);
  }
</style>
