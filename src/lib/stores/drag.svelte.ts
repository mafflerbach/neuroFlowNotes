/**
 * Drag & Drop state store for FolderTree operations.
 */

import type { FolderNode } from "../types";

class DragStore {
  draggedNode = $state<FolderNode | null>(null);
  dropTargetPath = $state<string | null>(null);

  /** Start dragging a node */
  startDrag(node: FolderNode) {
    this.draggedNode = node;
  }

  /** Set the current drop target */
  setDropTarget(path: string | null) {
    this.dropTargetPath = path;
  }

  /** Check if a folder is a valid drop target */
  isValidDropTarget(targetNode: FolderNode): boolean {
    if (!this.draggedNode) return false;
    if (!targetNode.is_dir) return false;

    // Can't drop on itself
    if (this.draggedNode.path === targetNode.path) return false;

    // Can't drop a folder into itself or its children
    if (this.draggedNode.is_dir) {
      const draggedPrefix = this.draggedNode.path + "/";
      if (targetNode.path.startsWith(draggedPrefix)) return false;
    }

    // Can't drop into current parent (no-op)
    const draggedParent = this.getParentPath(this.draggedNode.path);
    if (draggedParent === targetNode.path) return false;

    return true;
  }

  /** Get parent path from a path */
  getParentPath(path: string): string {
    const lastSlash = path.lastIndexOf("/");
    return lastSlash >= 0 ? path.substring(0, lastSlash) : "";
  }

  /** Calculate new path when dropping */
  getNewPath(targetFolderPath: string): string | null {
    if (!this.draggedNode) return null;

    const fileName = this.draggedNode.name;
    return targetFolderPath ? `${targetFolderPath}/${fileName}` : fileName;
  }

  /** End drag operation */
  endDrag() {
    this.draggedNode = null;
    this.dropTargetPath = null;
  }
}

export const dragStore = new DragStore();
