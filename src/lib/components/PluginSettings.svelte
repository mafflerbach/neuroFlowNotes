<script lang="ts">
  /**
   * Plugin Settings Component
   *
   * Displays all registered plugins with enable/disable toggles
   * and their individual settings.
   */
  import { pluginRegistry } from "../plugins";
  import type { RegisteredPlugin } from "../plugins/types";
  import { ChevronDown, ChevronRight } from "lucide-svelte";

  // Track expanded plugins (for settings)
  let expandedPlugins = $state<Set<string>>(new Set());

  // Get all plugins reactively
  const plugins = $derived(pluginRegistry.all);

  function toggleExpanded(pluginId: string) {
    if (expandedPlugins.has(pluginId)) {
      expandedPlugins.delete(pluginId);
    } else {
      expandedPlugins.add(pluginId);
    }
    expandedPlugins = new Set(expandedPlugins); // Trigger reactivity
  }

  async function handleToggleEnabled(pluginId: string) {
    await pluginRegistry.toggle(pluginId);
  }

  async function handleSettingChange(pluginId: string, key: string, value: unknown) {
    await pluginRegistry.updateSettings(pluginId, { [key]: value });
  }

  function getSettingValue(plugin: RegisteredPlugin, key: string): unknown {
    return plugin.config.settings[key];
  }

  // Type-safe value getters
  function getStringValue(value: unknown, defaultValue: unknown): string {
    if (typeof value === "string") return value;
    if (typeof defaultValue === "string") return defaultValue;
    return "";
  }

  function getNumberValue(value: unknown, defaultValue: unknown): number {
    if (typeof value === "number") return value;
    if (typeof defaultValue === "number") return defaultValue;
    return 0;
  }

  function getBooleanValue(value: unknown, defaultValue: unknown): boolean {
    if (typeof value === "boolean") return value;
    if (typeof defaultValue === "boolean") return defaultValue;
    return false;
  }
</script>

