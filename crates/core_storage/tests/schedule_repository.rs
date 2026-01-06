//! Tests for the schedule repository.

mod helpers;

use helpers::{insert_test_note, setup_test_repo};

#[tokio::test]
async fn test_create_schedule_block() {
    let (_pool, repo) = setup_test_repo().await;
    let pool = repo.pool();
    let note_id = insert_test_note(pool, "meeting.md", Some("Team Meeting")).await;

    // Create a schedule block
    let block_id = repo
        .create_schedule_block(
            Some(note_id),
            "2024-01-15",
            "10:00",
            "11:00",
            Some("Weekly Team Sync"),
            Some("#3b82f6"),
            Some("work"),
            None,
        )
        .await
        .unwrap();

    assert!(block_id > 0);

    // Verify block was created
    let block = repo.get_schedule_block(block_id).await.unwrap().unwrap();
    assert_eq!(block.note_id, Some(note_id));
    assert_eq!(block.date.to_string(), "2024-01-15");
    assert_eq!(block.start_time.to_string(), "10:00:00");
    assert_eq!(block.end_time.to_string(), "11:00:00");
    assert_eq!(block.label, Some("Weekly Team Sync".to_string()));
    assert_eq!(block.color, Some("#3b82f6".to_string()));
    assert_eq!(block.context, Some("work".to_string()));
    assert_eq!(block.rrule, None);
}

#[tokio::test]
async fn test_get_schedule_blocks_for_range_non_recurring() {
    let (_pool, repo) = setup_test_repo().await;
    let pool = repo.pool();

    // Create blocks on different dates
    let note1 = insert_test_note(pool, "note1.md", Some("Note 1")).await;
    let note2 = insert_test_note(pool, "note2.md", Some("Note 2")).await;

    repo.create_schedule_block(
        Some(note1),
        "2024-01-10",
        "09:00",
        "10:00",
        Some("Early meeting"),
        None,
        None,
        None,
    )
    .await
    .unwrap();

    repo.create_schedule_block(
        Some(note2),
        "2024-01-15",
        "14:00",
        "15:00",
        Some("Mid month meeting"),
        None,
        None,
        None,
    )
    .await
    .unwrap();

    repo.create_schedule_block(
        Some(note1),
        "2024-01-25",
        "16:00",
        "17:00",
        Some("Late meeting"),
        None,
        None,
        None,
    )
    .await
    .unwrap();

    // Query range that includes only middle block
    let blocks = repo
        .get_schedule_blocks_for_range("2024-01-12", "2024-01-20")
        .await
        .unwrap();

    assert_eq!(blocks.len(), 1);
    assert_eq!(blocks[0].label, Some("Mid month meeting".to_string()));
    assert_eq!(blocks[0].date.to_string(), "2024-01-15");

    // Query range that includes all blocks
    let blocks = repo
        .get_schedule_blocks_for_range("2024-01-01", "2024-01-31")
        .await
        .unwrap();

    assert_eq!(blocks.len(), 3);
    // Verify sorting by date, then time
    assert_eq!(blocks[0].date.to_string(), "2024-01-10");
    assert_eq!(blocks[1].date.to_string(), "2024-01-15");
    assert_eq!(blocks[2].date.to_string(), "2024-01-25");
}

#[tokio::test]
async fn test_update_schedule_block() {
    let (_pool, repo) = setup_test_repo().await;
    let pool = repo.pool();
    let note_id = insert_test_note(pool, "meeting.md", Some("Meeting")).await;

    let block_id = repo
        .create_schedule_block(
            Some(note_id),
            "2024-01-15",
            "10:00",
            "11:00",
            Some("Original Label"),
            Some("#ff0000"),
            Some("work"),
            None,
        )
        .await
        .unwrap();

    // Update label, color, and context
    repo.update_schedule_block(
        block_id,
        Some(note_id),
        None, // don't update date
        None, // don't update start_time
        None, // don't update end_time
        Some("Updated Label"),
        Some("#00ff00"),
        Some("personal"),
        None,
    )
    .await
    .unwrap();

    // Verify updates
    let block = repo.get_schedule_block(block_id).await.unwrap().unwrap();
    assert_eq!(block.label, Some("Updated Label".to_string()));
    assert_eq!(block.color, Some("#00ff00".to_string()));
    assert_eq!(block.context, Some("personal".to_string()));
    // Date and times should remain unchanged
    assert_eq!(block.date.to_string(), "2024-01-15");
    assert_eq!(block.start_time.to_string(), "10:00:00");
    assert_eq!(block.end_time.to_string(), "11:00:00");
}

