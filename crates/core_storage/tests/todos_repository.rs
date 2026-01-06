//! Tests for the todos repository.

mod helpers;

use core_index::ParsedTodo;
use helpers::{insert_test_note, setup_test_repo};

#[tokio::test]
async fn test_replace_todos_insert() {
    let (_pool, repo) = setup_test_repo().await;
    let pool = repo.pool();
    let note_id = insert_test_note(pool, "test.md", Some("Test Note")).await;

    // Create todos with various fields
    let todos = vec![
        ParsedTodo {
            description: "Complete project documentation".to_string(),
            raw_text: "- [ ] Complete project documentation".to_string(),
            completed: false,
            line_number: 10,
            heading_path: Some("Work > Projects".to_string()),
            context: Some("work".to_string()),
            priority: Some("high".to_string()),
            due_date: Some("2024-01-15".to_string()),
        },
        ParsedTodo {
            description: "Review pull requests".to_string(),
            raw_text: "- [ ] Review pull requests".to_string(),
            completed: false,
            line_number: 12,
            heading_path: Some("Work > Code Review".to_string()),
            context: Some("work".to_string()),
            priority: Some("medium".to_string()),
            due_date: None,
        },
        ParsedTodo {
            description: "Buy groceries".to_string(),
            raw_text: "- [x] Buy groceries".to_string(),
            completed: true,
            line_number: 15,
            heading_path: None,
            context: Some("home".to_string()),
            priority: Some("low".to_string()),
            due_date: None,
        },
    ];

    repo.replace_todos(note_id, &todos).await.unwrap();

    // Verify todos were inserted
    let stored_todos = repo.get_todos_for_note(note_id).await.unwrap();
    assert_eq!(stored_todos.len(), 3);

    // Check first todo details
    let todo1 = &stored_todos[0];
    assert_eq!(todo1.description, "Complete project documentation");
    assert!(!todo1.completed);
    assert_eq!(todo1.line_number, Some(10));
    assert_eq!(todo1.heading_path, Some("Work > Projects".to_string()));
    assert_eq!(todo1.context, Some("work".to_string()));
    assert_eq!(todo1.priority, Some("high".to_string()));
    assert_eq!(todo1.due_date, Some("2024-01-15".to_string()));
}

#[tokio::test]
async fn test_replace_todos_update() {
    let (_pool, repo) = setup_test_repo().await;
    let pool = repo.pool();
    let note_id = insert_test_note(pool, "test.md", Some("Test Note")).await;

    // Insert initial todos
    let initial_todos = vec![
        ParsedTodo {
            description: "Old task 1".to_string(),
            raw_text: "- [ ] Old task 1".to_string(),
            completed: false,
            line_number: 5,
            heading_path: None,
            context: None,
            priority: None,
            due_date: None,
        },
        ParsedTodo {
            description: "Old task 2".to_string(),
            raw_text: "- [ ] Old task 2".to_string(),
            completed: false,
            line_number: 6,
            heading_path: None,
            context: None,
            priority: None,
            due_date: None,
        },
    ];

    repo.replace_todos(note_id, &initial_todos).await.unwrap();

    // Replace with new todos
    let new_todos = vec![ParsedTodo {
        description: "New task".to_string(),
        raw_text: "- [ ] New task".to_string(),
        completed: false,
        line_number: 10,
        heading_path: None,
        context: Some("work".to_string()),
        priority: Some("high".to_string()),
        due_date: Some("2024-02-01".to_string()),
    }];

    repo.replace_todos(note_id, &new_todos).await.unwrap();

    // Verify old todos were replaced
    let stored_todos = repo.get_todos_for_note(note_id).await.unwrap();
    assert_eq!(stored_todos.len(), 1);
    assert_eq!(stored_todos[0].description, "New task");
    assert_eq!(stored_todos[0].context, Some("work".to_string()));
}

