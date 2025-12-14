//! Folder property management operations.

use crate::Result;
use shared_types::{FolderPropertyDto, PropertyWithInheritance};
use tracing::{debug, instrument};

use super::VaultRepository;

impl VaultRepository {
    // ========================================================================
    // Folder Property CRUD
    // ========================================================================

    /// Get all properties for a folder.
    pub async fn get_folder_properties(&self, folder_path: &str) -> Result<Vec<FolderPropertyDto>> {
        let rows = sqlx::query_as::<_, (i64, String, String, Option<String>, Option<String>)>(
            "SELECT id, folder_path, key, value, type FROM folder_properties WHERE folder_path = ? ORDER BY key",
        )
        .bind(folder_path)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|(id, folder_path, key, value, property_type)| FolderPropertyDto {
                id,
                folder_path,
                key,
                value,
                property_type,
            })
            .collect())
    }

    /// Set a folder property (upsert by folder_path + key).
    #[instrument(skip(self))]
    pub async fn set_folder_property(
        &self,
        folder_path: &str,
        key: &str,
        value: Option<&str>,
        property_type: Option<&str>,
    ) -> Result<i64> {
        let id = sqlx::query_scalar::<_, i64>(
            r#"
            INSERT INTO folder_properties (folder_path, key, value, type)
            VALUES (?, ?, ?, ?)
            ON CONFLICT(folder_path, key) DO UPDATE SET
                value = excluded.value,
                type = excluded.type
            RETURNING id
            "#,
        )
        .bind(folder_path)
        .bind(key)
        .bind(value)
        .bind(property_type)
        .fetch_one(&self.pool)
        .await?;

        debug!("Set folder property {} for folder {} (id={})", key, folder_path, id);
        Ok(id)
    }

    /// Delete a folder property by folder_path and key.
    pub async fn delete_folder_property(&self, folder_path: &str, key: &str) -> Result<()> {
        sqlx::query("DELETE FROM folder_properties WHERE folder_path = ? AND key = ?")
            .bind(folder_path)
            .bind(key)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    /// Delete all properties for a folder.
    pub async fn delete_all_folder_properties(&self, folder_path: &str) -> Result<()> {
        sqlx::query("DELETE FROM folder_properties WHERE folder_path = ?")
            .bind(folder_path)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    // ========================================================================
    // Inherited Properties
    // ========================================================================

    /// Get all ancestor folder paths for a given note path.
    /// For a note at "Projects/Work/notes/meeting.md", returns:
    /// ["Projects/Work/notes", "Projects/Work", "Projects", ""]
    fn get_ancestor_paths(note_path: &str) -> Vec<String> {
        let mut ancestors = Vec::new();

        // Get the directory containing the note
        if let Some(parent) = note_path.rsplit_once('/') {
            let mut current = parent.0.to_string();
            ancestors.push(current.clone());

            // Walk up the tree
            while let Some(idx) = current.rfind('/') {
                current = current[..idx].to_string();
                ancestors.push(current.clone());
            }
        }

        // Also include root folder (empty string represents vault root)
        ancestors.push(String::new());

        ancestors
    }

    /// Get inherited folder properties for a note path.
    /// Properties from parent folders override properties from ancestor folders.
    /// Returns properties sorted by folder depth (closest folder first).
    pub async fn get_inherited_folder_properties(
        &self,
        note_path: &str,
    ) -> Result<Vec<FolderPropertyDto>> {
        let ancestors = Self::get_ancestor_paths(note_path);

        if ancestors.is_empty() {
            return Ok(Vec::new());
        }

        // Build IN clause for all ancestor paths
        let placeholders: Vec<String> = ancestors.iter().map(|_| "?".to_string()).collect();
        let in_clause = placeholders.join(", ");

        let sql = format!(
            r#"
            SELECT id, folder_path, key, value, type
            FROM folder_properties
            WHERE folder_path IN ({})
            ORDER BY length(folder_path) DESC, key
            "#,
            in_clause
        );

        let mut query = sqlx::query_as::<_, (i64, String, String, Option<String>, Option<String>)>(&sql);
        for path in &ancestors {
            query = query.bind(path);
        }

        let rows = query.fetch_all(&self.pool).await?;

        Ok(rows
            .into_iter()
            .map(|(id, folder_path, key, value, property_type)| FolderPropertyDto {
                id,
                folder_path,
                key,
                value,
                property_type,
            })
            .collect())
    }

    /// Get properties for a note with inheritance info.
    /// Returns note's own properties plus inherited folder properties,
    /// with an `inherited` flag indicating the source.
    /// Note's own properties take precedence over folder properties.
    pub async fn get_properties_with_inheritance(
        &self,
        note_id: i64,
        note_path: &str,
    ) -> Result<Vec<PropertyWithInheritance>> {
        use std::collections::HashMap;

        let mut result: HashMap<String, PropertyWithInheritance> = HashMap::new();

        // First, get inherited folder properties (from most distant to closest ancestor)
        let folder_props = self.get_inherited_folder_properties(note_path).await?;

        // Process from furthest ancestor to closest (reverse order since query returns closest first)
        for prop in folder_props.into_iter().rev() {
            result.insert(prop.key.clone(), PropertyWithInheritance {
                id: prop.id,
                key: prop.key,
                value: prop.value,
                property_type: prop.property_type,
                sort_order: None,
                inherited: true,
                inherited_from: Some(prop.folder_path),
            });
        }

        // Then, get note's own properties (these override inherited ones)
        let note_props = self.get_properties_for_note(note_id).await?;
        for prop in note_props {
            result.insert(prop.key.clone(), PropertyWithInheritance {
                id: prop.id,
                key: prop.key,
                value: prop.value,
                property_type: prop.property_type,
                sort_order: prop.sort_order,
                inherited: false,
                inherited_from: None,
            });
        }

        // Convert to vec and sort by key
        let mut props: Vec<PropertyWithInheritance> = result.into_values().collect();
        props.sort_by(|a, b| {
            // Sort by: non-inherited first, then by sort_order if available, then by key
            match (a.inherited, b.inherited) {
                (false, true) => std::cmp::Ordering::Less,
                (true, false) => std::cmp::Ordering::Greater,
                _ => {
                    match (a.sort_order, b.sort_order) {
                        (Some(a_order), Some(b_order)) => a_order.cmp(&b_order),
                        (Some(_), None) => std::cmp::Ordering::Less,
                        (None, Some(_)) => std::cmp::Ordering::Greater,
                        (None, None) => a.key.cmp(&b.key),
                    }
                }
            }
        });

        Ok(props)
    }

    /// Get all folders that have properties defined.
    pub async fn get_folders_with_properties(&self) -> Result<Vec<String>> {
        let folders = sqlx::query_scalar::<_, String>(
            "SELECT DISTINCT folder_path FROM folder_properties ORDER BY folder_path"
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(folders)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_ancestor_paths() {
        let ancestors = VaultRepository::get_ancestor_paths("Projects/Work/notes/meeting.md");
        assert_eq!(ancestors, vec![
            "Projects/Work/notes".to_string(),
            "Projects/Work".to_string(),
            "Projects".to_string(),
            "".to_string(),
        ]);

        let ancestors = VaultRepository::get_ancestor_paths("note.md");
        assert_eq!(ancestors, vec!["".to_string()]);

        let ancestors = VaultRepository::get_ancestor_paths("folder/note.md");
        assert_eq!(ancestors, vec!["folder".to_string(), "".to_string()]);
    }
}
