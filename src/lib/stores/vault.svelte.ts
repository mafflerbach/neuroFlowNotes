/**
 * Vault store - manages the currently open vault state.
 */

import type { VaultInfo, FolderNode } from "../types";
import * as api from "../services/api";
import { getSetting, setSetting } from "../services/settings";

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
      // Save as last opened vault
      setSetting("lastVaultPath", path);
    } catch (e) {
      this.error = e instanceof Error ? e.message : String(e);
      throw e;
    } finally {
      this.isLoading = false;
    }
  }

  /** Try to open the last used vault */
  async openLastVault(): Promise<boolean> {
    const lastPath = getSetting("lastVaultPath");
    if (lastPath) {
      try {
        await this.open(lastPath);
        return true;
      } catch (e) {
        console.warn("[VaultStore] Failed to open last vault:", e);
        // Clear the invalid path
        setSetting("lastVaultPath", null);
      }
    }
    return false;
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
    if (!this.isOpen) return;

    try {
      this.folderTree = await api.getFolderTree();
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