#[tokio::test]
async fn test_get_incomplete_todos() {
    let (_pool, repo) = setup_test_repo().await;
    let pool = repo.pool();

    // Create multiple notes with mixed completed/incomplete todos
    let note1 = insert_test_note(pool, "note1.md", Some("Note 1")).await;
    let note2 = insert_test_note(pool, "note2.md", Some("Note 2")).await;

    let todos_note1 = vec![
        ParsedTodo {
            description: "Incomplete task 1".to_string(),
            raw_text: "- [ ] Incomplete task 1".to_string(),
            completed: false,
            line_number: 5,
            heading_path: None,
            context: None,
            priority: None,
            due_date: None,
        },
        ParsedTodo {
            description: "Completed task".to_string(),
            raw_text: "- [x] Completed task".to_string(),
            completed: true,
            line_number: 6,
            heading_path: None,
            context: None,
            priority: None,
            due_date: None,
        },
    ];

    let todos_note2 = vec![ParsedTodo {
        description: "Incomplete task 2".to_string(),
        raw_text: "- [ ] Incomplete task 2".to_string(),
        completed: false,
        line_number: 3,
        heading_path: None,
        context: None,
        priority: None,
        due_date: None,
    }];

    repo.replace_todos(note1, &todos_note1).await.unwrap();
    repo.replace_todos(note2, &todos_note2).await.unwrap();

    // Get only incomplete todos
    let incomplete = repo.get_incomplete_todos().await.unwrap();
    assert_eq!(incomplete.len(), 2);

    // Verify all returned todos are incomplete
    for todo in &incomplete {
        assert!(!todo.completed);
    }

    // Verify the correct descriptions
    let descriptions: Vec<String> = incomplete.iter().map(|t| t.description.clone()).collect();
    assert!(descriptions.contains(&"Incomplete task 1".to_string()));
    assert!(descriptions.contains(&"Incomplete task 2".to_string()));
}

#[tokio::test]
async fn test_update_todo_completion() {
    let (_pool, repo) = setup_test_repo().await;
    let pool = repo.pool();
    let note_id = insert_test_note(pool, "test.md", Some("Test Note")).await;

    // Create an incomplete todo
    let todos = vec![ParsedTodo {
        description: "Task to complete".to_string(),
        raw_text: "- [ ] Task to complete".to_string(),
        completed: false,
        line_number: 5,
        heading_path: None,
        context: None,
        priority: None,
        due_date: None,
    }];

    repo.replace_todos(note_id, &todos).await.unwrap();

    let stored_todos = repo.get_todos_for_note(note_id).await.unwrap();
    let todo_id = stored_todos[0].id;

    // Mark as completed
    repo.update_todo_completion(todo_id, true).await.unwrap();

    let updated_todo = repo.get_todo(todo_id).await.unwrap().unwrap();
    assert!(updated_todo.completed);
    assert!(updated_todo.completed_at.is_some());

    // Mark as incomplete again
    repo.update_todo_completion(todo_id, false).await.unwrap();

    let updated_todo = repo.get_todo(todo_id).await.unwrap().unwrap();
    assert!(!updated_todo.completed);
    assert!(updated_todo.completed_at.is_none());
}

#[tokio::test]
async fn test_query_tasks_with_filters() {
    let (_pool, repo) = setup_test_repo().await;
    let pool = repo.pool();

    // Create notes with todos with different priorities and contexts
    let note1 = insert_test_note(pool, "note1.md", Some("Note 1")).await;
    let note2 = insert_test_note(pool, "note2.md", Some("Note 2")).await;

    let todos_note1 = vec![
        ParsedTodo {
            description: "High priority work task".to_string(),
            raw_text: "- [ ] High priority work task".to_string(),
            completed: false,
            line_number: 5,
            heading_path: None,
            context: Some("work".to_string()),
            priority: Some("high".to_string()),
            due_date: Some("2024-01-20".to_string()),
        },
        ParsedTodo {
            description: "Low priority home task".to_string(),
            raw_text: "- [ ] Low priority home task".to_string(),
            completed: false,
            line_number: 6,
            heading_path: None,
            context: Some("home".to_string()),
            priority: Some("low".to_string()),
            due_date: None,
        },
    ];

    let todos_note2 = vec![ParsedTodo {
        description: "Medium priority work task".to_string(),
        raw_text: "- [ ] Medium priority work task".to_string(),
        completed: false,
        line_number: 3,
        heading_path: None,
        context: Some("work".to_string()),
        priority: Some("medium".to_string()),
        due_date: Some("2024-01-25".to_string()),
    }];

    repo.replace_todos(note1, &todos_note1).await.unwrap();
    repo.replace_todos(note2, &todos_note2).await.unwrap();

    // Query by priority
    use shared_types::TaskQuery;
    let query = TaskQuery {
        completed: Some(false),
        context: None,
        priority: Some("high".to_string()),
        due_from: None,
        due_to: None,
        property_filter: None,
        limit: Some(10),
    };

    let results = repo.query_tasks(&query).await.unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].todo.description, "High priority work task");

    // Query by context
    let query = TaskQuery {
        completed: Some(false),
        context: Some("work".to_string()),
        priority: None,
        due_from: None,
        due_to: None,
        property_filter: None,
        limit: Some(10),
    };

    let results = repo.query_tasks(&query).await.unwrap();
    assert_eq!(results.len(), 2);

    // Query by due date range
    let query = TaskQuery {
        completed: Some(false),
        context: None,
        priority: None,
        due_from: Some("2024-01-15".to_string()),
        due_to: Some("2024-01-22".to_string()),
        property_filter: None,
        limit: Some(10),
    };

    let results = repo.query_tasks(&query).await.unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].todo.description, "High priority work task");
}

