//! Tag management operations.

use crate::Result;
use shared_types::TagDto;

use super::VaultRepository;

impl VaultRepository {
    /// Replace all tags for a note.
    pub async fn replace_tags(&self, note_id: i64, tags: &[String]) -> Result<()> {
        // Delete existing tags
        sqlx::query("DELETE FROM tags WHERE note_id = ?")
            .bind(note_id)
            .execute(&self.pool)
            .await?;

        // Insert new tags
        for tag in tags {
            sqlx::query("INSERT INTO tags (note_id, tag) VALUES (?, ?)")
                .bind(note_id)
                .bind(tag)
                .execute(&self.pool)
                .await?;
        }

        Ok(())
    }

    /// Get all tags with their usage counts.
    pub async fn list_tags(&self) -> Result<Vec<TagDto>> {
        let rows = sqlx::query_as::<_, (String, i64)>(
            "SELECT tag, COUNT(*) as count FROM tags GROUP BY tag ORDER BY count DESC, tag",
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|(tag, count)| TagDto { tag, count }).collect())
    }

    /// Get tags for a specific note.
    pub async fn get_tags_for_note(&self, note_id: i64) -> Result<Vec<String>> {
        let tags = sqlx::query_scalar::<_, String>("SELECT tag FROM tags WHERE note_id = ?")
            .bind(note_id)
            .fetch_all(&self.pool)
            .await?;
        Ok(tags)
    }
}
