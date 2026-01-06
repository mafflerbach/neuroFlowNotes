//! Embedding storage and vector similarity search.
//!
//! Stores embeddings as BLOB and performs cosine similarity in Rust.
//! This approach avoids sqlite-vec extension dependencies while
//! still providing semantic search for typical vault sizes.

use crate::{Result, VaultRepository};
use sqlx::Row;
use tracing::debug;

/// Result from vector similarity search.
#[derive(Debug, Clone)]
pub struct VectorSearchResult {
    pub note_id: i64,
    pub path: String,
    pub title: Option<String>,
    pub content_preview: Option<String>,
    pub score: f64,
}

/// Maximum length for content preview (characters).
const PREVIEW_MAX_CHARS: usize = 300;

impl VaultRepository {
    /// Store or update an embedding for a note.
    pub async fn store_embedding(
        &self,
        note_id: i64,
        embedding: &[f32],
        content_hash: &str,
        content_preview: Option<&str>,
    ) -> Result<()> {
        let embedding_bytes = embedding_to_bytes(embedding);

        sqlx::query(
            r#"
            INSERT INTO note_embeddings (note_id, embedding, content_hash, content_preview, created_at)
            VALUES (?, ?, ?, ?, datetime('now'))
            ON CONFLICT(note_id) DO UPDATE SET
                embedding = excluded.embedding,
                content_hash = excluded.content_hash,
                content_preview = excluded.content_preview,
                created_at = excluded.created_at
            "#,
        )
        .bind(note_id)
        .bind(&embedding_bytes)
        .bind(content_hash)
        .bind(content_preview)
        .execute(&self.pool)
        .await?;

        debug!("Stored embedding for note {}", note_id);
        Ok(())
    }

    /// Get embedding for a note.
    pub async fn get_embedding(&self, note_id: i64) -> Result<Option<Vec<f32>>> {
        let row = sqlx::query("SELECT embedding FROM note_embeddings WHERE note_id = ?")
            .bind(note_id)
            .fetch_optional(&self.pool)
            .await?;

        Ok(row.map(|r| {
            let bytes: Vec<u8> = r.get("embedding");
            bytes_to_embedding(&bytes)
        }))
    }

    /// Check if a note needs re-embedding (hash changed or no embedding exists).
    pub async fn needs_embedding(&self, note_id: i64, current_hash: &str) -> Result<bool> {
        let row = sqlx::query("SELECT content_hash FROM note_embeddings WHERE note_id = ?")
            .bind(note_id)
            .fetch_optional(&self.pool)
            .await?;

        match row {
            None => Ok(true), // No embedding exists
            Some(r) => {
                let stored_hash: String = r.get("content_hash");
                Ok(stored_hash != current_hash)
            }
        }
    }

    /// Delete embedding when note is deleted.
    pub async fn delete_embedding(&self, note_id: i64) -> Result<()> {
        sqlx::query("DELETE FROM note_embeddings WHERE note_id = ?")
            .bind(note_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    /// Get count of notes with embeddings.
    pub async fn count_embeddings(&self) -> Result<i64> {
        let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM note_embeddings")
            .fetch_one(&self.pool)
            .await?;
        Ok(count.0)
    }

    /// Get count of notes with complete embeddings (including preview).
    pub async fn count_complete_embeddings(&self) -> Result<i64> {
        let count: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM note_embeddings WHERE content_preview IS NOT NULL"
        )
            .fetch_one(&self.pool)
            .await?;
        Ok(count.0)
    }

    /// Get note IDs that don't have embeddings or are missing content preview.
    pub async fn get_notes_without_embeddings(&self, limit: i32) -> Result<Vec<(i64, String)>> {
        let rows: Vec<(i64, String)> = sqlx::query_as(
            r#"
            SELECT n.id, n.path
            FROM notes n
            LEFT JOIN note_embeddings e ON n.id = e.note_id
            WHERE e.note_id IS NULL OR e.content_preview IS NULL
            LIMIT ?
            "#,
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows)
    }

    /// Perform vector similarity search using cosine similarity.
    /// Returns results sorted by similarity (highest first).
    pub async fn vector_search(
        &self,
        query_embedding: &[f32],
        limit: i32,
    ) -> Result<Vec<VectorSearchResult>> {
        // Fetch all embeddings (for small vaults, this is acceptable)
        // For larger vaults, we could add pre-filtering or use HNSW index
        let rows = sqlx::query(
            r#"
            SELECT e.note_id, e.embedding, e.content_preview, n.path, n.title
            FROM note_embeddings e
            JOIN notes n ON e.note_id = n.id
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        // Compute cosine similarities
        let mut results: Vec<VectorSearchResult> = rows
            .iter()
            .filter_map(|row| {
                let note_id: i64 = row.get("note_id");
                let embedding_bytes: Vec<u8> = row.get("embedding");
                let content_preview: Option<String> = row.get("content_preview");
                let path: String = row.get("path");
                let title: Option<String> = row.get("title");

                let embedding = bytes_to_embedding(&embedding_bytes);
                let score = cosine_similarity(query_embedding, &embedding);

                // Filter out very low similarity scores
                if score > 0.0 {
                    Some(VectorSearchResult {
                        note_id,
                        path,
                        title,
                        content_preview,
                        score,
                    })
                } else {
                    None
                }
            })
            .collect();

        // Sort by similarity (descending)
        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));

        // Take top N results
        results.truncate(limit as usize);

        debug!("Vector search returned {} results", results.len());
        Ok(results)
    }
}

