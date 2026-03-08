use std::{fs, path::PathBuf};

use chrono::Utc;
use todo::{models::Todo, storage};
use uuid::Uuid;

fn temp_path(name: &str) -> PathBuf {
    let mut path = std::env::temp_dir();
    path.push(format!("simple_todo_tui_test_{}_{}", name, Uuid::new_v4()));
    path
}

#[test]
fn load_missing_file_returns_empty() {
    let path = temp_path("missing").join("todos.json");
    let todos = storage::load_todos(&path).expect("load should succeed for missing file");
    assert!(todos.is_empty());
}

#[test]
fn save_and_load_roundtrip() {
    let path = temp_path("roundtrip").join("todos.json");
    let todo = Todo {
        id: Uuid::new_v4(),
        title: "hello".to_string(),
        completed: false,
        created_at: Utc::now(),
        completed_at: None,
    };

    storage::save_todos(&path, std::slice::from_ref(&todo)).expect("save should succeed");
    let loaded = storage::load_todos(&path).expect("load should succeed");

    assert_eq!(loaded.len(), 1);
    assert_eq!(loaded[0], todo);
}

#[test]
fn corrupt_json_returns_error() {
    let path = temp_path("corrupt").join("todos.json");
    fs::create_dir_all(path.parent().expect("path has parent")).expect("dir create should work");
    fs::write(&path, "not json").expect("write should succeed");

    let result = storage::load_todos(&path);
    assert!(result.is_err());
}

#[test]
fn atomic_save_produces_target_file() {
    let path = temp_path("atomic").join("todos.json");
    let todos = vec![Todo {
        id: Uuid::new_v4(),
        title: "atomic".to_string(),
        completed: true,
        created_at: Utc::now(),
        completed_at: Some(Utc::now().date_naive()),
    }];

    storage::save_todos(&path, &todos).expect("save should succeed");

    assert!(path.exists());
    let loaded = storage::load_todos(&path).expect("load should succeed");
    assert_eq!(loaded, todos);
}
