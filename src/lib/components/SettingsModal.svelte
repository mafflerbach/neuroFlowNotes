<script lang="ts">
  import { untrack } from "svelte";
  import { Modal, TextInput } from "./shared";
  import PropertiesEditor from "./PropertiesEditor.svelte";
  import PluginSettings from "./PluginSettings.svelte";
  import ImportModal from "./ImportModal.svelte";
  import { workspaceStore, type CalendarView } from "../stores/workspace.svelte";
  import { vaultStore } from "../stores/vault.svelte";
  import type { Theme } from "../services/settings";
  import { getAvailableThemes } from "../services/themes";
  import * as api from "../services/api";
  import type { TemplateSettings, EmbeddingSettings, EmbeddingStatus } from "../types";
  import { DEFAULT_TEMPLATE_SETTINGS, DEFAULT_EMBEDDING_SETTINGS } from "../types";
  import { Loader2, CheckCircle, XCircle } from "lucide-svelte";
  import { open as openDialog } from "@tauri-apps/plugin-dialog";

  interface Props {
    open: boolean;
    onClose: () => void;
    embeddingSettings: EmbeddingSettings;
    onEmbeddingSettingsChange: (settings: EmbeddingSettings) => void;
  }

  let { open, onClose, embeddingSettings, onEmbeddingSettingsChange }: Props = $props();

  // Track active section for tabs
  let activeSection = $state<"settings" | "properties" | "plugins">("settings");

  // Import modal state
  let showImportModal = $state(false);

  // Get available themes
  const availableThemes = getAvailableThemes();

  // Local state for settings
  let multiColumnEditable = $state(workspaceStore.multiColumnEditable);
  let defaultCalendarView = $state<CalendarView>(workspaceStore.getDefaultCalendarView());
  let theme = $state<Theme>(workspaceStore.getTheme());
  let vimMode = $state(workspaceStore.vimMode);

  // Embedding settings (local copy)
  let localEmbeddingSettings = $state<EmbeddingSettings>({ ...DEFAULT_EMBEDDING_SETTINGS });
  let embeddingStatus = $state<EmbeddingStatus | null>(null);
  let testingConnection = $state(false);
  let indexingProgress = $state<{ current: number; total: number } | null>(null);

  // Template settings
  let templateSettings = $state<TemplateSettings>({ ...DEFAULT_TEMPLATE_SETTINGS });
  let availableTemplates = $state<string[]>([]);
  let pathPreview = $state("");
  let loadingTemplates = $state(false);

  // Load template settings
  async function loadTemplateSettings() {
    if (!vaultStore.isOpen) return;
    loadingTemplates = true;
    try {
      const [settings, templates] = await Promise.all([
        api.getTemplateSettings(),
        api.listTemplates(),
      ]);
      templateSettings = settings;
      availableTemplates = templates;
      await updatePathPreview();
    } catch (e) {
      console.error("[SettingsModal] Failed to load template settings:", e);
    } finally {
      loadingTemplates = false;
    }
  }

  // Update path preview
  async function updatePathPreview() {
    try {
      const today = new Date().toISOString().split("T")[0];
      pathPreview = await api.previewDailyNotePath(templateSettings.daily_note_pattern, today);
    } catch (e) {
      pathPreview = "(invalid pattern)";
    }
  }

  // Track previous open state to detect open/close transitions (non-reactive)
  let wasOpen = false;

  // Reset local state when modal opens (only on open transition)
  $effect(() => {
    if (open && !wasOpen) {
      // Transitioning from closed to open - initialize state
      // Use untrack to avoid creating dependencies on store/prop reads
      untrack(() => {
        multiColumnEditable = workspaceStore.multiColumnEditable;
        defaultCalendarView = workspaceStore.getDefaultCalendarView();
        theme = workspaceStore.getTheme();
        vimMode = workspaceStore.vimMode;
        activeSection = "settings";
        loadTemplateSettings();
        // Copy embedding settings
        localEmbeddingSettings = { ...embeddingSettings };
        embeddingStatus = null;
        // Test connection if enabled
        if (embeddingSettings.enabled) {
          testConnection();
        }
      });
    }
    wasOpen = open;
  });

  // Test embedding service connection
  async function testConnection() {
    if (!vaultStore.isOpen) return;
    testingConnection = true;
    try {
      embeddingStatus = await api.getEmbeddingStatus(localEmbeddingSettings);
    } catch (e) {
      console.error("[SettingsModal] Failed to test embedding connection:", e);
      embeddingStatus = {
        connected: false,
        error: String(e),
        indexed_count: 0,
        total_count: 0,
      };
    } finally {
      testingConnection = false;
    }
  }

  // Rebuild embedding index for all notes
  async function rebuildIndex() {
    if (!vaultStore.isOpen || !embeddingStatus?.connected) return;

    try {
      // Get notes needing embeddings
      const notesNeeding = await api.getNotesNeedingEmbeddings(1000);
      if (notesNeeding.length === 0) {
        // Refresh status to confirm
        await testConnection();
        return;
      }

      indexingProgress = { current: 0, total: notesNeeding.length };

      for (let i = 0; i < notesNeeding.length; i++) {
        const [noteId] = notesNeeding[i];
        try {
          await api.generateNoteEmbedding(noteId, localEmbeddingSettings);
        } catch (e) {
          console.error(`[SettingsModal] Failed to generate embedding for note ${noteId}:`, e);
        }
        indexingProgress = { current: i + 1, total: notesNeeding.length };
      }

      // Refresh status after indexing
      await testConnection();
    } catch (e) {
      console.error("[SettingsModal] Failed to rebuild index:", e);
    } finally {
      indexingProgress = null;
    }
  }

  // Update preview when pattern changes
  $effect(() => {
    if (open && templateSettings.daily_note_pattern) {
      updatePathPreview();
    }
  });

  async function handleSave() {
    workspaceStore.multiColumnEditable = multiColumnEditable;
    workspaceStore.setDefaultCalendarView(defaultCalendarView);
    workspaceStore.setTheme(theme);
    workspaceStore.setVimMode(vimMode);

    // Save template settings
    if (vaultStore.isOpen) {
      try {
        await api.saveTemplateSettings(templateSettings);
      } catch (e) {
        console.error("[SettingsModal] Failed to save template settings:", e);
      }
    }

    // Save embedding settings
    onEmbeddingSettingsChange(localEmbeddingSettings);

    onClose();
  }

  function handleCancel() {
    multiColumnEditable = workspaceStore.multiColumnEditable;
    defaultCalendarView = workspaceStore.getDefaultCalendarView();
    onClose();
  }

  async function handleChangeVault() {
    try {
      const selected = await openDialog({
        directory: true,
        multiple: false,
        title: "Select Vault Folder",
      });

      if (selected && typeof selected === "string") {
        await vaultStore.open(selected);
        onClose();
      }
    } catch (e) {
      console.error("Failed to change vault:", e);
    }
  }
