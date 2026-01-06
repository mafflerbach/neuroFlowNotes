//! Tests for the queries repository.

mod helpers;

use core_index::markdown::ParsedTodo;
use helpers::{insert_test_note, insert_test_property, insert_test_tag, setup_test_repo};
use shared_types::{
    FilterMatchMode, PropertyFilter, PropertyOperator, QueryRequest, QueryResultType,
};

#[tokio::test]
async fn test_run_query_no_filters() {
    let (_pool, repo) = setup_test_repo().await;
    let pool = repo.pool();

    // Create test notes
    insert_test_note(pool, "note1.md", Some("Note 1")).await;
    insert_test_note(pool, "note2.md", Some("Note 2")).await;
    insert_test_note(pool, "note3.md", Some("Note 3")).await;

    // Query with no filters
    let request = QueryRequest {
        filters: vec![],
        match_mode: FilterMatchMode::All,
        result_type: QueryResultType::Notes,
        include_completed: false,
        limit: Some(100),
    };

    let response = repo.run_query(&request).await.unwrap();

    // Should return all notes
    assert_eq!(response.total_count, 3);
    assert_eq!(response.results.len(), 3);
}

#[tokio::test]
async fn test_run_query_property_equals() {
    let (_pool, repo) = setup_test_repo().await;
    let pool = repo.pool();

    // Create notes with different status values
    let note1 = insert_test_note(pool, "active.md", Some("Active Note")).await;
    insert_test_property(pool, note1, "status", "active", "text").await;

    let note2 = insert_test_note(pool, "done.md", Some("Done Note")).await;
    insert_test_property(pool, note2, "status", "done", "text").await;

    let _note3 = insert_test_note(pool, "no-status.md", Some("No Status")).await;

    // Query for status = active
    let request = QueryRequest {
        filters: vec![PropertyFilter {
            key: "status".to_string(),
            operator: PropertyOperator::Equals,
            value: Some("active".to_string()),
        }],
        match_mode: FilterMatchMode::All,
        result_type: QueryResultType::Notes,
        include_completed: false,
        limit: Some(100),
    };

    let response = repo.run_query(&request).await.unwrap();

    // Should return only note1
    assert_eq!(response.total_count, 1);
    assert_eq!(response.results.len(), 1);
    assert_eq!(response.results[0].note.as_ref().unwrap().path, "active.md");
}

#[tokio::test]
async fn test_run_query_property_exists() {
    let (_pool, repo) = setup_test_repo().await;
    let pool = repo.pool();

    // Create notes with and without priority
    let note1 = insert_test_note(pool, "with-priority.md", Some("With Priority")).await;
    insert_test_property(pool, note1, "priority", "high", "text").await;

    let _note2 = insert_test_note(pool, "without-priority.md", Some("Without Priority")).await;

    // Query for notes with priority property
    let request = QueryRequest {
        filters: vec![PropertyFilter {
            key: "priority".to_string(),
            operator: PropertyOperator::Exists,
            value: None,
        }],
        match_mode: FilterMatchMode::All,
        result_type: QueryResultType::Notes,
        include_completed: false,
        limit: Some(100),
    };

    let response = repo.run_query(&request).await.unwrap();

    // Should return only note1
    assert_eq!(response.total_count, 1);
    assert_eq!(response.results[0].note.as_ref().unwrap().path, "with-priority.md");
}

#[tokio::test]
async fn test_run_query_property_not_exists() {
    let (_pool, repo) = setup_test_repo().await;
    let pool = repo.pool();

    // Create notes with and without priority
    let note1 = insert_test_note(pool, "with-priority.md", Some("With Priority")).await;
    insert_test_property(pool, note1, "priority", "high", "text").await;

    let _note2 = insert_test_note(pool, "without-priority.md", Some("Without Priority")).await;

    // Query for notes WITHOUT priority property
    let request = QueryRequest {
        filters: vec![PropertyFilter {
            key: "priority".to_string(),
            operator: PropertyOperator::NotExists,
            value: None,
        }],
        match_mode: FilterMatchMode::All,
        result_type: QueryResultType::Notes,
        include_completed: false,
        limit: Some(100),
    };

    let response = repo.run_query(&request).await.unwrap();

    // Should return only note2
    assert_eq!(response.total_count, 1);
    assert_eq!(response.results[0].note.as_ref().unwrap().path, "without-priority.md");
}

