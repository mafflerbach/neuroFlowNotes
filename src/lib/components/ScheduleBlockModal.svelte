<script lang="ts">
  import { BLOCK_COLORS, DEFAULT_BLOCK_COLOR, getBlockColor } from "../constants/colors";
  import type { ScheduleBlockDto } from "../types";

  interface Props {
    open: boolean;
    mode: "create" | "edit";
    date: string; // YYYY-MM-DD
    initialHour?: number;
    block?: ScheduleBlockDto | null;
    onSave: (data: {
      date: string;
      start_time: string;
      end_time: string;
      label: string | null;
      color: string;
      context: string | null;
    }) => void;
    onDelete?: () => void;
    onClose: () => void;
  }

  let {
    open,
    mode,
    date,
    initialHour = 9,
    block = null,
    onSave,
    onDelete,
    onClose,
  }: Props = $props();

  // Form state
  let startTime = $state("09:00");
  let endTime = $state("10:00");
  let label = $state("");
  let selectedColor = $state(DEFAULT_BLOCK_COLOR.hex);
  let context = $state("");

  // Initialize form when block changes or modal opens
  $effect(() => {
    if (open) {
      if (mode === "edit" && block) {
        startTime = block.start_time.slice(0, 5);
        endTime = block.end_time.slice(0, 5);
        label = block.label || "";
        selectedColor = block.color || DEFAULT_BLOCK_COLOR.hex;
        context = block.context || "";
      } else {
        // Create mode
        startTime = `${initialHour.toString().padStart(2, "0")}:00`;
        endTime = `${(initialHour + 1).toString().padStart(2, "0")}:00`;
        label = "";
        selectedColor = DEFAULT_BLOCK_COLOR.hex;
        context = "";
      }
    }
  });

  function handleSubmit(e: Event) {
    e.preventDefault();

    onSave({
      date,
      start_time: startTime + ":00",
      end_time: endTime + ":00",
      label: label.trim() || null,
      color: selectedColor,
      context: context.trim() || null,
    });
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Escape") {
      onClose();
    }
  }

  function handleBackdropClick(e: MouseEvent) {
    if (e.target === e.currentTarget) {
      onClose();
    }
  }

  // Format date for display
  function formatDisplayDate(dateStr: string): string {
    const d = new Date(dateStr + "T00:00:00");
    return d.toLocaleDateString("en-US", {
      weekday: "long",
      month: "short",
      day: "numeric",
    });
  }

  // Get the current color object
  const currentColorObj = $derived(getBlockColor(selectedColor));
</script>

