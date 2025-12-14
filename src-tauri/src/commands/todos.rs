//! Todo/Task commands.

use crate::state::AppState;
use shared_types::{TaskQuery, TaskWithContext, TodoDto};
use tauri::State;
use tracing::instrument;

use super::{CommandError, Result};

/// Get todos for a specific note.
#[tauri::command]
pub async fn get_todos_for_note(state: State<'_, AppState>, note_id: i64) -> Result<Vec<TodoDto>> {
    let vault_guard = state.vault.read().await;
    let vault = vault_guard.as_ref().ok_or(CommandError::NoVaultOpen)?;

    vault
        .get_todos_for_note(note_id)
        .await
        .map_err(|e| CommandError::Vault(e.to_string()))
}

/// Toggle a todo's completion status.
#[tauri::command]
#[instrument(skip(state))]
pub async fn toggle_todo(
    state: State<'_, AppState>,
    todo_id: i64,
    completed: bool,
) -> Result<()> {
    let vault_guard = state.vault.read().await;
    let vault = vault_guard.as_ref().ok_or(CommandError::NoVaultOpen)?;

    vault
        .toggle_todo(todo_id, completed)
        .await
        .map_err(|e| CommandError::Vault(e.to_string()))
}

/// Get all incomplete todos.
#[tauri::command]
pub async fn get_incomplete_todos(state: State<'_, AppState>) -> Result<Vec<TodoDto>> {
    let vault_guard = state.vault.read().await;
    let vault = vault_guard.as_ref().ok_or(CommandError::NoVaultOpen)?;

    vault
        .get_incomplete_todos()
        .await
        .map_err(|e| CommandError::Vault(e.to_string()))
}

/// Query tasks with filters, returning enriched context from parent notes.
#[tauri::command]
pub async fn query_tasks(
    state: State<'_, AppState>,
    query: TaskQuery,
) -> Result<Vec<TaskWithContext>> {
    let vault_guard = state.vault.read().await;
    let vault = vault_guard.as_ref().ok_or(CommandError::NoVaultOpen)?;

    vault
        .repo()
        .query_tasks(&query)
        .await
        .map_err(|e| CommandError::Vault(e.to_string()))
}

/// Get all distinct contexts used in tasks.
#[tauri::command]
pub async fn get_task_contexts(state: State<'_, AppState>) -> Result<Vec<String>> {
    let vault_guard = state.vault.read().await;
    let vault = vault_guard.as_ref().ok_or(CommandError::NoVaultOpen)?;

    vault
        .repo()
        .get_task_contexts()
        .await
        .map_err(|e| CommandError::Vault(e.to_string()))
}
