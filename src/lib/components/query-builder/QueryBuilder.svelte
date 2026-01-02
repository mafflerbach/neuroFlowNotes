<script lang="ts">
  import { getPropertyKeys, getPropertyValues, runQuery, getAllTags, getFolderTree } from "../../services/api";
  import { workspaceStore } from "../../stores";
  import QueryFilters from "./QueryFilters.svelte";
  import QueryOptions from "./QueryOptions.svelte";
  import QueryResults from "./QueryResults.svelte";
  import { generateYamlCode } from "./queryUtils";
  import type {
    PropertyKeyInfo,
    PropertyFilter,
    FilterMatchMode,
    QueryResultType,
    QueryViewType,
    QueryRequest,
    QueryResultItem,
    FolderNode,
  } from "../../types";

  interface Props {
    onResultClick?: (noteId: number, notePath: string, noteTitle: string | null) => void;
  }

  let { onResultClick }: Props = $props();

  // Filter state
  let filters = $state<PropertyFilter[]>([
    { key: "", operator: "Exists", value: null },
  ]);
  let matchMode = $state<FilterMatchMode>("All");
  let resultType = $state<QueryResultType>("Tasks");
  let includeCompleted = $state(false);

  // View configuration
  let viewType = $state<QueryViewType>("Table");
  let kanbanGroupBy = $state("priority");
  let kanbanCardFields = $state(["context", "due_date"]);
  let cardCoverProperty = $state<string | null>(null);
  let cardDisplayFields = $state(["description"]);

  // Persistence key
  const STORAGE_KEY = "queryBuilderState";

  // Data
  let propertyKeys = $state<PropertyKeyInfo[]>([]);
  let propertyValues = $state<Record<string, string[]>>({});
  let allTags = $state<string[]>([]);
  let folderPaths = $state<string[]>([]);
  let results = $state<QueryResultItem[]>([]);
  let totalCount = $state(0);
  let loading = $state(false);
  let loadingKeys = $state(true);
  let showCode = $state(false);

  // Flatten folder tree into array of paths
  function flattenFolderTree(node: FolderNode, paths: string[] = []): string[] {
    if (node.is_dir && node.path) {
      paths.push(node.path);
    }
    for (const child of node.children) {
      if (child.is_dir) {
        flattenFolderTree(child, paths);
      }
    }
    return paths;
  }

  // Load persisted state
  function loadPersistedState(): {
    filters: PropertyFilter[];
    matchMode: FilterMatchMode;
    resultType: QueryResultType;
    includeCompleted: boolean;
  } | null {
    try {
      const stored = localStorage.getItem(STORAGE_KEY);
      if (stored) {
        return JSON.parse(stored);
      }
    } catch (e) {
      console.error("Failed to load query builder state:", e);
    }
    return null;
  }

  // Load data on mount
  $effect(() => {
    loadPropertyKeys();
    loadTags();
    loadFolders();
    restorePersistedState();
  });

  // Create derived value for persistence tracking
  const stateForPersistence = $derived(JSON.stringify({
    filters: filters.filter(f => f.key),
    matchMode,
    resultType,
    includeCompleted,
  }));

  // Persist state when it changes
  $effect(() => {
    if (stateForPersistence) {
      const timer = setTimeout(() => {
        try {
          localStorage.setItem(STORAGE_KEY, stateForPersistence);
        } catch (e) {
          console.error("Failed to persist query builder state:", e);
        }
      }, 100);
      return () => clearTimeout(timer);
    }
  });

  async function loadPropertyKeys() {
    loadingKeys = true;
    try {
      propertyKeys = await getPropertyKeys();
    } catch (e) {
      console.error("Failed to load property keys:", e);
    } finally {
      loadingKeys = false;
    }
  }

  async function loadTags() {
    try {
      allTags = await getAllTags();
    } catch (e) {
      console.error("Failed to load tags:", e);
    }
  }

  async function loadFolders() {
    try {
      const tree = await getFolderTree();
      folderPaths = flattenFolderTree(tree).sort();
    } catch (e) {
      console.error("Failed to load folders:", e);
    }
  }

  function restorePersistedState() {
    const persisted = loadPersistedState();
    if (persisted) {
      if (persisted.filters && persisted.filters.length > 0) {
        filters = persisted.filters;
        for (const f of persisted.filters) {
          if (f.key && !f.key.startsWith("_")) {
            loadValuesForKey(f.key);
          }
        }
      }
      matchMode = persisted.matchMode ?? "All";
      resultType = persisted.resultType ?? "Tasks";
      includeCompleted = persisted.includeCompleted ?? false;
    }
  }

  async function loadValuesForKey(key: string) {
    if (propertyValues[key]) return;
    try {
      // Backend auto-expands list-type properties
      const values = await getPropertyValues(key);
      propertyValues = { ...propertyValues, [key]: values };
    } catch (e) {
      console.error(`Failed to load values for key ${key}:`, e);
    }
  }

  // Run the query
  async function executeQuery() {
    const validFilters = filters.filter((f) => f.key);
    if (validFilters.length === 0) {
      return;
    }

    loading = true;
    try {
      const request: QueryRequest = {
        filters: validFilters,
        match_mode: matchMode,
        result_type: resultType,
        include_completed: includeCompleted,
        limit: 100,
      };

      const response = await runQuery(request);
      results = response.results;
      totalCount = response.total_count;
    } catch (e) {
      console.error("Query failed:", e);
      results = [];
      totalCount = 0;
    } finally {
      loading = false;
    }
  }

  // Navigate to a note
  function openNote(noteId: number, path: string, title: string | null) {
    if (onResultClick) {
      onResultClick(noteId, path, title);
    } else {
      workspaceStore.followLink({
        path,
        id: noteId,
        title: title ?? path.replace(".md", ""),
      });
    }
  }

  // Generate YAML code
  function getYamlCode(): string {
    return generateYamlCode({
      filters,
      matchMode,
      resultType,
      includeCompleted,
      viewType,
      kanbanGroupBy,
      kanbanCardFields,
      cardCoverProperty,
      cardDisplayFields,
    });
  }

  // Copy code to clipboard
  async function copyCode() {
    const code = getYamlCode();
    try {
      await navigator.clipboard.writeText(code);
    } catch (e) {
      console.error("Failed to copy to clipboard:", e);
    }
  }
