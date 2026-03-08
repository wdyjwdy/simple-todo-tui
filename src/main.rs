use std::{
    io::{self, Stdout},
    path::PathBuf,
    time::Duration,
};

use anyhow::Result;
use crossterm::{
    event::{self, Event},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};

use todo::app::{AppCommand, AppState, dispatch};
use todo::{input, storage, ui};

fn main() -> Result<()> {
    let data_path = storage::default_data_path();

    let (todos, startup_status) = match storage::load_todos(&data_path) {
        Ok(todos) => (todos, None),
        Err(err) => (
            vec![],
            Some(format!(
                "Failed to load todo file (starting empty): {}",
                err
            )),
        ),
    };

    let mut app_state = AppState::new(todos);
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
                        if let Err(err) = storage::save_todos(&data_path, &state.todos) {
                            state.status_message =
                                Some(format!("Save failed at {}: {}", data_path.display(), err));
                        }
                    }
                    AppCommand::Quit => {
                        if let Err(err) = storage::save_todos(&data_path, &state.todos) {
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
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let terminal = Terminal::new(backend)?;
    Ok(terminal)
}

fn restore_terminal(terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> Result<()> {
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    Ok(())
}