{#if open}
  <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
  <div
    class="modal-backdrop"
    role="dialog"
    aria-modal="true"
    aria-labelledby="modal-title"
    onkeydown={handleKeydown}
    onclick={handleBackdropClick}
  >
    <div class="modal-content">
      <div class="modal-header">
        <h2 id="modal-title">
          {mode === "create" ? "New Schedule Block" : "Edit Schedule Block"}
        </h2>
        <button class="close-btn" onclick={onClose} aria-label="Close">
          <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M18 6L6 18M6 6l12 12" />
          </svg>
        </button>
      </div>

      <form onsubmit={handleSubmit}>
        <div class="modal-body">
          <!-- Date display -->
          <div class="date-display">
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <rect x="3" y="4" width="18" height="18" rx="2" ry="2" />
              <line x1="16" y1="2" x2="16" y2="6" />
              <line x1="8" y1="2" x2="8" y2="6" />
              <line x1="3" y1="10" x2="21" y2="10" />
            </svg>
            <span>{formatDisplayDate(date)}</span>
          </div>

          <!-- Time range -->
          <div class="form-row">
            <div class="form-group">
              <label for="start-time">Start</label>
              <input
                id="start-time"
                type="time"
                bind:value={startTime}
                required
              />
            </div>
            <div class="form-group">
              <label for="end-time">End</label>
              <input
                id="end-time"
                type="time"
                bind:value={endTime}
                required
              />
            </div>
          </div>

          <!-- Label -->
          <div class="form-group">
            <label for="block-label">Label (optional)</label>
            <input
              id="block-label"
              type="text"
              bind:value={label}
              placeholder="Meeting, Focus time, etc."
            />
          </div>

          <!-- Context -->
          <div class="form-group">
            <label for="block-context">Context (optional)</label>
            <input
              id="block-context"
              type="text"
              bind:value={context}
              placeholder="Additional details..."
            />
          </div>

          <!-- Color picker -->
          <div class="form-group">
            <label>Color</label>
            <div class="color-picker">
              {#each BLOCK_COLORS as color (color.hex)}
                <button
                  type="button"
                  class="color-swatch"
                  class:selected={selectedColor === color.hex}
                  style="background-color: {color.hex}"
                  onclick={() => (selectedColor = color.hex)}
                  title={color.name}
                  aria-label={color.name}
                >
                  {#if selectedColor === color.hex}
                    <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke={color.textColor} stroke-width="3">
                      <polyline points="20 6 9 17 4 12" />
                    </svg>
                  {/if}
                </button>
              {/each}
            </div>
          </div>

          <!-- Preview -->
          <div class="preview-section">
            <label>Preview</label>
            <div
              class="block-preview"
              style="background-color: {selectedColor}; color: {currentColorObj.textColor}"
            >
              <span class="preview-time">{startTime} - {endTime}</span>
              {#if label}
                <span class="preview-label">{label}</span>
              {/if}
              {#if context}
                <span class="preview-context">{context}</span>
              {/if}
            </div>
          </div>
        </div>

        <div class="modal-footer">
          {#if mode === "edit" && onDelete}
            <button type="button" class="delete-btn" onclick={onDelete}>
              Delete
            </button>
          {/if}
          <div class="footer-actions">
            <button type="button" class="cancel-btn" onclick={onClose}>
              Cancel
            </button>
            <button type="submit" class="save-btn">
              {mode === "create" ? "Create" : "Save"}
            </button>
          </div>
        </div>
      </form>
    </div>
  </div>
{/if}

<style>
  .modal-backdrop {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background: rgba(0, 0, 0, 0.5);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
  }

  .modal-content {
    background: var(--modal-bg, #fff);
    border-radius: 12px;
    width: 100%;
    max-width: 420px;
    max-height: 90vh;
    overflow: hidden;
    display: flex;
    flex-direction: column;
    box-shadow: 0 20px 60px rgba(0, 0, 0, 0.3);
  }

  .modal-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 16px 20px;
    border-bottom: 1px solid var(--border-color, #e0e0e0);
  }

  .modal-header h2 {
    font-size: 18px;
    font-weight: 600;
    margin: 0;
    color: var(--text-color, #333);
  }

  .close-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 32px;
    height: 32px;
    border: none;
    background: transparent;
    border-radius: 6px;
    color: var(--text-muted, #666);
    cursor: pointer;
  }

  .close-btn:hover {
    background: var(--hover-bg, #f0f0f0);
    color: var(--text-color, #333);
  }

  .modal-body {
    padding: 20px;
    overflow-y: auto;
    flex: 1;
  }

  .date-display {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 10px 12px;
    background: var(--panel-bg, #f5f5f5);
    border-radius: 8px;
    margin-bottom: 20px;
    color: var(--text-color, #333);
    font-size: 14px;
    font-weight: 500;
  }

  .form-group {
    margin-bottom: 16px;
  }

  .form-group label {
    display: block;
    font-size: 13px;
    font-weight: 500;
    color: var(--text-muted, #666);
    margin-bottom: 6px;
  }

  .form-group input,
  .form-group select {
    width: 100%;
    padding: 10px 12px;
    border: 1px solid var(--border-color, #e0e0e0);
    border-radius: 8px;
    font-size: 14px;
    background: var(--input-bg, #fff);
    color: var(--text-color, #333);
  }

  .form-group input:focus,
  .form-group select:focus {
    outline: none;
    border-color: var(--primary-color, #4f6bed);
    box-shadow: 0 0 0 3px var(--primary-light-bg, #e0e7ff);
  }

  .form-row {
    display: flex;
    gap: 12px;
  }

  .form-row .form-group {
    flex: 1;
  }

  .color-picker {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
  }

  .color-swatch {
    width: 32px;
    height: 32px;
    border-radius: 8px;
    border: 2px solid transparent;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: transform 0.15s, border-color 0.15s;
  }

  .color-swatch:hover {
    transform: scale(1.1);
  }

  .color-swatch.selected {
    border-color: var(--text-color, #333);
    box-shadow: 0 0 0 2px var(--modal-bg, #fff);
  }

  .preview-section {
    margin-top: 20px;
  }

  .preview-section label {
    display: block;
    font-size: 13px;
    font-weight: 500;
    color: var(--text-muted, #666);
    margin-bottom: 8px;
  }

  .block-preview {
    padding: 12px 16px;
    border-radius: 8px;
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .preview-time {
    font-size: 12px;
    font-weight: 600;
  }

  .preview-label {
    font-size: 14px;
    font-weight: 500;
  }

  .preview-context {
    font-size: 12px;
    opacity: 0.85;
  }

  .modal-footer {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 16px 20px;
    border-top: 1px solid var(--border-color, #e0e0e0);
    gap: 12px;
  }

  .footer-actions {
    display: flex;
    gap: 8px;
    margin-left: auto;
  }

  .cancel-btn,
  .save-btn,
  .delete-btn {
    padding: 10px 20px;
    border-radius: 8px;
    font-size: 14px;
    font-weight: 500;
    cursor: pointer;
    border: none;
  }

  .cancel-btn {
    background: var(--btn-secondary-bg, #f0f0f0);
    color: var(--text-color, #333);
  }

  .cancel-btn:hover {
    background: var(--hover-bg, #e0e0e0);
  }

  .save-btn {
    background: var(--primary-color, #4f6bed);
    color: white;
  }

  .save-btn:hover:not(:disabled) {
    background: var(--primary-hover, #3b5998);
  }

  .save-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .delete-btn {
    background: var(--error-bg, #fee);
    color: var(--error-color, #d32f2f);
  }

  .delete-btn:hover {
    background: #fdd;
  }
</style>
