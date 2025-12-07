/**
 * Wiki-link autocomplete extension for CodeMirror
 * Triggers on [[ and shows available notes from the vault
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
import { listNotes } from "../services/api";
import type { NoteListItem } from "../types";

// Cache for notes list
let notesCache: NoteListItem[] = [];
let cacheTimestamp = 0;
const CACHE_TTL = 10000; // 10 seconds
let fetchPromise: Promise<NoteListItem[]> | null = null;

/**
 * Prefetch notes in the background
 */
function prefetchNotes(): void {
  const now = Date.now();
  if (now - cacheTimestamp < CACHE_TTL) return;
  if (fetchPromise) return; // Already fetching

  fetchPromise = listNotes()
    .then((notes) => {
      notesCache = notes;
      cacheTimestamp = Date.now();
      console.log("[WikiLink] Prefetched", notes.length, "notes");
      return notes;
    })
    .catch((error) => {
      console.error("Failed to fetch notes for autocomplete:", error);
      return notesCache;
    })
    .finally(() => {
      fetchPromise = null;
    });
}

/**
 * Get cached notes (synchronous)
 */
function getCachedNotes(): NoteListItem[] {
  // Trigger prefetch if cache is stale
  prefetchNotes();
  return notesCache;
}

/**
 * Completion source for wiki-links
 * Triggers when user types [[
 */
function wikiLinkCompletionSource(
  context: CompletionContext
): CompletionResult | null {
  // Look for [[ pattern before cursor
  const before = context.matchBefore(/\[\[[^\]]*$/);

  console.log("[WikiLink] Checking completion, explicit:", context.explicit, "before:", before?.text);

  // If no [[ pattern, skip
  if (!before) {
    return null;
  }

  // Get the text after [[
  const query = before.text.slice(2).toLowerCase();

  console.log("[WikiLink] Found pattern, query:", query);

  // Get cached notes (synchronous)
  const notes = getCachedNotes();
  console.log("[WikiLink] Got cached notes:", notes.length);

  // Filter and map notes to completions
  const options: Completion[] = notes
    .filter((note) => {
      if (!query) return true;
      const title = (note.title || "").toLowerCase();
      const path = note.path.toLowerCase();
      return title.includes(query) || path.includes(query);
    })
    .map((note) => {
      // Remove .md extension for display
      const displayPath = note.path.replace(/\.md$/, "");

      return {
        label: `[[${displayPath}]]`,
        type: "keyword",
      };
    })
    .slice(0, 20); // Limit to 20 suggestions

  if (options.length === 0) {
    console.log("[WikiLink] No options found");
    return null;
  }

  console.log("[WikiLink] Returning", options.length, "options, from:", before.from, "to:", context.pos);
  console.log("[WikiLink] First option:", options[0]);

  return {
    from: before.from,
    options,
  };
}

/**
 * ViewPlugin that detects [[ and triggers completion
 */
const wikiLinkTrigger = ViewPlugin.fromClass(
  class {
    constructor() {
      // Prefetch notes when editor initializes
      prefetchNotes();
    }

    update(update: ViewUpdate) {
      // Check if user just typed something
      if (!update.docChanged) return;

      // Check each change
      update.changes.iterChanges((_fromA, _toA, fromB, _toB, inserted) => {
        const insertedText = inserted.toString();
        // If user just typed '[' and the character before is also '['
        if (insertedText === "[") {
          const pos = fromB;
          if (pos > 0) {
            const charBefore = update.state.doc.sliceString(pos - 1, pos);
            if (charBefore === "[") {
              console.log("[WikiLink] Detected [[ trigger, starting completion");
              // Prefetch notes if needed
              prefetchNotes();
              // Delay slightly to allow the document to update
              setTimeout(() => {
                startCompletion(update.view);
              }, 10);
            }
          }
        }
      });
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
      activateOnTyping: false, // We trigger manually on [[
      maxRenderedOptions: 20,
      icons: false,
      override: [wikiLinkCompletionSource],
    }),
    keymap.of(completionKeymap),
    wikiLinkTrigger,
  ];
}

/**
 * Invalidate the notes cache (call when notes are added/deleted)
 */
export function invalidateNotesCache() {
  cacheTimestamp = 0;
}
