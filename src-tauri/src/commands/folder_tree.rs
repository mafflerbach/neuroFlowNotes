//! Folder tree commands and helpers.

use crate::state::AppState;
use shared_types::{FolderNode, NoteListItem};
use tauri::State;

use super::{CommandError, Result};

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
    "png", "jpg", "jpeg", "gif", "webp", "svg", "bmp", "ico", "mp3", "wav", "ogg", "m4a", "flac",
    "mp4", "webm", "mov", "avi", "pdf",
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
                if let Some(existing_dir) = node
                    .children
                    .iter_mut()
                    .find(|c| c.is_dir && c.path == relative)
                {
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
