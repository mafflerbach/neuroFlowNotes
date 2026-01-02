//! Schedule block operations.

use crate::Result;
use chrono::{Datelike, NaiveDate, NaiveTime, TimeZone, Timelike};
use rrule::{RRuleSet, Tz as RRuleTz};
use shared_types::ScheduleBlockDto;
use tracing::warn;

use super::VaultRepository;

impl VaultRepository {
    /// Create a schedule block.
    #[allow(clippy::too_many_arguments)]
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
    ///
    /// Note: `note_id` is always updated (can be set to None to clear the link).
    /// Other fields like `date`, `start_time`, `end_time` are only updated if Some.
    #[allow(clippy::too_many_arguments)]
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
        // note_id is always included (can be set to NULL to clear the link)
        let mut updates = vec!["note_id = ?"];
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
        q = q.bind(note_id);  // Always bind note_id (can be None/NULL)
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
}

/// Expand an RRULE to get occurrences within a date range.
pub(crate) fn expand_rrule(
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
