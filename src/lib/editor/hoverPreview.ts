/**
 * Hover Preview Extension for CodeMirror
 * Shows a preview tooltip when hovering over wiki links
 * Displays note content preview or image thumbnail
 */

import { EditorView, hoverTooltip } from "@codemirror/view";
import type { Tooltip } from "@codemirror/view";
import { convertFileSrc } from "@tauri-apps/api/core";
import { resolveEmbed } from "../services/api";
import type { EmbedContent } from "../types";

// Pattern to match wiki links: [[target]] or [[target#section]] or [[target|display]]
// Also matches non-embed links (without !)
const WIKILINK_PATTERN = /(?<!!)\[\[([^\]#|]+)(?:#([^\]|]+))?(?:\|[^\]]+)?\]\]/g;

// Cache for hover previews
const previewCache = new Map<string, { content: EmbedContent; timestamp: number }>();
const CACHE_TTL = 30000; // 30 seconds
const PREVIEW_MAX_LENGTH = 500; // Maximum characters to show in preview

function getCacheKey(target: string, section?: string): string {
  return section ? `${target}#${section}` : target;
}

async function getPreviewContent(target: string, section?: string): Promise<EmbedContent> {
  const key = getCacheKey(target, section);
  const cached = previewCache.get(key);

  if (cached && Date.now() - cached.timestamp < CACHE_TTL) {
    return cached.content;
  }

  const content = await resolveEmbed({ target, section, depth: 0 });
  previewCache.set(key, { content, timestamp: Date.now() });
  return content;
}

/**
 * Find wiki link at position
 */
function findWikiLinkAtPosition(
  lineText: string,
  posInLine: number
): { target: string; section?: string; start: number; end: number } | null {
  let match;
  WIKILINK_PATTERN.lastIndex = 0;

  while ((match = WIKILINK_PATTERN.exec(lineText)) !== null) {
    const start = match.index;
    const end = match.index + match[0].length;

    if (posInLine >= start && posInLine <= end) {
      return {
        target: match[1],
        section: match[2] || undefined,
        start,
        end,
      };
    }
  }

  return null;
}

/**
 * Truncate text to a maximum length with ellipsis
 */
function truncateText(text: string, maxLength: number): string {
  if (text.length <= maxLength) return text;

  // Try to cut at a word boundary
  const truncated = text.substring(0, maxLength);
  const lastSpace = truncated.lastIndexOf(" ");

  if (lastSpace > maxLength * 0.8) {
    return truncated.substring(0, lastSpace) + "...";
  }

  return truncated + "...";
}

/**
 * Strip markdown formatting from text for preview
 */
function stripMarkdown(text: string): string {
  return text
    // Remove headers
    .replace(/^#{1,6}\s+/gm, "")
    // Remove bold/italic markers
    .replace(/\*\*(.+?)\*\*/g, "$1")
    .replace(/__(.+?)__/g, "$1")
    .replace(/\*([^*]+)\*/g, "$1")
    .replace(/_([^_]+)_/g, "$1")
    // Remove strikethrough
    .replace(/~~(.+?)~~/g, "$1")
    // Remove code markers
    .replace(/`([^`]+)`/g, "$1")
    // Convert wiki links to plain text
    .replace(/\[\[([^\]|]+)(?:\|([^\]]+))?\]\]/g, (_, target, display) => display || target)
    // Remove markdown links
    .replace(/\[([^\]]+)\]\([^)]+\)/g, "$1")
    // Collapse multiple newlines
    .replace(/\n{3,}/g, "\n\n")
    .trim();
}

/**
 * Create the hover tooltip extension
 */
const hoverPreviewTooltip = hoverTooltip(
  async (view: EditorView, pos: number): Promise<Tooltip | null> => {
    const line = view.state.doc.lineAt(pos);
    const posInLine = pos - line.from;

    const link = findWikiLinkAtPosition(line.text, posInLine);
    if (!link) {
      return null;
    }

    // Don't show preview for the currently open note
    // This would require access to the current note path, which we don't have here
    // Could be added later via a facet

    return {
      pos: line.from + link.start,
      end: line.from + link.end,
      above: true,
      create(): { dom: HTMLElement } {
        const dom = document.createElement("div");
        dom.className = "cm-hover-preview";
        dom.innerHTML = '<div class="cm-hover-loading">Loading...</div>';

        // Load content asynchronously
        loadPreviewContent(dom, link.target, link.section);

        return { dom };
      },
    };
  },
  {
    hoverTime: 300, // Delay before showing preview (ms)
    hideOnChange: true,
  }
);

/**
 * Load and render preview content
 */
async function loadPreviewContent(
  container: HTMLElement,
  target: string,
  section?: string
): Promise<void> {
  try {
    const content = await getPreviewContent(target, section);

    if (content.error) {
      container.innerHTML = `<div class="cm-hover-error">${content.error}</div>`;
      return;
    }

    if (content.isImage && content.assetUrl) {
      // Convert the file path to a Tauri asset URL
      const imageUrl = convertFileSrc(content.assetUrl);
      container.innerHTML = `
        <div class="cm-hover-image">
          <img src="${imageUrl}" alt="${target}" loading="lazy" />
          <div class="cm-hover-image-name">${target}</div>
        </div>
      `;
      return;
    }

    if (content.content) {
      const strippedContent = stripMarkdown(content.content);
      const truncatedContent = truncateText(strippedContent, PREVIEW_MAX_LENGTH);

      container.innerHTML = `
        <div class="cm-hover-note">
          <div class="cm-hover-note-title">${content.path.replace(".md", "")}</div>
          <div class="cm-hover-note-content">${escapeHtml(truncatedContent)}</div>
        </div>
      `;
      return;
    }

    container.innerHTML = '<div class="cm-hover-empty">No content</div>';
  } catch (error) {
    container.innerHTML = `<div class="cm-hover-error">Failed to load preview</div>`;
  }
}

/**
 * Escape HTML for safe display
 */
function escapeHtml(text: string): string {
  return text
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;")
    .replace(/'/g, "&#039;")
    .replace(/\n/g, "<br>");
}

/**
 * Extension that provides hover preview for wiki links
 */
export function hoverPreviewExtension() {
  return [hoverPreviewTooltip];
}

/**
 * Invalidate the preview cache
 */
export function invalidatePreviewCache(target?: string): void {
  if (target) {
    for (const key of previewCache.keys()) {
      if (key === target || key.startsWith(`${target}#`)) {
        previewCache.delete(key);
      }
    }
  } else {
    previewCache.clear();
  }
}
