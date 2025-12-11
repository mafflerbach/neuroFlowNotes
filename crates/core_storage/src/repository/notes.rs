//! Note CRUD operations.

use crate::{Result, StorageError};
use chrono::{DateTime, Utc};
use core_index::NoteAnalysis;
use shared_types::{NoteDto, NoteListItem};
use tracing::{debug, instrument};

use super::VaultRepository;

impl VaultRepository {
    /// Insert or update a note.
    #[instrument(skip(self, hash))]
    pub async fn upsert_note(
        &self,
        path: &str,
        title: Option<&str>,
        hash: &str,
    ) -> Result<i64> {
        let now = Utc::now().to_rfc3339();
        // Use local date for created_date to avoid timezone issues
        let local_date = chrono::Local::now().format("%Y-%m-%d").to_string();

        let result = sqlx::query_scalar::<_, i64>(
            r#"
            INSERT INTO notes (path, title, hash, created_at, updated_at, created_date)
            VALUES (?, ?, ?, ?, ?, ?)
            ON CONFLICT(path) DO UPDATE SET
                title = excluded.title,
                hash = excluded.hash,
                updated_at = excluded.updated_at
            RETURNING id
            "#,
        )
        .bind(path)
        .bind(title)
        .bind(hash)
        .bind(&now)
        .bind(&now)
        .bind(&local_date)
        .fetch_one(&self.pool)
        .await?;

        debug!("Upserted note {} with id {}", path, result);
        Ok(result)
    }

    /// Get a note by ID.
    pub async fn get_note(&self, id: i64) -> Result<NoteDto> {
        let row = sqlx::query_as::<_, (i64, String, Option<String>, Option<String>, Option<String>, i32)>(
            "SELECT id, path, title, created_at, updated_at, pinned FROM notes WHERE id = ?",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or(StorageError::NoteNotFound(id))?;

        Ok(NoteDto {
            id: row.0,
            path: row.1,
            title: row.2,
            created_at: row.3.and_then(|s| DateTime::parse_from_rfc3339(&s).ok().map(|d| d.with_timezone(&Utc))),
            updated_at: row.4.and_then(|s| DateTime::parse_from_rfc3339(&s).ok().map(|d| d.with_timezone(&Utc))),
            pinned: row.5 != 0,
        })
    }

    /// Get a note by path.
    pub async fn get_note_by_path(&self, path: &str) -> Result<NoteDto> {
        let row = sqlx::query_as::<_, (i64, String, Option<String>, Option<String>, Option<String>, i32)>(
            "SELECT id, path, title, created_at, updated_at, pinned FROM notes WHERE path = ?",
        )
        .bind(path)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| StorageError::NoteNotFoundByPath(path.to_string()))?;

        Ok(NoteDto {
            id: row.0,
            path: row.1,
            title: row.2,
            created_at: row.3.and_then(|s| DateTime::parse_from_rfc3339(&s).ok().map(|d| d.with_timezone(&Utc))),
            updated_at: row.4.and_then(|s| DateTime::parse_from_rfc3339(&s).ok().map(|d| d.with_timezone(&Utc))),
            pinned: row.5 != 0,
        })
    }

    /// Get note ID by path, if it exists.
    pub async fn get_note_id_by_path(&self, path: &str) -> Result<Option<i64>> {
        let result = sqlx::query_scalar::<_, i64>("SELECT id FROM notes WHERE path = ?")
            .bind(path)
            .fetch_optional(&self.pool)
            .await?;
        Ok(result)
    }

    /// Get the hash of a note by path.
    pub async fn get_note_hash(&self, path: &str) -> Result<Option<String>> {
        let result = sqlx::query_scalar::<_, String>("SELECT hash FROM notes WHERE path = ?")
            .bind(path)
            .fetch_optional(&self.pool)
            .await?;
        Ok(result)
    }

    /// List all notes.
    pub async fn list_notes(&self) -> Result<Vec<NoteListItem>> {
        let rows = sqlx::query_as::<_, (i64, String, Option<String>, i32)>(
            "SELECT id, path, title, pinned FROM notes ORDER BY path",
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|(id, path, title, pinned)| NoteListItem {
                id,
                path,
                title,
                pinned: pinned != 0,
            })
            .collect())
    }

    /// Delete a note by path.
    #[instrument(skip(self))]
    pub async fn delete_note(&self, path: &str) -> Result<Option<i64>> {
        let id = sqlx::query_scalar::<_, i64>("SELECT id FROM notes WHERE path = ?")
            .bind(path)
            .fetch_optional(&self.pool)
            .await?;

        if let Some(note_id) = id {
            sqlx::query("DELETE FROM notes WHERE id = ?")
                .bind(note_id)
                .execute(&self.pool)
                .await?;
            debug!("Deleted note {} (id={})", path, note_id);
        }

        Ok(id)
    }

    /// Get total note count.
    pub async fn count_notes(&self) -> Result<i64> {
        let count = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM notes")
            .fetch_one(&self.pool)
            .await?;
        Ok(count)
    }

    /// Rename/move a note (update its path).
    #[instrument(skip(self))]
    pub async fn rename_note(&self, old_path: &str, new_path: &str) -> Result<i64> {
        let note_id = sqlx::query_scalar::<_, i64>("SELECT id FROM notes WHERE path = ?")
            .bind(old_path)
            .fetch_optional(&self.pool)
            .await?
            .ok_or_else(|| StorageError::NoteNotFoundByPath(old_path.to_string()))?;

        sqlx::query("UPDATE notes SET path = ?, updated_at = datetime('now') WHERE id = ?")
            .bind(new_path)
            .bind(note_id)
            .execute(&self.pool)
            .await?;

        debug!("Renamed note {} -> {} (id={})", old_path, new_path, note_id);
        Ok(note_id)
    }

    /// Index a single note (upsert + update related tables).
    #[instrument(skip(self, content, analysis))]
    pub async fn index_note(
        &self,
        path: &str,
        content: &str,
        hash: &str,
        analysis: &NoteAnalysis,
    ) -> Result<i64> {
        let note_id = self.upsert_note(path, analysis.title.as_deref(), hash).await?;

        self.replace_tags(note_id, &analysis.tags).await?;
        self.replace_todos(note_id, &analysis.todos).await?;
        self.replace_backlinks(note_id, &analysis.links).await?;
        self.update_fts(note_id, content).await?;

        debug!("Indexed note {} (id={})", path, note_id);
        Ok(note_id)
    }
}
