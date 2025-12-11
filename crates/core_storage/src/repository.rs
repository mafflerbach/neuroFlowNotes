//! Repository layer for vault database operations.

use crate::{Result, StorageError};
use chrono::{DateTime, Datelike, NaiveDate, NaiveTime, TimeZone, Timelike, Utc};
use core_index::{NoteAnalysis, ParsedTodo};
use rrule::{RRuleSet, Tz as RRuleTz};
use shared_types::{
    BacklinkDto, FilterMatchMode, NoteDto, NoteForDate, NoteListItem, NoteWithPropertyValue,
    PropertyDto, PropertyFilter, PropertyKeyInfo, PropertyOperator, QueryRequest, QueryResponse,
    QueryResultItem, QueryResultType, ScheduleBlockDto, SearchResult, TagDto, TaskQuery,
    TaskWithContext, TodoDto,
};
use sqlx::SqlitePool;
use tracing::{debug, instrument, warn};

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
                INSERT INTO todos (note_id, line_number, description, completed, heading_path, context, priority, due_date, created_at)
                VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
                "#,
            )
            .bind(note_id)
            .bind(todo.line_number as i32)
            .bind(&todo.description)
            .bind(todo.completed)
            .bind(&todo.heading_path)
            .bind(&todo.context)
            .bind(&todo.priority)
            .bind(&todo.due_date)
            .bind(&now)
            .execute(&self.pool)
            .await?;
        }

        Ok(())
    }

    /// Get todos for a note.
    pub async fn get_todos_for_note(&self, note_id: i64) -> Result<Vec<TodoDto>> {
        let rows = sqlx::query_as::<_, (i64, i64, Option<i32>, String, i32, Option<String>, Option<String>, Option<String>, Option<String>, Option<String>, Option<String>)>(
            "SELECT id, note_id, line_number, description, completed, heading_path, context, priority, due_date, created_at, completed_at FROM todos WHERE note_id = ?",
        )
        .bind(note_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|(id, note_id, line_number, description, completed, heading_path, context, priority, due_date, created_at, completed_at)| {
                TodoDto {
                    id,
                    note_id,
                    line_number: line_number.map(|n| n as i32),
                    description,
                    completed: completed != 0,
                    heading_path,
                    context,
                    priority,
                    due_date,
                    created_at: created_at.and_then(|s| DateTime::parse_from_rfc3339(&s).ok().map(|d| d.with_timezone(&Utc))),
                    completed_at: completed_at.and_then(|s| DateTime::parse_from_rfc3339(&s).ok().map(|d| d.with_timezone(&Utc))),
                }
            })
            .collect())
    }

    /// Get all incomplete todos.
    pub async fn get_incomplete_todos(&self) -> Result<Vec<TodoDto>> {
        let rows = sqlx::query_as::<_, (i64, i64, Option<i32>, String, i32, Option<String>, Option<String>, Option<String>, Option<String>, Option<String>, Option<String>)>(
            "SELECT id, note_id, line_number, description, completed, heading_path, context, priority, due_date, created_at, completed_at FROM todos WHERE completed = 0",
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|(id, note_id, line_number, description, completed, heading_path, context, priority, due_date, created_at, completed_at)| {
                TodoDto {
                    id,
                    note_id,
                    line_number: line_number.map(|n| n as i32),
                    description,
                    completed: completed != 0,
                    heading_path,
                    context,
                    priority,
                    due_date,
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
        let row = sqlx::query_as::<_, (i64, i64, Option<i32>, String, i32, Option<String>, Option<String>, Option<String>, Option<String>, Option<String>, Option<String>)>(
            "SELECT id, note_id, line_number, description, completed, heading_path, context, priority, due_date, created_at, completed_at FROM todos WHERE id = ?",
        )
        .bind(todo_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|(id, note_id, line_number, description, completed, heading_path, context, priority, due_date, created_at, completed_at)| {
            TodoDto {
                id,
                note_id,
                line_number: line_number.map(|n| n as i32),
                description,
                completed: completed != 0,
                heading_path,
                context,
                priority,
                due_date,
                created_at: created_at.and_then(|s| DateTime::parse_from_rfc3339(&s).ok().map(|d| d.with_timezone(&Utc))),
                completed_at: completed_at.and_then(|s| DateTime::parse_from_rfc3339(&s).ok().map(|d| d.with_timezone(&Utc))),
            }
        }))
    }

    /// Query tasks with filters, returning enriched context from parent notes.
    pub async fn query_tasks(&self, query: &TaskQuery) -> Result<Vec<TaskWithContext>> {
        // Build dynamic WHERE clause
        let mut conditions = Vec::new();
        let mut params: Vec<String> = Vec::new();

        if let Some(completed) = query.completed {
            conditions.push("t.completed = ?".to_string());
            params.push(if completed { "1".to_string() } else { "0".to_string() });
        }

        if let Some(ref ctx) = query.context {
            conditions.push("t.context = ?".to_string());
            params.push(ctx.clone());
        }

        if let Some(ref pri) = query.priority {
            conditions.push("t.priority = ?".to_string());
            params.push(pri.clone());
        }

        if let Some(ref due_from) = query.due_from {
            conditions.push("t.due_date >= ?".to_string());
            params.push(due_from.clone());
        }

        if let Some(ref due_to) = query.due_to {
            conditions.push("t.due_date <= ?".to_string());
            params.push(due_to.clone());
        }

        // Property filter (key=value format)
        let mut prop_key: Option<String> = None;
        let mut prop_value: Option<String> = None;
        if let Some(ref filter) = query.property_filter {
            if let Some((k, v)) = filter.split_once('=') {
                prop_key = Some(k.to_string());
                prop_value = Some(v.to_string());
                conditions.push("EXISTS (SELECT 1 FROM properties p WHERE p.note_id = t.note_id AND p.key = ? AND p.value = ?)".to_string());
            }
        }

        let where_clause = if conditions.is_empty() {
            "1=1".to_string()
        } else {
            conditions.join(" AND ")
        };

        let limit = query.limit.unwrap_or(100);

        let sql = format!(
            r#"
            SELECT
                t.id, t.note_id, t.line_number, t.description, t.completed, t.heading_path,
                t.context, t.priority, t.due_date, t.created_at, t.completed_at,
                n.path, n.title
            FROM todos t
            JOIN notes n ON t.note_id = n.id
            WHERE {}
            ORDER BY
                CASE WHEN t.due_date IS NOT NULL THEN 0 ELSE 1 END,
                t.due_date,
                CASE t.priority WHEN 'high' THEN 0 WHEN 'medium' THEN 1 WHEN 'low' THEN 2 ELSE 3 END,
                t.created_at DESC
            LIMIT ?
            "#,
            where_clause
        );

        // Build query dynamically
        let mut sqlx_query = sqlx::query_as::<_, (
            i64, i64, Option<i32>, String, i32, Option<String>,
            Option<String>, Option<String>, Option<String>, Option<String>, Option<String>,
            String, Option<String>
        )>(&sql);

        // Bind parameters in order
        for param in &params {
            sqlx_query = sqlx_query.bind(param);
        }
        if let Some(ref k) = prop_key {
            sqlx_query = sqlx_query.bind(k);
        }
        if let Some(ref v) = prop_value {
            sqlx_query = sqlx_query.bind(v);
        }
        sqlx_query = sqlx_query.bind(limit);

        let rows = sqlx_query.fetch_all(&self.pool).await?;

        let mut results = Vec::new();
        for (id, note_id, line_number, description, completed, heading_path, context, priority, due_date, created_at, completed_at, note_path, note_title) in rows {
            // Get properties for this note
            let note_properties = self.get_properties_for_note(note_id).await?;

            results.push(TaskWithContext {
                todo: TodoDto {
                    id,
                    note_id,
                    line_number: line_number.map(|n| n as i32),
                    description,
                    completed: completed != 0,
                    heading_path,
                    context,
                    priority,
                    due_date,
                    created_at: created_at.and_then(|s| DateTime::parse_from_rfc3339(&s).ok().map(|d| d.with_timezone(&Utc))),
                    completed_at: completed_at.and_then(|s| DateTime::parse_from_rfc3339(&s).ok().map(|d| d.with_timezone(&Utc))),
                },
                note_path,
                note_title,
                note_properties,
            });
        }

        Ok(results)
    }

    /// Get all distinct contexts used in tasks.
    pub async fn get_task_contexts(&self) -> Result<Vec<String>> {
        let contexts = sqlx::query_scalar::<_, String>(
            "SELECT DISTINCT context FROM todos WHERE context IS NOT NULL ORDER BY context"
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(contexts)
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
        rrule: Option<&str>,
    ) -> Result<i64> {
        let id = sqlx::query_scalar::<_, i64>(
            r#"
            INSERT INTO schedule_blocks (note_id, date, start_time, end_time, label, color, context, rrule)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)
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
        .bind(rrule)
        .fetch_one(&self.pool)
        .await?;

        Ok(id)
    }

    /// Get schedule blocks for a date range, expanding recurring blocks.
    pub async fn get_schedule_blocks_for_range(
        &self,
        start_date: &str,
        end_date: &str,
    ) -> Result<Vec<ScheduleBlockDto>> {
        // First get non-recurring blocks in the range
        let non_recurring_rows = sqlx::query_as::<_, (i64, Option<i64>, String, String, String, Option<String>, Option<String>, Option<String>, Option<String>)>(
            "SELECT id, note_id, date, start_time, end_time, label, color, context, rrule FROM schedule_blocks WHERE (rrule IS NULL OR rrule = '') AND date >= ? AND date <= ? ORDER BY date, start_time",
        )
        .bind(start_date)
        .bind(end_date)
        .fetch_all(&self.pool)
        .await?;

        let mut blocks: Vec<ScheduleBlockDto> = non_recurring_rows
            .into_iter()
            .filter_map(|(id, note_id, date, start_time, end_time, label, color, context, rrule)| {
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
                    rrule,
                    is_occurrence: false,
                })
            })
            .collect();

        // Now get recurring blocks and expand them
        // Filter by base date <= end_date since recurring events can't produce occurrences before their start
        let recurring_rows = sqlx::query_as::<_, (i64, Option<i64>, String, String, String, Option<String>, Option<String>, Option<String>, Option<String>)>(
            "SELECT id, note_id, date, start_time, end_time, label, color, context, rrule FROM schedule_blocks WHERE rrule IS NOT NULL AND rrule != '' AND date <= ?",
        )
        .bind(end_date)
        .fetch_all(&self.pool)
        .await?;

        let start = start_date.parse::<NaiveDate>().ok();
        let end = end_date.parse::<NaiveDate>().ok();

        if let (Some(start), Some(end)) = (start, end) {
            for (id, note_id, date_str, start_time_str, end_time_str, label, color, context, rrule_opt) in recurring_rows {
                if let Some(rrule_str) = rrule_opt {
                    let base_date: NaiveDate = match date_str.parse() {
                        Ok(d) => d,
                        Err(_) => continue,
                    };
                    let start_time: NaiveTime = match start_time_str.parse() {
                        Ok(t) => t,
                        Err(_) => continue,
                    };
                    let end_time: NaiveTime = match end_time_str.parse() {
                        Ok(t) => t,
                        Err(_) => continue,
                    };

                    // Expand rrule occurrences within the date range
                    match expand_rrule(&rrule_str, base_date, start_time, start, end) {
                        Ok(occurrences) => {
                            for occ_date in occurrences {
                                blocks.push(ScheduleBlockDto {
                                    id,
                                    note_id,
                                    date: occ_date,
                                    start_time,
                                    end_time,
                                    label: label.clone(),
                                    color: color.clone(),
                                    context: context.clone(),
                                    rrule: Some(rrule_str.clone()),
                                    is_occurrence: occ_date != base_date,
                                });
                            }
                        }
                        Err(e) => {
                            warn!("Failed to expand rrule for block {}: {}", id, e);
                            // Still include the base block if its date is in range
                            if base_date >= start && base_date <= end {
                                blocks.push(ScheduleBlockDto {
                                    id,
                                    note_id,
                                    date: base_date,
                                    start_time,
                                    end_time,
                                    label,
                                    color,
                                    context,
                                    rrule: Some(rrule_str),
                                    is_occurrence: false,
                                });
                            }
                        }
                    }
                }
            }
        }

        // Sort by date and time
        blocks.sort_by(|a, b| {
            a.date.cmp(&b.date).then_with(|| a.start_time.cmp(&b.start_time))
        });

        Ok(blocks)
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
        rrule: Option<&str>,
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
        updates.push("rrule = ?");

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
        q = q.bind(rrule);
        q = q.bind(id);

        q.execute(&self.pool).await?;
        Ok(())
    }

    /// Get a schedule block by ID.
    pub async fn get_schedule_block(&self, id: i64) -> Result<Option<ScheduleBlockDto>> {
        let row = sqlx::query_as::<_, (i64, Option<i64>, String, String, String, Option<String>, Option<String>, Option<String>, Option<String>)>(
            "SELECT id, note_id, date, start_time, end_time, label, color, context, rrule FROM schedule_blocks WHERE id = ?",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.and_then(|(id, note_id, date, start_time, end_time, label, color, context, rrule)| {
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
                rrule,
                is_occurrence: false,
            })
        }))
    }

    /// Get schedule blocks for a single date.
    pub async fn get_schedule_blocks_for_date(&self, date: &str) -> Result<Vec<ScheduleBlockDto>> {
        self.get_schedule_blocks_for_range(date, date).await
    }

    /// Get schedule blocks linked to a specific note.
    pub async fn get_schedule_blocks_for_note(&self, note_id: i64) -> Result<Vec<ScheduleBlockDto>> {
        let rows = sqlx::query_as::<_, (i64, Option<i64>, String, String, String, Option<String>, Option<String>, Option<String>, Option<String>)>(
            "SELECT id, note_id, date, start_time, end_time, label, color, context, rrule FROM schedule_blocks WHERE note_id = ? ORDER BY date, start_time",
        )
        .bind(note_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .filter_map(|(id, note_id, date, start_time, end_time, label, color, context, rrule)| {
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
                    rrule,
                    is_occurrence: false,
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

    /// Get properties for multiple notes at once (batch query to avoid N+1).
    /// Returns a HashMap from note_id to Vec<PropertyDto>.
    pub async fn get_properties_for_notes(
        &self,
        note_ids: &[i64],
    ) -> Result<std::collections::HashMap<i64, Vec<PropertyDto>>> {
        use std::collections::HashMap;

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

    // ========================================================================
    // Query Builder
    // ========================================================================

    /// Get all distinct property keys used in the vault.
    pub async fn get_property_keys(&self) -> Result<Vec<PropertyKeyInfo>> {
        // Get all distinct keys with usage count
        let rows = sqlx::query_as::<_, (String, i64)>(
            r#"
            SELECT key, COUNT(DISTINCT note_id) as usage_count
            FROM properties
            GROUP BY key
            ORDER BY usage_count DESC, key
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        let mut results = Vec::new();
        for (key, usage_count) in rows {
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

    /// Run a query with property filters.
    pub async fn run_query(&self, request: &QueryRequest) -> Result<QueryResponse> {
        let limit = request.limit.unwrap_or(100);

        // Build the WHERE clause for property filters
        let (note_id_subquery, params) = self.build_property_filter_sql(&request.filters, &request.match_mode)?;

        let mut results = Vec::new();
        let mut total_count: i64 = 0;

        // Get matching note IDs first
        let note_ids = self.get_matching_note_ids(&note_id_subquery, &params).await?;

        match request.result_type {
            QueryResultType::Tasks | QueryResultType::Both => {
                // Query tasks from matching notes
                let tasks = self.query_tasks_by_note_ids(&note_ids, request.include_completed, limit).await?;
                total_count += tasks.len() as i64;

                for task in tasks {
                    results.push(QueryResultItem {
                        item_type: "task".to_string(),
                        task: Some(task.clone()),
                        note: None,
                        properties: task.note_properties,
                    });
                }
            }
            QueryResultType::Notes => {}
        }

        match request.result_type {
            QueryResultType::Notes | QueryResultType::Both => {
                // Query notes directly
                let notes = self.query_notes_by_ids(&note_ids, limit).await?;

                // For Both mode, don't double-count notes that have tasks
                if matches!(request.result_type, QueryResultType::Notes) {
                    total_count += notes.len() as i64;
                }

                for (note, properties) in notes {
                    // In Both mode, skip notes already represented by tasks
                    if matches!(request.result_type, QueryResultType::Both) {
                        if results.iter().any(|r| {
                            r.task.as_ref().map(|t| t.todo.note_id) == Some(note.id)
                        }) {
                            continue;
                        }
                    }

                    results.push(QueryResultItem {
                        item_type: "note".to_string(),
                        task: None,
                        note: Some(note),
                        properties,
                    });
                }
            }
            QueryResultType::Tasks => {}
        }

        Ok(QueryResponse {
            results,
            total_count,
        })
    }

    /// Build SQL for property filters.
    fn build_property_filter_sql(
        &self,
        filters: &[PropertyFilter],
        match_mode: &FilterMatchMode,
    ) -> Result<(String, Vec<String>)> {
        if filters.is_empty() {
            // No filters - return all notes
            return Ok(("SELECT id FROM notes".to_string(), Vec::new()));
        }

        let mut conditions = Vec::new();
        let mut params = Vec::new();

        for filter in filters {
            let condition = match filter.operator {
                PropertyOperator::Exists => {
                    params.push(filter.key.clone());
                    "EXISTS (SELECT 1 FROM properties WHERE note_id = n.id AND key = ?)".to_string()
                }
                PropertyOperator::NotExists => {
                    params.push(filter.key.clone());
                    "NOT EXISTS (SELECT 1 FROM properties WHERE note_id = n.id AND key = ?)".to_string()
                }
                PropertyOperator::Equals => {
                    params.push(filter.key.clone());
                    params.push(filter.value.clone().unwrap_or_default());
                    "EXISTS (SELECT 1 FROM properties WHERE note_id = n.id AND key = ? AND value = ?)".to_string()
                }
                PropertyOperator::NotEquals => {
                    params.push(filter.key.clone());
                    params.push(filter.value.clone().unwrap_or_default());
                    "NOT EXISTS (SELECT 1 FROM properties WHERE note_id = n.id AND key = ? AND value = ?)".to_string()
                }
                PropertyOperator::Contains => {
                    params.push(filter.key.clone());
                    params.push(format!("%{}%", filter.value.clone().unwrap_or_default()));
                    "EXISTS (SELECT 1 FROM properties WHERE note_id = n.id AND key = ? AND value LIKE ?)".to_string()
                }
                PropertyOperator::StartsWith => {
                    params.push(filter.key.clone());
                    params.push(format!("{}%", filter.value.clone().unwrap_or_default()));
                    "EXISTS (SELECT 1 FROM properties WHERE note_id = n.id AND key = ? AND value LIKE ?)".to_string()
                }
                PropertyOperator::EndsWith => {
                    params.push(filter.key.clone());
                    params.push(format!("%{}", filter.value.clone().unwrap_or_default()));
                    "EXISTS (SELECT 1 FROM properties WHERE note_id = n.id AND key = ? AND value LIKE ?)".to_string()
                }
            };
            conditions.push(condition);
        }

        let joiner = match match_mode {
            FilterMatchMode::All => " AND ",
            FilterMatchMode::Any => " OR ",
        };

        let where_clause = conditions.join(joiner);
        let sql = format!("SELECT id FROM notes n WHERE {}", where_clause);

        Ok((sql, params))
    }

    /// Get note IDs matching the filter query.
    async fn get_matching_note_ids(&self, sql: &str, params: &[String]) -> Result<Vec<i64>> {
        let mut query = sqlx::query_scalar::<_, i64>(sql);
        for param in params {
            query = query.bind(param);
        }
        let ids = query.fetch_all(&self.pool).await?;
        Ok(ids)
    }

    /// Query tasks by note IDs.
    async fn query_tasks_by_note_ids(
        &self,
        note_ids: &[i64],
        include_completed: bool,
        limit: i32,
    ) -> Result<Vec<TaskWithContext>> {
        if note_ids.is_empty() {
            return Ok(Vec::new());
        }

        // Build IN clause
        let placeholders: Vec<String> = note_ids.iter().map(|_| "?".to_string()).collect();
        let in_clause = placeholders.join(", ");

        let completed_filter = if include_completed {
            "1=1"
        } else {
            "t.completed = 0"
        };

        let sql = format!(
            r#"
            SELECT
                t.id, t.note_id, t.line_number, t.description, t.completed, t.heading_path,
                t.context, t.priority, t.due_date, t.created_at, t.completed_at,
                n.path, n.title
            FROM todos t
            JOIN notes n ON t.note_id = n.id
            WHERE t.note_id IN ({}) AND {}
            ORDER BY
                CASE WHEN t.due_date IS NOT NULL THEN 0 ELSE 1 END,
                t.due_date,
                CASE t.priority WHEN 'high' THEN 0 WHEN 'medium' THEN 1 WHEN 'low' THEN 2 ELSE 3 END,
                t.created_at DESC
            LIMIT ?
            "#,
            in_clause, completed_filter
        );

        let mut query = sqlx::query_as::<_, (
            i64, i64, Option<i32>, String, i32, Option<String>,
            Option<String>, Option<String>, Option<String>, Option<String>, Option<String>,
            String, Option<String>
        )>(&sql);

        for id in note_ids {
            query = query.bind(id);
        }
        query = query.bind(limit);

        let rows = query.fetch_all(&self.pool).await?;

        // Batch fetch all properties for the note_ids we found in tasks
        let task_note_ids: Vec<i64> = rows.iter().map(|r| r.1).collect();
        let properties_map = self.get_properties_for_notes(&task_note_ids).await?;

        let mut results = Vec::new();
        for (id, note_id, line_number, description, completed, heading_path, context, priority, due_date, created_at, completed_at, note_path, note_title) in rows {
            let note_properties = properties_map.get(&note_id).cloned().unwrap_or_default();

            results.push(TaskWithContext {
                todo: TodoDto {
                    id,
                    note_id,
                    line_number: line_number.map(|n| n as i32),
                    description,
                    completed: completed != 0,
                    heading_path,
                    context,
                    priority,
                    due_date,
                    created_at: created_at.and_then(|s| DateTime::parse_from_rfc3339(&s).ok().map(|d| d.with_timezone(&Utc))),
                    completed_at: completed_at.and_then(|s| DateTime::parse_from_rfc3339(&s).ok().map(|d| d.with_timezone(&Utc))),
                },
                note_path,
                note_title,
                note_properties,
            });
        }

        Ok(results)
    }

    /// Query notes by IDs.
    async fn query_notes_by_ids(
        &self,
        note_ids: &[i64],
        limit: i32,
    ) -> Result<Vec<(NoteListItem, Vec<PropertyDto>)>> {
        if note_ids.is_empty() {
            return Ok(Vec::new());
        }

        let placeholders: Vec<String> = note_ids.iter().map(|_| "?".to_string()).collect();
        let in_clause = placeholders.join(", ");

        let sql = format!(
            "SELECT id, path, title, pinned FROM notes WHERE id IN ({}) ORDER BY path LIMIT ?",
            in_clause
        );

        let mut query = sqlx::query_as::<_, (i64, String, Option<String>, i32)>(&sql);
        for id in note_ids {
            query = query.bind(id);
        }
        query = query.bind(limit);

        let rows = query.fetch_all(&self.pool).await?;

        // Batch fetch all properties for the note_ids we found
        let found_note_ids: Vec<i64> = rows.iter().map(|r| r.0).collect();
        let properties_map = self.get_properties_for_notes(&found_note_ids).await?;

        let mut results = Vec::new();
        for (id, path, title, pinned) in rows {
            let properties = properties_map.get(&id).cloned().unwrap_or_default();
            results.push((
                NoteListItem {
                    id,
                    path,
                    title,
                    pinned: pinned != 0,
                },
                properties,
            ));
        }

        Ok(results)
    }

    // ========================================================================
    // Notes by Date
    // ========================================================================

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
        let scheduled_note_ids: std::collections::HashSet<i64> = results
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
        let journal_note_ids: std::collections::HashSet<i64> = journal_rows
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

/// Expand an RRULE to get occurrences within a date range.
fn expand_rrule(
    rrule_str: &str,
    base_date: NaiveDate,
    base_time: NaiveTime,
    range_start: NaiveDate,
    range_end: NaiveDate,
) -> std::result::Result<Vec<NaiveDate>, String> {
    // Build the full RRULE string with DTSTART in UTC format
    let dtstart = format!(
        "DTSTART:{}T{:02}{:02}{:02}Z",
        base_date.format("%Y%m%d"),
        base_time.hour(),
        base_time.minute(),
        base_time.second()
    );

    let full_rrule = format!("{}\nRRULE:{}", dtstart, rrule_str);

    // Parse the RRULE
    let rruleset: RRuleSet = full_rrule.parse().map_err(|e| format!("Invalid rrule: {}", e))?;

    // Convert range to chrono-tz datetimes for the rrule crate
    let after = RRuleTz::UTC.with_ymd_and_hms(
        range_start.year(),
        range_start.month(),
        range_start.day(),
        0, 0, 0
    ).single().ok_or("Invalid start date")?;

    let before = RRuleTz::UTC.with_ymd_and_hms(
        range_end.year(),
        range_end.month(),
        range_end.day(),
        23, 59, 59
    ).single().ok_or("Invalid end date")?;

    // Get occurrences in range (limit to 500 to prevent runaway)
    let occurrences = rruleset
        .after(after)
        .before(before)
        .all(500);

    // Check if there was a limit error
    if occurrences.limited {
        warn!("RRULE expansion hit limit of 500 occurrences");
    }

    // Extract dates
    let dates: Vec<NaiveDate> = occurrences
        .dates
        .into_iter()
        .map(|dt| NaiveDate::from_ymd_opt(dt.year(), dt.month(), dt.day()).unwrap())
        .collect();

    Ok(dates)
}
