<script lang="ts">
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
  let keyInputRef: HTMLInputElement | null = null;

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
      <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M12 5v14M5 12h14" />
      </svg>
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
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M18 6L6 18M6 6l12 12" />
            </svg>
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
            >
              <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <polyline points="20 6 9 17 4 12" />
              </svg>
            </button>
            <button
              class="cancel-btn"
              onclick={() => {
                isAddingNew = false;
                newKey = "";
                newValue = "";
              }}
            >
              <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M18 6L6 18M6 6l12 12" />
              </svg>
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
    background: var(--panel-bg, #fff);
    border-top: 1px solid var(--border-color, #e0e0e0);
  }

  .panel-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 10px 12px;
    border-bottom: 1px solid var(--border-color, #e0e0e0);
  }

  .panel-title {
    font-size: 12px;
    font-weight: 600;
    color: var(--text-muted, #666);
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
    border-radius: 4px;
    color: var(--text-muted, #666);
    cursor: pointer;
  }

  .add-btn:hover {
    background: var(--hover-bg, #f0f0f0);
    color: var(--text-color, #333);
  }

  .properties-list {
    padding: 8px;
    max-height: 200px;
    overflow-y: auto;
  }

  .loading-state {
    padding: 16px;
    text-align: center;
    color: var(--text-muted, #666);
    font-size: 13px;
  }

  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    padding: 16px;
    text-align: center;
  }

  .empty-state p {
    font-size: 13px;
    color: var(--text-muted, #999);
    margin: 0 0 8px 0;
  }

  .add-first-btn {
    font-size: 12px;
    color: var(--primary-color, #4f6bed);
    background: transparent;
    border: none;
    cursor: pointer;
    padding: 4px 8px;
    border-radius: 4px;
  }

  .add-first-btn:hover {
    background: var(--hover-bg, #f0f0f0);
  }

  .property-row {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 6px 0;
    border-bottom: 1px solid var(--border-light, #f0f0f0);
  }

  .property-row:last-child {
    border-bottom: none;
  }

  .property-key {
    flex: 0 0 100px;
    font-size: 12px;
    font-weight: 500;
    color: var(--text-muted, #666);
  }

  .property-value {
    flex: 1;
    min-width: 0;
  }

  .value-display {
    width: 100%;
    text-align: left;
    font-size: 13px;
    color: var(--text-color, #333);
    background: transparent;
    border: none;
    padding: 4px 8px;
    border-radius: 4px;
    cursor: pointer;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .value-display:hover {
    background: var(--hover-bg, #f5f5f5);
  }

  .key-input,
  .value-input {
    width: 100%;
    font-size: 13px;
    padding: 4px 8px;
    border: 1px solid var(--border-color, #e0e0e0);
    border-radius: 4px;
    background: var(--input-bg, #fff);
    color: var(--text-color, #333);
  }

  .key-input:focus,
  .value-input:focus {
    outline: none;
    border-color: var(--primary-color, #4f6bed);
  }

  .delete-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 24px;
    height: 24px;
    border: none;
    background: transparent;
    border-radius: 4px;
    color: var(--text-muted, #999);
    cursor: pointer;
    opacity: 0;
    transition: opacity 0.15s;
  }

  .property-row:hover .delete-btn {
    opacity: 1;
  }

  .delete-btn:hover {
    background: var(--error-bg, #fee);
    color: var(--error-color, #d32f2f);
  }

  .new-property {
    background: var(--new-property-bg, #f8f9fa);
    border-radius: 6px;
    padding: 8px;
    margin-top: 4px;
  }

  .new-property-actions {
    display: flex;
    gap: 4px;
  }

  .confirm-btn,
  .cancel-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 24px;
    height: 24px;
    border: none;
    border-radius: 4px;
    cursor: pointer;
  }

  .confirm-btn {
    background: var(--success-bg, #e8f5e9);
    color: var(--success-color, #2e7d32);
  }

  .confirm-btn:hover:not(:disabled) {
    background: var(--success-hover-bg, #c8e6c9);
  }

  .confirm-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .cancel-btn {
    background: var(--cancel-bg, #f5f5f5);
    color: var(--text-muted, #666);
  }

  .cancel-btn:hover {
    background: var(--hover-bg, #e0e0e0);
  }
</style>
