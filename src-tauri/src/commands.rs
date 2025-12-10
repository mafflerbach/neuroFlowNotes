//! Tauri commands - the IPC boundary between frontend and backend.

use crate::state::AppState;
use core_domain::Vault;
use shared_types::{
    BacklinkDto, CreateScheduleBlockRequest, EmbedContent, FolderNode, HeadingInfo, NoteContent,
    NoteDto, NoteForDate, NoteListItem, PropertyDto, ResolveEmbedRequest, ScheduleBlockDto,
    SearchResult, SetPropertyRequest, TagDto, TodoDto, UpdateScheduleBlockRequest, VaultInfo,
};
use tauri::{AppHandle, Emitter, State};
use thiserror::Error;
use tracing::{error, info, instrument};

/// Error type for commands.
#[derive(Debug, Error)]
pub enum CommandError {
    #[error("No vault is currently open")]
    NoVaultOpen,

    #[error("Vault error: {0}")]
    Vault(String),

    #[error("Note not found: {0}")]
    NoteNotFound(String),
}

impl serde::Serialize for CommandError {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

type Result<T> = std::result::Result<T, CommandError>;

// ============================================================================
// Vault Commands
// ============================================================================

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
                    let _ = app_clone.emit("notes:updated", shared_types::NotesUpdatedPayload { note_ids: ids });
                }
                core_domain::vault::VaultEvent::NotesDeleted(ids) => {
                    let _ = app_clone.emit("notes:deleted", shared_types::NotesDeletedPayload { note_ids: ids });
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

// ============================================================================
// Note Commands
// ============================================================================

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

// ============================================================================
// Todo Commands
// ============================================================================

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

// ============================================================================
// Tag Commands
// ============================================================================

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

// ============================================================================
// Backlink Commands
// ============================================================================

/// Get backlinks for a note.
#[tauri::command]
pub async fn get_backlinks(state: State<'_, AppState>, note_id: i64) -> Result<Vec<BacklinkDto>> {
    let vault_guard = state.vault.read().await;
    let vault = vault_guard.as_ref().ok_or(CommandError::NoVaultOpen)?;

    vault
        .repo()
        .get_backlinks(note_id)
        .await
        .map_err(|e| CommandError::Vault(e.to_string()))
}

// ============================================================================
// Search Commands
// ============================================================================

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

// ============================================================================
// Folder Tree Commands
// ============================================================================

/// Get the folder tree for the vault.
#[tauri::command]
pub async fn get_folder_tree(state: State<'_, AppState>) -> Result<FolderNode> {
    let vault_guard = state.vault.read().await;
    let vault = vault_guard.as_ref().ok_or(CommandError::NoVaultOpen)?;

    let notes = vault
        .list_notes()
        .await
        .map_err(|e| CommandError::Vault(e.to_string()))?;

    // Build tree from flat list of paths
    let mut root = build_folder_tree(&notes, vault.fs().root().to_string_lossy().to_string());

    // Also scan actual directories to include empty folders
    scan_directories(&mut root, vault.fs().root(), vault.fs().root())
        .await
        .map_err(|e| CommandError::Vault(e.to_string()))?;

    // Re-sort after adding directories
    sort_tree(&mut root);

    Ok(root)
}

/// Image/media file extensions to include in the tree
const MEDIA_EXTENSIONS: &[&str] = &[
    "png", "jpg", "jpeg", "gif", "webp", "svg", "bmp", "ico",
    "mp3", "wav", "ogg", "m4a", "flac",
    "mp4", "webm", "mov", "avi",
    "pdf",
];

/// Check if a file extension is a media type
fn is_media_file(path: &std::path::Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| MEDIA_EXTENSIONS.contains(&ext.to_lowercase().as_str()))
        .unwrap_or(false)
}

