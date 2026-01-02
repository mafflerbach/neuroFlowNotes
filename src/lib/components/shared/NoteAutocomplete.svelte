<script lang="ts">
  import { FileText, X, Link } from "lucide-svelte";
  import type { NoteListItem } from "../../types";
  import { listNotes } from "../../services/api";

  interface Props {
    selectedNote: NoteListItem | null;
    onSelect: (note: NoteListItem | null) => void;
    placeholder?: string;
  }

  let {
    selectedNote,
    onSelect,
    placeholder = "Search notes...",
  }: Props = $props();

  let query = $state("");
  let isOpen = $state(false);
  let inputElement: HTMLInputElement | undefined = $state();
  let allNotes = $state<NoteListItem[]>([]);

  // Load notes when component mounts or dropdown opens
  async function loadNotes() {
    try {
      allNotes = await listNotes();
    } catch (e) {
      console.error("Failed to load notes:", e);
    }
  }

  // Filter notes based on query
  const filteredNotes = $derived.by(() => {
    if (!query.trim()) return [];
    const q = query.toLowerCase();
    return allNotes
      .filter(
        (note: NoteListItem) =>
          note.title?.toLowerCase().includes(q) ||
          note.path.toLowerCase().includes(q)
      )
      .slice(0, 10); // Limit to 10 results
  });

  function handleSelect(note: NoteListItem) {
    onSelect(note);
    query = "";
    isOpen = false;
  }

  function handleClear() {
    onSelect(null);
    query = "";
  }

  function handleInputFocus() {
    isOpen = true;
    // Load notes on first focus
    if (allNotes.length === 0) {
      loadNotes();
    }
  }

  function handleInputBlur() {
    // Delay close to allow click on dropdown items
    setTimeout(() => {
      isOpen = false;
    }, 150);
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Escape") {
      isOpen = false;
      inputElement?.blur();
    }
  }

  // Format path for display (remove .md extension)
  function formatPath(path: string): string {
    return path.replace(/\.md$/, "");
  }
</script>

<div class="note-autocomplete">
  {#if selectedNote}
    <div class="selected-note">
      <Link size={14} />
      <span class="note-title">{selectedNote.title || formatPath(selectedNote.path)}</span>
      <span class="note-path">{formatPath(selectedNote.path)}</span>
      <button type="button" class="clear-btn" onclick={handleClear} aria-label="Clear selection">
        <X size={14} />
      </button>
    </div>
  {:else}
    <div class="input-wrapper">
      <input
        bind:this={inputElement}
        type="text"
        bind:value={query}
        {placeholder}
        onfocus={handleInputFocus}
        onblur={handleInputBlur}
        onkeydown={handleKeydown}
        autocomplete="off"
        autocorrect="off"
        autocapitalize="off"
        spellcheck="false"
      />
      {#if isOpen && filteredNotes.length > 0}
        <div class="dropdown">
          {#each filteredNotes as note (note.id)}
            <button
              type="button"
              class="dropdown-item"
              onmousedown={() => handleSelect(note)}
            >
              <FileText size={14} />
              <div class="item-content">
                <span class="item-title">{note.title || formatPath(note.path)}</span>
                <span class="item-path">{formatPath(note.path)}</span>
              </div>
            </button>
          {/each}
        </div>
      {/if}
      {#if isOpen && query.trim() && filteredNotes.length === 0}
        <div class="dropdown">
          <div class="no-results">No notes found</div>
        </div>
      {/if}
    </div>
  {/if}
</div>

<style>
  .note-autocomplete {
    position: relative;
  }

  .input-wrapper {
    position: relative;
  }

  .input-wrapper input {
    width: 100%;
    padding: var(--spacing-3);
    border: 1px solid var(--input-border);
    border-radius: var(--radius-lg);
    font-size: var(--font-size-md);
    background: var(--input-bg);
    color: var(--input-text);
  }

  .input-wrapper input:focus {
    outline: none;
    border-color: var(--input-border-focus);
    box-shadow: var(--shadow-focus);
  }

  .dropdown {
    position: absolute;
    top: 100%;
    left: 0;
    right: 0;
    margin-top: var(--spacing-1);
    background: var(--bg-surface-raised);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-lg);
    box-shadow: var(--shadow-lg);
    z-index: 100;
    max-height: 200px;
    overflow-y: auto;
  }

  .dropdown-item {
    display: flex;
    align-items: center;
    gap: var(--spacing-2);
    width: 100%;
    padding: var(--spacing-2) var(--spacing-3);
    border: none;
    background: transparent;
    cursor: pointer;
    text-align: left;
    color: var(--text-primary);
  }

  .dropdown-item:hover {
    background: var(--bg-hover);
  }

  .item-content {
    display: flex;
    flex-direction: column;
    gap: 2px;
    min-width: 0;
    flex: 1;
  }

  .item-title {
    font-size: var(--font-size-sm);
    font-weight: var(--font-weight-medium);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .item-path {
    font-size: var(--font-size-xs);
    color: var(--text-muted);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .no-results {
    padding: var(--spacing-3);
    text-align: center;
    color: var(--text-muted);
    font-size: var(--font-size-sm);
  }

  .selected-note {
    display: flex;
    align-items: center;
    gap: var(--spacing-2);
    padding: var(--spacing-2) var(--spacing-3);
    background: var(--bg-surface-raised);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-lg);
    color: var(--text-primary);
  }

  .selected-note .note-title {
    font-size: var(--font-size-sm);
    font-weight: var(--font-weight-medium);
    flex: 1;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .selected-note .note-path {
    font-size: var(--font-size-xs);
    color: var(--text-muted);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    max-width: 150px;
  }

  .clear-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    padding: var(--spacing-1);
    border: none;
    background: transparent;
    border-radius: var(--radius-sm);
    color: var(--text-muted);
    cursor: pointer;
  }

  .clear-btn:hover {
    background: var(--bg-hover);
    color: var(--text-primary);
  }
</style>
