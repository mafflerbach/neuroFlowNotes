//! Tests for the dates repository.

mod helpers;

use helpers::{insert_test_note, insert_test_property, setup_test_repo};

#[tokio::test]
async fn test_get_notes_for_date_scheduled() {
    let (_pool, repo) = setup_test_repo().await;
    let pool = repo.pool();

    // Create note
    let note_id = insert_test_note(pool, "meeting.md", Some("Team Meeting")).await;

    // Create schedule block for 2024-01-15
    repo.create_schedule_block(
        Some(note_id),
        "2024-01-15",
        "10:00",
        "11:00",
        Some("Meeting"),
        None,
        None,
        None,
    )
    .await
    .unwrap();

    // Query notes for that date
    let notes = repo.get_notes_for_date("2024-01-15").await.unwrap();

    // Verify
    assert_eq!(notes.len(), 1);
    assert_eq!(notes[0].note.id, note_id);
    assert_eq!(notes[0].note.path, "meeting.md");
    assert_eq!(notes[0].source, "scheduled");
    assert!(notes[0].schedule_block.is_some());
}

#[tokio::test]
async fn test_get_notes_for_date_journal() {
    let (_pool, repo) = setup_test_repo().await;
    let pool = repo.pool();

    // Create note with journal_date property
    let note_id = insert_test_note(pool, "journal.md", Some("Daily Journal")).await;
    insert_test_property(pool, note_id, "journal_date", "2024-01-15", "text").await;

    // Query notes for that date
    let notes = repo.get_notes_for_date("2024-01-15").await.unwrap();

    // Verify
    assert_eq!(notes.len(), 1);
    assert_eq!(notes[0].note.id, note_id);
    assert_eq!(notes[0].note.path, "journal.md");
    assert_eq!(notes[0].source, "journal");
    assert!(notes[0].schedule_block.is_none());
}

#[tokio::test]
async fn test_get_notes_for_date_created() {
    let (_pool, repo) = setup_test_repo().await;
    let pool = repo.pool();

    // Create note with specific created_date
    let note_id = sqlx::query_scalar::<_, i64>(
        "INSERT INTO notes (path, title, hash, created_at, updated_at, created_date) 
         VALUES (?, ?, 'test-hash', datetime('now'), datetime('now'), ?) 
         RETURNING id",
    )
    .bind("created.md")
    .bind("New Note")
    .bind("2024-01-15")
    .fetch_one(pool)
    .await
    .unwrap();

    // Query notes for that date
    let notes = repo.get_notes_for_date("2024-01-15").await.unwrap();

    // Verify
    assert_eq!(notes.len(), 1);
    assert_eq!(notes[0].note.id, note_id);
    assert_eq!(notes[0].note.path, "created.md");
    assert_eq!(notes[0].source, "created");
    assert!(notes[0].schedule_block.is_none());
}

#[tokio::test]
async fn test_get_notes_for_date_priority_order() {
    let (_pool, repo) = setup_test_repo().await;
    let pool = repo.pool();

    // Create note 1 with schedule block (priority 1)
    let note1_id = insert_test_note(pool, "scheduled.md", Some("Scheduled")).await;
    repo.create_schedule_block(
        Some(note1_id),
        "2024-01-15",
        "10:00",
        "11:00",
        Some("Event"),
        None,
        None,
        None,
    )
    .await
    .unwrap();

    // Create note 2 with journal_date (priority 2)
    let note2_id = insert_test_note(pool, "journal.md", Some("Journal")).await;
    insert_test_property(pool, note2_id, "journal_date", "2024-01-15", "text").await;

    // Create note 3 with created_date (priority 3)
    let note3_id = sqlx::query_scalar::<_, i64>(
        "INSERT INTO notes (path, title, hash, created_at, updated_at, created_date) 
         VALUES (?, ?, 'test-hash', datetime('now'), datetime('now'), ?) 
         RETURNING id",
    )
    .bind("created.md")
    .bind("Created")
    .bind("2024-01-15")
    .fetch_one(pool)
    .await
    .unwrap();

    // Query notes for that date
    let notes = repo.get_notes_for_date("2024-01-15").await.unwrap();

    // Verify all three notes are returned in order: scheduled, journal, created
    assert_eq!(notes.len(), 3);
    assert_eq!(notes[0].note.id, note1_id);
    assert_eq!(notes[0].source, "scheduled");
    assert_eq!(notes[1].note.id, note2_id);
    assert_eq!(notes[1].source, "journal");
    assert_eq!(notes[2].note.id, note3_id);
    assert_eq!(notes[2].source, "created");
}

#[tokio::test]
async fn test_get_notes_for_date_deduplication() {
    let (_pool, repo) = setup_test_repo().await;
    let pool = repo.pool();

    // Create note with BOTH schedule block AND journal_date for same date
    let note_id = insert_test_note(pool, "duplicate.md", Some("Duplicate Entry")).await;

    // Add schedule block
    repo.create_schedule_block(
        Some(note_id),
        "2024-01-15",
        "10:00",
        "11:00",
        Some("Event"),
        None,
        None,
        None,
    )
    .await
    .unwrap();

    // Add journal_date property
    insert_test_property(pool, note_id, "journal_date", "2024-01-15", "text").await;

    // Query notes for that date
    let notes = repo.get_notes_for_date("2024-01-15").await.unwrap();

    // Verify note appears only once (scheduled takes priority)
    assert_eq!(notes.len(), 1);
    assert_eq!(notes[0].note.id, note_id);
    assert_eq!(notes[0].source, "scheduled");
    assert!(notes[0].schedule_block.is_some());
}

