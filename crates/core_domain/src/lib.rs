//! Business logic for NeuroFlow Notes.
//!
//! This crate contains:
//! - Vault management (open, close, index)
//! - File watching and reindexing
//! - Todo operations (toggle, sync to file)
//! - Schedule block operations
//! - Daily note creation
//! - Obsidian vault import

pub mod importer;
pub mod templates;
pub mod todos;
pub mod vault;
pub mod watcher;

pub use importer::import_obsidian_vault;
pub use vault::Vault;
pub use watcher::FileWatcher;
