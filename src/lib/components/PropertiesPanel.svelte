<script lang="ts">
  import { Plus, X, Check, Calendar, Repeat, Clock, Type, Hash, ToggleLeft, List } from "lucide-svelte";
  import { getProperties, setProperty, deleteProperty } from "../services/api";
  import { getScheduleBlocksForNote, updateScheduleBlock } from "../services/api/calendar";
  import type { PropertyDto, ScheduleBlockDto } from "../types";

  interface Props {
    noteId: number;
  }

  let { noteId }: Props = $props();

  // Available property types
  type PropertyType = "text" | "date" | "number" | "boolean" | "list";
  const PROPERTY_TYPES: { value: PropertyType; label: string; icon: typeof Type }[] = [
    { value: "text", label: "Text", icon: Type },
    { value: "date", label: "Date", icon: Calendar },
    { value: "number", label: "Number", icon: Hash },
    { value: "boolean", label: "Boolean", icon: ToggleLeft },
    { value: "list", label: "List", icon: List },
  ];

  let properties = $state<PropertyDto[]>([]);
  let scheduleBlocks = $state<ScheduleBlockDto[]>([]);
  let isLoading = $state(false);
  let newKey = $state("");
  let newValue = $state("");
  let newType = $state<PropertyType>("text");
  let isAddingNew = $state(false);
  let editingKey = $state<string | null>(null);
  let editingTypeKey = $state<string | null>(null);  // Track which property's type is being edited
  let keyInputRef = $state<HTMLInputElement | null>(null);
  let editingBlockId = $state<number | null>(null);

  // Auto-focus key input when adding new property
  $effect(() => {
    if (isAddingNew && keyInputRef) {
      keyInputRef.focus();
    }
  });

  // Fetch properties when noteId changes
  $effect(() => {
    if (noteId) {
      fetchProperties();
    }
  });

  async function fetchProperties() {
    isLoading = true;
    try {
      // Fetch both properties and schedule blocks in parallel
      const [props, blocks] = await Promise.all([
        getProperties(noteId),
        getScheduleBlocksForNote(noteId),
      ]);
      properties = props;
      scheduleBlocks = blocks;
    } catch (e) {
      console.error("[PropertiesPanel] Failed to fetch properties:", e);
    } finally {
      isLoading = false;
    }
  }

  // Format date for display
  function formatDate(dateStr: string): string {
    const date = new Date(dateStr + "T00:00:00");
    return date.toLocaleDateString("en-US", {
      weekday: "short",
      month: "short",
      day: "numeric",
    });
  }

  // Format time for display (HH:MM:SS -> HH:MM)
  function formatTime(timeStr: string): string {
    return timeStr.slice(0, 5);
  }

  // Get human-readable recurrence label
  function getRecurrenceLabel(rrule: string): string {
    if (rrule.includes("FREQ=DAILY")) return "Daily";
    if (rrule.includes("BYDAY=MO,TU,WE,TH,FR")) return "Weekdays";
    if (rrule.includes("INTERVAL=2") && rrule.includes("FREQ=WEEKLY"))
      return "Every 2 weeks";
    if (rrule.includes("FREQ=WEEKLY")) return "Weekly";
    if (rrule.includes("FREQ=MONTHLY")) return "Monthly";
    return "Recurring";
  }

  async function handleValueChange(key: string, value: string, propType?: string | null) {
    try {
      await setProperty({
        note_id: noteId,
        key,
        value,
        property_type: propType ?? null,
      });
      await fetchProperties();
    } catch (e) {
      console.error("Failed to update property:", e);
    }
  }

  async function handleTypeChange(key: string, currentValue: string | null, newType: PropertyType) {
    try {
      await setProperty({
        note_id: noteId,
        key,
        value: currentValue,
        property_type: newType,
      });
      editingTypeKey = null;
      await fetchProperties();
    } catch (e) {
      console.error("Failed to update property type:", e);
    }
  }

  async function handleAddProperty() {
    if (newKey.trim()) {
      try {
        // Format the value based on type
        let formattedValue = newValue.trim() || null;
        if (newType === "boolean" && formattedValue) {
          formattedValue = formattedValue.toLowerCase() === "true" ? "true" : "false";
        }

        await setProperty({
          note_id: noteId,
          key: newKey.trim(),
          value: formattedValue,
          property_type: newType,
        });
        newKey = "";
        newValue = "";
        newType = "text";
        isAddingNew = false;
        await fetchProperties();
      } catch (e) {
        console.error("[PropertiesPanel] Failed to add property:", e);
      }
    }
  }

  // Get the icon component for a property type
  function getTypeIcon(propType: string | null) {
    const typeInfo = PROPERTY_TYPES.find(t => t.value === propType);
    return typeInfo?.icon || Type;
  }

  // Get default value for a type
  function getDefaultValueForType(propType: PropertyType): string {
    switch (propType) {
      case "boolean": return "false";
      case "date": return new Date().toISOString().split("T")[0];
      case "number": return "0";
      case "list": return "";
      default: return "";
    }
  }

  async function handleDeleteProperty(key: string) {
    try {
      await deleteProperty(noteId, key);
      await fetchProperties();
    } catch (e) {
      console.error("Failed to delete property:", e);
    }
  }

  function startEditing(key: string) {
    editingKey = key;
  }

  function stopEditing() {
    editingKey = null;
  }

  function startAddingProperty() {
    isAddingNew = true;
  }

  function handleKeyDown(event: KeyboardEvent) {
    if (event.key === "Enter") {
      // Only submit if both key and value are filled
      if (newKey.trim() && newValue.trim()) {
        handleAddProperty();
      }
    } else if (event.key === "Escape") {
      isAddingNew = false;
      newKey = "";
      newValue = "";
    }
  }

  async function handleBlockDateChange(block: ScheduleBlockDto, newDate: string) {
    try {
      await updateScheduleBlock({
        id: block.id,
        note_id: null,
        date: newDate,
        start_time: null,
        end_time: null,
        label: null,
        color: null,
        context: null,
        rrule: null,
      });
      editingBlockId = null;
      await fetchProperties();
    } catch (e) {
      console.error("[PropertiesPanel] Failed to update block date:", e);
    }
  }
