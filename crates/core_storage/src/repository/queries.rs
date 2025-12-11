//! Query builder and search operations.

use crate::Result;
use chrono::{DateTime, Utc};
use shared_types::{
    FilterMatchMode, NoteListItem, PropertyDto, PropertyFilter, PropertyOperator,
    QueryRequest, QueryResponse, QueryResultItem, QueryResultType, SearchResult,
    TaskWithContext, TodoDto,
};

use super::VaultRepository;

impl VaultRepository {
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
    pub(crate) async fn query_tasks_by_note_ids(
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
}