#[tokio::test]
async fn test_run_query_match_mode_all() {
    let (_pool, repo) = setup_test_repo().await;
    let pool = repo.pool();

    // Create notes with different property combinations
    let note1 = insert_test_note(pool, "both.md", Some("Both")).await;
    insert_test_property(pool, note1, "status", "active", "text").await;
    insert_test_property(pool, note1, "priority", "high", "text").await;

    let note2 = insert_test_note(pool, "status-only.md", Some("Status Only")).await;
    insert_test_property(pool, note2, "status", "active", "text").await;

    let note3 = insert_test_note(pool, "priority-only.md", Some("Priority Only")).await;
    insert_test_property(pool, note3, "priority", "high", "text").await;

    // Query for status=active AND priority=high
    let request = QueryRequest {
        filters: vec![
            PropertyFilter {
                key: "status".to_string(),
                operator: PropertyOperator::Equals,
                value: Some("active".to_string()),
            },
            PropertyFilter {
                key: "priority".to_string(),
                operator: PropertyOperator::Equals,
                value: Some("high".to_string()),
            },
        ],
        match_mode: FilterMatchMode::All,
        result_type: QueryResultType::Notes,
        include_completed: false,
        limit: Some(100),
    };

    let response = repo.run_query(&request).await.unwrap();

    // Should return only note1 (has both properties)
    assert_eq!(response.total_count, 1);
    assert_eq!(response.results[0].note.as_ref().unwrap().path, "both.md");
}

#[tokio::test]
async fn test_run_query_match_mode_any() {
    let (_pool, repo) = setup_test_repo().await;
    let pool = repo.pool();

    // Create notes with different property combinations
    let note1 = insert_test_note(pool, "both.md", Some("Both")).await;
    insert_test_property(pool, note1, "status", "active", "text").await;
    insert_test_property(pool, note1, "priority", "high", "text").await;

    let note2 = insert_test_note(pool, "status-only.md", Some("Status Only")).await;
    insert_test_property(pool, note2, "status", "active", "text").await;

    let note3 = insert_test_note(pool, "priority-only.md", Some("Priority Only")).await;
    insert_test_property(pool, note3, "priority", "high", "text").await;

    let _note4 = insert_test_note(pool, "neither.md", Some("Neither")).await;

    // Query for status=active OR priority=high
    let request = QueryRequest {
        filters: vec![
            PropertyFilter {
                key: "status".to_string(),
                operator: PropertyOperator::Equals,
                value: Some("active".to_string()),
            },
            PropertyFilter {
                key: "priority".to_string(),
                operator: PropertyOperator::Equals,
                value: Some("high".to_string()),
            },
        ],
        match_mode: FilterMatchMode::Any,
        result_type: QueryResultType::Notes,
        include_completed: false,
        limit: Some(100),
    };

    let response = repo.run_query(&request).await.unwrap();

    // Should return note1, note2, and note3 (any with status OR priority)
    assert_eq!(response.total_count, 3);
}

#[tokio::test]
async fn test_run_query_tags_filter() {
    let (_pool, repo) = setup_test_repo().await;
    let pool = repo.pool();

    // Create notes with different tags
    let note1 = insert_test_note(pool, "work.md", Some("Work Note")).await;
    insert_test_tag(pool, note1, "work").await;

    let note2 = insert_test_note(pool, "personal.md", Some("Personal Note")).await;
    insert_test_tag(pool, note2, "personal").await;

    let _note3 = insert_test_note(pool, "no-tags.md", Some("No Tags")).await;

    // Query for notes with #work tag
    let request = QueryRequest {
        filters: vec![PropertyFilter {
            key: "_tags".to_string(),
            operator: PropertyOperator::Equals,
            value: Some("work".to_string()),
        }],
        match_mode: FilterMatchMode::All,
        result_type: QueryResultType::Notes,
        include_completed: false,
        limit: Some(100),
    };

    let response = repo.run_query(&request).await.unwrap();

    // Should return only note1
    assert_eq!(response.total_count, 1);
    assert_eq!(response.results[0].note.as_ref().unwrap().path, "work.md");
}

