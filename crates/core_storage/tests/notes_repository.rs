//! Tests for the notes repository.

mod helpers;

use helpers::{setup_test_repo};

#[tokio::test]
async fn test_upsert_note_insert() {
    let (_pool, repo) = setup_test_repo().await;
    
    // Insert a new note
    let note_id = repo.upsert_note("test.md", Some("Test Note"), "hash123")
        .await
        .unwrap();
    
    // Verify note was created
    assert!(note_id > 0);
    
    let note = repo.get_note(note_id).await.unwrap();
    assert_eq!(note.path, "test.md");
    assert_eq!(note.title, Some("Test Note".to_string()));
}

#[tokio::test]
async fn test_upsert_note_update() {
    let (_pool, repo) = setup_test_repo().await;
    
    // Insert a note
    let note_id1 = repo.upsert_note("test.md", Some("Original Title"), "hash1")
        .await
        .unwrap();
    
    // Update the same note (same path)
    let note_id2 = repo.upsert_note("test.md", Some("Updated Title"), "hash2")
        .await
        .unwrap();
    
    // Should return the same ID
    assert_eq!(note_id1, note_id2);
    
    // Verify title was updated
    let note = repo.get_note(note_id1).await.unwrap();
    assert_eq!(note.title, Some("Updated Title".to_string()));
}

#[tokio::test]
async fn test_get_note_by_id() {
    let (_pool, repo) = setup_test_repo().await;
    
    let note_id = repo.upsert_note("test.md", Some("Test Note"), "hash123")
        .await
        .unwrap();
    
    let note = repo.get_note(note_id).await.unwrap();
    assert_eq!(note.id, note_id);
    assert_eq!(note.path, "test.md");
    assert_eq!(note.title, Some("Test Note".to_string()));
    assert!(!note.pinned);
    assert!(note.created_at.is_some());
    assert!(note.updated_at.is_some());
}

#[tokio::test]
async fn test_get_note_by_path() {
    let (_pool, repo) = setup_test_repo().await;
    
    let note_id = repo.upsert_note("test.md", Some("Test Note"), "hash123")
        .await
        .unwrap();
    
    let note = repo.get_note_by_path("test.md").await.unwrap();
    assert_eq!(note.id, note_id);
    assert_eq!(note.path, "test.md");
    assert_eq!(note.title, Some("Test Note".to_string()));
}

#[tokio::test]
async fn test_get_note_id_by_path() {
    let (_pool, repo) = setup_test_repo().await;
    
    let note_id = repo.upsert_note("test.md", Some("Test Note"), "hash123")
        .await
        .unwrap();
    
    // Existing note
    let found_id = repo.get_note_id_by_path("test.md").await.unwrap();
    assert_eq!(found_id, Some(note_id));
    
    // Non-existent note
    let not_found = repo.get_note_id_by_path("nonexistent.md").await.unwrap();
    assert_eq!(not_found, None);
}

#[tokio::test]
async fn test_get_note_hash() {
    let (_pool, repo) = setup_test_repo().await;
    
    repo.upsert_note("test.md", Some("Test Note"), "hash123")
        .await
        .unwrap();
    
    // Existing note
    let hash = repo.get_note_hash("test.md").await.unwrap();
    assert_eq!(hash, Some("hash123".to_string()));
    
    // Non-existent note
    let not_found = repo.get_note_hash("nonexistent.md").await.unwrap();
    assert_eq!(not_found, None);
}

#[tokio::test]
async fn test_list_notes() {
    let (_pool, repo) = setup_test_repo().await;
    
    // Insert multiple notes
    repo.upsert_note("note1.md", Some("Note 1"), "hash1").await.unwrap();
    repo.upsert_note("note2.md", Some("Note 2"), "hash2").await.unwrap();
    repo.upsert_note("note3.md", None, "hash3").await.unwrap();
    
    let notes = repo.list_notes().await.unwrap();
    assert_eq!(notes.len(), 3);
    
    // Should be ordered by path
    assert_eq!(notes[0].path, "note1.md");
    assert_eq!(notes[1].path, "note2.md");
    assert_eq!(notes[2].path, "note3.md");
    
    // Check titles
    assert_eq!(notes[0].title, Some("Note 1".to_string()));
    assert_eq!(notes[1].title, Some("Note 2".to_string()));
    assert_eq!(notes[2].title, None);
}