/// Recursively scan directories and add empty folders and media files to the tree.
#[async_recursion::async_recursion]
async fn scan_directories(
    node: &mut FolderNode,
    current_dir: &std::path::Path,
    vault_root: &std::path::Path,
) -> std::result::Result<(), String> {
    let mut entries = match tokio::fs::read_dir(current_dir).await {
        Ok(entries) => entries,
        Err(e) => return Err(e.to_string()),
    };

    while let Ok(Some(entry)) = entries.next_entry().await {
        let path = entry.path();
        let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

        // Skip hidden files/directories
        if file_name.starts_with('.') {
            continue;
        }

        // Get relative path
        let relative = path
            .strip_prefix(vault_root)
            .map_err(|e| e.to_string())?
            .to_string_lossy()
            .to_string();

        if path.is_dir() {
            // Check if this directory already exists in the tree
            let dir_exists = node.children.iter().any(|c| c.is_dir && c.path == relative);

            if !dir_exists {
                // Add the directory
                let mut new_dir = FolderNode {
                    name: file_name.to_string(),
                    path: relative.clone(),
                    is_dir: true,
                    children: Vec::new(),
                };
                // Recursively scan subdirectories
                scan_directories(&mut new_dir, &path, vault_root).await?;
                node.children.push(new_dir);
            } else {
                // Directory exists, find it and scan its subdirectories
                if let Some(existing_dir) = node.children.iter_mut().find(|c| c.is_dir && c.path == relative) {
                    scan_directories(existing_dir, &path, vault_root).await?;
                }
            }
        } else if is_media_file(&path) {
            // Add media files that aren't already in the tree
            let file_exists = node.children.iter().any(|c| !c.is_dir && c.path == relative);

            if !file_exists {
                node.children.push(FolderNode {
                    name: file_name.to_string(),
                    path: relative,
                    is_dir: false,
                    children: Vec::new(),
                });
            }
        }
    }

    Ok(())
}

/// Build a folder tree from a flat list of note paths.
fn build_folder_tree(notes: &[NoteListItem], vault_name: String) -> FolderNode {
    let mut root = FolderNode {
        name: vault_name.split('/').last().unwrap_or("Vault").to_string(),
        path: String::new(),
        is_dir: true,
        children: Vec::new(),
    };

    for note in notes {
        let parts: Vec<&str> = note.path.split('/').collect();
        insert_path(&mut root, &parts, &note.path);
    }

    // Sort children recursively
    sort_tree(&mut root);

    root
}

fn insert_path(node: &mut FolderNode, parts: &[&str], full_path: &str) {
    if parts.is_empty() {
        return;
    }

    let name = parts[0];
    let is_file = parts.len() == 1;

    // Find or create child
    let child_idx = node.children.iter().position(|c| c.name == name);

    if let Some(idx) = child_idx {
        if !is_file {
            insert_path(&mut node.children[idx], &parts[1..], full_path);
        }
    } else {
        let child_path = if node.path.is_empty() {
            name.to_string()
        } else {
            format!("{}/{}", node.path, name)
        };

        let mut child = FolderNode {
            name: name.to_string(),
            path: if is_file {
                full_path.to_string()
            } else {
                child_path
            },
            is_dir: !is_file,
            children: Vec::new(),
        };

        if !is_file {
            insert_path(&mut child, &parts[1..], full_path);
        }

        node.children.push(child);
    }
}

fn sort_tree(node: &mut FolderNode) {
    // Sort: directories first, then alphabetically
    node.children.sort_by(|a, b| {
        match (a.is_dir, b.is_dir) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
        }
    });

    for child in &mut node.children {
        sort_tree(child);
    }
}

// ============================================================================
// Property Commands
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

/// Set a property for a note.
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

/// Delete a property from a note.
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
// Schedule Block Commands
// ============================================================================

/// Create a schedule block.
#[tauri::command]
#[instrument(skip(state))]
pub async fn create_schedule_block(
    state: State<'_, AppState>,
    request: CreateScheduleBlockRequest,
) -> Result<i64> {
    let vault_guard = state.vault.read().await;
    let vault = vault_guard.as_ref().ok_or(CommandError::NoVaultOpen)?;

    vault
        .repo()
        .create_schedule_block(
            request.note_id,
            &request.date.to_string(),
            &request.start_time.to_string(),
            &request.end_time.to_string(),
            request.label.as_deref(),
            request.color.as_deref(),
            request.context.as_deref(),
            request.rrule.as_deref(),
        )
        .await
        .map_err(|e| CommandError::Vault(e.to_string()))
}

/// Get schedule blocks for a date range.
#[tauri::command]
pub async fn get_schedule_blocks(
    state: State<'_, AppState>,
    start_date: String,
    end_date: String,
) -> Result<Vec<ScheduleBlockDto>> {
    let vault_guard = state.vault.read().await;
    let vault = vault_guard.as_ref().ok_or(CommandError::NoVaultOpen)?;

    vault
        .repo()
        .get_schedule_blocks_for_range(&start_date, &end_date)
        .await
        .map_err(|e| CommandError::Vault(e.to_string()))
}

