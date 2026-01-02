<script lang="ts">
  import { Copy, Check } from "lucide-svelte";
  import { convertFileSrc } from "@tauri-apps/api/core";
  import { vaultStore } from "../../stores";
  import type { QueryResultItem, PropertyFilter, QueryViewType } from "../../types";

  interface Props {
    results: QueryResultItem[];
    totalCount: number;
    loading: boolean;
    filters: PropertyFilter[];
    viewType: QueryViewType;
    cardCoverProperty: string | null;
    cardDisplayFields: string[];
    onOpenNote: (noteId: number, path: string, title: string | null) => void;
  }

  let {
    results,
    totalCount,
    loading,
    filters,
    viewType,
    cardCoverProperty,
    cardDisplayFields,
    onOpenNote,
  }: Props = $props();

  // Track which item was just copied (for visual feedback)
  let copiedPath = $state<string | null>(null);

  // Get priority class for styling
  function priorityClass(priority: string | null): string {
    if (!priority) return "";
    return `priority-${priority}`;
  }

  // Check if any filters have a key set
  const hasFilters = $derived(filters.some((f) => f.key));

  // Get the value of a field from a result item
  function getFieldValue(item: QueryResultItem, field: string): string | null {
    // Check task fields first
    if (item.item_type === "task" && item.task) {
      const todo = item.task.todo;
      switch (field) {
        case "description":
          return todo.description;
        case "priority":
          return todo.priority;
        case "context":
          return todo.context;
        case "due_date":
          return todo.due_date;
        case "heading_path":
          return todo.heading_path;
      }
    }

    // Check note title
    if (item.item_type === "note" && item.note) {
      if (field === "title") {
        return item.note.title;
      }
      if (field === "description") {
        return item.note.title;
      }
    }

    // Check properties
    const prop = item.properties.find((p) => p.key === field);
    return prop?.value ?? null;
  }

  // Get cover image URL from a result item
  function getCoverImage(item: QueryResultItem): string | null {
    if (!cardCoverProperty) return null;
    const value = getFieldValue(item, cardCoverProperty);
    if (!value) return null;

    // If it's already an absolute URL (http/https), return as-is
    if (value.startsWith("http://") || value.startsWith("https://")) {
      return value;
    }

    // Resolve relative path against vault root
    const vaultPath = vaultStore.info?.path;
    if (!vaultPath) return null;

    // Handle paths that might start with ./ or just be relative
    const cleanPath = value.startsWith("./") ? value.slice(2) : value;
    const fullPath = `${vaultPath}/${cleanPath}`;

    // Convert to Tauri asset URL
    return convertFileSrc(fullPath);
  }

  // Get item title for card display
  function getItemTitle(item: QueryResultItem): string {
    if (item.item_type === "task" && item.task) {
      return item.task.todo.description;
    }
    if (item.item_type === "note" && item.note) {
      return item.note.title || item.note.path.replace(".md", "");
    }
    return "Untitled";
  }

  // Get item path for card click
  function getItemPath(item: QueryResultItem): string {
    if (item.item_type === "task" && item.task) {
      return item.task.note_path;
    }
    if (item.item_type === "note" && item.note) {
      return item.note.path;
    }
    return "";
  }

  // Get note ID for navigation
  function getItemNoteId(item: QueryResultItem): number {
    if (item.item_type === "task" && item.task) {
      return item.task.todo.note_id;
    }
    if (item.item_type === "note" && item.note) {
      return item.note.id;
    }
    return 0;
  }

  // Convert path to wiki link format (remove .md extension)
  function toWikiLink(path: string): string {
    return `[[${path.replace(/\.md$/, "")}]]`;
  }

  // Copy wiki link to clipboard
  async function copyWikiLink(path: string) {
    const wikiLink = toWikiLink(path);
    try {
      await navigator.clipboard.writeText(wikiLink);
      copiedPath = path;
      // Reset after 2 seconds
      setTimeout(() => {
        if (copiedPath === path) {
          copiedPath = null;
        }
      }, 2000);
    } catch (e) {
      console.error("Failed to copy to clipboard:", e);
    }
  }
</script>