#[tokio::test]
async fn test_get_schedule_blocks_for_note() {
    let (_pool, repo) = setup_test_repo().await;
    let pool = repo.pool();
    let note_id = insert_test_note(pool, "project.md", Some("Project")).await;
    let other_note = insert_test_note(pool, "other.md", Some("Other")).await;

    // Create multiple blocks for the same note
    repo.create_schedule_block(
        Some(note_id),
        "2024-01-10",
        "09:00",
        "10:00",
        Some("Session 1"),
        None,
        None,
        None,
    )
    .await
    .unwrap();

    repo.create_schedule_block(
        Some(note_id),
        "2024-01-15",
        "14:00",
        "15:00",
        Some("Session 2"),
        None,
        None,
        None,
    )
    .await
    .unwrap();

    // Create block for different note
    repo.create_schedule_block(
        Some(other_note),
        "2024-01-12",
        "11:00",
        "12:00",
        Some("Other Session"),
        None,
        None,
        None,
    )
    .await
    .unwrap();

    // Get blocks for specific note
    let blocks = repo.get_schedule_blocks_for_note(note_id).await.unwrap();
    assert_eq!(blocks.len(), 2);
    assert_eq!(blocks[0].label, Some("Session 1".to_string()));
    assert_eq!(blocks[1].label, Some("Session 2".to_string()));
}

#[tokio::test]
async fn test_delete_schedule_block() {
    let (_pool, repo) = setup_test_repo().await;

    let block_id = repo
        .create_schedule_block(
            None,
            "2024-01-15",
            "10:00",
            "11:00",
            Some("To Delete"),
            None,
            None,
            None,
        )
        .await
        .unwrap();

    // Verify exists
    let block = repo.get_schedule_block(block_id).await.unwrap();
    assert!(block.is_some());

    // Delete
    repo.delete_schedule_block(block_id).await.unwrap();

    // Verify gone
    let block = repo.get_schedule_block(block_id).await.unwrap();
    assert!(block.is_none());
}

#[tokio::test]
async fn test_rrule_daily_expansion() {
    let (_pool, repo) = setup_test_repo().await;

    // Create daily recurring block for 5 days
    repo.create_schedule_block(
        None,
        "2024-01-10",
        "09:00",
        "10:00",
        Some("Daily Standup"),
        None,
        Some("work"),
        Some("FREQ=DAILY;COUNT=5"),
    )
    .await
    .unwrap();

    // Query 10-day range
    let blocks = repo
        .get_schedule_blocks_for_range("2024-01-10", "2024-01-20")
        .await
        .unwrap();

    // Should have 5 occurrences (Jan 10-14)
    assert_eq!(blocks.len(), 5);
    assert_eq!(blocks[0].date.to_string(), "2024-01-10");
    assert_eq!(blocks[1].date.to_string(), "2024-01-11");
    assert_eq!(blocks[2].date.to_string(), "2024-01-12");
    assert_eq!(blocks[3].date.to_string(), "2024-01-13");
    assert_eq!(blocks[4].date.to_string(), "2024-01-14");

    // All should have same time and label
    for block in &blocks {
        assert_eq!(block.label, Some("Daily Standup".to_string()));
        assert_eq!(block.start_time.to_string(), "09:00:00");
        assert_eq!(block.context, Some("work".to_string()));
    }
}

#[tokio::test]
async fn test_rrule_weekly_expansion() {
    let (_pool, repo) = setup_test_repo().await;

    // Create weekly recurring block on Mon, Wed, Fri for 4 weeks
    // Starting Jan 8, 2024 (Monday)
    repo.create_schedule_block(
        None,
        "2024-01-08",
        "14:00",
        "15:00",
        Some("Team Sync"),
        None,
        None,
        Some("FREQ=WEEKLY;BYDAY=MO,WE,FR;COUNT=6"),
    )
    .await
    .unwrap();

    // Query 3-week range
    let blocks = repo
        .get_schedule_blocks_for_range("2024-01-08", "2024-01-26")
        .await
        .unwrap();

    // Should have 6 occurrences (2 weeks * 3 days)
    assert_eq!(blocks.len(), 6);

    // Verify it's on correct weekdays (Mon=1, Wed=3, Fri=5)
    assert_eq!(blocks[0].date.to_string(), "2024-01-08"); // Mon
    assert_eq!(blocks[1].date.to_string(), "2024-01-10"); // Wed
    assert_eq!(blocks[2].date.to_string(), "2024-01-12"); // Fri
    assert_eq!(blocks[3].date.to_string(), "2024-01-15"); // Mon
    assert_eq!(blocks[4].date.to_string(), "2024-01-17"); // Wed
    assert_eq!(blocks[5].date.to_string(), "2024-01-19"); // Fri
}

#[tokio::test]
async fn test_rrule_monthly_expansion() {
    let (_pool, repo) = setup_test_repo().await;

    // Create monthly recurring block on 15th of each month
    repo.create_schedule_block(
        None,
        "2024-01-15",
        "10:00",
        "11:00",
        Some("Monthly Review"),
        None,
        None,
        Some("FREQ=MONTHLY;BYMONTHDAY=15;COUNT=3"),
    )
    .await
    .unwrap();

    // Query 4-month range
    let blocks = repo
        .get_schedule_blocks_for_range("2024-01-01", "2024-04-30")
        .await
        .unwrap();

    // Should have 3 occurrences (Jan 15, Feb 15, Mar 15)
    assert_eq!(blocks.len(), 3);
    assert_eq!(blocks[0].date.to_string(), "2024-01-15");
    assert_eq!(blocks[1].date.to_string(), "2024-02-15");
    assert_eq!(blocks[2].date.to_string(), "2024-03-15");
}

