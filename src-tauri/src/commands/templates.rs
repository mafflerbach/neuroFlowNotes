//! Template commands - daily note creation and template settings.

use crate::state::AppState;
use chrono::NaiveDate;
use core_domain::templates::{render_template, TemplateContext};
use serde::{Deserialize, Serialize};
use shared_types::{DailyNoteResult, TemplateSettings};
use std::path::Path;
use tauri::State;
use tracing::{debug, info};

use super::{CommandError, Result};

/// Vault config structure (stored in .neuroflow/config.json).
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
struct VaultConfig {
    #[serde(default)]
    template_settings: TemplateSettings,
}

/// Default template content when no template file is configured.
const DEFAULT_TEMPLATE: &str = r#"# {{date}}

## Tasks
- [ ]

## Notes

## Reflection

"#;

/// Get template settings from vault config.
#[tauri::command]
pub async fn get_template_settings(state: State<'_, AppState>) -> Result<TemplateSettings> {
    let vault_guard = state.vault.read().await;
    let vault = vault_guard.as_ref().ok_or(CommandError::NoVaultOpen)?;

    let config_path = vault.fs().config_path();

    if !config_path.exists() {
        debug!("No config file, returning default template settings");
        return Ok(TemplateSettings::default());
    }

    let content = tokio::fs::read_to_string(&config_path)
        .await
        .map_err(|e| CommandError::Vault(format!("Failed to read vault config: {}", e)))?;

    let config: VaultConfig = serde_json::from_str(&content)
        .map_err(|e| CommandError::Vault(format!("Failed to parse vault config: {}", e)))?;

    debug!("Read template settings: {:?}", config.template_settings);
    Ok(config.template_settings)
}

/// Save template settings to vault config.
#[tauri::command]
pub async fn save_template_settings(
    state: State<'_, AppState>,
    settings: TemplateSettings,
) -> Result<()> {
    let vault_guard = state.vault.read().await;
    let vault = vault_guard.as_ref().ok_or(CommandError::NoVaultOpen)?;

    let config_path = vault.fs().config_path();

    // Read existing config or create new one
    let mut config: VaultConfig = if config_path.exists() {
        let content = tokio::fs::read_to_string(&config_path)
            .await
            .map_err(|e| CommandError::Vault(format!("Failed to read vault config: {}", e)))?;

        serde_json::from_str(&content).unwrap_or_default()
    } else {
        VaultConfig::default()
    };

    // Update template settings
    config.template_settings = settings;

    // Ensure parent directory exists
    if let Some(parent) = config_path.parent() {
        tokio::fs::create_dir_all(parent)
            .await
            .map_err(|e| CommandError::Vault(format!("Failed to create config directory: {}", e)))?;
    }

    // Write config
    let content = serde_json::to_string_pretty(&config)
        .map_err(|e| CommandError::Vault(format!("Failed to serialize vault config: {}", e)))?;

    tokio::fs::write(&config_path, content)
        .await
        .map_err(|e| CommandError::Vault(format!("Failed to write vault config: {}", e)))?;

    info!("Saved template settings");
    Ok(())
}

/// List all template files in the templates/ folder.
#[tauri::command]
pub async fn list_templates(state: State<'_, AppState>) -> Result<Vec<String>> {
    let vault_guard = state.vault.read().await;
    let vault = vault_guard.as_ref().ok_or(CommandError::NoVaultOpen)?;

    let templates_dir = vault.fs().root().join("templates");

    if !templates_dir.exists() {
        debug!("Templates directory doesn't exist");
        return Ok(vec![]);
    }

    let mut templates = Vec::new();
    let mut entries = tokio::fs::read_dir(&templates_dir)
        .await
        .map_err(|e| CommandError::Vault(format!("Failed to read templates directory: {}", e)))?;

    while let Ok(Some(entry)) = entries.next_entry().await {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) == Some("md") {
            if let Ok(relative) = path.strip_prefix(vault.fs().root()) {
                templates.push(relative.to_string_lossy().to_string());
            }
        }
    }

    templates.sort();
    debug!("Found {} templates", templates.len());
    Ok(templates)
}

