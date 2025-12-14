//! Backlink commands.

use crate::state::AppState;
use shared_types::BacklinkDto;
use tauri::State;

use super::{CommandError, Result};

/// Get backlinks for a note.
#[tauri::command]
pub async fn get_backlinks(state: State<'_, AppState>, note_id: i64) -> Result<Vec<BacklinkDto>> {
    let vault_guard = state.vault.read().await;
    let vault = vault_guard.as_ref().ok_or(CommandError::NoVaultOpen)?;

    vault
        .repo()
        .get_backlinks(note_id)
        .await
        .map_err(|e| CommandError::Vault(e.to_string()))
}
