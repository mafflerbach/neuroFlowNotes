<script lang="ts">
  /**
   * SearchModal - Full-text and semantic search for notes
   * Opens with Cmd/Ctrl+K
   */
  import { Search, FileText, X, Sparkles, Type, Layers } from "lucide-svelte";
  import { searchNotes, hybridSearchNotes } from "../services/api";
  import { workspaceStore } from "../stores";
  import type { SearchResult, HybridSearchResult, EmbeddingSettings, MatchType } from "../types";
  import { DEFAULT_EMBEDDING_SETTINGS } from "../types";

  interface Props {
    open: boolean;
    onclose: () => void;
    embeddingSettings?: EmbeddingSettings;
  }

  let { open = $bindable(), onclose, embeddingSettings }: Props = $props();

  let query = $state("");
  let results = $state<(SearchResult | HybridSearchResult)[]>([]);
  let loading = $state(false);
  let selectedIndex = $state(0);
  let inputRef = $state<HTMLInputElement | null>(null);
  let debounceTimer: ReturnType<typeof setTimeout> | null = null;
  let useSemanticSearch = $state(false);

  // Check if semantic search is available
  let semanticAvailable = $derived(embeddingSettings?.enabled ?? false);

  // Focus input when modal opens
  $effect(() => {
    if (open) {
      query = "";
      results = [];
      selectedIndex = 0;
      setTimeout(() => inputRef?.focus(), 50);
    }
  });

  async function performSearch(searchQuery: string) {
    if (!searchQuery.trim()) {
      results = [];
      return;
    }

    loading = true;
    try {
      if (semanticAvailable && useSemanticSearch) {
        // Use hybrid search
        results = await hybridSearchNotes(searchQuery, {
          limit: 20,
          useSemantic: true,
          settings: embeddingSettings || DEFAULT_EMBEDDING_SETTINGS,
        });
      } else {
        // FTS5 query syntax: prefix match with *
        const ftsQuery = searchQuery
          .split(/\s+/)
          .filter(Boolean)
          .map((term) => `"${term}"*`)
          .join(" ");
        results = await searchNotes(ftsQuery, 20);
      }
      selectedIndex = 0;
    } catch (e) {
      console.error("Search failed:", e);
      results = [];
    } finally {
      loading = false;
    }
  }

  function handleInput() {
    if (debounceTimer) clearTimeout(debounceTimer);
    debounceTimer = setTimeout(() => {
      performSearch(query);
    }, 150);
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Escape") {
      e.preventDefault();
      onclose();
    } else if (e.key === "ArrowDown") {
      e.preventDefault();
      selectedIndex = Math.min(selectedIndex + 1, results.length - 1);
    } else if (e.key === "ArrowUp") {
      e.preventDefault();
      selectedIndex = Math.max(selectedIndex - 1, 0);
    } else if (e.key === "Enter" && results.length > 0) {
      e.preventDefault();
      selectResult(results[selectedIndex]);
    }
  }

  function selectResult(result: SearchResult | HybridSearchResult) {
    workspaceStore.openDoc({
      path: result.path,
      id: result.note_id,
      title: result.title || result.path.split("/").pop()?.replace(".md", "") || null,
    });
    onclose();
  }

  function isHybridResult(result: SearchResult | HybridSearchResult): result is HybridSearchResult {
    return "match_type" in result;
  }

  function getMatchTypeLabel(matchType: MatchType): string {
    switch (matchType) {
      case "Keyword": return "Keyword";
      case "Semantic": return "Semantic";
      case "Both": return "Both";
      default: return "";
    }
  }

  function getResultScore(result: SearchResult | HybridSearchResult): string {
    if (isHybridResult(result)) {
      return result.combined_score.toFixed(2);
    }
    return result.score.toFixed(1);
  }

  function getScoreTooltip(result: SearchResult | HybridSearchResult): string {
    if (isHybridResult(result)) {
      const parts: string[] = [`Combined: ${result.combined_score.toFixed(3)}`];
      if (result.fts_score !== null) {
        parts.push(`FTS: ${result.fts_score.toFixed(1)}`);
      }
      if (result.vector_score !== null) {
        parts.push(`Vector: ${result.vector_score.toFixed(3)}`);
      }
      return parts.join(" | ");
    }
    return `Score: ${result.score.toFixed(1)}`;
  }

  function toggleSemanticSearch() {
    useSemanticSearch = !useSemanticSearch;
    if (query.trim()) {
      performSearch(query);
    }
  }

  function handleBackdropClick(e: MouseEvent) {
    if (e.target === e.currentTarget) {
      onclose();
    }
  }

  function getFileName(path: string): string {
    return path.split("/").pop()?.replace(".md", "") || path;
  }

  function getDirectory(path: string): string {
    const parts = path.split("/");
    if (parts.length > 1) {
      return parts.slice(0, -1).join("/");
    }
    return "";
  }
