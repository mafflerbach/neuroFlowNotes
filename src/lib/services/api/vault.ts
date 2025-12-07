/**
 * Vault API - vault management operations
 */

import { invoke } from "@tauri-apps/api/core";
import type { VaultInfo } from "../../types";

export async function openVault(path: string): Promise<VaultInfo> {
  return invoke<VaultInfo>("open_vault", { path });
}

export async function closeVault(): Promise<void> {
  return invoke("close_vault");
}

export async function getVaultInfo(): Promise<VaultInfo | null> {
  return invoke<VaultInfo | null>("get_vault_info");
}
