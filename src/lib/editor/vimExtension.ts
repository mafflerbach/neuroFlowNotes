/**
 * Vim Mode Extension for CodeMirror
 * Provides optional vim keybindings for the editor
 */

import { vim } from "@replit/codemirror-vim";
import type { Extension } from "@codemirror/state";
import { getSetting } from "../services/settings";

/**
 * Returns vim extension if vim mode is enabled in settings
 */
export function vimExtension(): Extension[] {
  if (getSetting("vimMode")) {
    return [vim()];
  }
  return [];
}

/**
 * Check if vim mode is currently enabled
 */
export function isVimModeEnabled(): boolean {
  return getSetting("vimMode");
}
