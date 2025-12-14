//! Embed types.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// Request to resolve an embed (![[target]] or ![[target#section]]).
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ResolveEmbedRequest {
    /// The target note name or path (without .md extension).
    pub target: String,
    /// Optional section slug to extract (e.g., "my-section" from "## My Section").
    pub section: Option<String>,
    /// Current embedding depth (starts at 0, max 3).
    pub depth: u8,
}

/// Result of resolving an embed.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct EmbedContent {
    /// The note ID if found in database.
    pub note_id: Option<i64>,
    /// The resolved path to the note or image.
    pub path: String,
    /// The markdown content to embed (for notes).
    pub content: Option<String>,
    /// Whether this is an image embed.
    pub is_image: bool,
    /// Asset URL for images (using Tauri asset protocol).
    pub asset_url: Option<String>,
    /// Error message if resolution failed.
    pub error: Option<String>,
}

/// Information about a heading in a note (for section autocomplete).
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct HeadingInfo {
    /// Heading level (1-6).
    pub level: u8,
    /// The heading text as displayed.
    pub text: String,
    /// URL-safe slug for linking (e.g., "my-section").
    pub slug: String,
}
