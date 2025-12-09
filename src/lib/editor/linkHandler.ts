/**
 * Link Handler Extension for CodeMirror
 * Handles Ctrl/Cmd+Click on wiki links to navigate to notes
 * Supports section anchors: [[note#section]]
 */

import { EditorView } from "@codemirror/view";
import { workspaceStore } from "../stores/workspace.svelte";
import { listNotes, saveNote } from "../services/api";

// Pattern to match wiki links: [[target]] or [[target#section]] or [[target|display]]
const WIKILINK_PATTERN = /\[\[([^\]#|]+)(?:#([^\]|]+))?(?:\|[^\]]+)?\]\]/g;

/**
 * Create a new note in the root directory and navigate to it
 */
async function createAndNavigateToNote(target: string): Promise<void> {
  // Ensure the path ends with .md and is in the root
  const filename = target.endsWith(".md") ? target : `${target}.md`;
  // Remove any leading slashes or path components - always create in root
  const cleanFilename = filename.split("/").pop() || filename;

  // Create empty note with just a heading
  const title = cleanFilename.replace(".md", "");
  const initialContent = `# ${title}\n\n`;

  try {
    const noteId = await saveNote(cleanFilename, initialContent);

    // Navigate to the newly created note
    workspaceStore.followLink({
      path: cleanFilename,
      id: noteId,
      title: title,
    });
  } catch (error) {
    console.error("[LinkHandler] Failed to create note:", error);
  }
}

/**
 * Find the wiki link at a given position in a line
 */
function findWikiLinkAtPosition(
  lineText: string,
  posInLine: number
): { target: string; section?: string } | null {
  let match;
  WIKILINK_PATTERN.lastIndex = 0;

  while ((match = WIKILINK_PATTERN.exec(lineText)) !== null) {
    const start = match.index;
    const end = match.index + match[0].length;

    if (posInLine >= start && posInLine <= end) {
      return {
        target: match[1],
        section: match[2] || undefined,
      };
    }
  }

  return null;
}

/**
 * Navigate to a note by its name or path
 */
async function navigateToNote(target: string, _section?: string): Promise<void> {
  // TODO: _section parameter will be used to scroll to section anchors
  try {
    const notes = await listNotes();

    // Try to find the note by various matching strategies
    const targetLower = target.toLowerCase();
    const targetPath = target.endsWith(".md") ? target : `${target}.md`;

    const note = notes.find((n) => {
      // Exact path match
      if (n.path === targetPath) return true;
      if (n.path === target) return true;

      // Filename match (for notes in subdirectories)
      const filename = n.path.split("/").pop()?.replace(".md", "");
      if (filename?.toLowerCase() === targetLower) return true;

      // Title match
      if (n.title?.toLowerCase() === targetLower) return true;

      return false;
    });

    if (note) {
      // Follow the link - transitions to doc-finder mode (State C)
      workspaceStore.followLink({
        path: note.path,
        id: note.id,
        title: note.title ?? note.path.replace(".md", ""),
      });
    } else {
      // Note doesn't exist - create it in the root directory
      await createAndNavigateToNote(target);
    }
  } catch (error) {
    console.error("[LinkHandler] Failed to navigate:", error);
  }
}

/**
 * DOM event handler for link clicks
 */
const linkClickHandler = EditorView.domEventHandlers({
  click(event: MouseEvent, view: EditorView): boolean {
    // Only handle Ctrl+Click (Windows/Linux) or Cmd+Click (Mac)
    const isMac = /Mac|iPod|iPhone|iPad/.test(navigator.platform);
    const isModifierHeld = isMac ? event.metaKey : event.ctrlKey;

    if (!isModifierHeld) {
      return false; // Let other handlers process the click
    }

    // Get the position at the click
    const pos = view.posAtCoords({ x: event.clientX, y: event.clientY });
    if (pos === null) {
      return false;
    }

    // Get the line at the click position
    const line = view.state.doc.lineAt(pos);
    const posInLine = pos - line.from;

    // Check if clicking on a wiki link
    const link = findWikiLinkAtPosition(line.text, posInLine);
    if (link) {
      event.preventDefault();
      navigateToNote(link.target, link.section);
      return true;
    }

    return false;
  },
});

/**
 * Cursor style for hoverable links
 * Changes cursor to pointer when hovering over wiki links with modifier held
 */
const linkCursorStyle = EditorView.theme({
  ".cm-content": {
    // Cursor changes are handled via CSS in the main theme
  },
});

/**
 * Extension that provides link click handling
 */
export function linkHandlerExtension() {
  return [linkClickHandler, linkCursorStyle];
}

/**
 * Check if a position is inside a wiki link (for external use)
 */
export function isPositionInWikiLink(lineText: string, posInLine: number): boolean {
  return findWikiLinkAtPosition(lineText, posInLine) !== null;
}
