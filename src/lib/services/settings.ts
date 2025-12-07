/**
 * Settings service - persists user preferences to localStorage.
 */

import type { CalendarView } from "../stores/workspace.svelte";

const STORAGE_KEY = "neuroflow-settings";

export type Theme = "system" | "light" | "dark";

export interface AppSettings {
  lastVaultPath: string | null;
  defaultCalendarView: CalendarView;
  theme: Theme;
}

const defaultSettings: AppSettings = {
  lastVaultPath: null,
  defaultCalendarView: "weekly",
  theme: "system",
};

/**
 * Load settings from localStorage.
 */
export function loadSettings(): AppSettings {
  try {
    const stored = localStorage.getItem(STORAGE_KEY);
    if (stored) {
      const parsed = JSON.parse(stored);
      return { ...defaultSettings, ...parsed };
    }
  } catch (e) {
    console.error("[Settings] Failed to load settings:", e);
  }
  return { ...defaultSettings };
}

/**
 * Save settings to localStorage.
 */
export function saveSettings(settings: Partial<AppSettings>): void {
  try {
    const current = loadSettings();
    const updated = { ...current, ...settings };
    localStorage.setItem(STORAGE_KEY, JSON.stringify(updated));
  } catch (e) {
    console.error("[Settings] Failed to save settings:", e);
  }
}

/**
 * Get a single setting value.
 */
export function getSetting<K extends keyof AppSettings>(key: K): AppSettings[K] {
  return loadSettings()[key];
}

/**
 * Set a single setting value.
 */
export function setSetting<K extends keyof AppSettings>(key: K, value: AppSettings[K]): void {
  saveSettings({ [key]: value });
}
