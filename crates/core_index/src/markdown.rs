//! Markdown parsing using pulldown-cmark + regex for wikilinks/tags.

use once_cell::sync::Lazy;
use pulldown_cmark::{Event, HeadingLevel, Options, Parser, Tag, TagEnd};
use regex::Regex;
use tracing::{debug, instrument};

use crate::frontmatter::{parse_frontmatter, PropertyValue};

/// Regex for matching [[wikilinks]].
/// Matches [[link]], [[link|display text]], [[link#section]], [[link#section|display]]
/// Also matches embeds: ![[link]], ![[link#section]]
static WIKILINK_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"!?\[\[([^\]#|]+)(?:#[^\]|]+)?(?:\|[^\]]+)?\]\]").unwrap());

/// Regex for matching wikilinks with section anchors.
/// Captures: 1=target, 2=section (optional), 3=display (optional)
static WIKILINK_FULL_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(!?)\[\[([^\]#|]+)(?:#([^\]|]+))?(?:\|([^\]]+))?\]\]").unwrap());

/// Regex for matching #tags.
/// Matches #tag but not ## headings or # in URLs
/// Tags must start with a letter and can contain letters, numbers, underscores, hyphens, and slashes
static TAG_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(?:^|[^\w#])#([a-zA-Z][a-zA-Z0-9_\-/]*)").unwrap());

/// Regex for matching @context annotations in tasks.
/// Matches @word (e.g., @home, @work, @phone, @computer, @errands)
static CONTEXT_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"@([a-zA-Z][a-zA-Z0-9_\-]*)").unwrap());

/// Regex for matching !priority annotations in tasks.
/// Matches !high, !medium, !low (or !h, !m, !l)
static PRIORITY_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"!(high|medium|low|h|m|l)\b").unwrap());

/// Regex for matching ^due-date annotations in tasks.
/// Matches ^YYYY-MM-DD or relative dates like ^today, ^tomorrow, ^monday, ^next-week
static DUE_DATE_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\^(\d{4}-\d{2}-\d{2}|today|tomorrow|monday|tuesday|wednesday|thursday|friday|saturday|sunday|next-week)").unwrap());

/// A parsed property from frontmatter.
#[derive(Debug, Clone)]
pub struct ParsedProperty {
    /// Property key.
    pub key: String,
    /// Property value as string.
    pub value: Option<String>,
    /// Inferred property type.
    pub property_type: String,
}

/// Result of analyzing a markdown note.
#[derive(Debug, Clone, Default)]
pub struct NoteAnalysis {
    /// The note's title (first H1 heading, or None).
    pub title: Option<String>,

    /// All headings in the document.
    pub headings: Vec<ParsedHeading>,

    /// All todo items found.
    pub todos: Vec<ParsedTodo>,

    /// All tags found (without the # prefix).
    pub tags: Vec<String>,

    /// All wikilinks found (target note names).
    pub links: Vec<String>,

    /// Properties from YAML frontmatter.
    pub properties: Vec<ParsedProperty>,
}

/// A heading in the document.
#[derive(Debug, Clone)]
pub struct ParsedHeading {
    /// Heading level (1-6).
    pub level: u8,

    /// Heading text.
    pub text: String,

    /// Line number where the heading starts (1-indexed).
    pub line_number: usize,

    /// Byte offset where the heading content starts (after the heading line).
    pub content_start: usize,

    /// Byte offset where the heading content ends (before next heading or EOF).
    pub content_end: usize,

    /// URL-safe slug generated from the heading text.
    pub slug: String,
}

/// A todo item found in the document.
#[derive(Debug, Clone)]
pub struct ParsedTodo {
    /// The todo description text (with annotations stripped).
    pub description: String,

    /// The raw text as written in markdown (with annotations).
    pub raw_text: String,

    /// Whether the todo is completed.
    pub completed: bool,

    /// Line number where the todo appears (1-indexed).
    pub line_number: usize,

    /// The heading path (e.g., "Plan > Sub-section").
    pub heading_path: Option<String>,

    /// GTD context (e.g., "home", "work", "phone").
    pub context: Option<String>,

    /// Priority level ("high", "medium", "low").
    pub priority: Option<String>,

    /// Due date as YYYY-MM-DD string.
    pub due_date: Option<String>,
}

/// Parse a markdown document and extract structured data.
#[instrument(skip(content))]
pub fn parse(content: &str) -> NoteAnalysis {
    let mut analysis = NoteAnalysis::default();

    // Parse frontmatter first
    let (frontmatter, body) = parse_frontmatter(content);

    // Extract properties from frontmatter
    for (key, value) in &frontmatter.properties {
        // Skip special keys that are handled separately
        if key.to_lowercase() == "tags" || key.to_lowercase() == "tag" {
            continue;
        }

        let (string_value, prop_type) = match value {
            PropertyValue::String(s) => (Some(s.clone()), "text"),
            PropertyValue::Number(n) => (Some(n.to_string()), "number"),
            PropertyValue::Bool(b) => (Some(b.to_string()), "boolean"),
            PropertyValue::List(items) => (Some(items.join(", ")), "list"),
            PropertyValue::Null => (None, "text"),
        };

        // Try to detect date type from value
        let detected_type = if prop_type == "text" {
            if let Some(ref v) = string_value {
                if v.len() == 10 && v.chars().nth(4) == Some('-') && v.chars().nth(7) == Some('-') {
                    "date"
                } else {
                    prop_type
                }
            } else {
                prop_type
            }
        } else {
            prop_type
        };

        analysis.properties.push(ParsedProperty {
            key: key.clone(),
            value: string_value,
            property_type: detected_type.to_string(),
        });
    }

    // Add frontmatter tags to analysis tags
    for tag in &frontmatter.tags {
        if !analysis.tags.contains(tag) {
            analysis.tags.push(tag.clone());
        }
    }

    // Use body content for further parsing (after frontmatter)
    let content_to_parse = if frontmatter.content_start > 0 { body } else { content };
    let content_len = content_to_parse.len();

    // Track line numbers
    let line_offsets = compute_line_offsets(content_to_parse);

    // Track current heading stack for heading_path
    let mut heading_stack: Vec<(u8, String)> = Vec::new();

    // Temporary storage for heading data before computing content boundaries
    struct TempHeading {
        level: u8,
        text: String,
        line_number: usize,
        heading_start_offset: usize,  // byte offset where the heading line starts (## chars)
        heading_end_offset: usize,    // byte offset after the heading line (after newline)
    }
    let mut temp_headings: Vec<TempHeading> = Vec::new();

    // Parse with pulldown-cmark
    let options = Options::ENABLE_TASKLISTS | Options::ENABLE_STRIKETHROUGH;
    let parser = Parser::new_ext(content_to_parse, options);

    let mut current_heading_level: Option<u8> = None;
    let mut current_heading_text = String::new();
    let mut current_heading_start: usize = 0;
    let mut in_task_item = false;
    let mut task_completed = false;
    let mut task_text = String::new();
    let mut current_offset: usize = 0;

    for (event, range) in parser.into_offset_iter() {
        // Track offset for todo line number computation
        if matches!(event, Event::End(TagEnd::Item)) {
            current_offset = range.start;
        }

        match event {
            Event::Start(Tag::Heading { level, .. }) => {
                current_heading_level = Some(heading_level_to_u8(level));
                current_heading_text.clear();
                current_heading_start = range.start;
            }

            Event::End(TagEnd::Heading(_)) => {
                if let Some(level) = current_heading_level.take() {
                    let text = current_heading_text.trim().to_string();
                    let line_number = offset_to_line(&line_offsets, current_heading_start);

                    // Set title from first H1
                    if level == 1 && analysis.title.is_none() {
                        analysis.title = Some(text.clone());
                    }

                    // Update heading stack
                    while heading_stack.last().map(|(l, _)| *l >= level).unwrap_or(false) {
                        heading_stack.pop();
                    }
                    heading_stack.push((level, text.clone()));

                    // Find where the heading line ends (after newline)
                    let heading_end_offset = content_to_parse[range.end..]
                        .find('\n')
                        .map(|i| range.end + i + 1)
                        .unwrap_or(range.end);

                    temp_headings.push(TempHeading {
                        level,
                        text,
                        line_number,
                        heading_start_offset: current_heading_start,
                        heading_end_offset,
                    });
                }
            }

            Event::Start(Tag::List(_)) => {}

            Event::Start(Tag::Item) => {}

            Event::End(TagEnd::Item) => {
                if in_task_item {
                    let raw_text = task_text.trim().to_string();
                    let line_number = offset_to_line(&line_offsets, current_offset);
                    let heading_path = build_heading_path(&heading_stack);

                    // Extract GTD annotations
                    let (description, context, priority, due_date) = parse_todo_annotations(&raw_text);

                    analysis.todos.push(ParsedTodo {
                        description,
                        raw_text,
                        completed: task_completed,
                        line_number,
                        heading_path,
                        context,
                        priority,
                        due_date,
                    });

                    in_task_item = false;
                    task_text.clear();
                }
            }

            Event::TaskListMarker(completed) => {
                in_task_item = true;
                task_completed = completed;
                task_text.clear();
            }

            Event::Text(text) => {
                if current_heading_level.is_some() {
                    current_heading_text.push_str(&text);
                }
                if in_task_item {
                    task_text.push_str(&text);
                }
            }

            Event::Code(code) => {
                if current_heading_level.is_some() {
                    current_heading_text.push_str(&code);
                }
                if in_task_item {
                    task_text.push_str(&code);
                }
            }

            _ => {}
        }
    }

    // Convert temp_headings to ParsedHeading with computed content boundaries
    for (i, th) in temp_headings.iter().enumerate() {
        // content_start is right after the heading line
        let content_start = th.heading_end_offset;

        // content_end is the byte offset where the next heading of same or higher level starts,
        // or EOF if no such heading exists.
        // For nested headings (e.g., H3 under H2), the H2's content_end includes all H3 content.
        // content_end is where the next heading of same or higher level starts
        let content_end = temp_headings[i + 1..]
            .iter()
            .find(|next| next.level <= th.level)
            .map(|next| next.heading_start_offset)
            .unwrap_or(content_len);

        analysis.headings.push(ParsedHeading {
            level: th.level,
            text: th.text.clone(),
            line_number: th.line_number,
            content_start,
            content_end,
            slug: slugify(&th.text),
        });
    }

    // Extract wikilinks and tags using regex (from body, not frontmatter)
    analysis.links = extract_wikilinks(content_to_parse);
    // Merge inline tags with frontmatter tags
    let inline_tags = extract_tags(content_to_parse);
    for tag in inline_tags {
        if !analysis.tags.contains(&tag) {
            analysis.tags.push(tag);
        }
    }

    debug!(
        "Parsed note: {} headings, {} todos, {} links, {} tags",
        analysis.headings.len(),
        analysis.todos.len(),
        analysis.links.len(),
        analysis.tags.len()
    );

    analysis
}

/// Extract wikilinks from content.
fn extract_wikilinks(content: &str) -> Vec<String> {
    WIKILINK_REGEX
        .captures_iter(content)
        .map(|cap| cap[1].to_string())
        .collect()
}

/// Extract tags from content.
fn extract_tags(content: &str) -> Vec<String> {
    let mut tags: Vec<String> = TAG_REGEX
        .captures_iter(content)
        .map(|cap| cap[1].to_string())
        .collect();

    // Deduplicate while preserving order
    let mut seen = std::collections::HashSet::new();
    tags.retain(|tag| seen.insert(tag.clone()));

    tags
}

/// Parse GTD annotations from a todo text.
///
/// Extracts @context, !priority, and ^due-date from the text.
/// Returns (clean_description, context, priority, due_date).
fn parse_todo_annotations(text: &str) -> (String, Option<String>, Option<String>, Option<String>) {
    // Extract context (@word)
    let context = CONTEXT_REGEX
        .captures(text)
        .map(|cap| cap[1].to_string());

    // Extract priority (!high, !medium, !low, !h, !m, !l)
    let priority = PRIORITY_REGEX
        .captures(text)
        .map(|cap| {
            // Normalize shorthand to full form
            match &cap[1] {
                "h" => "high".to_string(),
                "m" => "medium".to_string(),
                "l" => "low".to_string(),
                other => other.to_string(),
            }
        });

    // Extract due date (^YYYY-MM-DD or relative)
    let due_date = DUE_DATE_REGEX
        .captures(text)
        .map(|cap| {
            let date_str = &cap[1];
            // Convert relative dates to absolute
            resolve_relative_date(date_str)
        });

    // Create clean description by removing annotations
    let clean = CONTEXT_REGEX.replace_all(text, "");
    let clean = PRIORITY_REGEX.replace_all(&clean, "");
    let clean = DUE_DATE_REGEX.replace_all(&clean, "");
    // Clean up extra whitespace
    let description = clean
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");

    (description, context, priority, due_date)
}

/// Resolve relative date strings to YYYY-MM-DD format.
fn resolve_relative_date(date_str: &str) -> String {
    use chrono::{Datelike, Local, Weekday};

    let today = Local::now().date_naive();

    match date_str.to_lowercase().as_str() {
        "today" => today.format("%Y-%m-%d").to_string(),
        "tomorrow" => (today + chrono::Duration::days(1)).format("%Y-%m-%d").to_string(),
        "next-week" => (today + chrono::Duration::days(7)).format("%Y-%m-%d").to_string(),
        // Handle day names (find next occurrence)
        day_name => {
            let target_weekday = match day_name {
                "monday" => Some(Weekday::Mon),
                "tuesday" => Some(Weekday::Tue),
                "wednesday" => Some(Weekday::Wed),
                "thursday" => Some(Weekday::Thu),
                "friday" => Some(Weekday::Fri),
                "saturday" => Some(Weekday::Sat),
                "sunday" => Some(Weekday::Sun),
                _ => None, // Already a date string like 2024-12-15
            };

            if let Some(target) = target_weekday {
                let current_weekday = today.weekday();
                let days_until = (target.num_days_from_monday() as i64
                    - current_weekday.num_days_from_monday() as i64
                    + 7) % 7;
                // If it's today, go to next week
                let days_until = if days_until == 0 { 7 } else { days_until };
                (today + chrono::Duration::days(days_until)).format("%Y-%m-%d").to_string()
            } else {
                // Already an absolute date
                date_str.to_string()
            }
        }
    }
}

/// Compute byte offsets for each line start.
fn compute_line_offsets(content: &str) -> Vec<usize> {
    let mut offsets = vec![0];
    for (i, c) in content.char_indices() {
        if c == '\n' {
            offsets.push(i + 1);
        }
    }
    offsets
}

/// Convert a byte offset to a line number (1-indexed).
fn offset_to_line(line_offsets: &[usize], offset: usize) -> usize {
    match line_offsets.binary_search(&offset) {
        Ok(line) => line + 1,
        Err(line) => line,
    }
}

/// Build a heading path string from the heading stack.
fn build_heading_path(stack: &[(u8, String)]) -> Option<String> {
    if stack.is_empty() {
        None
    } else {
        Some(stack.iter().map(|(_, text)| text.as_str()).collect::<Vec<_>>().join(" > "))
    }
}

/// Convert pulldown-cmark HeadingLevel to u8.
fn heading_level_to_u8(level: HeadingLevel) -> u8 {
    match level {
        HeadingLevel::H1 => 1,
        HeadingLevel::H2 => 2,
        HeadingLevel::H3 => 3,
        HeadingLevel::H4 => 4,
        HeadingLevel::H5 => 5,
        HeadingLevel::H6 => 6,
    }
}

/// Generate a URL-safe slug from heading text.
///
/// Converts "My Heading Text" to "my-heading-text".
/// Handles special characters, emojis, and unicode.
pub fn slugify(text: &str) -> String {
    text.to_lowercase()
        .chars()
        .map(|c| {
            if c.is_alphanumeric() {
                c
            } else if c.is_whitespace() || c == '-' || c == '_' {
                '-'
            } else {
                // Skip other characters (punctuation, emojis, etc.)
                '\0'
            }
        })
        .filter(|&c| c != '\0')
        .collect::<String>()
        // Collapse multiple dashes
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

/// Extract a section's content by its slug.
///
/// Returns the content from the heading to the next heading of same or higher level,
/// or to EOF if no such heading exists.
pub fn extract_section(content: &str, section_slug: &str) -> Option<String> {
    // Parse frontmatter to get the body offset
    let (frontmatter, body) = parse_frontmatter(content);
    let analysis = parse(content);

    // Find the heading with matching slug
    let heading_idx = analysis.headings.iter().position(|h| h.slug == section_slug)?;
    let heading = &analysis.headings[heading_idx];

    // Heading offsets are relative to body (after frontmatter), so slice from body
    let content_to_slice = if frontmatter.content_start > 0 { body } else { content };
    let section_content = &content_to_slice[heading.content_start..heading.content_end];

    Some(section_content.to_string())
}

/// Extract section content including the heading itself.
pub fn extract_section_with_heading(content: &str, section_slug: &str) -> Option<String> {
    // Parse frontmatter to get the body offset
    let (frontmatter, body) = parse_frontmatter(content);
    let analysis = parse(content);

    // Find the heading with matching slug
    let heading_idx = analysis.headings.iter().position(|h| h.slug == section_slug)?;
    let heading = &analysis.headings[heading_idx];

    // Heading offsets and line numbers are relative to body (after frontmatter)
    let content_to_slice = if frontmatter.content_start > 0 { body } else { content };

    // Find the start of the heading line
    // content_start points to the line after the heading
    // We need to find the line number's starting position
    let heading_line_start = if heading.line_number == 1 {
        0
    } else {
        // Find the nth newline to get to the start of line_number
        let mut newline_count = 0;
        let mut pos = 0;
        for (i, c) in content_to_slice.char_indices() {
            if c == '\n' {
                newline_count += 1;
                if newline_count == heading.line_number - 1 {
                    pos = i + 1;
                    break;
                }
            }
        }
        pos
    };

    // Extract from heading start to content end
    let section_content = &content_to_slice[heading_line_start..heading.content_end];

    Some(section_content.to_string())
}

/// Update wiki links in content when a note is renamed.
///
/// Handles all forms: [[old]], [[old|alias]], [[old#section]], [[old#section|alias]], ![[old]]
pub fn update_wiki_links(content: &str, old_name: &str, new_name: &str) -> String {
    WIKILINK_FULL_REGEX.replace_all(content, |caps: &regex::Captures| {
        let embed_prefix = &caps[1]; // "!" or ""
        let target = &caps[2];
        let section = caps.get(3).map(|m| m.as_str());
        let display = caps.get(4).map(|m| m.as_str());

        // Check if target matches old name (case-insensitive for flexibility)
        let target_normalized = target.trim();
        let old_normalized = old_name.trim();

        if target_normalized.eq_ignore_ascii_case(old_normalized) {
            // Rebuild the link with new name
            let mut result = format!("{}[[{}", embed_prefix, new_name);
            if let Some(sec) = section {
                result.push('#');
                result.push_str(sec);
            }
            if let Some(disp) = display {
                result.push('|');
                result.push_str(disp);
            }
            result.push_str("]]");
            result
        } else {
            // No change
            caps[0].to_string()
        }
    }).to_string()
}

/// Toggle a todo's completion status and return the modified content.
///
/// This function finds the todo at the given line and toggles its checkbox.
pub fn toggle_todo(content: &str, line_number: usize, completed: bool) -> String {
    let lines: Vec<&str> = content.lines().collect();
    let mut result = Vec::with_capacity(lines.len());

    for (i, line) in lines.iter().enumerate() {
        let current_line = i + 1; // 1-indexed

        if current_line == line_number {
            // Toggle the checkbox on this line
            let new_line = if completed {
                // Change - [ ] to - [x]
                line.replacen("- [ ]", "- [x]", 1)
                    .replacen("* [ ]", "* [x]", 1)
            } else {
                // Change - [x] to - [ ]
                line.replacen("- [x]", "- [ ]", 1)
                    .replacen("- [X]", "- [ ]", 1)
                    .replacen("* [x]", "* [ ]", 1)
                    .replacen("* [X]", "* [ ]", 1)
            };
            result.push(new_line);
        } else {
            result.push((*line).to_string());
        }
    }

    // Preserve trailing newline if original had one
    let mut output = result.join("\n");
    if content.ends_with('\n') {
        output.push('\n');
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_headings() {
        let content = "# Title\n\nSome text\n\n## Section 1\n\n### Subsection\n";
        let analysis = parse(content);

        assert_eq!(analysis.title, Some("Title".to_string()));
        assert_eq!(analysis.headings.len(), 3);
        assert_eq!(analysis.headings[0].level, 1);
        assert_eq!(analysis.headings[1].level, 2);
        assert_eq!(analysis.headings[2].level, 3);
    }

    #[test]
    fn test_parse_todos() {
        let content = "# Tasks\n\n- [ ] Do something\n- [x] Done task\n- Regular item\n";
        let analysis = parse(content);

        assert_eq!(analysis.todos.len(), 2);
        assert!(!analysis.todos[0].completed);
        assert_eq!(analysis.todos[0].description, "Do something");
        assert!(analysis.todos[1].completed);
        assert_eq!(analysis.todos[1].description, "Done task");
    }

    #[test]
    fn test_parse_wikilinks() {
        let content = "Check [[other note]] and [[project/sub|display text]].\n";
        let analysis = parse(content);

        assert_eq!(analysis.links.len(), 2);
        assert_eq!(analysis.links[0], "other note");
        assert_eq!(analysis.links[1], "project/sub");
    }

    #[test]
    fn test_parse_tags() {
        let content = "This is #important and #work/project related.\n\n## Heading\n\nMore #important stuff.\n";
        let analysis = parse(content);

        // Should deduplicate
        assert_eq!(analysis.tags.len(), 2);
        assert!(analysis.tags.contains(&"important".to_string()));
        assert!(analysis.tags.contains(&"work/project".to_string()));
    }

    #[test]
    fn test_toggle_todo() {
        let content = "# Tasks\n\n- [ ] First\n- [ ] Second\n";

        let toggled = toggle_todo(content, 3, true);
        assert!(toggled.contains("- [x] First"));
        assert!(toggled.contains("- [ ] Second"));

        let untoggled = toggle_todo(&toggled, 3, false);
        assert!(untoggled.contains("- [ ] First"));
    }

    #[test]
    fn test_heading_path() {
        let content = "# Project\n\n## Tasks\n\n- [ ] Do thing\n\n### Subtasks\n\n- [ ] Sub thing\n";
        let analysis = parse(content);

        // First todo should have heading path "Project > Tasks"
        assert_eq!(
            analysis.todos[0].heading_path,
            Some("Project > Tasks".to_string())
        );
    }

    #[test]
    fn test_slugify() {
        assert_eq!(slugify("Hello World"), "hello-world");
        assert_eq!(slugify("My Section!"), "my-section");
        assert_eq!(slugify("Test   Multiple   Spaces"), "test-multiple-spaces");
        assert_eq!(slugify("With-Dashes-Already"), "with-dashes-already");
        assert_eq!(slugify("Numbers 123 Here"), "numbers-123-here");
        assert_eq!(slugify("UPPERCASE"), "uppercase");
        assert_eq!(slugify("  Leading and Trailing  "), "leading-and-trailing");
    }

    #[test]
    fn test_heading_slugs() {
        let content = "# Main Title\n\n## My Section\n\nSome content\n\n### Sub Section\n";
        let analysis = parse(content);

        assert_eq!(analysis.headings[0].slug, "main-title");
        assert_eq!(analysis.headings[1].slug, "my-section");
        assert_eq!(analysis.headings[2].slug, "sub-section");
    }

    #[test]
    fn test_extract_section() {
        let content = "# Title\n\nIntro text.\n\n## Section One\n\nSection one content.\n\n## Section Two\n\nSection two content.\n";

        let section = extract_section(content, "section-one");
        assert!(section.is_some());
        let section_text = section.unwrap();
        assert!(section_text.contains("Section one content"));
        assert!(!section_text.contains("Section two content"));
    }

    #[test]
    fn test_extract_section_with_heading() {
        let content = "# Title\n\n## My Section\n\nContent here.\n\n## Next Section\n";

        let section = extract_section_with_heading(content, "my-section");
        assert!(section.is_some());
        let section_text = section.unwrap();
        assert!(section_text.contains("## My Section"));
        assert!(section_text.contains("Content here"));
        // Should not include the next section
        assert!(!section_text.contains("## Next Section"));
    }

    #[test]
    fn test_update_wiki_links() {
        // Basic link
        let content = "See [[old note]] for details.";
        let updated = update_wiki_links(content, "old note", "new note");
        assert_eq!(updated, "See [[new note]] for details.");

        // Link with display text
        let content = "See [[old note|my link]] for details.";
        let updated = update_wiki_links(content, "old note", "new note");
        assert_eq!(updated, "See [[new note|my link]] for details.");

        // Link with section
        let content = "See [[old note#section]] for details.";
        let updated = update_wiki_links(content, "old note", "new note");
        assert_eq!(updated, "See [[new note#section]] for details.");

        // Embed
        let content = "![[old note]]";
        let updated = update_wiki_links(content, "old note", "new note");
        assert_eq!(updated, "![[new note]]");

        // Embed with section
        let content = "![[old note#heading]]";
        let updated = update_wiki_links(content, "old note", "new note");
        assert_eq!(updated, "![[new note#heading]]");

        // Multiple links
        let content = "See [[old note]] and [[old note#section]] and [[other]].";
        let updated = update_wiki_links(content, "old note", "new note");
        assert_eq!(updated, "See [[new note]] and [[new note#section]] and [[other]].");
    }

    #[test]
    fn test_wikilinks_with_sections() {
        let content = "Link to [[note#section]] and ![[embed#heading]].\n";
        let analysis = parse(content);

        // The links should capture just the note name, not the section
        assert!(analysis.links.contains(&"note".to_string()));
        assert!(analysis.links.contains(&"embed".to_string()));
    }

    #[test]
    fn test_parse_todo_annotations() {
        // Test with all annotations
        let (desc, ctx, pri, due) = parse_todo_annotations("Call mom @phone !high ^2024-12-15");
        assert_eq!(desc, "Call mom");
        assert_eq!(ctx, Some("phone".to_string()));
        assert_eq!(pri, Some("high".to_string()));
        assert_eq!(due, Some("2024-12-15".to_string()));

        // Test shorthand priority
        let (_, _, pri, _) = parse_todo_annotations("Task !h");
        assert_eq!(pri, Some("high".to_string()));

        let (_, _, pri, _) = parse_todo_annotations("Task !m");
        assert_eq!(pri, Some("medium".to_string()));

        let (_, _, pri, _) = parse_todo_annotations("Task !l");
        assert_eq!(pri, Some("low".to_string()));

        // Test context only
        let (desc, ctx, pri, due) = parse_todo_annotations("Fix bug @computer");
        assert_eq!(desc, "Fix bug");
        assert_eq!(ctx, Some("computer".to_string()));
        assert_eq!(pri, None);
        assert_eq!(due, None);

        // Test no annotations
        let (desc, ctx, pri, due) = parse_todo_annotations("Simple task");
        assert_eq!(desc, "Simple task");
        assert_eq!(ctx, None);
        assert_eq!(pri, None);
        assert_eq!(due, None);
    }

    #[test]
    fn test_parse_todos_with_gtd() {
        let content = "# Tasks\n\n- [ ] Call mom @phone !high ^2024-12-15\n- [ ] Buy groceries @errands\n- [x] Done task\n";
        let analysis = parse(content);

        assert_eq!(analysis.todos.len(), 3);

        // First todo with all GTD annotations
        assert_eq!(analysis.todos[0].description, "Call mom");
        assert_eq!(analysis.todos[0].raw_text, "Call mom @phone !high ^2024-12-15");
        assert_eq!(analysis.todos[0].context, Some("phone".to_string()));
        assert_eq!(analysis.todos[0].priority, Some("high".to_string()));
        assert_eq!(analysis.todos[0].due_date, Some("2024-12-15".to_string()));

        // Second todo with context only
        assert_eq!(analysis.todos[1].description, "Buy groceries");
        assert_eq!(analysis.todos[1].context, Some("errands".to_string()));
        assert_eq!(analysis.todos[1].priority, None);
        assert_eq!(analysis.todos[1].due_date, None);

        // Third todo - completed, no annotations
        assert_eq!(analysis.todos[2].description, "Done task");
        assert!(analysis.todos[2].completed);
        assert_eq!(analysis.todos[2].context, None);
    }

    #[test]
    fn test_relative_date_resolution() {
        // Test absolute date passes through
        assert_eq!(resolve_relative_date("2024-12-15"), "2024-12-15");

        // Relative dates resolve to real dates (we can only test format)
        let today = resolve_relative_date("today");
        assert!(today.len() == 10); // YYYY-MM-DD format
        assert!(today.starts_with("20")); // Starts with year

        let tomorrow = resolve_relative_date("tomorrow");
        assert!(tomorrow.len() == 10);

        let next_week = resolve_relative_date("next-week");
        assert!(next_week.len() == 10);

        let monday = resolve_relative_date("monday");
        assert!(monday.len() == 10);
    }

    #[test]
    fn test_extract_section_with_frontmatter() {
        let content = "---\ntitle: Test Note\ntags: [test]\n---\n\n# Title\n\nIntro text.\n\n## Section One\n\nSection one content.\n\n## Section Two\n\nSection two content.\n";

        // Test extract_section_with_heading with frontmatter
        let section = extract_section_with_heading(content, "section-one");
        assert!(section.is_some());
        let section_text = section.unwrap();
        assert!(section_text.contains("## Section One"), "Should contain heading: {}", section_text);
        assert!(section_text.contains("Section one content"), "Should contain content: {}", section_text);
        assert!(!section_text.contains("## Section Two"), "Should not contain next section: {}", section_text);
        assert!(!section_text.contains("title: Test Note"), "Should not contain frontmatter: {}", section_text);

        // Test extract_section (without heading) with frontmatter
        let section = extract_section(content, "section-one");
        assert!(section.is_some());
        let section_text = section.unwrap();
        assert!(section_text.contains("Section one content"), "Should contain content: {}", section_text);
        assert!(!section_text.contains("## Section One"), "Should not contain heading: {}", section_text);
        assert!(!section_text.contains("Section two content"), "Should not contain next section: {}", section_text);
    }
}
