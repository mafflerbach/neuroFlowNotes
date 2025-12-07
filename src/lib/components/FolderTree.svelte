<script lang="ts">
  import type { FolderNode } from "../types";
  import { editorStore, workspaceStore } from "../stores";
  import { listNotes } from "../services/api";
  import FolderTree from "./FolderTree.svelte";

  interface Props {
    node: FolderNode;
    depth?: number;
  }

  let { node, depth = 0 }: Props = $props();

  // Auto-expand first 2 levels
  let isExpanded = $state(false);
  $effect(() => {
    if (depth < 2) isExpanded = true;
  });

  function toggleExpand() {
    if (node.is_dir) {
      isExpanded = !isExpanded;
    }
  }

  async function handleClick() {
    if (node.is_dir) {
      toggleExpand();
    } else {
      // Find the note ID from the path
      try {
        const notes = await listNotes();
        const note = notes.find((n) => n.path === node.path);
        if (note) {
          workspaceStore.openDoc({
            path: note.path,
            id: note.id,
            title: note.title,
          });
        }
      } catch (e) {
        console.error("Failed to open note:", e);
      }
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Enter" || e.key === " ") {
      e.preventDefault();
      handleClick();
    }
  }

  const isActive = $derived(editorStore.currentPath === node.path);
  const paddingLeft = $derived(`${depth * 16 + 8}px`);
</script>

<div class="tree-node">
  <button
    class="tree-item"
    class:is-dir={node.is_dir}
    class:is-file={!node.is_dir}
    class:is-active={isActive}
    style:padding-left={paddingLeft}
    onclick={handleClick}
    onkeydown={handleKeydown}
  >
    {#if node.is_dir}
      <span class="icon">{isExpanded ? "▼" : "▶"}</span>
      <span class="folder-icon">📁</span>
    {:else}
      <span class="icon"></span>
      <span class="file-icon">📄</span>
    {/if}
    <span class="name">{node.name}</span>
  </button>

  {#if node.is_dir && isExpanded && node.children.length > 0}
    <div class="children">
      {#each node.children as child (child.path)}
        <FolderTree node={child} depth={depth + 1} />
      {/each}
    </div>
  {/if}
</div>

<style>
  .tree-node {
    user-select: none;
  }

  .tree-item {
    display: flex;
    align-items: center;
    gap: 4px;
    width: 100%;
    padding: 4px 8px;
    border: none;
    background: transparent;
    text-align: left;
    cursor: pointer;
    font-size: 13px;
    color: var(--text-color, #333);
    border-radius: 4px;
  }

  .tree-item:hover {
    background: var(--hover-bg, #f0f0f0);
  }

  .tree-item.is-active {
    background: var(--active-bg, #e0e7ff);
    color: var(--active-color, #3b5998);
  }

  .icon {
    width: 12px;
    font-size: 10px;
    color: var(--icon-color, #666);
  }

  .folder-icon,
  .file-icon {
    font-size: 14px;
  }

  .name {
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .is-file .name {
    /* Remove .md extension for display */
  }

  .children {
    /* Children are indented via padding-left on items */
  }
</style>
