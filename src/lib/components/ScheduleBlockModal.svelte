<script lang="ts">
  import { X, Calendar, Check } from "lucide-svelte";
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
      label: string;
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

    const trimmedLabel = label.trim();
    if (!trimmedLabel) return; // Shouldn't happen due to required, but safety check

    onSave({
      date,
      start_time: startTime + ":00",
      end_time: endTime + ":00",
      label: trimmedLabel,
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
    tabindex="-1"
    onkeydown={handleKeydown}
    onclick={handleBackdropClick}
  >
    <div class="modal-content">
      <div class="modal-header">
        <h2 id="modal-title">
          {mode === "create" ? "New Schedule Block" : "Edit Schedule Block"}
        </h2>
        <button class="close-btn" onclick={onClose} aria-label="Close">
          <X size={20} />
        </button>
      </div>

      <form onsubmit={handleSubmit}>
        <div class="modal-body">
          <!-- Date display -->
          <div class="date-display">
            <Calendar size={16} />
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

          <!-- Label (required - becomes the note title/filename) -->
          <div class="form-group">
            <label for="block-label">Title <span class="required">*</span></label>
            <input
              id="block-label"
              type="text"
              bind:value={label}
              placeholder="Meeting, Focus time, etc."
              required
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
            <span class="field-label">Color</span>
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
                    <Check size={14} strokeWidth={3} color={color.textColor} />
                  {/if}
                </button>
              {/each}
            </div>
          </div>

          <!-- Preview -->
          <div class="preview-section">
            <span class="field-label">Preview</span>
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
    max-width: 420px;
    max-height: 90vh;
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

  .modal-header h2 {
    font-size: var(--font-size-xl);
    font-weight: var(--font-weight-semibold);
    margin: 0;
    color: var(--text-primary);
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

  .date-display {
    display: flex;
    align-items: center;
    gap: var(--spacing-2);
    padding: var(--spacing-3);
    background: var(--bg-surface-raised);
    border-radius: var(--radius-lg);
    margin-bottom: var(--spacing-5);
    color: var(--text-primary);
    font-size: var(--font-size-md);
    font-weight: var(--font-weight-medium);
  }

  .form-group {
    margin-bottom: var(--spacing-4);
  }

  .form-group label,
  .field-label {
    display: block;
    font-size: var(--font-size-base);
    font-weight: var(--font-weight-medium);
    color: var(--text-muted);
    margin-bottom: var(--spacing-2);
  }

  .required {
    color: var(--color-error);
  }

  .form-group input {
    width: 100%;
    padding: var(--spacing-3);
    border: 1px solid var(--input-border);
    border-radius: var(--radius-lg);
    font-size: var(--font-size-md);
    background: var(--input-bg);
    color: var(--input-text);
  }

  .form-group input:focus {
    outline: none;
    border-color: var(--input-border-focus);
    box-shadow: var(--shadow-focus);
  }

  .form-row {
    display: flex;
    gap: var(--spacing-3);
  }

  .form-row .form-group {
    flex: 1;
  }

  .color-picker {
    display: flex;
    flex-wrap: wrap;
    gap: var(--spacing-2);
  }

  .color-swatch {
    width: 32px;
    height: 32px;
    border-radius: var(--radius-lg);
    border: 2px solid transparent;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: transform var(--transition-normal), border-color var(--transition-normal);
  }

  .color-swatch:hover {
    transform: scale(1.1);
  }

  .color-swatch.selected {
    border-color: var(--text-primary);
    box-shadow: 0 0 0 2px var(--modal-bg);
  }

  .preview-section {
    margin-top: var(--spacing-5);
  }

  .preview-section .field-label {
    margin-bottom: var(--spacing-2);
  }

  .block-preview {
    padding: var(--spacing-3) var(--spacing-4);
    border-radius: var(--radius-lg);
    display: flex;
    flex-direction: column;
    gap: var(--spacing-1);
  }

  .preview-time {
    font-size: var(--font-size-sm);
    font-weight: var(--font-weight-semibold);
  }

  .preview-label {
    font-size: var(--font-size-md);
    font-weight: var(--font-weight-medium);
  }

  .preview-context {
    font-size: var(--font-size-sm);
    opacity: 0.85;
  }

  .modal-footer {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: var(--spacing-4) var(--spacing-5);
    border-top: 1px solid var(--modal-border);
    gap: var(--spacing-3);
  }

  .footer-actions {
    display: flex;
    gap: var(--spacing-2);
    margin-left: auto;
  }

  .cancel-btn,
  .save-btn,
  .delete-btn {
    padding: var(--spacing-3) var(--spacing-5);
    border-radius: var(--radius-lg);
    font-size: var(--font-size-md);
    font-weight: var(--font-weight-medium);
    cursor: pointer;
    border: none;
  }

  .cancel-btn {
    background: var(--btn-secondary-bg);
    color: var(--btn-secondary-text);
  }

  .cancel-btn:hover {
    background: var(--btn-secondary-bg-hover);
  }

  .save-btn {
    background: var(--btn-primary-bg);
    color: var(--btn-primary-text);
  }

  .save-btn:hover:not(:disabled) {
    background: var(--btn-primary-bg-hover);
  }

  .save-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .delete-btn {
    background: var(--color-error-light);
    color: var(--color-error);
  }

  .delete-btn:hover {
    background: var(--color-error-hover);
  }
</style>
