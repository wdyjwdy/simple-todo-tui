use crate::models::{Filter, Mode, Todo};
use chrono::Utc;
use uuid::Uuid;

#[derive(Debug)]
pub struct AppState {
    pub todos: Vec<Todo>,
    pub selected_index: usize,
    pub filter: Filter,
    pub show_help: bool,
    pub mode: Mode,
    pub input_buffer: String,
    pub status_message: Option<String>,
}

impl AppState {
    pub fn new(mut todos: Vec<Todo>) -> Self {
        for todo in &mut todos {
            if todo.completed && todo.completed_at.is_none() {
                todo.completed_at = Some(todo.created_at.date_naive());
            }
        }

        let mut state = Self {
            todos,
            selected_index: 0,
            filter: Filter::All,
            show_help: true,
            mode: Mode::Normal,
            input_buffer: String::new(),
            status_message: None,
        };
        state.normalize_selection();
        state
    }

    pub fn filtered_indices(&self) -> Vec<usize> {
        self.todos
            .iter()
            .enumerate()
            .filter_map(|(idx, todo)| {
                let matches = match self.filter {
                    Filter::All => true,
                    Filter::Active => !todo.completed,
                    Filter::Completed => todo.completed,
                };
                matches.then_some(idx)
            })
            .collect()
    }

    pub fn selected_todo_index(&self) -> Option<usize> {
        let visible = self.filtered_indices();
        visible.get(self.selected_index).copied()
    }

    pub fn selected_todo_id(&self) -> Option<Uuid> {
        self.selected_todo_index().map(|idx| self.todos[idx].id)
    }

    pub fn counts(&self) -> (usize, usize, usize) {
        let total = self.todos.len();
        let completed = self.todos.iter().filter(|t| t.completed).count();
        let active = total.saturating_sub(completed);
        (total, active, completed)
    }

    fn set_selection_by_id_or_clamp(&mut self, id: Option<Uuid>) {
        let visible = self.filtered_indices();
        if visible.is_empty() {
            self.selected_index = 0;
            return;
        }

        if let Some(id) = id {
            if let Some(pos) = visible.iter().position(|&idx| self.todos[idx].id == id) {
                self.selected_index = pos;
                return;
            }
        }

        self.selected_index = self.selected_index.min(visible.len().saturating_sub(1));
    }

