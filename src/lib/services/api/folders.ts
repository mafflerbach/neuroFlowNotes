/**
 * Folders API - folder operations
 */

import { invoke } from "@tauri-apps/api/core";
import type { FolderNode } from "../../types";

export async function createFolder(path: string): Promise<void> {
  return invoke("create_folder", { path });
}

export async function renameFolder(oldPath: string, newPath: string): Promise<number[]> {
  return invoke<number[]>("rename_folder", { oldPath, newPath });
}

export async function deleteFolder(path: string): Promise<number[]> {
  return invoke<number[]>("delete_folder", { path });
}

export async function getFolderTree(): Promise<FolderNode> {
  return invoke<FolderNode>("get_folder_tree");
}
