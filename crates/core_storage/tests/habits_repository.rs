//! Tests for the habits repository.

mod helpers;

use helpers::setup_test_repo;
use shared_types::{CreateHabitRequest, HabitDateRange, HabitTableOrientation, HabitTrackerQuery, HabitType, HabitViewType, LogHabitEntryRequest, UpdateHabitEntryRequest, UpdateHabitRequest};

#[tokio::test]
async fn test_create_habit() {
    let (_pool, repo) = setup_test_repo().await;

    let request = CreateHabitRequest {
        name: "Exercise".to_string(),
        description: Some("Daily workout routine".to_string()),
        habit_type: HabitType::Boolean,
        unit: None,
        color: Some("#3b82f6".to_string()),
        target_value: None,
    };

    let habit_id = repo.create_habit(&request).await.unwrap();
    assert!(habit_id > 0);

    // Verify habit was created
    let habit = repo.get_habit(habit_id).await.unwrap().unwrap();
    assert_eq!(habit.name, "Exercise");
    assert_eq!(habit.description, Some("Daily workout routine".to_string()));
    assert_eq!(habit.habit_type, HabitType::Boolean);
    assert_eq!(habit.color, Some("#3b82f6".to_string()));
    assert!(!habit.archived);
}

#[tokio::test]
async fn test_create_habit_unarchive_existing() {
    let (_pool, repo) = setup_test_repo().await;

    // Create and archive a habit
    let request = CreateHabitRequest {
        name: "Meditation".to_string(),
        description: Some("Original description".to_string()),
        habit_type: HabitType::Boolean,
        unit: None,
        color: Some("#ff0000".to_string()),
        target_value: None,
    };

    let habit_id = repo.create_habit(&request).await.unwrap();
    repo.archive_habit(habit_id).await.unwrap();

    // Verify it's archived
    let habit = repo.get_habit(habit_id).await.unwrap().unwrap();
    assert!(habit.archived);

    // Create habit with same name but different attributes
    let new_request = CreateHabitRequest {
        name: "Meditation".to_string(), // Same name
        description: Some("Updated description".to_string()),
        habit_type: HabitType::Number,
        unit: Some("minutes".to_string()),
        color: Some("#00ff00".to_string()),
        target_value: Some(20.0),
    };

    let new_habit_id = repo.create_habit(&new_request).await.unwrap();

    // Should return same ID (unarchived existing)
    assert_eq!(new_habit_id, habit_id);

    // Verify it was unarchived and updated
    let habit = repo.get_habit(habit_id).await.unwrap().unwrap();
    assert!(!habit.archived);
    assert_eq!(habit.description, Some("Updated description".to_string()));
    assert_eq!(habit.habit_type, HabitType::Number);
    assert_eq!(habit.unit, Some("minutes".to_string()));
    assert_eq!(habit.color, Some("#00ff00".to_string()));
    assert_eq!(habit.target_value, Some(20.0));
}

#[tokio::test]
async fn test_get_habit_by_name_case_insensitive() {
    let (_pool, repo) = setup_test_repo().await;

    let request = CreateHabitRequest {
        name: "Reading".to_string(),
        description: None,
        habit_type: HabitType::Boolean,
        unit: None,
        color: None,
        target_value: None,
    };

    let habit_id = repo.create_habit(&request).await.unwrap();

    // Search with different cases
    let habit = repo.get_habit_by_name("reading").await.unwrap().unwrap();
    assert_eq!(habit.id, habit_id);

    let habit = repo.get_habit_by_name("READING").await.unwrap().unwrap();
    assert_eq!(habit.id, habit_id);

    let habit = repo.get_habit_by_name("ReAdInG").await.unwrap().unwrap();
    assert_eq!(habit.id, habit_id);

    // Non-existent habit
    let not_found = repo.get_habit_by_name("nonexistent").await.unwrap();
    assert!(not_found.is_none());
}

