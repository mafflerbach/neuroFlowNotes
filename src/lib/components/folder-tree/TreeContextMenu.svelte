<script lang="ts">
  import { FilePlus, FolderPlus, Pencil, Trash2 } from "lucide-svelte";

  interface Props {
    isDir: boolean;
    x: number;
    y: number;
    onNewFile?: () => void;
    onNewFolder?: () => void;
    onRename: () => void;
    onDelete: () => void;
    onClose: () => void;
  }

  let {
    isDir,
    x,
    y,
    onNewFile,
    onNewFolder,
    onRename,
    onDelete,
    onClose,
  }: Props = $props();

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Escape") {
      onClose();
    }
  }
</script>

<div
  class="context-menu"
  role="menu"
  tabindex="-1"
  style:left="{x}px"
  style:top="{y}px"
  onclick={(e) => e.stopPropagation()}
  onkeydown={handleKeydown}
>
  {#if isDir}
    <button class="menu-item" onclick={onNewFile}>
      <FilePlus size={14} />
      New Note
    </button>
    <button class="menu-item" onclick={onNewFolder}>
      <FolderPlus size={14} />
      New Folder
    </button>
    <div class="menu-divider"></div>
  {/if}
  <button class="menu-item" onclick={onRename}>
    <Pencil size={14} />
    Rename
  </button>
  <button class="menu-item danger" onclick={onDelete}>
    <Trash2 size={14} />
    Delete
  </button>
</div>

<style>
  .context-menu {
    position: fixed;
    z-index: var(--z-context-menu);
    min-width: 160px;
    background: var(--context-menu-bg);
    border: 1px solid var(--context-menu-border);
    border-radius: var(--radius-lg);
    box-shadow: var(--context-menu-shadow);
    padding: var(--spacing-1);
  }

  .menu-item {
    display: flex;
    align-items: center;
    gap: var(--spacing-2);
    width: 100%;
    padding: var(--spacing-2) var(--spacing-3);
    border: none;
    background: transparent;
    text-align: left;
    cursor: pointer;
    font-size: var(--font-size-base);
    color: var(--text-primary);
    border-radius: var(--radius-sm);
  }

  .menu-item:hover {
    background: var(--context-menu-item-hover-bg);
  }

  .menu-item.danger {
    color: var(--color-error);
  }

  .menu-item.danger:hover {
    background: var(--color-error-light);
  }

  .menu-divider {
    height: 1px;
    margin: var(--spacing-1) 0;
    background: var(--context-menu-separator);
  }
</style>
