//! Tag types.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// A tag with usage count.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct TagDto {
    pub tag: String,
    pub count: i64,
}
