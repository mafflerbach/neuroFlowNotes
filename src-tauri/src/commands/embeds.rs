//! Embed commands - resolution and image handling.

use crate::state::AppState;
use shared_types::{EmbedContent, HeadingInfo, ResolveEmbedRequest};
use tauri::{AppHandle, State};
use tracing::{info, instrument};

use super::{CommandError, Result};

/// Resolve an embed (![[target]] or ![[target#section]]).
/// Returns the content to embed, handling images and notes differently.
#[tauri::command]
pub async fn resolve_embed(
    state: State<'_, AppState>,
    _app: AppHandle,
    request: ResolveEmbedRequest,
) -> Result<EmbedContent> {
    // Check depth limit
    if request.depth > 3 {
        return Ok(EmbedContent {
            note_id: None,
            path: request.target.clone(),
            content: None,
            is_image: false,
            asset_url: None,
            error: Some("Maximum embed depth (3) exceeded".to_string()),
        });
    }

    let vault_guard = state.vault.read().await;
    let vault = vault_guard.as_ref().ok_or(CommandError::NoVaultOpen)?;

    // Check if target is a media file (image, audio, video, pdf)
    let image_extensions = ["png", "jpg", "jpeg", "gif", "webp", "svg", "bmp", "ico"];
    let audio_extensions = ["mp3", "wav", "ogg", "m4a", "flac"];
    let video_extensions = ["mp4", "webm", "mov", "avi"];
    let pdf_extensions = ["pdf"];

    let target_lower = request.target.to_lowercase();
    let is_image = image_extensions
        .iter()
        .any(|ext| target_lower.ends_with(&format!(".{}", ext)));
    let is_audio = audio_extensions
        .iter()
        .any(|ext| target_lower.ends_with(&format!(".{}", ext)));
    let is_video = video_extensions
        .iter()
        .any(|ext| target_lower.ends_with(&format!(".{}", ext)));
    let is_pdf = pdf_extensions
        .iter()
        .any(|ext| target_lower.ends_with(&format!(".{}", ext)));
    let is_media = is_image || is_audio || is_video || is_pdf;

    if is_media {
        // Resolve media file path
        info!("Resolving media path for target: {}", request.target);
        let media_path = vault.resolve_asset_path(&request.target).await;
        info!("Resolved media path: {:?}", media_path);

        match media_path {
            Some(full_path) => {
                // Return the full filesystem path - frontend will convert it using convertFileSrc
                Ok(EmbedContent {
                    note_id: None,
                    path: request.target,
                    content: None,
                    is_image: is_media, // Keep using is_image field for backwards compat (means "is media")
                    asset_url: Some(full_path.to_string_lossy().to_string()),
                    error: None,
                })
            }
            None => Ok(EmbedContent {
                note_id: None,
                path: request.target.clone(),
                content: None,
                is_image: is_media,
                asset_url: None,
                error: Some(format!("Media not found: {}", request.target)),
            }),
        }
    } else {
        // Resolve note
        let note_result = vault.resolve_note(&request.target).await;

        match note_result {
            Some((note_id, path)) => {
                // Read note content
                let content = vault
                    .read_note(&path)
                    .await
                    .map_err(|e| CommandError::Vault(e.to_string()))?;

                // Extract section if requested
                let final_content = if let Some(ref section) = request.section {
                    // Slugify the section name to match how headings are stored
                    let section_slug = core_index::markdown::slugify(section);
                    core_index::markdown::extract_section_with_heading(&content, &section_slug)
                        .unwrap_or_else(|| format!("Section '{}' not found", section))
                } else {
                    content
                };

                Ok(EmbedContent {
                    note_id: Some(note_id),
                    path,
                    content: Some(final_content),
                    is_image: false,
                    asset_url: None,
                    error: None,
                })
            }
            None => Ok(EmbedContent {
                note_id: None,
                path: request.target.clone(),
                content: None,
                is_image: false,
                asset_url: None,
                error: Some(format!("Note not found: {}", request.target)),
            }),
        }
    }
}

/// Get all headings from a note (for section autocomplete).
#[tauri::command]
pub async fn get_note_headings(
    state: State<'_, AppState>,
    path: String,
) -> Result<Vec<HeadingInfo>> {
    let vault_guard = state.vault.read().await;
    let vault = vault_guard.as_ref().ok_or(CommandError::NoVaultOpen)?;

    // Read note content
    let content = vault
        .read_note(&path)
        .await
        .map_err(|e| CommandError::Vault(e.to_string()))?;

    // Parse and extract headings
    let analysis = core_index::markdown::parse(&content);

    Ok(analysis
        .headings
        .into_iter()
        .map(|h| HeadingInfo {
            level: h.level,
            text: h.text,
            slug: h.slug,
        })
        .collect())
}

/// Save a pasted image to the vault's assets folder.
/// Returns the filename that was saved (e.g., "Pasted image 20251208143000.png").
#[tauri::command]
#[instrument(skip(state, image_data))]
pub async fn save_pasted_image(
    state: State<'_, AppState>,
    image_data: String,
    extension: String,
) -> Result<String> {
    use std::io::Write;

    let vault_guard = state.vault.read().await;
    let vault = vault_guard.as_ref().ok_or(CommandError::NoVaultOpen)?;

    // Get vault root path
    let vault_root = vault.root_path();

    // Generate filename with timestamp (matching Obsidian's format)
    let timestamp = chrono::Local::now().format("%Y%m%d%H%M%S");
    let filename = format!("Pasted image {}.{}", timestamp, extension);
    let file_path = vault_root.join(&filename);

    // Decode base64 image data
    use base64::Engine;
    let image_bytes = base64::engine::general_purpose::STANDARD
        .decode(&image_data)
        .map_err(|e| CommandError::Vault(format!("Failed to decode image data: {}", e)))?;

    // Write the file
    let mut file = std::fs::File::create(&file_path)
        .map_err(|e| CommandError::Vault(format!("Failed to create image file: {}", e)))?;
    file.write_all(&image_bytes)
        .map_err(|e| CommandError::Vault(format!("Failed to write image data: {}", e)))?;

    info!("Saved pasted image: {}", file_path.display());

    // Return the filename (relative to vault root)
    Ok(filename)
}
