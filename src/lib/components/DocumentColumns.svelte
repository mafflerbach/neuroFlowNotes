<script lang="ts">
  import { workspaceStore } from "../stores/workspace.svelte";
  import NoteEditor from "./NoteEditor.svelte";
  import PropertiesPanel from "./PropertiesPanel.svelte";

  interface Props {
    onLinkClick?: (path: string) => void;
  }

  let { onLinkClick }: Props = $props();

  const visibleDocs = $derived(workspaceStore.visibleDocs);
  const activeDoc = $derived(workspaceStore.activeDoc);
  const multiColumnEditable = $derived(workspaceStore.multiColumnEditable);

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

  function handleLinkClick(targetPath: string) {
    onLinkClick?.(targetPath);
  }

  // Mock properties for now - in real implementation these would come from SQLite
  function getPropertiesForNote(_noteId: number): { key: string; value: string; type: "text" }[] {
    return [];
  }
</script>

<div class="document-columns" class:single={visibleDocs.length === 1}>
  {#each visibleDocs as doc, i (doc.path)}
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
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M18 6L6 18M6 6l12 12" />
          </svg>
        </button>
      </div>

      <!-- Editor area -->
      <div class="column-content">
        <div class="editor-wrapper">
          <NoteEditor
            noteId={doc.id}
            readonly={!isEditable}
            onLinkClick={handleLinkClick}
          />
        </div>

        <!-- Properties panel (only for active doc) -->
        {#if isActive}
          <PropertiesPanel
            noteId={doc.id}
            properties={getPropertiesForNote(doc.id)}
          />
        {/if}
      </div>

      <!-- Read-only overlay indicator -->
      {#if !isEditable}
        <div class="readonly-indicator" title="Click to make active and edit">
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M1 12s4-8 11-8 11 8 11 8-4 8-11 8-11-8-11-8z" />
            <circle cx="12" cy="12" r="3" />
          </svg>
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
    background: var(--border-color, #e0e0e0);
    overflow: hidden;
  }

  .column {
    flex: 1;
    display: flex;
    flex-direction: column;
    background: var(--editor-bg, #fff);
    min-width: 300px;
    position: relative;
    transition: opacity 0.15s;
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
    padding: 8px 12px;
    background: var(--column-header-bg, #f8f9fa);
    border-bottom: 1px solid var(--border-color, #e0e0e0);
  }

  .column.active .column-header {
    background: var(--active-column-header-bg, #e0e7ff);
  }

  .column-title {
    flex: 1;
    text-align: left;
    font-size: 13px;
    font-weight: 500;
    color: var(--text-color, #333);
    background: transparent;
    border: none;
    cursor: pointer;
    padding: 4px 0;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .column.active .column-title {
    color: var(--primary-color, #4f6bed);
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
    border-radius: 4px;
    color: var(--text-muted, #999);
    cursor: pointer;
  }

  .close-btn:hover {
    background: var(--error-bg, #fee);
    color: var(--error-color, #d32f2f);
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
    top: 8px;
    right: 44px;
    display: flex;
    align-items: center;
    justify-content: center;
    width: 24px;
    height: 24px;
    background: var(--readonly-indicator-bg, rgba(0, 0, 0, 0.05));
    border-radius: 4px;
    color: var(--text-muted, #999);
  }
</style>
