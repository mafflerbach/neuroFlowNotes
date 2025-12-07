<script lang="ts">
  import { vaultStore } from "../stores";
  import FolderTree from "./FolderTree.svelte";
  import { open } from "@tauri-apps/plugin-dialog";
  import { saveNote, createFolder } from "../services/api";

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
</script>

<aside class="sidebar">
  <div class="sidebar-header">
    {#if vaultStore.isOpen}
      <div class="header-row">
        <h2 class="vault-name">{vaultStore.info?.name}</h2>
        <div class="header-actions">
          <button class="header-btn" onclick={startCreateFile} title="New Note">
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z" />
              <polyline points="14 2 14 8 20 8" />
              <line x1="12" y1="18" x2="12" y2="12" />
              <line x1="9" y1="15" x2="15" y2="15" />
            </svg>
          </button>
          <button class="header-btn" onclick={startCreateFolder} title="New Folder">
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z" />
              <line x1="12" y1="11" x2="12" y2="17" />
              <line x1="9" y1="14" x2="15" y2="14" />
            </svg>
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
      <div class="folder-tree">
        <!-- New item input for root -->
        {#if isCreatingNew}
          <div class="new-item-wrapper">
            <span class="item-icon">
              {#if newItemType === "folder"}
                <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z" />
                </svg>
              {:else}
                <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z" />
                  <polyline points="14 2 14 8 20 8" />
                </svg>
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
    width: 260px;
    min-width: 200px;
    max-width: 400px;
    height: 100%;
    background: var(--sidebar-bg, #f5f5f5);
    border-right: 1px solid var(--border-color, #e0e0e0);
  }

  .sidebar-header {
    padding: 12px 16px;
    border-bottom: 1px solid var(--border-color, #e0e0e0);
  }

  .header-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
  }

  .header-actions {
    display: flex;
    gap: 4px;
  }

  .header-btn {
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
  }

  .header-btn:hover {
    background: var(--hover-bg, #e8e8e8);
    color: var(--text-color, #333);
  }

  .vault-name {
    font-size: 16px;
    font-weight: 600;
    margin: 0;
    color: var(--text-color, #333);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .note-count {
    font-size: 12px;
    color: var(--text-muted, #666);
  }

  .new-item-wrapper {
    display: flex;
    align-items: center;
    gap: 4px;
    padding: 4px 8px;
  }

  .item-icon {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 14px;
    height: 14px;
    color: var(--text-muted, #666);
    flex-shrink: 0;
  }

  .new-item-input {
    flex: 1;
    padding: 4px 8px;
    border: 1px solid var(--primary-color, #4f6bed);
    border-radius: 4px;
    font-size: 13px;
    background: var(--input-bg, #fff);
    color: var(--text-color, #333);
    outline: none;
  }

  .new-item-input:focus {
    box-shadow: 0 0 0 2px rgba(79, 107, 237, 0.2);
  }

  .sidebar-content {
    flex: 1;
    overflow-y: auto;
    padding: 8px;
  }

  .folder-tree {
    /* Tree styles handled by FolderTree component */
  }

  .loading,
  .error,
  .empty-state {
    padding: 16px;
    text-align: center;
    color: var(--text-muted, #666);
  }

  .error {
    color: var(--error-color, #d32f2f);
  }

  .open-vault-btn {
    margin-top: 12px;
    padding: 8px 16px;
    background: var(--primary-color, #3b5998);
    color: white;
    border: none;
    border-radius: 6px;
    cursor: pointer;
    font-size: 14px;
  }

  .open-vault-btn:hover {
    background: var(--primary-hover, #2d4373);
  }

  .sidebar-footer {
    padding: 12px;
    border-top: 1px solid var(--border-color, #e0e0e0);
  }

  .action-btn {
    width: 100%;
    padding: 8px;
    background: transparent;
    border: 1px solid var(--border-color, #e0e0e0);
    border-radius: 6px;
    cursor: pointer;
    font-size: 13px;
    color: var(--text-color, #333);
  }

  .action-btn:hover {
    background: var(--hover-bg, #e8e8e8);
  }
</style>
