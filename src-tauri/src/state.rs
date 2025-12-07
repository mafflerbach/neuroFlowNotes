//! Application state management.

use core_domain::Vault;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Global application state.
pub struct AppState {
    /// The currently open vault (if any).
    pub vault: Arc<RwLock<Option<Vault>>>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            vault: Arc::new(RwLock::new(None)),
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}