#[tokio::test]
async fn test_get_notes_for_date_no_results() {
    let (_pool, repo) = setup_test_repo().await;

    // Query notes for a date with no notes
    let notes = repo.get_notes_for_date("2024-01-15").await.unwrap();

    // Verify empty result
    assert_eq!(notes.len(), 0);
}

#[tokio::test]
async fn test_get_notes_for_date_range() {
    let (_pool, repo) = setup_test_repo().await;
    let pool = repo.pool();

    // Create notes for different dates
    let note1_id = insert_test_note(pool, "note1.md", Some("Note 1")).await;
    repo.create_schedule_block(
        Some(note1_id),
        "2024-01-15",
        "10:00",
        "11:00",
        None,
        None,
        None,
        None,
    )
    .await
    .unwrap();

    let note2_id = insert_test_note(pool, "note2.md", Some("Note 2")).await;
    insert_test_property(pool, note2_id, "journal_date", "2024-01-16", "text").await;

    let note3_id = sqlx::query_scalar::<_, i64>(
        "INSERT INTO notes (path, title, hash, created_at, updated_at, created_date) 
         VALUES (?, ?, 'test-hash', datetime('now'), datetime('now'), ?) 
         RETURNING id",
    )
    .bind("note3.md")
    .bind("Note 3")
    .bind("2024-01-17")
    .fetch_one(pool)
    .await
    .unwrap();

    // Query notes for range
    let results = repo
        .get_notes_for_date_range("2024-01-15", "2024-01-17")
        .await
        .unwrap();

    // Verify we get 3 dates with 1 note each, sorted by date
    assert_eq!(results.len(), 3);

    // Check date order
    assert_eq!(results[0].0, "2024-01-15");
    assert_eq!(results[1].0, "2024-01-16");
    assert_eq!(results[2].0, "2024-01-17");

    // Verify notes
    assert_eq!(results[0].1.len(), 1);
    assert_eq!(results[0].1[0].note.id, note1_id);
    assert_eq!(results[0].1[0].source, "scheduled");

    assert_eq!(results[1].1.len(), 1);
    assert_eq!(results[1].1[0].note.id, note2_id);
    assert_eq!(results[1].1[0].source, "journal");

    assert_eq!(results[2].1.len(), 1);
    assert_eq!(results[2].1[0].note.id, note3_id);
    assert_eq!(results[2].1[0].source, "created");
}

#[tokio::test]
async fn test_get_notes_for_date_range_deduplication() {
    let (_pool, repo) = setup_test_repo().await;
    let pool = repo.pool();

    // Create note with multiple sources for same date
    let note_id = insert_test_note(pool, "multi.md", Some("Multi Source")).await;

    // Add schedule block
    repo.create_schedule_block(
        Some(note_id),
        "2024-01-15",
        "10:00",
        "11:00",
        None,
        None,
        None,
        None,
    )
    .await
    .unwrap();

    // Add journal_date
    insert_test_property(pool, note_id, "journal_date", "2024-01-15", "text").await;

    // Query range
    let results = repo
        .get_notes_for_date_range("2024-01-15", "2024-01-15")
        .await
        .unwrap();

    // Verify note appears only once (scheduled priority)
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].0, "2024-01-15");
    assert_eq!(results[0].1.len(), 1);
    assert_eq!(results[0].1[0].note.id, note_id);
    assert_eq!(results[0].1[0].source, "scheduled");
}

#[tokio::test]
async fn test_get_notes_for_date_range_empty() {
    let (_pool, repo) = setup_test_repo().await;

    // Query range with no notes
    let results = repo
        .get_notes_for_date_range("2024-01-15", "2024-01-17")
        .await
        .unwrap();

    // Verify empty result
    assert_eq!(results.len(), 0);
}

#[tokio::test]
async fn test_get_notes_for_date_recurring_schedule() {
    let (_pool, repo) = setup_test_repo().await;
    let pool = repo.pool();

    // Create note with recurring schedule (daily for 3 days)
    let note_id = insert_test_note(pool, "recurring.md", Some("Daily Standup")).await;
    repo.create_schedule_block(
        Some(note_id),
        "2024-01-15",
        "09:00",
        "09:15",
        Some("Standup"),
        None,
        None,
        Some("FREQ=DAILY;COUNT=3"),
    )
    .await
    .unwrap();

    // Query for the second occurrence
    let notes = repo.get_notes_for_date("2024-01-16").await.unwrap();

    // Verify recurring event appears
    assert_eq!(notes.len(), 1);
    assert_eq!(notes[0].note.id, note_id);
    assert_eq!(notes[0].source, "scheduled");
    assert!(notes[0].schedule_block.is_some());
}
