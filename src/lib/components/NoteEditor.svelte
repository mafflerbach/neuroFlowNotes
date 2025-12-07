<script lang="ts">
  import { onDestroy } from "svelte";
  import { EditorState } from "@codemirror/state";
  import { EditorView, keymap, lineNumbers, highlightActiveLine, highlightActiveLineGutter } from "@codemirror/view";
  import { defaultKeymap, history, historyKeymap } from "@codemirror/commands";
  import { markdown, markdownLanguage } from "@codemirror/lang-markdown";
  import { languages } from "@codemirror/language-data";
  import { defaultHighlightStyle, syntaxHighlighting } from "@codemirror/language";
  import { searchKeymap, highlightSelectionMatches } from "@codemirror/search";
  import { editorStore } from "../stores";
  import { wikiLinkCompletion, livePreview, markdownHighlight } from "../editor";

  interface Props {
    readonly?: boolean;
  }

  let { readonly = false }: Props = $props();

  let editorContainer: HTMLDivElement | undefined = $state();
  let view: EditorView | null = null;
  let currentNoteId: number | null = null;

  // Create a custom theme
  const editorTheme = EditorView.theme({
    "&": {
      height: "100%",
      fontSize: "var(--font-size-md)",
      color: "var(--text-primary)",
    },
    ".cm-content": {
      fontFamily: "var(--font-family-mono)",
      padding: "var(--spacing-4) 0",
      color: "var(--text-primary)",
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
    },
    "&.cm-focused .cm-selectionBackground, .cm-selectionBackground": {
      background: "var(--editor-selection-bg)",
    },
  });

  // Update listener to sync changes to store
  const updateListener = EditorView.updateListener.of((update) => {
    if (update.docChanged) {
      const content = update.state.doc.toString();
      editorStore.updateContent(content);
    }
  });

  // Save on Cmd/Ctrl+S
  const saveKeymap = keymap.of([
    {
      key: "Mod-s",
      run: () => {
        editorStore.save();
        return true;
      },
    },
  ]);

  function createEditor(content: string) {
    if (!editorContainer) return;

    if (view) {
      view.destroy();
    }

    const extensions = [
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
      keymap.of([...defaultKeymap, ...historyKeymap, ...searchKeymap]),
      saveKeymap,
      updateListener,
      editorTheme,
      EditorView.lineWrapping,
      // Wiki-link autocomplete ([[)
      wikiLinkCompletion(),
      // Live preview (hide markdown syntax on inactive lines)
      livePreview(),
    ];

    // Add readonly extension if readonly prop is true
    if (readonly) {
      extensions.push(EditorState.readOnly.of(true));
    }

    const state = EditorState.create({
      doc: content,
      extensions,
    });

    view = new EditorView({
      state,
      parent: editorContainer,
    });
  }

  // Create/destroy editor when note ID changes (not content)
  $effect(() => {
    const noteIdFromStore = editorStore.currentNote?.id;
    if (noteIdFromStore && editorContainer && noteIdFromStore !== currentNoteId) {
      currentNoteId = noteIdFromStore;
      createEditor(editorStore.currentNote!.content);
    } else if (!noteIdFromStore && view) {
      // Note closed, destroy editor
      view.destroy();
      view = null;
      currentNoteId = null;
    }
  });

  onDestroy(() => {
    if (view) {
      view.destroy();
      view = null;
    }
  });
</script>

<div class="note-editor">
  {#if editorStore.isLoading}
    <div class="editor-loading">Loading...</div>
  {:else if editorStore.error}
    <div class="editor-error">{editorStore.error}</div>
  {:else if editorStore.currentNote}
    <div class="editor-header">
      <span class="editor-path">{editorStore.currentNote.path}</span>
      {#if editorStore.isDirty}
        <span class="dirty-indicator">●</span>
      {/if}
    </div>
    <div class="editor-content" bind:this={editorContainer}></div>
  {:else}
    <div class="editor-empty">
      <p>Select a note to start editing</p>
      <p class="hint">Or create a new note from the sidebar</p>
    </div>
  {/if}
</div>

<style>
  .note-editor {
    display: flex;
    flex-direction: column;
    height: 100%;
    background: var(--editor-bg);
  }

  .editor-header {
    display: flex;
    align-items: center;
    gap: var(--spacing-2);
    padding: var(--spacing-2) var(--spacing-4);
    background: var(--editor-header-bg);
    border-bottom: 1px solid var(--border-default);
    font-size: var(--font-size-base);
  }

  .editor-path {
    color: var(--text-muted);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .dirty-indicator {
    color: var(--color-primary);
    font-size: var(--font-size-xs);
  }

  .editor-content {
    flex: 1;
    overflow: hidden;
    position: relative;
  }

  .editor-content :global(.cm-editor) {
    height: 100%;
  }

  /* Ensure autocomplete tooltip is visible */
  .editor-content :global(.cm-tooltip) {
    z-index: var(--z-dropdown);
  }

  .editor-content :global(.cm-tooltip-autocomplete) {
    z-index: var(--z-dropdown);
  }

  .editor-loading,
  .editor-error,
  .editor-empty {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: 100%;
    color: var(--text-muted);
  }

  .editor-error {
    color: var(--color-error);
  }

  .editor-empty p {
    margin: var(--spacing-1) 0;
  }

  .editor-empty .hint {
    font-size: var(--font-size-base);
    opacity: 0.7;
  }
</style>