#[tokio::test]
async fn test_get_todos_for_nonexistent_note() {
    let (_pool, repo) = setup_test_repo().await;

    // Query todos for a note that doesn't exist
    let todos = repo.get_todos_for_note(99999).await.unwrap();

    // Should return empty Vec, not error
    assert_eq!(todos.len(), 0);
}

#[tokio::test]
async fn test_query_tasks_with_property_filter() {
    let (_pool, repo) = setup_test_repo().await;
    let pool = repo.pool();

    // Create note with property
    let note_id = insert_test_note(pool, "test.md", Some("Test Note")).await;
    repo.set_property(note_id, "status", Some("active"), Some("text"))
        .await
        .unwrap();

    // Add todos to the note
    let todos = vec![ParsedTodo {
        description: "Task in active note".to_string(),
        raw_text: "- [ ] Task in active note".to_string(),
        completed: false,
        line_number: 5,
        heading_path: None,
        context: None,
        priority: Some("high".to_string()),
        due_date: None,
    }];

    repo.replace_todos(note_id, &todos).await.unwrap();

    // Query with property filter
    use shared_types::TaskQuery;
    let query = TaskQuery {
        completed: Some(false),
        context: None,
        priority: None,
        due_from: None,
        due_to: None,
        property_filter: Some("status=active".to_string()),
        limit: Some(10),
    };

    let results = repo.query_tasks(&query).await.unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].todo.description, "Task in active note");

    // Verify note properties are included
    assert_eq!(results[0].note_properties.len(), 1);
    assert_eq!(results[0].note_properties[0].key, "status");
    assert_eq!(results[0].note_properties[0].value, Some("active".to_string()));
}

#[tokio::test]
async fn test_cascade_delete() {
    let (_pool, repo) = setup_test_repo().await;
    let pool = repo.pool();
    let note_id = insert_test_note(pool, "test.md", Some("Test Note")).await;

    // Add todos to the note
    let todos = vec![
        ParsedTodo {
            description: "Task 1".to_string(),
            raw_text: "- [ ] Task 1".to_string(),
            completed: false,
            line_number: 5,
            heading_path: None,
            context: None,
            priority: None,
            due_date: None,
        },
        ParsedTodo {
            description: "Task 2".to_string(),
            raw_text: "- [ ] Task 2".to_string(),
            completed: false,
            line_number: 6,
            heading_path: None,
            context: None,
            priority: None,
            due_date: None,
        },
    ];

    repo.replace_todos(note_id, &todos).await.unwrap();

    // Verify todos exist
    let stored_todos = repo.get_todos_for_note(note_id).await.unwrap();
    assert_eq!(stored_todos.len(), 2);

    // Delete the note
    sqlx::query("DELETE FROM notes WHERE id = ?")
        .bind(note_id)
        .execute(pool)
        .await
        .unwrap();

    // Verify todos were cascade deleted
    let count = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM todos")
        .fetch_one(pool)
        .await
        .unwrap();
    assert_eq!(count, 0);
}
