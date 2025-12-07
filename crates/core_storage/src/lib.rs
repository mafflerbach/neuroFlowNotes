//! SQLite storage layer for NeuroFlow Notes.
//!
//! This crate handles all database operations:
//! - Schema creation and migrations
//! - Note CRUD operations
//! - Todo, tag, and backlink persistence
//! - Full-text search

pub mod schema;
pub mod repository;

pub use repository::VaultRepository;
pub use schema::init_database;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum StorageError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Note not found: {0}")]
    NoteNotFound(i64),

    #[error("Note not found by path: {0}")]
    NoteNotFoundByPath(String),
}

pub type Result<T> = std::result::Result<T, StorageError>;
