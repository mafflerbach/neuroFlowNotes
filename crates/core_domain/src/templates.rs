//! Template rendering for daily notes and other templated content.

use chrono::{Datelike, Local, NaiveDate};
use std::collections::HashMap;

/// Context for template rendering.
#[derive(Debug, Clone)]
pub struct TemplateContext {
    /// The date for the template (defaults to today).
    pub date: NaiveDate,
    /// Additional custom variables.
    pub custom: HashMap<String, String>,
}

impl Default for TemplateContext {
    fn default() -> Self {
        Self {
            date: Local::now().date_naive(),
            custom: HashMap::new(),
        }
    }
}

impl TemplateContext {
    /// Create a context for a specific date.
    pub fn for_date(date: NaiveDate) -> Self {
        Self {
            date,
            custom: HashMap::new(),
        }
    }

    /// Add a custom variable.
    pub fn with_var(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.custom.insert(key.into(), value.into());
        self
    }
}

/// Render a template string with the given context.
///
/// Supported variables:
/// - `{{date}}` - The date in YYYY-MM-DD format
/// - `{{weekday}}` - The day of the week (Monday, Tuesday, etc.)
/// - `{{week}}` - The ISO week number
/// - `{{year}}` - The year
/// - `{{month}}` - The month number (01-12)
/// - `{{day}}` - The day of month (01-31)
/// - `{{month_name}}` - The month name (January, February, etc.)
/// - Any custom variables from the context
pub fn render_template(input: &str, ctx: &TemplateContext) -> String {
    let weekday_names = [
        "Monday",
        "Tuesday",
        "Wednesday",
        "Thursday",
        "Friday",
        "Saturday",
        "Sunday",
    ];

    let month_names = [
        "January",
        "February",
        "March",
        "April",
        "May",
        "June",
        "July",
        "August",
        "September",
        "October",
        "November",
        "December",
    ];

    let weekday = weekday_names[ctx.date.weekday().num_days_from_monday() as usize];
    let month_name = month_names[ctx.date.month0() as usize];
    let week = ctx.date.iso_week().week();

    let mut result = input.to_string();

    // Built-in variables
    result = result.replace("{{date}}", &ctx.date.format("%Y-%m-%d").to_string());
    result = result.replace("{{weekday}}", weekday);
    result = result.replace("{{week}}", &format!("{:02}", week));
    result = result.replace("{{year}}", &ctx.date.year().to_string());
    result = result.replace("{{month}}", &format!("{:02}", ctx.date.month()));
    result = result.replace("{{day}}", &format!("{:02}", ctx.date.day()));
    result = result.replace("{{month_name}}", month_name);

    // Custom variables
    for (key, value) in &ctx.custom {
        let pattern = format!("{{{{{}}}}}", key);
        result = result.replace(&pattern, value);
    }

    result
}

/// Get the daily note path for a date.
pub fn daily_note_path(date: NaiveDate, folder: &str) -> String {
    let filename = date.format("%Y-%m-%d.md").to_string();
    if folder.is_empty() {
        filename
    } else {
        format!("{}/{}", folder.trim_end_matches('/'), filename)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_template() {
        let ctx = TemplateContext::for_date(NaiveDate::from_ymd_opt(2025, 12, 7).unwrap());
        let template = "# {{date}} – {{weekday}}\n\nWeek {{week}} of {{year}}";
        let result = render_template(template, &ctx);

        assert!(result.contains("# 2025-12-07 – Sunday"));
        assert!(result.contains("Week 49 of 2025"));
    }

    #[test]
    fn test_custom_variables() {
        let ctx = TemplateContext::default().with_var("author", "John");
        let template = "Created by {{author}}";
        let result = render_template(template, &ctx);

        assert_eq!(result, "Created by John");
    }

    #[test]
    fn test_daily_note_path() {
        let date = NaiveDate::from_ymd_opt(2025, 12, 7).unwrap();

        assert_eq!(daily_note_path(date, "daily"), "daily/2025-12-07.md");
        assert_eq!(daily_note_path(date, "daily/"), "daily/2025-12-07.md");
        assert_eq!(daily_note_path(date, ""), "2025-12-07.md");
    }
}
