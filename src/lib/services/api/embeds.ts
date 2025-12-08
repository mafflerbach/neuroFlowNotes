/**
 * Embed-related API functions - for resolving wiki links and embeds.
 */

import { invoke } from "@tauri-apps/api/core";
import type { EmbedContent, HeadingInfo, ResolveEmbedRequest } from "../../types";

/**
 * Resolve an embed (![[target]] or ![[target#section]]).
 * Returns content for notes or asset URL for images.
 */
export async function resolveEmbed(request: ResolveEmbedRequest): Promise<EmbedContent> {
  const result = await invoke<{
    note_id: number | null;
    path: string;
    content: string | null;
    is_image: boolean;
    asset_url: string | null;
    error: string | null;
  }>("resolve_embed", { request: {
    target: request.target,
    section: request.section ?? null,
    depth: request.depth,
  }});

  return {
    noteId: result.note_id,
    path: result.path,
    content: result.content,
    isImage: result.is_image,
    assetUrl: result.asset_url,
    error: result.error,
  };
}

/**
 * Get all headings from a note (for section autocomplete).
 */
export async function getNoteHeadings(path: string): Promise<HeadingInfo[]> {
  const result = await invoke<Array<{
    level: number;
    text: string;
    slug: string;
  }>>("get_note_headings", { path });

  return result.map(h => ({
    level: h.level,
    text: h.text,
    slug: h.slug,
  }));
}

/**
 * Save a pasted image to the vault's assets folder.
 * @param imageData Base64-encoded image data
 * @param extension File extension (e.g., "png", "jpg")
 * @returns The relative path to the saved image (e.g., "assets/Pasted image 20251208143000.png")
 */
export async function savePastedImage(imageData: string, extension: string): Promise<string> {
  return await invoke<string>("save_pasted_image", { imageData, extension });
}
