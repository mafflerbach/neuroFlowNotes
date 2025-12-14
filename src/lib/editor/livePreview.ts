/**
 * Live Preview extension for CodeMirror
 * Hides markdown syntax on non-active lines while preserving styling
 * Inspired by Obsidian's live preview mode
 */

import {
  EditorView,
  ViewPlugin,
  Decoration,
  WidgetType,
} from "@codemirror/view";
import type { ViewUpdate, DecorationSet } from "@codemirror/view";
import { RangeSetBuilder } from "@codemirror/state";
import type { EditorState } from "@codemirror/state";

/**
 * GTD context icons - map context names to emoji/icons
 */
const GTD_CONTEXT_ICONS: Record<string, string> = {
  phone: "📞",
  call: "📞",
  home: "🏠",
  work: "💼",
  office: "🏢",
  computer: "💻",
  pc: "💻",
  email: "📧",
  errands: "🚗",
  shopping: "🛒",
  waiting: "⏳",
  someday: "🌟",
  read: "📖",
  write: "✍️",
  meeting: "👥",
  agenda: "📋",
  focus: "🎯",
  quick: "⚡",
};

/**
 * Priority indicators
 */
const PRIORITY_STYLES: Record<string, { color: string; icon: string }> = {
  high: { color: "var(--color-error)", icon: "🔴" },
  medium: { color: "var(--color-warning)", icon: "🟡" },
  low: { color: "var(--text-muted)", icon: "🔵" },
};

/**
 * Widget for rendering checkboxes with GTD context icons
 */
class CheckboxWidget extends WidgetType {
  constructor(
    private checked: boolean,
    private context?: string,
    private priority?: string,
    private dueDate?: string
  ) {
    super();
  }

  toDOM() {
    const container = document.createElement("span");
    container.className = "cm-checkbox-widget";

    // Create the checkbox itself
    const checkbox = document.createElement("span");
    checkbox.className = this.checked
      ? "cm-checkbox cm-checkbox-checked"
      : "cm-checkbox cm-checkbox-unchecked";
    checkbox.textContent = this.checked ? "☑" : "☐";
    container.appendChild(checkbox);

    // Add context icon if present
    if (this.context) {
      const icon = GTD_CONTEXT_ICONS[this.context.toLowerCase()];
      if (icon) {
        const contextSpan = document.createElement("span");
        contextSpan.className = "cm-checkbox-context";
        contextSpan.textContent = icon;
        contextSpan.title = `@${this.context}`;
        container.appendChild(contextSpan);
      }
    }

    // Add priority indicator
    if (this.priority && PRIORITY_STYLES[this.priority.toLowerCase()]) {
      const priorityInfo = PRIORITY_STYLES[this.priority.toLowerCase()];
      const prioritySpan = document.createElement("span");
      prioritySpan.className = "cm-checkbox-priority";
      prioritySpan.textContent = priorityInfo.icon;
      prioritySpan.title = `!${this.priority}`;
      container.appendChild(prioritySpan);
    }

    // Add due date indicator if present
    if (this.dueDate) {
      const dueDateSpan = document.createElement("span");
      dueDateSpan.className = "cm-checkbox-due";
      dueDateSpan.textContent = "📅";
      dueDateSpan.title = this.dueDate;
      container.appendChild(dueDateSpan);
    }

    return container;
  }

  eq(other: CheckboxWidget) {
    return (
      this.checked === other.checked &&
      this.context === other.context &&
      this.priority === other.priority &&
      this.dueDate === other.dueDate
    );
  }

  ignoreEvent() {
    return false;
  }
}

/**
 * Widget for rendering list bullets
 */
class ListBulletWidget extends WidgetType {
  constructor(
    private level: number,
    private isOrdered: boolean,
    private number?: number
  ) {
    super();
  }

  toDOM() {
    const span = document.createElement("span");
    span.className = "cm-list-bullet-widget";
    span.setAttribute("data-level", String(this.level));

    if (this.isOrdered) {
      // Ordered list: show number with period
      span.textContent = `${this.number}. `;
      span.classList.add("cm-list-ordered");
    } else {
      // Unordered list: use different bullets per nesting level
      const bullets = ["•", "◦", "▪", "▫"];
      span.textContent = bullets[this.level % bullets.length] + " ";
      span.classList.add("cm-list-unordered");
    }

    return span;
  }

  eq(other: ListBulletWidget) {
    return (
      this.level === other.level &&
      this.isOrdered === other.isOrdered &&
      this.number === other.number
    );
  }

  ignoreEvent() {
    return false;
  }
}

/**
 * Patterns for markdown syntax to hide
 */
