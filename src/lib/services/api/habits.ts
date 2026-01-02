/**
 * Habits API - habit tracker operations
 */

import { invoke } from "@tauri-apps/api/core";
import type {
  HabitDto,
  HabitEntryDto,
  CreateHabitRequest,
  UpdateHabitRequest,
  LogHabitEntryRequest,
  UpdateHabitEntryRequest,
  HabitTrackerResponse,
} from "../../types";

// ============================================================================
// Habit CRUD
// ============================================================================

/**
 * Create a new habit.
 */
export async function createHabit(request: CreateHabitRequest): Promise<number> {
  return invoke<number>("create_habit", { request });
}

/**
 * List all habits.
 */
export async function listHabits(includeArchived: boolean = false): Promise<HabitDto[]> {
  return invoke<HabitDto[]>("list_habits", { includeArchived });
}

/**
 * Get a habit by ID.
 */
export async function getHabit(id: number): Promise<HabitDto | null> {
  return invoke<HabitDto | null>("get_habit", { id });
}

/**
 * Update a habit.
 */
export async function updateHabit(request: UpdateHabitRequest): Promise<void> {
  return invoke("update_habit", { request });
}

/**
 * Delete a habit.
 */
export async function deleteHabit(id: number): Promise<void> {
  return invoke("delete_habit", { id });
}

/**
 * Archive a habit (soft delete).
 */
export async function archiveHabit(id: number): Promise<void> {
  return invoke("archive_habit", { id });
}

// ============================================================================
// Habit Entry Operations
// ============================================================================

/**
 * Log a habit entry.
 */
export async function logHabitEntry(request: LogHabitEntryRequest): Promise<number> {
  return invoke<number>("log_habit_entry", { request });
}

/**
 * Get habit entries for a habit within a date range.
 */
export async function getHabitEntries(
  habitId: number,
  startDate: string,
  endDate: string
): Promise<HabitEntryDto[]> {
  return invoke<HabitEntryDto[]>("get_habit_entries", { habitId, startDate, endDate });
}

/**
 * Update a habit entry.
 */
export async function updateHabitEntry(request: UpdateHabitEntryRequest): Promise<void> {
  return invoke("update_habit_entry", { request });
}

/**
 * Delete a habit entry.
 */
export async function deleteHabitEntry(id: number): Promise<void> {
  return invoke("delete_habit_entry", { id });
}

/**
 * Toggle a boolean habit for a date.
 * Returns true if the habit is now marked as done, false if undone.
 */
export async function toggleHabit(habitId: number, date: string): Promise<boolean> {
  return invoke<boolean>("toggle_habit", { habitId, date });
}

// ============================================================================
// Habit Tracker Embed
// ============================================================================

/**
 * Execute a habit tracker embed from YAML content.
 * Parses the YAML and executes the query, returning habits with entries.
 */
export async function executeHabitTrackerEmbed(
  yamlContent: string
): Promise<HabitTrackerResponse> {
  return invoke<HabitTrackerResponse>("execute_habit_tracker_embed", { yamlContent });
}
