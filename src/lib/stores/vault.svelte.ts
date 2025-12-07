/**
 * Vault store - manages the currently open vault state.
 */

import type { VaultInfo, FolderNode } from "../types";
import * as api from "../services/api";

class VaultStore {
  info = $state<VaultInfo | null>(null);
  folderTree = $state<FolderNode | null>(null);
  isLoading = $state(false);
  error = $state<string | null>(null);

  get isOpen() {
    return this.info !== null;
  }

  async open(path: string) {
    this.isLoading = true;
    this.error = null;

    try {
      this.info = await api.openVault(path);
      await this.refreshFolderTree();
    } catch (e) {
      this.error = e instanceof Error ? e.message : String(e);
      throw e;
    } finally {
      this.isLoading = false;
    }
  }

  async close() {
    try {
      await api.closeVault();
    } finally {
      this.info = null;
      this.folderTree = null;
    }
  }

  async refreshFolderTree() {
    if (!this.isOpen) {
      console.log("[VaultStore] refreshFolderTree: vault not open");
      return;
    }

    try {
      console.log("[VaultStore] refreshFolderTree: fetching...");
      const tree = await api.getFolderTree();
      console.log("[VaultStore] refreshFolderTree: got tree with", tree?.children?.length, "children");
      this.folderTree = tree;
    } catch (e) {
      console.error("[VaultStore] Failed to refresh folder tree:", e);
    }
  }

  async refresh() {
    if (!this.isOpen) return;

    try {
      this.info = await api.getVaultInfo();
      await this.refreshFolderTree();
    } catch (e) {
      console.error("Failed to refresh vault:", e);
    }
  }
}

export const vaultStore = new VaultStore();
