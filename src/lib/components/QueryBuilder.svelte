<script lang="ts">
  import { getPropertyKeys, getPropertyValues, runQuery } from "../services/api";
  import { workspaceStore } from "../stores";
  import type {
    PropertyKeyInfo,
    PropertyFilter,
    PropertyOperator,
    FilterMatchMode,
    QueryResultType,
    QueryRequest,
    QueryResultItem,
  } from "../types";

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

  // Data
  let propertyKeys = $state<PropertyKeyInfo[]>([]);
  let propertyValues = $state<Record<string, string[]>>({});
  let results = $state<QueryResultItem[]>([]);
  let totalCount = $state(0);
  let loading = $state(false);
  let loadingKeys = $state(true);
  let showCode = $state(false);

  // Operator labels
  const operatorLabels: Record<PropertyOperator, string> = {
    Exists: "exists",
    NotExists: "does not exist",
    Equals: "equals",
    NotEquals: "does not equal",
    Contains: "contains",
    StartsWith: "starts with",
    EndsWith: "ends with",
  };

  // Operators that don't need a value
  const valuelessOperators: PropertyOperator[] = ["Exists", "NotExists"];

  // Load property keys on mount
  $effect(() => {
    loadPropertyKeys();
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

  // Load property values when a key is selected
  async function loadValuesForKey(key: string) {
    if (propertyValues[key]) return; // Already loaded
    try {
      const values = await getPropertyValues(key);
      propertyValues = { ...propertyValues, [key]: values };
    } catch (e) {
      console.error(`Failed to load values for key ${key}:`, e);
    }
  }

  // Add a new filter row
  function addFilter() {
    filters = [...filters, { key: "", operator: "Exists", value: null }];
  }

  // Remove a filter row
  function removeFilter(index: number) {
    if (filters.length <= 1) return;
    filters = filters.filter((_, i) => i !== index);
  }

  // Update a filter's key
  function updateFilterKey(index: number, key: string) {
    filters[index].key = key;
    if (key) {
      loadValuesForKey(key);
    }
  }

  // Update a filter's operator
  function updateFilterOperator(index: number, operator: PropertyOperator) {
    filters[index].operator = operator;
    // Clear value if switching to valueless operator
    if (valuelessOperators.includes(operator)) {
      filters[index].value = null;
    }
  }

  // Update a filter's value
  function updateFilterValue(index: number, value: string) {
    filters[index].value = value || null;
  }

  // Run the query
  async function executeQuery() {
    // Validate filters
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

  // Get priority class for styling
  function priorityClass(priority: string | null): string {
    if (!priority) return "";
    return `priority-${priority}`;
  }

  // Generate YAML code for the current query
  function generateYamlCode(): string {
    const validFilters = filters.filter((f) => f.key);

    let yaml = "```query\n";

    // Filters
    if (validFilters.length > 0) {
      yaml += "filters:\n";
      for (const filter of validFilters) {
        yaml += `  - key: ${filter.key}\n`;
        yaml += `    operator: ${filter.operator}\n`;
        if (filter.value && !valuelessOperators.includes(filter.operator)) {
          yaml += `    value: "${filter.value}"\n`;
        }
      }
    }

    // Match mode (only if multiple filters)
    if (validFilters.length > 1) {
      yaml += `match_mode: ${matchMode}\n`;
    }

    // Result type
    yaml += `result_type: ${resultType}\n`;

    // Include completed (only for tasks)
    if (resultType !== "Notes") {
      yaml += `include_completed: ${includeCompleted}\n`;
    }

    // View configuration
    yaml += "view:\n";
    yaml += "  view_type: Table\n";

    // Default columns based on result type
    if (resultType === "Notes") {
      yaml += "  columns:\n";
      yaml += "    - title\n";
      yaml += "    - path\n";
    } else {
      yaml += "  columns:\n";
      yaml += "    - description\n";
      yaml += "    - priority\n";
      yaml += "    - due_date\n";
      yaml += "    - note_title\n";
    }

    yaml += "```";

    return yaml;
  }

  // Copy code to clipboard
  async function copyCode() {
    const code = generateYamlCode();
    try {
      await navigator.clipboard.writeText(code);
    } catch (e) {
      console.error("Failed to copy to clipboard:", e);
    }
  }

  // Toggle code display
  function toggleShowCode() {
    showCode = !showCode;
  }
</script>

<div class="query-builder">
  <div class="header">
    <h2>Query Builder</h2>
    <div class="header-actions">
      <button class="code-btn" onclick={toggleShowCode} class:active={showCode}>
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
      <pre class="code-block">{generateYamlCode()}</pre>
      <p class="code-hint">Paste this into any note to embed live query results.</p>
    </div>
  {/if}

  <div class="filters-section">
    <div class="filters-header">
      <span class="section-label">Filters</span>
      <div class="match-mode">
        <label>
          <input
            type="radio"
            name="matchMode"
            value="All"
            checked={matchMode === "All"}
            onchange={() => (matchMode = "All")}
          />
          All (AND)
        </label>
        <label>
          <input
            type="radio"
            name="matchMode"
            value="Any"
            checked={matchMode === "Any"}
            onchange={() => (matchMode = "Any")}
          />
          Any (OR)
        </label>
      </div>
    </div>

    {#if loadingKeys}
      <div class="loading-keys">Loading properties...</div>
    {:else}
      <div class="filter-rows">
        {#each filters as filter, index (index)}
          <div class="filter-row">
            <select
              class="key-select"
              value={filter.key}
              onchange={(e) => updateFilterKey(index, e.currentTarget.value)}
            >
              <option value="">Select property...</option>
              {#each propertyKeys as pk}
                <option value={pk.key}>{pk.key} ({pk.usage_count})</option>
              {/each}
            </select>

            <select
              class="operator-select"
              value={filter.operator}
              onchange={(e) =>
                updateFilterOperator(index, e.currentTarget.value as PropertyOperator)}
            >
              {#each Object.entries(operatorLabels) as [op, label]}
                <option value={op}>{label}</option>
              {/each}
            </select>

            {#if !valuelessOperators.includes(filter.operator)}
              <input
                type="text"
                class="value-input"
                placeholder="Value..."
                value={filter.value ?? ""}
                oninput={(e) => updateFilterValue(index, e.currentTarget.value)}
                list={`values-${index}`}
              />
              {#if filter.key && propertyValues[filter.key]}
                <datalist id={`values-${index}`}>
                  {#each propertyValues[filter.key] as val}
                    <option value={val} />
                  {/each}
                </datalist>
              {/if}
            {/if}

            <div class="filter-actions">
              <button
                class="add-btn"
                onclick={addFilter}
                title="Add filter"
              >+</button>
              <button
                class="remove-btn"
                onclick={() => removeFilter(index)}
                disabled={filters.length <= 1}
                title="Remove filter"
              >-</button>
            </div>
          </div>
        {/each}
      </div>
    {/if}
  </div>

  <div class="options-section">
    <div class="result-type">
      <span class="section-label">Show:</span>
      <label>
        <input
          type="radio"
          name="resultType"
          value="Tasks"
          checked={resultType === "Tasks"}
          onchange={() => (resultType = "Tasks")}
        />
        Tasks
      </label>
      <label>
        <input
          type="radio"
          name="resultType"
          value="Notes"
          checked={resultType === "Notes"}
          onchange={() => (resultType = "Notes")}
        />
        Notes
      </label>
      <label>
        <input
          type="radio"
          name="resultType"
          value="Both"
          checked={resultType === "Both"}
          onchange={() => (resultType = "Both")}
        />
        Both
      </label>
    </div>

    {#if resultType !== "Notes"}
      <label class="completed-toggle">
        <input type="checkbox" bind:checked={includeCompleted} />
        Include completed tasks
      </label>
    {/if}
  </div>

  <div class="results-section">
    <div class="results-header">
      <span class="section-label">Results</span>
      {#if totalCount > 0}
        <span class="result-count">{totalCount} items</span>
      {/if}
    </div>

    <div class="results-list">
      {#if loading}
        <div class="loading">Running query...</div>
      {:else if results.length === 0}
        <div class="empty-state">
          {filters.some((f) => f.key)
            ? "No results match your query"
            : "Add filters and run query to see results"}
        </div>
      {:else}
        {#each results as item (`${item.item_type}-${item.item_type === "task" ? item.task?.todo.id : item.note?.id}`)}
          {#if item.item_type === "task" && item.task}
            <div class="result-item task-item" class:completed={item.task.todo.completed}>
              <div class="item-type-badge task">Task</div>
              <div class="item-content">
                <div class="item-main">
                  <span class="item-text">{item.task.todo.description}</span>
                  {#if item.task.todo.priority}
                    <span class="badge {priorityClass(item.task.todo.priority)}"
                      >{item.task.todo.priority}</span
                    >
                  {/if}
                  {#if item.task.todo.context}
                    <span class="badge context">@{item.task.todo.context}</span>
                  {/if}
                  {#if item.task.todo.due_date}
                    <span class="badge due-date">{item.task.todo.due_date}</span>
                  {/if}
                </div>
                <div class="item-meta">
                  <button
                    class="note-link"
                    onclick={() =>
                      openNote(
                        item.task!.todo.note_id,
                        item.task!.note_path,
                        item.task!.note_title
                      )}
                  >
                    {item.task.note_title || item.task.note_path.replace(".md", "")}
                  </button>
                  {#if item.properties.length > 0}
                    <span class="properties-preview">
                      {item.properties
                        .slice(0, 3)
                        .map((p) => `${p.key}: ${p.value}`)
                        .join(", ")}
                    </span>
                  {/if}
                </div>
              </div>
            </div>
          {:else if item.item_type === "note" && item.note}
            <div class="result-item note-item">
              <div class="item-type-badge note">Note</div>
              <div class="item-content">
                <div class="item-main">
                  <button
                    class="note-link title"
                    onclick={() =>
                      openNote(item.note!.id, item.note!.path, item.note!.title)}
                  >
                    {item.note.title || item.note.path.replace(".md", "")}
                  </button>
                </div>
                {#if item.properties.length > 0}
                  <div class="item-meta">
                    <span class="properties-preview">
                      {item.properties
                        .slice(0, 5)
                        .map((p) => `${p.key}: ${p.value}`)
                        .join(", ")}
                    </span>
                  </div>
                {/if}
              </div>
            </div>
          {/if}
        {/each}
      {/if}
    </div>
  </div>
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

  .section-label {
    font-size: var(--font-size-sm);
    font-weight: var(--font-weight-semibold);
    color: var(--text-primary);
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }

  .filters-section {
    padding: var(--spacing-3) var(--spacing-4);
    border-bottom: 1px solid var(--panel-border);
  }

  .filters-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: var(--spacing-3);
  }

  .match-mode {
    display: flex;
    gap: var(--spacing-3);
  }

  .match-mode label {
    display: flex;
    align-items: center;
    gap: var(--spacing-1);
    font-size: var(--font-size-sm);
    cursor: pointer;
    color: var(--text-primary);
  }

  .match-mode input[type="radio"] {
    accent-color: var(--color-primary);
    width: 14px;
    height: 14px;
  }

  .loading-keys {
    color: var(--text-muted);
    font-size: var(--font-size-sm);
    padding: var(--spacing-2);
  }

  .filter-rows {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-2);
  }

  .filter-row {
    display: flex;
    gap: var(--spacing-2);
    align-items: center;
  }

  .key-select,
  .operator-select {
    padding: var(--spacing-1) var(--spacing-2);
    font-size: var(--font-size-sm);
    border: 1px solid var(--panel-border);
    border-radius: var(--radius-sm);
    background: var(--bg-secondary);
    color: var(--text-primary);
    cursor: pointer;
  }

  .key-select {
    flex: 1;
    min-width: 120px;
  }

  .operator-select {
    min-width: 130px;
  }

  .value-input {
    flex: 1;
    padding: var(--spacing-1) var(--spacing-2);
    font-size: var(--font-size-sm);
    border: 1px solid var(--panel-border);
    border-radius: var(--radius-sm);
    background: var(--bg-secondary);
    color: var(--text-primary);
  }

  .filter-actions {
    display: flex;
    gap: var(--spacing-1);
  }

  .add-btn,
  .remove-btn {
    width: 24px;
    height: 24px;
    padding: 0;
    border: 1px solid var(--panel-border);
    border-radius: var(--radius-sm);
    background: var(--bg-secondary);
    color: var(--text-primary);
    cursor: pointer;
    font-size: var(--font-size-base);
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .add-btn:hover {
    background: var(--color-success);
    color: var(--color-white);
    border-color: var(--color-success);
  }

  .remove-btn:hover:not(:disabled) {
    background: var(--color-error);
    color: var(--color-white);
    border-color: var(--color-error);
  }

  .remove-btn:disabled {
    opacity: 0.3;
    cursor: not-allowed;
  }

  .options-section {
    padding: var(--spacing-3) var(--spacing-4);
    border-bottom: 1px solid var(--panel-border);
    display: flex;
    gap: var(--spacing-4);
    flex-wrap: wrap;
    align-items: center;
  }

  .result-type {
    display: flex;
    gap: var(--spacing-3);
    align-items: center;
  }

  .result-type label,
  .completed-toggle {
    display: flex;
    align-items: center;
    gap: var(--spacing-1);
    font-size: var(--font-size-sm);
    cursor: pointer;
    color: var(--text-primary);
  }

  .result-type input[type="radio"] {
    accent-color: var(--color-primary);
    width: 14px;
    height: 14px;
  }

  .completed-toggle input[type="checkbox"] {
    accent-color: var(--color-primary);
    width: 14px;
    height: 14px;
  }

  .results-section {
    flex: 1;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .results-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: var(--spacing-3) var(--spacing-4);
    border-bottom: 1px solid var(--panel-border);
  }

  .result-count {
    font-size: var(--font-size-sm);
    color: var(--text-muted);
  }

  .results-list {
    flex: 1;
    overflow-y: auto;
    overflow-x: hidden;
    padding: var(--spacing-2);
  }

  .loading,
  .empty-state {
    padding: var(--spacing-8);
    text-align: center;
    color: var(--text-muted);
  }

  .result-item {
    display: flex;
    gap: var(--spacing-2);
    padding: var(--spacing-2) var(--spacing-3);
    border-radius: var(--radius-sm);
    margin-bottom: var(--spacing-2);
    border-bottom: 1px solid var(--panel-border);
  }

  .result-item:hover {
    background: var(--bg-hover);
  }

  .result-item.completed .item-text {
    text-decoration: line-through;
    color: var(--text-muted);
  }

  .item-type-badge {
    font-size: var(--font-size-xs);
    padding: 2px 6px;
    border-radius: var(--radius-sm);
    font-weight: var(--font-weight-medium);
    flex-shrink: 0;
  }

  .item-type-badge.task {
    background: var(--color-primary);
    color: var(--color-white);
  }

  .item-type-badge.note {
    background: var(--color-info);
    color: var(--color-white);
  }

  .item-content {
    flex: 1;
    min-width: 0;
  }

  .item-main {
    display: flex;
    align-items: flex-start;
    gap: var(--spacing-2);
    flex-wrap: wrap;
  }

  .item-text {
    font-size: var(--font-size-sm);
    word-break: break-word;
  }

  .badge {
    font-size: var(--font-size-xs);
    padding: 1px 6px;
    border-radius: var(--radius-sm);
    background: var(--bg-surface-raised);
    color: var(--text-muted);
  }

  .badge.priority-high {
    background: var(--color-error);
    color: var(--color-white);
  }

  .badge.priority-medium {
    background: var(--color-warning);
    color: var(--color-black);
  }

  .badge.priority-low {
    background: var(--bg-surface-raised);
    color: var(--text-muted);
  }

  .badge.context {
    background: var(--color-info);
    color: var(--color-white);
  }

  .badge.due-date {
    background: var(--color-warning);
    color: var(--color-black);
  }

  .item-meta {
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    gap: var(--spacing-1);
    font-size: var(--font-size-sm);
    color: var(--text-muted);
    margin-top: var(--spacing-1);
  }

  .note-link {
    background: none;
    border: none;
    color: var(--text-link);
    cursor: pointer;
    padding: 0;
    font-size: inherit;
    text-decoration: none;
  }

  .note-link:hover {
    text-decoration: underline;
  }

  .note-link.title {
    font-size: var(--font-size-base);
    font-weight: var(--font-weight-medium);
  }

  .properties-preview {
    color: var(--text-secondary);
    font-size: var(--font-size-xs);
    word-break: break-word;
    line-height: 1.4;
  }
</style>
