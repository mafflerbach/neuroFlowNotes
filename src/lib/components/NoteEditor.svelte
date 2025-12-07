<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { EditorState } from "@codemirror/state";
  import { EditorView, keymap, lineNumbers, highlightActiveLine, highlightActiveLineGutter } from "@codemirror/view";
  import { defaultKeymap, history, historyKeymap } from "@codemirror/commands";
  import { markdown } from "@codemirror/lang-markdown";
  import { syntaxHighlighting, defaultHighlightStyle } from "@codemirror/language";
  import { searchKeymap, highlightSelectionMatches } from "@codemirror/search";
  import { editorStore } from "../stores";

  interface Props {
    noteId?: number;
    readonly?: boolean;
    onLinkClick?: (path: string) => void;
  }

  let { noteId, readonly = false, onLinkClick }: Props = $props();

  let editorContainer: HTMLDivElement | undefined = $state();
  let view: EditorView | null = null;
  let currentNoteId: number | null = null;

  // Create a custom theme
  const editorTheme = EditorView.theme({
    "&": {
      height: "100%",
      fontSize: "14px",
    },
    ".cm-content": {
      fontFamily: "'SF Mono', 'Fira Code', 'Consolas', monospace",
      padding: "16px 0",
    },
    ".cm-line": {
      padding: "0 16px",
    },
    ".cm-gutters": {
      background: "var(--editor-gutter-bg, #f5f5f5)",
      border: "none",
      color: "var(--editor-gutter-color, #999)",
    },
    ".cm-activeLineGutter": {
      background: "var(--editor-active-gutter-bg, #e8e8e8)",
    },
    ".cm-activeLine": {
      background: "var(--editor-active-line-bg, #f8f8f8)",
    },
    "&.cm-focused .cm-cursor": {
      borderLeftColor: "var(--primary-color, #3b5998)",
    },
    "&.cm-focused .cm-selectionBackground, .cm-selectionBackground": {
      background: "var(--editor-selection-bg, #c8daf8)",
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
      markdown(),
      syntaxHighlighting(defaultHighlightStyle),
      keymap.of([...defaultKeymap, ...historyKeymap, ...searchKeymap]),
      saveKeymap,
      updateListener,
      editorTheme,
      EditorView.lineWrapping,
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
    background: var(--editor-bg, #fff);
  }

  .editor-header {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 8px 16px;
    background: var(--editor-header-bg, #fafafa);
    border-bottom: 1px solid var(--border-color, #e0e0e0);
    font-size: 13px;
  }

  .editor-path {
    color: var(--text-muted, #666);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .dirty-indicator {
    color: var(--primary-color, #3b5998);
    font-size: 10px;
  }

  .editor-content {
    flex: 1;
    overflow: hidden;
  }

  .editor-content :global(.cm-editor) {
    height: 100%;
  }

  .editor-loading,
  .editor-error,
  .editor-empty {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: 100%;
    color: var(--text-muted, #666);
  }

  .editor-error {
    color: var(--error-color, #d32f2f);
  }

  .editor-empty p {
    margin: 4px 0;
  }

  .editor-empty .hint {
    font-size: 13px;
    opacity: 0.7;
  }
</style>
