<script lang="ts">
  import { Calendar, X } from "lucide-svelte";
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
    <Calendar size={16} />
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
      <X size={14} />
    </button>
  {/if}
</nav>

<style>
  .breadcrumb {
    display: flex;
    align-items: center;
    gap: var(--spacing-2);
    padding: var(--spacing-2) var(--spacing-3);
    background: var(--bg-surface-raised);
    border-bottom: 1px solid var(--border-default);
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
    border-radius: var(--radius-md);
    color: var(--text-muted);
    cursor: pointer;
    flex-shrink: 0;
  }

  .return-btn:hover {
    background: var(--bg-hover);
    color: var(--color-primary);
  }

  .separator {
    color: var(--text-muted);
    font-size: var(--font-size-sm);
    flex-shrink: 0;
  }

  .collapsed-indicator {
    display: flex;
    align-items: center;
    gap: var(--spacing-1);
    padding: var(--spacing-1) var(--spacing-2);
    border: none;
    background: var(--bg-hover);
    border-radius: var(--radius-sm);
    font-size: var(--font-size-sm);
    color: var(--text-muted);
    cursor: pointer;
    flex-shrink: 0;
  }

  .collapsed-indicator:hover {
    background: var(--border-default);
  }

  .ellipsis {
    font-weight: var(--font-weight-semibold);
  }

  .count {
    font-size: var(--font-size-xs);
    background: var(--text-muted);
    color: var(--text-inverse);
    padding: 1px var(--spacing-1);
    border-radius: var(--radius-lg);
    min-width: 16px;
    text-align: center;
  }

  .crumb {
    padding: var(--spacing-1) var(--spacing-3);
    border: none;
    background: transparent;
    border-radius: var(--radius-sm);
    font-size: var(--font-size-base);
    color: var(--text-muted);
    cursor: pointer;
    white-space: nowrap;
    max-width: 150px;
    overflow: hidden;
    text-overflow: ellipsis;
    flex-shrink: 0;
  }

  .crumb:hover {
    background: var(--bg-hover);
    color: var(--text-primary);
  }

  .crumb.active {
    background: var(--bg-active);
    color: var(--color-primary);
    font-weight: var(--font-weight-medium);
  }

  .close-all-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 24px;
    height: 24px;
    border: none;
    background: transparent;
    border-radius: var(--radius-sm);
    color: var(--text-muted);
    cursor: pointer;
    margin-left: auto;
    flex-shrink: 0;
  }

  .close-all-btn:hover {
    background: var(--color-error-light);
    color: var(--color-error);
  }
</style>
