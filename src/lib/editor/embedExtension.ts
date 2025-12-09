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
import {
  isImageFile,
  isAudioFile,
  isVideoFile,
  isPdfFile,
  getAudioMimeType,
} from "../utils/fileTypes";

// Pattern for embeds: ![[target]] or ![[target#section]] or ![[target|size]]
// Size can be: 200, 200px, 50%, x100 (height only), 200x100
const EMBED_PATTERN = /!\[\[([^\]#|]+)(?:#([^\]|]+))?(?:\|([^\]]+))?\]\]/g;

/**
 * Parse size parameter from embed syntax
 * Supports: 200, 200px, 50%, x100 (height only), 200x100
 */
interface ImageSize {
  width?: string;
  height?: string;
}

function parseImageSize(sizeStr: string | undefined): ImageSize | undefined {
  if (!sizeStr) return undefined;

  const trimmed = sizeStr.trim();
  if (!trimmed) return undefined;

  // Check for WIDTHxHEIGHT format (e.g., "200x100")
  const dimensionMatch = trimmed.match(/^(\d+)x(\d+)$/);
  if (dimensionMatch) {
    return {
      width: `${dimensionMatch[1]}px`,
      height: `${dimensionMatch[2]}px`,
    };
  }

  // Check for xHEIGHT format (e.g., "x100" for height only)
  const heightOnlyMatch = trimmed.match(/^x(\d+)(px)?$/);
  if (heightOnlyMatch) {
    return {
      height: `${heightOnlyMatch[1]}px`,
    };
  }

  // Check for percentage (e.g., "50%")
  if (trimmed.endsWith("%")) {
    return { width: trimmed };
  }

  // Check for pixel value (e.g., "200" or "200px")
  const pixelMatch = trimmed.match(/^(\d+)(px)?$/);
  if (pixelMatch) {
    return { width: `${pixelMatch[1]}px` };
  }

  return undefined;
}

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
    private depth: number,
    private size: ImageSize | undefined
  ) {
    super();
  }

  eq(other: EmbedWidget): boolean {
    return this.target === other.target &&
           this.section === other.section &&
           this.size?.width === other.size?.width &&
           this.size?.height === other.size?.height;
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
        // It's a media file - determine type from target filename
        if (isImageFile(this.target)) {
          this.renderImage(container, content);
        } else if (isAudioFile(this.target)) {
          this.renderAudio(container, content);
        } else if (isVideoFile(this.target)) {
          this.renderVideo(container, content);
        } else if (isPdfFile(this.target)) {
          this.renderPdf(container, content);
        } else {
          // Unknown media type, try as image
          this.renderImage(container, content);
        }
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

    // Apply custom size if specified
    if (this.size) {
      if (this.size.width) {
        img.style.width = this.size.width;
      }
      if (this.size.height) {
        img.style.height = this.size.height;
      }
      // If only one dimension is set, maintain aspect ratio
      if (this.size.width && !this.size.height) {
        img.style.height = "auto";
      }
      if (this.size.height && !this.size.width) {
        img.style.width = "auto";
      }
    }

    img.onerror = () => {
      container.innerHTML = "";
      this.renderError(container, `Failed to load image: ${this.target}`);
    };

    container.appendChild(img);
  }

  private renderAudio(container: HTMLElement, content: EmbedContent): void {
    container.classList.add("cm-embed-audio");

    const wrapper = document.createElement("div");
    wrapper.className = "cm-embed-audio-wrapper";

    const filename = document.createElement("div");
    filename.className = "cm-embed-audio-filename";
    filename.textContent = this.target.split("/").pop() || this.target;
    wrapper.appendChild(filename);

    // Get the asset URL directly - skip blob URL which has WebKit issues
    const assetUrl = convertFileSrc(content.assetUrl!);

    // Use <audio> element with source for better codec hints
    const audio = document.createElement("audio");
    audio.controls = true;
    audio.preload = "metadata";
    audio.style.width = "100%";
    audio.style.maxWidth = "400px";

    // Add source with type hint
    const source = document.createElement("source");
    source.src = assetUrl;
    source.type = getAudioMimeType(this.target);
    audio.appendChild(source);

    // Prevent CodeMirror from capturing clicks on the controls
    audio.addEventListener("mousedown", (e) => e.stopPropagation());
    audio.addEventListener("click", (e) => e.stopPropagation());

    audio.onerror = () => {
      const error = audio.error;
      console.error("[EmbedExtension] Audio playback error:", error?.code, error?.message);
      // Show unsupported format message
      const notice = document.createElement("div");
      notice.className = "cm-embed-audio-notice";
      notice.textContent = "Audio format not supported";
      notice.style.cssText = "font-size: 11px; color: var(--text-muted); margin-top: 4px;";
      wrapper.appendChild(notice);
    };

    wrapper.appendChild(audio);
    container.appendChild(wrapper);
  }

  private renderVideo(container: HTMLElement, content: EmbedContent): void {
    container.classList.add("cm-embed-video");

    const video = document.createElement("video");
    video.controls = true;
    video.preload = "auto"; // Load full file for duration detection

    // Use asset:// protocol
    const videoUrl = convertFileSrc(content.assetUrl!);
    video.src = videoUrl;

    // Prevent CodeMirror from capturing clicks on the video controls
    video.addEventListener("mousedown", (e) => e.stopPropagation());
    video.addEventListener("click", (e) => e.stopPropagation());

    video.onerror = () => {
      console.error("[EmbedExtension] Video load error for:", videoUrl);
      container.innerHTML = "";
      this.renderError(container, `Failed to load video: ${this.target}`);
    };

    container.appendChild(video);
  }

  private renderPdf(container: HTMLElement, content: EmbedContent): void {
    container.classList.add("cm-embed-pdf");

    const wrapper = document.createElement("div");
    wrapper.className = "cm-embed-pdf-wrapper";

    const filename = document.createElement("div");
    filename.className = "cm-embed-pdf-filename";
    filename.textContent = this.target.split("/").pop() || this.target;
    wrapper.appendChild(filename);

    const iframe = document.createElement("iframe");
    iframe.src = convertFileSrc(content.assetUrl!);
    iframe.title = this.target;

    // Prevent CodeMirror from capturing clicks on the PDF
    iframe.addEventListener("mousedown", (e) => e.stopPropagation());
    iframe.addEventListener("click", (e) => e.stopPropagation());

    wrapper.appendChild(iframe);

    container.appendChild(wrapper);
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
          const sizeParam = match[3];
          const size = parseImageSize(sizeParam);

          // Create widget decoration (inline, not block)
          const widget = new EmbedWidget(target, section, depth, size);
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
