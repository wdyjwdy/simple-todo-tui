use std::{fs, path::PathBuf};

use chrono::Utc;
use todo::{
    models::{Group, Todo},
    storage::{self, AppData},
};
use uuid::Uuid;

fn temp_path(name: &str) -> PathBuf {
    let mut path = std::env::temp_dir();
    path.push(format!("simple_todo_tui_test_{}_{}", name, Uuid::new_v4()));
    path
}

#[test]
fn load_missing_file_returns_empty_data() {
    let path = temp_path("missing").join("todos.json");
    let data = storage::load_data(&path).expect("load should succeed for missing file");
    assert!(data.groups.is_empty());
    assert!(data.todos.is_empty());
}

#[test]
fn save_and_load_roundtrip() {
    let path = temp_path("roundtrip").join("todos.json");
    let group = Group {
        id: Uuid::new_v4(),
        name: "inbox".to_string(),
        created_at: Utc::now(),
    };
    let todo = Todo {
        id: Uuid::new_v4(),
        group_id: group.id,
        title: "hello".to_string(),
        completed: false,
        created_at: Utc::now(),
        completed_at: None,
    };
    let data = AppData {
        groups: vec![group],
        todos: vec![todo],
    };

    storage::save_data(&path, &data).expect("save should succeed");
    let loaded = storage::load_data(&path).expect("load should succeed");

    assert_eq!(loaded, data);
}

#[test]
fn corrupt_json_returns_error() {
    let path = temp_path("corrupt").join("todos.json");
    fs::create_dir_all(path.parent().expect("path has parent")).expect("dir create should work");
    fs::write(&path, "not json").expect("write should succeed");

    let result = storage::load_data(&path);
    assert!(result.is_err());
}

#[test]
fn atomic_save_produces_target_file() {
    let path = temp_path("atomic").join("todos.json");
    let group_id = Uuid::new_v4();
    let data = AppData {
        groups: vec![Group {
            id: group_id,
            name: "atomic".to_string(),
            created_at: Utc::now(),
        }],
        todos: vec![Todo {
            id: Uuid::new_v4(),
            group_id,
            title: "atomic".to_string(),
            completed: true,
            created_at: Utc::now(),
            completed_at: Some(Utc::now().date_naive()),
        }],
    };

    storage::save_data(&path, &data).expect("save should succeed");

    assert!(path.exists());
    let loaded = storage::load_data(&path).expect("load should succeed");
    assert_eq!(loaded, data);
}
