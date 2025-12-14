//! Tag commands.

use crate::state::AppState;
use shared_types::TagDto;
use tauri::State;

use super::{CommandError, Result};

/// List all tags with counts.
#[tauri::command]
pub async fn list_tags(state: State<'_, AppState>) -> Result<Vec<TagDto>> {
    let vault_guard = state.vault.read().await;
    let vault = vault_guard.as_ref().ok_or(CommandError::NoVaultOpen)?;

    vault
        .repo()
        .list_tags()
        .await
        .map_err(|e| CommandError::Vault(e.to_string()))
}
