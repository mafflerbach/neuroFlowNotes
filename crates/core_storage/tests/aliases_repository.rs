//! Tests for the aliases repository.

mod helpers;

use helpers::{insert_test_note, setup_test_repo};

#[tokio::test]
async fn test_replace_aliases_insert() {
    let (_pool, repo) = setup_test_repo().await;
    let pool = repo.pool();
    let note_id = insert_test_note(pool, "main-note.md", Some("Main Note")).await;

    // Add multiple aliases
    let aliases = vec![
        "alias1".to_string(),
        "alias2".to_string(),
        "alternative-name".to_string(),
    ];

    repo.replace_aliases(note_id, &aliases).await.unwrap();

    // Verify all aliases were inserted
    let stored_aliases = repo.get_aliases_for_note(note_id).await.unwrap();
    assert_eq!(stored_aliases.len(), 3);
    assert!(stored_aliases.contains(&"alias1".to_string()));
    assert!(stored_aliases.contains(&"alias2".to_string()));
    assert!(stored_aliases.contains(&"alternative-name".to_string()));
}

#[tokio::test]
async fn test_find_note_by_alias_case_insensitive() {
    let (_pool, repo) = setup_test_repo().await;
    let pool = repo.pool();
    let note_id = insert_test_note(pool, "note.md", Some("Test Note")).await;

    // Add alias with mixed case
    let aliases = vec!["MyAlias".to_string()];
    repo.replace_aliases(note_id, &aliases).await.unwrap();

    // Search with lowercase
    let found = repo.find_note_by_alias("myalias").await.unwrap();
    assert_eq!(found, Some(note_id));

    // Search with uppercase
    let found = repo.find_note_by_alias("MYALIAS").await.unwrap();
    assert_eq!(found, Some(note_id));

    // Search with exact case
    let found = repo.find_note_by_alias("MyAlias").await.unwrap();
    assert_eq!(found, Some(note_id));

    // Search with different mixed case
    let found = repo.find_note_by_alias("mYaLiAs").await.unwrap();
    assert_eq!(found, Some(note_id));

    // Search for non-existent alias
    let not_found = repo.find_note_by_alias("nonexistent").await.unwrap();
    assert_eq!(not_found, None);
}

#[tokio::test]
async fn test_replace_aliases_update() {
    let (_pool, repo) = setup_test_repo().await;
    let pool = repo.pool();
    let note_id = insert_test_note(pool, "note.md", Some("Test Note")).await;

    // Add initial aliases
    let old_aliases = vec!["old1".to_string(), "old2".to_string()];
    repo.replace_aliases(note_id, &old_aliases).await.unwrap();

    // Verify old aliases exist
    let found = repo.find_note_by_alias("old1").await.unwrap();
    assert_eq!(found, Some(note_id));

    // Replace with new aliases
    let new_aliases = vec!["new1".to_string(), "new2".to_string(), "new3".to_string()];
    repo.replace_aliases(note_id, &new_aliases).await.unwrap();

    // Verify old aliases are gone
    let not_found = repo.find_note_by_alias("old1").await.unwrap();
    assert_eq!(not_found, None);

    let not_found = repo.find_note_by_alias("old2").await.unwrap();
    assert_eq!(not_found, None);

    // Verify new aliases exist
    let found = repo.find_note_by_alias("new1").await.unwrap();
    assert_eq!(found, Some(note_id));

    let found = repo.find_note_by_alias("new2").await.unwrap();
    assert_eq!(found, Some(note_id));

    let found = repo.find_note_by_alias("new3").await.unwrap();
    assert_eq!(found, Some(note_id));

    // Verify count is correct
    let stored_aliases = repo.get_aliases_for_note(note_id).await.unwrap();
    assert_eq!(stored_aliases.len(), 3);
}

#[tokio::test]
async fn test_cascade_delete() {
    let (_pool, repo) = setup_test_repo().await;
    let pool = repo.pool();
    let note_id = insert_test_note(pool, "note.md", Some("Test Note")).await;

    // Add aliases
    let aliases = vec![
        "alias1".to_string(),
        "alias2".to_string(),
        "alias3".to_string(),
    ];
    repo.replace_aliases(note_id, &aliases).await.unwrap();

    // Verify aliases exist
    let stored_aliases = repo.get_aliases_for_note(note_id).await.unwrap();
    assert_eq!(stored_aliases.len(), 3);

    // Delete the note
    sqlx::query("DELETE FROM notes WHERE id = ?")
        .bind(note_id)
        .execute(pool)
        .await
        .unwrap();

    // Verify aliases were cascade deleted
    let count = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM aliases")
        .fetch_one(pool)
        .await
        .unwrap();
    assert_eq!(count, 0);
}
