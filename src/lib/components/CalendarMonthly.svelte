<script lang="ts">
  import { workspaceStore } from "../stores/workspace.svelte";
  import type { NoteListItem } from "../types";
  import {
    formatDateKey,
    isToday,
    isSameDay,
    isSameMonth,
    getMonthCalendarGrid,
    getPreviousMonth,
    getNextMonth,
    DAY_NAMES_SHORT,
    MONTH_NAMES_FULL,
  } from "../utils/dateUtils";
  import { getMonthNotesForDisplay } from "../utils/docListUtils";

  interface Props {
    notesForMonth?: Map<string, NoteListItem[]>;
    onNoteClick?: (note: NoteListItem) => void;
    onDayClick?: (date: Date) => void;
  }

  let {
    notesForMonth = new Map(),
    onNoteClick,
    onDayClick,
  }: Props = $props();

  const selectedDate = $derived(workspaceStore.selectedDate);
  const calendarGrid = $derived(() => getMonthCalendarGrid(selectedDate));
  const allNotesForMonth = $derived(() => getMonthNotesForDisplay(notesForMonth));

  function isSelected(date: Date): boolean {
    return isSameDay(date, selectedDate);
  }

  function isCurrentMonth(date: Date): boolean {
    return isSameMonth(date, selectedDate);
  }

  function getNotesForDate(date: Date): NoteListItem[] {
    const dateKey = formatDateKey(date);
    return notesForMonth.get(dateKey) || [];
  }

  function goToPreviousMonth() {
    workspaceStore.selectDate(getPreviousMonth(selectedDate));
  }

  function goToNextMonth() {
    workspaceStore.selectDate(getNextMonth(selectedDate));
  }

  function handleDayClick(date: Date) {
    workspaceStore.selectDate(date);
    onDayClick?.(date);
  }
</script>

<div class="calendar-monthly">
  <!-- Month header with navigation -->
  <div class="month-header">
    <button class="nav-arrow" onclick={goToPreviousMonth}>
      <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M15 18l-6-6 6-6" />
      </svg>
    </button>

    <h2 class="month-title">
      {MONTH_NAMES_FULL[selectedDate.getMonth()]} {selectedDate.getFullYear()}
    </h2>

    <button class="nav-arrow" onclick={goToNextMonth}>
      <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M9 18l6-6-6-6" />
      </svg>
    </button>
  </div>

  <!-- Calendar grid -->
  <div class="calendar-grid">
    <!-- Day names header -->
    <div class="day-names">
      {#each DAY_NAMES_SHORT as name}
        <div class="day-name">{name}</div>
      {/each}
    </div>

    <!-- Weeks -->
    <div class="weeks">
      {#each calendarGrid() as week}
        <div class="week-row">
          {#each week as date}
            {@const notes = getNotesForDate(date)}
            <button
              class="day-cell"
              class:other-month={!isCurrentMonth(date)}
              class:is-today={isToday(date)}
              class:is-selected={isSelected(date)}
              class:has-notes={notes.length > 0}
              onclick={() => handleDayClick(date)}
            >
              <span class="day-number">{date.getDate()}</span>
              {#if notes.length > 0}
                <div class="note-dots">
                  {#each notes.slice(0, 3) as _ , i}
                    <span class="dot"></span>
                  {/each}
                  {#if notes.length > 3}
                    <span class="more">+{notes.length - 3}</span>
                  {/if}
                </div>
              {/if}
            </button>
          {/each}
        </div>
      {/each}
    </div>
  </div>

  <!-- Notes list for the month -->
  <div class="month-notes">
    <h3 class="notes-header">Notes this month</h3>
    {#if allNotesForMonth().length === 0}
      <div class="empty-state">No notes for this month</div>
    {:else}
      <div class="notes-list">
        {#each allNotesForMonth() as { date, note } (note.id)}
          <button
            class="note-item"
            onclick={() => onNoteClick?.(note)}
          >
            <span class="note-date">{date.slice(5)}</span>
            <span class="note-title">{note.title || note.path}</span>
          </button>
        {/each}
      </div>
    {/if}
  </div>
</div>

<style>
  .calendar-monthly {
    display: flex;
    flex-direction: column;
    height: 100%;
    background: var(--calendar-bg, #fff);
  }

  .month-header {
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

  .month-title {
    font-size: 18px;
    font-weight: 600;
    color: var(--text-color, #333);
    margin: 0;
  }

  .calendar-grid {
    padding: 12px;
  }

  .day-names {
    display: grid;
    grid-template-columns: repeat(7, 1fr);
    gap: 4px;
    margin-bottom: 8px;
  }

  .day-name {
    text-align: center;
    font-size: 11px;
    font-weight: 500;
    color: var(--text-muted, #666);
    text-transform: uppercase;
    letter-spacing: 0.5px;
    padding: 4px;
  }

  .weeks {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .week-row {
    display: grid;
    grid-template-columns: repeat(7, 1fr);
    gap: 4px;
  }

  .day-cell {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: flex-start;
    padding: 8px 4px;
    min-height: 48px;
    border: none;
    background: transparent;
    border-radius: 8px;
    cursor: pointer;
    gap: 4px;
  }

  .day-cell:hover {
    background: var(--hover-bg, #f5f5f5);
  }

  .day-cell.other-month {
    opacity: 0.4;
  }

  .day-cell.is-today .day-number {
    background: var(--primary-color, #4f6bed);
    color: white;
    border-radius: 50%;
    width: 28px;
    height: 28px;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .day-cell.is-selected {
    background: var(--active-bg, #e0e7ff);
  }

  .day-number {
    font-size: 14px;
    font-weight: 500;
    color: var(--text-color, #333);
  }

  .note-dots {
    display: flex;
    gap: 3px;
    align-items: center;
  }

  .dot {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    background: var(--note-dot-color, #888);
  }

  .day-cell.is-today .dot {
    background: var(--primary-color, #4f6bed);
  }

  .more {
    font-size: 9px;
    color: var(--text-muted, #666);
  }

  .month-notes {
    flex: 1;
    overflow-y: auto;
    border-top: 1px solid var(--border-color, #e0e0e0);
    padding: 12px;
  }

  .notes-header {
    font-size: 13px;
    font-weight: 600;
    color: var(--text-muted, #666);
    margin: 0 0 12px 0;
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }

  .empty-state {
    font-size: 13px;
    color: var(--text-muted, #999);
    text-align: center;
    padding: 24px;
  }

  .notes-list {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .note-item {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 8px 12px;
    border: none;
    background: transparent;
    border-radius: 6px;
    cursor: pointer;
    text-align: left;
    width: 100%;
  }

  .note-item:hover {
    background: var(--hover-bg, #f5f5f5);
  }

  .note-date {
    font-size: 12px;
    color: var(--text-muted, #666);
    font-family: monospace;
    flex-shrink: 0;
  }

  .note-title {
    font-size: 14px;
    color: var(--text-color, #333);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
</style>
