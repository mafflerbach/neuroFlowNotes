//! Vault commands - opening, closing, and vault info.

use crate::state::AppState;
use core_domain::Vault;
use shared_types::VaultInfo;
use tauri::{AppHandle, Emitter, State};
use tracing::{info, instrument};

use super::{CommandError, Result};

/// Open a vault at the given path.
#[tauri::command]
#[instrument(skip(state, app))]
pub async fn open_vault(
    state: State<'_, AppState>,
    app: AppHandle,
    path: String,
) -> Result<VaultInfo> {
    info!("Opening vault: {}", path);

    // Open the vault
    let mut vault = Vault::open(&path)
        .await
        .map_err(|e| CommandError::Vault(e.to_string()))?;

    // Subscribe to events and forward to frontend
    let mut rx = vault.subscribe();
    let app_clone = app.clone();
    tokio::spawn(async move {
        while let Ok(event) = rx.recv().await {
            match event {
                core_domain::vault::VaultEvent::NotesUpdated(ids) => {
                    let _ = app_clone.emit(
                        "notes:updated",
                        shared_types::NotesUpdatedPayload { note_ids: ids },
                    );
                }
                core_domain::vault::VaultEvent::NotesDeleted(ids) => {
                    let _ = app_clone.emit(
                        "notes:deleted",
                        shared_types::NotesDeletedPayload { note_ids: ids },
                    );
                }
                core_domain::vault::VaultEvent::IndexComplete(payload) => {
                    let _ = app_clone.emit("index:complete", payload);
                }
            }
        }
    });

    // Perform initial index
    vault
        .full_index()
        .await
        .map_err(|e| CommandError::Vault(e.to_string()))?;

    // Start file watcher
    vault
        .start_watcher()
        .await
        .map_err(|e| CommandError::Vault(e.to_string()))?;

    // Get vault info
    let info = vault
        .info()
        .await
        .map_err(|e| CommandError::Vault(e.to_string()))?;

    // Store in state
    *state.vault.write().await = Some(vault);

    Ok(info)
}

/// Close the current vault.
#[tauri::command]
#[instrument(skip(state))]
pub async fn close_vault(state: State<'_, AppState>) -> Result<()> {
    info!("Closing vault");

    let mut vault_guard = state.vault.write().await;
    if let Some(mut vault) = vault_guard.take() {
        vault.stop_watcher().await;
    }

    Ok(())
}

/// Get information about the current vault.
#[tauri::command]
pub async fn get_vault_info(state: State<'_, AppState>) -> Result<Option<VaultInfo>> {
    let vault_guard = state.vault.read().await;
    if let Some(vault) = vault_guard.as_ref() {
        let info = vault
            .info()
            .await
            .map_err(|e| CommandError::Vault(e.to_string()))?;
        Ok(Some(info))
    } else {
        Ok(None)
    }
}
