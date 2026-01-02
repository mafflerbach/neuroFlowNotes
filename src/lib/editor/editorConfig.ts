/**
 * Shared CodeMirror editor configuration
 * Provides common theme and extensions for all editor instances
 */

import { EditorState, type Extension } from "@codemirror/state";
import {
  EditorView,
  keymap,
  lineNumbers,
  highlightActiveLine,
  highlightActiveLineGutter,
} from "@codemirror/view";
import {
  defaultKeymap,
  history,
  historyKeymap,
  indentWithTab,
} from "@codemirror/commands";
import { markdown, markdownLanguage } from "@codemirror/lang-markdown";
import { languages } from "@codemirror/language-data";
import {
  defaultHighlightStyle,
  syntaxHighlighting,
} from "@codemirror/language";
import { searchKeymap, highlightSelectionMatches } from "@codemirror/search";

import { wikiLinkCompletion } from "./wikiLinkCompletion";
import { livePreview } from "./livePreview";
import { markdownHighlight } from "./markdownHighlight";
import { embedExtension } from "./embedExtension";
import { linkHandlerExtension } from "./linkHandler";
import { hoverPreviewExtension } from "./hoverPreview";
import { pasteHandlerExtension } from "./pasteHandler";
import { vimExtension } from "./vimExtension";
import { calloutExtension } from "./calloutExtension";
import { queryEmbedExtension } from "./queryEmbedExtension";
import { habitTrackerExtension } from "./habitTrackerExtension";
import { frontmatterConversionExtension } from "./frontmatterConversion";

/**
 * Shared editor theme using CSS variables for theming support
 */
export const editorTheme = EditorView.theme({
  "&": {
    height: "100%",
    fontSize: "var(--font-size-md)",
    color: "var(--text-primary)",
  },
  ".cm-content": {
    fontFamily: "var(--font-family-mono)",
    padding: "var(--spacing-4) 0",
    color: "var(--text-primary)",
    caretColor: "var(--editor-cursor)",
  },
  ".cm-line": {
    color: "var(--text-primary)",
    padding: "0 var(--spacing-4)",
  },
  ".cm-gutters": {
    background: "var(--editor-gutter-bg)",
    border: "none",
    color: "var(--editor-gutter-color)",
  },
  ".cm-activeLineGutter": {
    background: "var(--editor-active-gutter-bg)",
  },
  ".cm-activeLine": {
    background: "var(--editor-active-line-bg)",
  },
  "&.cm-focused .cm-cursor": {
    borderLeftColor: "var(--editor-cursor)",
    borderLeftWidth: "2px",
  },
  "&.cm-focused .cm-selectionBackground, .cm-selectionBackground": {
    background: "var(--editor-selection-bg)",
  },
});

/**
 * Create a save keymap that calls the provided save handler on Cmd/Ctrl+S
 */
export function createSaveKeymap(onSave: () => void): Extension {
  return keymap.of([
    {
      key: "Mod-s",
      run: () => {
        onSave();
        return true;
      },
    },
  ]);
}

/**
 * Options for creating editor extensions
 */
export interface EditorExtensionOptions {
  /** Update listener called when document changes */
  updateListener: Extension;
  /** Save keymap extension */
  saveKeymap: Extension;
  /** Whether the editor is read-only */
  readonly?: boolean;
}

/**
 * Create the standard set of editor extensions
 * Each editor instance can provide its own update listener and save handler
 */
export function createEditorExtensions(
  options: EditorExtensionOptions
): Extension[] {
  const extensions: Extension[] = [
    // Core editing features
    lineNumbers(),
    highlightActiveLineGutter(),
    highlightActiveLine(),
    history(),
    highlightSelectionMatches(),

    // Markdown with code block language support for 100+ languages
    markdown({
      base: markdownLanguage,
      codeLanguages: languages,
    }),

    // Custom markdown highlighting with theme-aware colors
    markdownHighlight(),

    // Default syntax highlighting as fallback for any missed tokens
    syntaxHighlighting(defaultHighlightStyle, { fallback: true }),

    // Keymaps
    keymap.of([
      indentWithTab,
      ...defaultKeymap,
      ...historyKeymap,
      ...searchKeymap,
    ]),
    options.saveKeymap,

    // Document change listener
    options.updateListener,

    // Theme
    editorTheme,

    // Line wrapping
    EditorView.lineWrapping,

    // Custom extensions
    wikiLinkCompletion(),
    livePreview(),
    embedExtension(),
    linkHandlerExtension(),
    hoverPreviewExtension(),
    pasteHandlerExtension(),

    // Vim keybindings (if enabled in settings)
    ...vimExtension(),

    // Callout blocks (> [!note], > [!warning], etc.)
    calloutExtension(),

    // Query embeds (```query``` blocks with live results)
    queryEmbedExtension(),

    // Habit tracker embeds (```habit-tracker``` blocks with interactive tables)
    habitTrackerExtension(),

    // Frontmatter detection and conversion to DB properties
    frontmatterConversionExtension(),
  ];

  // Add readonly extension if needed
  if (options.readonly) {
    extensions.push(EditorState.readOnly.of(true));
  }

  return extensions;
}
