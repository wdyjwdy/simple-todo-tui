use std::{
    io::{self, Stdout},
    path::PathBuf,
    time::Duration,
};

use anyhow::Result;
use crossterm::{
    cursor::{DisableBlinking, EnableBlinking},
    event::{self, Event},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};

use todo::app::{AppCommand, AppState, dispatch};
use todo::models::Filter;
use todo::{input, storage, ui};

fn main() -> Result<()> {
    let data_path = storage::default_data_path();

    let (data, startup_status) = match storage::load_data(&data_path) {
        Ok(data) => (data, None),
        Err(err) => (
            storage::AppData {
                groups: vec![],
                todos: vec![],
                show_help: true,
                todo_filter: Filter::All,
                group_filter: Filter::All,
            },
            Some(format!(
                "Failed to load todo file (starting empty): {}",
                err
            )),
        ),
    };

    let mut app_state = AppState::new(data.groups, data.todos);
    app_state.show_help = data.show_help;
    app_state.todo_filter = data.todo_filter;
    app_state.group_filter = data.group_filter;
    app_state.status_message = startup_status;

    let mut terminal = setup_terminal()?;
    let result = run_app(&mut terminal, app_state, data_path);
    restore_terminal(&mut terminal)?;
    result
}

fn run_app(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    mut state: AppState,
    data_path: PathBuf,
) -> Result<()> {
    loop {
        terminal.draw(|frame| ui::render(frame, &state, &data_path))?;

        if event::poll(Duration::from_millis(200))? {
            if let Event::Key(key) = event::read()? {
                let action = input::map_key_to_action(state.mode, key);
                let cmd = dispatch(action, &mut state);

                match cmd {
                    AppCommand::None => {}
                    AppCommand::Save => {
                        let data = storage::AppData {
                            groups: state.groups.clone(),
                            todos: state.todos.clone(),
                            show_help: state.show_help,
                            todo_filter: state.todo_filter,
                            group_filter: state.group_filter,
                        };
                        if let Err(err) = storage::save_data(&data_path, &data) {
                            state.status_message =
                                Some(format!("Save failed at {}: {}", data_path.display(), err));
                        }
                    }
                    AppCommand::Quit => {
                        let data = storage::AppData {
                            groups: state.groups.clone(),
                            todos: state.todos.clone(),
                            show_help: state.show_help,
                            todo_filter: state.todo_filter,
                            group_filter: state.group_filter,
                        };
                        if let Err(err) = storage::save_data(&data_path, &data) {
                            state.status_message =
                                Some(format!("Save failed at {}: {}", data_path.display(), err));
                            continue;
                        }
                        break;
                    }
                }
            }
        }
    }

    Ok(())
}

fn setup_terminal() -> Result<Terminal<CrosstermBackend<Stdout>>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, DisableBlinking)?;
    let backend = CrosstermBackend::new(stdout);
    let terminal = Terminal::new(backend)?;
    Ok(terminal)
}

fn restore_terminal(terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> Result<()> {
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), EnableBlinking, LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    Ok(())
}
