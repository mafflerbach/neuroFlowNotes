//! Habit and habit entry operations.

use crate::Result;
use chrono::{Datelike, Duration, Local, NaiveDate, Utc};
use shared_types::{
    CreateHabitRequest, HabitDateRange, HabitDto, HabitEntryDto, HabitTrackerQuery,
    HabitTrackerResponse, HabitType, HabitWithEntries, LogHabitEntryRequest,
    UpdateHabitEntryRequest, UpdateHabitRequest,
};
use tracing::{debug, instrument};

use super::VaultRepository;

impl VaultRepository {
    // ========================================================================
    // Habit CRUD
    // ========================================================================

    /// Create a new habit.
    /// If an archived habit with the same name exists, it will be unarchived and updated.
    #[instrument(skip(self))]
    pub async fn create_habit(&self, request: &CreateHabitRequest) -> Result<i64> {
        let now = Utc::now().to_rfc3339();
        let habit_type = request.habit_type.as_str();

        // Check if there's an archived habit with the same name
        let existing = sqlx::query_scalar::<_, i64>(
            "SELECT id FROM habits WHERE name = ? COLLATE NOCASE AND archived = 1",
        )
        .bind(&request.name)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(existing_id) = existing {
            // Unarchive and update the existing habit
            sqlx::query(
                r#"
                UPDATE habits
                SET archived = 0, description = ?, habit_type = ?, unit = ?, color = ?, target_value = ?
                WHERE id = ?
                "#,
            )
            .bind(&request.description)
            .bind(habit_type)
            .bind(&request.unit)
            .bind(&request.color)
            .bind(request.target_value)
            .bind(existing_id)
            .execute(&self.pool)
            .await?;

            debug!("Unarchived and updated habit {} with id {}", request.name, existing_id);
            return Ok(existing_id);
        }

        let id = sqlx::query_scalar::<_, i64>(
            r#"
            INSERT INTO habits (name, description, habit_type, unit, color, target_value, created_at)
            VALUES (?, ?, ?, ?, ?, ?, ?)
            RETURNING id
            "#,
        )
        .bind(&request.name)
        .bind(&request.description)
        .bind(habit_type)
        .bind(&request.unit)
        .bind(&request.color)
        .bind(request.target_value)
        .bind(&now)
        .fetch_one(&self.pool)
        .await?;

        debug!("Created habit {} with id {}", request.name, id);
        Ok(id)
    }

