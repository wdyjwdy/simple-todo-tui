use std::path::Path;

use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, Cell, Clear, Paragraph, Row, Table, TableState, Wrap},
};

use crate::{app::AppState, models::Mode};

pub fn render(frame: &mut Frame<'_>, state: &AppState, data_path: &Path) {
    if state.show_help {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(3),
                Constraint::Length(3),
            ])
            .split(frame.area());

        render_header(frame, state, chunks[0]);
        render_todo_list(frame, state, chunks[1]);
        render_footer(frame, state, data_path, chunks[2]);
    } else {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(3)])
            .split(frame.area());

        render_header(frame, state, chunks[0]);
        render_todo_list(frame, state, chunks[1]);
    }
    render_modal(frame, state);
}

fn render_header(frame: &mut Frame<'_>, state: &AppState, area: Rect) {
    let (total, active, completed) = state.counts();
    let header = format!(
        "Filter: {}  |  Total: {} Active: {} Done: {}",
        state.filter.label(),
        total,
        active,
        completed
    );

    let widget = Paragraph::new(header)
        .block(Block::default().title("Overview").borders(Borders::ALL))
        .alignment(Alignment::Left);

    frame.render_widget(widget, area);
}

fn render_todo_list(frame: &mut Frame<'_>, state: &AppState, area: Rect) {
    let visible = state.filtered_indices();

    if visible.is_empty() {
        let text = match state.filter.label() {
            "All" => "No todos yet. Press 'a' to add one.",
            _ => "No todos in this filter.",
        };
        let empty = Paragraph::new(text)
            .block(Block::default().title("Todos").borders(Borders::ALL))
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true });
        frame.render_widget(empty, area);
        return;
    }

    let rows: Vec<Row> = visible
        .iter()
        .map(|&idx| {
            let todo = &state.todos[idx];
            let prefix = if todo.completed { "[x]" } else { "[ ]" };
            let content = format!("{} {}", prefix, todo.title);
            let date = if todo.completed {
                todo.completed_at
                    .unwrap_or_else(|| todo.created_at.date_naive())
                    .format("%Y-%m-%d")
                    .to_string()
            } else {
                String::new()
            };

            let mut row = Row::new(vec![
                Cell::from(content),
                Cell::from(format!("{:>10}", date)),
            ]);
            if todo.completed {
                row = row.style(Style::default().fg(Color::Green));
            }
            row
        })
        .collect();

    let table = Table::new(rows, [Constraint::Min(1), Constraint::Length(10)])
        .column_spacing(1)
        .block(Block::default().title("Todos").borders(Borders::ALL))
        .row_highlight_style(Style::default().bg(Color::Rgb(0xDD, 0xEE, 0xFF)))
        .highlight_symbol("");

    let mut table_state = TableState::default();
    table_state.select(Some(
        state.selected_index.min(visible.len().saturating_sub(1)),
    ));
    frame.render_stateful_widget(table, area, &mut table_state);
}

fn render_footer(frame: &mut Frame<'_>, state: &AppState, data_path: &Path, area: Rect) {
    let mut lines = vec![
        "j/k: move | a: add | e: edit | x: toggle | d: delete | f: filter | .: help | q: quit"
            .to_string(),
        format!("Data: {}", data_path.display()),
    ];

    if let Some(msg) = &state.status_message {
        lines.push(format!("Status: {}", msg));
    }

    let footer = Paragraph::new(lines.join("\n"))
        .block(Block::default().title("Help").borders(Borders::ALL))
        .wrap(Wrap { trim: true });

    frame.render_widget(footer, area);
}

fn render_modal(frame: &mut Frame<'_>, state: &AppState) {
    match state.mode {
        Mode::Normal => {}
        Mode::Adding | Mode::Editing => {
            let area = centered_rect(70, 20, frame.area());
            frame.render_widget(Clear, area);
            let title = if state.mode == Mode::Adding {
                "Add Todo"
            } else {
                "Edit Todo"
            };
            let modal = Paragraph::new(state.input_buffer.clone()).block(
                Block::default()
                    .title(format!("{} (Enter to save, Esc to cancel)", title))
                    .borders(Borders::ALL),
            );
            frame.render_widget(modal, area);
        }
        Mode::ConfirmDelete => {
            let area = centered_rect(60, 20, frame.area());
            frame.render_widget(Clear, area);
            let modal = Paragraph::new("Delete selected todo? Enter = confirm, Esc = cancel")
                .alignment(Alignment::Center)
                .block(
                    Block::default()
                        .title("Confirm Delete")
                        .borders(Borders::ALL),
                );
            frame.render_widget(modal, area);
        }
    }
}

fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(area);

    let horizontal = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(vertical[1]);

    horizontal[1]
}
