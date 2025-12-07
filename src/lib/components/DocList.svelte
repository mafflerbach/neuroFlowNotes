<script lang="ts">
  import { Clock, FileText, FilePlus, BookOpen } from "lucide-svelte";
  import { workspaceStore } from "../stores/workspace.svelte";
  import type { NoteListItem } from "../types";
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
    onDocClick?: (note: NoteListItem) => void;
  }

  let {
    scheduledDocs = [],
    journalDocs = [],
    createdDocs = [],
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
          <FileText size={32} strokeWidth={1.5} />
        </div>
        <p>{emptyMessage()}</p>
      </div>
    {:else}
      <!-- Scheduled docs (highest priority) -->
      {#if scheduledDocs.length > 0}
        <div class="doc-section">
          <h4 class="section-title">
            <Clock size={14} />
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
            <BookOpen size={14} />
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
            <FilePlus size={14} />
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
    border-right: 1px solid var(--border-default);
  }

  .list-header {
    padding: var(--spacing-3) var(--spacing-4);
    border-bottom: 1px solid var(--border-default);
  }

  .date-title {
    font-size: var(--font-size-md);
    font-weight: var(--font-weight-semibold);
    color: var(--text-primary);
    margin: 0;
  }

  .list-content {
    flex: 1;
    overflow-y: auto;
    padding: var(--spacing-3);
  }

  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: var(--spacing-12) var(--spacing-6);
    text-align: center;
  }

  .empty-icon {
    color: var(--text-muted);
    margin-bottom: var(--spacing-3);
  }

  .empty-state p {
    font-size: var(--font-size-base);
    color: var(--text-muted);
    margin: 0;
  }

  .doc-section {
    margin-bottom: var(--spacing-5);
  }

  .doc-section:last-child {
    margin-bottom: 0;
  }

  .section-title {
    display: flex;
    align-items: center;
    gap: var(--spacing-1);
    font-size: var(--font-size-xs);
    font-weight: var(--font-weight-semibold);
    color: var(--text-secondary);
    text-transform: uppercase;
    letter-spacing: 0.5px;
    margin: 0 0 var(--spacing-2) 0;
  }

  .doc-items {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-1);
  }

  .doc-item {
    display: flex;
    align-items: center;
    gap: var(--spacing-2);
    padding: var(--spacing-2) var(--spacing-3);
    border: none;
    background: var(--bg-surface);
    border-radius: var(--radius-lg);
    cursor: pointer;
    text-align: left;
    width: 100%;
    transition: background var(--transition-normal);
  }

  .doc-item:hover {
    background: var(--bg-hover);
  }

  .schedule-indicator {
    width: 4px;
    height: 24px;
    border-radius: 2px;
    flex-shrink: 0;
  }

  .schedule-time {
    font-size: var(--font-size-sm);
    font-weight: var(--font-weight-medium);
    color: var(--text-secondary);
    font-family: var(--font-family-mono);
    flex-shrink: 0;
  }

  .doc-title {
    font-size: var(--font-size-md);
    color: var(--text-primary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .doc-item.scheduled {
    border-left: none;
  }

  .doc-item.journal {
    background: var(--bg-surface);
  }

  .doc-item.journal:hover {
    background: var(--bg-hover);
  }
</style>
