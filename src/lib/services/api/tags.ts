/**
 * Tags API - tag operations
 */

import { invoke } from "@tauri-apps/api/core";
import type { TagDto } from "../../types";

export async function listTags(): Promise<TagDto[]> {
  return invoke<TagDto[]>("list_tags");
}

/**
 * Get all tag names, sorted alphabetically.
 */
export async function getAllTags(): Promise<string[]> {
  const tags = await listTags();
  return tags.map((t) => t.tag).sort((a, b) => a.localeCompare(b));
}