/// Extract a preview from note content, stripping frontmatter and limiting length.
pub fn extract_content_preview(content: &str) -> String {
    // Skip YAML frontmatter if present
    let content = if let Some(stripped) = content.strip_prefix("---") {
        if let Some(end_idx) = stripped.find("---") {
            let after_frontmatter = &stripped[end_idx + 3..];
            after_frontmatter.trim_start()
        } else {
            content
        }
    } else {
        content
    };

    // Count characters (not bytes) for proper UTF-8 handling
    let char_count = content.chars().count();

    if char_count <= PREVIEW_MAX_CHARS {
        content.to_string()
    } else {
        // Take first N characters
        let preview: String = content.chars().take(PREVIEW_MAX_CHARS).collect();

        // Try to break at last space - find the character index of last space
        if let Some(last_space_char_idx) = preview.chars().enumerate()
            .filter(|(_, c)| *c == ' ')
            .map(|(i, _)| i)
            .last()
        {
            let truncated: String = preview.chars().take(last_space_char_idx).collect();
            format!("{}...", truncated)
        } else {
            format!("{}...", preview)
        }
    }
}

/// Convert f32 embedding to bytes for storage.
fn embedding_to_bytes(embedding: &[f32]) -> Vec<u8> {
    embedding
        .iter()
        .flat_map(|f| f.to_le_bytes())
        .collect()
}

/// Convert bytes back to f32 embedding.
fn bytes_to_embedding(bytes: &[u8]) -> Vec<f32> {
    bytes
        .chunks_exact(4)
        .map(|chunk| {
            let arr: [u8; 4] = chunk.try_into().unwrap();
            f32::from_le_bytes(arr)
        })
        .collect()
}

/// Compute cosine similarity between two vectors.
fn cosine_similarity(a: &[f32], b: &[f32]) -> f64 {
    if a.len() != b.len() || a.is_empty() {
        return 0.0;
    }

    let dot: f64 = a.iter().zip(b.iter()).map(|(x, y)| (*x as f64) * (*y as f64)).sum();
    let mag_a: f64 = a.iter().map(|x| (*x as f64).powi(2)).sum::<f64>().sqrt();
    let mag_b: f64 = b.iter().map(|x| (*x as f64).powi(2)).sum::<f64>().sqrt();

    if mag_a == 0.0 || mag_b == 0.0 {
        return 0.0;
    }

    dot / (mag_a * mag_b)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_embedding_bytes_roundtrip() {
        let embedding = vec![0.1, 0.2, 0.3, -0.4, 0.5];
        let bytes = embedding_to_bytes(&embedding);
        let recovered = bytes_to_embedding(&bytes);

        assert_eq!(embedding.len(), recovered.len());
        for (a, b) in embedding.iter().zip(recovered.iter()) {
            assert!((a - b).abs() < 1e-6);
        }
    }

    #[test]
    fn test_cosine_similarity_identical() {
        let a = vec![1.0, 2.0, 3.0];
        let b = vec![1.0, 2.0, 3.0];
        let sim = cosine_similarity(&a, &b);
        assert!((sim - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_cosine_similarity_orthogonal() {
        let a = vec![1.0, 0.0];
        let b = vec![0.0, 1.0];
        let sim = cosine_similarity(&a, &b);
        assert!(sim.abs() < 1e-6);
    }

    #[test]
    fn test_cosine_similarity_opposite() {
        let a = vec![1.0, 0.0];
        let b = vec![-1.0, 0.0];
        let sim = cosine_similarity(&a, &b);
        assert!((sim + 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_extract_content_preview_utf8() {
        // Test with German text containing multi-byte characters
        let content = r#"„Und wie willst du das anstellen?", fragte Vik skeptisch. V zuckte mit den Schultern. „Ich denke, ich werde einfach mal an der Blackwall anklopfen und hallo sagen. Und hoffen, dass sie sich blicken lässt." „V, du kannst nicht einfach so an die Blackwall gehen. Das ist viel zu gefährlich." Mehr Text hier um über 300 Zeichen zu kommen und den Abschnitt zu testen."#;

        // Should not panic on UTF-8 boundaries
        let preview = extract_content_preview(content);
        assert!(preview.ends_with("..."));
        assert!(preview.chars().count() <= PREVIEW_MAX_CHARS + 3); // +3 for "..."
    }

    #[test]
    fn test_extract_content_preview_short() {
        let content = "Short content";
        let preview = extract_content_preview(content);
        assert_eq!(preview, "Short content");
    }

    #[test]
    fn test_extract_content_preview_strips_frontmatter() {
        let content = "---\ntitle: Test\n---\nActual content here";
        let preview = extract_content_preview(content);
        assert_eq!(preview, "Actual content here");
    }
}
