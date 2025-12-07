/**
 * Live Preview extension for CodeMirror
 * Hides markdown syntax on non-active lines while preserving styling
 * Inspired by Obsidian's live preview mode
 */

import {
  EditorView,
  ViewPlugin,
  Decoration,
} from "@codemirror/view";
import type { ViewUpdate, DecorationSet } from "@codemirror/view";
import { RangeSetBuilder } from "@codemirror/state";
import type { EditorState } from "@codemirror/state";

/**
 * Patterns for markdown syntax to hide
 */
const HEADING_PATTERN = /^(#{1,6})\s/;
const BLOCKQUOTE_PATTERN = /^(>\s*)+/;
const LIST_MARKER_PATTERN = /^(\s*)([*+-]|\d+\.)\s/;
const CHECKBOX_PATTERN = /^(\s*)([*+-])\s\[([ xX])\]\s/;

/**
 * Get the line numbers that contain any part of the selection
 */
function getActiveLines(state: EditorState): Set<number> {
  const activeLines = new Set<number>();
  for (const range of state.selection.ranges) {
    const startLine = state.doc.lineAt(range.from).number;
    const endLine = state.doc.lineAt(range.to).number;
    for (let line = startLine; line <= endLine; line++) {
      activeLines.add(line);
    }
  }
  return activeLines;
}

/**
 * Create decorations to hide markdown syntax on non-active lines
 */
function createDecorations(view: EditorView): DecorationSet {
  const builder = new RangeSetBuilder<Decoration>();
  const activeLines = getActiveLines(view.state);
  const doc = view.state.doc;

  // Process each visible line
  for (const { from, to } of view.visibleRanges) {
    let pos = from;
    while (pos < to) {
      const line = doc.lineAt(pos);
      const lineNum = line.number;
      const lineText = line.text;

      // Skip active lines - show all syntax
      if (!activeLines.has(lineNum)) {
        const decorations = getLineDecorations(line.from, lineText);
        for (const deco of decorations) {
          builder.add(deco.from, deco.to, deco.decoration);
        }
      }

      pos = line.to + 1;
    }
  }

  return builder.finish();
}

interface DecorationRange {
  from: number;
  to: number;
  decoration: Decoration;
}

/**
 * Get decorations for a single line to hide markdown syntax
 */
function getLineDecorations(
  lineStart: number,
  lineText: string
): DecorationRange[] {
  const decorations: DecorationRange[] = [];
  const hideDecoration = Decoration.replace({});

  // Headings: hide the # symbols
  const headingMatch = lineText.match(HEADING_PATTERN);
  if (headingMatch) {
    decorations.push({
      from: lineStart,
      to: lineStart + headingMatch[1].length + 1, // Include the space
      decoration: hideDecoration,
    });
    return decorations; // Headings typically don't have other inline formatting
  }

  // Blockquotes: hide the > symbols
  const blockquoteMatch = lineText.match(BLOCKQUOTE_PATTERN);
  if (blockquoteMatch) {
    decorations.push({
      from: lineStart,
      to: lineStart + blockquoteMatch[0].length,
      decoration: hideDecoration,
    });
  }

  // Checkbox pattern: hide marker but show checkbox state visually
  const checkboxMatch = lineText.match(CHECKBOX_PATTERN);
  if (checkboxMatch) {
    // Hide "- [ ] " but the checkbox CSS will style it
    const markerEnd = checkboxMatch[1].length + checkboxMatch[2].length + 1;
    decorations.push({
      from: lineStart,
      to: lineStart + markerEnd,
      decoration: hideDecoration,
    });
  } else {
    // List markers: hide the bullet/number
    const listMatch = lineText.match(LIST_MARKER_PATTERN);
    if (listMatch && !blockquoteMatch) {
      // Keep the indentation, hide only the marker and space
      const indentLength = listMatch[1].length;
      const markerLength = listMatch[2].length + 1; // +1 for space
      decorations.push({
        from: lineStart + indentLength,
        to: lineStart + indentLength + markerLength,
        decoration: hideDecoration,
      });
    }
  }

  // Process inline formatting
  // Note: We need to be careful about overlapping matches

  // Bold: hide ** or __
  let match;
  const boldRegex = /(\*\*|__)(.+?)(\*\*|__)/g;
  while ((match = boldRegex.exec(lineText)) !== null) {
    // Hide opening marker
    decorations.push({
      from: lineStart + match.index,
      to: lineStart + match.index + match[1].length,
      decoration: hideDecoration,
    });
    // Hide closing marker
    const closeStart = match.index + match[1].length + match[2].length;
    decorations.push({
      from: lineStart + closeStart,
      to: lineStart + closeStart + match[3].length,
      decoration: hideDecoration,
    });
  }

  // Italic: hide * or _ (but not ** or __)
  const italicRegex = /(?<!\*|\w)(\*|_)(?!\*|_)(.+?)(?<!\*|_)\1(?!\*|_|\w)/g;
  while ((match = italicRegex.exec(lineText)) !== null) {
    // Check it's not part of bold
    const before = lineText[match.index - 1] || "";
    const after = lineText[match.index + match[0].length] || "";
    if (before !== "*" && before !== "_" && after !== "*" && after !== "_") {
      decorations.push({
        from: lineStart + match.index,
        to: lineStart + match.index + 1,
        decoration: hideDecoration,
      });
      decorations.push({
        from: lineStart + match.index + match[0].length - 1,
        to: lineStart + match.index + match[0].length,
        decoration: hideDecoration,
      });
    }
  }

  // Strikethrough: hide ~~
  const strikeRegex = /(~~)(.+?)(~~)/g;
  while ((match = strikeRegex.exec(lineText)) !== null) {
    decorations.push({
      from: lineStart + match.index,
      to: lineStart + match.index + 2,
      decoration: hideDecoration,
    });
    decorations.push({
      from: lineStart + match.index + match[0].length - 2,
      to: lineStart + match.index + match[0].length,
      decoration: hideDecoration,
    });
  }

  // Inline code: hide backticks
  const codeRegex = /(`)([^`]+)(`)/g;
  while ((match = codeRegex.exec(lineText)) !== null) {
    decorations.push({
      from: lineStart + match.index,
      to: lineStart + match.index + 1,
      decoration: hideDecoration,
    });
    decorations.push({
      from: lineStart + match.index + match[0].length - 1,
      to: lineStart + match.index + match[0].length,
      decoration: hideDecoration,
    });
  }

  // Wiki links: transform [[link]] to show just the link text
  const wikiRegex = /\[\[([^\]|]+)(?:\|([^\]]+))?\]\]/g;
  while ((match = wikiRegex.exec(lineText)) !== null) {
    // Hide [[
    decorations.push({
      from: lineStart + match.index,
      to: lineStart + match.index + 2,
      decoration: hideDecoration,
    });
    // If there's an alias, hide the path and |
    if (match[2]) {
      decorations.push({
        from: lineStart + match.index + 2,
        to: lineStart + match.index + 2 + match[1].length + 1, // path + |
        decoration: hideDecoration,
      });
    }
    // Hide ]]
    decorations.push({
      from: lineStart + match.index + match[0].length - 2,
      to: lineStart + match.index + match[0].length,
      decoration: hideDecoration,
    });
  }

  // Markdown links: [text](url) - hide the URL part
  const linkRegex = /\[([^\]]+)\]\(([^)]+)\)/g;
  while ((match = linkRegex.exec(lineText)) !== null) {
    // Hide ]( and url)
    const textEnd = match.index + 1 + match[1].length;
    decorations.push({
      from: lineStart + textEnd,
      to: lineStart + match.index + match[0].length,
      decoration: hideDecoration,
    });
    // Hide opening [
    decorations.push({
      from: lineStart + match.index,
      to: lineStart + match.index + 1,
      decoration: hideDecoration,
    });
  }

  // Sort decorations by position
  decorations.sort((a, b) => a.from - b.from);

  return decorations;
}

/**
 * ViewPlugin that manages live preview decorations
 */
const livePreviewPlugin = ViewPlugin.fromClass(
  class {
    decorations: DecorationSet;

    constructor(view: EditorView) {
      this.decorations = createDecorations(view);
    }

    update(update: ViewUpdate) {
      // Rebuild decorations if document changed, selection changed, or viewport changed
      if (
        update.docChanged ||
        update.selectionSet ||
        update.viewportChanged
      ) {
        this.decorations = createDecorations(update.view);
      }
    }
  },
  {
    decorations: (v) => v.decorations,
  }
);

/**
 * Extension that provides live preview mode
 */
export function livePreview() {
  return [livePreviewPlugin];
}
