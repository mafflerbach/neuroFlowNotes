//! Property commands - CRUD, management, and folder properties.
//!
//! Properties are stored in the database only (not in file frontmatter).
//! If users type frontmatter in the editor, it will be converted to DB
//! properties and removed from the file.

use crate::state::AppState;
use core_index::{parse_frontmatter, PropertyValue};
use shared_types::{
    ConvertFrontmatterResponse, DeletePropertyKeyRequest, FolderPropertyDto,
    MergePropertyKeysRequest, NoteWithPropertyValue, PropertyDto, PropertyOperationResult,
    PropertyValueInfo, PropertyWithInheritance, RenamePropertyKeyRequest,
    RenamePropertyValueRequest, SetFolderPropertyRequest, SetPropertyRequest,
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

/// Set a property for a note (DB-only, no file modification).
#[tauri::command]
#[instrument(skip(state))]
pub async fn set_property(state: State<'_, AppState>, request: SetPropertyRequest) -> Result<i64> {
    let vault_guard = state.vault.read().await;
    let vault = vault_guard.as_ref().ok_or(CommandError::NoVaultOpen)?;

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

/// Delete a property from a note (DB-only, no file modification).
#[tauri::command]
#[instrument(skip(state))]
pub async fn delete_property(state: State<'_, AppState>, note_id: i64, key: String) -> Result<()> {
    let vault_guard = state.vault.read().await;
    let vault = vault_guard.as_ref().ok_or(CommandError::NoVaultOpen)?;

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

// ============================================================================
// Frontmatter Conversion Commands
// ============================================================================

/// Convert frontmatter in content to DB properties.
///
/// Parses YAML frontmatter, stores properties in DB, and returns
/// the content without frontmatter. Tags are converted to inline format.
#[tauri::command]
#[instrument(skip(state, content))]
pub async fn convert_frontmatter_to_db(
    state: State<'_, AppState>,
    note_id: i64,
    content: String,
) -> Result<ConvertFrontmatterResponse> {
    let vault_guard = state.vault.read().await;
    let vault = vault_guard.as_ref().ok_or(CommandError::NoVaultOpen)?;

    let (frontmatter, body) = parse_frontmatter(&content);

    // If no frontmatter, return content as-is
    if frontmatter.content_start == 0 {
        return Ok(ConvertFrontmatterResponse {
            content,
            properties_converted: 0,
            tags_converted: vec![],
        });
    }

    let mut properties_converted = 0;

    // Store properties in DB (skip special keys like tags, aliases)
    for (key, value) in &frontmatter.properties {
        let key_lower = key.to_lowercase();

        // Skip tags and aliases - handled separately
        if key_lower == "tags" || key_lower == "tag" || key_lower == "aliases" || key_lower == "alias" {
            continue;
        }

        // Determine property type
        let prop_type = match value {
            PropertyValue::String(_) => Some("text"),
            PropertyValue::Number(_) => Some("number"),
            PropertyValue::Bool(_) => Some("boolean"),
            PropertyValue::List(_) => Some("list"),
            PropertyValue::Null => None,
        };

        let string_value = value.to_string_value();

        vault
            .repo()
            .set_property(note_id, key, string_value.as_deref(), prop_type)
            .await
            .map_err(|e| CommandError::Vault(e.to_string()))?;

        properties_converted += 1;
    }

    // Store aliases in DB
    if !frontmatter.aliases.is_empty() {
        vault
            .repo()
            .replace_aliases(note_id, &frontmatter.aliases)
            .await
            .map_err(|e| CommandError::Vault(e.to_string()))?;
    }

    // Convert frontmatter tags to inline format
    let tags_converted = frontmatter.tags.clone();

    // Build new content: inline tags + body
    let new_content = if !tags_converted.is_empty() {
        let inline_tags: String = tags_converted
            .iter()
            .map(|t| format!("#{}", t))
            .collect::<Vec<_>>()
            .join(" ");

        // Prepend inline tags to body
        let body_trimmed = body.trim_start();
        if body_trimmed.is_empty() {
            inline_tags
        } else {
            format!("{}\n\n{}", inline_tags, body_trimmed)
        }
    } else {
        body.to_string()
    };

    debug!(
        "Converted frontmatter: {} properties, {} tags",
        properties_converted,
        tags_converted.len()
    );

    Ok(ConvertFrontmatterResponse {
        content: new_content,
        properties_converted,
        tags_converted,
    })
}
