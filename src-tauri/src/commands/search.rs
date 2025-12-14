//! Search commands.

use crate::state::AppState;
use shared_types::SearchResult;
use tauri::State;

use super::{CommandError, Result};

/// Search notes.
#[tauri::command]
pub async fn search_notes(
    state: State<'_, AppState>,
    query: String,
    limit: Option<i32>,
) -> Result<Vec<SearchResult>> {
    let vault_guard = state.vault.read().await;
    let vault = vault_guard.as_ref().ok_or(CommandError::NoVaultOpen)?;

    vault
        .repo()
        .search(&query, limit.unwrap_or(50))
        .await
        .map_err(|e| CommandError::Vault(e.to_string()))
}
