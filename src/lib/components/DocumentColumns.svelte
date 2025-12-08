<script lang="ts">
  import { X, Eye } from "lucide-svelte";
  import { workspaceStore } from "../stores/workspace.svelte";
  import ColumnEditor from "./ColumnEditor.svelte";
  import PropertiesPanel from "./PropertiesPanel.svelte";

  const visibleDocs = $derived(workspaceStore.visibleDocs);
  const activeDoc = $derived(workspaceStore.activeDoc);
  const multiColumnEditable = $derived(workspaceStore.multiColumnEditable);

  function isMarkdownFile(path: string): boolean {
    return path.toLowerCase().endsWith(".md");
  }

  function isActiveColumn(path: string): boolean {
    return activeDoc?.path === path;
  }

  function handleColumnClick(path: string) {
    // Find index in breadcrumb and navigate to it
    const index = workspaceStore.breadcrumb.findIndex((d) => d.path === path);
    if (index >= 0) {
      workspaceStore.navigateBreadcrumb(index);
    }
  }

  function handleCloseDoc(path: string) {
    workspaceStore.closeDoc(path);
  }
</script>

<div class="document-columns" class:single={visibleDocs.length === 1}>
  {#each visibleDocs as doc (doc.path)}
    {@const isActive = isActiveColumn(doc.path)}
    {@const isEditable = multiColumnEditable || isActive}

    <div
      class="column"
      class:active={isActive}
      class:readonly={!isEditable}
    >
      <!-- Column header -->
      <div class="column-header">
        <button
          class="column-title"
          onclick={() => handleColumnClick(doc.path)}
        >
          {doc.title || doc.path.replace(/\.md$/, "").split("/").pop()}
        </button>

        <button
          class="close-btn"
          onclick={() => handleCloseDoc(doc.path)}
          title="Close document"
        >
          <X size={14} />
        </button>
      </div>

      <!-- Editor area -->
      <div class="column-content">
        <div class="editor-wrapper">
          <ColumnEditor path={doc.path} readonly={!isEditable} />
        </div>

        <!-- Properties panel for all markdown files -->
        {#if isMarkdownFile(doc.path)}
          <PropertiesPanel noteId={doc.id} />
        {/if}
      </div>

      <!-- Read-only overlay indicator -->
      {#if !isEditable}
        <div class="readonly-indicator" title="Click to make active and edit">
          <Eye size={16} />
        </div>
      {/if}
    </div>
  {/each}
</div>

<style>
  .document-columns {
    display: flex;
    flex: 1;
    gap: 1px;
    background: var(--border-default);
    overflow: hidden;
  }

  .column {
    flex: 1;
    display: flex;
    flex-direction: column;
    background: var(--editor-bg);
    min-width: 300px;
    position: relative;
    transition: opacity var(--transition-normal);
  }

  .document-columns.single .column {
    min-width: 100%;
  }

  .column.readonly {
    opacity: 0.7;
  }

  .column.readonly:hover {
    opacity: 0.85;
  }

  .column.active {
    opacity: 1;
  }

  .column-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: var(--spacing-2) var(--spacing-3);
    background: var(--editor-header-bg);
    border-bottom: 1px solid var(--border-default);
  }

  .column.active .column-header {
    background: var(--bg-active);
  }

  .column-title {
    flex: 1;
    text-align: left;
    font-size: var(--font-size-base);
    font-weight: var(--font-weight-medium);
    color: var(--text-primary);
    background: transparent;
    border: none;
    cursor: pointer;
    padding: var(--spacing-1) 0;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .column.active .column-title {
    color: var(--color-primary);
  }

  .column-title:hover {
    text-decoration: underline;
  }

  .close-btn {
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
  }

  .close-btn:hover {
    background: var(--color-error-light);
    color: var(--color-error);
  }

  .column-content {
    flex: 1;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .editor-wrapper {
    flex: 1;
    overflow: auto;
  }

  .readonly-indicator {
    position: absolute;
    top: var(--spacing-2);
    right: 44px;
    display: flex;
    align-items: center;
    justify-content: center;
    width: 24px;
    height: 24px;
    background: rgba(0, 0, 0, 0.05);
    border-radius: var(--radius-sm);
    color: var(--text-muted);
  }
</style>