#[tokio::test]
async fn test_delete_note() {
    let (_pool, repo) = setup_test_repo().await;
    
    let note_id = repo.upsert_note("test.md", Some("Test Note"), "hash123")
        .await
        .unwrap();
    
    // Delete the note
    let deleted_id = repo.delete_note("test.md").await.unwrap();
    assert_eq!(deleted_id, Some(note_id));
    
    // Verify note is gone
    let result = repo.get_note(note_id).await;
    assert!(result.is_err());
    
    // Deleting non-existent note should return None
    let not_found = repo.delete_note("nonexistent.md").await.unwrap();
    assert_eq!(not_found, None);
}

#[tokio::test]
async fn test_count_notes() {
    let (_pool, repo) = setup_test_repo().await;
    
    // Initially empty
    let count = repo.count_notes().await.unwrap();
    assert_eq!(count, 0);
    
    // Add notes
    repo.upsert_note("note1.md", Some("Note 1"), "hash1").await.unwrap();
    repo.upsert_note("note2.md", Some("Note 2"), "hash2").await.unwrap();
    
    let count = repo.count_notes().await.unwrap();
    assert_eq!(count, 2);
    
    // Delete one
    repo.delete_note("note1.md").await.unwrap();
    
    let count = repo.count_notes().await.unwrap();
    assert_eq!(count, 1);
}

#[tokio::test]
async fn test_rename_note() {
    let (_pool, repo) = setup_test_repo().await;
    
    let note_id = repo.upsert_note("old.md", Some("Test Note"), "hash123")
        .await
        .unwrap();
    
    // Rename the note
    let renamed_id = repo.rename_note("old.md", "new.md").await.unwrap();
    assert_eq!(renamed_id, note_id);
    
    // Verify old path doesn't exist
    let old_result = repo.get_note_by_path("old.md").await;
    assert!(old_result.is_err());
    
    // Verify new path exists
    let note = repo.get_note_by_path("new.md").await.unwrap();
    assert_eq!(note.id, note_id);
    assert_eq!(note.path, "new.md");
    assert_eq!(note.title, Some("Test Note".to_string()));
}

#[tokio::test]
async fn test_index_note() {
    let (_pool, repo) = setup_test_repo().await;
    
    use core_index::NoteAnalysis;
    
    let analysis = NoteAnalysis {
        title: Some("Test Note".to_string()),
        headings: vec![],
        tags: vec!["rust".to_string(), "testing".to_string()],
        todos: vec![],
        links: vec!["other.md".to_string()],
        properties: vec![],
    };
    
    // Create the linked note first
    repo.upsert_note("other.md", Some("Other Note"), "hash2").await.unwrap();
    
    // Index the note
    let note_id = repo.index_note("test.md", "# Test Note\nSome content", "hash1", &analysis)
        .await
        .unwrap();
    
    // Verify note was created
    let note = repo.get_note(note_id).await.unwrap();
    assert_eq!(note.path, "test.md");
    assert_eq!(note.title, Some("Test Note".to_string()));
    
    // Verify tags were created
    let tags = repo.get_tags_for_note(note_id).await.unwrap();
    assert_eq!(tags.len(), 2);
    assert!(tags.contains(&"rust".to_string()));
    assert!(tags.contains(&"testing".to_string()));
    
    // Verify backlink was created
    let other_note = repo.get_note_by_path("other.md").await.unwrap();
    let backlinks = repo.get_backlinks(other_note.id).await.unwrap();
    assert_eq!(backlinks.len(), 1);
    assert_eq!(backlinks[0].from_note_id, note_id);
}
