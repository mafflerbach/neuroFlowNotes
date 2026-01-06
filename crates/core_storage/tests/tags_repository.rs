//! Tests for the tags repository.

mod helpers;

use helpers::{count_rows, get_tags_for_note, insert_test_note, setup_test_repo};

#[tokio::test]
async fn test_replace_tags_insert() {
    let (_pool, repo) = setup_test_repo().await;
    let pool = repo.pool();
    let note_id = insert_test_note(pool, "test.md", Some("Test Note")).await;

    // Insert tags
    let tags = vec!["rust".to_string(), "testing".to_string()];
    repo.replace_tags(note_id, &tags)
        .await
        .unwrap();

    // Verify tags were inserted
    let tags = get_tags_for_note(pool, note_id).await;
    assert_eq!(tags.len(), 2);
    assert_eq!(tags[0], "rust");
    assert_eq!(tags[1], "testing");
}

#[tokio::test]
async fn test_replace_tags_update() {
    let (_pool, repo) = setup_test_repo().await;
    let pool = repo.pool();
    let note_id = insert_test_note(pool, "test.md", Some("Test Note")).await;

    // Insert initial tags
    let initial_tags = vec!["rust".to_string(), "testing".to_string()];
    repo.replace_tags(note_id, &initial_tags)
        .await
        .unwrap();

    // Replace with new tags
    let new_tags = vec!["python".to_string(), "automation".to_string()];
    repo.replace_tags(note_id, &new_tags)
        .await
        .unwrap();

    // Verify old tags were replaced
    let tags = get_tags_for_note(pool, note_id).await;
    assert_eq!(tags.len(), 2);
    assert_eq!(tags[0], "automation");
    assert_eq!(tags[1], "python");
}

#[tokio::test]
async fn test_replace_tags_empty() {
    let (_pool, repo) = setup_test_repo().await;
    let pool = repo.pool();
    let note_id = insert_test_note(pool, "test.md", Some("Test Note")).await;

    // Insert tags
    let tags = vec!["rust".to_string()];
    repo.replace_tags(note_id, &tags)
        .await
        .unwrap();

    // Clear all tags
    let empty_tags: Vec<String> = vec![];
    repo.replace_tags(note_id, &empty_tags)
        .await
        .unwrap();

    // Verify tags were cleared
    let tags = get_tags_for_note(pool, note_id).await;
    assert_eq!(tags.len(), 0);
}

#[tokio::test]
async fn test_list_tags() {
    let (_pool, repo) = setup_test_repo().await;
    let pool = repo.pool();
    
    // Create multiple notes with tags
    let note1 = insert_test_note(pool, "note1.md", Some("Note 1")).await;
    let note2 = insert_test_note(pool, "note2.md", Some("Note 2")).await;
    
    let tags1 = vec!["rust".to_string(), "testing".to_string()];
    repo.replace_tags(note1, &tags1)
        .await
        .unwrap();
    
    let tags2 = vec!["rust".to_string(), "python".to_string()];
    repo.replace_tags(note2, &tags2)
        .await
        .unwrap();

    // List all unique tags
    let tags = repo.list_tags().await.unwrap();
    
    assert_eq!(tags.len(), 3);
    assert!(tags.iter().any(|t| t.tag == "rust"));
    assert!(tags.iter().any(|t| t.tag == "testing"));
    assert!(tags.iter().any(|t| t.tag == "python"));
}

#[tokio::test]
async fn test_list_tags_with_counts() {
    let (_pool, repo) = setup_test_repo().await;
    let pool = repo.pool();
    
    // Create notes with overlapping tags
    let note1 = insert_test_note(pool, "note1.md", Some("Note 1")).await;
    let note2 = insert_test_note(pool, "note2.md", Some("Note 2")).await;
    let note3 = insert_test_note(pool, "note3.md", Some("Note 3")).await;
    
    let tags1 = vec!["rust".to_string()];
    repo.replace_tags(note1, &tags1)
        .await
        .unwrap();
    
    let tags2 = vec!["rust".to_string(), "python".to_string()];
    repo.replace_tags(note2, &tags2)
        .await
        .unwrap();
    
    let tags3 = vec!["rust".to_string()];
    repo.replace_tags(note3, &tags3)
        .await
        .unwrap();

    // List tags with counts
    let tags = repo.list_tags().await.unwrap();
    
    let rust_tag = tags.iter().find(|t| t.tag == "rust").unwrap();
    let python_tag = tags.iter().find(|t| t.tag == "python").unwrap();
    
    assert_eq!(rust_tag.count, 3);
    assert_eq!(python_tag.count, 1);
}

#[tokio::test]
async fn test_cascade_delete() {
    let (_pool, repo) = setup_test_repo().await;
    let pool = repo.pool();
    let note_id = insert_test_note(pool, "test.md", Some("Test Note")).await;

    // Insert tags
    let tags = vec!["rust".to_string()];
    repo.replace_tags(note_id, &tags)
        .await
        .unwrap();

    // Delete the note
    sqlx::query("DELETE FROM notes WHERE id = ?")
        .bind(note_id)
        .execute(pool)
        .await
        .unwrap();

    // Verify tags were cascade deleted
    let tag_count = count_rows(pool, "tags").await;
    assert_eq!(tag_count, 0);
}
