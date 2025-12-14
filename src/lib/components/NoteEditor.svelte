<script lang="ts">
  import { onDestroy } from "svelte";
  import { EditorState } from "@codemirror/state";
  import { EditorView } from "@codemirror/view";
  import { editorStore, workspaceStore } from "../stores";
  import { createEditorExtensions, createSaveKeymap } from "../editor";

  interface Props {
    readonly?: boolean;
  }

  let { readonly = false }: Props = $props();

  let editorContainer: HTMLDivElement | undefined = $state();
  let view: EditorView | null = null;
  let currentNoteId: number | null = null;
  let lastVimMode: boolean | null = null;

  /**
   * Convert heading text to slug (must match backend slugify function)
   */
  function slugify(text: string): string {
    return text
      .toLowerCase()
      .replace(/[^\w\s-]/g, "") // Remove non-word chars except spaces and dashes
      .replace(/[\s_]+/g, "-") // Replace spaces/underscores with dashes
      .replace(/-+/g, "-") // Collapse multiple dashes
      .replace(/^-|-$/g, ""); // Trim dashes from ends
  }

  /**
   * Find the line number of a heading by its slug
   */
  function findHeadingLine(content: string, targetSlug: string): number | null {
    const lines = content.split("\n");
    const headingPattern = /^(#{1,6})\s+(.+)$/;

    for (let i = 0; i < lines.length; i++) {
      const match = lines[i].match(headingPattern);
      if (match) {
        const headingText = match[2].trim();
        const slug = slugify(headingText);
        if (slug === targetSlug) {
          return i;
        }
      }
    }
    return null;
  }

  /**
   * Scroll to a section by its slug
   */
  function scrollToSection(slug: string) {
    if (!view || !editorStore.currentNote) return;

    const lineNumber = findHeadingLine(editorStore.currentNote.content, slug);
    if (lineNumber === null) {
      console.warn(`[NoteEditor] Section not found: ${slug}`);
      return;
    }

    // Get the position at the start of the line
    const line = view.state.doc.line(lineNumber + 1); // CodeMirror lines are 1-indexed
    const pos = line.from;

    // Scroll the heading into view and place cursor there
    view.dispatch({
      selection: { anchor: pos },
      effects: EditorView.scrollIntoView(pos, { y: "start", yMargin: 50 }),
    });

    // Focus the editor
    view.focus();
  }

  // Update listener to sync changes to store
  const updateListener = EditorView.updateListener.of((update) => {
    if (update.docChanged) {
      const content = update.state.doc.toString();
      editorStore.updateContent(content);
    }
  });

  // Save keymap using store's save method
  const saveKeymap = createSaveKeymap(() => editorStore.save());

  function createEditor(content: string) {
    if (!editorContainer) return;

    if (view) {
      view.destroy();
    }

    const extensions = createEditorExtensions({
      updateListener,
      saveKeymap,
      readonly,
    });

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

  // Handle pending section scroll (when navigating to [[note#section]])
  $effect(() => {
    const pending = workspaceStore.pendingScroll;
    if (pending && currentNoteId === pending.noteId && view) {
      // Small delay to ensure editor is fully rendered
      requestAnimationFrame(() => {
        scrollToSection(pending.section);
        workspaceStore.clearPendingScroll();
      });
    }
  });

  // Recreate editor when vim mode changes
  $effect(() => {
    const vimModeEnabled = workspaceStore.vimMode;
    // Only recreate if vim mode actually changed and we have an open note
    if (lastVimMode !== null && vimModeEnabled !== lastVimMode && view && editorStore.currentNote && currentNoteId) {
      // Recreate the editor with the new vim mode setting
      createEditor(editorStore.currentNote.content);
    }
    lastVimMode = vimModeEnabled;
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
