<script lang="ts">
  import type { FolderNode } from "../types";
  import { editorStore, workspaceStore, vaultStore } from "../stores";
  import { listNotes, renameNote, deleteNote, deleteFolder, createFolder, saveNote, getNoteContent } from "../services/api";
  import { replaceH1Title } from "../utils/docListUtils";
  import { ask } from "@tauri-apps/plugin-dialog";
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

  // Context menu state
  let showContextMenu = $state(false);
  let contextMenuX = $state(0);
  let contextMenuY = $state(0);

  // Rename state
  let isRenaming = $state(false);
  let renameValue = $state("");
  let renameInput = $state<HTMLInputElement | null>(null);

  // New item state
  let isCreatingNew = $state(false);
  let newItemType = $state<"file" | "folder">("file");
  let newItemName = $state("");
  let newItemInput = $state<HTMLInputElement | null>(null);

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

  function handleContextMenu(e: MouseEvent) {
    e.preventDefault();
    e.stopPropagation();
    showContextMenu = true;
    contextMenuX = e.clientX;
    contextMenuY = e.clientY;
  }

  function closeContextMenu() {
    showContextMenu = false;
  }

  function handleWindowClick() {
    closeContextMenu();
  }

  // Rename
  function startRename() {
    closeContextMenu();
    isRenaming = true;
    renameValue = node.name.replace(/\.md$/, "");
    // Focus input after render
    setTimeout(() => renameInput?.focus(), 0);
  }

  async function confirmRename() {
    if (!renameValue.trim()) {
      isRenaming = false;
      return;
    }

    const newName = node.is_dir ? renameValue : `${renameValue}.md`;
    if (newName === node.name) {
      isRenaming = false;
      return;
    }

    try {
      const dir = node.path.includes("/")
        ? node.path.substring(0, node.path.lastIndexOf("/"))
        : "";
      const newPath = dir ? `${dir}/${newName}` : newName;

      if (!node.is_dir) {
        // First, update the H1 in the file content
        console.log("[FolderTree] Updating H1 in file content...");
        const noteContent = await getNoteContent(node.path);
        const updatedContent = replaceH1Title(noteContent.content, renameValue);
        await saveNote(node.path, updatedContent);

        // Then rename the file
        console.log("[FolderTree] Renaming file:", node.path, "->", newPath);
        await renameNote(node.path, newPath);

        // Update workspace if this file is open
        workspaceStore.updateDocPath(node.path, newPath, renameValue);

        // Reload the editor if this file is currently open
        if (editorStore.currentPath === node.path) {
          await editorStore.openNote(newPath);
        }
      }
      await vaultStore.refreshFolderTree();
    } catch (e) {
      console.error("[FolderTree] Failed to rename:", e);
    }

    isRenaming = false;
  }

  function cancelRename() {
    isRenaming = false;
  }

  function handleRenameKeydown(e: KeyboardEvent) {
    if (e.key === "Enter") {
      e.preventDefault();
      confirmRename();
    } else if (e.key === "Escape") {
      cancelRename();
    }
  }

  // Delete
  async function handleDelete() {
    // Close context menu immediately
    closeContextMenu();

    const confirmMessage = node.is_dir
      ? `Delete folder "${node.name}" and all its contents?`
      : `Delete "${node.name}"?`;

    // Use Tauri's native dialog (properly async)
    const confirmed = await ask(confirmMessage, {
      title: "Confirm Delete",
      kind: "warning",
    });

    if (!confirmed) {
      return;
    }

    try {
      if (node.is_dir) {
        console.log("[FolderTree] Deleting folder:", node.path);
        await deleteFolder(node.path);
        // Close any open docs that were in this folder
        const folderPrefix = node.path ? `${node.path}/` : "";
        if (editorStore.currentPath?.startsWith(folderPrefix)) {
          workspaceStore.closeDoc(editorStore.currentPath);
        }
      } else {
        console.log("[FolderTree] Deleting file:", node.path);
        await deleteNote(node.path);
        // Close if this file is open
        if (editorStore.currentPath === node.path) {
          workspaceStore.closeDoc(node.path);
        }
      }
      await vaultStore.refreshFolderTree();
    } catch (e) {
      console.error("[FolderTree] Failed to delete:", e);
    }
  }

  // Create new file/folder
  function startCreateFile() {
    closeContextMenu();
    if (!node.is_dir) return;
    isExpanded = true;
    isCreatingNew = true;
    newItemType = "file";
    newItemName = "";
    setTimeout(() => newItemInput?.focus(), 0);
  }

  function startCreateFolder() {
    closeContextMenu();
    if (!node.is_dir) return;
    isExpanded = true;
    isCreatingNew = true;
    newItemType = "folder";
    newItemName = "";
    setTimeout(() => newItemInput?.focus(), 0);
  }

  async function confirmCreate() {
    if (!newItemName.trim()) {
      console.log("[FolderTree] confirmCreate: empty name, cancelling");
      isCreatingNew = false;
      return;
    }

    try {
      const basePath = node.path || "";
      const newPath = basePath ? `${basePath}/${newItemName}` : newItemName;

      console.log("[FolderTree] confirmCreate:", { type: newItemType, basePath, newPath });

      if (newItemType === "folder") {
        console.log("[FolderTree] Creating folder:", newPath);
        await createFolder(newPath);
        console.log("[FolderTree] Folder created successfully");
      } else {
        const filePath = newItemName.endsWith(".md") ? newPath : `${newPath}.md`;
        const content = `# ${newItemName.replace(/\.md$/, "")}\n\n`;
        console.log("[FolderTree] Creating file:", filePath);
        await saveNote(filePath, content);
        console.log("[FolderTree] File created successfully");
      }
      await vaultStore.refreshFolderTree();
    } catch (e) {
      console.error("[FolderTree] Failed to create:", e);
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

  const isActive = $derived(editorStore.currentPath === node.path);
  const paddingLeft = $derived(`${depth * 16 + 8}px`);
  const newItemPadding = $derived(`${(depth + 1) * 16 + 8}px`);
</script>

<svelte:window onclick={handleWindowClick} />

<div class="tree-node">
  {#if isRenaming}
    <div class="rename-input-wrapper" style:padding-left={paddingLeft}>
      <input
        bind:this={renameInput}
        type="text"
        class="rename-input"
        bind:value={renameValue}
        onkeydown={handleRenameKeydown}
        onblur={confirmRename}
      />
    </div>
  {:else}
    <button
      class="tree-item"
      class:is-dir={node.is_dir}
      class:is-file={!node.is_dir}
      class:is-active={isActive}
      style:padding-left={paddingLeft}
      onclick={handleClick}
      onkeydown={handleKeydown}
      oncontextmenu={handleContextMenu}
    >
      {#if node.is_dir}
        <span class="chevron" class:expanded={isExpanded}>
          <svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M9 18l6-6-6-6" />
          </svg>
        </span>
        <span class="folder-icon">
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z" />
          </svg>
        </span>
      {:else}
        <span class="chevron-placeholder"></span>
        <span class="file-icon">
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z" />
            <polyline points="14 2 14 8 20 8" />
          </svg>
        </span>
      {/if}
      <span class="name">{node.name}</span>
    </button>
  {/if}

  {#if node.is_dir && isExpanded}
    <!-- New item input -->
    {#if isCreatingNew}
      <div class="new-item-wrapper" style:padding-left={newItemPadding}>
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
          class="rename-input"
          placeholder={newItemType === "folder" ? "New folder..." : "New note..."}
          bind:value={newItemName}
          onkeydown={handleNewItemKeydown}
          onblur={confirmCreate}
        />
      </div>
    {/if}

    {#if node.children.length > 0}
      <div class="children">
        {#each node.children as child (child.path)}
          <FolderTree node={child} depth={depth + 1} />
        {/each}
      </div>
    {/if}
  {/if}

  <!-- Context Menu -->
  {#if showContextMenu}
    <div
      class="context-menu"
      role="menu"
      tabindex="-1"
      style:left="{contextMenuX}px"
      style:top="{contextMenuY}px"
      onclick={(e) => e.stopPropagation()}
      onkeydown={(e) => e.key === "Escape" && closeContextMenu()}
    >
      {#if node.is_dir}
        <button class="menu-item" onclick={startCreateFile}>
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z" />
            <polyline points="14 2 14 8 20 8" />
            <line x1="12" y1="18" x2="12" y2="12" />
            <line x1="9" y1="15" x2="15" y2="15" />
          </svg>
          New Note
        </button>
        <button class="menu-item" onclick={startCreateFolder}>
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z" />
            <line x1="12" y1="11" x2="12" y2="17" />
            <line x1="9" y1="14" x2="15" y2="14" />
          </svg>
          New Folder
        </button>
        <div class="menu-divider"></div>
      {/if}
      <button class="menu-item" onclick={startRename}>
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M17 3a2.828 2.828 0 1 1 4 4L7.5 20.5 2 22l1.5-5.5L17 3z" />
        </svg>
        Rename
      </button>
      <button class="menu-item danger" onclick={handleDelete}>
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <polyline points="3 6 5 6 21 6" />
          <path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2" />
        </svg>
        Delete
      </button>
    </div>
  {/if}
</div>

<style>
  .tree-node {
    user-select: none;
    position: relative;
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

  .chevron {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 14px;
    height: 14px;
    color: var(--text-muted, #666);
    transition: transform 0.15s ease;
    flex-shrink: 0;
  }

  .chevron.expanded {
    transform: rotate(90deg);
  }

  .chevron-placeholder {
    width: 14px;
    flex-shrink: 0;
  }

  .folder-icon,
  .file-icon,
  .item-icon {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 14px;
    height: 14px;
    color: var(--text-muted, #666);
    flex-shrink: 0;
  }

  .folder-icon {
    color: var(--primary-color, #4f6bed);
  }

  .name {
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .rename-input-wrapper,
  .new-item-wrapper {
    display: flex;
    align-items: center;
    gap: 4px;
    padding: 2px 8px;
  }

  .rename-input {
    flex: 1;
    padding: 4px 8px;
    border: 1px solid var(--primary-color, #4f6bed);
    border-radius: 4px;
    font-size: 13px;
    background: var(--input-bg, #fff);
    color: var(--text-color, #333);
    outline: none;
  }

  .rename-input:focus {
    box-shadow: 0 0 0 2px rgba(79, 107, 237, 0.2);
  }

  /* Context Menu */
  .context-menu {
    position: fixed;
    z-index: 1000;
    min-width: 160px;
    background: var(--modal-bg, #fff);
    border: 1px solid var(--border-color, #e0e0e0);
    border-radius: 8px;
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
    padding: 4px;
  }

  .menu-item {
    display: flex;
    align-items: center;
    gap: 8px;
    width: 100%;
    padding: 8px 12px;
    border: none;
    background: transparent;
    text-align: left;
    cursor: pointer;
    font-size: 13px;
    color: var(--text-color, #333);
    border-radius: 4px;
  }

  .menu-item:hover {
    background: var(--hover-bg, #f0f0f0);
  }

  .menu-item.danger {
    color: var(--error-color, #d32f2f);
  }

  .menu-item.danger:hover {
    background: var(--error-bg, #fee);
  }

  .menu-divider {
    height: 1px;
    margin: 4px 0;
    background: var(--border-color, #e0e0e0);
  }
</style>
