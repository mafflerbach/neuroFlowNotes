//! Import commands - vault import operations.

use crate::state::AppState;
use shared_types::{ImportResult, ImportVaultRequest};
use tauri::{AppHandle, Emitter, State};
use tracing::{info, instrument};

use super::{CommandError, Result};

/// Import an Obsidian vault into the current vault.
///
/// Copies all markdown files and assets, preserving folder structure.
/// Parses YAML frontmatter and converts to properties.
/// Merges frontmatter tags with inline tags.
#[tauri::command]
#[instrument(skip(state, app))]
pub async fn import_obsidian_vault(
    state: State<'_, AppState>,
    app: AppHandle,
    request: ImportVaultRequest,
) -> Result<ImportResult> {
    info!("Importing Obsidian vault from: {}", request.source_path);

    let vault_guard = state.vault.read().await;
    let vault = vault_guard.as_ref().ok_or(CommandError::NoVaultOpen)?;

    // Create progress channel
    let (tx, mut rx) = tokio::sync::mpsc::channel(100);
    let app_clone = app.clone();

    // Spawn task to forward progress to frontend
    tokio::spawn(async move {
        while let Some(progress) = rx.recv().await {
            let _ = app_clone.emit("import:progress", progress);
        }
    });

    // Run import
    let result = core_domain::import_obsidian_vault(
        vault,
        std::path::Path::new(&request.source_path),
        request.target_subfolder.as_deref(),
        Some(tx),
    )
    .await
    .map_err(|e| CommandError::Vault(e.to_string()))?;

    // Trigger re-index to pick up all changes
    vault
        .full_index()
        .await
        .map_err(|e| CommandError::Vault(e.to_string()))?;

    info!(
        "Import complete: {} notes, {} properties",
        result.notes_imported, result.properties_imported
    );

    Ok(result)
}
