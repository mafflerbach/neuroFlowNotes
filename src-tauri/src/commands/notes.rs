//! Note commands - CRUD operations and folder management.

use crate::state::AppState;
use shared_types::{NoteContent, NoteDto, NoteListItem};
use tauri::State;
use tracing::instrument;

use super::{CommandError, Result};

/// List all notes in the vault.
#[tauri::command]
pub async fn list_notes(state: State<'_, AppState>) -> Result<Vec<NoteListItem>> {
    let vault_guard = state.vault.read().await;
    let vault = vault_guard.as_ref().ok_or(CommandError::NoVaultOpen)?;

    vault
        .list_notes()
        .await
        .map_err(|e| CommandError::Vault(e.to_string()))
}

/// Get a note by ID.
#[tauri::command]
pub async fn get_note(state: State<'_, AppState>, note_id: i64) -> Result<NoteDto> {
    let vault_guard = state.vault.read().await;
    let vault = vault_guard.as_ref().ok_or(CommandError::NoVaultOpen)?;

    vault
        .repo()
        .get_note(note_id)
        .await
        .map_err(|e| CommandError::Vault(e.to_string()))
}

/// Get a note's content.
#[tauri::command]
pub async fn get_note_content(state: State<'_, AppState>, path: String) -> Result<NoteContent> {
    let vault_guard = state.vault.read().await;
    let vault = vault_guard.as_ref().ok_or(CommandError::NoVaultOpen)?;

    let content = vault
        .read_note(&path)
        .await
        .map_err(|e| CommandError::Vault(e.to_string()))?;

    let note = vault
        .repo()
        .get_note_by_path(&path)
        .await
        .map_err(|e| CommandError::Vault(e.to_string()))?;

    Ok(NoteContent {
        id: note.id,
        path: note.path,
        content,
    })
}

/// Save a note's content.
#[tauri::command]
#[instrument(skip(state, content))]
pub async fn save_note(state: State<'_, AppState>, path: String, content: String) -> Result<i64> {
    let vault_guard = state.vault.read().await;
    let vault = vault_guard.as_ref().ok_or(CommandError::NoVaultOpen)?;

    vault
        .write_note(&path, &content)
        .await
        .map_err(|e| CommandError::Vault(e.to_string()))
}

/// Rename a note (file and database path).
#[tauri::command]
#[instrument(skip(state))]
pub async fn rename_note(
    state: State<'_, AppState>,
    old_path: String,
    new_path: String,
) -> Result<i64> {
    let vault_guard = state.vault.read().await;
    let vault = vault_guard.as_ref().ok_or(CommandError::NoVaultOpen)?;

    vault
        .rename_note(&old_path, &new_path)
        .await
        .map_err(|e| CommandError::Vault(e.to_string()))
}

/// Delete a note (file and database record).
#[tauri::command]
#[instrument(skip(state))]
pub async fn delete_note(state: State<'_, AppState>, path: String) -> Result<Option<i64>> {
    let vault_guard = state.vault.read().await;
    let vault = vault_guard.as_ref().ok_or(CommandError::NoVaultOpen)?;

    vault
        .delete_note(&path)
        .await
        .map_err(|e| CommandError::Vault(e.to_string()))
}

/// Create a folder in the vault.
#[tauri::command]
#[instrument(skip(state))]
pub async fn create_folder(state: State<'_, AppState>, path: String) -> Result<()> {
    let vault_guard = state.vault.read().await;
    let vault = vault_guard.as_ref().ok_or(CommandError::NoVaultOpen)?;

    vault
        .create_folder(&path)
        .await
        .map_err(|e| CommandError::Vault(e.to_string()))
}

/// Rename/move a folder and update all note paths within it.
#[tauri::command]
#[instrument(skip(state))]
pub async fn rename_folder(
    state: State<'_, AppState>,
    old_path: String,
    new_path: String,
) -> Result<Vec<i64>> {
    let vault_guard = state.vault.read().await;
    let vault = vault_guard.as_ref().ok_or(CommandError::NoVaultOpen)?;

    vault
        .rename_folder(&old_path, &new_path)
        .await
        .map_err(|e| CommandError::Vault(e.to_string()))
}

/// Delete a folder and all its contents.
#[tauri::command]
#[instrument(skip(state))]
pub async fn delete_folder(state: State<'_, AppState>, path: String) -> Result<Vec<i64>> {
    let vault_guard = state.vault.read().await;
    let vault = vault_guard.as_ref().ok_or(CommandError::NoVaultOpen)?;

    vault
        .delete_folder(&path)
        .await
        .map_err(|e| CommandError::Vault(e.to_string()))
}
