//! Markdown parsing and analysis for NeuroFlow Notes.
//!
//! This crate extracts structured data from markdown files:
//! - Headings (for heading_path and title extraction)
//! - Todos (task list items)
//! - Wikilinks ([[link]])
//! - Tags (#tag)
//! - YAML frontmatter

pub mod frontmatter;
pub mod markdown;

pub use frontmatter::{parse_frontmatter, strip_frontmatter, Frontmatter, PropertyValue};
pub use markdown::{NoteAnalysis, ParsedHeading, ParsedTodo};
