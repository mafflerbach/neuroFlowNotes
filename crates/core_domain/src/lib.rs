//! Business logic for NeuroFlow Notes.
//!
//! This crate contains:
//! - Vault management (open, close, index)
//! - File watching and reindexing
//! - Todo operations (toggle, sync to file)
//! - Schedule block operations
//! - Daily note creation

pub mod vault;
pub mod watcher;
pub mod todos;
pub mod templates;

pub use vault::Vault;
pub use watcher::FileWatcher;
