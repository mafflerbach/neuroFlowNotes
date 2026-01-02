<script lang="ts">
  import { X, Calendar, Check, Repeat, Link } from "lucide-svelte";
  import { BLOCK_COLORS, DEFAULT_BLOCK_COLOR, getBlockColor, getBlockColorVar } from "../constants/colors";
  import type { ScheduleBlockDto, NoteListItem } from "../types";
  import NoteAutocomplete from "./shared/NoteAutocomplete.svelte";

  // Common recurrence patterns with their RRULE strings
  const RECURRENCE_OPTIONS = [
    { value: "", label: "Does not repeat" },
    { value: "FREQ=DAILY", label: "Daily" },
    { value: "FREQ=WEEKLY", label: "Weekly" },
    { value: "FREQ=WEEKLY;INTERVAL=2", label: "Every 2 weeks" },
    { value: "FREQ=MONTHLY", label: "Monthly" },
    { value: "FREQ=WEEKLY;BYDAY=MO,TU,WE,TH,FR", label: "Weekdays" },
  ] as const;

  interface Props {
    open: boolean;
    mode: "create" | "edit";
    date: string; // YYYY-MM-DD
    initialHour?: number;
    block?: ScheduleBlockDto | null;
    linkedNote?: NoteListItem | null; // The currently linked note (for edit mode)
    onSave: (data: {
      date: string;
      start_time: string;
      end_time: string;
      label: string;
      color: string;
      context: string | null;
      rrule: string | null;
      note_id: number | null;
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
    linkedNote = null,
    onSave,
    onDelete,
    onClose,
  }: Props = $props();

  // Form state
  let selectedDate = $state(""); // YYYY-MM-DD
  let startTime = $state("09:00");
  let endTime = $state("10:00");
  let label = $state("");
  let selectedColor = $state(DEFAULT_BLOCK_COLOR.id);
  let context = $state("");
  let recurrence = $state("");
  let selectedNote = $state<NoteListItem | null>(null);

  // Initialize form when block changes or modal opens
  $effect(() => {
    if (open) {
      if (mode === "edit" && block) {
        selectedDate = block.date;
        startTime = block.start_time.slice(0, 5);
        endTime = block.end_time.slice(0, 5);
        label = block.label || "";
        selectedColor = block.color || DEFAULT_BLOCK_COLOR.id;
        context = block.context || "";
        recurrence = block.rrule || "";
        selectedNote = linkedNote;
      } else {
        // Create mode
        selectedDate = date;
        startTime = `${initialHour.toString().padStart(2, "0")}:00`;
        endTime = `${(initialHour + 1).toString().padStart(2, "0")}:00`;
        label = "";
        selectedColor = DEFAULT_BLOCK_COLOR.id;
        context = "";
        recurrence = "";
        selectedNote = null;
      }
    }
  });

  function handleSubmit(e: Event) {
    e.preventDefault();

    const trimmedLabel = label.trim();
    if (!trimmedLabel) return; // Shouldn't happen due to required, but safety check

    onSave({
      date: selectedDate,
      start_time: startTime + ":00",
      end_time: endTime + ":00",
      label: trimmedLabel,
      color: selectedColor,
      context: context.trim() || null,
      rrule: recurrence || null,
      note_id: selectedNote?.id ?? null,
    });
  }

  function handleNoteSelect(note: NoteListItem | null) {
    selectedNote = note;
    // If a note is selected and label is empty, use the note's title
    if (note && !label.trim()) {
      label = note.title || note.path.replace(/\.md$/, "").split("/").pop() || "";
    }
  }

  // Check if this is an occurrence (not the master recurring block)
  const isOccurrence = $derived(block?.is_occurrence ?? false);

  // Get display label for current recurrence
  const recurrenceLabel = $derived(
    RECURRENCE_OPTIONS.find(opt => opt.value === recurrence)?.label || "Custom"
  );

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
          <!-- Date input -->
          <div class="form-group">
            <label for="block-date">
              <Calendar size={14} class="inline-icon" />
              Date
            </label>
            <input
              id="block-date"
              type="date"
              bind:value={selectedDate}
              required
            />
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
              autocomplete="off"
              autocorrect="off"
              autocapitalize="off"
              spellcheck="false"
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
              autocomplete="off"
              autocorrect="off"
              autocapitalize="off"
              spellcheck="false"
            />
          </div>

          <!-- Link to existing note -->
          <div class="form-group">
            <label>
              <Link size={14} class="inline-icon" />
              Link to note (optional)
            </label>
            <NoteAutocomplete
              selectedNote={selectedNote}
              onSelect={handleNoteSelect}
              placeholder="Search for a note to link..."
            />
            <p class="field-hint">
              Link an existing note, or leave empty to create one when clicking the block.
            </p>
          </div>

          <!-- Recurrence -->
          <div class="form-group">
            <label for="block-recurrence">
              <Repeat size={14} class="inline-icon" />
              Repeat
            </label>
            {#if isOccurrence}
              <p class="occurrence-notice">
                This is an occurrence of a recurring event. Edit the master event to change recurrence.
              </p>
            {:else}
              <select
                id="block-recurrence"
                bind:value={recurrence}
                class="recurrence-select"
              >
                {#each RECURRENCE_OPTIONS as option (option.value)}
                  <option value={option.value}>{option.label}</option>
                {/each}
              </select>
            {/if}
          </div>

          <!-- Color picker -->
          <div class="form-group">
            <span class="field-label">Color</span>
            <div class="color-picker">
              {#each BLOCK_COLORS as color (color.id)}
                <button
                  type="button"
                  class="color-swatch"
                  class:selected={selectedColor === color.id}
                  style="background-color: {getBlockColorVar(color.id)}"
                  onclick={() => (selectedColor = color.id)}
                  title={color.name}
                  aria-label={color.name}
                >
                  {#if selectedColor === color.id}
                    <Check size={14} strokeWidth={3} style="color: {color.textColor}" />
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
              style="background-color: {getBlockColorVar(selectedColor)}; color: {currentColorObj.textColor}"
            >
              <span class="preview-time">{startTime} - {endTime}</span>
              {#if label}
                <span class="preview-label">{label}</span>
              {/if}
              {#if context}
                <span class="preview-context">{context}</span>
              {/if}
              {#if recurrence}
                <span class="preview-recurrence">
                  <Repeat size={12} />
                  {recurrenceLabel}
                </span>
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

  .field-hint {
    font-size: var(--font-size-xs);
    color: var(--text-muted);
    margin: var(--spacing-1) 0 0 0;
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

  /* Recurrence styles */
  .recurrence-select {
    width: 100%;
    padding: var(--spacing-3);
    border: 1px solid var(--input-border);
    border-radius: var(--radius-lg);
    font-size: var(--font-size-md);
    background: var(--input-bg);
    color: var(--input-text);
    cursor: pointer;
  }

  .recurrence-select:focus {
    outline: none;
    border-color: var(--input-border-focus);
    box-shadow: var(--shadow-focus);
  }

  .occurrence-notice {
    font-size: var(--font-size-sm);
    color: var(--text-muted);
    margin: 0;
    padding: var(--spacing-2);
    background: var(--bg-surface-raised);
    border-radius: var(--radius-md);
  }

  .preview-recurrence {
    display: flex;
    align-items: center;
    gap: var(--spacing-1);
    font-size: var(--font-size-xs);
    opacity: 0.85;
    margin-top: var(--spacing-1);
  }

  :global(.inline-icon) {
    display: inline;
    vertical-align: middle;
    margin-right: var(--spacing-1);
  }
</style>
