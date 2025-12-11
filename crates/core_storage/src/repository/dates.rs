//! Notes by date operations.

use crate::Result;
use shared_types::{NoteForDate, NoteListItem};
use std::collections::{HashMap, HashSet};

use super::VaultRepository;

impl VaultRepository {
    /// Get notes for a specific date, ordered by: scheduled > journal > created.
    pub async fn get_notes_for_date(&self, date: &str) -> Result<Vec<NoteForDate>> {
        let mut results = Vec::new();

        // 1. Notes with schedule blocks on this date (including recurring block occurrences)
        // Use get_schedule_blocks_for_date which handles RRULE expansion
        let blocks = self.get_schedule_blocks_for_date(date).await?;

        for block in blocks {
            // Only include blocks that have a linked note
            if let Some(note_id) = block.note_id {
                if let Ok(note) = self.get_note(note_id).await {
                    results.push(NoteForDate {
                        note: NoteListItem {
                            id: note.id,
                            path: note.path,
                            title: note.title,
                            pinned: note.pinned,
                        },
                        source: "scheduled".to_string(),
                        schedule_block: Some(block),
                    });
                }
            }
        }

        // Collect note IDs already included from schedule blocks
        let scheduled_note_ids: HashSet<i64> = results
            .iter()
            .map(|r| r.note.id)
            .collect();

        // 2. Notes with journal_date property matching this date
        let journal_rows = sqlx::query_as::<_, (i64, String, Option<String>, i32)>(
            r#"
            SELECT n.id, n.path, n.title, n.pinned
            FROM notes n
            JOIN properties p ON n.id = p.note_id
            WHERE p.key = 'journal_date' AND p.value = ?
            "#,
        )
        .bind(date)
        .fetch_all(&self.pool)
        .await?;

        // Collect journal note IDs first (before consuming the iterator)
        let journal_note_ids: HashSet<i64> = journal_rows
            .iter()
            .map(|(id, _, _, _)| *id)
            .collect();

        for (id, path, title, pinned) in journal_rows {
            // Skip if already included from schedule blocks
            if scheduled_note_ids.contains(&id) {
                continue;
            }
            results.push(NoteForDate {
                note: NoteListItem {
                    id,
                    path,
                    title,
                    pinned: pinned != 0,
                },
                source: "journal".to_string(),
                schedule_block: None,
            });
        }

        // 3. Notes created on this date (using created_date for local timezone accuracy)
        let created_rows = sqlx::query_as::<_, (i64, String, Option<String>, i32)>(
            r#"
            SELECT id, path, title, pinned
            FROM notes
            WHERE created_date = ?
            "#,
        )
        .bind(date)
        .fetch_all(&self.pool)
        .await?;

        for (id, path, title, pinned) in created_rows {
            // Skip if already included from schedule blocks or journal
            if scheduled_note_ids.contains(&id) || journal_note_ids.contains(&id) {
                continue;
            }
            results.push(NoteForDate {
                note: NoteListItem {
                    id,
                    path,
                    title,
                    pinned: pinned != 0,
                },
                source: "created".to_string(),
                schedule_block: None,
            });
        }

        Ok(results)
    }

    /// Get notes for a date range (for weekly/monthly views).
    pub async fn get_notes_for_date_range(
        &self,
        start_date: &str,
        end_date: &str,
    ) -> Result<Vec<(String, Vec<NoteForDate>)>> {
        // This is a simplified approach - get all data and group by date
        // For larger datasets, consider optimizing with a single query

        let mut date_notes: HashMap<String, Vec<NoteForDate>> = HashMap::new();

        // 1. Get all schedule blocks in range (only those with linked notes)
        let blocks = self.get_schedule_blocks_for_range(start_date, end_date).await?;
        for block in blocks {
            // Only include blocks that have a linked note
            if let Some(note_id) = block.note_id {
                let date_str = block.date.to_string();
                let note = self.get_note(note_id).await?;
                let entry = date_notes.entry(date_str.clone()).or_default();
                entry.push(NoteForDate {
                    note: NoteListItem {
                        id: note.id,
                        path: note.path,
                        title: note.title,
                        pinned: note.pinned,
                    },
                    source: "scheduled".to_string(),
                    schedule_block: Some(block),
                });
            }
        }

        // 2. Get journal_date notes in range
        let journal_rows = sqlx::query_as::<_, (i64, String, Option<String>, i32, String)>(
            r#"
            SELECT n.id, n.path, n.title, n.pinned, p.value
            FROM notes n
            JOIN properties p ON n.id = p.note_id
            WHERE p.key = 'journal_date' AND p.value >= ? AND p.value <= ?
            "#,
        )
        .bind(start_date)
        .bind(end_date)
        .fetch_all(&self.pool)
        .await?;

        for (id, path, title, pinned, date_val) in journal_rows {
            let entry = date_notes.entry(date_val).or_default();
            // Only add if not already present from schedule blocks
            if !entry.iter().any(|n| n.note.id == id) {
                entry.push(NoteForDate {
                    note: NoteListItem {
                        id,
                        path,
                        title,
                        pinned: pinned != 0,
                    },
                    source: "journal".to_string(),
                    schedule_block: None,
                });
            }
        }

        // 3. Get created notes in range (using created_date for local timezone accuracy)
        let created_rows = sqlx::query_as::<_, (i64, String, Option<String>, i32, String)>(
            r#"
            SELECT id, path, title, pinned, created_date
            FROM notes
            WHERE created_date >= ? AND created_date <= ?
            AND created_date IS NOT NULL
            "#,
        )
        .bind(start_date)
        .bind(end_date)
        .fetch_all(&self.pool)
        .await?;

        for (id, path, title, pinned, created_date) in created_rows {
            let entry = date_notes.entry(created_date).or_default();
            // Only add if not already present
            if !entry.iter().any(|n| n.note.id == id) {
                entry.push(NoteForDate {
                    note: NoteListItem {
                        id,
                        path,
                        title,
                        pinned: pinned != 0,
                    },
                    source: "created".to_string(),
                    schedule_block: None,
                });
            }
        }

        // Convert to sorted vec
        let mut result: Vec<(String, Vec<NoteForDate>)> = date_notes.into_iter().collect();
        result.sort_by(|a, b| a.0.cmp(&b.0));
        Ok(result)
    }
}
