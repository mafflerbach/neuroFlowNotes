/**
 * Import API - vault import operations
 */

import { invoke } from "@tauri-apps/api/core";
import type { ImportVaultRequest, ImportResult } from "../../types";

/**
 * Import an Obsidian vault into the current vault.
 *
 * Copies all markdown files and assets, preserving folder structure.
 * Parses YAML frontmatter and converts to properties.
 * Merges frontmatter tags with inline tags.
 *
 * Listen for "import:progress" events to get real-time progress updates.
 */
export async function importObsidianVault(request: ImportVaultRequest): Promise<ImportResult> {
  return invoke<ImportResult>("import_obsidian_vault", { request });
}
