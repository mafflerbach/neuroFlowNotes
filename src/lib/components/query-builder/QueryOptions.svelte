<script lang="ts">
  import type { QueryResultType, QueryViewType, PropertyKeyInfo } from "../../types";

  interface Props {
    resultType: QueryResultType;
    includeCompleted: boolean;
    viewType: QueryViewType;
    kanbanGroupBy: string;
    kanbanCardFields: string[];
    propertyKeys: PropertyKeyInfo[];
    onUpdateResultType: (type: QueryResultType) => void;
    onUpdateIncludeCompleted: (include: boolean) => void;
    onUpdateViewType: (type: QueryViewType) => void;
    onUpdateKanbanGroupBy: (groupBy: string) => void;
    onUpdateKanbanCardFields: (fields: string[]) => void;
  }

  let {
    resultType,
    includeCompleted,
    viewType,
    kanbanGroupBy,
    kanbanCardFields,
    propertyKeys,
    onUpdateResultType,
    onUpdateIncludeCompleted,
    onUpdateViewType,
    onUpdateKanbanGroupBy,
    onUpdateKanbanCardFields,
  }: Props = $props();

  function toggleCardField(field: string, checked: boolean) {
    if (checked) {
      onUpdateKanbanCardFields([...kanbanCardFields, field]);
    } else {
      onUpdateKanbanCardFields(kanbanCardFields.filter((f) => f !== field));
    }
  }
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
        value="Kanban"
        checked={viewType === "Kanban"}
        onchange={() => onUpdateViewType("Kanban")}
      />
      Kanban
    </label>
  </div>

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
              onchange={(e) => toggleCardField("priority", e.currentTarget.checked)}
            />
            Priority
          </label>
          <label>
            <input
              type="checkbox"
              checked={kanbanCardFields.includes("context")}
              onchange={(e) => toggleCardField("context", e.currentTarget.checked)}
            />
            Context
          </label>
          <label>
            <input
              type="checkbox"
              checked={kanbanCardFields.includes("due_date")}
              onchange={(e) => toggleCardField("due_date", e.currentTarget.checked)}
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

  .kanban-options {
    display: flex;
    gap: var(--spacing-4);
    margin-top: var(--spacing-3);
    padding: var(--spacing-3);
    background: var(--bg-surface-sunken);
    border-radius: var(--radius-md);
    flex-wrap: wrap;
  }

  .kanban-option {
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
</style>