#[tokio::test]
async fn test_run_query_tags_contains_any() {
    let (_pool, repo) = setup_test_repo().await;
    let pool = repo.pool();

    // Create notes with different tags
    let note1 = insert_test_note(pool, "work.md", Some("Work Note")).await;
    insert_test_tag(pool, note1, "work").await;

    let note2 = insert_test_note(pool, "urgent.md", Some("Urgent Note")).await;
    insert_test_tag(pool, note2, "urgent").await;

    let note3 = insert_test_note(pool, "both.md", Some("Both Tags")).await;
    insert_test_tag(pool, note3, "work").await;
    insert_test_tag(pool, note3, "urgent").await;

    let _note4 = insert_test_note(pool, "other.md", Some("Other")).await;
    insert_test_tag(pool, _note4, "personal").await;

    // Query for notes with #work OR #urgent
    let request = QueryRequest {
        filters: vec![PropertyFilter {
            key: "_tags".to_string(),
            operator: PropertyOperator::ContainsAny,
            value: Some("work,urgent".to_string()),
        }],
        match_mode: FilterMatchMode::All,
        result_type: QueryResultType::Notes,
        include_completed: false,
        limit: Some(100),
    };

    let response = repo.run_query(&request).await.unwrap();

    // Should return note1, note2, and note3
    assert_eq!(response.total_count, 3);
}

#[tokio::test]
async fn test_run_query_tags_contains_all() {
    let (_pool, repo) = setup_test_repo().await;
    let pool = repo.pool();

    // Create notes with different tag combinations
    let note1 = insert_test_note(pool, "work-only.md", Some("Work Only")).await;
    insert_test_tag(pool, note1, "work").await;

    let note2 = insert_test_note(pool, "urgent-only.md", Some("Urgent Only")).await;
    insert_test_tag(pool, note2, "urgent").await;

    let note3 = insert_test_note(pool, "both.md", Some("Both Tags")).await;
    insert_test_tag(pool, note3, "work").await;
    insert_test_tag(pool, note3, "urgent").await;

    // Query for notes with BOTH #work AND #urgent
    let request = QueryRequest {
        filters: vec![PropertyFilter {
            key: "_tags".to_string(),
            operator: PropertyOperator::ContainsAll,
            value: Some("work,urgent".to_string()),
        }],
        match_mode: FilterMatchMode::All,
        result_type: QueryResultType::Notes,
        include_completed: false,
        limit: Some(100),
    };

    let response = repo.run_query(&request).await.unwrap();

    // Should return only note3
    assert_eq!(response.total_count, 1);
    assert_eq!(response.results[0].note.as_ref().unwrap().path, "both.md");
}

#[tokio::test]
async fn test_run_query_path_filter() {
    let (_pool, repo) = setup_test_repo().await;
    let pool = repo.pool();

    // Create notes in different folders
    insert_test_note(pool, "work/project1.md", Some("Project 1")).await;
    insert_test_note(pool, "work/project2.md", Some("Project 2")).await;
    insert_test_note(pool, "personal/journal.md", Some("Journal")).await;

    // Query for notes in work/ folder
    let request = QueryRequest {
        filters: vec![PropertyFilter {
            key: "_path".to_string(),
            operator: PropertyOperator::StartsWith,
            value: Some("work".to_string()),
        }],
        match_mode: FilterMatchMode::All,
        result_type: QueryResultType::Notes,
        include_completed: false,
        limit: Some(100),
    };

    let response = repo.run_query(&request).await.unwrap();

    // Should return both work notes
    assert_eq!(response.total_count, 2);
    assert!(response.results[0].note.as_ref().unwrap().path.starts_with("work/"));
    assert!(response.results[1].note.as_ref().unwrap().path.starts_with("work/"));
}