/// Get schedule blocks for a single date.
#[tauri::command]
pub async fn get_schedule_blocks_for_date(
    state: State<'_, AppState>,
    date: String,
) -> Result<Vec<ScheduleBlockDto>> {
    let vault_guard = state.vault.read().await;
    let vault = vault_guard.as_ref().ok_or(CommandError::NoVaultOpen)?;

    vault
        .repo()
        .get_schedule_blocks_for_date(&date)
        .await
        .map_err(|e| CommandError::Vault(e.to_string()))
}

/// Get schedule blocks linked to a specific note.
#[tauri::command]
pub async fn get_schedule_blocks_for_note(
    state: State<'_, AppState>,
    note_id: i64,
) -> Result<Vec<ScheduleBlockDto>> {
    let vault_guard = state.vault.read().await;
    let vault = vault_guard.as_ref().ok_or(CommandError::NoVaultOpen)?;

    vault
        .repo()
        .get_schedule_blocks_for_note(note_id)
        .await
        .map_err(|e| CommandError::Vault(e.to_string()))
}

/// Update a schedule block.
#[tauri::command]
#[instrument(skip(state))]
pub async fn update_schedule_block(
    state: State<'_, AppState>,
    request: UpdateScheduleBlockRequest,
) -> Result<()> {
    let vault_guard = state.vault.read().await;
    let vault = vault_guard.as_ref().ok_or(CommandError::NoVaultOpen)?;

    vault
        .repo()
        .update_schedule_block(
            request.id,
            request.note_id,
            request.date.as_ref().map(|d| d.to_string()).as_deref(),
            request.start_time.as_ref().map(|t| t.to_string()).as_deref(),
            request.end_time.as_ref().map(|t| t.to_string()).as_deref(),
            request.label.as_deref(),
            request.color.as_deref(),
            request.context.as_deref(),
            request.rrule.as_deref(),
        )
        .await
        .map_err(|e| CommandError::Vault(e.to_string()))
}

/// Delete a schedule block.
#[tauri::command]
#[instrument(skip(state))]
pub async fn delete_schedule_block(state: State<'_, AppState>, id: i64) -> Result<()> {
    let vault_guard = state.vault.read().await;
    let vault = vault_guard.as_ref().ok_or(CommandError::NoVaultOpen)?;

    vault
        .repo()
        .delete_schedule_block(id)
        .await
        .map_err(|e| CommandError::Vault(e.to_string()))
}

// ============================================================================
// Notes by Date Commands
// ============================================================================

/// Get notes for a specific date (ordered by: scheduled > journal > created).
#[tauri::command]
pub async fn get_notes_for_date(
    state: State<'_, AppState>,
    date: String,
) -> Result<Vec<NoteForDate>> {
    let vault_guard = state.vault.read().await;
    let vault = vault_guard.as_ref().ok_or(CommandError::NoVaultOpen)?;

    vault
        .repo()
        .get_notes_for_date(&date)
        .await
        .map_err(|e| CommandError::Vault(e.to_string()))
}

/// Get notes for a date range (for weekly/monthly views).
#[tauri::command]
pub async fn get_notes_for_date_range(
    state: State<'_, AppState>,
    start_date: String,
    end_date: String,
) -> Result<Vec<(String, Vec<NoteForDate>)>> {
    let vault_guard = state.vault.read().await;
    let vault = vault_guard.as_ref().ok_or(CommandError::NoVaultOpen)?;

    vault
        .repo()
        .get_notes_for_date_range(&start_date, &end_date)
        .await
        .map_err(|e| CommandError::Vault(e.to_string()))
}

// ============================================================================
// Embed Commands
// ============================================================================

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
    let is_image = image_extensions.iter().any(|ext| target_lower.ends_with(&format!(".{}", ext)));
    let is_audio = audio_extensions.iter().any(|ext| target_lower.ends_with(&format!(".{}", ext)));
    let is_video = video_extensions.iter().any(|ext| target_lower.ends_with(&format!(".{}", ext)));
    let is_pdf = pdf_extensions.iter().any(|ext| target_lower.ends_with(&format!(".{}", ext)));
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
                let content = vault.read_note(&path).await.map_err(|e| CommandError::Vault(e.to_string()))?;

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
    let content = vault.read_note(&path).await.map_err(|e| CommandError::Vault(e.to_string()))?;

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
