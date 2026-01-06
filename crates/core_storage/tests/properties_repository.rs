//! Tests for the properties repository.

mod helpers;

use helpers::{insert_test_note, setup_test_repo};

#[tokio::test]
async fn test_set_property_insert() {
    let (_pool, repo) = setup_test_repo().await;
    let pool = repo.pool();
    let note_id = insert_test_note(pool, "test.md", Some("Test Note")).await;
    
    // Set a property
    let prop_id = repo.set_property(note_id, "status", Some("active"), Some("text"))
        .await
        .unwrap();
    
    assert!(prop_id > 0);
    
    // Verify property was created
    let prop = repo.get_property(note_id, "status").await.unwrap().unwrap();
    assert_eq!(prop.key, "status");
    assert_eq!(prop.value, Some("active".to_string()));
    assert_eq!(prop.property_type, Some("text".to_string()));
}

#[tokio::test]
async fn test_set_property_update() {
    let (_pool, repo) = setup_test_repo().await;
    let pool = repo.pool();
    let note_id = insert_test_note(pool, "test.md", Some("Test Note")).await;
    
    // Set initial property
    let prop_id1 = repo.set_property(note_id, "status", Some("active"), Some("text"))
        .await
        .unwrap();
    
    // Update the same property
    let prop_id2 = repo.set_property(note_id, "status", Some("completed"), Some("text"))
        .await
        .unwrap();
    
    // Should return same ID (upsert)
    assert_eq!(prop_id1, prop_id2);
    
    // Verify value was updated
    let prop = repo.get_property(note_id, "status").await.unwrap().unwrap();
    assert_eq!(prop.value, Some("completed".to_string()));
}

#[tokio::test]
async fn test_get_properties_for_note() {
    let (_pool, repo) = setup_test_repo().await;
    let pool = repo.pool();
    let note_id = insert_test_note(pool, "test.md", Some("Test Note")).await;
    
    // Add multiple properties
    repo.set_property(note_id, "status", Some("active"), Some("text")).await.unwrap();
    repo.set_property(note_id, "priority", Some("high"), Some("text")).await.unwrap();
    repo.set_property(note_id, "tags", Some("work"), Some("text")).await.unwrap();
    
    // Get all properties
    let props = repo.get_properties_for_note(note_id).await.unwrap();
    assert_eq!(props.len(), 3);
    
    // Should be ordered by key
    let keys: Vec<String> = props.iter().map(|p| p.key.clone()).collect();
    assert!(keys.contains(&"status".to_string()));
    assert!(keys.contains(&"priority".to_string()));
    assert!(keys.contains(&"tags".to_string()));
}

#[tokio::test]
async fn test_get_properties_for_notes_batch() {
    let (_pool, repo) = setup_test_repo().await;
    let pool = repo.pool();
    
    let note1 = insert_test_note(pool, "note1.md", Some("Note 1")).await;
    let note2 = insert_test_note(pool, "note2.md", Some("Note 2")).await;
    let note3 = insert_test_note(pool, "note3.md", Some("Note 3")).await;
    
    // Add properties to different notes
    repo.set_property(note1, "status", Some("active"), Some("text")).await.unwrap();
    repo.set_property(note2, "status", Some("done"), Some("text")).await.unwrap();
    repo.set_property(note2, "priority", Some("high"), Some("text")).await.unwrap();
    // note3 has no properties
    
    // Batch query
    let note_ids = vec![note1, note2, note3];
    let props_map = repo.get_properties_for_notes(&note_ids).await.unwrap();
    
    assert_eq!(props_map.len(), 3);
    assert_eq!(props_map.get(&note1).unwrap().len(), 1);
    assert_eq!(props_map.get(&note2).unwrap().len(), 2);
    assert_eq!(props_map.get(&note3).unwrap().len(), 0); // Empty vec for note with no properties
}

