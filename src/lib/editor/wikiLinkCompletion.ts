/**
 * Wiki-link autocomplete extension for CodeMirror
 * Triggers on [[ and shows available notes from the vault
 * Also provides section completion after # (e.g., [[note#section]])
 */

import {
  autocompletion,
  completionKeymap,
  startCompletion,
} from "@codemirror/autocomplete";
import type {
  CompletionContext,
  CompletionResult,
  Completion,
} from "@codemirror/autocomplete";
import { keymap, ViewPlugin, tooltips } from "@codemirror/view";
import type { ViewUpdate } from "@codemirror/view";
import { Prec, EditorState } from "@codemirror/state";
import { listNotes, getNoteHeadings, getFolderTree } from "../services/api";
import type { NoteListItem, HeadingInfo, FolderNode } from "../types";

// Cache for notes list (for section completion - need note IDs)
let notesCache: NoteListItem[] = [];
let notesCacheTimestamp = 0;
const NOTES_CACHE_TTL = 10000; // 10 seconds

// Cache for all files (notes + media)
interface FileItem {
  path: string;
  name: string;
  isMedia: boolean;
}
let filesCache: FileItem[] = [];
let filesCacheTimestamp = 0;
const FILES_CACHE_TTL = 10000; // 10 seconds
let filesFetchPromise: Promise<FileItem[]> | null = null;

// Cache for headings (per note path)
const headingsCache = new Map<string, { headings: HeadingInfo[]; timestamp: number }>();
const HEADINGS_CACHE_TTL = 30000; // 30 seconds

// Media file extensions
const MEDIA_EXTENSIONS = [
  "png", "jpg", "jpeg", "gif", "webp", "svg", "bmp", "ico",
  "mp3", "wav", "ogg", "m4a", "flac",
  "mp4", "webm", "mov", "avi",
  "pdf",
];

function isMediaFile(filename: string): boolean {
  const ext = filename.split(".").pop()?.toLowerCase() ?? "";
  return MEDIA_EXTENSIONS.includes(ext);
}

/**
 * Flatten folder tree into a list of files
 */
function flattenFolderTree(node: FolderNode, files: FileItem[] = []): FileItem[] {
  if (!node.is_dir) {
    files.push({
      path: node.path,
      name: node.name,
      isMedia: isMediaFile(node.name),
    });
  }
  for (const child of node.children) {
    flattenFolderTree(child, files);
  }
  return files;
}

/**
 * Prefetch all files (notes + media) in the background
 */
function prefetchFiles(): void {
  const now = Date.now();
  if (now - filesCacheTimestamp < FILES_CACHE_TTL) return;
  if (filesFetchPromise) return; // Already fetching

  filesFetchPromise = getFolderTree()
    .then((tree) => {
      filesCache = flattenFolderTree(tree);
      filesCacheTimestamp = Date.now();
      console.log("[WikiLink] Prefetched", filesCache.length, "files");
      return filesCache;
    })
    .catch((error) => {
      console.error("Failed to fetch files for autocomplete:", error);
      return filesCache;
    })
    .finally(() => {
      filesFetchPromise = null;
    });
}

/**
 * Get cached files (synchronous)
 */
function getCachedFiles(): FileItem[] {
  // Trigger prefetch if cache is stale
  prefetchFiles();
  return filesCache;
}

/**
 * Prefetch notes in the background (for section completion)
 */
function prefetchNotes(): void {
  const now = Date.now();
  if (now - notesCacheTimestamp < NOTES_CACHE_TTL) return;

  listNotes()
    .then((notes) => {
      notesCache = notes;
      notesCacheTimestamp = Date.now();
    })
    .catch((error) => {
      console.error("Failed to fetch notes for autocomplete:", error);
    });
}

/**
 * Get cached notes (synchronous)
 */
function getCachedNotes(): NoteListItem[] {
  prefetchNotes();
  return notesCache;
}

/**
 * Get cached headings for a note (async)
 */
async function getCachedHeadings(notePath: string): Promise<HeadingInfo[]> {
  const cached = headingsCache.get(notePath);
  if (cached && Date.now() - cached.timestamp < HEADINGS_CACHE_TTL) {
    return cached.headings;
  }

  try {
    const headings = await getNoteHeadings(notePath);
    headingsCache.set(notePath, { headings, timestamp: Date.now() });
    return headings;
  } catch (error) {
    console.error("[WikiLink] Failed to fetch headings for", notePath, error);
    return [];
  }
}

/**
 * Find the note path from the wiki link text before #
 */
function findNotePathForSection(linkText: string): string | null {
  const notes = getCachedNotes();
  const noteName = linkText.toLowerCase();

  const note = notes.find((n) => {
    const path = n.path.replace(/\.md$/, "").toLowerCase();
    const title = (n.title || "").toLowerCase();
    return path === noteName || path.endsWith(`/${noteName}`) || title === noteName;
  });

  return note?.path ?? null;
}

/**
 * Async completion source for section headings
 */
