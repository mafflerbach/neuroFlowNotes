//! Habit tracker commands - habit CRUD and entry logging.

use crate::state::AppState;
use shared_types::{
    CreateHabitRequest, HabitDto, HabitEntryDto, HabitTrackerQuery, HabitTrackerResponse,
    LogHabitEntryRequest, UpdateHabitEntryRequest, UpdateHabitRequest,
};
use tauri::State;
use tracing::instrument;

use super::{CommandError, Result};

// ============================================================================
// Habit CRUD Commands
// ============================================================================

/// Create a new habit.
#[tauri::command]
#[instrument(skip(state))]
pub async fn create_habit(state: State<'_, AppState>, request: CreateHabitRequest) -> Result<i64> {
    let vault_guard = state.vault.read().await;
    let vault = vault_guard.as_ref().ok_or(CommandError::NoVaultOpen)?;

    vault
        .repo()
        .create_habit(&request)
        .await
        .map_err(|e| CommandError::Vault(e.to_string()))
}

/// List all habits.
#[tauri::command]
pub async fn list_habits(
    state: State<'_, AppState>,
    include_archived: bool,
) -> Result<Vec<HabitDto>> {
    let vault_guard = state.vault.read().await;
    let vault = vault_guard.as_ref().ok_or(CommandError::NoVaultOpen)?;

    vault
        .repo()
        .list_habits(include_archived)
        .await
        .map_err(|e| CommandError::Vault(e.to_string()))
}

/// Get a habit by ID.
#[tauri::command]
pub async fn get_habit(state: State<'_, AppState>, id: i64) -> Result<Option<HabitDto>> {
    let vault_guard = state.vault.read().await;
    let vault = vault_guard.as_ref().ok_or(CommandError::NoVaultOpen)?;

    vault
        .repo()
        .get_habit(id)
        .await
        .map_err(|e| CommandError::Vault(e.to_string()))
}

/// Update a habit.
#[tauri::command]
#[instrument(skip(state))]
pub async fn update_habit(state: State<'_, AppState>, request: UpdateHabitRequest) -> Result<()> {
    let vault_guard = state.vault.read().await;
    let vault = vault_guard.as_ref().ok_or(CommandError::NoVaultOpen)?;

    vault
        .repo()
        .update_habit(&request)
        .await
        .map_err(|e| CommandError::Vault(e.to_string()))
}

/// Delete a habit.
#[tauri::command]
#[instrument(skip(state))]
pub async fn delete_habit(state: State<'_, AppState>, id: i64) -> Result<()> {
    let vault_guard = state.vault.read().await;
    let vault = vault_guard.as_ref().ok_or(CommandError::NoVaultOpen)?;

    vault
        .repo()
        .delete_habit(id)
        .await
        .map_err(|e| CommandError::Vault(e.to_string()))
}

/// Archive a habit (soft delete).
#[tauri::command]
#[instrument(skip(state))]
pub async fn archive_habit(state: State<'_, AppState>, id: i64) -> Result<()> {
    let vault_guard = state.vault.read().await;
    let vault = vault_guard.as_ref().ok_or(CommandError::NoVaultOpen)?;

    vault
        .repo()
        .archive_habit(id)
        .await
        .map_err(|e| CommandError::Vault(e.to_string()))
}

// ============================================================================
// Habit Entry Commands
// ============================================================================

/// Log a habit entry.
#[tauri::command]
#[instrument(skip(state))]
pub async fn log_habit_entry(
    state: State<'_, AppState>,
    request: LogHabitEntryRequest,
) -> Result<i64> {
    let vault_guard = state.vault.read().await;
    let vault = vault_guard.as_ref().ok_or(CommandError::NoVaultOpen)?;

    vault
        .repo()
        .log_habit_entry(&request)
        .await
        .map_err(|e| CommandError::Vault(e.to_string()))
}

/// Get habit entries for a habit within a date range.
#[tauri::command]
pub async fn get_habit_entries(
    state: State<'_, AppState>,
    habit_id: i64,
    start_date: String,
    end_date: String,
) -> Result<Vec<HabitEntryDto>> {
    let vault_guard = state.vault.read().await;
    let vault = vault_guard.as_ref().ok_or(CommandError::NoVaultOpen)?;

    vault
        .repo()
        .get_habit_entries(habit_id, &start_date, &end_date)
        .await
        .map_err(|e| CommandError::Vault(e.to_string()))
}

/// Update a habit entry.
#[tauri::command]
#[instrument(skip(state))]
pub async fn update_habit_entry(
    state: State<'_, AppState>,
    request: UpdateHabitEntryRequest,
) -> Result<()> {
    let vault_guard = state.vault.read().await;
    let vault = vault_guard.as_ref().ok_or(CommandError::NoVaultOpen)?;

    vault
        .repo()
        .update_habit_entry(&request)
        .await
        .map_err(|e| CommandError::Vault(e.to_string()))
}

/// Delete a habit entry.
#[tauri::command]
#[instrument(skip(state))]
pub async fn delete_habit_entry(state: State<'_, AppState>, id: i64) -> Result<()> {
    let vault_guard = state.vault.read().await;
    let vault = vault_guard.as_ref().ok_or(CommandError::NoVaultOpen)?;

    vault
        .repo()
        .delete_habit_entry(id)
        .await
        .map_err(|e| CommandError::Vault(e.to_string()))
}

/// Toggle a boolean habit for a date.
/// Returns true if the habit is now marked as done, false if undone.
#[tauri::command]
#[instrument(skip(state))]
pub async fn toggle_habit(state: State<'_, AppState>, habit_id: i64, date: String) -> Result<bool> {
    let vault_guard = state.vault.read().await;
    let vault = vault_guard.as_ref().ok_or(CommandError::NoVaultOpen)?;

    vault
        .repo()
        .toggle_habit_for_date(habit_id, &date)
        .await
        .map_err(|e| CommandError::Vault(e.to_string()))
}

// ============================================================================
// Habit Tracker Embed Commands
// ============================================================================

/// Execute a habit tracker embed query from YAML content.
#[tauri::command]
pub async fn execute_habit_tracker_embed(
    state: State<'_, AppState>,
    yaml_content: String,
) -> Result<HabitTrackerResponse> {
    let vault_guard = state.vault.read().await;
    let vault = vault_guard.as_ref().ok_or(CommandError::NoVaultOpen)?;

    // Parse YAML content into HabitTrackerQuery
    let query: HabitTrackerQuery = match serde_yaml::from_str(&yaml_content) {
        Ok(q) => q,
        Err(e) => {
            return Ok(HabitTrackerResponse {
                query: HabitTrackerQuery::default(),
                habits: vec![],
                date_range_start: String::new(),
                date_range_end: String::new(),
                error: Some(format!("Failed to parse habit tracker config: {}", e)),
            });
        }
    };

    vault
        .repo()
        .execute_habit_tracker_query(&query)
        .await
        .map_err(|e| CommandError::Vault(e.to_string()))
}