<div class="results-section">
  <div class="results-header">
    <span class="section-label">Results</span>
    {#if totalCount > 0}
      <span class="result-count">{totalCount} items</span>
    {/if}
  </div>

  {#if viewType === "Card"}
    <div class="results-cards">
      {#if loading}
        <div class="loading">Running query...</div>
      {:else if results.length === 0}
        <div class="empty-state">
          {hasFilters
            ? "No results match your query"
            : "Add filters and run query to see results"}
        </div>
      {:else}
        <div class="card-grid">
          {#each results as item (`${item.item_type}-${item.item_type === "task" ? item.task?.todo.id : item.note?.id}`)}
            {@const coverImage = getCoverImage(item)}
            <button
              class="result-card"
              class:has-cover={coverImage}
              onclick={() => onOpenNote(getItemNoteId(item), getItemPath(item), getItemTitle(item))}
            >
              {#if coverImage}
                <div class="card-cover">
                  <img src={coverImage} alt="" loading="lazy" />
                </div>
              {/if}
              <div class="card-content">
                <div class="card-title">{getItemTitle(item)}</div>
                {#if cardDisplayFields.length > 0}
                  <div class="card-fields">
                    {#each cardDisplayFields as field}
                      {@const value = getFieldValue(item, field)}
                      {#if value && field !== "description"}
                        <div class="card-field">
                          <span class="field-label">{field}:</span>
                          <span class="field-value" class:priority-high={field === "priority" && value === "high"} class:priority-medium={field === "priority" && value === "medium"}>{value}</span>
                        </div>
                      {/if}
                    {/each}
                  </div>
                {/if}
                <div class="card-type-badge" class:task={item.item_type === "task"} class:note={item.item_type === "note"}>
                  {item.item_type === "task" ? "Task" : "Note"}
                </div>
              </div>
            </button>
          {/each}
        </div>
      {/if}
    </div>
  {:else}
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
                <div class="note-link-row">
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
                  <button
                    class="copy-btn"
                    class:copied={copiedPath === item.task.note_path}
                    onclick={() => copyWikiLink(item.task!.note_path)}
                    title="Copy wiki link"
                  >
                    {#if copiedPath === item.task.note_path}
                      <Check size={12} />
                    {:else}
                      <Copy size={12} />
                    {/if}
                  </button>
                </div>
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
                <button
                  class="copy-btn"
                  class:copied={copiedPath === item.note.path}
                  onclick={() => copyWikiLink(item.note!.path)}
                  title="Copy wiki link"
                >
                  {#if copiedPath === item.note.path}
                    <Check size={14} />
                  {:else}
                    <Copy size={14} />
                  {/if}
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
  {/if}
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

  .note-link-row {
    display: flex;
    align-items: center;
    gap: var(--spacing-1);
  }

  .copy-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    padding: var(--spacing-1);
    border: none;
    background: transparent;
    border-radius: var(--radius-sm);
    color: var(--text-muted);
    cursor: pointer;
    opacity: 0;
    transition: opacity var(--transition-normal), color var(--transition-normal), background var(--transition-normal);
  }

  .result-item:hover .copy-btn {
    opacity: 1;
  }

  .copy-btn:hover {
    background: var(--bg-hover);
    color: var(--text-primary);
  }

  .copy-btn.copied {
    opacity: 1;
    color: var(--color-success);
  }

  /* Card View Styles */
  .results-cards {
    flex: 1;
    overflow-y: auto;
    overflow-x: hidden;
    padding: var(--spacing-3);
  }

  .card-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(220px, 1fr));
    gap: var(--spacing-3);
  }

  .result-card {
    display: flex;
    flex-direction: column;
    background: var(--bg-surface);
    border: 1px solid var(--border-default);
    border-radius: var(--radius-md);
    overflow: hidden;
    cursor: pointer;
    transition: transform var(--transition-normal), box-shadow var(--transition-normal), border-color var(--transition-normal);
    text-align: left;
    padding: 0;
  }

  .result-card:hover {
    transform: translateY(-2px);
    box-shadow: var(--shadow-lg);
    border-color: var(--color-primary);
  }

  .card-cover {
    width: 100%;
    height: 120px;
    overflow: hidden;
    background: var(--bg-surface-sunken);
  }

  .card-cover img {
    width: 100%;
    height: 100%;
    object-fit: cover;
  }

  .card-content {
    padding: var(--spacing-3);
    display: flex;
    flex-direction: column;
    gap: var(--spacing-2);
    flex: 1;
    position: relative;
  }

  .card-title {
    font-size: var(--font-size-sm);
    font-weight: var(--font-weight-medium);
    color: var(--text-primary);
    line-height: 1.4;
    display: -webkit-box;
    -webkit-line-clamp: 2;
    -webkit-box-orient: vertical;
    overflow: hidden;
  }

  .card-fields {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-1);
  }

  .card-field {
    display: flex;
    gap: var(--spacing-1);
    font-size: var(--font-size-xs);
  }

  .field-label {
    color: var(--text-muted);
    text-transform: capitalize;
  }

  .field-value {
    color: var(--text-secondary);
  }

  .field-value.priority-high {
    color: var(--color-error);
    font-weight: var(--font-weight-medium);
  }

  .field-value.priority-medium {
    color: var(--color-warning);
    font-weight: var(--font-weight-medium);
  }

  .card-type-badge {
    position: absolute;
    top: var(--spacing-2);
    right: var(--spacing-2);
    font-size: 10px;
    padding: 2px 6px;
    border-radius: var(--radius-sm);
    font-weight: var(--font-weight-medium);
  }

  .card-type-badge.task {
    background: var(--color-primary);
    color: var(--color-white);
  }

  .card-type-badge.note {
    background: var(--color-info);
    color: var(--color-white);
  }
</style>
