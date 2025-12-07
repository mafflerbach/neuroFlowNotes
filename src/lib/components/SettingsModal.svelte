<script lang="ts">
  import { workspaceStore, type CalendarView } from "../stores/workspace.svelte";

  interface Props {
    open: boolean;
    onClose: () => void;
  }

  let { open, onClose }: Props = $props();

  // Local state for settings
  let multiColumnEditable = $state(workspaceStore.multiColumnEditable);
  let defaultCalendarView = $state<CalendarView>(workspaceStore.getDefaultCalendarView());

  // Reset local state when modal opens
  $effect(() => {
    if (open) {
      multiColumnEditable = workspaceStore.multiColumnEditable;
      defaultCalendarView = workspaceStore.getDefaultCalendarView();
    }
  });

  function handleSave() {
    workspaceStore.multiColumnEditable = multiColumnEditable;
    workspaceStore.setDefaultCalendarView(defaultCalendarView);
    onClose();
  }

  function handleCancel() {
    // Reset to current values
    multiColumnEditable = workspaceStore.multiColumnEditable;
    defaultCalendarView = workspaceStore.getDefaultCalendarView();
    onClose();
  }

  function handleBackdropClick(event: MouseEvent) {
    if (event.target === event.currentTarget) {
      handleCancel();
    }
  }

  function handleKeyDown(event: KeyboardEvent) {
    if (event.key === "Escape") {
      handleCancel();
    }
  }
</script>

