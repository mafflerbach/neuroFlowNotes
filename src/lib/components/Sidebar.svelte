<script lang="ts">
  import { FilePlus, FolderPlus, Folder, File } from "lucide-svelte";
  import { vaultStore, dragStore } from "../stores";
  import FolderTree from "./FolderTree.svelte";
  import { open } from "@tauri-apps/plugin-dialog";
  import { saveNote, createFolder, renameNote, renameFolder } from "../services/api";

  // Force reactivity by deriving from the store
  const folderTree = $derived(vaultStore.folderTree);
  const treeChildren = $derived(folderTree?.children ?? []);

  // New item creation state for root
  let isCreatingNew = $state(false);
  let newItemType = $state<"file" | "folder">("file");
  let newItemName = $state("");
  let newItemInput = $state<HTMLInputElement | null>(null);

  // Debug logging
  $effect(() => {
    console.log("[Sidebar] folderTree changed, children count:", treeChildren.length);
  });

  async function handleOpenVault() {
    try {
      const selected = await open({
        directory: true,
        multiple: false,
        title: "Select Vault Folder",
      });

      if (selected && typeof selected === "string") {
        await vaultStore.open(selected);
      }
    } catch (e) {
      console.error("Failed to open vault:", e);
    }
  }

  function startCreateFile() {
    isCreatingNew = true;
    newItemType = "file";
    newItemName = "";
    setTimeout(() => newItemInput?.focus(), 0);
  }

  function startCreateFolder() {
    isCreatingNew = true;
    newItemType = "folder";
    newItemName = "";
    setTimeout(() => newItemInput?.focus(), 0);
  }

  async function confirmCreate() {
    if (!newItemName.trim()) {
      isCreatingNew = false;
      return;
    }

    try {
      if (newItemType === "folder") {
        await createFolder(newItemName);
      } else {
        const filePath = newItemName.endsWith(".md") ? newItemName : `${newItemName}.md`;
        const content = `# ${newItemName.replace(/\.md$/, "")}\n\n`;
        await saveNote(filePath, content);
      }
      await vaultStore.refreshFolderTree();
    } catch (e) {
      console.error("Failed to create:", e);
    }

    isCreatingNew = false;
  }

  function cancelCreate() {
    isCreatingNew = false;
  }

  function handleNewItemKeydown(e: KeyboardEvent) {
    if (e.key === "Enter") {
      e.preventDefault();
      confirmCreate();
    } else if (e.key === "Escape") {
      cancelCreate();
    }
  }

  // Root drop zone for moving items to vault root
  let isRootDropTarget = $state(false);

  function isValidRootDrop(): boolean {
    if (!dragStore.draggedNode) return false;
    // Can't drop if already at root
    const parentPath = dragStore.getParentPath(dragStore.draggedNode.path);
    return parentPath !== "";
  }

  function handleRootDragOver(e: DragEvent) {
    if (!dragStore.draggedNode) return;
    if (!isValidRootDrop()) return;

    e.preventDefault();
    e.stopPropagation();

    if (e.dataTransfer) {
      e.dataTransfer.dropEffect = "move";
    }

    isRootDropTarget = true;
    dragStore.setDropTarget("");
  }

  function handleRootDragLeave(e: DragEvent) {
    const relatedTarget = e.relatedTarget as HTMLElement | null;
    if (relatedTarget && (e.currentTarget as HTMLElement).contains(relatedTarget)) {
      return;
    }
    isRootDropTarget = false;
    if (dragStore.dropTargetPath === "") {
      dragStore.setDropTarget(null);
    }
  }

  async function handleRootDrop(e: DragEvent) {
    e.preventDefault();
    e.stopPropagation();

    const draggedNode = dragStore.draggedNode;
    if (!draggedNode) return;
    if (!isValidRootDrop()) return;

    const newPath = draggedNode.name;

    try {

      if (draggedNode.is_dir) {
        await renameFolder(draggedNode.path, newPath);
      } else {
        await renameNote(draggedNode.path, newPath);
      }

      await vaultStore.refreshFolderTree();
    } catch (err) {
      console.error("[Sidebar] Move to root failed:", err);
    }

    isRootDropTarget = false;
    dragStore.endDrag();
  }
</script>

