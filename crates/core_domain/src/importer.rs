//! Obsidian vault importer.
//!
//! Imports an Obsidian vault into NeuroFlow Notes by:
//! - Copying markdown files (preserving folder structure)
//! - Copying asset files (images, attachments)
//! - Parsing YAML frontmatter and converting to properties
//! - Merging frontmatter tags with inline tags
//! - Preserving wikilink syntax

use crate::vault::Vault;
use core_fs::hash_content;
use core_index::frontmatter::{parse_frontmatter, PropertyValue};
use core_index::markdown::parse;
use shared_types::{ImportProgress, ImportResult};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::time::Instant;
use tokio::sync::mpsc;
use tracing::{debug, info};

/// Image/asset file extensions to copy.
const ASSET_EXTENSIONS: &[&str] = &[
    "png", "jpg", "jpeg", "gif", "webp", "svg", "bmp", // Images
    "pdf", "doc", "docx", "xls", "xlsx", // Documents
    "mp3", "wav", "ogg", "m4a", // Audio
    "mp4", "webm", "mov", // Video
];

/// Import an Obsidian vault into the current vault.
///
/// Returns an ImportResult with statistics.
pub async fn import_obsidian_vault(
    vault: &Vault,
    source_path: &Path,
    target_subfolder: Option<&str>,
    progress_tx: Option<mpsc::Sender<ImportProgress>>,
) -> Result<ImportResult, crate::vault::VaultError> {
    let start = Instant::now();
    let mut result = ImportResult {
        notes_imported: 0,
        files_copied: 0,
        properties_imported: 0,
        tags_imported: 0,
        duration_ms: 0,
        warnings: vec![],
    };

    info!("Starting Obsidian vault import from {}", source_path.display());

    // Validate source path
    if !source_path.exists() {
        return Err(crate::vault::VaultError::PathNotFound(source_path.to_path_buf()));
    }
    if !source_path.is_dir() {
        return Err(crate::vault::VaultError::NotADirectory(source_path.to_path_buf()));
    }

    // Collect all files to import
    let (markdown_files, asset_files) = collect_files(source_path).await?;
    let total_files = markdown_files.len() + asset_files.len();

    info!(
        "Found {} markdown files and {} asset files to import",
        markdown_files.len(),
        asset_files.len()
    );

    // Calculate target base path
    let target_base = target_subfolder.unwrap_or("");

    // Create target subfolder if specified
    if !target_base.is_empty() {
        vault.create_folder(target_base).await?;
    }

    // Copy asset files first
    for (i, (rel_path, full_path)) in asset_files.iter().enumerate() {
        let target_path = if target_base.is_empty() {
            rel_path.clone()
        } else {
            format!("{}/{}", target_base, rel_path)
        };

        match copy_file(full_path, &vault.fs().to_absolute(Path::new(&target_path))).await {
            Ok(_) => {
                result.files_copied += 1;
                debug!("Copied asset: {} -> {}", rel_path, target_path);
            }
            Err(e) => {
                result.warnings.push(format!("Failed to copy {}: {}", rel_path, e));
            }
        }

        // Send progress
        if let Some(tx) = &progress_tx {
            let _ = tx.send(ImportProgress {
                current_file: rel_path.clone(),
                files_processed: (i + 1) as i64,
                total_files: total_files as i64,
                properties_imported: result.properties_imported,
                tags_imported: result.tags_imported,
            }).await;
        }
    }

    // Import markdown files
    let asset_count = asset_files.len();
    for (i, (rel_path, full_path)) in markdown_files.iter().enumerate() {
        let target_path = if target_base.is_empty() {
            rel_path.clone()
        } else {
            format!("{}/{}", target_base, rel_path)
        };

        match import_markdown_file(vault, full_path, &target_path, &mut result).await {
            Ok(_) => {
                result.notes_imported += 1;
                result.files_copied += 1;
                debug!("Imported note: {} -> {}", rel_path, target_path);
            }
            Err(e) => {
                result.warnings.push(format!("Failed to import {}: {}", rel_path, e));
            }
        }

        // Send progress
        if let Some(tx) = &progress_tx {
            let _ = tx.send(ImportProgress {
                current_file: rel_path.clone(),
                files_processed: (asset_count + i + 1) as i64,
                total_files: total_files as i64,
                properties_imported: result.properties_imported,
                tags_imported: result.tags_imported,
            }).await;
        }
    }

    result.duration_ms = start.elapsed().as_millis() as u64;

    info!(
        "Import complete: {} notes, {} files, {} properties, {} tags in {}ms",
        result.notes_imported,
        result.files_copied,
        result.properties_imported,
        result.tags_imported,
        result.duration_ms
    );

    Ok(result)
}