{#if open}
  <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
  <div
    class="modal-backdrop"
    role="dialog"
    aria-modal="true"
    aria-labelledby="settings-title"
    onclick={handleBackdropClick}
    onkeydown={handleKeyDown}
  >
    <div class="modal">
      <div class="modal-header">
        <h2 id="settings-title" class="modal-title">Settings</h2>
        <button class="close-btn" onclick={handleCancel} title="Close">
          <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M18 6L6 18M6 6l12 12" />
          </svg>
        </button>
      </div>

      <div class="modal-content">
        <!-- Editor Settings Section -->
        <section class="settings-section">
          <h3 class="section-title">Editor</h3>

          <div class="setting-row">
            <div class="setting-info">
              <label for="multi-column-editable" class="setting-label">
                Multi-column editing
              </label>
              <p class="setting-description">
                When enabled, all visible document columns are editable. When disabled, only the active column is editable and others are read-only.
              </p>
            </div>
            <div class="setting-control">
              <label class="toggle">
                <input
                  type="checkbox"
                  id="multi-column-editable"
                  bind:checked={multiColumnEditable}
                />
                <span class="toggle-slider"></span>
              </label>
            </div>
          </div>
        </section>

        <!-- Calendar Settings Section -->
        <section class="settings-section">
          <h3 class="section-title">Calendar</h3>

          <div class="setting-row">
            <div class="setting-info">
              <span class="setting-label">Default view</span>
              <p class="setting-description">
                The calendar view to show when opening the app.
              </p>
            </div>
            <div class="setting-control">
              <select class="select-control" bind:value={defaultCalendarView}>
                <option value="monthly">Monthly</option>
                <option value="weekly">Weekly</option>
                <option value="daily">Daily</option>
              </select>
            </div>
          </div>
        </section>

        <!-- Vault Settings Section -->
        <section class="settings-section">
          <h3 class="section-title">Vault</h3>

          <div class="setting-row">
            <div class="setting-info">
              <span class="setting-label">Vault path</span>
              <p class="setting-description">
                The current vault location.
              </p>
            </div>
            <div class="setting-control">
              <button class="action-btn">
                Change vault
              </button>
            </div>
          </div>
        </section>
      </div>

      <div class="modal-footer">
        <button class="btn btn-secondary" onclick={handleCancel}>
          Cancel
        </button>
        <button class="btn btn-primary" onclick={handleSave}>
          Save changes
        </button>
      </div>
    </div>
  </div>
{/if}

<style>
  .modal-backdrop {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.5);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
  }

  .modal {
    background: var(--modal-bg, #fff);
    border-radius: 12px;
    box-shadow: 0 20px 60px rgba(0, 0, 0, 0.3);
    width: 90%;
    max-width: 520px;
    max-height: 80vh;
    display: flex;
    flex-direction: column;
  }

  .modal-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 16px 20px;
    border-bottom: 1px solid var(--border-color, #e0e0e0);
  }

  .modal-title {
    font-size: 18px;
    font-weight: 600;
    color: var(--text-color, #333);
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
    border-radius: 6px;
    color: var(--text-muted, #666);
    cursor: pointer;
  }

  .close-btn:hover {
    background: var(--hover-bg, #f0f0f0);
  }

  .modal-content {
    flex: 1;
    overflow-y: auto;
    padding: 20px;
  }

  .settings-section {
    margin-bottom: 24px;
  }

  .settings-section:last-child {
    margin-bottom: 0;
  }

  .section-title {
    font-size: 13px;
    font-weight: 600;
    color: var(--text-muted, #666);
    text-transform: uppercase;
    letter-spacing: 0.5px;
    margin: 0 0 12px 0;
  }

  .setting-row {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: 16px;
    padding: 12px 0;
    border-bottom: 1px solid var(--border-light, #f0f0f0);
  }

  .setting-row:last-child {
    border-bottom: none;
  }

  .setting-info {
    flex: 1;
  }

  .setting-label {
    font-size: 14px;
    font-weight: 500;
    color: var(--text-color, #333);
  }

  .setting-description {
    font-size: 12px;
    color: var(--text-muted, #666);
    margin: 4px 0 0 0;
    line-height: 1.4;
  }

  .setting-control {
    flex-shrink: 0;
  }

  /* Toggle switch */
  .toggle {
    position: relative;
    display: inline-block;
    width: 44px;
    height: 24px;
  }

  .toggle input {
    opacity: 0;
    width: 0;
    height: 0;
  }

  .toggle-slider {
    position: absolute;
    cursor: pointer;
    inset: 0;
    background: var(--toggle-bg, #ccc);
    border-radius: 24px;
    transition: 0.2s;
  }

  .toggle-slider::before {
    position: absolute;
    content: "";
    height: 18px;
    width: 18px;
    left: 3px;
    bottom: 3px;
    background: white;
    border-radius: 50%;
    transition: 0.2s;
  }

  .toggle input:checked + .toggle-slider {
    background: var(--primary-color, #4f6bed);
  }

  .toggle input:checked + .toggle-slider::before {
    transform: translateX(20px);
  }

  /* Select control */
  .select-control {
    padding: 6px 12px;
    font-size: 13px;
    border: 1px solid var(--border-color, #e0e0e0);
    border-radius: 6px;
    background: var(--input-bg, #fff);
    color: var(--text-color, #333);
    cursor: pointer;
  }

  .select-control:focus {
    outline: none;
    border-color: var(--primary-color, #4f6bed);
  }

  /* Action button */
  .action-btn {
    padding: 6px 12px;
    font-size: 13px;
    font-weight: 500;
    color: var(--primary-color, #4f6bed);
    background: transparent;
    border: 1px solid var(--primary-color, #4f6bed);
    border-radius: 6px;
    cursor: pointer;
  }

  .action-btn:hover {
    background: var(--primary-light-bg, #e0e7ff);
  }

  .modal-footer {
    display: flex;
    justify-content: flex-end;
    gap: 12px;
    padding: 16px 20px;
    border-top: 1px solid var(--border-color, #e0e0e0);
  }

  .btn {
    padding: 8px 16px;
    font-size: 14px;
    font-weight: 500;
    border-radius: 6px;
    cursor: pointer;
    transition: background 0.15s;
  }

  .btn-secondary {
    color: var(--text-color, #333);
    background: var(--btn-secondary-bg, #f0f0f0);
    border: none;
  }

  .btn-secondary:hover {
    background: var(--hover-bg, #e0e0e0);
  }

  .btn-primary {
    color: white;
    background: var(--primary-color, #4f6bed);
    border: none;
  }

  .btn-primary:hover {
    background: var(--primary-hover, #3d5bd9);
  }
</style>