/// Create or open a daily note for the given date.
#[tauri::command]
pub async fn create_daily_note(
    state: State<'_, AppState>,
    date: String,
) -> Result<DailyNoteResult> {
    let vault_guard = state.vault.read().await;
    let vault = vault_guard.as_ref().ok_or(CommandError::NoVaultOpen)?;

    // Parse the date
    let parsed_date = NaiveDate::parse_from_str(&date, "%Y-%m-%d")
        .map_err(|e| CommandError::Vault(format!("Invalid date format: {}. Expected YYYY-MM-DD", e)))?;

    // Get template settings
    let config_path = vault.fs().config_path();
    let settings: TemplateSettings = if config_path.exists() {
        let content = tokio::fs::read_to_string(&config_path)
            .await
            .map_err(|e| CommandError::Vault(format!("Failed to read vault config: {}", e)))?;

        serde_json::from_str::<VaultConfig>(&content)
            .map(|c| c.template_settings)
            .unwrap_or_default()
    } else {
        TemplateSettings::default()
    };

    // Create template context
    let ctx = TemplateContext::for_date(parsed_date);

    // Render the file path
    let note_path = render_template(&settings.daily_note_pattern, &ctx);

    // Check if note already exists
    if vault.fs().exists(Path::new(&note_path)).await {
        // Note exists, return its info
        let note = vault
            .repo()
            .get_note_by_path(&note_path)
            .await
            .map_err(|e| CommandError::Vault(e.to_string()))?;

        debug!("Daily note already exists: {}", note_path);
        return Ok(DailyNoteResult {
            id: note.id,
            path: note.path,
            title: note.title,
            created: false,
        });
    }

    // Get template content
    let template_content = if let Some(ref template_path) = settings.daily_template_path {
        // Try to read the template file
        match vault.fs().read_file(Path::new(template_path)).await {
            Ok(content) => content,
            Err(e) => {
                debug!("Failed to read template file {}: {}, using default", template_path, e);
                DEFAULT_TEMPLATE.to_string()
            }
        }
    } else {
        DEFAULT_TEMPLATE.to_string()
    };

    // Render the template content
    let rendered_content = render_template(&template_content, &ctx);

    // Create the note
    let note_id = vault
        .write_note(&note_path, &rendered_content)
        .await
        .map_err(|e| CommandError::Vault(e.to_string()))?;

    info!("Created daily note: {} (id={})", note_path, note_id);

    // Extract title from path
    let title = Path::new(&note_path)
        .file_stem()
        .and_then(|s| s.to_str())
        .map(|s| s.to_string());

    Ok(DailyNoteResult {
        id: note_id,
        path: note_path,
        title,
        created: true,
    })
}

/// Create a new note from a template.
#[tauri::command]
pub async fn create_note_from_template(
    state: State<'_, AppState>,
    target_path: String,
    template_path: String,
) -> Result<i64> {
    let vault_guard = state.vault.read().await;
    let vault = vault_guard.as_ref().ok_or(CommandError::NoVaultOpen)?;

    // Check if target already exists
    if vault.fs().exists(Path::new(&target_path)).await {
        return Err(CommandError::Vault(format!(
            "File already exists: {}",
            target_path
        )));
    }

    // Read template file
    let template_content = vault
        .fs()
        .read_file(Path::new(&template_path))
        .await
        .map_err(|e| {
            CommandError::Vault(format!(
                "Failed to read template '{}': {}",
                template_path, e
            ))
        })?;

    // Create template context with current date
    let ctx = TemplateContext::default();

    // Render template with variable substitution
    let rendered_content = render_template(&template_content, &ctx);

    // Write the note
    let note_id = vault
        .write_note(&target_path, &rendered_content)
        .await
        .map_err(|e| CommandError::Vault(e.to_string()))?;

    info!(
        "Created note from template '{}': {} (id={})",
        template_path, target_path, note_id
    );

    Ok(note_id)
}

/// Render a preview of the daily note path for a given date (for settings UI).
#[tauri::command]
pub async fn preview_daily_note_path(
    pattern: String,
    date: String,
) -> Result<String> {
    let parsed_date = NaiveDate::parse_from_str(&date, "%Y-%m-%d")
        .map_err(|e| CommandError::Vault(format!("Invalid date format: {}. Expected YYYY-MM-DD", e)))?;

    let ctx = TemplateContext::for_date(parsed_date);
    Ok(render_template(&pattern, &ctx))
}