#[tokio::test]
async fn test_run_query_property_contains() {
    let (_pool, repo) = setup_test_repo().await;
    let pool = repo.pool();

    // Create notes with descriptions
    let note1 = insert_test_note(pool, "rust-note.md", Some("Rust Note")).await;
    insert_test_property(pool, note1, "description", "Learn Rust programming", "text").await;

    let note2 = insert_test_note(pool, "python-note.md", Some("Python Note")).await;
    insert_test_property(pool, note2, "description", "Python tutorial", "text").await;

    // Query for notes with "Rust" in description
    let request = QueryRequest {
        filters: vec![PropertyFilter {
            key: "description".to_string(),
            operator: PropertyOperator::Contains,
            value: Some("Rust".to_string()),
        }],
        match_mode: FilterMatchMode::All,
        result_type: QueryResultType::Notes,
        include_completed: false,
        limit: Some(100),
    };

    let response = repo.run_query(&request).await.unwrap();

    // Should return only note1
    assert_eq!(response.total_count, 1);
    assert_eq!(response.results[0].note.as_ref().unwrap().path, "rust-note.md");
}

#[tokio::test]
async fn test_run_query_result_type_tasks() {
    let (_pool, repo) = setup_test_repo().await;
    let pool = repo.pool();

    // Create note with property and todos
    let note1 = insert_test_note(pool, "project.md", Some("Project")).await;
    insert_test_property(pool, note1, "status", "active", "text").await;

    // Add todos to the note
    let todos = vec![
        ParsedTodo {
            description: "Task 1".to_string(),
            raw_text: "- [ ] Task 1".to_string(),
            completed: false,
            line_number: 1,
            heading_path: None,
            context: None,
            priority: None,
            due_date: None,
        },
        ParsedTodo {
            description: "Task 2".to_string(),
            raw_text: "- [ ] Task 2".to_string(),
            completed: false,
            line_number: 2,
            heading_path: None,
            context: None,
            priority: None,
            due_date: None,
        },
    ];
    repo.replace_todos(note1, &todos).await.unwrap();

    // Query for tasks from notes with status=active
    let request = QueryRequest {
        filters: vec![PropertyFilter {
            key: "status".to_string(),
            operator: PropertyOperator::Equals,
            value: Some("active".to_string()),
        }],
        match_mode: FilterMatchMode::All,
        result_type: QueryResultType::Tasks,
        include_completed: false,
        limit: Some(100),
    };

    let response = repo.run_query(&request).await.unwrap();

    // Should return 2 tasks
    assert_eq!(response.total_count, 2);
    assert_eq!(response.results[0].item_type, "task");
    assert_eq!(response.results[1].item_type, "task");
    assert!(response.results[0].task.is_some());
    assert!(response.results[1].task.is_some());
}

#[tokio::test]
async fn test_run_query_date_operators() {
    let (_pool, repo) = setup_test_repo().await;
    let pool = repo.pool();

    // Create notes with different due dates
    let note1 = insert_test_note(pool, "past.md", Some("Past")).await;
    insert_test_property(pool, note1, "due_date", "2024-01-10", "text").await;

    let note2 = insert_test_note(pool, "current.md", Some("Current")).await;
    insert_test_property(pool, note2, "due_date", "2024-01-15", "text").await;

    let note3 = insert_test_note(pool, "future.md", Some("Future")).await;
    insert_test_property(pool, note3, "due_date", "2024-01-20", "text").await;

    // Query for notes with due_date before 2024-01-15
    let request = QueryRequest {
        filters: vec![PropertyFilter {
            key: "due_date".to_string(),
            operator: PropertyOperator::DateBefore,
            value: Some("2024-01-15".to_string()),
        }],
        match_mode: FilterMatchMode::All,
        result_type: QueryResultType::Notes,
        include_completed: false,
        limit: Some(100),
    };

    let response = repo.run_query(&request).await.unwrap();

    // Should return only note1
    assert_eq!(response.total_count, 1);
    assert_eq!(response.results[0].note.as_ref().unwrap().path, "past.md");

    // Query for notes with due_date on or after 2024-01-15
    let request2 = QueryRequest {
        filters: vec![PropertyFilter {
            key: "due_date".to_string(),
            operator: PropertyOperator::DateOnOrAfter,
            value: Some("2024-01-15".to_string()),
        }],
        match_mode: FilterMatchMode::All,
        result_type: QueryResultType::Notes,
        include_completed: false,
        limit: Some(100),
    };

    let response2 = repo.run_query(&request2).await.unwrap();

    // Should return note2 and note3
    assert_eq!(response2.total_count, 2);
}