#[tokio::test]
async fn test_list_habits_with_archived_filter() {
    let (_pool, repo) = setup_test_repo().await;

    // Create 3 habits
    let habit1 = repo.create_habit(&CreateHabitRequest {
        name: "Habit 1".to_string(),
        description: None,
        habit_type: HabitType::Boolean,
        unit: None,
        color: None,
        target_value: None,
    }).await.unwrap();

    let habit2 = repo.create_habit(&CreateHabitRequest {
        name: "Habit 2".to_string(),
        description: None,
        habit_type: HabitType::Boolean,
        unit: None,
        color: None,
        target_value: None,
    }).await.unwrap();

    let habit3 = repo.create_habit(&CreateHabitRequest {
        name: "Habit 3".to_string(),
        description: None,
        habit_type: HabitType::Boolean,
        unit: None,
        color: None,
        target_value: None,
    }).await.unwrap();

    // Archive one habit
    repo.archive_habit(habit2).await.unwrap();

    // List without archived
    let habits = repo.list_habits(false).await.unwrap();
    assert_eq!(habits.len(), 2);
    let ids: Vec<i64> = habits.iter().map(|h| h.id).collect();
    assert!(ids.contains(&habit1));
    assert!(ids.contains(&habit3));
    assert!(!ids.contains(&habit2));

    // List with archived
    let habits = repo.list_habits(true).await.unwrap();
    assert_eq!(habits.len(), 3);
}

#[tokio::test]
async fn test_update_habit_partial() {
    let (_pool, repo) = setup_test_repo().await;

    let habit_id = repo.create_habit(&CreateHabitRequest {
        name: "Original Name".to_string(),
        description: Some("Original Description".to_string()),
        habit_type: HabitType::Boolean,
        unit: None,
        color: Some("#ff0000".to_string()),
        target_value: None,
    }).await.unwrap();

    // Update only name
    repo.update_habit(&UpdateHabitRequest {
        id: habit_id,
        name: Some("Updated Name".to_string()),
        description: None,
        habit_type: None,
        unit: None,
        color: None,
        target_value: None,
        archived: None,
        sort_order: None,
    }).await.unwrap();

    let habit = repo.get_habit(habit_id).await.unwrap().unwrap();
    assert_eq!(habit.name, "Updated Name");
    assert_eq!(habit.description, Some("Original Description".to_string())); // Unchanged
    assert_eq!(habit.color, Some("#ff0000".to_string())); // Unchanged

    // Update only color
    repo.update_habit(&UpdateHabitRequest {
        id: habit_id,
        name: None,
        description: None,
        habit_type: None,
        unit: None,
        color: Some("#00ff00".to_string()),
        target_value: None,
        archived: None,
        sort_order: None,
    }).await.unwrap();

    let habit = repo.get_habit(habit_id).await.unwrap().unwrap();
    assert_eq!(habit.name, "Updated Name"); // Still updated from before
    assert_eq!(habit.color, Some("#00ff00".to_string())); // Now updated
}

#[tokio::test]
async fn test_log_habit_entry() {
    let (_pool, repo) = setup_test_repo().await;

    let habit_id = repo.create_habit(&CreateHabitRequest {
        name: "Steps".to_string(),
        description: None,
        habit_type: HabitType::Number,
        unit: Some("steps".to_string()),
        color: None,
        target_value: Some(10000.0),
    }).await.unwrap();

    // Log an entry
    let entry_id = repo.log_habit_entry(&LogHabitEntryRequest {
        habit_id,
        date: "2024-01-15".to_string(),
        time: Some("14:30".to_string()),
        value: "8500".to_string(),
        notes: Some("Good walk today".to_string()),
    }).await.unwrap();

    assert!(entry_id > 0);

    // Verify entry was created
    let entries = repo.get_habit_entries(habit_id, "2024-01-15", "2024-01-15").await.unwrap();
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].habit_id, habit_id);
    assert_eq!(entries[0].date, "2024-01-15");
    assert_eq!(entries[0].time, Some("14:30".to_string()));
    assert_eq!(entries[0].value, "8500");
    assert_eq!(entries[0].notes, Some("Good walk today".to_string()));
}

