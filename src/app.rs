use crate::models::{Filter, Group, Mode, Todo};
use chrono::Utc;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PanelFocus {
    Groups,
    Todos,
}

#[derive(Debug)]
pub struct AppState {
    pub groups: Vec<Group>,
    pub todos: Vec<Todo>,
    pub selected_group_index: usize,
    pub selected_todo_index: usize,
    pub focused_panel: PanelFocus,
    pub filter: Filter,
    pub show_help: bool,
    pub mode: Mode,
    pub input_buffer: String,
    pub input_cursor: usize,
    pub status_message: Option<String>,
}

impl AppState {
    pub fn new(mut groups: Vec<Group>, mut todos: Vec<Todo>) -> Self {
        for todo in &mut todos {
            if todo.completed && todo.completed_at.is_none() {
                todo.completed_at = Some(todo.created_at.date_naive());
            }
        }

        if groups.is_empty() {
            groups.push(Group::new("Inbox".to_string()));
        }

        let group_ids: Vec<Uuid> = groups.iter().map(|g| g.id).collect();
        todos.retain(|todo| group_ids.contains(&todo.group_id));

        let mut state = Self {
            groups,
            todos,
            selected_group_index: 0,
            selected_todo_index: 0,
            focused_panel: PanelFocus::Todos,
            filter: Filter::All,
            show_help: true,
            mode: Mode::Normal,
            input_buffer: String::new(),
            input_cursor: 0,
            status_message: None,
        };
        state.normalize_selection();
        state
    }

    pub fn selected_group_id(&self) -> Option<Uuid> {
        self.groups.get(self.selected_group_index).map(|g| g.id)
    }

    pub fn filtered_todo_indices(&self) -> Vec<usize> {
        let Some(group_id) = self.selected_group_id() else {
            return vec![];
        };

        self.todos
            .iter()
            .enumerate()
            .filter_map(|(idx, todo)| {
                let in_group = todo.group_id == group_id;
                let matches_filter = match self.filter {
                    Filter::All => true,
                    Filter::Open => !todo.completed,
                    Filter::Done => todo.completed,
                };
                (in_group && matches_filter).then_some(idx)
            })
            .collect()
    }

    pub fn selected_todo_index(&self) -> Option<usize> {
        let visible = self.filtered_todo_indices();
        visible.get(self.selected_todo_index).copied()
    }

    pub fn selected_todo_id(&self) -> Option<Uuid> {
        self.selected_todo_index().map(|idx| self.todos[idx].id)
    }

    pub fn group_progress(&self, group_id: Uuid) -> (usize, usize) {
        let total = self.todos.iter().filter(|t| t.group_id == group_id).count();
        let completed = self
            .todos
            .iter()
            .filter(|t| t.group_id == group_id && t.completed)
            .count();
        (completed, total)
    }

    fn set_group_selection_by_id_or_clamp(&mut self, id: Option<Uuid>) {
        if self.groups.is_empty() {
            self.selected_group_index = 0;
            return;
        }

        if let Some(id) = id
            && let Some(pos) = self.groups.iter().position(|g| g.id == id)
        {
            self.selected_group_index = pos;
            return;
        }

        self.selected_group_index = self
            .selected_group_index
            .min(self.groups.len().saturating_sub(1));
    }

    fn set_todo_selection_by_id_or_clamp(&mut self, id: Option<Uuid>) {
        let visible = self.filtered_todo_indices();
        if visible.is_empty() {
            self.selected_todo_index = 0;
            return;
        }

        if let Some(id) = id
            && let Some(pos) = visible.iter().position(|&idx| self.todos[idx].id == id)
        {
            self.selected_todo_index = pos;
            return;
        }

        self.selected_todo_index = self
            .selected_todo_index
            .min(visible.len().saturating_sub(1));
    }

    pub fn normalize_selection(&mut self) {
        let selected_group_id = self.selected_group_id();
        self.set_group_selection_by_id_or_clamp(selected_group_id);
        self.set_todo_selection_by_id_or_clamp(self.selected_todo_id());
    }

    fn input_len(&self) -> usize {
        self.input_buffer.chars().count()
    }

