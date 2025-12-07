//! Repository layer for vault database operations.

use crate::{Result, StorageError};
use chrono::{DateTime, Utc};
use core_index::{NoteAnalysis, ParsedTodo};
use shared_types::{BacklinkDto, NoteDto, NoteForDate, NoteListItem, PropertyDto, ScheduleBlockDto, SearchResult, TagDto, TodoDto};
use sqlx::SqlitePool;
use tracing::{debug, instrument};

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

    // ========================================================================
    // Notes
    // ========================================================================

    /// Insert or update a note.
    #[instrument(skip(self, hash))]
    pub async fn upsert_note(
        &self,
        path: &str,
        title: Option<&str>,
        hash: &str,
    ) -> Result<i64> {
        let now = Utc::now().to_rfc3339();

        let result = sqlx::query_scalar::<_, i64>(
            r#"
            INSERT INTO notes (path, title, hash, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?)
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

    // ========================================================================
    // Tags
    // ========================================================================

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

    // ========================================================================
    // Backlinks
    // ========================================================================

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

    // ========================================================================
    // Todos
    // ========================================================================

    /// Replace all todos for a note.
    pub async fn replace_todos(&self, note_id: i64, todos: &[ParsedTodo]) -> Result<()> {
        let now = Utc::now().to_rfc3339();

        // Delete existing todos
        sqlx::query("DELETE FROM todos WHERE note_id = ?")
            .bind(note_id)
            .execute(&self.pool)
            .await?;

        // Insert new todos
        for todo in todos {
            sqlx::query(
                r#"
                INSERT INTO todos (note_id, line_number, description, completed, heading_path, created_at)
                VALUES (?, ?, ?, ?, ?, ?)
                "#,
            )
            .bind(note_id)
            .bind(todo.line_number as i32)
            .bind(&todo.description)
            .bind(todo.completed)
            .bind(&todo.heading_path)
            .bind(&now)
            .execute(&self.pool)
            .await?;
        }

        Ok(())
    }

    /// Get todos for a note.
    pub async fn get_todos_for_note(&self, note_id: i64) -> Result<Vec<TodoDto>> {
        let rows = sqlx::query_as::<_, (i64, i64, Option<i32>, String, i32, Option<String>, Option<String>, Option<String>)>(
            "SELECT id, note_id, line_number, description, completed, heading_path, created_at, completed_at FROM todos WHERE note_id = ?",
        )
        .bind(note_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|(id, note_id, line_number, description, completed, heading_path, created_at, completed_at)| {
                TodoDto {
                    id,
                    note_id,
                    line_number: line_number.map(|n| n as i32),
                    description,
                    completed: completed != 0,
                    heading_path,
                    created_at: created_at.and_then(|s| DateTime::parse_from_rfc3339(&s).ok().map(|d| d.with_timezone(&Utc))),
                    completed_at: completed_at.and_then(|s| DateTime::parse_from_rfc3339(&s).ok().map(|d| d.with_timezone(&Utc))),
                }
            })
            .collect())
    }

    /// Get all incomplete todos.
    pub async fn get_incomplete_todos(&self) -> Result<Vec<TodoDto>> {
        let rows = sqlx::query_as::<_, (i64, i64, Option<i32>, String, i32, Option<String>, Option<String>, Option<String>)>(
            "SELECT id, note_id, line_number, description, completed, heading_path, created_at, completed_at FROM todos WHERE completed = 0",
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|(id, note_id, line_number, description, completed, heading_path, created_at, completed_at)| {
                TodoDto {
                    id,
                    note_id,
                    line_number: line_number.map(|n| n as i32),
                    description,
                    completed: completed != 0,
                    heading_path,
                    created_at: created_at.and_then(|s| DateTime::parse_from_rfc3339(&s).ok().map(|d| d.with_timezone(&Utc))),
                    completed_at: completed_at.and_then(|s| DateTime::parse_from_rfc3339(&s).ok().map(|d| d.with_timezone(&Utc))),
                }
            })
            .collect())
    }

    /// Update a todo's completion status.
    pub async fn update_todo_completion(&self, todo_id: i64, completed: bool) -> Result<()> {
        let completed_at = if completed {
            Some(Utc::now().to_rfc3339())
        } else {
            None
        };

        sqlx::query("UPDATE todos SET completed = ?, completed_at = ? WHERE id = ?")
            .bind(completed)
            .bind(completed_at)
            .bind(todo_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    /// Get a todo by ID.
    pub async fn get_todo(&self, todo_id: i64) -> Result<Option<TodoDto>> {
        let row = sqlx::query_as::<_, (i64, i64, Option<i32>, String, i32, Option<String>, Option<String>, Option<String>)>(
            "SELECT id, note_id, line_number, description, completed, heading_path, created_at, completed_at FROM todos WHERE id = ?",
        )
        .bind(todo_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|(id, note_id, line_number, description, completed, heading_path, created_at, completed_at)| {
            TodoDto {
                id,
                note_id,
                line_number: line_number.map(|n| n as i32),
                description,
                completed: completed != 0,
                heading_path,
                created_at: created_at.and_then(|s| DateTime::parse_from_rfc3339(&s).ok().map(|d| d.with_timezone(&Utc))),
                completed_at: completed_at.and_then(|s| DateTime::parse_from_rfc3339(&s).ok().map(|d| d.with_timezone(&Utc))),
            }
        }))
    }

    // ========================================================================
    // Schedule Blocks
    // ========================================================================

    /// Create a schedule block.
    pub async fn create_schedule_block(
        &self,
        note_id: Option<i64>,
        date: &str,
        start_time: &str,
        end_time: &str,
        label: Option<&str>,
        color: Option<&str>,
        context: Option<&str>,
    ) -> Result<i64> {
        let id = sqlx::query_scalar::<_, i64>(
            r#"
            INSERT INTO schedule_blocks (note_id, date, start_time, end_time, label, color, context)
            VALUES (?, ?, ?, ?, ?, ?, ?)
            RETURNING id
            "#,
        )
        .bind(note_id)
        .bind(date)
        .bind(start_time)
        .bind(end_time)
        .bind(label)
        .bind(color)
        .bind(context)
        .fetch_one(&self.pool)
        .await?;

        Ok(id)
    }

    /// Get schedule blocks for a date range.
    pub async fn get_schedule_blocks_for_range(
        &self,
        start_date: &str,
        end_date: &str,
    ) -> Result<Vec<ScheduleBlockDto>> {
        let rows = sqlx::query_as::<_, (i64, Option<i64>, String, String, String, Option<String>, Option<String>, Option<String>)>(
            "SELECT id, note_id, date, start_time, end_time, label, color, context FROM schedule_blocks WHERE date >= ? AND date <= ? ORDER BY date, start_time",
        )
        .bind(start_date)
        .bind(end_date)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .filter_map(|(id, note_id, date, start_time, end_time, label, color, context)| {
                let date = date.parse().ok()?;
                let start_time = start_time.parse().ok()?;
                let end_time = end_time.parse().ok()?;
                Some(ScheduleBlockDto {
                    id,
                    note_id,
                    date,
                    start_time,
                    end_time,
                    label,
                    color,
                    context,
                })
            })
            .collect())
    }

    /// Delete a schedule block.
    pub async fn delete_schedule_block(&self, id: i64) -> Result<()> {
        sqlx::query("DELETE FROM schedule_blocks WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    /// Update a schedule block.
    pub async fn update_schedule_block(
        &self,
        id: i64,
        note_id: Option<i64>,
        date: Option<&str>,
        start_time: Option<&str>,
        end_time: Option<&str>,
        label: Option<&str>,
        color: Option<&str>,
        context: Option<&str>,
    ) -> Result<()> {
        // Build dynamic update query
        let mut updates = vec![];
        if note_id.is_some() { updates.push("note_id = ?"); }
        if date.is_some() { updates.push("date = ?"); }
        if start_time.is_some() { updates.push("start_time = ?"); }
        if end_time.is_some() { updates.push("end_time = ?"); }
        updates.push("label = ?");
        updates.push("color = ?");
        updates.push("context = ?");

        let query = format!(
            "UPDATE schedule_blocks SET {} WHERE id = ?",
            updates.join(", ")
        );

        let mut q = sqlx::query(&query);
        if let Some(v) = note_id { q = q.bind(v); }
        if let Some(v) = date { q = q.bind(v); }
        if let Some(v) = start_time { q = q.bind(v); }
        if let Some(v) = end_time { q = q.bind(v); }
        q = q.bind(label);
        q = q.bind(color);
        q = q.bind(context);
        q = q.bind(id);

        q.execute(&self.pool).await?;
        Ok(())
    }

    /// Get a schedule block by ID.
    pub async fn get_schedule_block(&self, id: i64) -> Result<Option<ScheduleBlockDto>> {
        let row = sqlx::query_as::<_, (i64, Option<i64>, String, String, String, Option<String>, Option<String>, Option<String>)>(
            "SELECT id, note_id, date, start_time, end_time, label, color, context FROM schedule_blocks WHERE id = ?",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.and_then(|(id, note_id, date, start_time, end_time, label, color, context)| {
            let date = date.parse().ok()?;
            let start_time = start_time.parse().ok()?;
            let end_time = end_time.parse().ok()?;
            Some(ScheduleBlockDto {
                id,
                note_id,
                date,
                start_time,
                end_time,
                label,
                color,
                context,
            })
        }))
    }

    /// Get schedule blocks for a single date.
    pub async fn get_schedule_blocks_for_date(&self, date: &str) -> Result<Vec<ScheduleBlockDto>> {
        self.get_schedule_blocks_for_range(date, date).await
    }

    /// Get schedule blocks linked to a specific note.
    pub async fn get_schedule_blocks_for_note(&self, note_id: i64) -> Result<Vec<ScheduleBlockDto>> {
        let rows = sqlx::query_as::<_, (i64, Option<i64>, String, String, String, Option<String>, Option<String>, Option<String>)>(
            "SELECT id, note_id, date, start_time, end_time, label, color, context FROM schedule_blocks WHERE note_id = ? ORDER BY date, start_time",
        )
        .bind(note_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .filter_map(|(id, note_id, date, start_time, end_time, label, color, context)| {
                let date = date.parse().ok()?;
                let start_time = start_time.parse().ok()?;
                let end_time = end_time.parse().ok()?;
                Some(ScheduleBlockDto {
                    id,
                    note_id,
                    date,
                    start_time,
                    end_time,
                    label,
                    color,
                    context,
                })
            })
            .collect())
    }

    // ========================================================================
    // Properties
    // ========================================================================

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
    // Notes by Date
    // ========================================================================

    /// Get notes for a specific date, ordered by: scheduled > journal > created.
    pub async fn get_notes_for_date(&self, date: &str) -> Result<Vec<NoteForDate>> {
        let mut results = Vec::new();

        // 1. Notes with schedule blocks on this date
        let scheduled_rows = sqlx::query_as::<_, (i64, String, Option<String>, i32, i64, String, String, Option<String>, Option<String>, Option<String>)>(
            r#"
            SELECT n.id, n.path, n.title, n.pinned, sb.id, sb.start_time, sb.end_time, sb.label, sb.color, sb.context
            FROM notes n
            JOIN schedule_blocks sb ON n.id = sb.note_id
            WHERE sb.date = ?
            ORDER BY sb.start_time
            "#,
        )
        .bind(date)
        .fetch_all(&self.pool)
        .await?;

        for (id, path, title, pinned, sb_id, start_time, end_time, label, color, context) in scheduled_rows {
            let date_parsed = date.parse().ok();
            let start_parsed = start_time.parse().ok();
            let end_parsed = end_time.parse().ok();

            if let (Some(d), Some(st), Some(et)) = (date_parsed, start_parsed, end_parsed) {
                results.push(NoteForDate {
                    note: NoteListItem {
                        id,
                        path,
                        title,
                        pinned: pinned != 0,
                    },
                    source: "scheduled".to_string(),
                    schedule_block: Some(ScheduleBlockDto {
                        id: sb_id,
                        note_id: Some(id),
                        date: d,
                        start_time: st,
                        end_time: et,
                        label,
                        color,
                        context,
                    }),
                });
            }
        }

        // 2. Notes with journal_date property matching this date
        let journal_rows = sqlx::query_as::<_, (i64, String, Option<String>, i32)>(
            r#"
            SELECT n.id, n.path, n.title, n.pinned
            FROM notes n
            JOIN properties p ON n.id = p.note_id
            WHERE p.key = 'journal_date' AND p.value = ?
            AND n.id NOT IN (SELECT note_id FROM schedule_blocks WHERE date = ?)
            "#,
        )
        .bind(date)
        .bind(date)
        .fetch_all(&self.pool)
        .await?;

        for (id, path, title, pinned) in journal_rows {
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

        // 3. Notes created on this date
        let created_rows = sqlx::query_as::<_, (i64, String, Option<String>, i32)>(
            r#"
            SELECT n.id, n.path, n.title, n.pinned
            FROM notes n
            WHERE date(n.created_at) = ?
            AND n.id NOT IN (SELECT note_id FROM schedule_blocks WHERE date = ?)
            AND n.id NOT IN (SELECT note_id FROM properties WHERE key = 'journal_date' AND value = ?)
            "#,
        )
        .bind(date)
        .bind(date)
        .bind(date)
        .fetch_all(&self.pool)
        .await?;

        for (id, path, title, pinned) in created_rows {
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

        let mut date_notes: std::collections::HashMap<String, Vec<NoteForDate>> = std::collections::HashMap::new();

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

        // 3. Get created notes in range
        let created_rows = sqlx::query_as::<_, (i64, String, Option<String>, i32, String)>(
            r#"
            SELECT id, path, title, pinned, date(created_at) as created_date
            FROM notes
            WHERE date(created_at) >= ? AND date(created_at) <= ?
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

    // ========================================================================
    // Full-Text Search
    // ========================================================================

    /// Update the FTS index for a note.
    pub async fn update_fts(&self, note_id: i64, content: &str) -> Result<()> {
        // Delete existing FTS entry
        sqlx::query("DELETE FROM notes_fts WHERE rowid = ?")
            .bind(note_id)
            .execute(&self.pool)
            .await?;

        // Insert new FTS entry
        sqlx::query("INSERT INTO notes_fts (rowid, content) VALUES (?, ?)")
            .bind(note_id)
            .bind(content)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    /// Search notes using full-text search.
    pub async fn search(&self, query: &str, limit: i32) -> Result<Vec<SearchResult>> {
        let rows = sqlx::query_as::<_, (i64, String, Option<String>, String, f64)>(
            r#"
            SELECT n.id, n.path, n.title, snippet(notes_fts, 0, '<mark>', '</mark>', '...', 32), bm25(notes_fts)
            FROM notes_fts
            JOIN notes n ON notes_fts.rowid = n.id
            WHERE notes_fts MATCH ?
            ORDER BY bm25(notes_fts)
            LIMIT ?
            "#,
        )
        .bind(query)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|(note_id, path, title, snippet, score)| SearchResult {
                note_id,
                path,
                title,
                snippet: Some(snippet),
                score: -score, // bm25 returns negative scores, lower is better
            })
            .collect())
    }

    // ========================================================================
    // Indexing Helper
    // ========================================================================

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