</script>

<div class="properties-panel">
  <!-- Schedule Blocks Section (if any) -->
  {#if scheduleBlocks.length > 0}
    <div class="schedule-section">
      <div class="section-header">
        <Calendar size={14} />
        <span>Schedule</span>
      </div>
      <div class="schedule-blocks">
        {#each scheduleBlocks as block (block.id + "-" + block.date)}
          <div class="schedule-block-info" style:--block-color={block.color || 'var(--color-primary)'}>
            <div class="block-color-indicator"></div>
            <div class="block-main">
              {#if block.label}
                <div class="block-label">{block.label}</div>
              {/if}
              <div class="block-details">
                <div class="block-date">
                  {#if editingBlockId === block.id}
                    <input
                      type="date"
                      class="date-input"
                      value={block.date}
                      onblur={(e) => {
                        const newDate = e.currentTarget.value;
                        if (newDate && newDate !== block.date) {
                          handleBlockDateChange(block, newDate);
                        } else {
                          editingBlockId = null;
                        }
                      }}
                      onkeydown={(e) => {
                        if (e.key === "Enter") {
                          const newDate = e.currentTarget.value;
                          if (newDate && newDate !== block.date) {
                            handleBlockDateChange(block, newDate);
                          } else {
                            editingBlockId = null;
                          }
                        } else if (e.key === "Escape") {
                          editingBlockId = null;
                        }
                      }}
                    />
                  {:else}
                    <button
                      class="date-display-btn"
                      onclick={() => editingBlockId = block.id}
                      title="Click to change date"
                    >
                      {formatDate(block.date)}
                    </button>
                  {/if}
                </div>
                <div class="block-time">
                  <Clock size={12} />
                  {formatTime(block.start_time)} - {formatTime(block.end_time)}
                </div>
                {#if block.rrule}
                  <div class="block-recurrence">
                    <Repeat size={12} />
                    {getRecurrenceLabel(block.rrule)}
                  </div>
                {/if}
                {#if block.context}
                  <div class="block-context">
                    @{block.context}
                  </div>
                {/if}
              </div>
            </div>
          </div>
        {/each}
      </div>
    </div>
  {/if}

  <!-- Properties Section -->
  <div class="panel-header">
    <h3 class="panel-title">Properties</h3>
    <button
      class="add-btn"
      onclick={startAddingProperty}
      title="Add property"
      disabled={isAddingNew}
    >
      <Plus size={16} />
    </button>
  </div>

  <div class="properties-list">
    {#if isLoading}
      <div class="loading-state">Loading...</div>
    {:else if properties.length === 0 && !isAddingNew}
      <div class="empty-state">
        <p>No properties</p>
        <button class="add-first-btn" onclick={startAddingProperty}>
          Add property
        </button>
      </div>
    {:else}
      {#each properties as prop (prop.id)}
        {@const TypeIcon = getTypeIcon(prop.property_type)}
        <div class="property-row">
          <div class="property-key">
            {#if editingTypeKey === prop.key}
              <select
                class="type-select-inline"
                value={prop.property_type || "text"}
                onchange={(e) => handleTypeChange(prop.key, prop.value, e.currentTarget.value as PropertyType)}
                onblur={() => editingTypeKey = null}
              >
                {#each PROPERTY_TYPES as ptype}
                  <option value={ptype.value}>{ptype.label}</option>
                {/each}
              </select>
            {:else}
              <button
                class="type-icon-btn"
                title={`Type: ${prop.property_type || "text"} (click to change)`}
                onclick={() => editingTypeKey = prop.key}
              >
                <TypeIcon size={12} />
              </button>
            {/if}
            <span>{prop.key}</span>
          </div>
          <div class="property-value">
            {#if editingKey === prop.key}
              {#if prop.property_type === "date"}
                <input
                  type="date"
                  class="value-input"
                  value={prop.value || ""}
                  onblur={(e) => {
                    handleValueChange(prop.key, e.currentTarget.value);
                    stopEditing();
                  }}
                  onkeydown={(e) => {
                    if (e.key === "Enter") {
                      handleValueChange(prop.key, e.currentTarget.value);
                      stopEditing();
                    } else if (e.key === "Escape") {
                      stopEditing();
                    }
                  }}
                />
              {:else if prop.property_type === "number"}
                <input
                  type="number"
                  class="value-input"
                  value={prop.value || ""}
                  onblur={(e) => {
                    handleValueChange(prop.key, e.currentTarget.value);
                    stopEditing();
                  }}
                  onkeydown={(e) => {
                    if (e.key === "Enter") {
                      handleValueChange(prop.key, e.currentTarget.value);
                      stopEditing();
                    } else if (e.key === "Escape") {
                      stopEditing();
                    }
                  }}
                />
              {:else if prop.property_type === "boolean"}
                <select
                  class="value-input"
                  value={prop.value || "false"}
                  onchange={(e) => {
                    handleValueChange(prop.key, e.currentTarget.value);
                    stopEditing();
                  }}
                  onkeydown={(e) => {
                    if (e.key === "Escape") {
                      stopEditing();
                    }
                  }}
                >
                  <option value="true">true</option>
                  <option value="false">false</option>
                </select>
              {:else}
                <input
                  type="text"
                  class="value-input"
                  value={prop.value || ""}
                  placeholder={prop.property_type === "list" ? "comma, separated, values" : ""}
                  onblur={(e) => {
                    handleValueChange(prop.key, e.currentTarget.value);
                    stopEditing();
                  }}
                  onkeydown={(e) => {
                    if (e.key === "Enter") {
                      handleValueChange(prop.key, e.currentTarget.value);
                      stopEditing();
                    } else if (e.key === "Escape") {
                      stopEditing();
                    }
                  }}
                  autocomplete="off"
                  autocorrect="off"
                  autocapitalize="off"
                  spellcheck="false"
                />
              {/if}
            {:else}
              <button
                class="value-display"
                onclick={() => startEditing(prop.key)}
              >
                {#if prop.property_type === "boolean"}
                  <span class="boolean-badge" class:true={prop.value === "true"}>
                    {prop.value || "false"}
                  </span>
                {:else}
                  {prop.value || "—"}
                {/if}
              </button>
            {/if}
          </div>
          <button
            class="delete-btn"
            onclick={() => handleDeleteProperty(prop.key)}
            title="Delete property"
          >
            <X size={14} />
          </button>
        </div>
      {/each}

      {#if isAddingNew}
        <div class="new-property-form">
          <div class="new-property-row">
            <div class="property-key-input">
              <input
                type="text"
                class="key-input"
                placeholder="Property name"
                bind:this={keyInputRef}
                bind:value={newKey}
                onkeydown={handleKeyDown}
                autocomplete="off"
                autocorrect="off"
                autocapitalize="off"
                spellcheck="false"
              />
            </div>
            <div class="property-type-select">
              <select
                class="type-select"
                bind:value={newType}
                onchange={() => {
                  newValue = getDefaultValueForType(newType);
                }}
              >
                {#each PROPERTY_TYPES as ptype}
                  <option value={ptype.value}>{ptype.label}</option>
                {/each}
              </select>
            </div>
          </div>
          <div class="new-property-row">
            <div class="property-value-input">
              {#if newType === "date"}
                <input
                  type="date"
                  class="value-input"
                  bind:value={newValue}
                  onkeydown={handleKeyDown}
                />
              {:else if newType === "number"}
                <input
                  type="number"
                  class="value-input"
                  placeholder="0"
                  bind:value={newValue}
                  onkeydown={handleKeyDown}
                />
              {:else if newType === "boolean"}
                <select
                  class="value-input"
                  bind:value={newValue}
                >
                  <option value="true">true</option>
                  <option value="false">false</option>
                </select>
              {:else if newType === "list"}
                <input
                  type="text"
                  class="value-input"
                  placeholder="comma, separated, values"
                  bind:value={newValue}
                  onkeydown={handleKeyDown}
                  autocomplete="off"
                  autocorrect="off"
                  autocapitalize="off"
                  spellcheck="false"
                />
              {:else}
                <input
                  type="text"
                  class="value-input"
                  placeholder="Value"
                  bind:value={newValue}
                  onkeydown={handleKeyDown}
                  autocomplete="off"
                  autocorrect="off"
                  autocapitalize="off"
                  spellcheck="false"
                />
              {/if}
            </div>
            <div class="new-property-actions">
              <button
                class="confirm-btn"
                onclick={handleAddProperty}
                disabled={!newKey.trim()}
                aria-label="Add property"
              >
                <Check size={14} />
              </button>
              <button
                class="cancel-btn"
                onclick={() => {
                  isAddingNew = false;
                  newKey = "";
                  newValue = "";
                  newType = "text";
                }}
                aria-label="Cancel"
              >
                <X size={14} />
              </button>
            </div>
          </div>
        </div>
      {/if}
    {/if}
  </div>
</div>

<style>
  .properties-panel {
    display: flex;
    flex-direction: column;
    background: var(--panel-bg);
    border-top: 1px solid var(--panel-border);
  }

  /* Schedule Section Styles */
  .schedule-section {
    border-bottom: 1px solid var(--panel-border);
    padding: var(--spacing-3);
  }

  .section-header {
    display: flex;
    align-items: center;
    gap: var(--spacing-2);
    font-size: var(--font-size-sm);
    font-weight: var(--font-weight-semibold);
    color: var(--text-muted);
    text-transform: uppercase;
    letter-spacing: 0.5px;
    margin-bottom: var(--spacing-2);
  }

  .schedule-blocks {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-2);
  }

  .schedule-block-info {
    display: flex;
    align-items: stretch;
    gap: var(--spacing-2);
    padding: var(--spacing-2);
    background: var(--bg-surface-raised);
    border-radius: var(--radius-md);
  }

  .block-color-indicator {
    width: 4px;
    border-radius: 2px;
    background: var(--block-color);
    flex-shrink: 0;
  }

  .block-main {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: var(--spacing-1);
  }

  .block-label {
    font-size: var(--font-size-sm);
    font-weight: var(--font-weight-medium);
    color: var(--text-primary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .block-details {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: var(--spacing-2);
  }

  .block-date {
    font-size: var(--font-size-sm);
    color: var(--text-secondary);
  }

  .date-display-btn {
    font-size: var(--font-size-sm);
    color: var(--text-secondary);
    background: transparent;
    border: none;
    padding: 0;
    border-radius: var(--radius-sm);
    cursor: pointer;
  }

  .date-display-btn:hover {
    background: var(--bg-hover);
  }

  .date-input {
    font-size: var(--font-size-sm);
    padding: var(--spacing-1) var(--spacing-2);
    border: 1px solid var(--input-border);
    border-radius: var(--radius-sm);
    background: var(--input-bg);
    color: var(--input-text);
  }

  .date-input:focus {
    outline: none;
    border-color: var(--input-border-focus);
  }

  .block-time {
    display: flex;
    align-items: center;
    gap: var(--spacing-1);
    font-size: var(--font-size-sm);
    color: var(--text-secondary);
  }

  .block-recurrence {
    display: flex;
    align-items: center;
    gap: var(--spacing-1);
    font-size: var(--font-size-xs);
    color: var(--color-primary);
    background: var(--color-primary-light);
    padding: 2px var(--spacing-2);
    border-radius: var(--radius-sm);
  }

  .block-context {
    font-size: var(--font-size-xs);
    color: var(--text-muted);
    font-style: italic;
  }

  .panel-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: var(--spacing-3);
    border-bottom: 1px solid var(--panel-border);
  }

  .panel-title {
    font-size: var(--font-size-sm);
    font-weight: var(--font-weight-semibold);
    color: var(--text-muted);
    text-transform: uppercase;
    letter-spacing: 0.5px;
    margin: 0;
  }

  .add-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 24px;
    height: 24px;
    border: none;
    background: transparent;
    border-radius: var(--radius-sm);
    color: var(--text-muted);
    cursor: pointer;
  }

  .add-btn:hover {
    background: var(--bg-hover);
    color: var(--text-primary);
  }

  .properties-list {
    padding: var(--spacing-2);
    max-height: 200px;
    overflow-y: auto;
  }

  .loading-state {
    padding: var(--spacing-4);
    text-align: center;
    color: var(--text-muted);
    font-size: var(--font-size-base);
  }

  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    padding: var(--spacing-4);
    text-align: center;
  }

  .empty-state p {
    font-size: var(--font-size-base);
    color: var(--text-muted);
    margin: 0 0 var(--spacing-2) 0;
  }

  .add-first-btn {
    font-size: var(--font-size-sm);
    color: var(--color-primary);
    background: transparent;
    border: none;
    cursor: pointer;
    padding: var(--spacing-1) var(--spacing-2);
    border-radius: var(--radius-sm);
  }

  .add-first-btn:hover {
    background: var(--bg-hover);
  }

  .property-row {
    display: flex;
    align-items: center;
    gap: var(--spacing-2);
    padding: var(--spacing-2) 0;
    border-bottom: 1px solid var(--border-light);
  }

  .property-row:last-child {
    border-bottom: none;
  }

  .property-key {
    flex: 0 0 100px;
    font-size: var(--font-size-sm);
    font-weight: var(--font-weight-medium);
    color: var(--text-muted);
    display: flex;
    align-items: center;
    gap: var(--spacing-1);
  }

  .type-icon-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 2px;
    border: none;
    background: transparent;
    color: var(--text-muted);
    opacity: 0.7;
    cursor: pointer;
    border-radius: var(--radius-sm);
    transition: all 0.15s;
  }

  .type-icon-btn:hover {
    opacity: 1;
    background: var(--bg-hover);
    color: var(--color-primary);
  }

  .type-select-inline {
    padding: 1px 4px;
    font-size: var(--font-size-xs);
    border: 1px solid var(--input-border);
    border-radius: var(--radius-sm);
    background: var(--input-bg);
    color: var(--input-text);
    cursor: pointer;
    max-width: 70px;
  }

  .type-select-inline:focus {
    outline: none;
    border-color: var(--input-border-focus);
  }

  .property-value {
    flex: 1;
    min-width: 0;
  }

  .value-display {
    width: 100%;
    text-align: left;
    font-size: var(--font-size-base);
    color: var(--text-primary);
    background: transparent;
    border: none;
    padding: var(--spacing-1) var(--spacing-2);
    border-radius: var(--radius-sm);
    cursor: pointer;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .value-display:hover {
    background: var(--bg-hover);
  }

  .key-input,
  .value-input {
    width: 100%;
    font-size: var(--font-size-base);
    padding: var(--spacing-1) var(--spacing-2);
    border: 1px solid var(--input-border);
    border-radius: var(--radius-sm);
    background: var(--input-bg);
    color: var(--input-text);
  }

  .key-input:focus,
  .value-input:focus {
    outline: none;
    border-color: var(--input-border-focus);
  }

  .delete-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 24px;
    height: 24px;
    border: none;
    background: transparent;
    border-radius: var(--radius-sm);
    color: var(--text-muted);
    cursor: pointer;
    opacity: 0;
    transition: opacity var(--transition-normal);
  }

  .property-row:hover .delete-btn {
    opacity: 1;
  }

  .delete-btn:hover {
    background: var(--color-error-light);
    color: var(--color-error);
  }

  .new-property-actions {
    display: flex;
    gap: var(--spacing-1);
  }

  .confirm-btn,
  .cancel-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 24px;
    height: 24px;
    border: none;
    border-radius: var(--radius-sm);
    cursor: pointer;
  }

  .confirm-btn {
    background: var(--color-success-light);
    color: var(--color-success);
  }

  .confirm-btn:hover:not(:disabled) {
    background: var(--color-success-hover);
  }

  .confirm-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .cancel-btn {
    background: var(--bg-surface-sunken);
    color: var(--text-muted);
  }

  .cancel-btn:hover {
    background: var(--bg-hover);
  }

  /* New property form styles */
  .new-property-form {
    background: var(--bg-surface-raised);
    border-radius: var(--radius-md);
    padding: var(--spacing-2);
    margin-top: var(--spacing-1);
    display: flex;
    flex-direction: column;
    gap: var(--spacing-2);
  }

  .new-property-row {
    display: flex;
    gap: var(--spacing-2);
    align-items: center;
  }

  .property-key-input {
    flex: 1;
  }

  .property-type-select {
    flex-shrink: 0;
  }

  .type-select {
    padding: var(--spacing-1) var(--spacing-2);
    font-size: var(--font-size-sm);
    border: 1px solid var(--input-border);
    border-radius: var(--radius-sm);
    background: var(--input-bg);
    color: var(--input-text);
    cursor: pointer;
  }

  .type-select:focus {
    outline: none;
    border-color: var(--input-border-focus);
  }

  .property-value-input {
    flex: 1;
  }

  /* Boolean badge */
  .boolean-badge {
    display: inline-block;
    padding: 2px 8px;
    border-radius: var(--radius-sm);
    font-size: var(--font-size-xs);
    font-weight: var(--font-weight-medium);
    background: var(--color-error-light);
    color: var(--color-error);
  }

  .boolean-badge.true {
    background: var(--color-success-light);
    color: var(--color-success);
  }
</style>
