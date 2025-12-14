//! Folder tree types.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// A node in the folder tree.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct FolderNode {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub children: Vec<FolderNode>,
}
