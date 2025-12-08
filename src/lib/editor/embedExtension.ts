/**
 * Embed Extension for CodeMirror
 * Renders embedded content (![[note]] or ![[note#section]] or ![[image.png]])
 * as inline widgets on non-active lines.
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
import { convertFileSrc } from "@tauri-apps/api/core";
import { resolveEmbed } from "../services/api";
import type { EmbedContent } from "../types";

// Pattern for embeds: ![[target]] or ![[target#section]]
const EMBED_PATTERN = /!\[\[([^\]#|]+)(?:#([^\]|]+))?(?:\|[^\]]+)?\]\]/g;

/**
 * Cache for resolved embeds to avoid repeated API calls
 */
const embedCache = new Map<string, { content: EmbedContent; timestamp: number }>();
const CACHE_TTL = 30000; // 30 seconds

function getCacheKey(target: string, section?: string): string {
  return section ? `${target}#${section}` : target;
}

async function resolveEmbedCached(target: string, section?: string, depth: number = 0): Promise<EmbedContent> {
  const key = getCacheKey(target, section);
  const cached = embedCache.get(key);

  if (cached && Date.now() - cached.timestamp < CACHE_TTL) {
    return cached.content;
  }

  const content = await resolveEmbed({ target, section, depth });
  embedCache.set(key, { content, timestamp: Date.now() });
  return content;
}

/**
 * Widget that displays embedded content as an inline widget
 */
class EmbedWidget extends WidgetType {
  constructor(
    private target: string,
    private section: string | undefined,
    private depth: number
  ) {
    super();
  }

  eq(other: EmbedWidget): boolean {
    return this.target === other.target && this.section === other.section;
  }

  toDOM(_view: EditorView): HTMLElement {
    const wrapper = document.createElement("span");
    wrapper.className = "cm-embed-wrapper";

    const container = document.createElement("div");
    container.className = "cm-embed-container cm-embed-loading";
    container.dataset.target = this.target;
    if (this.section) {
      container.dataset.section = this.section;
    }

    // Show loading state
    const loading = document.createElement("div");
    loading.className = "cm-embed-loading-text";
    loading.textContent = `Loading ${this.target}${this.section ? `#${this.section}` : ""}...`;
    container.appendChild(loading);

    wrapper.appendChild(container);

    // Resolve the embed asynchronously
    this.resolveAndRender(container);

    return wrapper;
  }

  private async resolveAndRender(container: HTMLElement): Promise<void> {
    try {
      const content = await resolveEmbedCached(this.target, this.section, this.depth);

      // Clear loading state
      container.innerHTML = "";
      container.classList.remove("cm-embed-loading");

      if (content.error) {
        this.renderError(container, content.error);
      } else if (content.isImage && content.assetUrl) {
        this.renderImage(container, content);
      } else if (content.content) {
        this.renderNote(container, content);
      } else {
        this.renderError(container, "No content found");
      }
    } catch (error) {
      container.innerHTML = "";
      container.classList.remove("cm-embed-loading");
      this.renderError(container, error instanceof Error ? error.message : "Failed to load embed");
    }
  }

  private renderError(container: HTMLElement, message: string): void {
    container.classList.add("cm-embed-error");

    const errorDiv = document.createElement("div");
    errorDiv.className = "cm-embed-error-content";
    errorDiv.textContent = message;
    container.appendChild(errorDiv);
  }

  private renderImage(container: HTMLElement, content: EmbedContent): void {
    container.classList.add("cm-embed-image");

    const img = document.createElement("img");
    // Convert the file path to a Tauri asset URL
    img.src = convertFileSrc(content.assetUrl!);
    img.alt = this.target;
    img.loading = "lazy";
    img.onerror = () => {
      container.innerHTML = "";
      this.renderError(container, `Failed to load image: ${this.target}`);
    };

    container.appendChild(img);
  }