</script>

<Modal {open} title="Settings" onClose={handleCancel} maxWidth={activeSection === "properties" ? "80vw" : "580px"}>
  <div class="settings-layout">
    <!-- Vertical Navigation -->
    <nav class="settings-nav">
      <button
        class="nav-item"
        class:active={activeSection === "settings"}
        onclick={() => (activeSection = "settings")}
      >
        Settings
      </button>
      <button
        class="nav-item"
        class:active={activeSection === "properties"}
        onclick={() => (activeSection = "properties")}
      >
        Properties
      </button>
      <button
        class="nav-item"
        class:active={activeSection === "plugins"}
        onclick={() => (activeSection = "plugins")}
      >
        Plugins
      </button>
    </nav>

    <!-- Content Area -->
    <div class="settings-content">
      {#if activeSection === "settings"}
        <!-- Appearance Section -->
        <section class="settings-section">
          <h3 class="section-title">Appearance</h3>

          <div class="setting-row">
            <div class="setting-info">
              <span class="setting-label">Theme</span>
              <p class="setting-description">Choose your preferred color theme.</p>
            </div>
            <div class="setting-control">
              <select class="select-control" bind:value={theme}>
                {#each availableThemes as themeOption}
                  <option value={themeOption.id}>{themeOption.name}</option>
                {/each}
              </select>
            </div>
          </div>
        </section>

        <!-- Editor Settings Section -->
        <section class="settings-section">
          <h3 class="section-title">Editor</h3>

          <div class="setting-row">
            <div class="setting-info">
              <label for="vim-mode" class="setting-label">
                Vim mode
              </label>
              <p class="setting-description">
                Enable vim keybindings in the editor. Requires reopening notes to take effect.
              </p>
            </div>
            <div class="setting-control">
              <label class="toggle">
                <input
                  type="checkbox"
                  id="vim-mode"
                  bind:checked={vimMode}
                />
                <span class="toggle-slider"></span>
              </label>
            </div>
          </div>

          <div class="setting-row">
            <div class="setting-info">
              <label for="multi-column-editable" class="setting-label">
                Multi-column editing
              </label>
              <p class="setting-description">
                When enabled, all visible document columns are editable. When disabled, only the active column is editable.
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
              <p class="setting-description">The calendar view to show when opening the app.</p>
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

        <!-- Daily Notes Section -->
        <section class="settings-section">
          <h3 class="section-title">Daily Notes</h3>

          <div class="setting-row">
            <div class="setting-info">
              <span class="setting-label">Template file</span>
              <p class="setting-description">
                The template to use when creating daily notes.
                Place templates in the <code>templates/</code> folder.
              </p>
            </div>
            <div class="setting-control">
              <select
                class="select-control"
                bind:value={templateSettings.daily_template_path}
                disabled={loadingTemplates}
              >
                <option value={null}>None (use default)</option>
                {#each availableTemplates as template}
                  <option value={template}>{template}</option>
                {/each}
              </select>
            </div>
          </div>

          <div class="setting-row">
            <div class="setting-info">
              <span class="setting-label">File path pattern</span>
              <p class="setting-description">
                Pattern for daily note paths. Variables: date, year, month, day, weekday, month_name
              </p>
              {#if pathPreview}
                <p class="path-preview">
                  Preview: <code>{pathPreview}</code>
                </p>
              {/if}
            </div>
            <div class="setting-control pattern-control">
              <TextInput
                class="input-control"
                bind:value={templateSettings.daily_note_pattern}
                placeholder="journal/year/month/date.md"
              />
            </div>
          </div>
        </section>

        <!-- Semantic Search Section -->
        <section class="settings-section">
          <h3 class="section-title">Semantic Search</h3>

          <div class="setting-row">
            <div class="setting-info">
              <label for="embedding-enabled" class="setting-label">
                Enable semantic search
              </label>
              <p class="setting-description">
                Use AI embeddings for semantic search. Requires LM Studio running locally.
              </p>
            </div>
            <div class="setting-control">
              <label class="toggle">
                <input
                  type="checkbox"
                  id="embedding-enabled"
                  bind:checked={localEmbeddingSettings.enabled}
                />
                <span class="toggle-slider"></span>
              </label>
            </div>
          </div>

          {#if localEmbeddingSettings.enabled}
            <div class="setting-row">
              <div class="setting-info">
                <span class="setting-label">LM Studio endpoint</span>
                <p class="setting-description">
                  The URL of your LM Studio server (default: http://localhost:1234/v1)
                </p>
              </div>
              <div class="setting-control pattern-control">
                <TextInput
                  class="input-control"
                  bind:value={localEmbeddingSettings.endpoint_url}
                  placeholder="http://localhost:1234/v1"
                />
              </div>
            </div>

            <div class="setting-row">
              <div class="setting-info">
                <span class="setting-label">Model name</span>
                <p class="setting-description">
                  The embedding model loaded in LM Studio (e.g., nomic-embed-text)
                </p>
              </div>
              <div class="setting-control pattern-control">
                <TextInput
                  class="input-control"
                  bind:value={localEmbeddingSettings.model}
                  placeholder="nomic-ai/nomic-embed-text-v1.5-GGUF"
                />
              </div>
            </div>

            <div class="setting-row">
              <div class="setting-info">
                <span class="setting-label">Dimensions</span>
                <p class="setting-description">
                  Vector dimensions for your model (768 for nomic-embed-text)
                </p>
              </div>
              <div class="setting-control">
                <input
                  type="number"
                  class="input-control input-small"
                  bind:value={localEmbeddingSettings.dimensions}
                  min="1"
                  max="4096"
                />
              </div>
            </div>

            <div class="setting-row">
              <div class="setting-info">
                <span class="setting-label">Connection status</span>
                {#if embeddingStatus}
                  <p class="setting-description">
                    {#if embeddingStatus.connected}
                      Indexed: {embeddingStatus.indexed_count} / {embeddingStatus.total_count} notes
                    {:else}
                      {embeddingStatus.error || "Not connected"}
                    {/if}
                  </p>
                {:else}
                  <p class="setting-description">Click test to check connection</p>
                {/if}
              </div>
              <div class="setting-control connection-status">
                {#if testingConnection}
                  <Loader2 size={18} class="spin" />
                {:else if embeddingStatus?.connected}
                  <CheckCircle size={18} class="status-connected" />
                {:else if embeddingStatus}
                  <XCircle size={18} class="status-error" />
                {/if}
                <button class="action-btn" onclick={testConnection} disabled={testingConnection}>
                  Test
                </button>
              </div>
            </div>

            {#if embeddingStatus?.connected}
              <div class="setting-row">
                <div class="setting-info">
                  <span class="setting-label">Build index</span>
                  <p class="setting-description">
                    {#if indexingProgress}
                      Generating embeddings... {indexingProgress.current} / {indexingProgress.total}
                    {:else if embeddingStatus.indexed_count < embeddingStatus.total_count}
                      {embeddingStatus.total_count - embeddingStatus.indexed_count} notes need embeddings
                    {:else}
                      All notes are indexed
                    {/if}
                  </p>
                </div>
                <div class="setting-control connection-status">
                  {#if indexingProgress}
                    <Loader2 size={18} class="spin" />
                    <span class="progress-text">{Math.round((indexingProgress.current / indexingProgress.total) * 100)}%</span>
                  {:else}
                    <button
                      class="action-btn"
                      onclick={rebuildIndex}
                      disabled={embeddingStatus.indexed_count >= embeddingStatus.total_count}
                    >
                      Rebuild
                    </button>
                  {/if}
                </div>
              </div>
            {/if}
          {/if}
        </section>

        <!-- Vault Settings Section -->
        <section class="settings-section">
          <h3 class="section-title">Vault</h3>

          <div class="setting-row">
            <div class="setting-info">
              <span class="setting-label">Vault path</span>
              <p class="setting-description">
                {#if vaultStore.info?.path}
                  <code class="vault-path">{vaultStore.info.path}</code>
                {:else}
                  No vault open
                {/if}
              </p>
            </div>
            <div class="setting-control">
              <button class="action-btn" onclick={handleChangeVault}>Change vault</button>
            </div>
          </div>

          <div class="setting-row">
            <div class="setting-info">
              <span class="setting-label">Import Obsidian Vault</span>
              <p class="setting-description">Import notes from an existing Obsidian vault, including frontmatter properties and tags.</p>
            </div>
            <div class="setting-control">
              <button class="action-btn" onclick={() => showImportModal = true}>Import</button>
            </div>
          </div>
        </section>
      {:else if activeSection === "properties"}
        <section class="settings-section properties-section">
          <h3 class="section-title">Property Management</h3>
          <p class="section-description">
            Manage property keys and values across your vault. Rename keys to fix typos,
            normalize casing, or merge similar properties.
          </p>
          <PropertiesEditor />
        </section>
      {:else if activeSection === "plugins"}
        <section class="settings-section">
          <h3 class="section-title">Plugins</h3>
          <p class="section-description">
            Enable or disable plugins to extend NeuroFlow with additional features.
            Each plugin can have its own settings.
          </p>
          <PluginSettings />
        </section>
      {/if}
    </div>
  </div>

  {#snippet footer()}
    {#if activeSection === "settings"}
      <button class="btn btn-secondary" onclick={handleCancel}>Cancel</button>
      <button class="btn btn-primary" onclick={handleSave}>Save changes</button>
    {:else}
      <button class="btn btn-secondary" onclick={handleCancel}>Close</button>
    {/if}
  {/snippet}
</Modal>

<ImportModal open={showImportModal} onClose={() => showImportModal = false} />

<style>
  .settings-layout {
    display: flex;
    gap: var(--spacing-4);
    min-height: 400px;
  }

  .settings-nav {
    flex-shrink: 0;
    display: flex;
    flex-direction: column;
    gap: var(--spacing-1);
    padding-right: var(--spacing-4);
    border-right: 1px solid var(--border-light);
    min-width: 140px;
  }

  .nav-item {
    padding: var(--spacing-2) var(--spacing-3);
    text-align: left;
    font-size: var(--font-size-md);
    font-weight: var(--font-weight-medium);
    color: var(--text-secondary);
    background: transparent;
    border: none;
    border-radius: var(--radius-md);
    cursor: pointer;
    transition: background var(--transition-fast), color var(--transition-fast);
  }

  .nav-item:hover {
    background: var(--bg-hover);
    color: var(--text-primary);
  }

  .nav-item.active {
    background: var(--bg-selected);
    color: var(--text-primary);
  }

  .settings-content {
    flex: 1;
    min-width: 0;
    overflow-y: auto;
  }

  .settings-section {
    margin-bottom: var(--spacing-6);
  }

  .settings-section:last-child {
    margin-bottom: 0;
  }

  .section-description {
    font-size: var(--font-size-sm);
    color: var(--text-muted);
    margin: 0 0 var(--spacing-4) 0;
    line-height: var(--line-height-normal);
  }

  .properties-section {
    display: flex;
    flex-direction: column;
    min-height: 0;
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

  .select-control:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }

  /* Input control */
  .input-control {
    padding: var(--spacing-2) var(--spacing-3);
    font-size: var(--font-size-base);
    border: 1px solid var(--input-border);
    border-radius: var(--radius-md);
    background: var(--input-bg);
    color: var(--input-text);
    min-width: 280px;
  }

  .input-control:focus {
    outline: none;
    border-color: var(--input-border-focus);
  }

  .pattern-control {
    flex-shrink: 1;
  }

  .path-preview {
    font-size: var(--font-size-sm);
    color: var(--text-muted);
    margin: var(--spacing-2) 0 0 0;
  }

  .path-preview code {
    background: var(--bg-surface-sunken);
    padding: var(--spacing-1) var(--spacing-2);
    border-radius: var(--radius-sm);
    font-family: var(--font-mono);
    font-size: var(--font-size-sm);
  }

  .setting-description code {
    background: var(--bg-surface-sunken);
    padding: 0 var(--spacing-1);
    border-radius: var(--radius-sm);
    font-family: var(--font-mono);
    font-size: var(--font-size-xs);
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

  /* Footer buttons */
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

  /* Small input */
  .input-small {
    width: 100px;
    min-width: 100px;
  }

  /* Connection status */
  .connection-status {
    display: flex;
    align-items: center;
    gap: var(--spacing-2);
  }

  .connection-status :global(.status-connected) {
    color: var(--green);
  }

  .connection-status :global(.status-error) {
    color: var(--red);
  }

  .connection-status :global(.spin) {
    animation: spin 1s linear infinite;
    color: var(--text-muted);
  }

  @keyframes spin {
    from { transform: rotate(0deg); }
    to { transform: rotate(360deg); }
  }

  .action-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .progress-text {
    font-size: var(--font-size-sm);
    color: var(--text-muted);
    font-weight: var(--font-weight-medium);
  }

  .vault-path {
    display: block;
    word-break: break-all;
    font-size: var(--font-size-xs);
  }
</style>
