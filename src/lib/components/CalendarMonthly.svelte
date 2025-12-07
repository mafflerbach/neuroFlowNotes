<script lang="ts">
  import { ChevronLeft, ChevronRight } from "lucide-svelte";
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
    workspaceStore.setCalendarView("daily");
    onDayClick?.(date);
  }
</script>

<div class="calendar-monthly">
  <!-- Month header with navigation -->
  <div class="month-header">
    <button class="nav-arrow" onclick={goToPreviousMonth} aria-label="Previous month">
      <ChevronLeft size={20} />
    </button>

    <h2 class="month-title">
      {MONTH_NAMES_FULL[selectedDate.getMonth()]} {selectedDate.getFullYear()}
    </h2>

    <button class="nav-arrow" onclick={goToNextMonth} aria-label="Next month">
      <ChevronRight size={20} />
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
                  {#each notes.slice(0, 3)}
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
    background: var(--calendar-bg);
  }

  .month-header {
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

  .month-title {
    font-size: var(--font-size-xl);
    font-weight: var(--font-weight-semibold);
    color: var(--text-primary);
    margin: 0;
  }

  .calendar-grid {
    padding: var(--spacing-3);
  }

  .day-names {
    display: grid;
    grid-template-columns: repeat(7, 1fr);
    gap: var(--spacing-1);
    margin-bottom: var(--spacing-2);
  }

  .day-name {
    text-align: center;
    font-size: var(--font-size-xs);
    font-weight: var(--font-weight-medium);
    color: var(--text-muted);
    text-transform: uppercase;
    letter-spacing: 0.5px;
    padding: var(--spacing-1);
  }

  .weeks {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-1);
  }

  .week-row {
    display: grid;
    grid-template-columns: repeat(7, 1fr);
    gap: var(--spacing-1);
  }

  .day-cell {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: flex-start;
    padding: var(--spacing-2) var(--spacing-1);
    min-height: 48px;
    border: none;
    background: transparent;
    border-radius: var(--radius-lg);
    cursor: pointer;
    gap: var(--spacing-1);
  }

  .day-cell:hover {
    background: var(--bg-hover);
  }

  .day-cell.other-month {
    opacity: 0.4;
  }

  .day-cell.is-today .day-number {
    background: var(--calendar-selected-bg);
    color: var(--calendar-selected-text);
    border-radius: var(--radius-full);
    width: 28px;
    height: 28px;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .day-cell.is-selected {
    background: var(--bg-active);
  }

  .day-number {
    font-size: var(--font-size-md);
    font-weight: var(--font-weight-medium);
    color: var(--text-primary);
  }

  .note-dots {
    display: flex;
    gap: 3px;
    align-items: center;
  }

  .dot {
    width: 6px;
    height: 6px;
    border-radius: var(--radius-full);
    background: var(--text-muted);
  }

  .day-cell.is-today .dot {
    background: var(--color-primary);
  }

  .more {
    font-size: 9px;
    color: var(--text-muted);
  }

  .month-notes {
    flex: 1;
    overflow-y: auto;
    border-top: 1px solid var(--calendar-border);
    padding: var(--spacing-3);
  }

  .notes-header {
    font-size: var(--font-size-base);
    font-weight: var(--font-weight-semibold);
    color: var(--text-muted);
    margin: 0 0 var(--spacing-3) 0;
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }

  .empty-state {
    font-size: var(--font-size-base);
    color: var(--text-muted);
    text-align: center;
    padding: var(--spacing-6);
  }

  .notes-list {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-1);
  }

  .note-item {
    display: flex;
    align-items: center;
    gap: var(--spacing-3);
    padding: var(--spacing-2) var(--spacing-3);
    border: none;
    background: transparent;
    border-radius: var(--radius-md);
    cursor: pointer;
    text-align: left;
    width: 100%;
  }

  .note-item:hover {
    background: var(--bg-hover);
  }

  .note-date {
    font-size: var(--font-size-sm);
    color: var(--text-muted);
    font-family: var(--font-family-mono);
    flex-shrink: 0;
  }

  .note-title {
    font-size: var(--font-size-md);
    color: var(--text-primary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
</style>