    fn input_byte_index_for_char_index(&self, char_index: usize) -> usize {
        if char_index >= self.input_len() {
            return self.input_buffer.len();
        }

        self.input_buffer
            .char_indices()
            .nth(char_index)
            .map(|(idx, _)| idx)
            .unwrap_or(self.input_buffer.len())
    }

    fn insert_input_char(&mut self, c: char) {
        let byte_idx = self.input_byte_index_for_char_index(self.input_cursor);
        self.input_buffer.insert(byte_idx, c);
        self.input_cursor += 1;
    }

    fn backspace_input_char(&mut self) {
        if self.input_cursor == 0 {
            return;
        }

        let end = self.input_byte_index_for_char_index(self.input_cursor);
        let start = self.input_byte_index_for_char_index(self.input_cursor - 1);
        self.input_buffer.replace_range(start..end, "");
        self.input_cursor -= 1;
    }

    fn move_input_cursor_left(&mut self) {
        self.input_cursor = self.input_cursor.saturating_sub(1);
    }

    fn move_input_cursor_right(&mut self) {
        self.input_cursor = (self.input_cursor + 1).min(self.input_len());
    }

    fn move_input_cursor_home(&mut self) {
        self.input_cursor = 0;
    }

    fn move_input_cursor_end(&mut self) {
        self.input_cursor = self.input_len();
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    MoveUp,
    MoveDown,
    FocusLeft,
    FocusRight,
    StartAdd,
    StartEdit,
    StartDeleteConfirm,
    CycleFilter,
    ToggleSelected,
    ToggleHelp,
    InputChar(char),
    Backspace,
    MoveCursorLeft,
    MoveCursorRight,
    MoveCursorHome,
    MoveCursorEnd,
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
        Mode::AddingTodo
        | Mode::EditingTodo
        | Mode::ConfirmDeleteTodo
        | Mode::AddingGroup
        | Mode::EditingGroup
        | Mode::ConfirmDeleteGroup => handle_modal_mode(action, state),
    }
}

