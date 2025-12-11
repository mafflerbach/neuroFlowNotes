<!--
  Properties Editor Component
  Manages property keys and values across the vault.
  Three-column layout: Keys | Values | Notes using the property
-->
<script lang="ts">
  import { getPropertyKeys } from "../services/api/query";
  import {
    getPropertyValuesWithCounts,
    getNotesWithProperty,
    getNotesWithPropertyValue,
    renamePropertyKey,
    renamePropertyValue,
    mergePropertyKeys,
    deletePropertyKey,
  } from "../services/api/properties";
  import type { PropertyKeyInfo, PropertyValueInfo, NoteWithPropertyValue } from "../types";

  // State
  let propertyKeys = $state<PropertyKeyInfo[]>([]);
  let selectedKey = $state<string | null>(null);
  let propertyValues = $state<PropertyValueInfo[]>([]);
  let selectedValue = $state<string | null>(null);
  let notesUsingProperty = $state<NoteWithPropertyValue[]>([]);

  let loading = $state(true);
  let loadingValues = $state(false);
  let loadingNotes = $state(false);
  let message = $state<{ type: "success" | "error"; text: string } | null>(null);

  // Edit state
  let editingKey = $state<string | null>(null);
  let editKeyValue = $state("");
  let editingValue = $state<string | null>(null);
  let editValueValue = $state("");

  // Merge state
  let mergeMode = $state(false);
  let mergeSourceKey = $state<string | null>(null);
  let mergeTargetKey = $state<string | null>(null);

  // Load property keys on mount
  $effect(() => {
    loadPropertyKeys();
  });

  // Load values when key is selected
  $effect(() => {
    if (selectedKey) {
      loadPropertyValues(selectedKey);
      selectedValue = null;
    } else {
      propertyValues = [];
      selectedValue = null;
    }
  });

  // Load notes when key or value changes
  $effect(() => {
    if (selectedKey) {
      if (selectedValue) {
        loadNotesWithValue(selectedKey, selectedValue);
      } else {
        loadNotesWithKey(selectedKey);
      }
    } else {
      notesUsingProperty = [];
    }
  });

  async function loadPropertyKeys() {
    loading = true;
    try {
      propertyKeys = await getPropertyKeys();
    } catch (e) {
      console.error("Failed to load property keys:", e);
      showMessage("error", "Failed to load properties");
    } finally {
      loading = false;
    }
  }

  async function loadPropertyValues(key: string) {
    loadingValues = true;
    try {
      propertyValues = await getPropertyValuesWithCounts(key);
    } catch (e) {
      console.error("Failed to load property values:", e);
      showMessage("error", "Failed to load values");
    } finally {
      loadingValues = false;
    }
  }

  async function loadNotesWithKey(key: string) {
    loadingNotes = true;
    try {
      notesUsingProperty = await getNotesWithProperty(key);
    } catch (e) {
      console.error("Failed to load notes:", e);
    } finally {
      loadingNotes = false;
    }
  }

  async function loadNotesWithValue(key: string, value: string) {
    loadingNotes = true;
    try {
      notesUsingProperty = await getNotesWithPropertyValue(key, value);
    } catch (e) {
      console.error("Failed to load notes:", e);
    } finally {
      loadingNotes = false;
    }
  }

  function showMessage(type: "success" | "error", text: string) {
    message = { type, text };
    setTimeout(() => {
      message = null;
    }, 3000);
  }

  // Key operations
  function startEditKey(key: string) {
    editingKey = key;
    editKeyValue = key;
  }

  function cancelEditKey() {
    editingKey = null;
    editKeyValue = "";
  }

  async function saveEditKey() {
    if (!editingKey || !editKeyValue.trim()) return;
    if (editingKey === editKeyValue) {
      cancelEditKey();
      return;
    }

    try {
      const result = await renamePropertyKey({
        old_key: editingKey,
        new_key: editKeyValue.trim(),
      });
      showMessage(
        "success",
        `Renamed "${editingKey}" to "${editKeyValue.trim()}" (${result.notes_affected} notes)`
      );
      if (selectedKey === editingKey) {
        selectedKey = editKeyValue.trim();
      }
      cancelEditKey();
      await loadPropertyKeys();
    } catch (e) {
      console.error("Failed to rename key:", e);
      showMessage("error", "Failed to rename property key");
    }
  }

  async function handleDeleteKey(key: string) {
    if (!confirm(`Delete property "${key}" from all notes? This cannot be undone.`)) {
      return;
    }

    try {
      const result = await deletePropertyKey({ key });
      showMessage("success", `Deleted "${key}" from ${result.notes_affected} notes`);
      if (selectedKey === key) {
        selectedKey = null;
      }
      await loadPropertyKeys();
    } catch (e) {
      console.error("Failed to delete key:", e);
      showMessage("error", "Failed to delete property key");
    }
  }

  // Value operations
  function startEditValue(value: string) {
    editingValue = value;
    editValueValue = value;
  }

  function cancelEditValue() {
    editingValue = null;
    editValueValue = "";
  }

  async function saveEditValue() {
    if (!selectedKey || !editingValue || !editValueValue.trim()) return;
    if (editingValue === editValueValue) {
      cancelEditValue();
      return;
    }

    try {
      const result = await renamePropertyValue({
        key: selectedKey,
        old_value: editingValue,
        new_value: editValueValue.trim(),
      });
      showMessage(
        "success",
        `Renamed "${editingValue}" to "${editValueValue.trim()}" (${result.notes_affected} notes)`
      );
      if (selectedValue === editingValue) {
        selectedValue = editValueValue.trim();
      }
      cancelEditValue();
      await loadPropertyValues(selectedKey);
    } catch (e) {
      console.error("Failed to rename value:", e);
      showMessage("error", "Failed to rename property value");
    }
  }

  // Merge operations
  function startMerge() {
    mergeMode = true;
    mergeSourceKey = null;
    mergeTargetKey = null;
  }

  function cancelMerge() {
    mergeMode = false;
    mergeSourceKey = null;
    mergeTargetKey = null;
  }

  function selectForMerge(key: string) {
    if (!mergeSourceKey) {
      mergeSourceKey = key;
    } else if (key !== mergeSourceKey) {
      mergeTargetKey = key;
    }
  }

  async function executeMerge() {
    if (!mergeSourceKey || !mergeTargetKey) return;

    if (
      !confirm(
        `Merge "${mergeSourceKey}" into "${mergeTargetKey}"?\n\nAll values from "${mergeSourceKey}" will be moved to "${mergeTargetKey}", and "${mergeSourceKey}" will be deleted.`
      )
    ) {
      return;
    }

    try {
      const result = await mergePropertyKeys({
        source_key: mergeSourceKey,
        target_key: mergeTargetKey,
      });
      showMessage(
        "success",
        `Merged "${mergeSourceKey}" into "${mergeTargetKey}" (${result.notes_affected} notes)`
      );
      if (selectedKey === mergeSourceKey) {
        selectedKey = mergeTargetKey;
      }
      cancelMerge();
      await loadPropertyKeys();
    } catch (e) {
      console.error("Failed to merge keys:", e);
      showMessage("error", "Failed to merge property keys");
    }
  }

  function handleKeydown(e: KeyboardEvent, type: "key" | "value") {
    if (e.key === "Enter") {
      e.preventDefault();
      if (type === "key") {
        saveEditKey();
      } else {
        saveEditValue();
      }
    } else if (e.key === "Escape") {
      e.preventDefault();
      if (type === "key") {
        cancelEditKey();
      } else {
        cancelEditValue();
      }
    }
  }

  function getDisplayTitle(note: NoteWithPropertyValue): string {
    if (note.title) return note.title;
    // Extract filename without extension from path
    const parts = note.path.split("/");
    const filename = parts[parts.length - 1];
    return filename.replace(/\.md$/, "");
  }