  private renderNote(container: HTMLElement, content: EmbedContent): void {
    container.classList.add("cm-embed-note");

    // Header with link to source
    const header = document.createElement("div");
    header.className = "cm-embed-header";

    const link = document.createElement("a");
    link.className = "cm-embed-link";
    link.href = "#";
    link.textContent = this.section ? `${content.path}#${this.section}` : content.path;
    link.dataset.notePath = content.path;
    if (this.section) {
      link.dataset.section = this.section;
    }
    header.appendChild(link);
    container.appendChild(header);

    // Content (rendered as plain text for now - could add markdown rendering later)
    const contentDiv = document.createElement("div");
    contentDiv.className = "cm-embed-content";

    // Simple markdown-to-text rendering (basic)
    const renderedContent = this.renderMarkdownContent(content.content!);
    contentDiv.innerHTML = renderedContent;

    container.appendChild(contentDiv);
  }

  private renderMarkdownContent(markdown: string): string {
    // Basic markdown rendering for embedded content
    // This is intentionally simple - complex rendering would need a proper markdown parser
    let html = markdown
      // Escape HTML
      .replace(/&/g, "&amp;")
      .replace(/</g, "&lt;")
      .replace(/>/g, "&gt;")
      // Headers
      .replace(/^### (.+)$/gm, "<h4>$1</h4>")
      .replace(/^## (.+)$/gm, "<h3>$1</h3>")
      .replace(/^# (.+)$/gm, "<h2>$1</h2>")
      // Bold
      .replace(/\*\*(.+?)\*\*/g, "<strong>$1</strong>")
      .replace(/__(.+?)__/g, "<strong>$1</strong>")
      // Italic
      .replace(/\*([^*]+)\*/g, "<em>$1</em>")
      .replace(/_([^_]+)_/g, "<em>$1</em>")
      // Code
      .replace(/`([^`]+)`/g, "<code>$1</code>")
      // Wiki links (just show as links)
      .replace(/\[\[([^\]|]+)(?:\|([^\]]+))?\]\]/g, (_, target, display) =>
        `<span class="cm-embed-wikilink">${display || target}</span>`
      )
      // Line breaks
      .replace(/\n\n/g, "</p><p>")
      .replace(/\n/g, "<br>");

    return `<p>${html}</p>`;
  }

  ignoreEvent(): boolean {
    return false;
  }
}

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
 * Create decorations for embed patterns - uses inline replace decorations
 */
function createDecorations(view: EditorView, depth: number = 0): DecorationSet {
  const builder = new RangeSetBuilder<Decoration>();
  const activeLines = getActiveLines(view.state);
  const doc = view.state.doc;

  // Process each visible line
  for (const { from, to } of view.visibleRanges) {
    let pos = from;
    while (pos <= to) {
      const line = doc.lineAt(pos);
      const lineNum = line.number;
      const lineText = line.text;

      // Only process non-active lines
      if (!activeLines.has(lineNum)) {
        // Find embed patterns in the line
        let match;
        EMBED_PATTERN.lastIndex = 0;
        while ((match = EMBED_PATTERN.exec(lineText)) !== null) {
          const target = match[1];
          const section = match[2];

          // Create widget decoration (inline, not block)
          const widget = new EmbedWidget(target, section, depth);
          const deco = Decoration.replace({
            widget,
            inclusive: false,
          });

          builder.add(line.from + match.index, line.from + match.index + match[0].length, deco);
        }
      }

      pos = line.to + 1;
    }
  }

  return builder.finish();
}

/**
 * ViewPlugin that manages embed decorations
 */
const embedPlugin = ViewPlugin.fromClass(
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
 * Extension that provides embed rendering
 */
export function embedExtension() {
  return [embedPlugin];
}

/**
 * Invalidate the embed cache (call when notes are updated)
 */
export function invalidateEmbedCache(target?: string): void {
  if (target) {
    // Invalidate specific target
    for (const key of embedCache.keys()) {
      if (key === target || key.startsWith(`${target}#`)) {
        embedCache.delete(key);
      }
    }
  } else {
    // Invalidate all
    embedCache.clear();
  }
}