#[tokio::test]
async fn test_get_habit_entries_date_range() {
    let (_pool, repo) = setup_test_repo().await;

    let habit_id = repo.create_habit(&CreateHabitRequest {
        name: "Running".to_string(),
        description: None,
        habit_type: HabitType::Number,
        unit: Some("km".to_string()),
        color: None,
        target_value: Some(5.0),
    }).await.unwrap();

    // Log entries on different days
    repo.log_habit_entry(&LogHabitEntryRequest {
        habit_id,
        date: "2024-01-05".to_string(),
        time: None,
        value: "3.5".to_string(),
        notes: None,
    }).await.unwrap();

    repo.log_habit_entry(&LogHabitEntryRequest {
        habit_id,
        date: "2024-01-10".to_string(),
        time: None,
        value: "5.2".to_string(),
        notes: None,
    }).await.unwrap();

    repo.log_habit_entry(&LogHabitEntryRequest {
        habit_id,
        date: "2024-01-15".to_string(),
        time: None,
        value: "6.1".to_string(),
        notes: None,
    }).await.unwrap();

    repo.log_habit_entry(&LogHabitEntryRequest {
        habit_id,
        date: "2024-01-20".to_string(),
        time: None,
        value: "4.8".to_string(),
        notes: None,
    }).await.unwrap();

    // Query range Jan 8-17
    let entries = repo.get_habit_entries(habit_id, "2024-01-08", "2024-01-17").await.unwrap();
    assert_eq!(entries.len(), 2);
    assert_eq!(entries[0].date, "2024-01-10");
    assert_eq!(entries[0].value, "5.2");
    assert_eq!(entries[1].date, "2024-01-15");
    assert_eq!(entries[1].value, "6.1");
}

#[tokio::test]
async fn test_toggle_habit_for_date_on_off() {
    let (_pool, repo) = setup_test_repo().await;

    let habit_id = repo.create_habit(&CreateHabitRequest {
        name: "Floss".to_string(),
        description: None,
        habit_type: HabitType::Boolean,
        unit: None,
        color: None,
        target_value: None,
    }).await.unwrap();

    // Toggle ON
    let result = repo.toggle_habit_for_date(habit_id, "2024-01-15").await.unwrap();
    assert!(result); // Should return true (now ON)

    // Verify entry exists
    let entries = repo.get_habit_entries(habit_id, "2024-01-15", "2024-01-15").await.unwrap();
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].value, "true");

    // Toggle OFF
    let result = repo.toggle_habit_for_date(habit_id, "2024-01-15").await.unwrap();
    assert!(!result); // Should return false (now OFF)

    // Verify entry was deleted
    let entries = repo.get_habit_entries(habit_id, "2024-01-15", "2024-01-15").await.unwrap();
    assert_eq!(entries.len(), 0);
}

#[tokio::test]
async fn test_update_habit_entry() {
    let (_pool, repo) = setup_test_repo().await;

    let habit_id = repo.create_habit(&CreateHabitRequest {
        name: "Water Intake".to_string(),
        description: None,
        habit_type: HabitType::Number,
        unit: Some("glasses".to_string()),
        color: None,
        target_value: Some(8.0),
    }).await.unwrap();

    let entry_id = repo.log_habit_entry(&LogHabitEntryRequest {
        habit_id,
        date: "2024-01-15".to_string(),
        time: Some("10:00".to_string()),
        value: "3".to_string(),
        notes: Some("Morning".to_string()),
    }).await.unwrap();

    // Update value and notes
    repo.update_habit_entry(&UpdateHabitEntryRequest {
        id: entry_id,
        value: Some("5".to_string()),
        notes: Some("Morning + Afternoon".to_string()),
        time: Some("15:00".to_string()),
    }).await.unwrap();

    // Verify updates
    let entries = repo.get_habit_entries(habit_id, "2024-01-15", "2024-01-15").await.unwrap();
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].value, "5");
    assert_eq!(entries[0].notes, Some("Morning + Afternoon".to_string()));
    assert_eq!(entries[0].time, Some("15:00".to_string()));
}

