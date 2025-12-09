/**
 * Callout Extension for CodeMirror
 * Renders callout blocks (> [!type]) with styling and optional collapse
 *
 * Syntax:
 * > [!note] Title
 * > Content here
 *
 * > [!warning]- Collapsed by default
 * > Hidden content
 *
 * > [!info]+ Expanded by default (same as no modifier)
 * > Visible content
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

// Callout types with their icons and colors
const CALLOUT_TYPES: Record<string, { icon: string; color: string }> = {
  note: { icon: "📝", color: "var(--color-info)" },
  abstract: { icon: "📋", color: "var(--color-info)" },
  summary: { icon: "📋", color: "var(--color-info)" },
  tldr: { icon: "📋", color: "var(--color-info)" },
  info: { icon: "ℹ️", color: "var(--color-info)" },
  todo: { icon: "☑️", color: "var(--color-info)" },
  tip: { icon: "💡", color: "var(--color-success)" },
  hint: { icon: "💡", color: "var(--color-success)" },
  important: { icon: "🔥", color: "var(--color-success)" },
  success: { icon: "✅", color: "var(--color-success)" },
  check: { icon: "✅", color: "var(--color-success)" },
  done: { icon: "✅", color: "var(--color-success)" },
  question: { icon: "❓", color: "var(--color-warning)" },
  help: { icon: "❓", color: "var(--color-warning)" },
  faq: { icon: "❓", color: "var(--color-warning)" },
  warning: { icon: "⚠️", color: "var(--color-warning)" },
  caution: { icon: "⚠️", color: "var(--color-warning)" },
  attention: { icon: "⚠️", color: "var(--color-warning)" },
  failure: { icon: "❌", color: "var(--color-error)" },
  fail: { icon: "❌", color: "var(--color-error)" },
  missing: { icon: "❌", color: "var(--color-error)" },
  danger: { icon: "⛔", color: "var(--color-error)" },
  error: { icon: "🚫", color: "var(--color-error)" },
  bug: { icon: "🐛", color: "var(--color-error)" },
  example: { icon: "📎", color: "var(--md-wiki-link)" },
  quote: { icon: "💬", color: "var(--text-muted)" },
  cite: { icon: "💬", color: "var(--text-muted)" },
};

// Pattern to match callout header: > [!type] or > [!type]- or > [!type]+
const CALLOUT_HEADER_PATTERN = /^>\s*\[!(\w+)\]([-+])?\s*(.*)?$/;
// Pattern to match continued callout lines
const CALLOUT_CONTENT_PATTERN = /^>\s?(.*)$/;

interface CalloutBlock {
  type: string;
  title: string;
  collapsed: boolean;
  defaultCollapsed: boolean;
  startLine: number;
  endLine: number;
  headerLineFrom: number;
  headerLineTo: number;
  contentFrom: number;
  contentTo: number;
}

/**
 * Find callout blocks in the document
 */
function findCalloutBlocks(state: EditorState): CalloutBlock[] {
  const blocks: CalloutBlock[] = [];
  const doc = state.doc;
  let currentBlock: CalloutBlock | null = null;

  for (let i = 1; i <= doc.lines; i++) {
    const line = doc.line(i);
    const text = line.text;

    const headerMatch = text.match(CALLOUT_HEADER_PATTERN);
    if (headerMatch) {
      // Finish previous block if any
      if (currentBlock) {
        blocks.push(currentBlock);
      }

      const type = headerMatch[1].toLowerCase();
      const modifier = headerMatch[2]; // - or + or undefined
      const title = headerMatch[3]?.trim() || type.charAt(0).toUpperCase() + type.slice(1);

      currentBlock = {
        type,
        title,
        collapsed: modifier === "-",
        defaultCollapsed: modifier === "-",
        startLine: i,
        endLine: i,
        headerLineFrom: line.from,
        headerLineTo: line.to,
        contentFrom: line.to + 1,
        contentTo: line.to,
      };
    } else if (currentBlock && CALLOUT_CONTENT_PATTERN.test(text)) {
      // Continue the current block
      currentBlock.endLine = i;
      currentBlock.contentTo = line.to;
    } else if (currentBlock) {
      // End of callout block
      blocks.push(currentBlock);
      currentBlock = null;
    }
  }

  // Don't forget the last block
  if (currentBlock) {
    blocks.push(currentBlock);
  }

  return blocks;
}

