use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Group {
    pub id: Uuid,
    pub name: String,
    pub created_at: DateTime<Utc>,
}

impl Group {
    pub fn new(name: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            created_at: Utc::now(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Todo {
    pub id: Uuid,
    pub group_id: Uuid,
    pub title: String,
    pub completed: bool,
    pub created_at: DateTime<Utc>,
    #[serde(default)]
    pub completed_at: Option<NaiveDate>,
}

impl Todo {
    pub fn new(title: String, group_id: Uuid) -> Self {
        Self {
            id: Uuid::new_v4(),
            group_id,
            title,
            completed: false,
            created_at: Utc::now(),
            completed_at: None,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum Filter {
    All,
    Open,
    Done,
}

impl Filter {
    pub fn next(self) -> Self {
        match self {
            Filter::All => Filter::Open,
            Filter::Open => Filter::Done,
            Filter::Done => Filter::All,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Filter::All => "all",
            Filter::Open => "open",
            Filter::Done => "done",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Normal,
    AddingTodo,
    EditingTodo,
    ConfirmDeleteTodo,
    AddingGroup,
    EditingGroup,
    ConfirmDeleteGroup,
}
