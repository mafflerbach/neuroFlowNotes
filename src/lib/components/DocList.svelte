<script lang="ts">
  import { workspaceStore } from "../stores/workspace.svelte";
  import type { NoteListItem, ScheduleBlockDto } from "../types";
  import {
    formatDateDisplay,
    formatWeekRange,
    formatMonthYear,
  } from "../utils/dateUtils";
  import { formatTimeShort } from "../utils/blockLayoutUtils";
  import type { DocWithSource } from "../utils/docListUtils";

  interface Props {
    scheduledDocs?: DocWithSource[];
    journalDocs?: DocWithSource[];
    createdDocs?: DocWithSource[];
    viewMode?: "daily" | "weekly" | "monthly";
    onDocClick?: (note: NoteListItem) => void;
  }

  let {
    scheduledDocs = [],
    journalDocs = [],
    createdDocs = [],
    viewMode = "daily",
    onDocClick,
  }: Props = $props();

  const selectedDate = $derived(workspaceStore.selectedDate);
  const calendarView = $derived(workspaceStore.calendarView);

  const headerTitle = $derived(() => {
    if (calendarView === "weekly") {
      return formatWeekRange(selectedDate);
    } else if (calendarView === "monthly") {
      return formatMonthYear(selectedDate);
    }
    return formatDateDisplay(selectedDate);
  });

  const emptyMessage = $derived(() => {
    if (calendarView === "weekly") {
      return "No notes for this week";
    } else if (calendarView === "monthly") {
      return "No notes for this month";
    }
    return "No notes for this day";
  });

  const hasDocs = $derived(
    scheduledDocs.length > 0 || journalDocs.length > 0 || createdDocs.length > 0
  );

  // Sort created docs alphabetically by filename
  const sortedCreatedDocs = $derived(
    [...createdDocs].sort((a, b) => {
      const nameA = a.note.path.split("/").pop()?.toLowerCase() || "";
      const nameB = b.note.path.split("/").pop()?.toLowerCase() || "";
      return nameA.localeCompare(nameB);
    })
  );
</script>

<div class="doc-list">
  <div class="list-header">
    <h3 class="date-title">{headerTitle()}</h3>
  </div>

  <div class="list-content">
    {#if !hasDocs}
      <div class="empty-state">
        <div class="empty-icon">
          <svg width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
            <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z" />
            <polyline points="14 2 14 8 20 8" />
          </svg>
        </div>
        <p>{emptyMessage()}</p>
      </div>
    {:else}
      <!-- Scheduled docs (highest priority) -->
      {#if scheduledDocs.length > 0}
        <div class="doc-section">
          <h4 class="section-title">
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <circle cx="12" cy="12" r="10" />
              <polyline points="12 6 12 12 16 14" />
            </svg>
            Scheduled
          </h4>
          <div class="doc-items">
            {#each scheduledDocs as doc (doc.note.id)}
              <button
                class="doc-item scheduled"
                onclick={() => onDocClick?.(doc.note)}
              >
                {#if doc.scheduleBlock}
                  <span
                    class="schedule-indicator"
                    style="background-color: {doc.scheduleBlock.color || '#4f6bed'}"
                  ></span>
                  <span class="schedule-time">
                    {formatTimeShort(doc.scheduleBlock.start_time)}
                  </span>
                {/if}
                <span class="doc-title">{doc.note.title || doc.note.path}</span>
              </button>
            {/each}
          </div>
        </div>
      {/if}

      <!-- Journal docs -->
      {#if journalDocs.length > 0}
        <div class="doc-section">
          <h4 class="section-title">
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M4 19.5A2.5 2.5 0 0 1 6.5 17H20" />
              <path d="M6.5 2H20v20H6.5A2.5 2.5 0 0 1 4 19.5v-15A2.5 2.5 0 0 1 6.5 2z" />
            </svg>
            Journal
          </h4>
          <div class="doc-items">
            {#each journalDocs as doc (doc.note.id)}
              <button
                class="doc-item journal"
                onclick={() => onDocClick?.(doc.note)}
              >
                <span class="doc-title">{doc.note.title || doc.note.path}</span>
              </button>
            {/each}
          </div>
        </div>
      {/if}

      <!-- Created docs -->
      {#if createdDocs.length > 0}
        <div class="doc-section">
          <h4 class="section-title">
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z" />
              <polyline points="14 2 14 8 20 8" />
              <line x1="12" y1="18" x2="12" y2="12" />
              <line x1="9" y1="15" x2="15" y2="15" />
            </svg>
            Created
          </h4>
          <div class="doc-items">
            {#each sortedCreatedDocs as doc (doc.note.id)}
              <button
                class="doc-item created"
                onclick={() => onDocClick?.(doc.note)}
              >
                <span class="doc-title">{doc.note.title || doc.note.path}</span>
              </button>
            {/each}
          </div>
        </div>
      {/if}
    {/if}
  </div>
</div>

<style>
  .doc-list {
    display: flex;
    flex-direction: column;
    height: 100%;
    background: var(--panel-bg);
    border-right: 1px solid var(--border-color);
  }

  .list-header {
    padding: 12px 16px;
    border-bottom: 1px solid var(--border-color);
  }

  .date-title {
    font-size: 14px;
    font-weight: 600;
    color: var(--text-color);
    margin: 0;
  }

  .list-content {
    flex: 1;
    overflow-y: auto;
    padding: 12px;
  }

  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: 48px 24px;
    text-align: center;
  }

  .empty-icon {
    color: var(--text-muted);
    margin-bottom: 12px;
  }

  .empty-state p {
    font-size: 13px;
    color: var(--text-muted);
    margin: 0;
  }

  .doc-section {
    margin-bottom: 20px;
  }

  .doc-section:last-child {
    margin-bottom: 0;
  }

  .section-title {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: 11px;
    font-weight: 600;
    color: var(--text-muted);
    text-transform: uppercase;
    letter-spacing: 0.5px;
    margin: 0 0 8px 0;
  }

  .doc-items {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .doc-item {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 10px 12px;
    border: none;
    background: var(--surface-color);
    border-radius: 8px;
    cursor: pointer;
    text-align: left;
    width: 100%;
    transition: background 0.15s;
  }

  .doc-item:hover {
    background: var(--hover-bg);
  }

  .schedule-indicator {
    width: 4px;
    height: 24px;
    border-radius: 2px;
    flex-shrink: 0;
  }

  .schedule-time {
    font-size: 12px;
    font-weight: 500;
    color: var(--text-muted);
    font-family: monospace;
    flex-shrink: 0;
  }

  .doc-title {
    font-size: 14px;
    color: var(--text-color);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .doc-item.scheduled {
    border-left: none;
  }

  .doc-item.journal {
    background: var(--surface-color);
  }

  .doc-item.journal:hover {
    background: var(--hover-bg);
  }
</style>