/**
 * Track collapsed state for callouts
 */
const collapsedState = new Map<string, boolean>();

function getCalloutKey(block: CalloutBlock): string {
  return `${block.startLine}-${block.type}-${block.title}`;
}

function isCalloutCollapsed(block: CalloutBlock): boolean {
  const key = getCalloutKey(block);
  if (collapsedState.has(key)) {
    return collapsedState.get(key)!;
  }
  return block.defaultCollapsed;
}

function toggleCalloutCollapsed(block: CalloutBlock): void {
  const key = getCalloutKey(block);
  const current = isCalloutCollapsed(block);
  collapsedState.set(key, !current);
}

/**
 * Widget for callout header
 */
class CalloutHeaderWidget extends WidgetType {
  constructor(private block: CalloutBlock) {
    super();
  }

  eq(other: CalloutHeaderWidget): boolean {
    return (
      this.block.type === other.block.type &&
      this.block.title === other.block.title &&
      this.block.startLine === other.block.startLine
    );
  }

  toDOM(view: EditorView): HTMLElement {
    const calloutInfo = CALLOUT_TYPES[this.block.type] || CALLOUT_TYPES.note;
    const collapsed = isCalloutCollapsed(this.block);
    const hasContent = this.block.contentTo > this.block.headerLineTo;

    const wrapper = document.createElement("div");
    wrapper.className = `cm-callout-header cm-callout-${this.block.type}`;
    wrapper.style.borderLeftColor = calloutInfo.color;

    // Collapse toggle (only if there's content)
    if (hasContent) {
      const toggle = document.createElement("button");
      toggle.className = "cm-callout-toggle";
      toggle.textContent = collapsed ? "▶" : "▼";
      toggle.type = "button";
      toggle.setAttribute("aria-label", collapsed ? "Expand callout" : "Collapse callout");
      toggle.onmousedown = (e) => {
        // Prevent editor from getting focus/selection changes
        e.preventDefault();
        e.stopPropagation();
      };
      toggle.onclick = (e) => {
        e.preventDefault();
        e.stopPropagation();
        toggleCalloutCollapsed(this.block);
        // Force view update
        view.dispatch({ effects: [] });
      };
      wrapper.appendChild(toggle);
    }

    // Icon
    const icon = document.createElement("span");
    icon.className = "cm-callout-icon";
    icon.textContent = calloutInfo.icon;
    wrapper.appendChild(icon);

    // Title
    const title = document.createElement("span");
    title.className = "cm-callout-title";
    title.textContent = this.block.title;
    title.style.color = calloutInfo.color;
    wrapper.appendChild(title);

    return wrapper;
  }

  ignoreEvent(): boolean {
    return false;
  }
}

/**
 * Get active lines (where cursor is)
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

interface DecorationEntry {
  from: number;
  to: number;
  decoration: Decoration;
  // Line decorations should come before range decorations at the same position
  isLine: boolean;
}

/**
 * Create decorations for callout blocks
 */