const HEADING_PATTERN = /^(#{1,6})\s/;
const BLOCKQUOTE_PATTERN = /^(>\s*)+/;
const LIST_MARKER_PATTERN = /^(\s*)([*+-]|\d+\.)\s/;
const CHECKBOX_PATTERN = /^(\s*)([*+-])\s\[([ xX])\]\s/;

/**
 * GTD annotation patterns
 */
const CONTEXT_PATTERN = /@([a-zA-Z][a-zA-Z0-9_-]*)/;
const PRIORITY_PATTERN = /!([a-zA-Z]+)/;
const DUE_DATE_PATTERN = /\^(\d{4}-\d{2}-\d{2})/;

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
  const activeLines = getActiveLines(view.state);
  const doc = view.state.doc;

  // Collect all decorations first, then sort and add to builder
  const allDecorations: DecorationRange[] = [];

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
        allDecorations.push(...decorations);
      }

      pos = line.to + 1;
    }
  }

  // Sort all decorations by position before adding to builder
  allDecorations.sort((a, b) => a.from - b.from || a.to - b.to);

  // Build the decoration set
  const builder = new RangeSetBuilder<Decoration>();
  for (const deco of allDecorations) {
    builder.add(deco.from, deco.to, deco.decoration);
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

  // Checkbox pattern: replace with styled checkbox widget including GTD icons
  const checkboxMatch = lineText.match(CHECKBOX_PATTERN);
  if (checkboxMatch) {
    const indentLength = checkboxMatch[1].length;
    const checked = checkboxMatch[3].toLowerCase() === "x";

    // Extract GTD annotations from the rest of the line
    const restOfLine = lineText.slice(checkboxMatch[0].length);
    const contextMatch = restOfLine.match(CONTEXT_PATTERN);
    const priorityMatch = restOfLine.match(PRIORITY_PATTERN);
    const dueDateMatch = restOfLine.match(DUE_DATE_PATTERN);

    const context = contextMatch ? contextMatch[1] : undefined;
    const priority = priorityMatch ? priorityMatch[1] : undefined;
    const dueDate = dueDateMatch ? dueDateMatch[1] : undefined;

    // Replace the bullet marker and checkbox brackets with our widget
    // "- [ ] " -> CheckboxWidget
    const fullMatchLength =
      checkboxMatch[1].length + // indentation
      checkboxMatch[2].length + // bullet marker
      1 + // space before [
      3 + // [ ] or [x]
      1; // space after ]

    decorations.push({
      from: lineStart + indentLength,
      to: lineStart + fullMatchLength,
      decoration: Decoration.replace({
        widget: new CheckboxWidget(checked, context, priority, dueDate),
      }),
    });

    // Hide GTD annotation syntax (but keep the description visible)
    // Hide @context
    if (contextMatch) {
      const contextStart =
        lineStart + checkboxMatch[0].length + (restOfLine.indexOf("@" + contextMatch[1]));
      decorations.push({
        from: contextStart,
        to: contextStart + contextMatch[0].length,
        decoration: hideDecoration,
      });
    }

    // Hide !priority
    if (priorityMatch) {
      const priorityStart =
        lineStart + checkboxMatch[0].length + (restOfLine.indexOf("!" + priorityMatch[1]));
      decorations.push({
        from: priorityStart,
        to: priorityStart + priorityMatch[0].length,
        decoration: hideDecoration,
      });
    }

    // Hide ^due-date
    if (dueDateMatch) {
      const dueDateStart =
        lineStart + checkboxMatch[0].length + (restOfLine.indexOf("^" + dueDateMatch[1]));
      decorations.push({
        from: dueDateStart,
        to: dueDateStart + dueDateMatch[0].length,
        decoration: hideDecoration,
      });
    }
  } else {
    // List markers: replace with styled bullets/numbers
    const listMatch = lineText.match(LIST_MARKER_PATTERN);
    if (listMatch && !blockquoteMatch) {
      const indentLength = listMatch[1].length;
      const markerLength = listMatch[2].length + 1; // +1 for space
      const marker = listMatch[2];

      // Calculate nesting level based on indentation (2 spaces per level)
      const level = Math.floor(indentLength / 2);

      // Determine if ordered or unordered
      const isOrdered = /^\d+\.$/.test(marker);
      const number = isOrdered ? parseInt(marker) : undefined;

      // Replace marker with styled widget
      decorations.push({
        from: lineStart + indentLength,
        to: lineStart + indentLength + markerLength,
        decoration: Decoration.replace({
          widget: new ListBulletWidget(level, isOrdered, number),
        }),
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