</script>

<div class="properties-editor">
  <!-- Header with message -->
  {#if message}
    <div class="message message-{message.type}">
      {message.text}
    </div>
  {/if}

  <div class="columns">
    <!-- Property Keys Column -->
    <div class="column keys-column">
      <div class="column-header">
        <h4>Property Keys</h4>
        <div class="header-actions">
          {#if mergeMode}
            <button class="btn btn-sm btn-ghost" onclick={cancelMerge}>Cancel</button>
            {#if mergeSourceKey && mergeTargetKey}
              <button class="btn btn-sm btn-primary" onclick={executeMerge}>Merge</button>
            {/if}
          {:else}
            <button class="btn btn-sm btn-ghost" onclick={startMerge} title="Merge two property keys">
              Merge
            </button>
          {/if}
        </div>
      </div>

      {#if mergeMode}
        <div class="merge-help">
          {#if !mergeSourceKey}
            Click the key to merge FROM
          {:else if !mergeTargetKey}
            Click the key to merge INTO
          {:else}
            "{mergeSourceKey}" will be merged into "{mergeTargetKey}"
          {/if}
        </div>
      {/if}

      <div class="list-container">
        {#if loading}
          <div class="loading">Loading...</div>
        {:else if propertyKeys.length === 0}
          <div class="empty">No properties found</div>
        {:else}
          <ul class="property-list">
            {#each propertyKeys as keyInfo}
              <li
                class="property-item"
                class:selected={selectedKey === keyInfo.key && !mergeMode}
                class:merge-source={mergeMode && mergeSourceKey === keyInfo.key}
                class:merge-target={mergeMode && mergeTargetKey === keyInfo.key}
              >
                {#if editingKey === keyInfo.key}
                  <input
                    type="text"
                    class="edit-input"
                    bind:value={editKeyValue}
                    onkeydown={(e) => handleKeydown(e, "key")}
                  />
                  <div class="item-actions">
                    <button class="btn btn-icon" onclick={saveEditKey} title="Save">&#10003;</button>
                    <button class="btn btn-icon" onclick={cancelEditKey} title="Cancel">&#10005;</button>
                  </div>
                {:else}
                  <button
                    class="item-content"
                    onclick={() => mergeMode ? selectForMerge(keyInfo.key) : (selectedKey = keyInfo.key)}
                  >
                    <span class="item-name">{keyInfo.key}</span>
                    <span class="item-count">{keyInfo.usage_count}</span>
                  </button>
                  {#if !mergeMode}
                    <div class="item-actions">
                      <button
                        class="btn btn-icon"
                        onclick={(e) => { e.stopPropagation(); startEditKey(keyInfo.key); }}
                        title="Rename"
                      >&#9998;</button>
                      <button
                        class="btn btn-icon btn-danger"
                        onclick={(e) => { e.stopPropagation(); handleDeleteKey(keyInfo.key); }}
                        title="Delete"
                      >&#10005;</button>
                    </div>
                  {/if}
                {/if}
              </li>
            {/each}
          </ul>
        {/if}
      </div>
    </div>

    <!-- Property Values Column -->
    <div class="column values-column">
      <div class="column-header">
        <h4>Values</h4>
        {#if selectedKey}
          <span class="header-context">{selectedKey}</span>
        {/if}
      </div>

      <div class="list-container">
        {#if !selectedKey}
          <div class="empty">Select a key</div>
        {:else if loadingValues}
          <div class="loading">Loading...</div>
        {:else if propertyValues.length === 0}
          <div class="empty">No values</div>
        {:else}
          <ul class="property-list">
            <!-- "All values" option -->
            <li
              class="property-item"
              class:selected={selectedValue === null}
            >
              <button class="item-content" onclick={() => (selectedValue = null)}>
                <span class="item-name italic">All values</span>
                <span class="item-count">{notesUsingProperty.length}</span>
              </button>
            </li>
            {#each propertyValues as valueInfo}
              <li
                class="property-item"
                class:selected={selectedValue === valueInfo.value}
              >
                {#if editingValue === valueInfo.value}
                  <input
                    type="text"
                    class="edit-input"
                    bind:value={editValueValue}
                    onkeydown={(e) => handleKeydown(e, "value")}
                  />
                  <div class="item-actions">
                    <button class="btn btn-icon" onclick={saveEditValue} title="Save">&#10003;</button>
                    <button class="btn btn-icon" onclick={cancelEditValue} title="Cancel">&#10005;</button>
                  </div>
                {:else}
                  <button class="item-content" onclick={() => (selectedValue = valueInfo.value)}>
                    <span class="item-name">{valueInfo.value}</span>
                    <span class="item-count">{valueInfo.usage_count}</span>
                  </button>
                  <div class="item-actions">
                    <button
                      class="btn btn-icon"
                      onclick={(e) => { e.stopPropagation(); startEditValue(valueInfo.value); }}
                      title="Rename this value"
                    >&#9998;</button>
                  </div>
                {/if}
              </li>
            {/each}
          </ul>
        {/if}
      </div>
    </div>

    <!-- Notes Using Property Column -->
    <div class="column notes-column">
      <div class="column-header">
        <h4>Notes</h4>
        {#if selectedKey}
          <span class="header-context">
            {selectedKey}{selectedValue ? ` = "${selectedValue}"` : ""}
          </span>
        {/if}
      </div>

      <div class="list-container">
        {#if !selectedKey}
          <div class="empty">Select a key to see notes</div>
        {:else if loadingNotes}
          <div class="loading">Loading...</div>
        {:else if notesUsingProperty.length === 0}
          <div class="empty">No notes found</div>
        {:else}
          <ul class="notes-list">
            {#each notesUsingProperty as note}
              <li class="note-item">
                <div class="note-info">
                  <span class="note-title">{getDisplayTitle(note)}</span>
                  <span class="note-path">{note.path}</span>
                </div>
                {#if note.value && !selectedValue}
                  <span class="note-value">{note.value}</span>
                {/if}
              </li>
            {/each}
          </ul>
        {/if}
      </div>
    </div>
  </div>
</div>

<style>
  .properties-editor {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-3);
    min-height: 400px;
    height: 450px;
  }

  .message {
    padding: var(--spacing-2) var(--spacing-3);
    border-radius: var(--radius-md);
    font-size: var(--font-size-sm);
  }

  .message-success {
    background: var(--color-success-light);
    color: var(--color-success);
  }

  .message-error {
    background: var(--color-error-light);
    color: var(--color-error);
  }

  .columns {
    display: flex;
    gap: var(--spacing-3);
    flex: 1;
    min-height: 0;
  }

  .column {
    flex: 1;
    display: flex;
    flex-direction: column;
    min-width: 0;
    border: 1px solid var(--border-default);
    border-radius: var(--radius-md);
    overflow: hidden;
  }

  .keys-column {
    flex: 0.8;
  }

  .values-column {
    flex: 0.8;
  }

  .notes-column {
    flex: 1.4;
  }

  .column-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: var(--spacing-2);
    padding: var(--spacing-2) var(--spacing-3);
    background: var(--bg-surface-raised);
    border-bottom: 1px solid var(--border-light);
    flex-shrink: 0;
  }

  .column-header h4 {
    margin: 0;
    font-size: var(--font-size-sm);
    font-weight: var(--font-weight-semibold);
    color: var(--text-primary);
  }

  .header-context {
    font-size: var(--font-size-xs);
    color: var(--text-muted);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .header-actions {
    display: flex;
    gap: var(--spacing-1);
    flex-shrink: 0;
  }

  .merge-help {
    padding: var(--spacing-2) var(--spacing-3);
    background: var(--color-info-light);
    color: var(--text-primary);
    font-size: var(--font-size-xs);
    border-bottom: 1px solid var(--border-light);
    flex-shrink: 0;
  }

  .list-container {
    flex: 1;
    overflow-y: auto;
    background: var(--bg-surface);
  }

  .loading,
  .empty {
    padding: var(--spacing-4);
    text-align: center;
    color: var(--text-muted);
    font-size: var(--font-size-sm);
  }

  .property-list,
  .notes-list {
    list-style: none;
    margin: 0;
    padding: 0;
  }

  .property-item,
  .note-item {
    display: flex;
    align-items: center;
    gap: var(--spacing-2);
    padding: var(--spacing-1) var(--spacing-2);
    border-bottom: 1px solid var(--border-light);
  }

  .property-item:last-child,
  .note-item:last-child {
    border-bottom: none;
  }

  .property-item:hover,
  .note-item:hover {
    background: var(--bg-hover);
  }

  .property-item.selected {
    background: var(--bg-selected);
  }

  .property-item.merge-source {
    background: var(--color-warning-light);
  }

  .property-item.merge-target {
    background: var(--color-success-light);
  }

  .item-content {
    flex: 1;
    display: flex;
    justify-content: space-between;
    align-items: center;
    min-width: 0;
    padding: var(--spacing-1) 0;
    background: none;
    border: none;
    cursor: pointer;
    text-align: left;
    color: inherit;
    font: inherit;
  }

  .item-name {
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    color: var(--text-primary);
    font-size: var(--font-size-sm);
  }

  .item-name.italic {
    font-style: italic;
    color: var(--text-muted);
  }

  .item-count {
    flex-shrink: 0;
    padding: 0 var(--spacing-1);
    font-size: var(--font-size-xs);
    color: var(--text-muted);
  }

  .item-actions {
    display: flex;
    gap: var(--spacing-1);
    opacity: 0;
    transition: opacity var(--transition-fast);
  }

  .property-item:hover .item-actions {
    opacity: 1;
  }

  .edit-input {
    flex: 1;
    padding: var(--spacing-1) var(--spacing-2);
    background: var(--input-bg);
    border: 1px solid var(--input-border-focus);
    border-radius: var(--radius-sm);
    color: var(--input-text);
    font-size: var(--font-size-sm);
  }

  .edit-input:focus {
    outline: none;
  }

  /* Note item specific styles */
  .note-item {
    padding: var(--spacing-2) var(--spacing-3);
  }

  .note-info {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .note-title {
    font-size: var(--font-size-sm);
    color: var(--text-primary);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .note-path {
    font-size: var(--font-size-xs);
    color: var(--text-muted);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .note-value {
    flex-shrink: 0;
    padding: var(--spacing-1) var(--spacing-2);
    background: var(--bg-surface-raised);
    border-radius: var(--radius-sm);
    font-size: var(--font-size-xs);
    color: var(--text-secondary);
    max-width: 120px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  /* Buttons */
  .btn {
    padding: var(--spacing-1) var(--spacing-2);
    font-size: var(--font-size-sm);
    border-radius: var(--radius-sm);
    cursor: pointer;
    transition: background var(--transition-fast);
  }

  .btn-sm {
    padding: 2px var(--spacing-2);
    font-size: var(--font-size-xs);
  }

  .btn-icon {
    width: 22px;
    height: 22px;
    padding: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    background: transparent;
    border: none;
    color: var(--text-muted);
    font-size: var(--font-size-sm);
  }

  .btn-icon:hover {
    color: var(--text-primary);
    background: var(--bg-hover);
  }

  .btn-icon.btn-danger:hover {
    color: var(--color-error);
    background: var(--color-error-light);
  }

  .btn-ghost {
    background: transparent;
    border: none;
    color: var(--text-muted);
  }

  .btn-ghost:hover {
    color: var(--text-primary);
    background: var(--bg-hover);
  }

  .btn-primary {
    background: var(--btn-primary-bg);
    border: none;
    color: var(--btn-primary-text);
  }

  .btn-primary:hover {
    background: var(--btn-primary-bg-hover);
  }
</style>
