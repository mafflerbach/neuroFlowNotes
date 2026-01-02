//! Repository layer for vault database operations.
//!
//! This module provides the `VaultRepository` type and its implementations,
//! organized into submodules by domain:
//!
//! - `notes` - Note CRUD operations
//! - `tags` - Tag management
//! - `backlinks` - Backlink tracking
//! - `todos` - Todo/task operations
//! - `schedule` - Schedule block operations
//! - `properties` - Property management
//! - `queries` - Query builder and search
//! - `dates` - Notes by date operations
//! - `aliases` - Note alias management
//! - `embeddings` - Vector embedding storage and search

mod notes;
mod tags;
mod backlinks;
mod todos;
mod schedule;
mod properties;
mod folder_properties;
mod queries;
mod dates;
mod aliases;
mod habits;
mod embeddings;

pub use embeddings::VectorSearchResult;
pub use embeddings::extract_content_preview;

use sqlx::SqlitePool;

/// Repository for vault database operations.
#[derive(Clone)]
pub struct VaultRepository {
    pool: SqlitePool,
}

impl VaultRepository {
    /// Create a new repository with the given connection pool.
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// Get the underlying pool (for transactions, etc.).
    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }
}
