//! Test helpers for core_storage tests.
//!
//! Provides utilities for setting up test databases, inserting test data,
//! and common assertions.

use core_storage::{init_database, VaultRepository};
use sqlx::SqlitePool;

/// Create an in-memory SQLite database for testing.
/// The database is initialized with the full schema.
pub async fn setup_test_db() -> SqlitePool {
    let pool = SqlitePool::connect(":memory:")
        .await
        .expect("Failed to create in-memory test database");

    init_database(&pool)
        .await
        .expect("Failed to initialize test database schema");

    pool
}

/// Create a VaultRepository with a test database.
pub async fn setup_test_repo() -> (SqlitePool, VaultRepository) {
    let pool = setup_test_db().await;
    let repo = VaultRepository::new(pool.clone());
    (pool, repo)
}

/// Insert a test note and return its ID.
///
/// # Arguments
/// * `pool` - Database connection pool
/// * `path` - Note path (e.g., "test.md")
/// * `title` - Optional note title
///
/// # Returns
/// The ID of the inserted note
pub async fn insert_test_note(
    pool: &SqlitePool,
    path: &str,
    title: Option<&str>,
) -> i64 {
    sqlx::query_scalar(
        "INSERT INTO notes (path, title, hash, created_at, updated_at) 
         VALUES (?, ?, 'test-hash', datetime('now'), datetime('now')) 
         RETURNING id"
    )
    .bind(path)
    .bind(title)
    .fetch_one(pool)
    .await
    .expect("Failed to insert test note")
}

/// Insert a test property for a note.
pub async fn insert_test_property(
    pool: &SqlitePool,
    note_id: i64,
    key: &str,
    value: &str,
    property_type: &str,
) {
    sqlx::query(
        "INSERT INTO properties (note_id, key, value, type, sort_order) 
         VALUES (?, ?, ?, ?, 0)"
    )
    .bind(note_id)
    .bind(key)
    .bind(value)
    .bind(property_type)
    .execute(pool)
    .await
    .expect("Failed to insert test property");
}

/// Insert a test tag for a note.
pub async fn insert_test_tag(pool: &SqlitePool, note_id: i64, tag: &str) {
    sqlx::query("INSERT INTO tags (note_id, tag) VALUES (?, ?)")
        .bind(note_id)
        .bind(tag)
        .execute(pool)
        .await
        .expect("Failed to insert test tag");
}

/// Get all tags for a note (for assertions).
pub async fn get_tags_for_note(pool: &SqlitePool, note_id: i64) -> Vec<String> {
    sqlx::query_scalar("SELECT tag FROM tags WHERE note_id = ? ORDER BY tag")
        .bind(note_id)
        .fetch_all(pool)
        .await
        .expect("Failed to fetch tags")
}

/// Get all properties for a note (for assertions).
pub async fn get_properties_for_note(
    pool: &SqlitePool,
    note_id: i64,
) -> Vec<(String, String)> {
    sqlx::query_as("SELECT key, value FROM properties WHERE note_id = ? ORDER BY key")
        .bind(note_id)
        .fetch_all(pool)
        .await
        .expect("Failed to fetch properties")
}

/// Count rows in a table (for assertions).
pub async fn count_rows(pool: &SqlitePool, table: &str) -> i64 {
    let query = format!("SELECT COUNT(*) FROM {}", table);
    sqlx::query_scalar(&query)
        .fetch_one(pool)
        .await
        .expect("Failed to count rows")
}
