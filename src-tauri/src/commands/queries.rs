//! Query builder commands.

use crate::state::AppState;
use shared_types::{PropertyKeyInfo, QueryEmbed, QueryEmbedResponse, QueryRequest, QueryResponse, TabResult};
use tauri::State;
use tracing::info;

use super::{CommandError, Result};

/// Get all property keys used in the vault (for query builder dropdown).
#[tauri::command]
pub async fn get_property_keys(state: State<'_, AppState>) -> Result<Vec<PropertyKeyInfo>> {
    let vault_guard = state.vault.read().await;
    let vault = vault_guard.as_ref().ok_or(CommandError::NoVaultOpen)?;

    vault
        .repo()
        .get_property_keys()
        .await
        .map_err(|e| CommandError::Vault(e.to_string()))
}

/// Get all distinct values for a property key (for query builder value autocomplete).
#[tauri::command]
pub async fn get_property_values(state: State<'_, AppState>, key: String) -> Result<Vec<String>> {
    let vault_guard = state.vault.read().await;
    let vault = vault_guard.as_ref().ok_or(CommandError::NoVaultOpen)?;

    vault
        .repo()
        .get_property_values(&key)
        .await
        .map_err(|e| CommandError::Vault(e.to_string()))
}

/// Run a query with property filters.
#[tauri::command]
pub async fn run_query(state: State<'_, AppState>, request: QueryRequest) -> Result<QueryResponse> {
    let vault_guard = state.vault.read().await;
    let vault = vault_guard.as_ref().ok_or(CommandError::NoVaultOpen)?;

    vault
        .repo()
        .run_query(&request)
        .await
        .map_err(|e| CommandError::Vault(e.to_string()))
}

/// Execute a query embed from YAML content.
/// This parses the YAML and executes the query, returning both the parsed config and results.
/// Supports both single-query mode and multi-tab mode.
#[tauri::command]
pub async fn execute_query_embed(
    state: State<'_, AppState>,
    yaml_content: String,
) -> Result<QueryEmbedResponse> {
    info!("execute_query_embed called with: {}", yaml_content);

    // Parse YAML into QueryEmbed
    let query: QueryEmbed = match serde_yaml::from_str::<QueryEmbed>(&yaml_content) {
        Ok(q) => {
            info!(
                "Parsed query: result_type={:?}, filters={}",
                q.result_type,
                q.filters.len()
            );
            q
        }
        Err(e) => {
            info!("YAML parse error: {}", e);
            return Ok(QueryEmbedResponse {
                query: QueryEmbed::default(),
                results: vec![],
                total_count: 0,
                tab_results: vec![],
                error: Some(format!("Invalid query YAML: {}", e)),
            });
        }
    };

    let vault_guard = state.vault.read().await;
    let vault = match vault_guard.as_ref() {
        Some(v) => v,
        None => {
            return Ok(QueryEmbedResponse {
                query: query.clone(),
                results: vec![],
                total_count: 0,
                tab_results: vec![],
                error: Some("No vault is currently open".to_string()),
            });
        }
    };

    // Check if we're in tab mode
    if !query.tabs.is_empty() {
        // Multi-tab mode: execute each tab's query
        let mut tab_results = Vec::new();
        let tabs = query.tabs.clone();

        for tab in &tabs {
            let request = QueryRequest {
                filters: tab.filters.clone(),
                match_mode: tab.match_mode.clone(),
                result_type: tab.result_type.clone(),
                include_completed: tab.include_completed,
                limit: Some(tab.limit),
            };

            match vault.repo().run_query(&request).await {
                Ok(response) => {
                    tab_results.push(TabResult {
                        name: tab.name.clone(),
                        results: response.results,
                        total_count: response.total_count,
                        view: tab.view.clone(),
                    });
                }
                Err(e) => {
                    // Return error for this tab but continue with others
                    return Ok(QueryEmbedResponse {
                        query,
                        results: vec![],
                        total_count: 0,
                        tab_results: vec![],
                        error: Some(format!(
                            "Query execution failed for tab '{}': {}",
                            tab.name, e
                        )),
                    });
                }
            }
        }

        Ok(QueryEmbedResponse {
            query,
            results: vec![],
            total_count: 0,
            tab_results,
            error: None,
        })
    } else {
        // Single-query mode (original behavior)
        info!("Single-query mode: result_type={:?}", query.result_type);
        let request = QueryRequest {
            filters: query.filters.clone(),
            match_mode: query.match_mode.clone(),
            result_type: query.result_type.clone(),
            include_completed: query.include_completed,
            limit: Some(query.limit),
        };

        info!("Running query...");
        match vault.repo().run_query(&request).await {
            Ok(response) => {
                info!("Query completed: {} results", response.results.len());
                Ok(QueryEmbedResponse {
                    query,
                    results: response.results,
                    total_count: response.total_count,
                    tab_results: vec![],
                    error: None,
                })
            }
            Err(e) => {
                info!("Query failed: {}", e);
                Ok(QueryEmbedResponse {
                    query,
                    results: vec![],
                    total_count: 0,
                    tab_results: vec![],
                    error: Some(format!("Query execution failed: {}", e)),
                })
            }
        }
    }
}