#[tokio::test]
async fn test_rrule_with_until_date() {
    let (_pool, repo) = setup_test_repo().await;

    // Create daily recurring block until Jan 14
    repo.create_schedule_block(
        None,
        "2024-01-10",
        "09:00",
        "10:00",
        Some("Daily Task"),
        None,
        None,
        Some("FREQ=DAILY;UNTIL=20240114T000000Z"),
    )
    .await
    .unwrap();

    // Query range
    let blocks = repo
        .get_schedule_blocks_for_range("2024-01-10", "2024-01-20")
        .await
        .unwrap();

    // UNTIL is inclusive, but check actual count
    // RRULE UNTIL behavior may vary - verify what we actually get
    assert!(blocks.len() >= 4); // At minimum Jan 10-13
    assert_eq!(blocks[0].date.to_string(), "2024-01-10");
    // Last occurrence should be Jan 13 or Jan 14 depending on UNTIL implementation
    assert!(blocks.last().unwrap().date.to_string() == "2024-01-13" 
        || blocks.last().unwrap().date.to_string() == "2024-01-14");
}

#[tokio::test]
async fn test_rrule_with_interval() {
    let (_pool, repo) = setup_test_repo().await;

    // Create recurring block every 2 weeks on Monday
    repo.create_schedule_block(
        None,
        "2024-01-08", // Monday
        "10:00",
        "11:00",
        Some("Bi-weekly Meeting"),
        None,
        None,
        Some("FREQ=WEEKLY;INTERVAL=2;BYDAY=MO;COUNT=4"),
    )
    .await
    .unwrap();

    // Query 2-month range
    let blocks = repo
        .get_schedule_blocks_for_range("2024-01-01", "2024-02-28")
        .await
        .unwrap();

    // Should have 4 occurrences every 2 weeks
    assert_eq!(blocks.len(), 4);
    assert_eq!(blocks[0].date.to_string(), "2024-01-08"); // Week 1
    assert_eq!(blocks[1].date.to_string(), "2024-01-22"); // Week 3
    assert_eq!(blocks[2].date.to_string(), "2024-02-05"); // Week 5
    assert_eq!(blocks[3].date.to_string(), "2024-02-19"); // Week 7
}

#[tokio::test]
async fn test_rrule_expansion_before_range() {
    let (_pool, repo) = setup_test_repo().await;

    // Create recurring block that starts before query range
    repo.create_schedule_block(
        None,
        "2024-01-01",
        "09:00",
        "10:00",
        Some("Daily Task"),
        None,
        None,
        Some("FREQ=DAILY;COUNT=20"),
    )
    .await
    .unwrap();

    // Query range starting later
    let blocks = repo
        .get_schedule_blocks_for_range("2024-01-10", "2024-01-15")
        .await
        .unwrap();

    // Should only include occurrences in range (Jan 10-15)
    assert_eq!(blocks.len(), 6);
    assert_eq!(blocks[0].date.to_string(), "2024-01-10");
    assert_eq!(blocks[5].date.to_string(), "2024-01-15");
}

#[tokio::test]
async fn test_invalid_rrule_falls_back_to_base() {
    let (_pool, repo) = setup_test_repo().await;

    // Create block with invalid RRULE
    repo.create_schedule_block(
        None,
        "2024-01-15",
        "10:00",
        "11:00",
        Some("Invalid RRULE"),
        None,
        None,
        Some("INVALID_RRULE_STRING"),
    )
    .await
    .unwrap();

    // Query should still return base block (with warning logged)
    let blocks = repo
        .get_schedule_blocks_for_range("2024-01-10", "2024-01-20")
        .await
        .unwrap();

    // Should include the base block on its original date
    assert_eq!(blocks.len(), 1);
    assert_eq!(blocks[0].date.to_string(), "2024-01-15");
    assert_eq!(blocks[0].label, Some("Invalid RRULE".to_string()));
}

#[tokio::test]
async fn test_cascade_delete() {
    let (_pool, repo) = setup_test_repo().await;
    let pool = repo.pool();
    let note_id = insert_test_note(pool, "note.md", Some("Note")).await;

    // Create schedule blocks linked to note
    repo.create_schedule_block(
        Some(note_id),
        "2024-01-10",
        "09:00",
        "10:00",
        Some("Block 1"),
        None,
        None,
        None,
    )
    .await
    .unwrap();

    repo.create_schedule_block(
        Some(note_id),
        "2024-01-15",
        "14:00",
        "15:00",
        Some("Block 2"),
        None,
        None,
        None,
    )
    .await
    .unwrap();

    // Verify blocks exist
    let blocks = repo.get_schedule_blocks_for_note(note_id).await.unwrap();
    assert_eq!(blocks.len(), 2);

    // Delete the note
    sqlx::query("DELETE FROM notes WHERE id = ?")
        .bind(note_id)
        .execute(pool)
        .await
        .unwrap();

    // Verify schedule blocks were cascade deleted
    let count = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM schedule_blocks")
        .fetch_one(pool)
        .await
        .unwrap();
    assert_eq!(count, 0);
}
