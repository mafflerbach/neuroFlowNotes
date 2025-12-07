/**
 * Tags API - tag operations
 */

import { invoke } from "@tauri-apps/api/core";
import type { TagDto } from "../../types";

export async function listTags(): Promise<TagDto[]> {
  return invoke<TagDto[]>("list_tags");
}