    pub fn normalize_selection(&mut self) {
        self.set_selection_by_id_or_clamp(self.selected_todo_id());
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    MoveUp,
    MoveDown,
    StartAdd,
    StartEdit,
    StartDeleteConfirm,
    CycleFilter,
    ToggleSelected,
    ToggleHelp,
    InputChar(char),
    Backspace,
    Submit,
    Cancel,
    Quit,
    NoOp,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppCommand {
    None,
    Save,
    Quit,
}

pub fn dispatch(action: Action, state: &mut AppState) -> AppCommand {
    match state.mode {
        Mode::Normal => handle_normal_mode(action, state),
        Mode::Adding | Mode::Editing | Mode::ConfirmDelete => handle_modal_mode(action, state),
    }
}

fn handle_normal_mode(action: Action, state: &mut AppState) -> AppCommand {
    match action {
        Action::MoveUp => {
            if state.selected_index > 0 {
                state.selected_index -= 1;
            }
            AppCommand::None
        }
        Action::MoveDown => {
            let len = state.filtered_indices().len();
            if len > 0 {
                state.selected_index = (state.selected_index + 1).min(len - 1);
            }
            AppCommand::None
        }
        Action::StartAdd => {
            state.mode = Mode::Adding;
            state.input_buffer.clear();
            state.status_message = None;
            AppCommand::None
        }
        Action::StartEdit => {
            if let Some(idx) = state.selected_todo_index() {
                state.mode = Mode::Editing;
                state.input_buffer = state.todos[idx].title.clone();
                state.status_message = None;
            } else {
                state.status_message = Some("No todo selected to edit".to_string());
            }
            AppCommand::None
        }
        Action::StartDeleteConfirm => {
            if state.selected_todo_index().is_some() {
                state.mode = Mode::ConfirmDelete;
                state.status_message = None;
            } else {
                state.status_message = Some("No todo selected to delete".to_string());
            }
            AppCommand::None
        }
        Action::CycleFilter => {
            let selected_id = state.selected_todo_id();
            state.filter = state.filter.next();
            state.set_selection_by_id_or_clamp(selected_id);
            AppCommand::None
        }
        Action::ToggleSelected => {
            if let Some(idx) = state.selected_todo_index() {
                let selected_id = state.todos[idx].id;
                state.todos[idx].completed = !state.todos[idx].completed;
                if state.todos[idx].completed {
                    state.todos[idx].completed_at = Some(Utc::now().date_naive());
                } else {
                    state.todos[idx].completed_at = None;
                }
                state.set_selection_by_id_or_clamp(Some(selected_id));
                state.status_message = None;
                AppCommand::Save
            } else {
                state.status_message = Some("No todo selected to toggle".to_string());
                AppCommand::None
            }
        }
        Action::ToggleHelp => {
            state.show_help = !state.show_help;
            AppCommand::None
        }
        Action::Quit => AppCommand::Quit,
        Action::NoOp
        | Action::InputChar(_)
        | Action::Backspace
        | Action::Submit
        | Action::Cancel => AppCommand::None,
    }
}

fn handle_modal_mode(action: Action, state: &mut AppState) -> AppCommand {
    match state.mode {
        Mode::Adding | Mode::Editing => handle_text_input_modal(action, state),
        Mode::ConfirmDelete => handle_confirm_delete_modal(action, state),
        Mode::Normal => AppCommand::None,
    }
}

fn handle_text_input_modal(action: Action, state: &mut AppState) -> AppCommand {
    match action {
        Action::InputChar(c) => {
            state.input_buffer.push(c);
            AppCommand::None
        }
        Action::Backspace => {
            state.input_buffer.pop();
            AppCommand::None
        }
        Action::Cancel => {
            state.mode = Mode::Normal;
            state.input_buffer.clear();
            AppCommand::None
        }
        Action::Submit => {
            let title = state.input_buffer.trim().to_string();
            if title.is_empty() {
                state.status_message = Some("Title cannot be empty".to_string());
                return AppCommand::None;
            }

            let selected_id = state.selected_todo_id();
            match state.mode {
                Mode::Adding => {
                    let new_todo = Todo::new(title);
                    let new_id = new_todo.id;
                    state.todos.insert(0, new_todo);
                    state.mode = Mode::Normal;
                    state.input_buffer.clear();
                    state.status_message = None;
                    state.set_selection_by_id_or_clamp(Some(new_id));
                    AppCommand::Save
                }
                Mode::Editing => {
                    if let Some(idx) = state.selected_todo_index() {
                        state.todos[idx].title = title;
                        state.mode = Mode::Normal;
                        state.input_buffer.clear();
                        state.status_message = None;
                        state.set_selection_by_id_or_clamp(selected_id);
                        AppCommand::Save
                    } else {
                        state.mode = Mode::Normal;
                        state.input_buffer.clear();
                        state.status_message = Some("No todo selected to edit".to_string());
                        AppCommand::None
                    }
                }
                _ => AppCommand::None,
            }
        }
        _ => AppCommand::None,
    }
}

fn handle_confirm_delete_modal(action: Action, state: &mut AppState) -> AppCommand {
    match action {
        Action::Submit => {
            if let Some(idx) = state.selected_todo_index() {
                state.todos.remove(idx);
                state.mode = Mode::Normal;
                state.input_buffer.clear();
                state.status_message = None;
                state.normalize_selection();
                AppCommand::Save
            } else {
                state.mode = Mode::Normal;
                state.status_message = Some("No todo selected to delete".to_string());
                AppCommand::None
            }
        }
        Action::Cancel => {
            state.mode = Mode::Normal;
            AppCommand::None
        }
        _ => AppCommand::None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn build_state_with_two() -> AppState {
        let mut state = AppState::new(vec![Todo::new("one".into()), Todo::new("two".into())]);
        state.filter = Filter::All;
        state
    }

    #[test]
    fn add_rejects_empty_title() {
        let mut state = AppState::new(vec![]);
        dispatch(Action::StartAdd, &mut state);
        let cmd = dispatch(Action::Submit, &mut state);
        assert_eq!(cmd, AppCommand::None);
        assert!(state.todos.is_empty());
        assert_eq!(state.mode, Mode::Adding);
        assert_eq!(
            state.status_message.as_deref(),
            Some("Title cannot be empty")
        );
    }

    #[test]
    fn add_accepts_non_empty_title() {
        let mut state = AppState::new(vec![Todo::new("existing".into())]);
        dispatch(Action::StartAdd, &mut state);
        for c in "task".chars() {
            dispatch(Action::InputChar(c), &mut state);
        }
        let cmd = dispatch(Action::Submit, &mut state);
        assert_eq!(cmd, AppCommand::Save);
        assert_eq!(state.todos.len(), 2);
        assert_eq!(state.todos[0].title, "task");
        assert!(!state.todos[0].completed);
        assert_eq!(state.todos[1].title, "existing");
        assert_eq!(state.selected_index, 0);
        assert_eq!(state.mode, Mode::Normal);
    }

    #[test]
    fn edit_selected_updates_title_only() {
        let mut state = build_state_with_two();
        let original = state.todos[0].clone();

        dispatch(Action::StartEdit, &mut state);
        state.input_buffer = "renamed".to_string();
        let cmd = dispatch(Action::Submit, &mut state);

        assert_eq!(cmd, AppCommand::Save);
        assert_eq!(state.todos[0].title, "renamed");
        assert_eq!(state.todos[0].id, original.id);
        assert_eq!(state.todos[0].completed, original.completed);
    }

    #[test]
    fn toggle_flips_completion_state() {
        let mut state = build_state_with_two();
        assert!(!state.todos[0].completed);
        assert!(state.todos[0].completed_at.is_none());
        let cmd = dispatch(Action::ToggleSelected, &mut state);
        assert_eq!(cmd, AppCommand::Save);
        assert!(state.todos[0].completed);
        assert!(state.todos[0].completed_at.is_some());

        let cmd = dispatch(Action::ToggleSelected, &mut state);
        assert_eq!(cmd, AppCommand::Save);
        assert!(!state.todos[0].completed);
        assert!(state.todos[0].completed_at.is_none());
    }

    #[test]
    fn delete_updates_selection_safely() {
        let mut state = build_state_with_two();
        dispatch(Action::MoveDown, &mut state);
        assert_eq!(state.selected_index, 1);

        dispatch(Action::StartDeleteConfirm, &mut state);
        let cmd = dispatch(Action::Submit, &mut state);

        assert_eq!(cmd, AppCommand::Save);
        assert_eq!(state.todos.len(), 1);
        assert_eq!(state.selected_index, 0);
    }

    #[test]
    fn filter_and_clamp_selection() {
        let mut state = AppState::new(vec![Todo::new("a".into()), Todo::new("b".into())]);
        state.todos[1].completed = true;
        dispatch(Action::MoveDown, &mut state);
        assert_eq!(state.selected_index, 1);

        dispatch(Action::CycleFilter, &mut state);
        assert_eq!(state.filter, Filter::Active);
        assert_eq!(state.filtered_indices().len(), 1);
        assert_eq!(state.selected_index, 0);
    }
}
