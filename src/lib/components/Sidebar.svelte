<script lang="ts">
  import { vaultStore } from "../stores";
  import FolderTree from "./FolderTree.svelte";
  import { open } from "@tauri-apps/plugin-dialog";

  // Force reactivity by deriving from the store
  const folderTree = $derived(vaultStore.folderTree);
  const treeChildren = $derived(folderTree?.children ?? []);

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
</script>

<aside class="sidebar">
  <div class="sidebar-header">
    {#if vaultStore.isOpen}
      <h2 class="vault-name">{vaultStore.info?.name}</h2>
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
    padding: 16px;
    border-bottom: 1px solid var(--border-color, #e0e0e0);
  }

  .vault-name {
    font-size: 16px;
    font-weight: 600;
    margin: 0;
    color: var(--text-color, #333);
  }

  .note-count {
    font-size: 12px;
    color: var(--text-muted, #666);
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
