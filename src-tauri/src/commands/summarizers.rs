//! Summarizer commands - Execute external scripts for content summarization.
//!
//! These commands run Node.js and Python scripts to summarize web links
//! and YouTube transcripts using a local LLM.
//!
//! Paths are relative to the vault root directory.

use crate::state::AppState;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::process::Stdio;
use tauri::{AppHandle, Manager, State};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tracing::{error, info};

use super::{CommandError, Result};

/// Result from a summarizer script execution.
#[derive(Debug, Serialize, Deserialize)]
pub struct SummarizerResult {
    pub success: bool,
    pub processed: usize,
    pub failed: usize,
    pub output_lines: Vec<String>,
}

/// Progress message from summarizer scripts.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SummarizerProgress {
    #[serde(rename = "type")]
    pub msg_type: String,
    #[serde(flatten)]
    pub data: serde_json::Value,
}

/// Get the scripts directory path.
fn get_scripts_dir(app: &AppHandle) -> Result<PathBuf> {
    // In development, scripts are in src-tauri/scripts/
    // In production, they should be bundled as resources
    let resource_dir = app
        .path()
        .resource_dir()
        .map_err(|e| CommandError::Vault(format!("Failed to get resource dir: {}", e)))?;

    let scripts_dir = resource_dir.join("scripts");

    // Fallback to development path if resource dir doesn't have scripts
    if !scripts_dir.exists() {
        // Try development path
        let dev_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("scripts");
        if dev_path.exists() {
            return Ok(dev_path);
        }
    }

    Ok(scripts_dir)
}

/// Run the link summarizer script.
/// Output path is relative to vault root.
///
/// # Arguments
/// * `urls` - List of URLs to summarize
/// * `output_dir` - Directory to save summary notes (relative to vault)
/// * `endpoint` - LM Studio API endpoint
/// * `model` - Model name (optional)
/// * `tags` - Default tags, comma-separated (optional)
#[tauri::command]
pub async fn run_link_summarizer(
    app: AppHandle,
    state: State<'_, AppState>,
    urls: Vec<String>,
    output_dir: String,
    endpoint: String,
    model: Option<String>,
    tags: Option<String>,
) -> Result<SummarizerResult> {
    let vault_guard = state.vault.read().await;
    let vault = vault_guard.as_ref().ok_or(CommandError::NoVaultOpen)?;
    let vault_root = vault.fs().root();

    // Resolve output path relative to vault
    let output_path = resolve_path(&output_dir, vault_root);

    info!(
        "Running link summarizer for {} URLs, output: {:?}",
        urls.len(),
        output_path
    );

    let scripts_dir = get_scripts_dir(&app)?;
    let script_path = scripts_dir.join("link-summarizer.js");

    if !script_path.exists() {
        return Err(CommandError::Vault(format!(
            "Link summarizer script not found at: {:?}",
            script_path
        )));
    }

    // Build command arguments
    let mut args: Vec<String> = vec![
        script_path.to_string_lossy().to_string(),
        "--output-dir".to_string(),
        output_path.to_string_lossy().to_string(),
        "--endpoint".to_string(),
        endpoint,
    ];

    if let Some(m) = model {
        if !m.is_empty() {
            args.push("--model".to_string());
            args.push(m);
        }
    }

    if let Some(t) = tags {
        if !t.is_empty() {
            args.push("--tags".to_string());
            args.push(t);
        }
    }

    // Add all URLs
    for url in &urls {
        args.push(url.clone());
    }

    // Run node with the script
    let mut child = Command::new("node")
        .args(&args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| {
            CommandError::Vault(format!(
                "Failed to spawn node process: {}. Is Node.js installed?",
                e
            ))
        })?;

    // Collect output lines
    let stdout = child.stdout.take().unwrap();
    let stderr = child.stderr.take().unwrap();

    let mut stdout_reader = BufReader::new(stdout).lines();
    let mut stderr_reader = BufReader::new(stderr).lines();

    let mut output_lines = Vec::new();
    let mut processed = 0;
    let mut failed = 0;

    // Read stdout (JSON progress messages)
    while let Ok(Some(line)) = stdout_reader.next_line().await {
        output_lines.push(line.clone());

        // Parse JSON to track progress
        if let Ok(msg) = serde_json::from_str::<SummarizerProgress>(&line) {
            match msg.msg_type.as_str() {
                "success" => processed += 1,
                "error" => failed += 1,
                "complete" => {
                    if let Some(p) = msg.data.get("successfulSummaries") {
                        processed = p.as_u64().unwrap_or(0) as usize;
                    }
                    if let Some(f) = msg.data.get("failedSummaries") {
                        failed = f.as_u64().unwrap_or(0) as usize;
                    }
                }
                _ => {}
            }
        }
    }

    // Read stderr
    while let Ok(Some(line)) = stderr_reader.next_line().await {
        error!("link-summarizer stderr: {}", line);
        output_lines.push(format!("STDERR: {}", line));
    }

    // Wait for process to complete
    let status = child.wait().await.map_err(|e| {
        CommandError::Vault(format!("Failed to wait for link summarizer: {}", e))
    })?;

    Ok(SummarizerResult {
        success: status.success(),
        processed,
        failed,
        output_lines,
    })
}

