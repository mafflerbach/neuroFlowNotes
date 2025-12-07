//! Todo operations - toggling completion and syncing to files.

use crate::vault::{Vault, VaultError, VaultEvent};
use core_index::markdown::toggle_todo;
use shared_types::TodoDto;
use std::path::Path;
use tracing::{debug, instrument};

impl Vault {
    /// Toggle a todo's completion status.
    ///
    /// This will:
    /// 1. Get the todo from the database
    /// 2. Read the note file
    /// 3. Toggle the checkbox in the markdown
    /// 4. Write the file back
    /// 5. Reindex the note
    #[instrument(skip(self))]
    pub async fn toggle_todo(&self, todo_id: i64, completed: bool) -> Result<(), VaultError> {
        // Get the todo
        let todo = self
            .repo()
            .get_todo(todo_id)
            .await?
            .ok_or_else(|| VaultError::Storage(core_storage::StorageError::NoteNotFound(todo_id)))?;

        // Get the note
        let note = self.repo().get_note(todo.note_id).await?;

        // Read file content
        let content = self.fs().read_file(Path::new(&note.path)).await?;

        // Toggle the todo in the content
        let line_number = todo.line_number.unwrap_or(0) as usize;
        let new_content = toggle_todo(&content, line_number, completed);

        // Write back
        self.fs().write_file(Path::new(&note.path), &new_content).await?;

        // Reindex (will update the todo in DB)
        if let Some(note_id) = self.index_file(Path::new(&note.path)).await? {
            // Emit event
            self.emit(VaultEvent::NotesUpdated(vec![note_id]));
        }

        debug!("Toggled todo {} to completed={}", todo_id, completed);
        Ok(())
    }

    /// Get todos for a specific note.
    pub async fn get_todos_for_note(&self, note_id: i64) -> Result<Vec<TodoDto>, VaultError> {
        Ok(self.repo().get_todos_for_note(note_id).await?)
    }

    /// Get all incomplete todos across the vault.
    pub async fn get_incomplete_todos(&self) -> Result<Vec<TodoDto>, VaultError> {
        Ok(self.repo().get_incomplete_todos().await?)
    }
}
