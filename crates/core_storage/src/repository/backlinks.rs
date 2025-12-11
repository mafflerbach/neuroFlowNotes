//! Backlink tracking operations.

use crate::Result;
use shared_types::{BacklinkDto, NoteListItem};

use super::VaultRepository;

impl VaultRepository {
    /// Replace all backlinks originating from a note.
    pub async fn replace_backlinks(&self, from_note_id: i64, to_paths: &[String]) -> Result<()> {
        // Delete existing backlinks from this note
        sqlx::query("DELETE FROM backlinks WHERE from_note_id = ?")
            .bind(from_note_id)
            .execute(&self.pool)
            .await?;

        // Insert new backlinks (only if target note exists)
        for path in to_paths {
            sqlx::query(
                r#"
                INSERT INTO backlinks (from_note_id, to_note_id)
                SELECT ?, id FROM notes WHERE path = ? OR path = ? || '.md'
                "#,
            )
            .bind(from_note_id)
            .bind(path)
            .bind(path)
            .execute(&self.pool)
            .await?;
        }

        Ok(())
    }

    /// Get backlinks pointing to a note.
    pub async fn get_backlinks(&self, note_id: i64) -> Result<Vec<BacklinkDto>> {
        let rows = sqlx::query_as::<_, (i64, String, Option<String>)>(
            r#"
            SELECT n.id, n.path, n.title
            FROM backlinks b
            JOIN notes n ON b.from_note_id = n.id
            WHERE b.to_note_id = ?
            "#,
        )
        .bind(note_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|(from_note_id, from_note_path, from_note_title)| BacklinkDto {
                from_note_id,
                from_note_path,
                from_note_title,
            })
            .collect())
    }

    /// Get notes that link to a specific note name (for reference updating on rename).
    /// This searches for notes that have backlinks to the target, regardless of how they reference it.
    pub async fn get_notes_linking_to(&self, target_note_id: i64) -> Result<Vec<NoteListItem>> {
        let rows = sqlx::query_as::<_, (i64, String, Option<String>, i32)>(
            r#"
            SELECT DISTINCT n.id, n.path, n.title, n.pinned
            FROM backlinks b
            JOIN notes n ON b.from_note_id = n.id
            WHERE b.to_note_id = ?
            "#,
        )
        .bind(target_note_id)
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
}
