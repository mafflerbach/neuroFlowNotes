<script lang="ts">
  import { workspaceStore } from "../stores/workspace.svelte";

  interface Props {
    onReturnToCalendar?: () => void;
  }

  let { onReturnToCalendar }: Props = $props();

  const breadcrumb = $derived(workspaceStore.breadcrumb);
  const visibleDocs = $derived(workspaceStore.visibleDocs);

  function handleCrumbClick(index: number) {
    workspaceStore.navigateBreadcrumb(index);
  }

  function handleReturnToCalendar() {
    workspaceStore.returnToCalendar();
    onReturnToCalendar?.();
  }

  function getDisplayTitle(doc: { path: string; title: string | null }): string {
    return doc.title || doc.path.replace(/\.md$/, "").split("/").pop() || doc.path;
  }
</script>

<nav class="breadcrumb" aria-label="Document navigation">
  <!-- Return to calendar button -->
  <button
    class="return-btn"
    onclick={handleReturnToCalendar}
    title="Return to calendar"
  >
    <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
      <rect x="3" y="4" width="18" height="18" rx="2" ry="2" />
      <line x1="16" y1="2" x2="16" y2="6" />
      <line x1="8" y1="2" x2="8" y2="6" />
      <line x1="3" y1="10" x2="21" y2="10" />
    </svg>
  </button>

  <span class="separator">/</span>

  <!-- Collapsed indicator if there are more than 3 docs -->
  {#if breadcrumb.length > 3}
    <button
      class="collapsed-indicator"
      onclick={() => handleCrumbClick(breadcrumb.length - 4)}
      title="Show earlier documents"
    >
      <span class="ellipsis">...</span>
      <span class="count">{breadcrumb.length - 3}</span>
    </button>
    <span class="separator">/</span>
  {/if}

  <!-- Visible breadcrumb items (last 3) -->
  {#each visibleDocs as doc, i}
    {@const globalIndex = breadcrumb.length - visibleDocs.length + i}
    {@const isLast = i === visibleDocs.length - 1}

    <button
      class="crumb"
      class:active={isLast}
      onclick={() => handleCrumbClick(globalIndex)}
      title={doc.path}
    >
      {getDisplayTitle(doc)}
    </button>

    {#if !isLast}
      <span class="separator">/</span>
    {/if}
  {/each}

  <!-- Close all button -->
  {#if breadcrumb.length > 0}
    <button
      class="close-all-btn"
      onclick={() => workspaceStore.closeAllDocs()}
      title="Close all documents"
    >
      <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M18 6L6 18M6 6l12 12" />
      </svg>
    </button>
  {/if}
</nav>

<style>
  .breadcrumb {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 8px 12px;
    background: var(--breadcrumb-bg, #f8f9fa);
    border-bottom: 1px solid var(--border-color, #e0e0e0);
    overflow-x: auto;
    scrollbar-width: none;
  }

  .breadcrumb::-webkit-scrollbar {
    display: none;
  }

  .return-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 28px;
    height: 28px;
    border: none;
    background: transparent;
    border-radius: 6px;
    color: var(--text-muted, #666);
    cursor: pointer;
    flex-shrink: 0;
  }

  .return-btn:hover {
    background: var(--hover-bg, #e8e8e8);
    color: var(--primary-color, #4f6bed);
  }

  .separator {
    color: var(--text-muted, #999);
    font-size: 12px;
    flex-shrink: 0;
  }

  .collapsed-indicator {
    display: flex;
    align-items: center;
    gap: 4px;
    padding: 4px 8px;
    border: none;
    background: var(--collapsed-bg, #e8e8e8);
    border-radius: 4px;
    font-size: 12px;
    color: var(--text-muted, #666);
    cursor: pointer;
    flex-shrink: 0;
  }

  .collapsed-indicator:hover {
    background: var(--hover-bg, #ddd);
  }

  .ellipsis {
    font-weight: 600;
  }

  .count {
    font-size: 10px;
    background: var(--count-bg, #ccc);
    color: var(--count-color, #fff);
    padding: 1px 4px;
    border-radius: 8px;
    min-width: 16px;
    text-align: center;
  }

  .crumb {
    padding: 4px 10px;
    border: none;
    background: transparent;
    border-radius: 4px;
    font-size: 13px;
    color: var(--text-muted, #666);
    cursor: pointer;
    white-space: nowrap;
    max-width: 150px;
    overflow: hidden;
    text-overflow: ellipsis;
    flex-shrink: 0;
  }

  .crumb:hover {
    background: var(--hover-bg, #e8e8e8);
    color: var(--text-color, #333);
  }

  .crumb.active {
    background: var(--active-crumb-bg, #e0e7ff);
    color: var(--primary-color, #4f6bed);
    font-weight: 500;
  }

  .close-all-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 24px;
    height: 24px;
    border: none;
    background: transparent;
    border-radius: 4px;
    color: var(--text-muted, #999);
    cursor: pointer;
    margin-left: auto;
    flex-shrink: 0;
  }

  .close-all-btn:hover {
    background: var(--error-bg, #fee);
    color: var(--error-color, #d32f2f);
  }
</style>
