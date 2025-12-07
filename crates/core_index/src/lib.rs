//! Markdown parsing and analysis for NeuroFlow Notes.
//!
//! This crate extracts structured data from markdown files:
//! - Headings (for heading_path and title extraction)
//! - Todos (task list items)
//! - Wikilinks ([[link]])
//! - Tags (#tag)

pub mod markdown;

pub use markdown::{NoteAnalysis, ParsedHeading, ParsedTodo};
