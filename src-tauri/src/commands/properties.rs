//! Property commands - CRUD, management, and folder properties.

use std::path::Path;

use crate::state::AppState;
use core_index::{delete_frontmatter_property, set_frontmatter_property};
use shared_types::{
    DeletePropertyKeyRequest, FolderPropertyDto, MergePropertyKeysRequest, NoteWithPropertyValue,
    PropertyDto, PropertyOperationResult, PropertyValueInfo, PropertyWithInheritance,
    RenamePropertyKeyRequest, RenamePropertyValueRequest, SetFolderPropertyRequest,
    SetPropertyRequest,
};
use tauri::State;
use tracing::{debug, instrument};

use super::{CommandError, Result};

// ============================================================================
// Basic Property Commands
// ============================================================================

/// Get all properties for a note.
#[tauri::command]
pub async fn get_properties(state: State<'_, AppState>, note_id: i64) -> Result<Vec<PropertyDto>> {
    let vault_guard = state.vault.read().await;
    let vault = vault_guard.as_ref().ok_or(CommandError::NoVaultOpen)?;

    vault
        .repo()
        .get_properties_for_note(note_id)
        .await
        .map_err(|e| CommandError::Vault(e.to_string()))
}

/// Set a property for a note - updates both the YAML frontmatter and database.
#[tauri::command]
#[instrument(skip(state))]
pub async fn set_property(state: State<'_, AppState>, request: SetPropertyRequest) -> Result<i64> {
    let vault_guard = state.vault.read().await;
    let vault = vault_guard.as_ref().ok_or(CommandError::NoVaultOpen)?;

    // Get the note to find its path
    let note = vault
        .repo()
        .get_note(request.note_id)
        .await
        .map_err(|e| CommandError::Vault(e.to_string()))?;

    // Read the current file content
    let path = Path::new(&note.path);
    let content = vault
        .fs()
        .read_file(path)
        .await
        .map_err(|e| CommandError::Vault(e.to_string()))?;

    // Update the frontmatter
    let new_content = set_frontmatter_property(
        &content,
        &request.key,
        request.value.as_deref(),
        request.property_type.as_deref(),
    );

    // Write the updated content back to the file
    vault
        .fs()
        .write_file(path, &new_content)
        .await
        .map_err(|e| CommandError::Vault(e.to_string()))?;

    debug!("Updated frontmatter property '{}' in {}", request.key, note.path);

    // Update the database as well (for immediate query availability)
    vault
        .repo()
        .set_property(
            request.note_id,
            &request.key,
            request.value.as_deref(),
            request.property_type.as_deref(),
        )
        .await
        .map_err(|e| CommandError::Vault(e.to_string()))
}

/// Delete a property from a note - removes from both YAML frontmatter and database.
#[tauri::command]
#[instrument(skip(state))]
pub async fn delete_property(state: State<'_, AppState>, note_id: i64, key: String) -> Result<()> {
    let vault_guard = state.vault.read().await;
    let vault = vault_guard.as_ref().ok_or(CommandError::NoVaultOpen)?;

    // Get the note to find its path
    let note = vault
        .repo()
        .get_note(note_id)
        .await
        .map_err(|e| CommandError::Vault(e.to_string()))?;

    // Read the current file content
    let path = Path::new(&note.path);
    let content = vault
        .fs()
        .read_file(path)
        .await
        .map_err(|e| CommandError::Vault(e.to_string()))?;

    // Remove from frontmatter
    let new_content = delete_frontmatter_property(&content, &key);

    // Write the updated content back to the file
    vault
        .fs()
        .write_file(path, &new_content)
        .await
        .map_err(|e| CommandError::Vault(e.to_string()))?;

    debug!("Deleted frontmatter property '{}' from {}", key, note.path);

    // Delete from database as well
    vault
        .repo()
        .delete_property(note_id, &key)
        .await
        .map_err(|e| CommandError::Vault(e.to_string()))
}

// ============================================================================
// Property Management Commands (Bulk Operations)
// ============================================================================

/// Rename a property key across all notes.
#[tauri::command]
#[instrument(skip(state))]
pub async fn rename_property_key(
    state: State<'_, AppState>,
    request: RenamePropertyKeyRequest,
) -> Result<PropertyOperationResult> {
    let vault_guard = state.vault.read().await;
    let vault = vault_guard.as_ref().ok_or(CommandError::NoVaultOpen)?;

    let (affected_count, notes_affected) = vault
        .repo()
        .rename_property_key(&request.old_key, &request.new_key)
        .await
        .map_err(|e| CommandError::Vault(e.to_string()))?;

    Ok(PropertyOperationResult {
        affected_count,
        notes_affected,
    })
}

/// Rename a property value across all notes with that key.
#[tauri::command]
#[instrument(skip(state))]
pub async fn rename_property_value(
    state: State<'_, AppState>,
    request: RenamePropertyValueRequest,
) -> Result<PropertyOperationResult> {
    let vault_guard = state.vault.read().await;
    let vault = vault_guard.as_ref().ok_or(CommandError::NoVaultOpen)?;

    let (affected_count, notes_affected) = vault
        .repo()
        .rename_property_value(&request.key, &request.old_value, &request.new_value)
        .await
        .map_err(|e| CommandError::Vault(e.to_string()))?;

    Ok(PropertyOperationResult {
        affected_count,
        notes_affected,
    })
}

