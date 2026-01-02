/**
 * Frontmatter Conversion Extension for CodeMirror
 *
 * Detects when the user completes a frontmatter block (types closing `---`)
 * and offers to convert it to DB properties.
 */

import { EditorView, ViewPlugin } from "@codemirror/view";
import type { ViewUpdate } from "@codemirror/view";
import { convertFrontmatterToDb } from "../services/api";
import { workspaceStore } from "../stores/workspace.svelte";

// Pattern to detect frontmatter: starts with ---, ends with ---
const FRONTMATTER_START = /^---\s*$/;
const FRONTMATTER_END = /^---\s*$/;

interface FrontmatterBlock {
  startLine: number;
  endLine: number;
  from: number;
  to: number;
}

/**
 * Check if document has a complete frontmatter block
 */
function findFrontmatterBlock(doc: string): FrontmatterBlock | null {
  const lines = doc.split("\n");

  // Must start with ---
  if (lines.length < 2 || !FRONTMATTER_START.test(lines[0])) {
    return null;
  }

  // Find closing ---
  for (let i = 1; i < lines.length; i++) {
    if (FRONTMATTER_END.test(lines[i])) {
      // Calculate byte positions
      let from = 0;
      let to = 0;
      for (let j = 0; j <= i; j++) {
        if (j === 0) {
          from = 0;
        }
        to += lines[j].length + 1; // +1 for newline
      }
      return {
        startLine: 0,
        endLine: i,
        from,
        to: Math.min(to, doc.length),
      };
    }
  }

  return null;
}

/**
 * Check if the user just typed the closing --- of a frontmatter block
 */
function detectFrontmatterClose(update: ViewUpdate): boolean {
  if (!update.docChanged) return false;

  const doc = update.state.doc.toString();

  // Check if document starts with ---
  if (!doc.startsWith("---")) return false;

  // Check each change to see if they added "---" that closes frontmatter
  let addedClosingDelimiter = false;

  update.changes.iterChanges((fromA, toA, fromB, toB, inserted) => {
    const insertedText = inserted.toString();

    // Check if this change added a "-" that could complete "---"
    if (insertedText.includes("-")) {
      // Get the line where the change occurred
      const lineNumber = update.state.doc.lineAt(fromB).number;

      // Skip if this is the first line (opening delimiter)
      if (lineNumber === 1) return;

      // Get the full line after the change
      const line = update.state.doc.line(lineNumber);
      const lineText = line.text;

      // Check if this line is now "---"
      if (FRONTMATTER_END.test(lineText)) {
        // Verify there's content between the delimiters
        const linesAbove = update.state.doc.sliceString(0, line.from);
        const aboveLines = linesAbove.split("\n");

        // Must have opening --- and at least one content line
        if (
          aboveLines.length >= 2 &&
          FRONTMATTER_START.test(aboveLines[0]) &&
          aboveLines.slice(1).some((l) => l.trim().length > 0)
        ) {
          addedClosingDelimiter = true;
        }
      }
    }
  });

  return addedClosingDelimiter;
}

/**
 * Show confirmation dialog and handle conversion
 */
async function handleFrontmatterConversion(view: EditorView): Promise<void> {
  const activeDoc = workspaceStore.activeDoc;
  if (!activeDoc) {
    console.warn("[FrontmatterConversion] No active document");
    return;
  }

  const content = view.state.doc.toString();
  const block = findFrontmatterBlock(content);

  if (!block) {
    console.warn("[FrontmatterConversion] No frontmatter block found");
    return;
  }

  // Show confirmation dialog
  const shouldConvert = confirm(
    "Frontmatter detected. Convert to database properties?\n\n" +
      "- Properties will be stored in the database\n" +
      "- Tags will become inline #tags\n" +
      "- Frontmatter will be removed from the file"
  );

  if (!shouldConvert) {
    return;
  }

  try {
    const response = await convertFrontmatterToDb(activeDoc.id, content);

    // Replace the editor content with the converted content
    view.dispatch({
      changes: {
        from: 0,
        to: view.state.doc.length,
        insert: response.content,
      },
    });

    // Show success message
    const message =
      `Converted ${response.properties_converted} properties` +
      (response.tags_converted.length > 0
        ? ` and ${response.tags_converted.length} tags`
        : "");
    console.log("[FrontmatterConversion]", message);
  } catch (error) {
    console.error("[FrontmatterConversion] Failed to convert:", error);
    alert("Failed to convert frontmatter: " + String(error));
  }
}

/**
 * ViewPlugin that watches for frontmatter completion
 */
const frontmatterConversionPlugin = ViewPlugin.fromClass(
  class {
    private pendingConversion = false;

    update(update: ViewUpdate) {
      // Avoid triggering multiple times
      if (this.pendingConversion) return;

      if (detectFrontmatterClose(update)) {
        this.pendingConversion = true;

        // Defer the conversion to let the editor update settle
        setTimeout(() => {
          handleFrontmatterConversion(update.view).finally(() => {
            this.pendingConversion = false;
          });
        }, 100);
      }
    }
  }
);

/**
 * Extension that detects frontmatter completion and offers conversion
 */
export function frontmatterConversionExtension() {
  return frontmatterConversionPlugin;
}