fn handle_normal_mode(action: Action, state: &mut AppState) -> AppCommand {
    match action {
        Action::MoveUp => {
            match state.focused_panel {
                PanelFocus::Groups => {
                    if state.selected_group_index > 0 {
                        state.selected_group_index -= 1;
                        state.set_todo_selection_by_id_or_clamp(None);
                    }
                }
                PanelFocus::Todos => {
                    if state.selected_todo_index > 0 {
                        state.selected_todo_index -= 1;
                    }
                }
            }
            AppCommand::None
        }
        Action::MoveDown => {
            match state.focused_panel {
                PanelFocus::Groups => {
                    let len = state.groups.len();
                    if len > 0 {
                        state.selected_group_index = (state.selected_group_index + 1).min(len - 1);
                        state.set_todo_selection_by_id_or_clamp(None);
                    }
                }
                PanelFocus::Todos => {
                    let len = state.filtered_todo_indices().len();
                    if len > 0 {
                        state.selected_todo_index = (state.selected_todo_index + 1).min(len - 1);
                    }
                }
            }
            AppCommand::None
        }
        Action::FocusLeft => {
            state.focused_panel = PanelFocus::Groups;
            AppCommand::None
        }
        Action::FocusRight => {
            state.focused_panel = PanelFocus::Todos;
            AppCommand::None
        }
        Action::StartAdd => {
            state.input_buffer.clear();
            state.input_cursor = 0;
            state.status_message = None;
            state.mode = match state.focused_panel {
                PanelFocus::Groups => Mode::AddingGroup,
                PanelFocus::Todos => Mode::AddingTodo,
            };
            AppCommand::None
        }
        Action::StartEdit => {
            match state.focused_panel {
                PanelFocus::Groups => {
                    if let Some(group) = state.groups.get(state.selected_group_index) {
                        state.mode = Mode::EditingGroup;
                        state.input_buffer = group.name.clone();
                        state.input_cursor = state.input_buffer.chars().count();
                        state.status_message = None;
                    } else {
                        state.status_message = Some("No group selected to edit".to_string());
                    }
                }
                PanelFocus::Todos => {
                    if let Some(idx) = state.selected_todo_index() {
                        state.mode = Mode::EditingTodo;
                        state.input_buffer = state.todos[idx].title.clone();
                        state.input_cursor = state.input_buffer.chars().count();
                        state.status_message = None;
                    } else {
                        state.status_message = Some("No todo selected to edit".to_string());
                    }
                }
            }
            AppCommand::None
        }
        Action::StartDeleteConfirm => {
            match state.focused_panel {
                PanelFocus::Groups => {
                    if state.groups.len() <= 1 {
                        state.status_message = Some("Cannot delete the last group".to_string());
                    } else if state.groups.get(state.selected_group_index).is_some() {
                        state.mode = Mode::ConfirmDeleteGroup;
                        state.status_message = None;
                    } else {
                        state.status_message = Some("No group selected to delete".to_string());
                    }
                }
                PanelFocus::Todos => {
                    if state.selected_todo_index().is_some() {
                        state.mode = Mode::ConfirmDeleteTodo;
                        state.status_message = None;
                    } else {
                        state.status_message = Some("No todo selected to delete".to_string());
                    }
                }
            }
            AppCommand::None
        }
        Action::CycleFilter => {
            if state.focused_panel == PanelFocus::Todos {
                let selected_id = state.selected_todo_id();
                state.filter = state.filter.next();
                state.set_todo_selection_by_id_or_clamp(selected_id);
            }
            AppCommand::None
        }
        Action::ToggleSelected => {
            if state.focused_panel == PanelFocus::Todos {
                if let Some(idx) = state.selected_todo_index() {
                    let selected_id = state.todos[idx].id;
                    state.todos[idx].completed = !state.todos[idx].completed;
                    if state.todos[idx].completed {
                        state.todos[idx].completed_at = Some(Utc::now().date_naive());
                    } else {
                        state.todos[idx].completed_at = None;
                    }
                    state.set_todo_selection_by_id_or_clamp(Some(selected_id));
                    state.status_message = None;
                    return AppCommand::Save;
                }
                state.status_message = Some("No todo selected to toggle".to_string());
            }
            AppCommand::None
        }
        Action::ToggleHelp => {
            state.show_help = !state.show_help;
            AppCommand::None
        }
        Action::Quit => AppCommand::Quit,
        Action::NoOp
        | Action::InputChar(_)
        | Action::Backspace
        | Action::MoveCursorLeft
        | Action::MoveCursorRight
        | Action::MoveCursorHome
        | Action::MoveCursorEnd
        | Action::Submit
        | Action::Cancel => AppCommand::None,
    }
}

fn handle_modal_mode(action: Action, state: &mut AppState) -> AppCommand {
    match state.mode {
        Mode::AddingTodo | Mode::EditingTodo | Mode::AddingGroup | Mode::EditingGroup => {
            handle_text_input_modal(action, state)
        }
        Mode::ConfirmDeleteTodo | Mode::ConfirmDeleteGroup => {
            handle_confirm_delete_modal(action, state)
        }
        Mode::Normal => AppCommand::None,
    }
}

