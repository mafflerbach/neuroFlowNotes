//! Property management operations.

use crate::Result;
use shared_types::{NoteWithPropertyValue, PropertyDto, PropertyKeyInfo};
use std::collections::HashMap;
use tracing::{debug, instrument};

use super::VaultRepository;

impl VaultRepository {
    /// Get all properties for a note.
    pub async fn get_properties_for_note(&self, note_id: i64) -> Result<Vec<PropertyDto>> {
        let rows = sqlx::query_as::<_, (i64, i64, String, Option<String>, Option<String>, Option<i32>)>(
            "SELECT id, note_id, key, value, type, sort_order FROM properties WHERE note_id = ? ORDER BY sort_order, key",
        )
        .bind(note_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|(id, note_id, key, value, property_type, sort_order)| PropertyDto {
                id,
                note_id,
                key,
                value,
                property_type,
                sort_order,
            })
            .collect())
    }

    /// Get properties for multiple notes at once (batch query to avoid N+1).
    /// Returns a HashMap from note_id to Vec<PropertyDto>.
    pub async fn get_properties_for_notes(
        &self,
        note_ids: &[i64],
    ) -> Result<HashMap<i64, Vec<PropertyDto>>> {
        if note_ids.is_empty() {
            return Ok(HashMap::new());
        }

        let placeholders: Vec<String> = note_ids.iter().map(|_| "?".to_string()).collect();
        let in_clause = placeholders.join(", ");

        let sql = format!(
            "SELECT id, note_id, key, value, type, sort_order FROM properties WHERE note_id IN ({}) ORDER BY note_id, sort_order, key",
            in_clause
        );

        let mut query = sqlx::query_as::<_, (i64, i64, String, Option<String>, Option<String>, Option<i32>)>(&sql);
        for id in note_ids {
            query = query.bind(id);
        }

        let rows = query.fetch_all(&self.pool).await?;

        let mut result: HashMap<i64, Vec<PropertyDto>> = HashMap::new();
        for (id, note_id, key, value, property_type, sort_order) in rows {
            result.entry(note_id).or_default().push(PropertyDto {
                id,
                note_id,
                key,
                value,
                property_type,
                sort_order,
            });
        }

        // Ensure all requested note_ids have an entry (even if empty)
        for &note_id in note_ids {
            result.entry(note_id).or_default();
        }

        Ok(result)
    }

    /// Set a property (upsert by note_id + key).
    pub async fn set_property(
        &self,
        note_id: i64,
        key: &str,
        value: Option<&str>,
        property_type: Option<&str>,
    ) -> Result<i64> {
        let id = sqlx::query_scalar::<_, i64>(
            r#"
            INSERT INTO properties (note_id, key, value, type)
            VALUES (?, ?, ?, ?)
            ON CONFLICT(note_id, key) DO UPDATE SET
                value = excluded.value,
                type = excluded.type
            RETURNING id
            "#,
        )
        .bind(note_id)
        .bind(key)
        .bind(value)
        .bind(property_type)
        .fetch_one(&self.pool)
        .await?;

        debug!("Set property {} for note {} (id={})", key, note_id, id);
        Ok(id)
    }

    /// Delete a property by note_id and key.
    pub async fn delete_property(&self, note_id: i64, key: &str) -> Result<()> {
        sqlx::query("DELETE FROM properties WHERE note_id = ? AND key = ?")
            .bind(note_id)
            .bind(key)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    /// Delete all properties for a note.
    pub async fn delete_all_properties(&self, note_id: i64) -> Result<()> {
        sqlx::query("DELETE FROM properties WHERE note_id = ?")
            .bind(note_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    /// Get a specific property by note_id and key.
    pub async fn get_property(&self, note_id: i64, key: &str) -> Result<Option<PropertyDto>> {
        let row = sqlx::query_as::<_, (i64, i64, String, Option<String>, Option<String>, Option<i32>)>(
            "SELECT id, note_id, key, value, type, sort_order FROM properties WHERE note_id = ? AND key = ?",
        )
        .bind(note_id)
        .bind(key)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|(id, note_id, key, value, property_type, sort_order)| PropertyDto {
            id,
            note_id,
            key,
            value,
            property_type,
            sort_order,
        }))
    }

    // ========================================================================
    // Property Management (Bulk Operations)
    // ========================================================================

    /// Rename a property key across all notes.
    #[instrument(skip(self))]
    pub async fn rename_property_key(&self, old_key: &str, new_key: &str) -> Result<(i64, i64)> {
        // First check if new_key already exists for notes that have old_key
        // If both keys exist for a note, we need to handle the conflict

        // Get count of notes that will be affected
        let notes_affected = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(DISTINCT note_id) FROM properties WHERE key = ?"
        )
        .bind(old_key)
        .fetch_one(&self.pool)
        .await?;

        // Update the key name (ON CONFLICT will handle duplicates by keeping new_key value)
        let result = sqlx::query(
            r#"
            UPDATE properties
            SET key = ?
            WHERE key = ?
            AND note_id NOT IN (SELECT note_id FROM properties WHERE key = ?)
            "#
        )
        .bind(new_key)
        .bind(old_key)
        .bind(new_key)
        .execute(&self.pool)
        .await?;

        let affected_count = result.rows_affected() as i64;

        // Delete any remaining old_key entries (those where new_key already exists)
        sqlx::query("DELETE FROM properties WHERE key = ?")
            .bind(old_key)
            .execute(&self.pool)
            .await?;

        debug!("Renamed property key '{}' -> '{}': {} properties, {} notes", old_key, new_key, affected_count, notes_affected);
        Ok((affected_count, notes_affected))
    }

    /// Rename a property value across all notes with that key.
    #[instrument(skip(self))]
    pub async fn rename_property_value(&self, key: &str, old_value: &str, new_value: &str) -> Result<(i64, i64)> {
        let notes_affected = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(DISTINCT note_id) FROM properties WHERE key = ? AND value = ?"
        )
        .bind(key)
        .bind(old_value)
        .fetch_one(&self.pool)
        .await?;

        let result = sqlx::query(
            "UPDATE properties SET value = ? WHERE key = ? AND value = ?"
        )
        .bind(new_value)
        .bind(key)
        .bind(old_value)
        .execute(&self.pool)
        .await?;

        let affected_count = result.rows_affected() as i64;

        debug!("Renamed property value '{}' -> '{}' for key '{}': {} properties, {} notes",
               old_value, new_value, key, affected_count, notes_affected);
        Ok((affected_count, notes_affected))
    }

    /// Merge two property keys (rename source to target).
    /// If a note has both keys, the target key's value is kept.
    #[instrument(skip(self))]
    pub async fn merge_property_keys(&self, source_key: &str, target_key: &str) -> Result<(i64, i64)> {
        // Count notes with source key (before merge)
        let notes_affected = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(DISTINCT note_id) FROM properties WHERE key = ?"
        )
        .bind(source_key)
        .fetch_one(&self.pool)
        .await?;

        // Rename source_key to target_key for notes that don't already have target_key
        let result = sqlx::query(
            r#"
            UPDATE properties
            SET key = ?
            WHERE key = ?
            AND note_id NOT IN (SELECT note_id FROM properties WHERE key = ?)
            "#
        )
        .bind(target_key)
        .bind(source_key)
        .bind(target_key)
        .execute(&self.pool)
        .await?;

        let affected_count = result.rows_affected() as i64;

        // Delete remaining source_key entries (notes that had both keys)
        sqlx::query("DELETE FROM properties WHERE key = ?")
            .bind(source_key)
            .execute(&self.pool)
            .await?;

        debug!("Merged property key '{}' into '{}': {} properties moved, {} notes affected",
               source_key, target_key, affected_count, notes_affected);
        Ok((affected_count, notes_affected))
    }

    /// Delete a property key from all notes.
    #[instrument(skip(self))]
    pub async fn delete_property_key(&self, key: &str) -> Result<(i64, i64)> {
        let notes_affected = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(DISTINCT note_id) FROM properties WHERE key = ?"
        )
        .bind(key)
        .fetch_one(&self.pool)
        .await?;

        let result = sqlx::query("DELETE FROM properties WHERE key = ?")
            .bind(key)
            .execute(&self.pool)
            .await?;

        let affected_count = result.rows_affected() as i64;

        debug!("Deleted property key '{}': {} properties, {} notes", key, affected_count, notes_affected);
        Ok((affected_count, notes_affected))
    }

    /// Get all distinct values for a property key with usage counts.
    pub async fn get_property_values_with_counts(&self, key: &str) -> Result<Vec<(String, i64)>> {
        let values = sqlx::query_as::<_, (String, i64)>(
            r#"
            SELECT value, COUNT(*) as count
            FROM properties
            WHERE key = ? AND value IS NOT NULL AND value != ''
            GROUP BY value
            ORDER BY count DESC, value
            "#,
        )
        .bind(key)
        .fetch_all(&self.pool)
        .await?;

        Ok(values)
    }

    /// Get all notes that have a specific property key, along with their value.
    pub async fn get_notes_with_property(&self, key: &str) -> Result<Vec<NoteWithPropertyValue>> {
        let rows = sqlx::query_as::<_, (i64, String, Option<String>, Option<String>)>(
            r#"
            SELECT n.id, n.path, n.title, p.value
            FROM notes n
            INNER JOIN properties p ON n.id = p.note_id
            WHERE p.key = ?
            ORDER BY p.value, n.title, n.path
            "#,
        )
        .bind(key)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|(note_id, path, title, value)| NoteWithPropertyValue {
                note_id,
                path,
                title,
                value,
            })
            .collect())
    }

    /// Get all notes that have a specific property key and value.
    pub async fn get_notes_with_property_value(&self, key: &str, value: &str) -> Result<Vec<NoteWithPropertyValue>> {
        let rows = sqlx::query_as::<_, (i64, String, Option<String>, Option<String>)>(
            r#"
            SELECT n.id, n.path, n.title, p.value
            FROM notes n
            INNER JOIN properties p ON n.id = p.note_id
            WHERE p.key = ? AND p.value = ?
            ORDER BY n.title, n.path
            "#,
        )
        .bind(key)
        .bind(value)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|(note_id, path, title, value)| NoteWithPropertyValue {
                note_id,
                path,
                title,
                value,
            })
            .collect())
    }

    /// Get all distinct property keys used in the vault.
    pub async fn get_property_keys(&self) -> Result<Vec<PropertyKeyInfo>> {
        // Get all distinct keys with usage count and most common property type
        let rows = sqlx::query_as::<_, (String, i64, Option<String>)>(
            r#"
            SELECT
                key,
                COUNT(DISTINCT note_id) as usage_count,
                (
                    SELECT type
                    FROM properties p2
                    WHERE p2.key = properties.key AND p2.type IS NOT NULL
                    GROUP BY type
                    ORDER BY COUNT(*) DESC
                    LIMIT 1
                ) as most_common_type
            FROM properties
            GROUP BY key
            ORDER BY usage_count DESC, key
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        let mut results = Vec::new();
        for (key, usage_count, property_type) in rows {
            // Get sample values for each key (up to 10 unique values)
            let sample_values = sqlx::query_scalar::<_, String>(
                r#"
                SELECT DISTINCT value
                FROM properties
                WHERE key = ? AND value IS NOT NULL AND value != ''
                LIMIT 10
                "#,
            )
            .bind(&key)
            .fetch_all(&self.pool)
            .await?;

            results.push(PropertyKeyInfo {
                key,
                usage_count,
                sample_values,
                property_type,
            });
        }

        Ok(results)
    }

    /// Get all distinct values for a property key.
    pub async fn get_property_values(&self, key: &str) -> Result<Vec<String>> {
        let values = sqlx::query_scalar::<_, String>(
            r#"
            SELECT DISTINCT value
            FROM properties
            WHERE key = ? AND value IS NOT NULL AND value != ''
            ORDER BY value
            "#,
        )
        .bind(key)
        .fetch_all(&self.pool)
        .await?;

        Ok(values)
    }
}
