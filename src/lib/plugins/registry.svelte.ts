/**
 * Plugin Registry
 *
 * Manages plugin registration, lifecycle, and settings.
 * Uses Svelte 5 runes for reactive state.
 */

import type {
  Plugin,
  PluginConfig,
  RegisteredPlugin,
  BackendHooks,
} from "./types";

// =============================================================================
// Plugin Registry State
// =============================================================================

class PluginRegistry {
  private plugins = $state<Map<string, RegisteredPlugin>>(new Map());
  private initialized = $state(false);
  private backendHooks: BackendHooks | null = null;

  /** Get all registered plugins */
  get all(): RegisteredPlugin[] {
    return Array.from(this.plugins.values());
  }

  /** Get only enabled plugins */
  get enabled(): RegisteredPlugin[] {
    return this.all.filter((p) => p.config.enabled);
  }

  /** Check if registry is initialized */
  get isInitialized(): boolean {
    return this.initialized;
  }

  /** Initialize the registry with backend hooks */
  async initialize(hooks: BackendHooks): Promise<void> {
    this.backendHooks = hooks;
    await this.loadAllConfigs();
    this.initialized = true;
  }

  /** Reload all plugin configs from the vault (call when vault changes) */
  async reloadConfigs(): Promise<void> {
    if (!this.backendHooks) {
      console.warn("Cannot reload configs: backend hooks not available");
      return;
    }
    await this.loadAllConfigs();
  }

  /** Internal: Load configs for all registered plugins */
  private async loadAllConfigs(): Promise<void> {
    if (!this.backendHooks) return;

    for (const [pluginId, registered] of this.plugins) {
      try {
        const savedConfig = await this.backendHooks.readPluginConfig<PluginConfig>(pluginId);
        if (savedConfig) {
          registered.config = {
            ...registered.config,
            ...savedConfig,
            settings: {
              ...registered.plugin.defaultSettings,
              ...savedConfig.settings,
            },
          };
        } else {
          // No saved config - reset to defaults
          registered.config = {
            enabled: false,
            settings: { ...registered.plugin.defaultSettings },
          };
        }

        // Call onEnable if plugin is enabled
        if (registered.config.enabled && registered.plugin.onEnable) {
          await registered.plugin.onEnable(registered.config.settings);
        }
      } catch (e) {
        console.error(`Failed to load config for plugin ${pluginId}:`, e);
      }
    }
  }

  /** Register a plugin */
  register<T>(plugin: Plugin<T>): void {
    if (this.plugins.has(plugin.meta.id)) {
      console.warn(`Plugin ${plugin.meta.id} is already registered`);
      return;
    }

    const registered: RegisteredPlugin<T> = {
      plugin,
      config: {
        enabled: false,
        settings: { ...plugin.defaultSettings },
      },
    };

    this.plugins.set(plugin.meta.id, registered as RegisteredPlugin);
    console.log(`Plugin registered: ${plugin.meta.name} (${plugin.meta.id})`);
  }

  /** Unregister a plugin */
  async unregister(pluginId: string): Promise<void> {
    const registered = this.plugins.get(pluginId);
    if (!registered) return;

    // Disable first if enabled
    if (registered.config.enabled) {
      await this.disable(pluginId);
    }

    this.plugins.delete(pluginId);
  }

  /** Get a specific plugin */
  get<T>(pluginId: string): RegisteredPlugin<T> | undefined {
    return this.plugins.get(pluginId) as RegisteredPlugin<T> | undefined;
  }

  /** Enable a plugin */
  async enable(pluginId: string): Promise<void> {
    const registered = this.plugins.get(pluginId);
    if (!registered || registered.config.enabled) return;

    registered.config.enabled = true;

    // Call lifecycle hook
    if (registered.plugin.onEnable) {
      await registered.plugin.onEnable(registered.config.settings);
    }

    // Persist config
    await this.saveConfig(pluginId);
  }

  /** Disable a plugin */
  async disable(pluginId: string): Promise<void> {
    const registered = this.plugins.get(pluginId);
    if (!registered || !registered.config.enabled) return;

    // Call lifecycle hook
    if (registered.plugin.onDisable) {
      await registered.plugin.onDisable();
    }

    registered.config.enabled = false;

    // Persist config
    await this.saveConfig(pluginId);
  }

  /** Toggle plugin enabled state */
  async toggle(pluginId: string): Promise<void> {
    const registered = this.plugins.get(pluginId);
    if (!registered) return;

    if (registered.config.enabled) {
      await this.disable(pluginId);
    } else {
      await this.enable(pluginId);
    }
  }

  /** Update plugin settings */
  async updateSettings<T>(pluginId: string, settings: Partial<T>): Promise<void> {
    const registered = this.plugins.get(pluginId);
    if (!registered) return;

    registered.config.settings = {
      ...registered.config.settings,
      ...settings,
    };

    // Call lifecycle hook if enabled
    if (registered.config.enabled && registered.plugin.onSettingsChange) {
      await registered.plugin.onSettingsChange(registered.config.settings);
    }

    // Persist config
    await this.saveConfig(pluginId);
  }

  /** Save plugin config to file */
  private async saveConfig(pluginId: string): Promise<void> {
    const registered = this.plugins.get(pluginId);
    if (!registered || !this.backendHooks) return;

    try {
      await this.backendHooks.writePluginConfig(pluginId, registered.config);
    } catch (e) {
      console.error(`Failed to save config for plugin ${pluginId}:`, e);
    }
  }

  /** Get backend hooks (for plugins to use) */
  getBackendHooks(): BackendHooks | null {
    return this.backendHooks;
  }
}

// =============================================================================
// Singleton Instance
// =============================================================================

export const pluginRegistry = new PluginRegistry();

// =============================================================================
// Helper function to get hooks for plugins
// =============================================================================

export function useBackendHooks(): BackendHooks {
  const hooks = pluginRegistry.getBackendHooks();
  if (!hooks) {
    throw new Error("Plugin registry not initialized. Backend hooks not available.");
  }
  return hooks;
}
