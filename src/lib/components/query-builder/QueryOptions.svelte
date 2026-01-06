<script lang="ts">
  import { X } from "lucide-svelte";
  import type { QueryResultType, QueryViewType, PropertyKeyInfo } from "../../types";

  interface Props {
    resultType: QueryResultType;
    includeCompleted: boolean;
    viewType: QueryViewType;
    kanbanGroupBy: string;
    kanbanCardFields: string[];
    cardCoverProperty: string | null;
    cardDisplayFields: string[];
    propertyKeys: PropertyKeyInfo[];
    onUpdateResultType: (type: QueryResultType) => void;
    onUpdateIncludeCompleted: (include: boolean) => void;
    onUpdateViewType: (type: QueryViewType) => void;
    onUpdateKanbanGroupBy: (groupBy: string) => void;
    onUpdateKanbanCardFields: (fields: string[]) => void;
    onUpdateCardCoverProperty: (prop: string | null) => void;
    onUpdateCardDisplayFields: (fields: string[]) => void;
  }

  let {
    resultType,
    includeCompleted,
    viewType,
    kanbanGroupBy,
    kanbanCardFields,
    cardCoverProperty,
    cardDisplayFields,
    propertyKeys,
    onUpdateResultType,
    onUpdateIncludeCompleted,
    onUpdateViewType,
    onUpdateKanbanGroupBy,
    onUpdateKanbanCardFields,
    onUpdateCardCoverProperty,
    onUpdateCardDisplayFields,
  }: Props = $props();

  function toggleKanbanCardField(field: string, checked: boolean) {
    if (checked) {
      onUpdateKanbanCardFields([...kanbanCardFields, field]);
    } else {
      onUpdateKanbanCardFields(kanbanCardFields.filter((f) => f !== field));
    }
  }

  function addCardDisplayField(field: string) {
    if (field && !cardDisplayFields.includes(field)) {
      onUpdateCardDisplayFields([...cardDisplayFields, field]);
    }
  }

  function removeCardDisplayField(field: string) {
    onUpdateCardDisplayFields(cardDisplayFields.filter((f) => f !== field));
  }

  // Properties that could be images (common image property names)
  const imagePropertyKeys = $derived(
    propertyKeys.filter((p) =>
      ["cover", "image", "thumbnail", "banner", "poster", "photo", "picture", "img"].some(
        (term) => p.key.toLowerCase().includes(term)
      )
    )
  );

  // Built-in fields available for cards
  const builtInFields = ["description", "priority", "context", "due_date", "heading_path"];

  // All available fields for the add dropdown (excluding already selected)
  const availableFieldsToAdd = $derived(() => {
    const allFields = [...builtInFields, ...propertyKeys.map((p) => p.key)];
    const uniqueFields = [...new Set(allFields)];
    return uniqueFields.filter((f) => !cardDisplayFields.includes(f));
  });
</script>