#[tokio::test]
async fn test_delete_property() {
    let (_pool, repo) = setup_test_repo().await;
    let pool = repo.pool();
    let note_id = insert_test_note(pool, "test.md", Some("Test Note")).await;
    
    // Add properties
    repo.set_property(note_id, "status", Some("active"), Some("text")).await.unwrap();
    repo.set_property(note_id, "priority", Some("high"), Some("text")).await.unwrap();
    
    // Delete one property
    repo.delete_property(note_id, "status").await.unwrap();
    
    // Verify it's gone
    let prop = repo.get_property(note_id, "status").await.unwrap();
    assert!(prop.is_none());
    
    // Other property should still exist
    let prop = repo.get_property(note_id, "priority").await.unwrap();
    assert!(prop.is_some());
}

#[tokio::test]
async fn test_delete_all_properties() {
    let (_pool, repo) = setup_test_repo().await;
    let pool = repo.pool();
    let note_id = insert_test_note(pool, "test.md", Some("Test Note")).await;
    
    // Add multiple properties
    repo.set_property(note_id, "status", Some("active"), Some("text")).await.unwrap();
    repo.set_property(note_id, "priority", Some("high"), Some("text")).await.unwrap();
    
    // Delete all properties
    repo.delete_all_properties(note_id).await.unwrap();
    
    // Verify all are gone
    let props = repo.get_properties_for_note(note_id).await.unwrap();
    assert_eq!(props.len(), 0);
}

#[tokio::test]
async fn test_replace_properties() {
    let (_pool, repo) = setup_test_repo().await;
    let pool = repo.pool();
    let note_id = insert_test_note(pool, "test.md", Some("Test Note")).await;
    
    use core_index::ParsedProperty;
    
    // Add initial DB-only property
    repo.set_property(note_id, "db_only", Some("value"), Some("text")).await.unwrap();
    
    // Replace with frontmatter properties (using upsert logic)
    let frontmatter_props = vec![
        ParsedProperty {
            key: "author".to_string(),
            value: Some("Alice".to_string()),
            property_type: "text".to_string(),
        },
        ParsedProperty {
            key: "date".to_string(),
            value: Some("2024-01-01".to_string()),
            property_type: "date".to_string(),
        },
    ];
    
    repo.replace_properties(note_id, &frontmatter_props).await.unwrap();
    
    // Verify frontmatter properties exist
    let props = repo.get_properties_for_note(note_id).await.unwrap();
    assert_eq!(props.len(), 3); // 2 frontmatter + 1 DB-only
    
    let keys: Vec<String> = props.iter().map(|p| p.key.clone()).collect();
    assert!(keys.contains(&"author".to_string()));
    assert!(keys.contains(&"date".to_string()));
    assert!(keys.contains(&"db_only".to_string())); // DB-only property preserved
}

#[tokio::test]
async fn test_get_property_by_key() {
    let (_pool, repo) = setup_test_repo().await;
    let pool = repo.pool();
    let note_id = insert_test_note(pool, "test.md", Some("Test Note")).await;
    
    repo.set_property(note_id, "status", Some("active"), Some("text")).await.unwrap();
    
    // Get existing property
    let prop = repo.get_property(note_id, "status").await.unwrap();
    assert!(prop.is_some());
    let prop = prop.unwrap();
    assert_eq!(prop.key, "status");
    assert_eq!(prop.value, Some("active".to_string()));
    
    // Get non-existent property
    let not_found = repo.get_property(note_id, "nonexistent").await.unwrap();
    assert!(not_found.is_none());
}

#[tokio::test]
async fn test_cascade_delete() {
    let (_pool, repo) = setup_test_repo().await;
    let pool = repo.pool();
    let note_id = insert_test_note(pool, "test.md", Some("Test Note")).await;
    
    // Add properties
    repo.set_property(note_id, "status", Some("active"), Some("text")).await.unwrap();
    repo.set_property(note_id, "priority", Some("high"), Some("text")).await.unwrap();
    
    // Delete the note
    sqlx::query("DELETE FROM notes WHERE id = ?")
        .bind(note_id)
        .execute(pool)
        .await
        .unwrap();
    
    // Verify properties were cascade deleted
    let count = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM properties")
        .fetch_one(pool)
        .await
        .unwrap();
    assert_eq!(count, 0);
}
