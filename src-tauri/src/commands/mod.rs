//! Tauri commands - the IPC boundary between frontend and backend.
//!
//! This module is organized by domain:
//! - vault: Opening, closing, and vault info
//! - notes: Note CRUD operations and folder management
//! - todos: Task/todo operations
//! - tags: Tag listing
//! - backlinks: Backlink queries
//! - search: Full-text search
//! - folder_tree: Folder tree building
//! - properties: Property CRUD and management
//! - schedule: Schedule blocks and notes by date
//! - embeds: Embed resolution and image handling
//! - queries: Query builder operations
//! - import: Vault import operations

mod backlinks;
mod embeds;
mod folder_tree;
mod import;
mod notes;
mod plugins;
mod properties;
mod queries;
mod schedule;
mod search;
mod tags;
mod todos;
mod vault;

use thiserror::Error;

/// Error type for commands.
#[derive(Debug, Error)]
pub enum CommandError {
    #[error("No vault is currently open")]
    NoVaultOpen,

    #[error("Vault error: {0}")]
    Vault(String),

    #[allow(dead_code)]
    #[error("Note not found: {0}")]
    NoteNotFound(String),
}

impl serde::Serialize for CommandError {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

pub type Result<T> = std::result::Result<T, CommandError>;

// Re-export all commands for use in main.rs
pub use backlinks::*;
pub use embeds::*;
pub use folder_tree::*;
pub use import::*;
pub use notes::*;
pub use plugins::*;
pub use properties::*;
pub use queries::*;
pub use schedule::*;
pub use search::*;
pub use tags::*;
pub use todos::*;
pub use vault::*;
