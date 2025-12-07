<script lang="ts">
  import { Plus, X, Check } from "lucide-svelte";
  import { getProperties, setProperty, deleteProperty } from "../services/api";
  import type { PropertyDto } from "../types";

  interface Props {
    noteId: number;
  }

  let { noteId }: Props = $props();

  let properties = $state<PropertyDto[]>([]);
  let isLoading = $state(false);
  let newKey = $state("");
  let newValue = $state("");
  let isAddingNew = $state(false);
  let editingKey = $state<string | null>(null);
  let keyInputRef = $state<HTMLInputElement | null>(null);

  // Auto-focus key input when adding new property
  $effect(() => {
    if (isAddingNew && keyInputRef) {
      keyInputRef.focus();
    }
  });

  // Debug effect
  $effect(() => {
    console.log("[PropertiesPanel] State:", {
      noteId,
      propertiesCount: properties.length,
      isAddingNew,
      isLoading,
    });
  });

  // Fetch properties when noteId changes
  $effect(() => {
    if (noteId) {
      console.log("[PropertiesPanel] noteId changed:", noteId);
      fetchProperties();
    }
  });

  async function fetchProperties() {
    console.log("[PropertiesPanel] Fetching properties for noteId:", noteId);
    isLoading = true;
    try {
      properties = await getProperties(noteId);
      console.log("[PropertiesPanel] Got properties:", properties);
    } catch (e) {
      console.error("[PropertiesPanel] Failed to fetch properties:", e);
    } finally {
      isLoading = false;
    }
  }

  async function handleValueChange(key: string, value: string) {
    try {
      await setProperty({
        note_id: noteId,
        key,
        value,
        property_type: null,
      });
      await fetchProperties();
    } catch (e) {
      console.error("Failed to update property:", e);
    }
  }

  async function handleAddProperty() {
    console.log("[PropertiesPanel] handleAddProperty called", {
      noteId,
      newKey,
      newValue,
    });
    if (newKey.trim()) {
      try {
        const request = {
          note_id: noteId,
          key: newKey.trim(),
          value: newValue.trim() || null,
          property_type: null,
        };
        console.log("[PropertiesPanel] Calling setProperty with:", request);
        const result = await setProperty(request);
        console.log("[PropertiesPanel] setProperty returned:", result);
        newKey = "";
        newValue = "";
        isAddingNew = false;
        await fetchProperties();
      } catch (e) {
        console.error("[PropertiesPanel] Failed to add property:", e);
      }
    } else {
      console.log("[PropertiesPanel] newKey is empty, not adding");
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
    console.log("[PropertiesPanel] startAddingProperty called, current isAddingNew:", isAddingNew);
    isAddingNew = true;
    console.log("[PropertiesPanel] isAddingNew set to:", isAddingNew);
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
</script>

<div class="properties-panel">
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
        <div class="property-row">
          <div class="property-key">
            <span>{prop.key}</span>
          </div>
          <div class="property-value">
            {#if editingKey === prop.key}
              <input
                type="text"
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
            {:else}
              <button
                class="value-display"
                onclick={() => startEditing(prop.key)}
              >
                {prop.value || "—"}
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
        <div class="property-row new-property">
          <div class="property-key">
            <input
              type="text"
              class="key-input"
              placeholder="Key"
              bind:this={keyInputRef}
              bind:value={newKey}
              onkeydown={handleKeyDown}
            />
          </div>
          <div class="property-value">
            <input
              type="text"
              class="value-input"
              placeholder="Value"
              bind:value={newValue}
              onkeydown={handleKeyDown}
            />
          </div>
          <div class="new-property-actions">
            <button
              class="confirm-btn"
              onclick={handleAddProperty}
              disabled={!newKey.trim() || !newValue.trim()}
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
              }}
              aria-label="Cancel"
            >
              <X size={14} />
            </button>
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

  .new-property {
    background: var(--bg-surface-raised);
    border-radius: var(--radius-md);
    padding: var(--spacing-2);
    margin-top: var(--spacing-1);
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
</style>
