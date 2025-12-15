<script lang="ts">
  import type { QueryResultItem, PropertyFilter } from "../../types";

  interface Props {
    results: QueryResultItem[];
    totalCount: number;
    loading: boolean;
    filters: PropertyFilter[];
    onOpenNote: (noteId: number, path: string, title: string | null) => void;
  }

  let { results, totalCount, loading, filters, onOpenNote }: Props = $props();

  // Get priority class for styling
  function priorityClass(priority: string | null): string {
    if (!priority) return "";
    return `priority-${priority}`;
  }

  // Check if any filters have a key set
  const hasFilters = $derived(filters.some((f) => f.key));
</script>

<div class="results-section">
  <div class="results-header">
    <span class="section-label">Results</span>
    {#if totalCount > 0}
      <span class="result-count">{totalCount} items</span>
    {/if}
  </div>

  <div class="results-list">
    {#if loading}
      <div class="loading">Running query...</div>
    {:else if results.length === 0}
      <div class="empty-state">
        {hasFilters
          ? "No results match your query"
          : "Add filters and run query to see results"}
      </div>
    {:else}
      {#each results as item (`${item.item_type}-${item.item_type === "task" ? item.task?.todo.id : item.note?.id}`)}
        {#if item.item_type === "task" && item.task}
          <div class="result-item task-item" class:completed={item.task.todo.completed}>
            <div class="item-type-badge task">Task</div>
            <div class="item-content">
              <div class="item-main">
                <span class="item-text">{item.task.todo.description}</span>
                {#if item.task.todo.priority}
                  <span class="badge {priorityClass(item.task.todo.priority)}">
                    {item.task.todo.priority}
                  </span>
                {/if}
                {#if item.task.todo.context}
                  <span class="badge context">@{item.task.todo.context}</span>
                {/if}
                {#if item.task.todo.due_date}
                  <span class="badge due-date">{item.task.todo.due_date}</span>
                {/if}
              </div>
              <div class="item-meta">
                <button
                  class="note-link"
                  onclick={() =>
                    onOpenNote(
                      item.task!.todo.note_id,
                      item.task!.note_path,
                      item.task!.note_title
                    )}
                >
                  {item.task.note_title || item.task.note_path.replace(".md", "")}
                </button>
                {#if item.properties.length > 0}
                  <span class="properties-preview">
                    {item.properties
                      .slice(0, 3)
                      .map((p) => `${p.key}: ${p.value}`)
                      .join(", ")}
                  </span>
                {/if}
              </div>
            </div>
          </div>
        {:else if item.item_type === "note" && item.note}
          <div class="result-item note-item">
            <div class="item-type-badge note">Note</div>
            <div class="item-content">
              <div class="item-main">
                <button
                  class="note-link title"
                  onclick={() => onOpenNote(item.note!.id, item.note!.path, item.note!.title)}
                >
                  {item.note.title || item.note.path.replace(".md", "")}
                </button>
              </div>
              {#if item.properties.length > 0}
                <div class="item-meta">
                  <span class="properties-preview">
                    {item.properties
                      .slice(0, 5)
                      .map((p) => `${p.key}: ${p.value}`)
                      .join(", ")}
                  </span>
                </div>
              {/if}
            </div>
          </div>
        {/if}
      {/each}
    {/if}
  </div>
</div>

<style>
  .results-section {
    flex: 1;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .results-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: var(--spacing-3) var(--spacing-4);
    border-bottom: 1px solid var(--panel-border);
  }

  .section-label {
    font-size: var(--font-size-sm);
    font-weight: var(--font-weight-semibold);
    color: var(--text-primary);
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }

  .result-count {
    font-size: var(--font-size-sm);
    color: var(--text-muted);
  }

  .results-list {
    flex: 1;
    overflow-y: auto;
    overflow-x: hidden;
    padding: var(--spacing-2);
  }

  .loading,
  .empty-state {
    padding: var(--spacing-8);
    text-align: center;
    color: var(--text-muted);
  }

  .result-item {
    display: flex;
    gap: var(--spacing-2);
    padding: var(--spacing-2) var(--spacing-3);
    border-radius: var(--radius-sm);
    margin-bottom: var(--spacing-2);
    border-bottom: 1px solid var(--panel-border);
  }

  .result-item:hover {
    background: var(--bg-hover);
  }

  .result-item.completed .item-text {
    text-decoration: line-through;
    color: var(--text-muted);
  }

  .item-type-badge {
    font-size: var(--font-size-xs);
    padding: 2px 6px;
    border-radius: var(--radius-sm);
    font-weight: var(--font-weight-medium);
    flex-shrink: 0;
  }

  .item-type-badge.task {
    background: var(--color-primary);
    color: var(--color-white);
  }

  .item-type-badge.note {
    background: var(--color-info);
    color: var(--color-white);
  }

  .item-content {
    flex: 1;
    min-width: 0;
  }

  .item-main {
    display: flex;
    align-items: flex-start;
    gap: var(--spacing-2);
    flex-wrap: wrap;
  }

  .item-text {
    font-size: var(--font-size-sm);
    color: var(--text-secondary);
    word-break: break-word;
  }

  .badge {
    font-size: var(--font-size-xs);
    padding: 1px 6px;
    border-radius: var(--radius-sm);
    background: var(--bg-surface-raised);
    color: var(--text-muted);
  }

  .badge.priority-high {
    background: var(--color-error);
    color: var(--color-white);
  }

  .badge.priority-medium {
    background: var(--color-warning);
    color: var(--color-black);
  }

  .badge.priority-low {
    background: var(--bg-surface-raised);
    color: var(--text-muted);
  }

  .badge.context {
    background: var(--color-info);
    color: var(--color-white);
  }

  .badge.due-date {
    background: var(--color-warning);
    color: var(--color-black);
  }

  .item-meta {
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    gap: var(--spacing-1);
    font-size: var(--font-size-sm);
    color: var(--text-muted);
    margin-top: var(--spacing-1);
  }

  .note-link {
    background: none;
    border: none;
    color: var(--text-link);
    cursor: pointer;
    padding: 0;
    font-size: inherit;
    text-decoration: none;
  }

  .note-link:hover {
    text-decoration: underline;
  }

  .note-link.title {
    font-size: var(--font-size-base);
    font-weight: var(--font-weight-medium);
  }

  .properties-preview {
    color: var(--text-secondary);
    font-size: var(--font-size-xs);
    word-break: break-word;
    line-height: 1.4;
  }
</style>