</script>

{#if open}
  <!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
  <div class="search-backdrop" onclick={handleBackdropClick}>
    <div class="search-modal">
      <div class="search-input-wrapper">
        <Search size={18} class="search-icon" />
        <input
          bind:this={inputRef}
          type="text"
          class="search-input"
          placeholder={useSemanticSearch ? "Semantic search..." : "Search notes..."}
          bind:value={query}
          oninput={handleInput}
          onkeydown={handleKeydown}
          autocomplete="off"
          autocorrect="off"
          autocapitalize="off"
          spellcheck="false"
        />
        {#if query}
          <button class="clear-btn" onclick={() => { query = ""; results = []; }}>
            <X size={14} />
          </button>
        {/if}
        {#if semanticAvailable}
          <button
            class="semantic-toggle"
            class:active={useSemanticSearch}
            onclick={toggleSemanticSearch}
            title={useSemanticSearch ? "Disable semantic search" : "Enable semantic search"}
          >
            <Sparkles size={16} />
          </button>
        {/if}
      </div>

      <div class="search-results">
        {#if loading}
          <div class="search-loading">Searching...</div>
        {:else if query && results.length === 0}
          <div class="search-empty">No results found</div>
        {:else if results.length > 0}
          {#each results as result, i}
            <!-- svelte-ignore a11y_click_events_have_key_events a11y_no_noninteractive_element_interactions -->
            <div
              class="search-result"
              class:selected={i === selectedIndex}
              onclick={() => selectResult(result)}
              onmouseenter={() => (selectedIndex = i)}
              role="option"
              aria-selected={i === selectedIndex}
            >
              <FileText size={16} class="result-icon" />
              <div class="result-content">
                <div class="result-title-row">
                  <span class="result-title">
                    {result.title || getFileName(result.path)}
                  </span>
                  {#if isHybridResult(result)}
                    <span class="match-type-badge" class:keyword={result.match_type === "Keyword"} class:semantic={result.match_type === "Semantic"} class:both={result.match_type === "Both"}>
                      {#if result.match_type === "Keyword"}
                        <Type size={10} />
                      {:else if result.match_type === "Semantic"}
                        <Sparkles size={10} />
                      {:else}
                        <Layers size={10} />
                      {/if}
                      {getMatchTypeLabel(result.match_type)}
                    </span>
                  {/if}
                </div>
                {#if getDirectory(result.path)}
                  <div class="result-path">{getDirectory(result.path)}</div>
                {/if}
                {#if result.snippet}
                  <div class="result-snippet">{@html result.snippet}</div>
                {/if}
              </div>
              <div class="result-score" title={getScoreTooltip(result)}>
                {getResultScore(result)}
              </div>
            </div>
          {/each}
        {:else}
          <div class="search-hint">
            <p>Type to search across all notes</p>
            <div class="search-tips">
              <span class="tip"><kbd>Enter</kbd> to open</span>
              <span class="tip"><kbd>↑↓</kbd> to navigate</span>
              <span class="tip"><kbd>Esc</kbd> to close</span>
              {#if semanticAvailable}
                <span class="tip"><Sparkles size={12} /> Semantic available</span>
              {/if}
            </div>
          </div>
        {/if}
      </div>
    </div>
  </div>
{/if}

<style>
  .search-backdrop {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background: rgba(0, 0, 0, 0.5);
    display: flex;
    align-items: flex-start;
    justify-content: center;
    padding-top: 15vh;
    z-index: 1000;
  }

  .search-modal {
    width: 600px;
    max-width: 90vw;
    max-height: 70vh;
    background: var(--bg-surface);
    border-radius: var(--radius-lg);
    box-shadow: var(--shadow-lg);
    overflow: hidden;
    display: flex;
    flex-direction: column;
  }

  .search-input-wrapper {
    display: flex;
    align-items: center;
    gap: var(--spacing-2);
    padding: var(--spacing-3) var(--spacing-4);
    border-bottom: 1px solid var(--border-default);
  }

  .search-input-wrapper :global(.search-icon) {
    color: var(--text-muted);
    flex-shrink: 0;
  }

  .search-input {
    flex: 1;
    border: none;
    background: transparent;
    font-size: var(--font-size-lg);
    color: var(--text-primary);
    outline: none;
  }

  .search-input::placeholder {
    color: var(--text-muted);
  }

  .clear-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    padding: var(--spacing-1);
    background: transparent;
    border: none;
    color: var(--text-muted);
    cursor: pointer;
    border-radius: var(--radius-sm);
  }

  .clear-btn:hover {
    background: var(--bg-hover);
    color: var(--text-primary);
  }

  .search-results {
    flex: 1;
    overflow-y: auto;
    padding: var(--spacing-2);
  }

  .search-loading,
  .search-empty,
  .search-hint {
    padding: var(--spacing-4);
    text-align: center;
    color: var(--text-muted);
  }

  .search-tips {
    display: flex;
    gap: var(--spacing-4);
    justify-content: center;
    margin-top: var(--spacing-3);
    font-size: var(--font-size-sm);
  }

  .tip {
    display: flex;
    align-items: center;
    gap: var(--spacing-1);
  }

  kbd {
    padding: 2px 6px;
    background: var(--bg-tertiary);
    border-radius: var(--radius-sm);
    font-size: var(--font-size-xs);
    font-family: var(--font-mono);
  }

  .search-result {
    display: flex;
    align-items: flex-start;
    gap: var(--spacing-3);
    padding: var(--spacing-2) var(--spacing-3);
    border-radius: var(--radius-md);
    cursor: pointer;
  }

  .search-result:hover,
  .search-result.selected {
    background: var(--bg-hover);
  }

  .search-result.selected {
    background: var(--color-primary-light);
  }

  .search-result :global(.result-icon) {
    color: var(--text-muted);
    flex-shrink: 0;
    margin-top: 2px;
  }

  .result-content {
    flex: 1;
    min-width: 0;
  }

  .semantic-toggle {
    display: flex;
    align-items: center;
    justify-content: center;
    padding: var(--spacing-1);
    background: transparent;
    border: none;
    color: var(--text-muted);
    cursor: pointer;
    border-radius: var(--radius-sm);
    transition: all 0.15s ease;
  }

  .semantic-toggle:hover {
    background: var(--bg-hover);
    color: var(--text-primary);
  }

  .semantic-toggle.active {
    background: var(--color-primary-light);
    color: var(--color-primary);
  }

  .result-title-row {
    display: flex;
    align-items: center;
    gap: var(--spacing-2);
  }

  .result-title {
    font-weight: var(--font-weight-medium);
    color: var(--text-primary);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .match-type-badge {
    display: inline-flex;
    align-items: center;
    gap: 3px;
    padding: 2px 6px;
    border-radius: var(--radius-sm);
    font-size: var(--font-size-xs);
    flex-shrink: 0;
  }

  .match-type-badge.keyword {
    background: var(--blue-light);
    color: var(--blue);
  }

  .match-type-badge.semantic {
    background: var(--purple-light);
    color: var(--purple);
  }

  .match-type-badge.both {
    background: var(--green-light);
    color: var(--green);
  }

  .result-path {
    font-size: var(--font-size-sm);
    color: var(--text-muted);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .result-snippet {
    font-size: var(--font-size-sm);
    color: var(--text-secondary);
    margin-top: var(--spacing-1);
    overflow: hidden;
    text-overflow: ellipsis;
    display: -webkit-box;
    -webkit-line-clamp: 2;
    -webkit-box-orient: vertical;
  }

  .result-snippet :global(mark) {
    background: var(--yellow);
    color: var(--text-primary);
    padding: 0 2px;
    border-radius: 2px;
  }

  .result-score {
    font-size: var(--font-size-xs);
    color: var(--text-muted);
    flex-shrink: 0;
  }
</style>
