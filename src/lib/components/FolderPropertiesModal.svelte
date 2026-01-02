<script lang="ts">
  import { X, Plus, Trash2, FolderOpen } from "lucide-svelte";
  import { getFolderProperties, setFolderProperty, deleteFolderProperty } from "../services/api/properties";
  import type { FolderPropertyDto } from "../types";

  interface Props {
    open: boolean;
    folderPath: string;
    folderName: string;
    onClose: () => void;
  }

  let { open, folderPath, folderName, onClose }: Props = $props();

  let properties = $state<FolderPropertyDto[]>([]);
  let isLoading = $state(false);
  let newKey = $state("");
  let newValue = $state("");
  let isAddingNew = $state(false);
  let editingKey = $state<string | null>(null);
  let keyInputRef = $state<HTMLInputElement | null>(null);

  // Fetch properties when modal opens or folderPath changes
  $effect(() => {
    if (open && folderPath !== undefined) {
      fetchProperties();
    }
  });

  // Auto-focus key input when adding new property
  $effect(() => {
    if (isAddingNew && keyInputRef) {
      keyInputRef.focus();
    }
  });

  async function fetchProperties() {
    isLoading = true;
    try {
      properties = await getFolderProperties(folderPath);
    } catch (e) {
      console.error("[FolderPropertiesModal] Failed to fetch properties:", e);
    } finally {
      isLoading = false;
    }
  }

  async function handleValueChange(key: string, value: string) {
    try {
      await setFolderProperty({
        folder_path: folderPath,
        key,
        value,
        property_type: null,
      });
      await fetchProperties();
    } catch (e) {
      console.error("[FolderPropertiesModal] Failed to update property:", e);
    }
  }

  async function handleAddProperty() {
    if (newKey.trim()) {
      try {
        await setFolderProperty({
          folder_path: folderPath,
          key: newKey.trim(),
          value: newValue.trim() || null,
          property_type: null,
        });
        newKey = "";
        newValue = "";
        isAddingNew = false;
        await fetchProperties();
      } catch (e) {
        console.error("[FolderPropertiesModal] Failed to add property:", e);
      }
    }
  }

  async function handleDeleteProperty(key: string) {
    try {
      await deleteFolderProperty(folderPath, key);
      await fetchProperties();
    } catch (e) {
      console.error("[FolderPropertiesModal] Failed to delete property:", e);
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
      if (newKey.trim()) {
        handleAddProperty();
      }
    } else if (event.key === "Escape") {
      isAddingNew = false;
      newKey = "";
      newValue = "";
    }
  }

  function handleBackdropClick(e: MouseEvent) {
    if (e.target === e.currentTarget) {
      onClose();
    }
  }

  function handleEscape(e: KeyboardEvent) {
    if (e.key === "Escape") {
      onClose();
    }
  }
</script>