</script>

<div class="query-builder">
  <div class="header">
    <h2>Query Builder</h2>
    <div class="header-actions">
      <button class="code-btn" onclick={() => (showCode = !showCode)} class:active={showCode}>
        {showCode ? "Hide Code" : "See Code"}
      </button>
      <button class="run-btn" onclick={executeQuery} disabled={loading}>
        {loading ? "Running..." : "Run Query"}
      </button>
    </div>
  </div>

  {#if showCode}
    <div class="code-section">
      <div class="code-header">
        <span class="section-label">Query Embed Code</span>
        <button class="copy-btn" onclick={copyCode}>Copy</button>
      </div>
      <pre class="code-block">{getYamlCode()}</pre>
      <p class="code-hint">Paste this into any note to embed live query results.</p>
    </div>
  {/if}

  <QueryFilters
    {filters}
    {matchMode}
    {propertyKeys}
    {propertyValues}
    {allTags}
    {folderPaths}
    {loadingKeys}
    onUpdateFilters={(f) => (filters = f)}
    onUpdateMatchMode={(m) => (matchMode = m)}
    onLoadValuesForKey={loadValuesForKey}
  />

  <QueryOptions
    {resultType}
    {includeCompleted}
    {viewType}
    {kanbanGroupBy}
    {kanbanCardFields}
    {cardCoverProperty}
    {cardDisplayFields}
    {propertyKeys}
    onUpdateResultType={(t) => (resultType = t)}
    onUpdateIncludeCompleted={(c) => (includeCompleted = c)}
    onUpdateViewType={(t) => (viewType = t)}
    onUpdateKanbanGroupBy={(g) => (kanbanGroupBy = g)}
    onUpdateKanbanCardFields={(f) => (kanbanCardFields = f)}
    onUpdateCardCoverProperty={(p) => (cardCoverProperty = p)}
    onUpdateCardDisplayFields={(f) => (cardDisplayFields = f)}
  />

  <QueryResults
    {results}
    {totalCount}
    {loading}
    {filters}
    {viewType}
    {cardCoverProperty}
    {cardDisplayFields}
    onOpenNote={openNote}
  />
</div>

<style>
  .query-builder {
    display: flex;
    flex-direction: column;
    height: 100%;
    background: var(--panel-bg);
    overflow: hidden;
  }

  .header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: var(--spacing-4);
    border-bottom: 1px solid var(--panel-border);
  }

  .header h2 {
    margin: 0;
    font-size: var(--font-size-lg);
    font-weight: var(--font-weight-semibold);
    color: var(--text-primary);
  }

  .header-actions {
    display: flex;
    gap: var(--spacing-2);
  }

  .code-btn {
    padding: var(--spacing-2) var(--spacing-3);
    background: transparent;
    color: var(--text-muted);
    border: 1px solid var(--panel-border);
    border-radius: var(--radius-sm);
    cursor: pointer;
    font-size: var(--font-size-sm);
    font-weight: var(--font-weight-medium);
    transition: all 0.15s;
    display: inline-flex;
    align-items: center;
    gap: var(--spacing-1);
  }

  .code-btn::before {
    content: "</>";
    font-family: var(--font-family-mono);
    font-size: var(--font-size-xs);
    opacity: 0.7;
  }

  .code-btn:hover {
    background: var(--bg-hover);
    color: var(--text-primary);
    border-color: var(--border-default);
  }

  .code-btn.active {
    background: var(--btn-primary-bg);
    color: var(--btn-primary-text);
    border-color: var(--btn-primary-bg);
  }

  .code-btn.active::before {
    opacity: 1;
  }

  .code-section {
    padding: var(--spacing-3) var(--spacing-4);
    background: var(--bg-surface-sunken);
    border-bottom: 1px solid var(--panel-border);
  }

  .code-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: var(--spacing-2);
  }

  .section-label {
    font-size: var(--font-size-sm);
    font-weight: var(--font-weight-semibold);
    color: var(--text-primary);
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }

  .copy-btn {
    padding: var(--spacing-1) var(--spacing-3);
    background: transparent;
    color: var(--color-success);
    border: 1px solid var(--color-success);
    border-radius: var(--radius-sm);
    cursor: pointer;
    font-size: var(--font-size-xs);
    font-weight: var(--font-weight-medium);
    transition: all 0.15s;
  }

  .copy-btn:hover {
    background: var(--color-success);
    color: var(--color-white);
  }

  .copy-btn:active {
    transform: scale(0.97);
  }

  .code-block {
    margin: 0;
    padding: var(--spacing-3);
    background: var(--bg-surface-sunken);
    border-radius: var(--radius-sm);
    border: 1px solid var(--border-default);
    font-family: var(--font-family-mono);
    font-size: var(--font-size-sm);
    line-height: 1.6;
    overflow-x: auto;
    white-space: pre;
    color: var(--text-primary);
  }

  .code-hint {
    margin: var(--spacing-2) 0 0 0;
    font-size: var(--font-size-xs);
    color: var(--text-muted);
    font-style: italic;
  }

  .run-btn {
    padding: var(--spacing-2) var(--spacing-4);
    background: var(--btn-primary-bg);
    color: var(--btn-primary-text);
    border: none;
    border-radius: var(--radius-sm);
    cursor: pointer;
    font-weight: var(--font-weight-medium);
    transition: background 0.15s;
  }

  .run-btn:hover:not(:disabled) {
    background: var(--btn-primary-bg-hover);
  }

  .run-btn:disabled {
    background: var(--btn-secondary-bg);
    color: var(--text-muted);
    cursor: not-allowed;
  }
</style>
