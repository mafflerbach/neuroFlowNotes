/**
 * Type definitions for the embedding and linking system.
 */

/**
 * Request to resolve an embed (![[target]] or ![[target#section]]).
 */
export interface ResolveEmbedRequest {
  /** The target note name or path (without .md extension). */
  target: string;
  /** Optional section slug to extract (e.g., "my-section" from "## My Section"). */
  section?: string;
  /** Current embedding depth (starts at 0, max 3). */
  depth: number;
}

/**
 * Result of resolving an embed.
 */
export interface EmbedContent {
  /** The note ID if found in database. */
  noteId: number | null;
  /** The resolved path to the note or image. */
  path: string;
  /** The markdown content to embed (for notes). */
  content: string | null;
  /** Whether this is an image embed. */
  isImage: boolean;
  /** Asset URL for images (using Tauri asset protocol). */
  assetUrl: string | null;
  /** Error message if resolution failed. */
  error: string | null;
}

/**
 * Information about a heading in a note (for section autocomplete).
 */
export interface HeadingInfo {
  /** Heading level (1-6). */
  level: number;
  /** The heading text as displayed. */
  text: string;
  /** URL-safe slug for linking (e.g., "my-section"). */
  slug: string;
}
