//! YAML frontmatter parsing for Obsidian-style notes.
//!
//! Parses frontmatter delimited by `---` at the start of a file.
//! Supports standard Obsidian properties like tags, aliases, etc.

use serde_yaml::Value;
use std::collections::HashMap;
use tracing::debug;

/// Parsed frontmatter from a markdown file.
#[derive(Debug, Clone, Default)]
pub struct Frontmatter {
    /// Raw key-value properties from frontmatter.
    pub properties: HashMap<String, PropertyValue>,
    /// Tags extracted from frontmatter (without # prefix).
    pub tags: Vec<String>,
    /// Aliases for the note.
    pub aliases: Vec<String>,
    /// The byte offset where content starts (after frontmatter).
    pub content_start: usize,
}

/// A property value that can be a string, list, or other type.
#[derive(Debug, Clone)]
pub enum PropertyValue {
    String(String),
    List(Vec<String>),
    Number(f64),
    Bool(bool),
    Null,
}

impl PropertyValue {
    /// Convert to a string representation for storage.
    pub fn to_string_value(&self) -> Option<String> {
        match self {
            PropertyValue::String(s) => Some(s.clone()),
            PropertyValue::Number(n) => Some(n.to_string()),
            PropertyValue::Bool(b) => Some(b.to_string()),
            PropertyValue::List(items) => Some(items.join(", ")),
            PropertyValue::Null => None,
        }
    }

    /// Get as list of strings (for tags, aliases).
    pub fn as_list(&self) -> Vec<String> {
        match self {
            PropertyValue::List(items) => items.clone(),
            PropertyValue::String(s) => vec![s.clone()],
            _ => vec![],
        }
    }
}

/// Parse YAML frontmatter from markdown content.
///
/// Returns the parsed frontmatter and the remaining content.
pub fn parse_frontmatter(content: &str) -> (Frontmatter, &str) {
    let mut frontmatter = Frontmatter::default();

    // Check for frontmatter delimiter at start
    if !content.starts_with("---") {
        return (frontmatter, content);
    }

    // Find the closing delimiter
    let after_opening = &content[3..];
    let closing_pos = after_opening.find("\n---");

    let Some(closing_pos) = closing_pos else {
        // No closing delimiter, treat as no frontmatter
        return (frontmatter, content);
    };

    // Extract YAML content (skip the opening newline if present)
    let yaml_start = if after_opening.starts_with('\n') { 1 } else { 0 };
    let yaml_content = &after_opening[yaml_start..closing_pos];

    // Calculate content start (after closing --- and newline)
    let content_start = 3 + closing_pos + 4; // opening --- + yaml + \n---
    let content_start = if content.len() > content_start && content.as_bytes()[content_start] == b'\n' {
        content_start + 1
    } else {
        content_start
    };

    frontmatter.content_start = content_start;

    // Parse YAML
    match serde_yaml::from_str::<Value>(yaml_content) {
        Ok(Value::Mapping(map)) => {
            for (key, value) in map {
                if let Value::String(key_str) = key {
                    let prop_value = yaml_value_to_property(&value);

                    // Handle special keys
                    match key_str.to_lowercase().as_str() {
                        "tags" | "tag" => {
                            frontmatter.tags = extract_tags_from_value(&value);
                        }
                        "aliases" | "alias" => {
                            frontmatter.aliases = prop_value.as_list();
                        }
                        _ => {}
                    }

                    // Store all properties
                    frontmatter.properties.insert(key_str, prop_value);
                }
            }
        }
        Ok(_) => {
            debug!("Frontmatter is not a YAML mapping");
        }
        Err(e) => {
            debug!("Failed to parse frontmatter YAML: {}", e);
        }
    }

    // Return content after frontmatter
    let remaining_content = if content_start < content.len() {
        &content[content_start..]
    } else {
        ""
    };

    (frontmatter, remaining_content)
}

/// Extract tags from a YAML value.
///
/// Handles both list format and single string format.
/// Also handles Obsidian's nested tags format.
fn extract_tags_from_value(value: &Value) -> Vec<String> {
    let mut tags = Vec::new();

    match value {
        Value::Sequence(seq) => {
            for item in seq {
                if let Value::String(s) = item {
                    // Remove # prefix if present
                    let tag = s.trim_start_matches('#').to_string();
                    if !tag.is_empty() {
                        tags.push(tag);
                    }
                }
            }
        }
        Value::String(s) => {
            // Could be comma-separated or space-separated tags
            for tag in s.split([',', ' ']) {
                let tag = tag.trim().trim_start_matches('#');
                if !tag.is_empty() {
                    tags.push(tag.to_string());
                }
            }
        }
        _ => {}
    }

    tags
}

