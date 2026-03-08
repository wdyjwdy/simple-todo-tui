use std::{
    fs,
    io::{self, ErrorKind},
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::models::{Group, Todo};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AppData {
    pub groups: Vec<Group>,
    pub todos: Vec<Todo>,
}

pub fn default_data_path() -> PathBuf {
    if let Some(mut dir) = dirs::data_local_dir() {
        dir.push("simple-todo-tui");
        dir.push("todos.json");
        dir
    } else {
        PathBuf::from("todos.json")
    }
}

pub fn load_data(path: &Path) -> Result<AppData> {
    match fs::read_to_string(path) {
        Ok(raw) => {
            let data: AppData = serde_json::from_str(&raw)
                .with_context(|| format!("Failed to parse todos JSON at {}", path.display()))?;
            Ok(data)
        }
        Err(err) if err.kind() == ErrorKind::NotFound => Ok(AppData {
            groups: vec![],
            todos: vec![],
        }),
        Err(err) => {
            Err(err).with_context(|| format!("Failed to read todos from {}", path.display()))
        }
    }
}

pub fn save_data(path: &Path, data: &AppData) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).with_context(|| {
            format!(
                "Failed to create parent directory for todos at {}",
                parent.display()
            )
        })?;
    }

    let payload = serde_json::to_string_pretty(data).context("Failed to serialize todos")?;
    let tmp_path = path.with_extension("json.tmp");

    fs::write(&tmp_path, payload)
        .with_context(|| format!("Failed writing temp file {}", tmp_path.display()))?;

    match fs::rename(&tmp_path, path) {
        Ok(()) => Ok(()),
        Err(_) => {
            fs::copy(&tmp_path, path).with_context(|| {
                format!(
                    "Failed to atomically rename temp file, and fallback copy failed to {}",
                    path.display()
                )
            })?;
            fs::remove_file(&tmp_path).or_else(ignore_if_not_found)?;
            Ok(())
        }
    }
}

fn ignore_if_not_found(err: io::Error) -> io::Result<()> {
    if err.kind() == ErrorKind::NotFound {
        Ok(())
    } else {
        Err(err)
    }
}
