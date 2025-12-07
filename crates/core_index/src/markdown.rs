//! Markdown parsing using pulldown-cmark + regex for wikilinks/tags.

use once_cell::sync::Lazy;
use pulldown_cmark::{Event, HeadingLevel, Options, Parser, Tag, TagEnd};
use regex::Regex;
use tracing::{debug, instrument};

/// Regex for matching [[wikilinks]].
/// Matches [[link]] or [[link|display text]]
static WIKILINK_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\[\[([^\]|]+)(?:\|[^\]]+)?\]\]").unwrap());

/// Regex for matching #tags.
/// Matches #tag but not ## headings or # in URLs
/// Tags must start with a letter and can contain letters, numbers, underscores, hyphens, and slashes
static TAG_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(?:^|[^\w#])#([a-zA-Z][a-zA-Z0-9_\-/]*)").unwrap());

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
}

/// A todo item found in the document.
#[derive(Debug, Clone)]
pub struct ParsedTodo {
    /// The todo description text.
    pub description: String,

    /// Whether the todo is completed.
    pub completed: bool,

    /// Line number where the todo appears (1-indexed).
    pub line_number: usize,

    /// The heading path (e.g., "Plan > Sub-section").
    pub heading_path: Option<String>,
}

/// Parse a markdown document and extract structured data.
#[instrument(skip(content))]
pub fn parse(content: &str) -> NoteAnalysis {
    let mut analysis = NoteAnalysis::default();

    // Track line numbers
    let line_offsets = compute_line_offsets(content);

    // Track current heading stack for heading_path
    let mut heading_stack: Vec<(u8, String)> = Vec::new();

    // Parse with pulldown-cmark
    let options = Options::ENABLE_TASKLISTS | Options::ENABLE_STRIKETHROUGH;
    let parser = Parser::new_ext(content, options);

    let mut current_heading_level: Option<u8> = None;
    let mut current_heading_text = String::new();
    let mut in_task_item = false;
    let mut task_completed = false;
    let mut task_text = String::new();
    let mut current_offset: usize = 0;

    for (event, range) in parser.into_offset_iter() {
        current_offset = range.start;

        match event {
            Event::Start(Tag::Heading { level, .. }) => {
                current_heading_level = Some(heading_level_to_u8(level));
                current_heading_text.clear();
            }

            Event::End(TagEnd::Heading(_)) => {
                if let Some(level) = current_heading_level.take() {
                    let text = current_heading_text.trim().to_string();
                    let line_number = offset_to_line(&line_offsets, current_offset);

                    // Set title from first H1
                    if level == 1 && analysis.title.is_none() {
                        analysis.title = Some(text.clone());
                    }

                    // Update heading stack
                    while heading_stack.last().map(|(l, _)| *l >= level).unwrap_or(false) {
                        heading_stack.pop();
                    }
                    heading_stack.push((level, text.clone()));

                    analysis.headings.push(ParsedHeading {
                        level,
                        text,
                        line_number,
                    });
                }
            }

            Event::Start(Tag::List(_)) => {}

            Event::Start(Tag::Item) => {}

            Event::End(TagEnd::Item) => {
                if in_task_item {
                    let description = task_text.trim().to_string();
                    let line_number = offset_to_line(&line_offsets, current_offset);
                    let heading_path = build_heading_path(&heading_stack);

                    analysis.todos.push(ParsedTodo {
                        description,
                        completed: task_completed,
                        line_number,
                        heading_path,
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

    // Extract wikilinks and tags using regex
    analysis.links = extract_wikilinks(content);
    analysis.tags = extract_tags(content);

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
}