/// Convert a YAML value to a PropertyValue.
fn yaml_value_to_property(value: &Value) -> PropertyValue {
    match value {
        Value::String(s) => PropertyValue::String(s.clone()),
        Value::Number(n) => PropertyValue::Number(n.as_f64().unwrap_or(0.0)),
        Value::Bool(b) => PropertyValue::Bool(*b),
        Value::Null => PropertyValue::Null,
        Value::Sequence(seq) => {
            let items: Vec<String> = seq
                .iter()
                .filter_map(|v| match v {
                    Value::String(s) => Some(s.clone()),
                    Value::Number(n) => Some(n.to_string()),
                    Value::Bool(b) => Some(b.to_string()),
                    _ => None,
                })
                .collect();
            PropertyValue::List(items)
        }
        Value::Mapping(_) => {
            // For nested objects, serialize to JSON string
            PropertyValue::String(serde_yaml::to_string(value).unwrap_or_default())
        }
        Value::Tagged(_) => PropertyValue::Null,
    }
}

/// Strip frontmatter from content and return just the body.
pub fn strip_frontmatter(content: &str) -> &str {
    let (fm, body) = parse_frontmatter(content);
    if fm.content_start > 0 {
        body
    } else {
        content
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_frontmatter() {
        let content = r#"---
title: My Note
status: draft
---

# My Note

Some content here.
"#;
        let (fm, body) = parse_frontmatter(content);

        assert!(fm.properties.contains_key("title"));
        assert!(fm.properties.contains_key("status"));

        if let PropertyValue::String(title) = &fm.properties["title"] {
            assert_eq!(title, "My Note");
        } else {
            panic!("Expected string for title");
        }

        assert!(body.starts_with("\n# My Note"));
    }

    #[test]
    fn test_parse_tags_list() {
        let content = r#"---
tags:
  - rust
  - programming
  - notes
---
Content
"#;
        let (fm, _) = parse_frontmatter(content);

        assert_eq!(fm.tags, vec!["rust", "programming", "notes"]);
    }

    #[test]
    fn test_parse_tags_with_hash() {
        let content = "---\ntags:\n  - \"#rust\"\n  - \"#programming\"\n---\nContent\n";
        let (fm, _) = parse_frontmatter(content);

        // Tags should have # stripped
        assert_eq!(fm.tags, vec!["rust", "programming"]);
    }

    #[test]
    fn test_parse_aliases() {
        let content = r#"---
aliases:
  - My Alias
  - Another Name
---
Content
"#;
        let (fm, _) = parse_frontmatter(content);

        assert_eq!(fm.aliases, vec!["My Alias", "Another Name"]);
    }

    #[test]
    fn test_no_frontmatter() {
        let content = "# Just a heading\n\nNo frontmatter here.";
        let (fm, body) = parse_frontmatter(content);

        assert!(fm.properties.is_empty());
        assert_eq!(body, content);
        assert_eq!(fm.content_start, 0);
    }

    #[test]
    fn test_unclosed_frontmatter() {
        let content = "---\ntitle: Broken\nNo closing delimiter";
        let (fm, body) = parse_frontmatter(content);

        // Should treat as no frontmatter
        assert!(fm.properties.is_empty());
        assert_eq!(body, content);
    }

    #[test]
    fn test_numeric_and_bool_properties() {
        let content = r#"---
count: 42
rating: 4.5
published: true
draft: false
---
Content
"#;
        let (fm, _) = parse_frontmatter(content);

        if let PropertyValue::Number(n) = &fm.properties["count"] {
            assert_eq!(*n, 42.0);
        } else {
            panic!("Expected number for count");
        }

        if let PropertyValue::Bool(b) = &fm.properties["published"] {
            assert!(*b);
        } else {
            panic!("Expected bool for published");
        }
    }

    #[test]
    fn test_strip_frontmatter() {
        let content = r#"---
title: Test
---

# Content"#;
        let body = strip_frontmatter(content);
        assert!(body.contains("# Content"));
        assert!(!body.contains("---"));
    }
}
