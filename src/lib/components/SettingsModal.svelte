<script lang="ts">
  import { X } from "lucide-svelte";
  import { workspaceStore, type CalendarView } from "../stores/workspace.svelte";
  import type { Theme } from "../services/settings";

  interface Props {
    open: boolean;
    onClose: () => void;
  }

  let { open, onClose }: Props = $props();

  // Local state for settings
  let multiColumnEditable = $state(workspaceStore.multiColumnEditable);
  let defaultCalendarView = $state<CalendarView>(workspaceStore.getDefaultCalendarView());
  let theme = $state<Theme>(workspaceStore.getTheme());

  // Reset local state when modal opens
  $effect(() => {
    if (open) {
      multiColumnEditable = workspaceStore.multiColumnEditable;
      defaultCalendarView = workspaceStore.getDefaultCalendarView();
      theme = workspaceStore.getTheme();
    }
  });

  function handleSave() {
    workspaceStore.multiColumnEditable = multiColumnEditable;
    workspaceStore.setDefaultCalendarView(defaultCalendarView);
    workspaceStore.setTheme(theme);
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
    tabindex="-1"
    onclick={handleBackdropClick}
    onkeydown={handleKeyDown}
  >
    <div class="modal">
      <div class="modal-header">
        <h2 id="settings-title" class="modal-title">Settings</h2>
        <button class="close-btn" onclick={handleCancel} title="Close">
          <X size={18} />
        </button>
      </div>

      <div class="modal-content">
        <!-- Appearance Section -->
        <section class="settings-section">
          <h3 class="section-title">Appearance</h3>

          <div class="setting-row">
            <div class="setting-info">
              <span class="setting-label">Theme</span>
              <p class="setting-description">
                Choose your preferred color theme.
              </p>
            </div>
            <div class="setting-control">
              <select class="select-control" bind:value={theme}>
                <option value="system">System</option>
                <option value="light">Light</option>
                <option value="dark">Dark</option>
              </select>
            </div>
          </div>
        </section>

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
    background: var(--modal-backdrop-bg);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: var(--z-modal-backdrop);
  }

  .modal {
    background: var(--modal-bg);
    border-radius: var(--radius-xl);
    box-shadow: var(--modal-shadow);
    width: 90%;
    max-width: 520px;
    max-height: 80vh;
    display: flex;
    flex-direction: column;
    z-index: var(--z-modal);
  }

  .modal-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: var(--spacing-4) var(--spacing-5);
    border-bottom: 1px solid var(--modal-border);
  }

  .modal-title {
    font-size: var(--font-size-xl);
    font-weight: var(--font-weight-semibold);
    color: var(--text-primary);
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
  }

  .modal-content {
    flex: 1;
    overflow-y: auto;
    padding: var(--spacing-5);
  }

  .settings-section {
    margin-bottom: var(--spacing-6);
  }

  .settings-section:last-child {
    margin-bottom: 0;
  }

  .section-title {
    font-size: var(--font-size-base);
    font-weight: var(--font-weight-semibold);
    color: var(--text-muted);
    text-transform: uppercase;
    letter-spacing: 0.5px;
    margin: 0 0 var(--spacing-3) 0;
  }

  .setting-row {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: var(--spacing-4);
    padding: var(--spacing-3) 0;
    border-bottom: 1px solid var(--border-light);
  }

  .setting-row:last-child {
    border-bottom: none;
  }

  .setting-info {
    flex: 1;
  }

  .setting-label {
    font-size: var(--font-size-md);
    font-weight: var(--font-weight-medium);
    color: var(--text-primary);
  }

  .setting-description {
    font-size: var(--font-size-sm);
    color: var(--text-muted);
    margin: var(--spacing-1) 0 0 0;
    line-height: var(--line-height-normal);
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
    background: var(--border-default);
    border-radius: var(--radius-full);
    transition: var(--transition-normal);
  }

  .toggle-slider::before {
    position: absolute;
    content: "";
    height: 18px;
    width: 18px;
    left: 3px;
    bottom: 3px;
    background: var(--bg-surface);
    border-radius: var(--radius-full);
    transition: var(--transition-normal);
  }

  .toggle input:checked + .toggle-slider {
    background: var(--color-primary);
  }

  .toggle input:checked + .toggle-slider::before {
    transform: translateX(20px);
  }

  /* Select control */
  .select-control {
    padding: var(--spacing-2) var(--spacing-3);
    font-size: var(--font-size-base);
    border: 1px solid var(--input-border);
    border-radius: var(--radius-md);
    background: var(--input-bg);
    color: var(--input-text);
    cursor: pointer;
  }

  .select-control:focus {
    outline: none;
    border-color: var(--input-border-focus);
  }

  /* Action button */
  .action-btn {
    padding: var(--spacing-2) var(--spacing-3);
    font-size: var(--font-size-base);
    font-weight: var(--font-weight-medium);
    color: var(--color-primary);
    background: transparent;
    border: 1px solid var(--color-primary);
    border-radius: var(--radius-md);
    cursor: pointer;
  }

  .action-btn:hover {
    background: var(--color-primary-light);
  }

  .modal-footer {
    display: flex;
    justify-content: flex-end;
    gap: var(--spacing-3);
    padding: var(--spacing-4) var(--spacing-5);
    border-top: 1px solid var(--modal-border);
  }

  .btn {
    padding: var(--spacing-2) var(--spacing-4);
    font-size: var(--font-size-md);
    font-weight: var(--font-weight-medium);
    border-radius: var(--radius-md);
    cursor: pointer;
    transition: background var(--transition-normal);
  }

  .btn-secondary {
    color: var(--btn-secondary-text);
    background: var(--btn-secondary-bg);
    border: none;
  }

  .btn-secondary:hover {
    background: var(--btn-secondary-bg-hover);
  }

  .btn-primary {
    color: var(--btn-primary-text);
    background: var(--btn-primary-bg);
    border: none;
  }

  .btn-primary:hover {
    background: var(--btn-primary-bg-hover);
  }
</style>
