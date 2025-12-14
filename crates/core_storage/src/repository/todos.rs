//! Todo/task operations.

use crate::Result;
use chrono::{DateTime, Utc};
use core_index::ParsedTodo;
use shared_types::{TaskQuery, TaskWithContext, TodoDto};

use super::VaultRepository;

impl VaultRepository {
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
                    line_number,
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
                    line_number,
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
                line_number,
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
                    line_number,
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
}