<aside class="sidebar">
  <div class="sidebar-header">
    {#if vaultStore.isOpen}
      <div class="header-row">
        <h2 class="vault-name">{vaultStore.info?.name}</h2>
        <div class="header-actions">
          <button class="header-btn" onclick={startCreateFile} title="New Note">
            <FilePlus size={14} />
          </button>
          <button class="header-btn" onclick={startCreateFolder} title="New Folder">
            <FolderPlus size={14} />
          </button>
        </div>
      </div>
      <span class="note-count">{vaultStore.info?.note_count} notes</span>
    {:else}
      <h2 class="vault-name">NeuroFlow Notes</h2>
    {/if}
  </div>

  <div class="sidebar-content">
    {#if vaultStore.isLoading}
      <div class="loading">Loading...</div>
    {:else if vaultStore.error}
      <div class="error">{vaultStore.error}</div>
    {:else if folderTree}
      <!-- svelte-ignore a11y_interactive_supports_focus -->
      <div
        class="folder-tree"
        class:is-root-drop-target={isRootDropTarget}
        ondragover={handleRootDragOver}
        ondragleave={handleRootDragLeave}
        ondrop={handleRootDrop}
        role="tree"
      >
        <!-- New item input for root -->
        {#if isCreatingNew}
          <div class="new-item-wrapper">
            <span class="item-icon">
              {#if newItemType === "folder"}
                <Folder size={14} />
              {:else}
                <File size={14} />
              {/if}
            </span>
            <input
              bind:this={newItemInput}
              type="text"
              class="new-item-input"
              placeholder={newItemType === "folder" ? "New folder..." : "New note..."}
              bind:value={newItemName}
              onkeydown={handleNewItemKeydown}
              onblur={confirmCreate}
            />
          </div>
        {/if}
        {#each treeChildren as child (child.path)}
          <FolderTree node={child} />
        {/each}
      </div>
    {:else}
      <div class="empty-state">
        <p>No vault open</p>
        <button class="open-vault-btn" onclick={handleOpenVault}>
          Open Vault
        </button>
      </div>
    {/if}
  </div>

  <div class="sidebar-footer">
    {#if vaultStore.isOpen}
      <button class="action-btn" onclick={handleOpenVault}>
        Switch Vault
      </button>
    {/if}
  </div>
</aside>

<style>
  .sidebar {
    display: flex;
    flex-direction: column;
    width: var(--sidebar-width);
    min-width: 200px;
    max-width: 400px;
    height: 100%;
    background: var(--sidebar-bg);
    border-right: 1px solid var(--sidebar-border);
  }

  .sidebar-header {
    padding: var(--spacing-3) var(--spacing-4);
    border-bottom: 1px solid var(--sidebar-border);
  }

  .header-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--spacing-2);
  }

  .header-actions {
    display: flex;
    gap: var(--spacing-1);
  }

  .header-btn {
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
  }

  .header-btn:hover {
    background: var(--bg-hover);
    color: var(--text-primary);
  }

  .vault-name {
    font-size: var(--font-size-lg);
    font-weight: var(--font-weight-semibold);
    margin: 0;
    color: var(--text-primary);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .note-count {
    font-size: var(--font-size-sm);
    color: var(--text-muted);
  }

  .new-item-wrapper {
    display: flex;
    align-items: center;
    gap: var(--spacing-1);
    padding: var(--spacing-1) var(--spacing-2);
  }

  .item-icon {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 14px;
    height: 14px;
    color: var(--text-muted);
    flex-shrink: 0;
  }

  .new-item-input {
    flex: 1;
    padding: var(--spacing-1) var(--spacing-2);
    border: 1px solid var(--input-border-focus);
    border-radius: var(--radius-sm);
    font-size: var(--font-size-base);
    background: var(--input-bg);
    color: var(--input-text);
    outline: none;
  }

  .new-item-input:focus {
    box-shadow: var(--shadow-focus);
  }

  .sidebar-content {
    flex: 1;
    overflow-y: auto;
    padding: var(--spacing-2);
  }

  .loading,
  .error,
  .empty-state {
    padding: var(--spacing-4);
    text-align: center;
    color: var(--text-muted);
  }

  .error {
    color: var(--color-error);
  }

  .open-vault-btn {
    margin-top: var(--spacing-3);
    padding: var(--spacing-2) var(--spacing-4);
    background: var(--btn-primary-bg);
    color: var(--btn-primary-text);
    border: none;
    border-radius: var(--radius-md);
    cursor: pointer;
    font-size: var(--font-size-md);
  }

  .open-vault-btn:hover {
    background: var(--btn-primary-bg-hover);
  }

  .sidebar-footer {
    padding: var(--spacing-3);
    border-top: 1px solid var(--sidebar-border);
  }

  .action-btn {
    width: 100%;
    padding: var(--spacing-2);
    background: transparent;
    border: 1px solid var(--border-default);
    border-radius: var(--radius-md);
    cursor: pointer;
    font-size: var(--font-size-base);
    color: var(--text-primary);
  }

  .action-btn:hover {
    background: var(--bg-hover);
  }

  .folder-tree {
    min-height: 100px;
    border-radius: var(--radius-md);
    transition: background var(--transition-normal), outline var(--transition-normal);
  }

  .folder-tree.is-root-drop-target {
    background: var(--color-primary-light);
    outline: 2px dashed var(--color-primary);
    outline-offset: -2px;
  }
</style>