fn handle_text_input_modal(action: Action, state: &mut AppState) -> AppCommand {
    match action {
        Action::InputChar(c) => {
            state.insert_input_char(c);
            AppCommand::None
        }
        Action::Backspace => {
            state.backspace_input_char();
            AppCommand::None
        }
        Action::MoveCursorLeft => {
            state.move_input_cursor_left();
            AppCommand::None
        }
        Action::MoveCursorRight => {
            state.move_input_cursor_right();
            AppCommand::None
        }
        Action::MoveCursorHome => {
            state.move_input_cursor_home();
            AppCommand::None
        }
        Action::MoveCursorEnd => {
            state.move_input_cursor_end();
            AppCommand::None
        }
        Action::Cancel => {
            state.mode = Mode::Normal;
            state.input_buffer.clear();
            state.input_cursor = 0;
            AppCommand::None
        }
        Action::Submit => {
            let value = state.input_buffer.trim().to_string();
            if value.is_empty() {
                state.status_message = Some("Title cannot be empty".to_string());
                return AppCommand::None;
            }

            match state.mode {
                Mode::AddingGroup => {
                    let group = Group::new(value);
                    let group_id = group.id;
                    state.groups.insert(0, group);
                    state.mode = Mode::Normal;
                    state.input_buffer.clear();
                    state.input_cursor = 0;
                    state.status_message = None;
                    state.set_group_selection_by_id_or_clamp(Some(group_id));
                    state.set_todo_selection_by_id_or_clamp(None);
                    AppCommand::Save
                }
                Mode::EditingGroup => {
                    if let Some(group) = state.groups.get_mut(state.selected_group_index) {
                        group.name = value;
                        state.mode = Mode::Normal;
                        state.input_buffer.clear();
                        state.input_cursor = 0;
                        state.status_message = None;
                        AppCommand::Save
                    } else {
                        state.mode = Mode::Normal;
                        state.input_buffer.clear();
                        state.input_cursor = 0;
                        state.status_message = Some("No group selected to edit".to_string());
                        AppCommand::None
                    }
                }
                Mode::AddingTodo => {
                    if let Some(group_id) = state.selected_group_id() {
                        let todo = Todo::new(value, group_id);
                        let todo_id = todo.id;
                        state.todos.insert(0, todo);
                        state.mode = Mode::Normal;
                        state.input_buffer.clear();
                        state.input_cursor = 0;
                        state.status_message = None;
                        state.set_todo_selection_by_id_or_clamp(Some(todo_id));
                        AppCommand::Save
                    } else {
                        state.mode = Mode::Normal;
                        state.input_buffer.clear();
                        state.input_cursor = 0;
                        state.status_message = Some("No group selected for new todo".to_string());
                        AppCommand::None
                    }
                }
                Mode::EditingTodo => {
                    if let Some(idx) = state.selected_todo_index() {
                        state.todos[idx].title = value;
                        state.mode = Mode::Normal;
                        state.input_buffer.clear();
                        state.input_cursor = 0;
                        state.status_message = None;
                        AppCommand::Save
                    } else {
                        state.mode = Mode::Normal;
                        state.input_buffer.clear();
                        state.input_cursor = 0;
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
        Action::Submit => match state.mode {
            Mode::ConfirmDeleteGroup => {
                if state.groups.len() <= 1 {
                    state.mode = Mode::Normal;
                    state.status_message = Some("Cannot delete the last group".to_string());
                    return AppCommand::None;
                }

                if let Some(group) = state.groups.get(state.selected_group_index).cloned() {
                    state.groups.remove(state.selected_group_index);
                    state.todos.retain(|todo| todo.group_id != group.id);
                    state.mode = Mode::Normal;
                    state.input_buffer.clear();
                    state.input_cursor = 0;
                    state.status_message = None;
                    state.normalize_selection();
                    AppCommand::Save
                } else {
                    state.mode = Mode::Normal;
                    state.status_message = Some("No group selected to delete".to_string());
                    AppCommand::None
                }
            }
            Mode::ConfirmDeleteTodo => {
                if let Some(idx) = state.selected_todo_index() {
                    state.todos.remove(idx);
                    state.mode = Mode::Normal;
                    state.input_buffer.clear();
                    state.input_cursor = 0;
                    state.status_message = None;
                    state.set_todo_selection_by_id_or_clamp(None);
                    AppCommand::Save
                } else {
                    state.mode = Mode::Normal;
                    state.status_message = Some("No todo selected to delete".to_string());
                    AppCommand::None
                }
            }
            _ => AppCommand::None,
        },
        Action::Cancel => {
            state.mode = Mode::Normal;
            state.input_cursor = 0;
            AppCommand::None
        }
        _ => AppCommand::None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn build_state() -> AppState {
        let g1 = Group::new("g1".into());
        let g2 = Group::new("g2".into());
        let t1 = Todo::new("one".into(), g1.id);
        let t2 = Todo::new("two".into(), g1.id);
        let t3 = Todo::new("three".into(), g2.id);
        AppState::new(vec![g1, g2], vec![t1, t2, t3])
    }

    #[test]
    fn add_todo_in_todo_panel() {
        let mut state = build_state();
        state.focused_panel = PanelFocus::Todos;

        dispatch(Action::StartAdd, &mut state);
        for c in "task".chars() {
            dispatch(Action::InputChar(c), &mut state);
        }

        let cmd = dispatch(Action::Submit, &mut state);
        assert_eq!(cmd, AppCommand::Save);
        assert_eq!(state.mode, Mode::Normal);
        assert_eq!(state.todos[0].title, "task");
        assert_eq!(state.todos[0].group_id, state.groups[0].id);
    }

    #[test]
    fn add_group_in_group_panel() {
        let mut state = build_state();
        state.focused_panel = PanelFocus::Groups;

        dispatch(Action::StartAdd, &mut state);
        for c in "new group".chars() {
            dispatch(Action::InputChar(c), &mut state);
        }

        let cmd = dispatch(Action::Submit, &mut state);
        assert_eq!(cmd, AppCommand::Save);
        assert_eq!(state.groups[0].name, "new group");
        assert_eq!(state.selected_group_index, 0);
    }

    #[test]
    fn move_group_selection_updates_visible_todos() {
        let mut state = build_state();
        state.focused_panel = PanelFocus::Groups;
        assert_eq!(state.filtered_todo_indices().len(), 2);

        dispatch(Action::MoveDown, &mut state);

        assert_eq!(state.selected_group_index, 1);
        assert_eq!(state.filtered_todo_indices().len(), 1);
    }

    #[test]
    fn cannot_delete_last_group() {
        let group = Group::new("only".into());
        let mut state = AppState::new(vec![group], vec![]);
        state.focused_panel = PanelFocus::Groups;

        let cmd = dispatch(Action::StartDeleteConfirm, &mut state);

        assert_eq!(cmd, AppCommand::None);
        assert_eq!(state.mode, Mode::Normal);
        assert_eq!(
            state.status_message.as_deref(),
            Some("Cannot delete the last group")
        );
    }

    #[test]
    fn deleting_group_cascades_todos() {
        let mut state = build_state();
        state.focused_panel = PanelFocus::Groups;

        dispatch(Action::StartDeleteConfirm, &mut state);
        let cmd = dispatch(Action::Submit, &mut state);

        assert_eq!(cmd, AppCommand::Save);
        assert_eq!(state.groups.len(), 1);
        assert_eq!(state.todos.len(), 1);
    }

    #[test]
    fn filter_applies_only_in_todo_panel() {
        let mut state = build_state();
        state.todos[0].completed = true;

        state.focused_panel = PanelFocus::Groups;
        dispatch(Action::CycleFilter, &mut state);
        assert_eq!(state.filter, Filter::All);

        state.focused_panel = PanelFocus::Todos;
        dispatch(Action::CycleFilter, &mut state);
        assert_eq!(state.filter, Filter::Open);
    }

    #[test]
    fn text_input_cursor_move_and_insert() {
        let mut state = build_state();
        state.focused_panel = PanelFocus::Todos;
        dispatch(Action::StartAdd, &mut state);

        dispatch(Action::InputChar('a'), &mut state);
        dispatch(Action::InputChar('c'), &mut state);
        dispatch(Action::MoveCursorLeft, &mut state);
        dispatch(Action::InputChar('b'), &mut state);

        assert_eq!(state.input_buffer, "abc");
        assert_eq!(state.input_cursor, 2);
    }

    #[test]
    fn text_input_backspace_at_cursor() {
        let mut state = build_state();
        state.focused_panel = PanelFocus::Todos;
        dispatch(Action::StartAdd, &mut state);

        dispatch(Action::InputChar('a'), &mut state);
        dispatch(Action::InputChar('b'), &mut state);
        dispatch(Action::InputChar('c'), &mut state);
        dispatch(Action::MoveCursorLeft, &mut state);
        dispatch(Action::Backspace, &mut state);

        assert_eq!(state.input_buffer, "ac");
        assert_eq!(state.input_cursor, 1);
    }
}