<div class="plugin-settings">
  {#if plugins.length === 0}
    <div class="no-plugins">
      <p>No plugins available.</p>
      <p class="hint">Plugins extend NeuroFlow with additional features.</p>
    </div>
  {:else}
    <div class="plugins-list">
      {#each plugins as registered (registered.plugin.meta.id)}
        {@const plugin = registered.plugin}
        {@const config = registered.config}
        {@const isExpanded = expandedPlugins.has(plugin.meta.id)}
        {@const hasSettings = plugin.settingsSchema.length > 0}

        <div class="plugin-card" class:enabled={config.enabled}>
          <div class="plugin-header">
            <button
              class="expand-btn"
              onclick={() => toggleExpanded(plugin.meta.id)}
              disabled={!hasSettings}
              class:has-settings={hasSettings}
            >
              {#if hasSettings}
                {#if isExpanded}
                  <ChevronDown size={16} />
                {:else}
                  <ChevronRight size={16} />
                {/if}
              {:else}
                <span class="no-expand-spacer"></span>
              {/if}
            </button>

            <div class="plugin-info">
              <div class="plugin-title-row">
                <span class="plugin-name">{plugin.meta.name}</span>
                <span class="plugin-version">v{plugin.meta.version}</span>
              </div>
              <p class="plugin-description">{plugin.meta.description}</p>
            </div>

            <label class="toggle">
              <input
                type="checkbox"
                checked={config.enabled}
                onchange={() => handleToggleEnabled(plugin.meta.id)}
              />
              <span class="toggle-slider"></span>
            </label>
          </div>

          {#if isExpanded && hasSettings && config.enabled}
            <div class="plugin-settings-panel">
              {#each plugin.settingsSchema as section}
                <div class="settings-section">
                  {#if section.title}
                    <h4 class="settings-section-title">{section.title}</h4>
                  {/if}
                  {#if section.description}
                    <p class="settings-section-desc">{section.description}</p>
                  {/if}

                  {#each section.fields as field}
                    {@const value = getSettingValue(registered, field.key)}
                    <div class="setting-field">
                      <label class="field-label" for="{plugin.meta.id}-{field.key}">
                        {field.label}
                        {#if field.required}
                          <span class="required">*</span>
                        {/if}
                      </label>
                      {#if field.description}
                        <p class="field-description">{field.description}</p>
                      {/if}

                      <div class="field-input">
                        {#if field.type === "string"}
                          <input
                            type="text"
                            id="{plugin.meta.id}-{field.key}"
                            value={getStringValue(value, field.default)}
                            placeholder={field.placeholder}
                            onchange={(e) => handleSettingChange(plugin.meta.id, field.key, e.currentTarget.value)}
                            autocomplete="off"
                            autocorrect="off"
                            autocapitalize="off"
                            spellcheck="false"
                          />
                        {:else if field.type === "password"}
                          <input
                            type="password"
                            id="{plugin.meta.id}-{field.key}"
                            value={getStringValue(value, field.default)}
                            placeholder={field.placeholder}
                            onchange={(e) => handleSettingChange(plugin.meta.id, field.key, e.currentTarget.value)}
                            autocomplete="off"
                            autocorrect="off"
                            autocapitalize="off"
                            spellcheck="false"
                          />
                        {:else if field.type === "number"}
                          <input
                            type="number"
                            id="{plugin.meta.id}-{field.key}"
                            value={getNumberValue(value, field.default)}
                            min={field.min}
                            max={field.max}
                            onchange={(e) => handleSettingChange(plugin.meta.id, field.key, Number(e.currentTarget.value))}
                          />
                        {:else if field.type === "boolean"}
                          <label class="toggle small">
                            <input
                              type="checkbox"
                              id="{plugin.meta.id}-{field.key}"
                              checked={getBooleanValue(value, field.default)}
                              onchange={(e) => handleSettingChange(plugin.meta.id, field.key, e.currentTarget.checked)}
                            />
                            <span class="toggle-slider"></span>
                          </label>
                        {:else if field.type === "select"}
                          <select
                            id="{plugin.meta.id}-{field.key}"
                            value={getStringValue(value, field.default)}
                            onchange={(e) => handleSettingChange(plugin.meta.id, field.key, e.currentTarget.value)}
                          >
                            {#each field.options ?? [] as option}
                              <option value={option.value}>{option.label}</option>
                            {/each}
                          </select>
                        {:else if field.type === "textarea"}
                          <textarea
                            id="{plugin.meta.id}-{field.key}"
                            value={getStringValue(value, field.default)}
                            placeholder={field.placeholder}
                            rows="4"
                            onchange={(e) => handleSettingChange(plugin.meta.id, field.key, e.currentTarget.value)}
                          ></textarea>
                        {/if}
                      </div>
                    </div>
                  {/each}
                </div>
              {/each}
            </div>
          {/if}
        </div>
      {/each}
    </div>
  {/if}
</div>

<style>
  .plugin-settings {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-3);
  }

  .no-plugins {
    text-align: center;
    padding: var(--spacing-6);
    color: var(--text-muted);
  }

  .no-plugins p {
    margin: 0;
  }

  .no-plugins .hint {
    font-size: var(--font-size-sm);
    margin-top: var(--spacing-2);
  }

  .plugins-list {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-3);
  }

  .plugin-card {
    border: 1px solid var(--border-light);
    border-radius: var(--radius-md);
    background: var(--bg-surface);
    overflow: hidden;
  }

  .plugin-card.enabled {
    border-color: var(--color-primary);
  }

  .plugin-header {
    display: flex;
    align-items: flex-start;
    gap: var(--spacing-3);
    padding: var(--spacing-3);
  }

  .expand-btn {
    flex-shrink: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    width: 24px;
    height: 24px;
    padding: 0;
    border: none;
    background: transparent;
    color: var(--text-muted);
    cursor: pointer;
    border-radius: var(--radius-sm);
  }

  .expand-btn:hover:not(:disabled) {
    background: var(--bg-hover);
    color: var(--text-primary);
  }

  .expand-btn:disabled {
    cursor: default;
    opacity: 0.3;
  }

  .no-expand-spacer {
    width: 16px;
  }

  .plugin-info {
    flex: 1;
    min-width: 0;
  }

  .plugin-title-row {
    display: flex;
    align-items: center;
    gap: var(--spacing-2);
  }

  .plugin-name {
    font-weight: var(--font-weight-semibold);
    color: var(--text-primary);
  }

  .plugin-version {
    font-size: var(--font-size-xs);
    color: var(--text-muted);
  }

  .plugin-description {
    margin: var(--spacing-1) 0 0 0;
    font-size: var(--font-size-sm);
    color: var(--text-secondary);
    line-height: var(--line-height-normal);
  }

  /* Toggle switch */
  .toggle {
    position: relative;
    display: inline-block;
    width: 44px;
    height: 24px;
    flex-shrink: 0;
  }

  .toggle.small {
    width: 36px;
    height: 20px;
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

  .toggle.small .toggle-slider::before {
    height: 14px;
    width: 14px;
  }

  .toggle input:checked + .toggle-slider {
    background: var(--color-primary);
  }

  .toggle input:checked + .toggle-slider::before {
    transform: translateX(20px);
  }

  .toggle.small input:checked + .toggle-slider::before {
    transform: translateX(16px);
  }

  /* Settings Panel */
  .plugin-settings-panel {
    border-top: 1px solid var(--border-light);
    padding: var(--spacing-3);
    background: var(--bg-surface-sunken);
  }

  .settings-section {
    margin-bottom: var(--spacing-4);
  }

  .settings-section:last-child {
    margin-bottom: 0;
  }

  .settings-section-title {
    font-size: var(--font-size-sm);
    font-weight: var(--font-weight-semibold);
    color: var(--text-muted);
    text-transform: uppercase;
    letter-spacing: 0.5px;
    margin: 0 0 var(--spacing-2) 0;
  }

  .settings-section-desc {
    font-size: var(--font-size-sm);
    color: var(--text-muted);
    margin: 0 0 var(--spacing-3) 0;
  }

  .setting-field {
    margin-bottom: var(--spacing-3);
  }

  .setting-field:last-child {
    margin-bottom: 0;
  }

  .field-label {
    display: block;
    font-size: var(--font-size-sm);
    font-weight: var(--font-weight-medium);
    color: var(--text-primary);
    margin-bottom: var(--spacing-1);
  }

  .field-label .required {
    color: var(--color-error);
  }

  .field-description {
    font-size: var(--font-size-xs);
    color: var(--text-muted);
    margin: 0 0 var(--spacing-2) 0;
  }

  .field-input input[type="text"],
  .field-input input[type="password"],
  .field-input input[type="number"],
  .field-input select,
  .field-input textarea {
    width: 100%;
    padding: var(--spacing-2);
    font-size: var(--font-size-sm);
    border: 1px solid var(--input-border);
    border-radius: var(--radius-sm);
    background: var(--input-bg);
    color: var(--input-text);
  }

  .field-input input:focus,
  .field-input select:focus,
  .field-input textarea:focus {
    outline: none;
    border-color: var(--input-border-focus);
  }

  .field-input textarea {
    resize: vertical;
    min-height: 80px;
  }
</style>
