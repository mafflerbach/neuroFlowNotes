//! Schedule block commands and notes by date.

use crate::state::AppState;
use shared_types::{
    CreateScheduleBlockRequest, NoteForDate, ScheduleBlockDto, UpdateScheduleBlockRequest,
};
use tauri::State;
use tracing::instrument;

use super::{CommandError, Result};

// ============================================================================
// Schedule Block Commands
// ============================================================================

/// Create a schedule block.
#[tauri::command]
#[instrument(skip(state))]
pub async fn create_schedule_block(
    state: State<'_, AppState>,
    request: CreateScheduleBlockRequest,
) -> Result<i64> {
    let vault_guard = state.vault.read().await;
    let vault = vault_guard.as_ref().ok_or(CommandError::NoVaultOpen)?;

    vault
        .repo()
        .create_schedule_block(
            request.note_id,
            &request.date.to_string(),
            &request.start_time.to_string(),
            &request.end_time.to_string(),
            request.label.as_deref(),
            request.color.as_deref(),
            request.context.as_deref(),
            request.rrule.as_deref(),
        )
        .await
        .map_err(|e| CommandError::Vault(e.to_string()))
}

/// Get schedule blocks for a date range.
#[tauri::command]
pub async fn get_schedule_blocks(
    state: State<'_, AppState>,
    start_date: String,
    end_date: String,
) -> Result<Vec<ScheduleBlockDto>> {
    let vault_guard = state.vault.read().await;
    let vault = vault_guard.as_ref().ok_or(CommandError::NoVaultOpen)?;

    vault
        .repo()
        .get_schedule_blocks_for_range(&start_date, &end_date)
        .await
        .map_err(|e| CommandError::Vault(e.to_string()))
}

/// Get schedule blocks for a single date.
#[tauri::command]
pub async fn get_schedule_blocks_for_date(
    state: State<'_, AppState>,
    date: String,
) -> Result<Vec<ScheduleBlockDto>> {
    let vault_guard = state.vault.read().await;
    let vault = vault_guard.as_ref().ok_or(CommandError::NoVaultOpen)?;

    vault
        .repo()
        .get_schedule_blocks_for_date(&date)
        .await
        .map_err(|e| CommandError::Vault(e.to_string()))
}

/// Get schedule blocks linked to a specific note.
#[tauri::command]
pub async fn get_schedule_blocks_for_note(
    state: State<'_, AppState>,
    note_id: i64,
) -> Result<Vec<ScheduleBlockDto>> {
    let vault_guard = state.vault.read().await;
    let vault = vault_guard.as_ref().ok_or(CommandError::NoVaultOpen)?;

    vault
        .repo()
        .get_schedule_blocks_for_note(note_id)
        .await
        .map_err(|e| CommandError::Vault(e.to_string()))
}

/// Update a schedule block.
#[tauri::command]
#[instrument(skip(state))]
pub async fn update_schedule_block(
    state: State<'_, AppState>,
    request: UpdateScheduleBlockRequest,
) -> Result<()> {
    let vault_guard = state.vault.read().await;
    let vault = vault_guard.as_ref().ok_or(CommandError::NoVaultOpen)?;

    vault
        .repo()
        .update_schedule_block(
            request.id,
            request.note_id,
            request.date.as_ref().map(|d| d.to_string()).as_deref(),
            request
                .start_time
                .as_ref()
                .map(|t| t.to_string())
                .as_deref(),
            request.end_time.as_ref().map(|t| t.to_string()).as_deref(),
            request.label.as_deref(),
            request.color.as_deref(),
            request.context.as_deref(),
            request.rrule.as_deref(),
        )
        .await
        .map_err(|e| CommandError::Vault(e.to_string()))
}

/// Delete a schedule block.
#[tauri::command]
#[instrument(skip(state))]
pub async fn delete_schedule_block(state: State<'_, AppState>, id: i64) -> Result<()> {
    let vault_guard = state.vault.read().await;
    let vault = vault_guard.as_ref().ok_or(CommandError::NoVaultOpen)?;

    vault
        .repo()
        .delete_schedule_block(id)
        .await
        .map_err(|e| CommandError::Vault(e.to_string()))
}

// ============================================================================
// Notes by Date Commands
// ============================================================================

/// Get notes for a specific date (ordered by: scheduled > journal > created).
#[tauri::command]
pub async fn get_notes_for_date(
    state: State<'_, AppState>,
    date: String,
) -> Result<Vec<NoteForDate>> {
    let vault_guard = state.vault.read().await;
    let vault = vault_guard.as_ref().ok_or(CommandError::NoVaultOpen)?;

    vault
        .repo()
        .get_notes_for_date(&date)
        .await
        .map_err(|e| CommandError::Vault(e.to_string()))
}

/// Get notes for a date range (for weekly/monthly views).
#[tauri::command]
pub async fn get_notes_for_date_range(
    state: State<'_, AppState>,
    start_date: String,
    end_date: String,
) -> Result<Vec<(String, Vec<NoteForDate>)>> {
    let vault_guard = state.vault.read().await;
    let vault = vault_guard.as_ref().ok_or(CommandError::NoVaultOpen)?;

    vault
        .repo()
        .get_notes_for_date_range(&start_date, &end_date)
        .await
        .map_err(|e| CommandError::Vault(e.to_string()))
}
