use std::path::Path;

use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::Line,
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph, Wrap},
};

use crate::{app::AppState, models::Mode};

pub fn render(frame: &mut Frame<'_>, state: &AppState, data_path: &Path) {
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

    let items: Vec<ListItem> = visible
        .iter()
        .map(|&idx| {
            let todo = &state.todos[idx];
            let prefix = if todo.completed { "[x]" } else { "[ ]" };
            let content = if todo.completed {
                let completed_at = todo
                    .completed_at
                    .unwrap_or_else(|| todo.created_at.date_naive())
                    .format("%Y-%m-%d")
                    .to_string();
                format!("{} {} {}", prefix, todo.title, completed_at)
            } else {
                format!("{} {}", prefix, todo.title)
            };
            let mut item = ListItem::new(Line::from(content));
            if todo.completed {
                item = item.style(Style::default().fg(Color::Green));
            }
            item
        })
        .collect();

    let list = List::new(items)
        .block(Block::default().title("Todos").borders(Borders::ALL))
        .highlight_style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("> ");

    let mut list_state = ListState::default();
    list_state.select(Some(
        state.selected_index.min(visible.len().saturating_sub(1)),
    ));
    frame.render_stateful_widget(list, area, &mut list_state);
}

fn render_footer(frame: &mut Frame<'_>, state: &AppState, data_path: &Path, area: Rect) {
    let base_help = "j/k: move | a: add | e: edit | x: toggle | d: delete | f: filter | q: quit";
    let mut lines = vec![
        base_help.to_string(),
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
