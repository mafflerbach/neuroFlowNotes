//! Alias management operations.
//!
//! Aliases allow notes to be found/linked by names other than their filename.

use crate::Result;

use super::VaultRepository;

impl VaultRepository {
    /// Replace all aliases for a note.
    pub async fn replace_aliases(&self, note_id: i64, aliases: &[String]) -> Result<()> {
        // Delete existing aliases
        sqlx::query("DELETE FROM aliases WHERE note_id = ?")
            .bind(note_id)
            .execute(&self.pool)
            .await?;

        // Insert new aliases
        for alias in aliases {
            sqlx::query("INSERT INTO aliases (note_id, alias) VALUES (?, ?)")
                .bind(note_id)
                .bind(alias)
                .execute(&self.pool)
                .await?;
        }

        Ok(())
    }

    /// Get aliases for a specific note.
    pub async fn get_aliases_for_note(&self, note_id: i64) -> Result<Vec<String>> {
        let aliases = sqlx::query_scalar::<_, String>("SELECT alias FROM aliases WHERE note_id = ?")
            .bind(note_id)
            .fetch_all(&self.pool)
            .await?;
        Ok(aliases)
    }

    /// Find a note by alias.
    /// Returns the note ID if found.
    pub async fn find_note_by_alias(&self, alias: &str) -> Result<Option<i64>> {
        let note_id = sqlx::query_scalar::<_, i64>(
            "SELECT note_id FROM aliases WHERE alias = ? COLLATE NOCASE LIMIT 1",
        )
        .bind(alias)
        .fetch_optional(&self.pool)
        .await?;
        Ok(note_id)
    }
}
