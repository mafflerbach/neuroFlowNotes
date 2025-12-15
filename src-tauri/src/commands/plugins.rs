//! Plugin system commands - config storage and HTTP client.

use crate::state::AppState;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::path::PathBuf;
use tauri::State;
use tracing::{debug, info};

use super::{CommandError, Result};

// =============================================================================
// Plugin Config Commands
// =============================================================================

/// Get the plugins directory path for the current vault.
fn get_plugins_dir(vault_root: &std::path::Path) -> PathBuf {
    vault_root.join(".neuroflow").join("plugins")
}

/// Read a plugin's config file.
#[tauri::command]
pub async fn read_plugin_config(
    state: State<'_, AppState>,
    plugin_id: String,
) -> Result<Option<Value>> {
    let vault_guard = state.vault.read().await;
    let vault = vault_guard.as_ref().ok_or(CommandError::NoVaultOpen)?;

    let plugins_dir = get_plugins_dir(vault.fs().root());
    let config_path = plugins_dir.join(&plugin_id).join("config.json");

    if !config_path.exists() {
        debug!("No config file for plugin {}", plugin_id);
        return Ok(None);
    }

    let content = tokio::fs::read_to_string(&config_path)
        .await
        .map_err(|e| CommandError::Vault(format!("Failed to read plugin config: {}", e)))?;

    let config: Value = serde_json::from_str(&content)
        .map_err(|e| CommandError::Vault(format!("Failed to parse plugin config: {}", e)))?;

    debug!("Read config for plugin {}", plugin_id);
    Ok(Some(config))
}

/// Write a plugin's config file.
#[tauri::command]
pub async fn write_plugin_config(
    state: State<'_, AppState>,
    plugin_id: String,
    config: Value,
) -> Result<()> {
    let vault_guard = state.vault.read().await;
    let vault = vault_guard.as_ref().ok_or(CommandError::NoVaultOpen)?;

    let plugins_dir = get_plugins_dir(vault.fs().root());
    let plugin_dir = plugins_dir.join(&plugin_id);

    // Ensure plugin directory exists
    tokio::fs::create_dir_all(&plugin_dir)
        .await
        .map_err(|e| CommandError::Vault(format!("Failed to create plugin directory: {}", e)))?;

    let config_path = plugin_dir.join("config.json");
    let content = serde_json::to_string_pretty(&config)
        .map_err(|e| CommandError::Vault(format!("Failed to serialize plugin config: {}", e)))?;

    tokio::fs::write(&config_path, content)
        .await
        .map_err(|e| CommandError::Vault(format!("Failed to write plugin config: {}", e)))?;

    info!("Wrote config for plugin {}", plugin_id);
    Ok(())
}

/// List all plugin configs in the vault.
#[tauri::command]
pub async fn list_plugin_configs(state: State<'_, AppState>) -> Result<Vec<String>> {
    let vault_guard = state.vault.read().await;
    let vault = vault_guard.as_ref().ok_or(CommandError::NoVaultOpen)?;

    let plugins_dir = get_plugins_dir(vault.fs().root());

    if !plugins_dir.exists() {
        return Ok(vec![]);
    }

    let mut plugin_ids = Vec::new();
    let mut entries = tokio::fs::read_dir(&plugins_dir)
        .await
        .map_err(|e| CommandError::Vault(format!("Failed to read plugins directory: {}", e)))?;

    while let Ok(Some(entry)) = entries.next_entry().await {
        if entry.path().is_dir() {
            if let Some(name) = entry.file_name().to_str() {
                plugin_ids.push(name.to_string());
            }
        }
    }

    Ok(plugin_ids)
}

// =============================================================================
// HTTP Client Commands (for LLM APIs, etc.)
// =============================================================================

#[derive(Debug, Deserialize)]
pub struct HttpRequestOptions {
    pub url: String,
    pub method: String,
    pub headers: Option<HashMap<String, String>>,
    pub body: Option<Value>,
    #[serde(default = "default_timeout")]
    pub timeout: u64,
}

fn default_timeout() -> u64 {
    30000 // 30 seconds
}

#[derive(Debug, Serialize)]
pub struct HttpResponse {
    pub status: u16,
    pub headers: HashMap<String, String>,
    pub body: Value,
}

/// Make an HTTP request (for plugins to call external APIs).
#[tauri::command]
pub async fn plugin_http_request(options: HttpRequestOptions) -> Result<HttpResponse> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_millis(options.timeout))
        .build()
        .map_err(|e| CommandError::Vault(format!("Failed to create HTTP client: {}", e)))?;

    let mut request = match options.method.to_uppercase().as_str() {
        "GET" => client.get(&options.url),
        "POST" => client.post(&options.url),
        "PUT" => client.put(&options.url),
        "DELETE" => client.delete(&options.url),
        _ => return Err(CommandError::Vault(format!("Unsupported HTTP method: {}", options.method))),
    };

    // Add headers
    if let Some(headers) = options.headers {
        for (key, value) in headers {
            request = request.header(&key, &value);
        }
    }

    // Add body for POST/PUT
    if let Some(body) = options.body {
        request = request.json(&body);
    }

    debug!("Making HTTP request: {} {}", options.method, options.url);

    let response = request
        .send()
        .await
        .map_err(|e| CommandError::Vault(format!("HTTP request failed: {}", e)))?;

    let status = response.status().as_u16();
    let headers: HashMap<String, String> = response
        .headers()
        .iter()
        .filter_map(|(k, v)| {
            v.to_str().ok().map(|v| (k.to_string(), v.to_string()))
        })
        .collect();

    let body: Value = response
        .json()
        .await
        .unwrap_or(Value::Null);

    debug!("HTTP response: status={}", status);

    Ok(HttpResponse {
        status,
        headers,
        body,
    })
}