/// Collect all files from the source directory.
///
/// Returns (markdown_files, asset_files) where each is a Vec of (relative_path, absolute_path).
async fn collect_files(source: &Path) -> Result<(Vec<(String, PathBuf)>, Vec<(String, PathBuf)>), crate::vault::VaultError> {
    let mut markdown_files = Vec::new();
    let mut asset_files = Vec::new();

    collect_files_recursive(source, source, &mut markdown_files, &mut asset_files).await?;

    Ok((markdown_files, asset_files))
}

/// Recursively collect files.
fn collect_files_recursive<'a>(
    root: &'a Path,
    dir: &'a Path,
    markdown_files: &'a mut Vec<(String, PathBuf)>,
    asset_files: &'a mut Vec<(String, PathBuf)>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), crate::vault::VaultError>> + Send + 'a>> {
    Box::pin(async move {
        let mut entries = tokio::fs::read_dir(dir)
            .await
            .map_err(core_fs::FsError::from)?;

        while let Some(entry) = entries.next_entry().await.map_err(core_fs::FsError::from)? {
            let path = entry.path();
            let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

            // Skip hidden files and directories
            if file_name.starts_with('.') {
                continue;
            }

            // Skip Obsidian config directory
            if file_name == ".obsidian" {
                continue;
            }

            if path.is_dir() {
                collect_files_recursive(root, &path, markdown_files, asset_files).await?;
            } else {
                // Calculate relative path
                let rel_path = path
                    .strip_prefix(root)
                    .map_err(|_| core_fs::FsError::InvalidPath(path.to_string_lossy().to_string()))?
                    .to_string_lossy()
                    .to_string();

                let extension = path
                    .extension()
                    .and_then(|e| e.to_str())
                    .map(|e| e.to_lowercase())
                    .unwrap_or_default();

                if extension == "md" {
                    markdown_files.push((rel_path, path));
                } else if ASSET_EXTENSIONS.contains(&extension.as_str()) {
                    asset_files.push((rel_path, path));
                }
            }
        }

        Ok(())
    })
}

/// Copy a file to the target location.
async fn copy_file(source: &Path, target: &Path) -> std::io::Result<()> {
    // Ensure parent directory exists
    if let Some(parent) = target.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }

    tokio::fs::copy(source, target).await?;
    Ok(())
}

/// Import a single markdown file.
async fn import_markdown_file(
    vault: &Vault,
    source: &Path,
    target_path: &str,
    result: &mut ImportResult,
) -> Result<(), crate::vault::VaultError> {
    // Read source content
    let content = tokio::fs::read_to_string(source)
        .await
        .map_err(core_fs::FsError::from)?;

    // Parse frontmatter
    let (frontmatter, body) = parse_frontmatter(&content);

    // Parse the body for inline tags
    let analysis = parse(body);

    // Merge frontmatter tags with inline tags
    let mut all_tags: HashSet<String> = HashSet::new();
    for tag in frontmatter.tags.iter() {
        all_tags.insert(tag.clone());
        result.tags_imported += 1;
    }
    for tag in analysis.tags.iter() {
        all_tags.insert(tag.clone());
    }

    // Write the file (we keep the frontmatter in the content)
    vault.fs().write_file(Path::new(target_path), &content).await?;

    // Index the note
    let hash = hash_content(&content);
    let note_id = vault.repo().index_note(target_path, &content, &hash, &analysis).await?;

    // Import frontmatter properties (excluding tags which we handle separately)
    for (key, value) in frontmatter.properties.iter() {
        // Skip special keys that are handled differently
        let key_lower = key.to_lowercase();
        if key_lower == "tags" || key_lower == "tag" {
            // Tags are already merged into inline tags
            continue;
        }
        if key_lower == "aliases" || key_lower == "alias" {
            // Could store aliases in a special property or skip
            // For now, store as a property
        }

        // Convert PropertyValue to string
        if let Some(string_value) = value.to_string_value() {
            vault.repo().set_property(
                note_id,
                key,
                Some(&string_value),
                infer_property_type(value).as_deref(),
            ).await?;
            result.properties_imported += 1;
        }
    }

    // If we have frontmatter tags that weren't in the inline content,
    // we need to add them to the note_tags table
    // (This is already handled by the indexer via the merged tags)

    Ok(())
}

/// Infer the property type from the value.
fn infer_property_type(value: &PropertyValue) -> Option<String> {
    match value {
        PropertyValue::String(s) => {
            // Check if it looks like a date
            if s.len() == 10 && s.chars().filter(|c| *c == '-').count() == 2 {
                Some("date".to_string())
            } else {
                Some("text".to_string())
            }
        }
        PropertyValue::Number(_) => Some("number".to_string()),
        PropertyValue::Bool(_) => Some("checkbox".to_string()),
        PropertyValue::List(_) => Some("list".to_string()),
        PropertyValue::Null => None,
    }
}
