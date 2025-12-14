//! Property types - including note properties, folder properties, and bulk operations.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

// ============================================================================
// Note Property Types
// ============================================================================

/// A key-value property for a note (stored in SQLite, not frontmatter).
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct PropertyDto {
    pub id: i64,
    pub note_id: i64,
    pub key: String,
    pub value: Option<String>,
    /// Type hint: "text", "date", "number", "boolean", "list"
    pub property_type: Option<String>,
    pub sort_order: Option<i32>,
}

/// Request to set a property value.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct SetPropertyRequest {
    pub note_id: i64,
    pub key: String,
    pub value: Option<String>,
    pub property_type: Option<String>,
}

/// Information about a property key used in the vault.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct PropertyKeyInfo {
    /// The property key name.
    pub key: String,
    /// Number of notes using this property.
    pub usage_count: i64,
    /// Sample values for this property (up to 10).
    pub sample_values: Vec<String>,
    /// Most common property type for this key (text, date, number, boolean, list).
    pub property_type: Option<String>,
}

// ============================================================================
// Property Management Types (bulk operations)
// ============================================================================

/// Request to rename a property key across all notes.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct RenamePropertyKeyRequest {
    /// The current key name.
    pub old_key: String,
    /// The new key name.
    pub new_key: String,
}

/// Request to rename a property value across all notes with that key.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct RenamePropertyValueRequest {
    /// The property key.
    pub key: String,
    /// The current value.
    pub old_value: String,
    /// The new value.
    pub new_value: String,
}

/// Request to merge two property keys (rename source to target, merging if target exists).
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct MergePropertyKeysRequest {
    /// The property key to merge from (will be deleted).
    pub source_key: String,
    /// The property key to merge into (will keep all values).
    pub target_key: String,
}

/// Request to delete a property key from all notes.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct DeletePropertyKeyRequest {
    /// The property key to delete.
    pub key: String,
}

/// Response for bulk property operations.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct PropertyOperationResult {
    /// Number of properties affected.
    pub affected_count: i64,
    /// Number of notes affected.
    pub notes_affected: i64,
}

/// Information about a property value used in the vault.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct PropertyValueInfo {
    /// The property value.
    pub value: String,
    /// Number of notes using this value.
    pub usage_count: i64,
}

/// A note that uses a specific property, including the value.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct NoteWithPropertyValue {
    /// The note ID.
    pub note_id: i64,
    /// The note path.
    pub path: String,
    /// The note title (if any).
    pub title: Option<String>,
    /// The property value in this note.
    pub value: Option<String>,
}

// ============================================================================
// Folder Property Types
// ============================================================================

/// A key-value property for a folder (inherited by notes in that folder tree).
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct FolderPropertyDto {
    pub id: i64,
    pub folder_path: String,
    pub key: String,
    pub value: Option<String>,
    /// Type hint: "text", "date", "number", "boolean", "list"
    pub property_type: Option<String>,
}

/// Request to set a folder property value.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct SetFolderPropertyRequest {
    pub folder_path: String,
    pub key: String,
    pub value: Option<String>,
    pub property_type: Option<String>,
}

/// A property with inheritance information.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct PropertyWithInheritance {
    pub id: i64,
    pub key: String,
    pub value: Option<String>,
    pub property_type: Option<String>,
    pub sort_order: Option<i32>,
    /// True if this property is inherited from a folder (not directly set on the note).
    pub inherited: bool,
    /// The folder path this property is inherited from (if inherited).
    pub inherited_from: Option<String>,
}
