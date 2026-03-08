use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

use crate::{app::Action, models::Mode};

pub fn map_key_to_action(mode: Mode, key: KeyEvent) -> Action {
    if key.kind != KeyEventKind::Press {
        return Action::NoOp;
    }

    match mode {
        Mode::Normal => map_normal_mode(key),
        Mode::AddingTodo | Mode::EditingTodo | Mode::AddingGroup | Mode::EditingGroup => {
            map_text_input_mode(key)
        }
        Mode::ConfirmDeleteTodo | Mode::ConfirmDeleteGroup => map_confirm_delete_mode(key),
    }
}

fn map_normal_mode(key: KeyEvent) -> Action {
    match key.code {
        KeyCode::Char('q') => Action::Quit,
        KeyCode::Char('j') | KeyCode::Down => Action::MoveDown,
        KeyCode::Char('k') | KeyCode::Up => Action::MoveUp,
        KeyCode::Char('h') | KeyCode::Left => Action::FocusLeft,
        KeyCode::Char('l') | KeyCode::Right => Action::FocusRight,
        KeyCode::Char('a') => Action::StartAdd,
        KeyCode::Char('e') => Action::StartEdit,
        KeyCode::Char('x') | KeyCode::Enter => Action::ToggleSelected,
        KeyCode::Char('d') => Action::StartDeleteConfirm,
        KeyCode::Char('f') => Action::CycleFilter,
        KeyCode::Char('.') => Action::ToggleHelp,
        _ => Action::NoOp,
    }
}

fn map_text_input_mode(key: KeyEvent) -> Action {
    match key.code {
        KeyCode::Esc => Action::Cancel,
        KeyCode::Enter => Action::Submit,
        KeyCode::Backspace => Action::Backspace,
        KeyCode::Char(c) => {
            if key.modifiers.contains(KeyModifiers::CONTROL) {
                Action::NoOp
            } else {
                Action::InputChar(c)
            }
        }
        _ => Action::NoOp,
    }
}

fn map_confirm_delete_mode(key: KeyEvent) -> Action {
    match key.code {
        KeyCode::Esc => Action::Cancel,
        KeyCode::Enter => Action::Submit,
        _ => Action::NoOp,
    }
}
