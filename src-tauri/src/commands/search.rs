//! Search commands.

use crate::state::AppState;
use core_embedding::{hybrid_search, EmbeddingClient};
use core_storage::extract_content_preview;
use shared_types::{
    EmbeddingSettings, EmbeddingStatus, HybridSearchResult, SearchResult,
};
use tauri::State;

use super::{CommandError, Result};

/// Search notes using FTS5.
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

/// Search notes using hybrid FTS5 + vector search.
#[tauri::command]
pub async fn hybrid_search_notes(
    state: State<'_, AppState>,
    query: String,
    limit: Option<i32>,
    use_semantic: Option<bool>,
    settings: Option<EmbeddingSettings>,
) -> Result<Vec<HybridSearchResult>> {
    let vault_guard = state.vault.read().await;
    let vault = vault_guard.as_ref().ok_or(CommandError::NoVaultOpen)?;

    // Use provided settings or defaults
    let embedding_settings = settings.unwrap_or_default();
    let client = EmbeddingClient::new(embedding_settings);

    hybrid_search(
        &client,
        vault.repo(),
        &query,
        limit.unwrap_or(50),
        use_semantic.unwrap_or(true),
    )
    .await
    .map_err(|e| CommandError::Vault(e.to_string()))
}

/// Check embedding service status.
#[tauri::command]
pub async fn get_embedding_status(
    state: State<'_, AppState>,
    settings: EmbeddingSettings,
) -> Result<EmbeddingStatus> {
    let vault_guard = state.vault.read().await;
    let vault = vault_guard.as_ref().ok_or(CommandError::NoVaultOpen)?;

    let client = EmbeddingClient::new(settings.clone());

    // Check if service is reachable
    let connected = if settings.enabled {
        client.health_check().await.unwrap_or(false)
    } else {
        false
    };

    // Get embedding counts (only count complete embeddings with preview)
    let indexed_count = vault
        .repo()
        .count_complete_embeddings()
        .await
        .map_err(|e| CommandError::Vault(e.to_string()))?;

    // Get total note count
    let total_count = vault
        .repo()
        .count_notes()
        .await
        .map_err(|e| CommandError::Vault(e.to_string()))?;

    Ok(EmbeddingStatus {
        connected,
        error: if settings.enabled && !connected {
            Some("Could not connect to embedding service".to_string())
        } else {
            None
        },
        indexed_count,
        total_count,
    })
}

/// Test embedding service connection.
#[tauri::command]
pub async fn test_embedding_connection(
    settings: EmbeddingSettings,
) -> Result<bool> {
    let client = EmbeddingClient::new(settings);
    client
        .health_check()
        .await
        .map_err(|e| CommandError::Vault(e.to_string()))
}

/// Generate embedding for a single note by ID.
#[tauri::command]
pub async fn generate_note_embedding(
    state: State<'_, AppState>,
    note_id: i64,
    settings: EmbeddingSettings,
) -> Result<bool> {
    let vault_guard = state.vault.read().await;
    let vault = vault_guard.as_ref().ok_or(CommandError::NoVaultOpen)?;

    // Get note info
    let note = vault
        .repo()
        .get_note(note_id)
        .await
        .map_err(|e| CommandError::Vault(e.to_string()))?;

    // Read note content from filesystem
    let content = vault
        .fs()
        .read_file(std::path::Path::new(&note.path))
        .await
        .map_err(|e| CommandError::Vault(e.to_string()))?;

    // Generate embedding
    let client = EmbeddingClient::new(settings);
    let embedding = client
        .embed(&content)
        .await
        .map_err(|e| CommandError::Vault(e.to_string()))?;

    // Compute content hash
    let hash = core_fs::hash_content(&content);

    // Extract content preview for search results
    let preview = extract_content_preview(&content);

    // Store embedding with preview
    vault
        .repo()
        .store_embedding(note_id, &embedding, &hash, Some(&preview))
        .await
        .map_err(|e| CommandError::Vault(e.to_string()))?;

    Ok(true)
}

/// Get list of note IDs that need embeddings.
#[tauri::command]
pub async fn get_notes_needing_embeddings(
    state: State<'_, AppState>,
    limit: i32,
) -> Result<Vec<(i64, String)>> {
    let vault_guard = state.vault.read().await;
    let vault = vault_guard.as_ref().ok_or(CommandError::NoVaultOpen)?;

    vault
        .repo()
        .get_notes_without_embeddings(limit)
        .await
        .map_err(|e| CommandError::Vault(e.to_string()))
}