    /// Get a habit by ID.
    pub async fn get_habit(&self, id: i64) -> Result<Option<HabitDto>> {
        let row = sqlx::query_as::<_, (i64, String, Option<String>, String, Option<String>, Option<String>, Option<f64>, i32, i32)>(
            "SELECT id, name, description, habit_type, unit, color, target_value, archived, sort_order FROM habits WHERE id = ?"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| HabitDto {
            id: r.0,
            name: r.1,
            description: r.2,
            habit_type: HabitType::from_string(&r.3),
            unit: r.4,
            color: r.5,
            target_value: r.6,
            archived: r.7 != 0,
            sort_order: r.8,
        }))
    }

    /// Get a habit by name.
    pub async fn get_habit_by_name(&self, name: &str) -> Result<Option<HabitDto>> {
        let row = sqlx::query_as::<_, (i64, String, Option<String>, String, Option<String>, Option<String>, Option<f64>, i32, i32)>(
            "SELECT id, name, description, habit_type, unit, color, target_value, archived, sort_order FROM habits WHERE name = ? COLLATE NOCASE"
        )
        .bind(name)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| HabitDto {
            id: r.0,
            name: r.1,
            description: r.2,
            habit_type: HabitType::from_string(&r.3),
            unit: r.4,
            color: r.5,
            target_value: r.6,
            archived: r.7 != 0,
            sort_order: r.8,
        }))
    }

    /// List all habits.
    pub async fn list_habits(&self, include_archived: bool) -> Result<Vec<HabitDto>> {
        let query = if include_archived {
            "SELECT id, name, description, habit_type, unit, color, target_value, archived, sort_order FROM habits ORDER BY sort_order, name"
        } else {
            "SELECT id, name, description, habit_type, unit, color, target_value, archived, sort_order FROM habits WHERE archived = 0 ORDER BY sort_order, name"
        };

        let rows = sqlx::query_as::<_, (i64, String, Option<String>, String, Option<String>, Option<String>, Option<f64>, i32, i32)>(query)
            .fetch_all(&self.pool)
            .await?;

        Ok(rows
            .into_iter()
            .map(|r| HabitDto {
                id: r.0,
                name: r.1,
                description: r.2,
                habit_type: HabitType::from_string(&r.3),
                unit: r.4,
                color: r.5,
                target_value: r.6,
                archived: r.7 != 0,
                sort_order: r.8,
            })
            .collect())
    }

    /// Update a habit.
    #[instrument(skip(self))]
    pub async fn update_habit(&self, request: &UpdateHabitRequest) -> Result<()> {
        // Build dynamic update query
        let mut updates = Vec::new();
        let mut binds: Vec<Box<dyn std::any::Any + Send + Sync>> = Vec::new();

        if let Some(ref name) = request.name {
            updates.push("name = ?");
            binds.push(Box::new(name.clone()));
        }
        if let Some(ref desc) = request.description {
            updates.push("description = ?");
            binds.push(Box::new(desc.clone()));
        }
        if let Some(ref habit_type) = request.habit_type {
            updates.push("habit_type = ?");
            binds.push(Box::new(habit_type.as_str().to_string()));
        }
        if let Some(ref unit) = request.unit {
            updates.push("unit = ?");
            binds.push(Box::new(unit.clone()));
        }
        if let Some(ref color) = request.color {
            updates.push("color = ?");
            binds.push(Box::new(color.clone()));
        }
        if let Some(target) = request.target_value {
            updates.push("target_value = ?");
            binds.push(Box::new(target));
        }
        if let Some(archived) = request.archived {
            updates.push("archived = ?");
            binds.push(Box::new(if archived { 1i32 } else { 0i32 }));
        }
        if let Some(sort_order) = request.sort_order {
            updates.push("sort_order = ?");
            binds.push(Box::new(sort_order));
        }

        if updates.is_empty() {
            return Ok(());
        }

        // For simplicity, we'll use individual queries for each field
        // This is less efficient but simpler to implement safely
        if let Some(ref name) = request.name {
            sqlx::query("UPDATE habits SET name = ? WHERE id = ?")
                .bind(name)
                .bind(request.id)
                .execute(&self.pool)
                .await?;
        }
        if let Some(ref desc) = request.description {
            sqlx::query("UPDATE habits SET description = ? WHERE id = ?")
                .bind(desc)
                .bind(request.id)
                .execute(&self.pool)
                .await?;
        }
        if let Some(ref habit_type) = request.habit_type {
            sqlx::query("UPDATE habits SET habit_type = ? WHERE id = ?")
                .bind(habit_type.as_str())
                .bind(request.id)
                .execute(&self.pool)
                .await?;
        }
        if let Some(ref unit) = request.unit {
            sqlx::query("UPDATE habits SET unit = ? WHERE id = ?")
                .bind(unit)
                .bind(request.id)
                .execute(&self.pool)
                .await?;
        }
        if let Some(ref color) = request.color {
            sqlx::query("UPDATE habits SET color = ? WHERE id = ?")
                .bind(color)
                .bind(request.id)
                .execute(&self.pool)
                .await?;
        }
        if let Some(target) = request.target_value {
            sqlx::query("UPDATE habits SET target_value = ? WHERE id = ?")
                .bind(target)
                .bind(request.id)
                .execute(&self.pool)
                .await?;
        }
        if let Some(archived) = request.archived {
            sqlx::query("UPDATE habits SET archived = ? WHERE id = ?")
                .bind(if archived { 1 } else { 0 })
                .bind(request.id)
                .execute(&self.pool)
                .await?;
        }
        if let Some(sort_order) = request.sort_order {
            sqlx::query("UPDATE habits SET sort_order = ? WHERE id = ?")
                .bind(sort_order)
                .bind(request.id)
                .execute(&self.pool)
                .await?;
        }

        debug!("Updated habit {}", request.id);
        Ok(())
    }

    /// Delete a habit (and all its entries via CASCADE).
    #[instrument(skip(self))]
    pub async fn delete_habit(&self, id: i64) -> Result<()> {
        sqlx::query("DELETE FROM habits WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;

        debug!("Deleted habit {}", id);
        Ok(())
    }

    /// Archive a habit (soft delete).
    #[instrument(skip(self))]
    pub async fn archive_habit(&self, id: i64) -> Result<()> {
        sqlx::query("UPDATE habits SET archived = 1 WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;

        debug!("Archived habit {}", id);
        Ok(())
    }

    // ========================================================================
    // Habit Entry Operations
    // ========================================================================

    /// Log a habit entry.
    #[instrument(skip(self))]
    pub async fn log_habit_entry(&self, request: &LogHabitEntryRequest) -> Result<i64> {
        let now = Utc::now().to_rfc3339();

        let id = sqlx::query_scalar::<_, i64>(
            r#"
            INSERT INTO habit_entries (habit_id, date, time, value, notes, created_at)
            VALUES (?, ?, ?, ?, ?, ?)
            RETURNING id
            "#,
        )
        .bind(request.habit_id)
        .bind(&request.date)
        .bind(&request.time)
        .bind(&request.value)
        .bind(&request.notes)
        .bind(&now)
        .fetch_one(&self.pool)
        .await?;

        debug!(
            "Logged habit entry for habit {} on {}",
            request.habit_id, request.date
        );
        Ok(id)
    }

    /// Get habit entries for a habit within a date range.
    pub async fn get_habit_entries(
        &self,
        habit_id: i64,
        start_date: &str,
        end_date: &str,
    ) -> Result<Vec<HabitEntryDto>> {
        let rows = sqlx::query_as::<_, (i64, i64, String, Option<String>, String, Option<String>)>(
            r#"
            SELECT id, habit_id, date, time, value, notes
            FROM habit_entries
            WHERE habit_id = ? AND date >= ? AND date <= ?
            ORDER BY date, time
            "#,
        )
        .bind(habit_id)
        .bind(start_date)
        .bind(end_date)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|r| HabitEntryDto {
                id: r.0,
                habit_id: r.1,
                date: r.2,
                time: r.3,
                value: r.4,
                notes: r.5,
            })
            .collect())
    }

    /// Get all habit entries for a specific date.
    pub async fn get_all_entries_for_date(&self, date: &str) -> Result<Vec<HabitEntryDto>> {
        let rows = sqlx::query_as::<_, (i64, i64, String, Option<String>, String, Option<String>)>(
            r#"
            SELECT id, habit_id, date, time, value, notes
            FROM habit_entries
            WHERE date = ?
            ORDER BY habit_id, time
            "#,
        )
        .bind(date)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|r| HabitEntryDto {
                id: r.0,
                habit_id: r.1,
                date: r.2,
                time: r.3,
                value: r.4,
                notes: r.5,
            })
            .collect())
    }

    /// Update a habit entry.
    #[instrument(skip(self))]
    pub async fn update_habit_entry(&self, request: &UpdateHabitEntryRequest) -> Result<()> {
        if let Some(ref value) = request.value {
            sqlx::query("UPDATE habit_entries SET value = ? WHERE id = ?")
                .bind(value)
                .bind(request.id)
                .execute(&self.pool)
                .await?;
        }
        if let Some(ref notes) = request.notes {
            sqlx::query("UPDATE habit_entries SET notes = ? WHERE id = ?")
                .bind(notes)
                .bind(request.id)
                .execute(&self.pool)
                .await?;
        }
        if let Some(ref time) = request.time {
            sqlx::query("UPDATE habit_entries SET time = ? WHERE id = ?")
                .bind(time)
                .bind(request.id)
                .execute(&self.pool)
                .await?;
        }

        debug!("Updated habit entry {}", request.id);
        Ok(())
    }

    /// Delete a habit entry.
    #[instrument(skip(self))]
    pub async fn delete_habit_entry(&self, id: i64) -> Result<()> {
        sqlx::query("DELETE FROM habit_entries WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;

        debug!("Deleted habit entry {}", id);
        Ok(())
    }

    /// Toggle a boolean habit for a date.
    /// Returns true if the habit is now marked as done, false if undone.
    #[instrument(skip(self))]
    pub async fn toggle_habit_for_date(&self, habit_id: i64, date: &str) -> Result<bool> {
        // Check if there's an existing entry for this date
        let existing = sqlx::query_scalar::<_, i64>(
            "SELECT id FROM habit_entries WHERE habit_id = ? AND date = ? LIMIT 1",
        )
        .bind(habit_id)
        .bind(date)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(entry_id) = existing {
            // Entry exists - delete it (toggle off)
            sqlx::query("DELETE FROM habit_entries WHERE id = ?")
                .bind(entry_id)
                .execute(&self.pool)
                .await?;
            debug!("Toggled habit {} OFF for {}", habit_id, date);
            Ok(false)
        } else {
            // No entry - create one (toggle on)
            let now = Utc::now().to_rfc3339();
            sqlx::query(
                "INSERT INTO habit_entries (habit_id, date, value, created_at) VALUES (?, ?, 'true', ?)",
            )
            .bind(habit_id)
            .bind(date)
            .bind(&now)
            .execute(&self.pool)
            .await?;
            debug!("Toggled habit {} ON for {}", habit_id, date);
            Ok(true)
        }
    }

    // ========================================================================
    // Habit Tracker Query (for embeds)
    // ========================================================================

    /// Execute a habit tracker query for embedding.
    pub async fn execute_habit_tracker_query(
        &self,
        query: &HabitTrackerQuery,
    ) -> Result<HabitTrackerResponse> {
        // Calculate date range - use reference date if provided, otherwise today
        let reference_date = query
            .date
            .as_ref()
            .and_then(|d| NaiveDate::parse_from_str(d, "%Y-%m-%d").ok())
            .unwrap_or_else(|| Local::now().date_naive());

        let (start_date, end_date) = match query.date_range {
            HabitDateRange::SingleDay => (reference_date, reference_date),
            HabitDateRange::Last7Days => (reference_date - Duration::days(6), reference_date),
            HabitDateRange::Last30Days => (reference_date - Duration::days(29), reference_date),
            HabitDateRange::ThisWeek => {
                let weekday = reference_date.weekday().num_days_from_monday();
                let start = reference_date - Duration::days(weekday as i64);
                let end = start + Duration::days(6);
                (start, end)
            }
            HabitDateRange::ThisMonth => {
                let start = NaiveDate::from_ymd_opt(reference_date.year(), reference_date.month(), 1).unwrap_or(reference_date);
                let next_month = if reference_date.month() == 12 {
                    NaiveDate::from_ymd_opt(reference_date.year() + 1, 1, 1)
                } else {
                    NaiveDate::from_ymd_opt(reference_date.year(), reference_date.month() + 1, 1)
                };
                let end = next_month.unwrap_or(reference_date) - Duration::days(1);
                (start, end)
            }
            HabitDateRange::Custom => {
                let start = query
                    .start_date
                    .as_ref()
                    .and_then(|s| NaiveDate::parse_from_str(s, "%Y-%m-%d").ok())
                    .unwrap_or(reference_date - Duration::days(6));
                let end = query
                    .end_date
                    .as_ref()
                    .and_then(|s| NaiveDate::parse_from_str(s, "%Y-%m-%d").ok())
                    .unwrap_or(reference_date);
                (start, end)
            }
        };

        let start_str = start_date.format("%Y-%m-%d").to_string();
        let end_str = end_date.format("%Y-%m-%d").to_string();

        // Get habits (filtered by names/ids if specified, preserving query order)
        let habits = if query.habits.is_empty() {
            self.list_habits(false).await?
        } else {
            let all_habits = self.list_habits(false).await?;
            // Build habits in the order specified in the query
            let mut ordered_habits = Vec::new();
            for query_habit in &query.habits {
                let query_lower = query_habit.to_lowercase();
                if let Some(habit) = all_habits.iter().find(|h| {
                    query_habit == &h.id.to_string() || query_lower == h.name.to_lowercase()
                }) {
                    ordered_habits.push(habit.clone());
                }
            }
            ordered_habits
        };

        // Get entries for each habit
        let mut habits_with_entries = Vec::new();
        for habit in habits {
            let entries = self
                .get_habit_entries(habit.id, &start_str, &end_str)
                .await?;

            // Group entries by date
            let mut entries_by_date: std::collections::HashMap<String, Vec<HabitEntryDto>> =
                std::collections::HashMap::new();
            for entry in entries {
                entries_by_date
                    .entry(entry.date.clone())
                    .or_default()
                    .push(entry);
            }

            // Convert to sorted vec of tuples
            let mut sorted_entries: Vec<(String, Vec<HabitEntryDto>)> =
                entries_by_date.into_iter().collect();
            sorted_entries.sort_by(|a, b| a.0.cmp(&b.0));

            habits_with_entries.push(HabitWithEntries {
                habit,
                entries_by_date: sorted_entries,
            });
        }

        Ok(HabitTrackerResponse {
            query: query.clone(),
            habits: habits_with_entries,
            date_range_start: start_str,
            date_range_end: end_str,
            error: None,
        })
    }
}
