//! Tests for the backlinks repository.

mod helpers;

use helpers::{count_rows, insert_test_note, setup_test_repo};

#[tokio::test]
async fn test_replace_backlinks_insert() {
    let (_pool, repo) = setup_test_repo().await;
    let pool = repo.pool();
    
    // Create notes
    let note1 = insert_test_note(pool, "note1.md", Some("Note 1")).await;
    let note2 = insert_test_note(pool, "note2.md", Some("Note 2")).await;
    let note3 = insert_test_note(pool, "note3.md", Some("Note 3")).await;
    
    // Add backlinks from note1 to note2 and note3
    let links = vec!["note2.md".to_string(), "note3.md".to_string()];
    repo.replace_backlinks(note1, &links).await.unwrap();
    
    // Verify backlinks were created
    let backlinks = repo.get_backlinks(note2).await.unwrap();
    assert_eq!(backlinks.len(), 1);
    assert_eq!(backlinks[0].from_note_id, note1);
    assert_eq!(backlinks[0].from_note_path, "note1.md");
    
    let backlinks = repo.get_backlinks(note3).await.unwrap();
    assert_eq!(backlinks.len(), 1);
    assert_eq!(backlinks[0].from_note_id, note1);
}

#[tokio::test]
async fn test_replace_backlinks_update() {
    let (_pool, repo) = setup_test_repo().await;
    let pool = repo.pool();
    
    // Create notes
    let note1 = insert_test_note(pool, "note1.md", Some("Note 1")).await;
    let note2 = insert_test_note(pool, "note2.md", Some("Note 2")).await;
    let note3 = insert_test_note(pool, "note3.md", Some("Note 3")).await;
    
    // Add initial backlinks
    let links = vec!["note2.md".to_string()];
    repo.replace_backlinks(note1, &links).await.unwrap();
    
    // Replace with different backlinks
    let new_links = vec!["note3.md".to_string()];
    repo.replace_backlinks(note1, &new_links).await.unwrap();
    
    // Verify old backlink was removed
    let backlinks = repo.get_backlinks(note2).await.unwrap();
    assert_eq!(backlinks.len(), 0);
    
    // Verify new backlink exists
    let backlinks = repo.get_backlinks(note3).await.unwrap();
    assert_eq!(backlinks.len(), 1);
    assert_eq!(backlinks[0].from_note_id, note1);
}

#[tokio::test]
async fn test_replace_backlinks_with_extension() {
    let (_pool, repo) = setup_test_repo().await;
    let pool = repo.pool();
    
    // Create notes
    let note1 = insert_test_note(pool, "note1.md", Some("Note 1")).await;
    let note2 = insert_test_note(pool, "note2.md", Some("Note 2")).await;
    
    // Test that both "note2" and "note2.md" work
    let links_without_ext = vec!["note2".to_string()];
    repo.replace_backlinks(note1, &links_without_ext).await.unwrap();
    
    let backlinks = repo.get_backlinks(note2).await.unwrap();
    assert_eq!(backlinks.len(), 1);
    assert_eq!(backlinks[0].from_note_id, note1);
}

#[tokio::test]
async fn test_get_backlinks_bidirectional() {
    let (_pool, repo) = setup_test_repo().await;
    let pool = repo.pool();
    
    // Create notes
    let note1 = insert_test_note(pool, "note1.md", Some("Note 1")).await;
    let note2 = insert_test_note(pool, "note2.md", Some("Note 2")).await;
    
    // Note1 links to Note2
    let links1 = vec!["note2.md".to_string()];
    repo.replace_backlinks(note1, &links1).await.unwrap();
    
    // Note2 links to Note1
    let links2 = vec!["note1.md".to_string()];
    repo.replace_backlinks(note2, &links2).await.unwrap();
    
    // Verify bidirectional links
    let backlinks1 = repo.get_backlinks(note1).await.unwrap();
    assert_eq!(backlinks1.len(), 1);
    assert_eq!(backlinks1[0].from_note_id, note2);
    
    let backlinks2 = repo.get_backlinks(note2).await.unwrap();
    assert_eq!(backlinks2.len(), 1);
    assert_eq!(backlinks2[0].from_note_id, note1);
}

#[tokio::test]
async fn test_get_notes_linking_to() {
    let (_pool, repo) = setup_test_repo().await;
    let pool = repo.pool();
    
    // Create notes
    let target = insert_test_note(pool, "target.md", Some("Target Note")).await;
    let note1 = insert_test_note(pool, "note1.md", Some("Note 1")).await;
    let note2 = insert_test_note(pool, "note2.md", Some("Note 2")).await;
    
    // Both notes link to target
    let links1 = vec!["target.md".to_string()];
    repo.replace_backlinks(note1, &links1).await.unwrap();
    
    let links2 = vec!["target.md".to_string()];
    repo.replace_backlinks(note2, &links2).await.unwrap();
    
    // Get all notes linking to target
    let linking_notes = repo.get_notes_linking_to(target).await.unwrap();
    assert_eq!(linking_notes.len(), 2);
    
    let paths: Vec<String> = linking_notes.iter().map(|n| n.path.clone()).collect();
    assert!(paths.contains(&"note1.md".to_string()));
    assert!(paths.contains(&"note2.md".to_string()));
}

#[tokio::test]
async fn test_cascade_delete() {
    let (_pool, repo) = setup_test_repo().await;
    let pool = repo.pool();
    
    // Create notes
    let note1 = insert_test_note(pool, "note1.md", Some("Note 1")).await;
    let _note2 = insert_test_note(pool, "note2.md", Some("Note 2")).await;
    
    // Add backlink
    let links = vec!["note2.md".to_string()];
    repo.replace_backlinks(note1, &links).await.unwrap();
    
    // Delete the source note
    sqlx::query("DELETE FROM notes WHERE id = ?")
        .bind(note1)
        .execute(pool)
        .await
        .unwrap();
    
    // Verify backlinks were cascade deleted
    let backlink_count = count_rows(pool, "backlinks").await;
    assert_eq!(backlink_count, 0);
}
