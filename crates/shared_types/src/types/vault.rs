//! Vault types.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// Information about an open vault.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct VaultInfo {
    pub path: String,
    pub name: String,
    pub note_count: i64,
}

/// Entry in the recent vaults list.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct RecentVault {
    pub path: String,
    pub name: String,
    pub last_opened: DateTime<Utc>,
}