{#if open}
  <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
  <div
    class="modal-backdrop"
    role="dialog"
    aria-modal="true"
    aria-labelledby="modal-title"
    tabindex="-1"
    onkeydown={handleEscape}
    onclick={handleBackdropClick}
  >
    <div class="modal-content">
      <div class="modal-header">
        <div class="header-title">
          <FolderOpen size={18} />
          <h2 id="modal-title">
            {folderName || "Root"} Properties
          </h2>
        </div>
        <button class="close-btn" onclick={onClose} aria-label="Close">
          <X size={20} />
        </button>
      </div>

      <div class="modal-body">
        <p class="info-text">
          Properties set here will be inherited by all notes in this folder and subfolders.
        </p>

        {#if isLoading}
          <div class="loading">Loading...</div>
        {:else}
          <div class="properties-list">
            {#each properties as prop (prop.id)}
              <div class="property-row">
                <span class="property-key">{prop.key}</span>
                {#if editingKey === prop.key}
                  <input
                    type="text"
                    class="property-value-input"
                    value={prop.value ?? ""}
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
                {:else}
                  <button
                    class="property-value"
                    onclick={() => startEditing(prop.key)}
                  >
                    {prop.value || "(empty)"}
                  </button>
                {/if}
                <button
                  class="delete-btn"
                  onclick={() => handleDeleteProperty(prop.key)}
                  aria-label="Delete property"
                >
                  <Trash2 size={14} />
                </button>
              </div>
            {/each}

            {#if properties.length === 0 && !isAddingNew}
              <p class="empty-message">No properties set for this folder.</p>
            {/if}

            {#if isAddingNew}
              <div class="property-row new-property">
                <input
                  bind:this={keyInputRef}
                  type="text"
                  class="property-key-input"
                  placeholder="Key"
                  bind:value={newKey}
                  onkeydown={handleKeyDown}
                  autocomplete="off"
                  autocorrect="off"
                  autocapitalize="off"
                  spellcheck="false"
                />
                <input
                  type="text"
                  class="property-value-input"
                  placeholder="Value"
                  bind:value={newValue}
                  onkeydown={handleKeyDown}
                  autocomplete="off"
                  autocorrect="off"
                  autocapitalize="off"
                  spellcheck="false"
                />
                <button
                  class="cancel-add-btn"
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
            {/if}
          </div>

          <button class="add-property-btn" onclick={startAddingProperty}>
            <Plus size={14} />
            Add Property
          </button>
        {/if}
      </div>

      <div class="modal-footer">
        <button class="done-btn" onclick={onClose}>Done</button>
      </div>
    </div>
  </div>
{/if}

<style>
  .modal-backdrop {
    position: fixed;
    inset: 0;
    background: var(--modal-backdrop-bg);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: var(--z-modal-backdrop);
  }

  .modal-content {
    background: var(--modal-bg);
    border-radius: var(--radius-xl);
    width: 100%;
    max-width: 480px;
    max-height: 80vh;
    overflow: hidden;
    display: flex;
    flex-direction: column;
    box-shadow: var(--modal-shadow);
    z-index: var(--z-modal);
  }

  .modal-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: var(--spacing-4) var(--spacing-5);
    border-bottom: 1px solid var(--modal-border);
  }

  .header-title {
    display: flex;
    align-items: center;
    gap: var(--spacing-2);
    color: var(--text-primary);
  }

  .header-title h2 {
    font-size: var(--font-size-lg);
    font-weight: var(--font-weight-semibold);
    margin: 0;
  }

  .close-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 32px;
    height: 32px;
    border: none;
    background: transparent;
    border-radius: var(--radius-md);
    color: var(--text-muted);
    cursor: pointer;
  }

  .close-btn:hover {
    background: var(--bg-hover);
    color: var(--text-primary);
  }

  .modal-body {
    padding: var(--spacing-5);
    overflow-y: auto;
    flex: 1;
  }

  .info-text {
    font-size: var(--font-size-sm);
    color: var(--text-muted);
    margin: 0 0 var(--spacing-4) 0;
    padding: var(--spacing-3);
    background: var(--bg-surface-raised);
    border-radius: var(--radius-md);
  }

  .loading {
    text-align: center;
    color: var(--text-muted);
    padding: var(--spacing-4);
  }

  .properties-list {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-2);
    margin-bottom: var(--spacing-4);
  }

  .property-row {
    display: flex;
    align-items: center;
    gap: var(--spacing-2);
    padding: var(--spacing-2);
    background: var(--bg-surface);
    border-radius: var(--radius-md);
  }

  .property-key {
    font-weight: var(--font-weight-medium);
    color: var(--text-muted);
    min-width: 100px;
    font-size: var(--font-size-sm);
  }

  .property-value {
    flex: 1;
    text-align: left;
    padding: var(--spacing-1) var(--spacing-2);
    border: none;
    background: transparent;
    color: var(--text-primary);
    cursor: pointer;
    border-radius: var(--radius-sm);
    font-size: var(--font-size-sm);
  }

  .property-value:hover {
    background: var(--bg-hover);
  }

  .property-key-input,
  .property-value-input {
    flex: 1;
    padding: var(--spacing-2);
    border: 1px solid var(--input-border);
    border-radius: var(--radius-sm);
    background: var(--input-bg);
    color: var(--input-text);
    font-size: var(--font-size-sm);
  }

  .property-key-input:focus,
  .property-value-input:focus {
    outline: none;
    border-color: var(--input-border-focus);
    box-shadow: var(--shadow-focus);
  }

  .property-key-input {
    max-width: 120px;
  }

  .delete-btn,
  .cancel-add-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 28px;
    height: 28px;
    border: none;
    background: transparent;
    color: var(--text-muted);
    cursor: pointer;
    border-radius: var(--radius-sm);
  }

  .delete-btn:hover {
    background: var(--color-error-light);
    color: var(--color-error);
  }

  .cancel-add-btn:hover {
    background: var(--bg-hover);
  }

  .empty-message {
    text-align: center;
    color: var(--text-muted);
    font-size: var(--font-size-sm);
    padding: var(--spacing-4);
  }

  .add-property-btn {
    display: flex;
    align-items: center;
    gap: var(--spacing-1);
    padding: var(--spacing-2) var(--spacing-3);
    border: 1px dashed var(--input-border);
    background: transparent;
    color: var(--text-muted);
    cursor: pointer;
    border-radius: var(--radius-md);
    font-size: var(--font-size-sm);
    width: 100%;
    justify-content: center;
  }

  .add-property-btn:hover {
    background: var(--bg-hover);
    border-color: var(--text-muted);
    color: var(--text-primary);
  }

  .modal-footer {
    display: flex;
    justify-content: flex-end;
    padding: var(--spacing-4) var(--spacing-5);
    border-top: 1px solid var(--modal-border);
  }

  .done-btn {
    padding: var(--spacing-2) var(--spacing-5);
    border: none;
    background: var(--btn-primary-bg);
    color: var(--btn-primary-text);
    cursor: pointer;
    border-radius: var(--radius-md);
    font-size: var(--font-size-sm);
    font-weight: var(--font-weight-medium);
  }

  .done-btn:hover {
    background: var(--btn-primary-bg-hover);
  }

  .new-property {
    background: var(--bg-surface-raised);
  }
</style>
