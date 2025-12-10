<script lang="ts">
  import { ChevronRight, Folder, File, Image, FileAudio, FileVideo, FileText } from "lucide-svelte";
  import type { FolderNode } from "../types";
  import { editorStore, workspaceStore, vaultStore, dragStore } from "../stores";
  import { listNotes, renameNote, deleteNote, deleteFolder, renameFolder, createFolder, saveNote, getNoteContent } from "../services/api";
  import { replaceH1Title } from "../utils/docListUtils";
  import { isImageFile, isAudioFile, isVideoFile, isPdfFile, isMediaFile } from "../utils/fileTypes";
  import { ask } from "@tauri-apps/plugin-dialog";
  import FolderTree from "./FolderTree.svelte";
  import TreeContextMenu from "./folder-tree/TreeContextMenu.svelte";

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
    } else if (isMediaFile(node.name)) {
      // Open media file in the viewer
      workspaceStore.openMediaFile(node.path);
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

      if (node.is_dir) {
        await renameFolder(node.path, newPath);
        workspaceStore.updateFolderPath(node.path, newPath);
      } else {
        // First, update the H1 in the file content
        const noteContent = await getNoteContent(node.path);
        const updatedContent = replaceH1Title(noteContent.content, renameValue);
        await saveNote(node.path, updatedContent);

        // Then rename the file
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
        await deleteFolder(node.path);
        // Close any open docs that were in this folder
        const folderPrefix = node.path ? `${node.path}/` : "";
        if (editorStore.currentPath?.startsWith(folderPrefix)) {
          workspaceStore.closeDoc(editorStore.currentPath);
        }
      } else {
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
      isCreatingNew = false;
      return;
    }

    try {
      const basePath = node.path || "";
      const newPath = basePath ? `${basePath}/${newItemName}` : newItemName;

      if (newItemType === "folder") {
        await createFolder(newPath);
      } else {
        const filePath = newItemName.endsWith(".md") ? newPath : `${newPath}.md`;
        const content = `# ${newItemName.replace(/\.md$/, "")}\n\n`;
        await saveNote(filePath, content);
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

  // Drag & Drop handlers
  function handleDragStart(e: DragEvent) {
    if (!e.dataTransfer) return;

    dragStore.startDrag(node);
    e.dataTransfer.effectAllowed = "move";
    e.dataTransfer.setData("text/plain", node.path);
  }

  function handleDragOver(e: DragEvent) {
    if (!dragStore.draggedNode) return;
    if (!dragStore.isValidDropTarget(node)) return;

    e.preventDefault();
    e.stopPropagation();

    if (e.dataTransfer) {
      e.dataTransfer.dropEffect = "move";
    }

    dragStore.setDropTarget(node.path);
  }

  function handleDragLeave(e: DragEvent) {
    // Only clear if we're actually leaving this element
    const relatedTarget = e.relatedTarget as HTMLElement | null;
    if (relatedTarget && (e.currentTarget as HTMLElement).contains(relatedTarget)) {
      return;
    }

    if (dragStore.dropTargetPath === node.path) {
      dragStore.setDropTarget(null);
    }
  }

  async function handleDrop(e: DragEvent) {
    const draggedNode = dragStore.draggedNode;
    if (!draggedNode) return; // Let event bubble up
    if (!dragStore.isValidDropTarget(node)) return; // Let event bubble up to root drop zone

    // Only prevent default and stop propagation if this IS a valid drop target
    e.preventDefault();
    e.stopPropagation();

    const newPath = dragStore.getNewPath(node.path);
    if (!newPath) return;

    try {

      if (draggedNode.is_dir) {
        await renameFolder(draggedNode.path, newPath);
      } else {
        await renameNote(draggedNode.path, newPath);
      }

      // Update workspace if the moved file is open
      workspaceStore.updateDocPath(draggedNode.path, newPath);

      // Reload editor if needed
      if (editorStore.currentPath === draggedNode.path) {
        await editorStore.openNote(newPath);
      }

      await vaultStore.refreshFolderTree();
    } catch (e) {
      console.error("[FolderTree] Failed to move:", e);
    }

    dragStore.endDrag();
  }

  function handleDragEnd() {
    dragStore.endDrag();
  }

  const isActive = $derived(editorStore.currentPath === node.path);
  const isDragging = $derived(dragStore.draggedNode?.path === node.path);
  const isDropTarget = $derived(
    dragStore.dropTargetPath === node.path && dragStore.isValidDropTarget(node)
  );
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
      class:is-dragging={isDragging}
      class:is-drop-target={isDropTarget}
      style:padding-left={paddingLeft}
      draggable="true"
      onclick={handleClick}
      onkeydown={handleKeydown}
      oncontextmenu={handleContextMenu}
      ondragstart={handleDragStart}
      ondragover={handleDragOver}
      ondragleave={handleDragLeave}
      ondrop={handleDrop}
      ondragend={handleDragEnd}
    >
      {#if node.is_dir}
        <span class="chevron" class:expanded={isExpanded}>
          <ChevronRight size={10} />
        </span>
        <span class="folder-icon">
          <Folder size={14} />
        </span>
      {:else}
        <span class="chevron-placeholder"></span>
        <span class="file-icon" class:is-image={isImageFile(node.name)} class:is-audio={isAudioFile(node.name)} class:is-video={isVideoFile(node.name)} class:is-pdf={isPdfFile(node.name)}>
          {#if isImageFile(node.name)}
            <Image size={14} />
          {:else if isAudioFile(node.name)}
            <FileAudio size={14} />
          {:else if isVideoFile(node.name)}
            <FileVideo size={14} />
          {:else if isPdfFile(node.name)}
            <FileText size={14} />
          {:else}
            <File size={14} />
          {/if}
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
            <Folder size={14} />
          {:else}
            <File size={14} />
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
    <TreeContextMenu
      isDir={node.is_dir}
      x={contextMenuX}
      y={contextMenuY}
      onNewFile={startCreateFile}
      onNewFolder={startCreateFolder}
      onRename={startRename}
      onDelete={handleDelete}
      onClose={closeContextMenu}
    />
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
    gap: var(--spacing-1);
    width: 100%;
    padding: var(--spacing-1) var(--spacing-2);
    border: none;
    background: transparent;
    text-align: left;
    cursor: pointer;
    font-size: var(--font-size-base);
    color: var(--text-primary);
    border-radius: var(--radius-sm);
  }

  .tree-item:hover {
    background: var(--tree-item-hover-bg);
  }

  .tree-item.is-active {
    background: var(--tree-item-selected-bg);
    color: var(--color-primary);
  }

  .tree-item.is-dragging {
    opacity: 0.5;
    background: var(--bg-surface-sunken);
  }

  .tree-item.is-drop-target {
    background: var(--color-primary-light);
    outline: 2px dashed var(--color-primary);
    outline-offset: -2px;
  }

  .tree-item.is-drop-target .folder-icon {
    color: var(--color-primary);
  }

  .chevron {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 14px;
    height: 14px;
    color: var(--tree-icon-color);
    transition: transform var(--transition-normal);
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
    color: var(--tree-file-icon-color);
    flex-shrink: 0;
  }

  .folder-icon {
    color: var(--tree-folder-icon-color);
  }

  .file-icon.is-image {
    color: #10b981; /* Green for images */
  }

  .file-icon.is-audio {
    color: #8b5cf6; /* Purple for audio */
  }

  .file-icon.is-video {
    color: #f59e0b; /* Amber for video */
  }

  .file-icon.is-pdf {
    color: #ef4444; /* Red for PDF */
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
    gap: var(--spacing-1);
    padding: 2px var(--spacing-2);
  }

  .rename-input {
    flex: 1;
    padding: var(--spacing-1) var(--spacing-2);
    border: 1px solid var(--input-border-focus);
    border-radius: var(--radius-sm);
    font-size: var(--font-size-base);
    background: var(--input-bg);
    color: var(--input-text);
    outline: none;
  }

  .rename-input:focus {
    box-shadow: var(--shadow-focus);
  }
</style>