async function sectionCompletionSource(
  _context: CompletionContext,
  notePath: string,
  sectionQuery: string,
  sectionFrom: number
): Promise<CompletionResult | null> {
  const headings = await getCachedHeadings(notePath);

  if (headings.length === 0) {
    return null;
  }

  const options: Completion[] = headings
    .filter((h) => {
      if (!sectionQuery) return true;
      return h.slug.includes(sectionQuery) || h.text.toLowerCase().includes(sectionQuery);
    })
    .map((h) => ({
      label: h.slug,
      detail: `${"#".repeat(h.level)} ${h.text}`,
      type: "text",
      apply: `${h.slug}]]`,
    }));

  if (options.length === 0) {
    return null;
  }

  return {
    from: sectionFrom,
    options,
  };
}

/**
 * Completion source for wiki-links
 * Triggers when user types [[ or after #
 * Returns a Promise for async section completion
 */
function wikiLinkCompletionSource(
  context: CompletionContext
): CompletionResult | Promise<CompletionResult | null> | null {
  // Look for [[ pattern before cursor (with optional ! for embeds)
  const before = context.matchBefore(/!?\[\[[^\]]*$/);

  // If no [[ pattern, skip
  if (!before) {
    return null;
  }

  // Check if this is an embed (![[) or regular link ([[)
  const isEmbed = before.text.startsWith("!");
  const linkStart = isEmbed ? 3 : 2; // Skip ![[  or [[

  // Get the text after [[ or ![[
  const linkContent = before.text.slice(linkStart);

  // Check if we're completing a section (after #)
  const hashIndex = linkContent.indexOf("#");
  if (hashIndex !== -1) {
    const noteName = linkContent.slice(0, hashIndex);
    const sectionQuery = linkContent.slice(hashIndex + 1).toLowerCase();

    // Find the note path
    const notePath = findNotePathForSection(noteName);
    if (!notePath) {
      return null;
    }

    // Return async completion for sections
    const sectionFrom = before.from + linkStart + hashIndex + 1;
    return sectionCompletionSource(context, notePath, sectionQuery, sectionFrom);
  }

  // Regular file completion (synchronous from cache)
  const query = linkContent.toLowerCase();

  // Get cached files (notes + media)
  const files = getCachedFiles();

  // Filter and map files to completions
  const options: Completion[] = files
    .filter((file) => {
      if (!query) return true;
      const name = file.name.toLowerCase();
      const path = file.path.toLowerCase();
      return name.includes(query) || path.includes(query);
    })
    .map((file) => {
      // For markdown files, remove .md extension for display
      const displayPath = file.path.endsWith(".md")
        ? file.path.replace(/\.md$/, "")
        : file.path;

      // Determine icon/type based on file type
      const type = file.isMedia ? "variable" : "keyword";

      // Build the full link as the apply string
      const prefix = isEmbed ? "![[" : "[[";
      const replacement = `${prefix}${displayPath}]]`;

      return {
        label: displayPath,
        detail: file.isMedia ? "media" : "note",
        type,
        apply: replacement,
      };
    })
    .slice(0, 30); // Limit to 30 suggestions

  if (options.length === 0) {
    return null;
  }

  return {
    from: before.from,
    options,
    // This is important: it tells CodeMirror that partial matches should stay open
    filter: false,
  };
}

/**
 * Check if cursor is inside an unclosed wiki link on the current line
 */
function isInsideWikiLink(state: EditorState): boolean {
  const pos = state.selection.main.head;
  const line = state.doc.lineAt(pos);
  const textBefore = state.doc.sliceString(line.from, pos);

  // Find last [[ before cursor
  const lastOpen = textBefore.lastIndexOf("[[");
  if (lastOpen === -1) return false;

  // Check if there's a ]] after the [[ but before cursor
  const afterOpen = textBefore.slice(lastOpen);
  return !afterOpen.includes("]]");
}

/**
 * ViewPlugin that detects when cursor is inside [[ to trigger completion
 */
const wikiLinkTrigger = ViewPlugin.fromClass(
  class {
    constructor() {
      // Prefetch files when editor initializes
      prefetchFiles();
      prefetchNotes();
    }

    update(update: ViewUpdate) {
      // Trigger completion on any document change or selection change
      if (!update.docChanged && !update.selectionSet) return;

      // Check if we're inside an unclosed wiki link
      if (isInsideWikiLink(update.state)) {
        // Prefetch files if needed
        prefetchFiles();
        // Delay slightly to allow the document to update
        setTimeout(() => {
          startCompletion(update.view);
        }, 10);
      }
    }
  }
);

/**
 * Extension that provides wiki-link autocomplete
 */
export function wikiLinkCompletion() {
  return [
    // Configure tooltips to use fixed positioning so they're not clipped by overflow:hidden
    tooltips({
      position: "fixed",
    }),
    autocompletion({
      activateOnTyping: true, // Activate on any typing - completion source filters to [[
      maxRenderedOptions: 30,
      icons: false,
      override: [wikiLinkCompletionSource],
      defaultKeymap: true,
    }),
    // Use highest precedence so Enter accepts completion instead of inserting newline
    Prec.highest(keymap.of(completionKeymap)),
    wikiLinkTrigger,
  ];
}

/**
 * Invalidate the notes cache (call when notes are added/deleted)
 */
export function invalidateNotesCache() {
  notesCacheTimestamp = 0;
  filesCacheTimestamp = 0;
}