<div class="options-section">
  <div class="result-type">
    <span class="section-label">Show:</span>
    <label>
      <input
        type="radio"
        name="resultType"
        value="Tasks"
        checked={resultType === "Tasks"}
        onchange={() => onUpdateResultType("Tasks")}
      />
      Tasks
    </label>
    <label>
      <input
        type="radio"
        name="resultType"
        value="Notes"
        checked={resultType === "Notes"}
        onchange={() => onUpdateResultType("Notes")}
      />
      Notes
    </label>
    <label>
      <input
        type="radio"
        name="resultType"
        value="Both"
        checked={resultType === "Both"}
        onchange={() => onUpdateResultType("Both")}
      />
      Both
    </label>
  </div>

  {#if resultType !== "Notes"}
    <label class="completed-toggle">
      <input
        type="checkbox"
        checked={includeCompleted}
        onchange={(e) => onUpdateIncludeCompleted(e.currentTarget.checked)}
      />
      Include completed tasks
    </label>
  {/if}
</div>

<div class="view-section">
  <div class="view-type">
    <span class="section-label">View:</span>
    <label>
      <input
        type="radio"
        name="viewType"
        value="Table"
        checked={viewType === "Table"}
        onchange={() => onUpdateViewType("Table")}
      />
      Table
    </label>
    <label>
      <input
        type="radio"
        name="viewType"
        value="List"
        checked={viewType === "List"}
        onchange={() => onUpdateViewType("List")}
      />
      List
    </label>
    <label>
      <input
        type="radio"
        name="viewType"
        value="Card"
        checked={viewType === "Card"}
        onchange={() => onUpdateViewType("Card")}
      />
      Card
    </label>
    <label>
      <input
        type="radio"
        name="viewType"
        value="Kanban"
        checked={viewType === "Kanban"}
        onchange={() => onUpdateViewType("Kanban")}
      />
      Kanban
    </label>
  </div>

  {#if viewType === "Card"}
    <div class="card-options">
      <div class="card-option">
        <label class="option-label">Cover image:</label>
        <select
          class="option-select"
          value={cardCoverProperty ?? ""}
          onchange={(e) => onUpdateCardCoverProperty(e.currentTarget.value || null)}
        >
          <option value="">None</option>
          {#if imagePropertyKeys.length > 0}
            <optgroup label="Suggested">
              {#each imagePropertyKeys as propKey}
                <option value={propKey.key}>{propKey.key}</option>
              {/each}
            </optgroup>
          {/if}
          <optgroup label="All Properties">
            {#each propertyKeys as propKey}
              <option value={propKey.key}>{propKey.key}</option>
            {/each}
          </optgroup>
        </select>
      </div>
      <div class="card-option display-fields-option">
        <label class="option-label">Display fields:</label>
        <div class="display-fields-list">
          {#each cardDisplayFields as field (field)}
            <div class="display-field-item">
              <span class="field-name">{field}</span>
              <button
                type="button"
                class="remove-field-btn"
                onclick={() => removeCardDisplayField(field)}
                title="Remove field"
              >
                <X size={12} />
              </button>
            </div>
          {/each}
          {#if availableFieldsToAdd().length > 0}
            <select
              class="add-field-select"
              onchange={(e) => {
                addCardDisplayField(e.currentTarget.value);
                e.currentTarget.value = "";
              }}
            >
              <option value="">+ Add field</option>
              <optgroup label="Built-in">
                {#each builtInFields.filter((f) => !cardDisplayFields.includes(f)) as field}
                  <option value={field}>{field}</option>
                {/each}
              </optgroup>
              {#if propertyKeys.length > 0}
                <optgroup label="Properties">
                  {#each propertyKeys.filter((p) => !cardDisplayFields.includes(p.key)) as propKey}
                    <option value={propKey.key}>{propKey.key}</option>
                  {/each}
                </optgroup>
              {/if}
            </select>
          {/if}
        </div>
      </div>
    </div>
  {/if}

  {#if viewType === "Kanban"}
    <div class="kanban-options">
      <div class="kanban-option">
        <label class="option-label">Group by:</label>
        <select
          class="option-select"
          value={kanbanGroupBy}
          onchange={(e) => onUpdateKanbanGroupBy(e.currentTarget.value)}
        >
          <optgroup label="Task Fields">
            <option value="priority">Priority</option>
            <option value="context">Context</option>
            <option value="due_date">Due Date</option>
            <option value="completed">Completed</option>
          </optgroup>
          {#if propertyKeys.length > 0}
            <optgroup label="Properties">
              {#each propertyKeys as propKey}
                <option value={propKey.key}>{propKey.key}</option>
              {/each}
            </optgroup>
          {/if}
        </select>
      </div>
      <div class="kanban-option">
        <label class="option-label">Card fields:</label>
        <div class="card-fields-checkboxes">
          <label>
            <input
              type="checkbox"
              checked={kanbanCardFields.includes("priority")}
              onchange={(e) => toggleKanbanCardField("priority", e.currentTarget.checked)}
            />
            Priority
          </label>
          <label>
            <input
              type="checkbox"
              checked={kanbanCardFields.includes("context")}
              onchange={(e) => toggleKanbanCardField("context", e.currentTarget.checked)}
            />
            Context
          </label>
          <label>
            <input
              type="checkbox"
              checked={kanbanCardFields.includes("due_date")}
              onchange={(e) => toggleKanbanCardField("due_date", e.currentTarget.checked)}
            />
            Due Date
          </label>
        </div>
      </div>
    </div>
  {/if}
</div>

<style>
  .options-section {
    padding: var(--spacing-3) var(--spacing-4);
    border-bottom: 1px solid var(--panel-border);
    display: flex;
    gap: var(--spacing-4);
    flex-wrap: wrap;
    align-items: center;
  }

  .section-label {
    font-size: var(--font-size-sm);
    font-weight: var(--font-weight-semibold);
    color: var(--text-primary);
    text-transform: uppercase;
    letter-spacing: 0.5px;
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

  .view-section {
    padding: var(--spacing-3) var(--spacing-4);
    border-bottom: 1px solid var(--panel-border);
  }

  .view-type {
    display: flex;
    gap: var(--spacing-3);
    align-items: center;
  }

  .view-type label {
    display: flex;
    align-items: center;
    gap: var(--spacing-1);
    font-size: var(--font-size-sm);
    cursor: pointer;
    color: var(--text-primary);
  }

  .view-type input[type="radio"] {
    accent-color: var(--color-primary);
    width: 14px;
    height: 14px;
  }

  .kanban-options,
  .card-options {
    display: flex;
    gap: var(--spacing-4);
    margin-top: var(--spacing-3);
    padding: var(--spacing-3);
    background: var(--bg-surface-sunken);
    border-radius: var(--radius-md);
    flex-wrap: wrap;
  }

  .kanban-option,
  .card-option {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-1);
  }

  .option-label {
    font-size: var(--font-size-xs);
    font-weight: var(--font-weight-medium);
    color: var(--text-muted);
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }

  .option-select {
    padding: var(--spacing-1) var(--spacing-2);
    border: 1px solid var(--border-default);
    border-radius: var(--radius-sm);
    background: var(--bg-surface);
    color: var(--text-primary);
    font-size: var(--font-size-sm);
    cursor: pointer;
  }

  .card-fields-checkboxes {
    display: flex;
    gap: var(--spacing-3);
  }

  .card-fields-checkboxes label {
    display: flex;
    align-items: center;
    gap: var(--spacing-1);
    font-size: var(--font-size-sm);
    cursor: pointer;
    color: var(--text-primary);
  }

  .card-fields-checkboxes input[type="checkbox"] {
    accent-color: var(--color-primary);
    width: 14px;
    height: 14px;
  }

  /* Display fields list styles */
  .display-fields-option {
    flex: 1;
    min-width: 200px;
  }

  .display-fields-list {
    display: flex;
    flex-wrap: wrap;
    gap: var(--spacing-1);
    align-items: center;
  }

  .display-field-item {
    display: flex;
    align-items: center;
    gap: var(--spacing-1);
    padding: var(--spacing-1) var(--spacing-2);
    background: var(--bg-surface);
    border: 1px solid var(--border-default);
    border-radius: var(--radius-sm);
    font-size: var(--font-size-xs);
    color: var(--text-primary);
  }

  .display-field-item .field-name {
    max-width: 120px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .remove-field-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 2px;
    border: none;
    background: transparent;
    color: var(--text-muted);
    cursor: pointer;
    border-radius: var(--radius-xs);
    transition: color var(--transition-normal), background var(--transition-normal);
  }

  .remove-field-btn:hover {
    color: var(--color-error);
    background: var(--bg-hover);
  }

  .add-field-select {
    padding: var(--spacing-1) var(--spacing-2);
    border: 1px dashed var(--border-default);
    border-radius: var(--radius-sm);
    background: transparent;
    color: var(--text-muted);
    font-size: var(--font-size-xs);
    cursor: pointer;
    transition: border-color var(--transition-normal), color var(--transition-normal);
  }

  .add-field-select:hover {
    border-color: var(--color-primary);
    color: var(--text-primary);
  }

  .add-field-select:focus {
    outline: none;
    border-color: var(--color-primary);
  }
</style>