/// Run the transcript summarizer script.
/// Paths are relative to vault root.
///
/// # Arguments
/// * `input_dir` - Directory containing transcript .md files (relative to vault)
/// * `output_dir` - Directory to save summarized notes (relative to vault)
/// * `endpoint` - LM Studio API endpoint
/// * `model` - Model name (optional)
/// * `asset_template` - Path template for thumbnails (optional)
#[tauri::command]
pub async fn run_transcript_summarizer(
    app: AppHandle,
    state: State<'_, AppState>,
    input_dir: String,
    output_dir: String,
    endpoint: String,
    model: Option<String>,
    asset_template: Option<String>,
) -> Result<SummarizerResult> {
    let vault_guard = state.vault.read().await;
    let vault = vault_guard.as_ref().ok_or(CommandError::NoVaultOpen)?;
    let vault_root = vault.fs().root();

    // Resolve paths relative to vault
    let input_path = resolve_path(&input_dir, vault_root);
    let output_path = resolve_path(&output_dir, vault_root);

    info!(
        "Running transcript summarizer: {:?} -> {:?}",
        input_path, output_path
    );

    let scripts_dir = get_scripts_dir(&app)?;
    let script_path = scripts_dir.join("transcript_summarizer.py");

    if !script_path.exists() {
        return Err(CommandError::Vault(format!(
            "Transcript summarizer script not found at: {:?}",
            script_path
        )));
    }

    // Build command arguments
    let mut args: Vec<String> = vec![
        script_path.to_string_lossy().to_string(),
        "--input-dir".to_string(),
        input_path.to_string_lossy().to_string(),
        "--output-dir".to_string(),
        output_path.to_string_lossy().to_string(),
        "--endpoint".to_string(),
        endpoint,
    ];

    if let Some(m) = model {
        if !m.is_empty() {
            args.push("--model".to_string());
            args.push(m);
        }
    }

    if let Some(t) = asset_template {
        if !t.is_empty() {
            // Resolve asset template path relative to vault root
            // This ensures thumbnails are saved in the vault, not src-tauri/
            let resolved_template = resolve_path(&t, vault_root);
            info!(
                "Asset template resolved: {} -> {}",
                t,
                resolved_template.display()
            );
            args.push("--asset-template".to_string());
            args.push(resolved_template.to_string_lossy().to_string());
        }
    }

    // Check for venv in scripts dir, otherwise use system Python
    let venv_python = scripts_dir.join(".venv").join("bin").join("python");
    let python_cmd = if venv_python.exists() {
        venv_python.to_string_lossy().to_string()
    } else if Command::new("python3")
        .arg("--version")
        .output()
        .await
        .is_ok()
    {
        "python3".to_string()
    } else {
        "python".to_string()
    };

    let mut child = Command::new(&python_cmd)
        .args(&args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| {
            CommandError::Vault(format!(
                "Failed to spawn python process: {}. Is Python installed?",
                e
            ))
        })?;

    // Collect output lines
    let stdout = child.stdout.take().unwrap();
    let stderr = child.stderr.take().unwrap();

    let mut stdout_reader = BufReader::new(stdout).lines();
    let mut stderr_reader = BufReader::new(stderr).lines();

    let mut output_lines = Vec::new();
    let mut processed = 0;
    let mut failed = 0;

    // Read stdout (JSON progress messages)
    while let Ok(Some(line)) = stdout_reader.next_line().await {
        output_lines.push(line.clone());

        // Parse JSON to track progress
        if let Ok(msg) = serde_json::from_str::<SummarizerProgress>(&line) {
            match msg.msg_type.as_str() {
                "success" => processed += 1,
                "error" => failed += 1,
                "complete" => {
                    if let Some(p) = msg.data.get("processed") {
                        processed = p.as_u64().unwrap_or(0) as usize;
                    }
                    if let Some(f) = msg.data.get("failed") {
                        failed = f.as_u64().unwrap_or(0) as usize;
                    }
                }
                _ => {}
            }
        }
    }

    // Read stderr
    while let Ok(Some(line)) = stderr_reader.next_line().await {
        error!("transcript-summarizer stderr: {}", line);
        output_lines.push(format!("STDERR: {}", line));
    }

    // Wait for process to complete
    let status = child.wait().await.map_err(|e| {
        CommandError::Vault(format!("Failed to wait for transcript summarizer: {}", e))
    })?;

    Ok(SummarizerResult {
        success: status.success(),
        processed,
        failed,
        output_lines,
    })
}

/// Resolve a path relative to the vault root.
/// - Absolute paths are returned as-is
/// - Paths starting with `~/` are expanded to home directory
/// - Relative paths are joined with the vault root
fn resolve_path(path: &str, vault_root: &std::path::Path) -> PathBuf {
    let path = path.trim();

    // Handle absolute paths
    if path.starts_with('/') {
        return PathBuf::from(path);
    }

    // Handle tilde expansion
    if path.starts_with("~/") {
        if let Some(home) = dirs::home_dir() {
            return home.join(&path[2..]);
        }
    } else if path == "~" {
        if let Some(home) = dirs::home_dir() {
            return home;
        }
    }

    // Relative path - join with vault root
    vault_root.join(path)
}

/// Count pending transcript files in a directory.
/// Path is relative to vault root.
#[tauri::command]
pub async fn count_pending_transcripts(
    state: State<'_, AppState>,
    input_dir: String,
) -> Result<usize> {
    let vault_guard = state.vault.read().await;
    let vault = vault_guard.as_ref().ok_or(CommandError::NoVaultOpen)?;
    let vault_root = vault.fs().root();

    let path = resolve_path(&input_dir, vault_root);

    if !path.exists() {
        return Ok(0);
    }

    let count = std::fs::read_dir(&path)
        .map_err(|e| CommandError::Vault(format!("Failed to read directory: {}", e)))?
        .filter_map(|entry| entry.ok())
        .filter(|entry| {
            entry
                .path()
                .extension()
                .map_or(false, |ext| ext == "md")
        })
        .count();

    Ok(count)
}