#[tokio::test]
async fn test_execute_habit_tracker_query_last_7_days() {
    let (_pool, repo) = setup_test_repo().await;

    let habit_id = repo.create_habit(&CreateHabitRequest {
        name: "Journal".to_string(),
        description: None,
        habit_type: HabitType::Boolean,
        unit: None,
        color: None,
        target_value: None,
    }).await.unwrap();

    // Log entries for last 10 days from reference date
    let reference_date = "2024-01-15";
    
    // Days -9 to 0 from reference
    for i in 0..10 {
        let date = format!("2024-01-{:02}", 6 + i);
        if i % 2 == 0 { // Every other day
            repo.log_habit_entry(&LogHabitEntryRequest {
                habit_id,
                date,
                time: None,
                value: "true".to_string(),
                notes: None,
            }).await.unwrap();
        }
    }

    // Query last 7 days from Jan 15
    let query = HabitTrackerQuery {
        habits: vec![],
        view: HabitViewType::Table,
        orientation: HabitTableOrientation::Horizontal,
        date_range: HabitDateRange::Last7Days,
        date: Some(reference_date.to_string()),
        start_date: None,
        end_date: None,
        editable: true,
        show_summary: true,
    };

    let response = repo.execute_habit_tracker_query(&query).await.unwrap();
    
    // Should return Jan 9-15 (7 days)
    assert_eq!(response.date_range_start, "2024-01-09");
    assert_eq!(response.date_range_end, "2024-01-15");
    assert_eq!(response.habits.len(), 1);
}

#[tokio::test]
async fn test_execute_habit_tracker_query_custom_range() {
    let (_pool, repo) = setup_test_repo().await;

    let habit_id = repo.create_habit(&CreateHabitRequest {
        name: "Exercise".to_string(),
        description: None,
        habit_type: HabitType::Boolean,
        unit: None,
        color: None,
        target_value: None,
    }).await.unwrap();

    // Log entries
    repo.log_habit_entry(&LogHabitEntryRequest {
        habit_id,
        date: "2024-01-10".to_string(),
        time: None,
        value: "true".to_string(),
        notes: None,
    }).await.unwrap();

    repo.log_habit_entry(&LogHabitEntryRequest {
        habit_id,
        date: "2024-01-15".to_string(),
        time: None,
        value: "true".to_string(),
        notes: None,
    }).await.unwrap();

    repo.log_habit_entry(&LogHabitEntryRequest {
        habit_id,
        date: "2024-01-20".to_string(),
        time: None,
        value: "true".to_string(),
        notes: None,
    }).await.unwrap();

    // Query custom range
    let query = HabitTrackerQuery {
        habits: vec![],
        view: HabitViewType::Table,
        orientation: HabitTableOrientation::Horizontal,
        date_range: HabitDateRange::Custom,
        date: None,
        start_date: Some("2024-01-12".to_string()),
        end_date: Some("2024-01-18".to_string()),
        editable: true,
        show_summary: true,
    };

    let response = repo.execute_habit_tracker_query(&query).await.unwrap();
    
    assert_eq!(response.date_range_start, "2024-01-12");
    assert_eq!(response.date_range_end, "2024-01-18");
    
    // Should only include Jan 15 entry
    assert_eq!(response.habits.len(), 1);
    assert_eq!(response.habits[0].entries_by_date.len(), 1);
    assert_eq!(response.habits[0].entries_by_date[0].0, "2024-01-15");
}

#[tokio::test]
async fn test_delete_habit_cascade_entries() {
    let (_pool, repo) = setup_test_repo().await;
    let pool = repo.pool();

    let habit_id = repo.create_habit(&CreateHabitRequest {
        name: "Habit to Delete".to_string(),
        description: None,
        habit_type: HabitType::Boolean,
        unit: None,
        color: None,
        target_value: None,
    }).await.unwrap();

    // Log multiple entries
    for i in 1..=5 {
        repo.log_habit_entry(&LogHabitEntryRequest {
            habit_id,
            date: format!("2024-01-{:02}", i + 10),
            time: None,
            value: "true".to_string(),
            notes: None,
        }).await.unwrap();
    }

    // Verify entries exist
    let entries = repo.get_habit_entries(habit_id, "2024-01-01", "2024-01-31").await.unwrap();
    assert_eq!(entries.len(), 5);

    // Delete the habit
    repo.delete_habit(habit_id).await.unwrap();

    // Verify habit is gone
    let habit = repo.get_habit(habit_id).await.unwrap();
    assert!(habit.is_none());

    // Verify entries were cascade deleted
    let count = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM habit_entries WHERE habit_id = ?")
        .bind(habit_id)
        .fetch_one(pool)
        .await
        .unwrap();
    assert_eq!(count, 0);
}