/// Merge two property keys (rename source to target).
#[tauri::command]
#[instrument(skip(state))]
pub async fn merge_property_keys(
    state: State<'_, AppState>,
    request: MergePropertyKeysRequest,
) -> Result<PropertyOperationResult> {
    let vault_guard = state.vault.read().await;
    let vault = vault_guard.as_ref().ok_or(CommandError::NoVaultOpen)?;

    let (affected_count, notes_affected) = vault
        .repo()
        .merge_property_keys(&request.source_key, &request.target_key)
        .await
        .map_err(|e| CommandError::Vault(e.to_string()))?;

    Ok(PropertyOperationResult {
        affected_count,
        notes_affected,
    })
}

/// Delete a property key from all notes.
#[tauri::command]
#[instrument(skip(state))]
pub async fn delete_property_key(
    state: State<'_, AppState>,
    request: DeletePropertyKeyRequest,
) -> Result<PropertyOperationResult> {
    let vault_guard = state.vault.read().await;
    let vault = vault_guard.as_ref().ok_or(CommandError::NoVaultOpen)?;

    let (affected_count, notes_affected) = vault
        .repo()
        .delete_property_key(&request.key)
        .await
        .map_err(|e| CommandError::Vault(e.to_string()))?;

    Ok(PropertyOperationResult {
        affected_count,
        notes_affected,
    })
}

/// Get all distinct values for a property key with usage counts.
#[tauri::command]
pub async fn get_property_values_with_counts(
    state: State<'_, AppState>,
    key: String,
) -> Result<Vec<PropertyValueInfo>> {
    let vault_guard = state.vault.read().await;
    let vault = vault_guard.as_ref().ok_or(CommandError::NoVaultOpen)?;

    let values = vault
        .repo()
        .get_property_values_with_counts(&key)
        .await
        .map_err(|e| CommandError::Vault(e.to_string()))?;

    Ok(values
        .into_iter()
        .map(|(value, usage_count)| PropertyValueInfo { value, usage_count })
        .collect())
}

/// Get all notes that have a specific property key, along with their value.
#[tauri::command]
pub async fn get_notes_with_property(
    state: State<'_, AppState>,
    key: String,
) -> Result<Vec<NoteWithPropertyValue>> {
    let vault_guard = state.vault.read().await;
    let vault = vault_guard.as_ref().ok_or(CommandError::NoVaultOpen)?;

    vault
        .repo()
        .get_notes_with_property(&key)
        .await
        .map_err(|e| CommandError::Vault(e.to_string()))
}

/// Get all notes that have a specific property key and value.
#[tauri::command]
pub async fn get_notes_with_property_value(
    state: State<'_, AppState>,
    key: String,
    value: String,
) -> Result<Vec<NoteWithPropertyValue>> {
    let vault_guard = state.vault.read().await;
    let vault = vault_guard.as_ref().ok_or(CommandError::NoVaultOpen)?;

    vault
        .repo()
        .get_notes_with_property_value(&key, &value)
        .await
        .map_err(|e| CommandError::Vault(e.to_string()))
}

// ============================================================================
// Folder Property Commands
// ============================================================================

/// Get all properties for a folder.
#[tauri::command]
pub async fn get_folder_properties(
    state: State<'_, AppState>,
    folder_path: String,
) -> Result<Vec<FolderPropertyDto>> {
    let vault_guard = state.vault.read().await;
    let vault = vault_guard.as_ref().ok_or(CommandError::NoVaultOpen)?;

    vault
        .repo()
        .get_folder_properties(&folder_path)
        .await
        .map_err(|e| CommandError::Vault(e.to_string()))
}

/// Set a folder property.
#[tauri::command]
#[instrument(skip(state))]
pub async fn set_folder_property(
    state: State<'_, AppState>,
    request: SetFolderPropertyRequest,
) -> Result<i64> {
    let vault_guard = state.vault.read().await;
    let vault = vault_guard.as_ref().ok_or(CommandError::NoVaultOpen)?;

    vault
        .repo()
        .set_folder_property(
            &request.folder_path,
            &request.key,
            request.value.as_deref(),
            request.property_type.as_deref(),
        )
        .await
        .map_err(|e| CommandError::Vault(e.to_string()))
}

/// Delete a folder property.
#[tauri::command]
#[instrument(skip(state))]
pub async fn delete_folder_property(
    state: State<'_, AppState>,
    folder_path: String,
    key: String,
) -> Result<()> {
    let vault_guard = state.vault.read().await;
    let vault = vault_guard.as_ref().ok_or(CommandError::NoVaultOpen)?;

    vault
        .repo()
        .delete_folder_property(&folder_path, &key)
        .await
        .map_err(|e| CommandError::Vault(e.to_string()))
}

/// Get properties for a note with inheritance info.
/// Returns note's own properties plus inherited folder properties.
#[tauri::command]
pub async fn get_properties_with_inheritance(
    state: State<'_, AppState>,
    note_id: i64,
    note_path: String,
) -> Result<Vec<PropertyWithInheritance>> {
    let vault_guard = state.vault.read().await;
    let vault = vault_guard.as_ref().ok_or(CommandError::NoVaultOpen)?;

    vault
        .repo()
        .get_properties_with_inheritance(note_id, &note_path)
        .await
        .map_err(|e| CommandError::Vault(e.to_string()))
}

/// Get all folders that have properties defined.
#[tauri::command]
pub async fn get_folders_with_properties(state: State<'_, AppState>) -> Result<Vec<String>> {
    let vault_guard = state.vault.read().await;
    let vault = vault_guard.as_ref().ok_or(CommandError::NoVaultOpen)?;

    vault
        .repo()
        .get_folders_with_properties()
        .await
        .map_err(|e| CommandError::Vault(e.to_string()))
}
