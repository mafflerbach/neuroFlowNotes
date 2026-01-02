//! Shared types (DTOs) for NeuroFlow Notes.
//!
//! These types are used for communication between Rust backend and TypeScript frontend.
//! They are designed to be serializable and have TypeScript bindings generated via ts-rs.
//!
//! ## Module Organization
//!
//! Types are organized by domain:
//! - `note` - Note data types
//! - `todo` - Task/todo types
//! - `schedule` - Schedule block types
//! - `tag` - Tag types
//! - `backlink` - Backlink types
//! - `vault` - Vault info types
//! - `search` - Search types (FTS5 and hybrid search)
//! - `embedding` - Embedding settings for semantic search
//! - `folder` - Folder tree types
//! - `property` - Property types (note, folder, bulk operations)
//! - `event` - Tauri event payloads
//! - `embed` - Embed resolution types
//! - `query` - Query builder types
//! - `query_embed` - Query embed/block types
//! - `import` - Vault import types

mod types;

// Re-export all types at the crate root for backward compatibility
pub use types::*;