function createDecorations(view: EditorView): DecorationSet {
  const blocks = findCalloutBlocks(view.state);
  const activeLines = getActiveLines(view.state);
  const allDecorations: DecorationEntry[] = [];

  for (const block of blocks) {
    // Check if cursor is in this callout
    let cursorInCallout = false;
    for (let line = block.startLine; line <= block.endLine; line++) {
      if (activeLines.has(line)) {
        cursorInCallout = true;
        break;
      }
    }

    // If cursor is in callout, show raw markdown
    if (cursorInCallout) {
      continue;
    }

    const collapsed = isCalloutCollapsed(block);

    // Add line class to header line for styling
    allDecorations.push({
      from: block.headerLineFrom,
      to: block.headerLineFrom,
      decoration: Decoration.line({
        class: `cm-callout-header-line cm-callout-${block.type}-header-line`,
      }),
      isLine: true,
    });

    // Replace header line content with styled widget
    const headerWidget = new CalloutHeaderWidget(block);
    allDecorations.push({
      from: block.headerLineFrom,
      to: block.headerLineTo,
      decoration: Decoration.replace({ widget: headerWidget }),
      isLine: false,
    });

    // If collapsed and has content, hide content lines
    if (collapsed && block.contentTo > block.headerLineTo) {
      // Add line decoration to hide content
      for (let lineNum = block.startLine + 1; lineNum <= block.endLine; lineNum++) {
        const line = view.state.doc.line(lineNum);
        allDecorations.push({
          from: line.from,
          to: line.to,
          decoration: Decoration.replace({}),
          isLine: false,
        });
      }
    } else if (block.contentTo > block.headerLineTo) {
      // Show content with callout styling - use line decorations for the border
      for (let lineNum = block.startLine + 1; lineNum <= block.endLine; lineNum++) {
        const line = view.state.doc.line(lineNum);
        const text = line.text;

        if (CALLOUT_CONTENT_PATTERN.test(text)) {
          // Add line class for the left border styling
          allDecorations.push({
            from: line.from,
            to: line.from,
            decoration: Decoration.line({
              class: `cm-callout-content-line cm-callout-${block.type}-line`,
            }),
            isLine: true,
          });

          // Hide the "> " prefix (livePreview might also do this, but that's OK)
          const prefixEnd = text.startsWith("> ") ? 2 : text.startsWith(">") ? 1 : 0;
          if (prefixEnd > 0) {
            allDecorations.push({
              from: line.from,
              to: line.from + prefixEnd,
              decoration: Decoration.replace({}),
              isLine: false,
            });
          }
        }
      }
    }
  }

  // Sort decorations: by from position, then line decorations before others, then by to position
  allDecorations.sort((a, b) => {
    if (a.from !== b.from) return a.from - b.from;
    // Line decorations have negative startSide, so they should come first
    if (a.isLine !== b.isLine) return a.isLine ? -1 : 1;
    return a.to - b.to;
  });

  // Build the decoration set
  const builder = new RangeSetBuilder<Decoration>();
  for (const deco of allDecorations) {
    builder.add(deco.from, deco.to, deco.decoration);
  }

  return builder.finish();
}

/**
 * ViewPlugin that manages callout decorations
 */
const calloutPlugin = ViewPlugin.fromClass(
  class {
    decorations: DecorationSet;

    constructor(view: EditorView) {
      this.decorations = createDecorations(view);
    }

    update(update: ViewUpdate) {
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

// Inject callout styles directly
const injectStyles = () => {
  if (typeof document === "undefined") return;
  if (document.getElementById("callout-extension-styles")) return;

  const style = document.createElement("style");
  style.id = "callout-extension-styles";
  style.textContent = `
    /* Header line - collapse to remove gap */
    div.cm-callout-header-line {
      margin: -1.3em 0;
      padding-left: 16px !important;
    }
    /* Header widget - no extra margin since parent has padding */
    .cm-callout-header {
      margin-left: 0;
    }
    /* Content lines */
    div.cm-callout-content-line {
      border-left: 4px solid var(--color-info);
      margin-left: 16px;
      padding-left: 12px !important;
    }
    div.cm-callout-note-line,
    div.cm-callout-info-line,
    div.cm-callout-abstract-line,
    div.cm-callout-summary-line,
    div.cm-callout-tldr-line,
    div.cm-callout-todo-line {
      border-left-color: var(--color-info);
    }
    div.cm-callout-tip-line,
    div.cm-callout-hint-line,
    div.cm-callout-important-line,
    div.cm-callout-success-line,
    div.cm-callout-check-line,
    div.cm-callout-done-line {
      border-left-color: var(--color-success);
    }
    div.cm-callout-warning-line,
    div.cm-callout-caution-line,
    div.cm-callout-attention-line,
    div.cm-callout-question-line,
    div.cm-callout-help-line,
    div.cm-callout-faq-line {
      border-left-color: var(--color-warning);
    }
    div.cm-callout-danger-line,
    div.cm-callout-error-line,
    div.cm-callout-failure-line,
    div.cm-callout-fail-line,
    div.cm-callout-missing-line,
    div.cm-callout-bug-line {
      border-left-color: var(--color-error);
    }
    div.cm-callout-example-line {
      border-left-color: var(--md-wiki-link);
    }
    div.cm-callout-quote-line,
    div.cm-callout-cite-line {
      border-left-color: var(--text-muted);
    }
  `;
  document.head.appendChild(style);
};

/**
 * Extension that provides callout rendering
 */
export function calloutExtension() {
  injectStyles();
  return [calloutPlugin];
}
